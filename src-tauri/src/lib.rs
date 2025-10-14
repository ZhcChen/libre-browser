use chrono::Local;
use directories::ProjectDirs;
use flate2::{write::GzEncoder, Compression};
use once_cell::sync::OnceCell;
#[cfg(all(unix, target_os = "macos"))]
use std::os::unix::fs::PermissionsExt;
use std::{
    env,
    fs::{self, File},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use image::{ImageBuffer, Rgba};
use sha2::{Digest, Sha256};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::{thread, time::Duration};

static LOG_FILE: OnceCell<std::sync::Mutex<File>> = OnceCell::new();
static PROC_MAP: OnceCell<std::sync::Mutex<HashMap<String, Child>>> = OnceCell::new();

#[derive(Debug, Serialize, Deserialize)]
pub struct EngineInfo {
    pub version: String,
    pub installed_at: String,
}

fn logs_dir_path() -> PathBuf {
    // 优先使用系统推荐目录；失败则回退到 $HOME/.libre-browser/logs
    if let Some(proj) = ProjectDirs::from("com", "chen", "libre-browser") {
        let mut p = proj.data_local_dir().to_path_buf();
        p.push("logs");
        return p;
    }
    let mut base = env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    base.push(".libre-browser");
    base.push("logs");
    base
}

fn rotate_logs() -> std::io::Result<PathBuf> {
    let dir = logs_dir_path();
    fs::create_dir_all(&dir)?;
    let current = dir.join("current.log");
    if current.exists() {
        let ts = Local::now().format("%Y%m%d-%H%M%S");
        let archived = dir.join(format!("{}.log", ts));
        fs::rename(&current, &archived)?;
        // gzip compress
        let mut input = File::open(&archived)?;
        let mut gz_path = archived.clone();
        gz_path.set_extension("log.gz");
        let mut encoder = GzEncoder::new(File::create(&gz_path)?, Compression::default());
        std::io::copy(&mut input, &mut encoder)?;
        encoder.finish()?;
        // remove original
        let _ = fs::remove_file(&archived);
    }
    // create new
    let file = File::create(&current)?;
    LOG_FILE.set(std::sync::Mutex::new(file)).ok();
    Ok(dir)
}

fn write_log(level: &str, msg: &str) {
    if let Some(m) = LOG_FILE.get() {
        if let Ok(mut f) = m.lock() {
            let ts = Local::now().format("%Y-%m-%d %H:%M:%S");
            let _ = writeln!(f, "[{}] [{}] {}", ts, level, msg);
            let _ = f.flush();
        }
    }
}

fn proc_map() -> &'static std::sync::Mutex<HashMap<String, Child>> {
    PROC_MAP.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

fn engines_dir_path() -> PathBuf {
    if let Some(proj) = ProjectDirs::from("com", "chen", "libre-browser") {
        let mut p = proj.data_local_dir().to_path_buf();
        p.push("engines");
        return p;
    }
    let mut base = env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    base.push(".libre-browser");
    base.push("engines");
    base
}

fn engine_metadata_path() -> PathBuf {
    let mut path = engines_dir_path();
    path.push("metadata.json");
    path
}

fn load_engine_metadata() -> HashMap<String, String> {
    let path = engine_metadata_path();
    if let Ok(mut f) = File::open(&path) {
        let mut content = String::new();
        if f.read_to_string(&mut content).is_ok() {
            if let Ok(metadata) = serde_json::from_str::<HashMap<String, String>>(&content) {
                return metadata;
            }
        }
    }
    HashMap::new()
}

fn save_engine_installation_time(version: &str) {
    let mut metadata = load_engine_metadata();
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    metadata.insert(version.to_string(), timestamp.clone());

    let path = engine_metadata_path();
    if let Ok(mut f) = File::create(&path) {
        let json = serde_json::to_string_pretty(&metadata).unwrap_or_default();
        let _ = f.write_all(json.as_bytes());
        write_log(
            "INFO",
            &format!(
                "Recorded installation time for version {}: {}",
                version, timestamp
            ),
        );
    }
}

fn data_local_base() -> PathBuf {
    if let Some(proj) = ProjectDirs::from("com", "chen", "libre-browser") {
        return proj.data_local_dir().to_path_buf();
    }
    let mut base = env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    base.push(".libre-browser");
    base
}

fn profiles_dir_path() -> PathBuf {
    let mut p = data_local_base();
    p.push("profiles");
    p
}

fn profile_dir_path(label: &str) -> PathBuf {
    let mut p = profiles_dir_path();
    p.push(label);
    p
}

fn chrome_log_path(label: &str) -> PathBuf {
    profile_dir_path(label).join("chrome_debug.log")
}

fn ensure_dir(p: &PathBuf) {
    let _ = fs::create_dir_all(p);
}

fn tail_file_lines(path: &Path, max_lines: usize, max_bytes: usize) -> Option<String> {
    let mut f = File::open(path).ok()?;
    let len = f.metadata().ok()?.len();
    let start = len.saturating_sub(max_bytes as u64);
    if start > 0 {
        let _ = f.seek(SeekFrom::Start(start));
    }
    let mut buf = String::new();
    let _ = f.read_to_string(&mut buf).ok()?;
    let lines: Vec<&str> = buf.lines().collect();
    let n = lines.len();
    let take = if n > max_lines {
        &lines[n - max_lines..]
    } else {
        &lines[..]
    };
    Some(take.join("\n"))
}

fn is_probably_version(name: &str) -> bool {
    // Simple check: digits and dots only, like 114.0.5735.90
    !name.is_empty() && name.chars().all(|c| c.is_ascii_digit() || c == '.')
}

fn extract_zip_to<R: Read + std::io::Seek>(mut reader: R, dest: &Path) -> Result<(), String> {
    let mut archive =
        zip::ZipArchive::new(&mut reader).map_err(|e| format!("open zip failed: {e}"))?;
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("zip idx {i} failed: {e}"))?;
        let outpath = dest.join(file.mangled_name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).map_err(|e| format!("mkdir failed: {e}"))?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("mkdir parent failed: {e}"))?;
            }
            let mut outfile =
                File::create(&outpath).map_err(|e| format!("create file failed: {e}"))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("write file failed: {e}"))?;
        }
        // set permissions is optional
    }
    Ok(())
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn logs_dir() -> String {
    logs_dir_path().to_string_lossy().to_string()
}

#[tauri::command]
fn log_info(message: &str) {
    write_log("INFO", message);
}

#[tauri::command]
fn engines_dir() -> String {
    engines_dir_path().to_string_lossy().to_string()
}

#[tauri::command]
fn list_installed_engines() -> Vec<EngineInfo> {
    let mut out = Vec::new();
    let root = engines_dir_path();
    let metadata = load_engine_metadata();

    if let Ok(entries) = fs::read_dir(&root) {
        let mut dirs: Vec<_> = entries
            .flatten()
            .filter_map(|e| {
                let path = e.path();
                if path.is_dir()
                    && path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(is_probably_version)
                        .unwrap_or(false)
                {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        // 按版本号降序排序（最新版本靠前）
        dirs.sort_by(|a, b| {
            let a_version = a.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let b_version = b.file_name().and_then(|n| n.to_str()).unwrap_or("");
            cmp_version_desc(b_version, a_version)
        });

        for dir in dirs {
            if let Some(version) = dir.file_name().and_then(|n| n.to_str()) {
                let installed_at = metadata.get(version).cloned().unwrap_or_else(|| {
                    // 如果没有记录安装时间，使用文件夹的修改时间
                    if let Ok(md) = dir.metadata() {
                        if let Ok(modified) = md.modified() {
                            modified
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| {
                                    let dt =
                                        chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                                            .unwrap_or_default();
                                    dt.format("%Y-%m-%d %H:%M:%S").to_string()
                                })
                                .unwrap_or_else(|_| "未知".to_string())
                        } else {
                            "未知".to_string()
                        }
                    } else {
                        "未知".to_string()
                    }
                });

                out.push(EngineInfo {
                    version: version.to_string(),
                    installed_at,
                });
            }
        }
    }
    out
}

// 版本比较函数，降序排序（新版本在前）
fn cmp_version_desc(a: &str, b: &str) -> std::cmp::Ordering {
    let pa: Vec<u32> = a.split('.').filter_map(|s| s.parse().ok()).collect();
    let pb: Vec<u32> = b.split('.').filter_map(|s| s.parse().ok()).collect();

    for i in 0..std::cmp::max(pa.len(), pb.len()) {
        let va = pa.get(i).unwrap_or(&0);
        let vb = pb.get(i).unwrap_or(&0);
        match va.cmp(vb) {
            std::cmp::Ordering::Equal => continue,
            cmp => return cmp,
        }
    }
    std::cmp::Ordering::Equal
}

fn find_engine_binary() -> Option<PathBuf> {
    let root = engines_dir_path();
    if !root.exists() {
        return None;
    }
    let mut dirs: Vec<_> = fs::read_dir(&root)
        .ok()?
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            if p.is_dir() {
                Some(p)
            } else {
                None
            }
        })
        .collect();
    // sort desc by name
    dirs.sort_by(|a, b| {
        b.file_name()
            .unwrap_or_default()
            .cmp(a.file_name().unwrap_or_default())
    });
    for d in dirs {
        // Common macOS paths for Chrome-for-Testing or Chromium
        let candidates = [
            "Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing",
            "Chromium.app/Contents/MacOS/Chromium",
            "Google Chrome.app/Contents/MacOS/Google Chrome",
        ];
        // search shallow folders (like chrome-mac*/...)
        if let Ok(subs) = fs::read_dir(&d) {
            for s in subs.flatten() {
                let sp = s.path();
                for rel in &candidates {
                    let p = sp.join(rel);
                    if p.is_file() {
                        write_log(
                            "INFO",
                            &format!("find_engine_binary hit {}", p.to_string_lossy()),
                        );
                        return Some(p);
                    }
                }
            }
        }
        for rel in &candidates {
            let p = d.join(rel);
            if p.is_file() {
                write_log(
                    "INFO",
                    &format!("find_engine_binary hit {}", p.to_string_lossy()),
                );
                return Some(p);
            }
        }
    }
    write_log("INFO", "find_engine_binary not found");
    None
}

#[tauri::command]
fn download_engine(version: &str, url: &str) -> Result<String, String> {
    write_log(
        "INFO",
        &format!("download_engine start version={version} url={url}"),
    );
    let root = engines_dir_path();
    fs::create_dir_all(&root).map_err(|e| format!("create engines dir failed: {e}"))?;
    let dest_dir = root.join(version);
    if dest_dir.exists() {
        write_log(
            "INFO",
            &format!("version {version} already exists, removing"),
        );
        let _ = fs::remove_dir_all(&dest_dir);
    }
    fs::create_dir_all(&dest_dir).map_err(|e| format!("create version dir failed: {e}"))?;

    // Download to temp file
    let tmp_path = root.join(format!("{}.tmp", version));
    let mut resp = reqwest::blocking::get(url).map_err(|e| format!("http get failed: {e}"))?;
    let status = resp.status();
    if !status.is_success() {
        return Err(format!("http status: {status}"));
    }
    let mut tmpf = File::create(&tmp_path).map_err(|e| format!("create tmp failed: {e}"))?;
    resp.copy_to(&mut tmpf)
        .map_err(|e| format!("write tmp failed: {e}"))?;

    // Try unzip if it looks like zip
    let is_zip = url.to_ascii_lowercase().ends_with(".zip");
    if is_zip {
        let mut f = File::open(&tmp_path).map_err(|e| format!("open tmp failed: {e}"))?;
        extract_zip_to(&mut f, &dest_dir)?;
        let _ = fs::remove_file(&tmp_path);
    } else {
        // keep the downloaded file inside dest_dir
        let filename = url.split('/').next_back().unwrap_or("download.bin");
        let target = dest_dir.join(filename);
        fs::rename(&tmp_path, &target).map_err(|e| format!("move file failed: {e}"))?;
    }
    write_log(
        "INFO",
        &format!(
            "download_engine ok version={version} dir={}",
            dest_dir.to_string_lossy()
        ),
    );
    Ok(dest_dir.to_string_lossy().to_string())
}

#[tauri::command]
fn download_engine_archive(version: &str, url: &str) -> Result<String, String> {
    write_log(
        "INFO",
        &format!("download_engine_archive start version={version} url={url}"),
    );
    let root = engines_dir_path();
    fs::create_dir_all(&root).map_err(|e| format!("create engines dir failed: {e}"))?;
    let zip_path = root.join(format!("{}.zip", version));
    if zip_path.exists() {
        let _ = fs::remove_file(&zip_path);
    }
    let mut resp = reqwest::blocking::get(url).map_err(|e| format!("http get failed: {e}"))?;
    let status = resp.status();
    if !status.is_success() {
        return Err(format!("http status: {status}"));
    }
    let mut file = File::create(&zip_path).map_err(|e| format!("create zip failed: {e}"))?;
    resp.copy_to(&mut file)
        .map_err(|e| format!("write zip failed: {e}"))?;
    write_log(
        "INFO",
        &format!(
            "download_engine_archive ok -> {}",
            zip_path.to_string_lossy()
        ),
    );
    Ok(zip_path.to_string_lossy().to_string())
}

#[tauri::command]
fn extract_engine_archive(version: &str) -> Result<String, String> {
    write_log(
        "INFO",
        &format!("extract_engine_archive start version={version}"),
    );
    let root = engines_dir_path();
    let zip_path = root.join(format!("{}.zip", version));
    if !zip_path.exists() {
        return Err("archive not found".into());
    }
    let dest_dir = root.join(version);
    if dest_dir.exists() {
        let _ = fs::remove_dir_all(&dest_dir);
    }
    fs::create_dir_all(&dest_dir).map_err(|e| format!("create version dir failed: {e}"))?;
    let mut f = File::open(&zip_path).map_err(|e| format!("open zip failed: {e}"))?;
    extract_zip_to(&mut f, &dest_dir)?;
    // 修复 macOS 执行权限并清除隔离属性（若存在）
    fix_macos_exec_and_quarantine(&dest_dir);
    let _ = fs::remove_file(&zip_path);

    // 记录安装时间
    save_engine_installation_time(version);

    write_log(
        "INFO",
        &format!(
            "extract_engine_archive ok -> {}",
            dest_dir.to_string_lossy()
        ),
    );
    Ok(dest_dir.to_string_lossy().to_string())
}

#[cfg(target_os = "macos")]
fn path_contains_macos_exec_dir(p: &Path) -> bool {
    let mut seen_contents = false;
    for c in p.components() {
        let s = c.as_os_str().to_string_lossy();
        if s == "Contents" {
            seen_contents = true;
        }
        if seen_contents && s == "MacOS" {
            return true;
        }
    }
    false
}

// 图标生成相关函数
fn custom_icons_dir() -> PathBuf {
    let mut base = data_local_base();
    base.push("custom_icons");
    base
}

fn generate_browser_icon(name: &str, index: usize) -> Result<PathBuf, String> {
    write_log(
        "INFO",
        &format!("generate_browser_icon name={} index={}", name, index),
    );

    let icons_dir = custom_icons_dir();
    fs::create_dir_all(&icons_dir).map_err(|e| format!("create icons dir failed: {e}"))?;

    // 生成基于名称和索引的一致性哈希颜色
    let hasher_input = format!("{}-{}", name, index);
    let mut hasher = Sha256::new();
    hasher.update(hasher_input.as_bytes());
    let hash = hasher.finalize();

    // 从哈希值生成颜色
    let r = (hash[0] as u32) % 156 + 100; // 100-255 确保颜色不会太暗
    let g = (hash[1] as u32) % 156 + 100;
    let b = (hash[2] as u32) % 156 + 100;

    // 生成512x512的图标
    let size = 512u32;
    let mut img = ImageBuffer::new(size, size);

    // 绘制背景圆形
    let center_x = size / 2;
    let center_y = size / 2;
    let radius = size / 2 - 20;

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let dx = x as i32 - center_x as i32;
        let dy = y as i32 - center_y as i32;
        let distance = ((dx * dx + dy * dy) as f64).sqrt();

        if distance <= radius as f64 {
            // 在圆形内绘制
            *pixel = Rgba([r as u8, g as u8, b as u8, 255]);
        } else {
            // 透明背景
            *pixel = Rgba([0, 0, 0, 0]);
        }
    }

    // 生成图标文件名
    let icon_name = format!(
        "icon_{}_{}.png",
        name.replace(
            &[' ', '-', '/', '\\', ':', '*', '?', '"', '<', '>', '|'][..],
            "_"
        ),
        index
    );
    let icon_path = icons_dir.join(icon_name);

    // 保存图标
    img.save(&icon_path)
        .map_err(|e| format!("save icon failed: {e}"))?;

    write_log(
        "INFO",
        &format!("Generated icon: {}", icon_path.to_string_lossy()),
    );
    Ok(icon_path)
}

#[cfg(target_os = "macos")]
fn create_custom_app_bundle(
    source_app: &Path,
    target_name: &str,
    icon_path: &Path,
) -> Result<PathBuf, String> {
    write_log(
        "INFO",
        &format!(
            "create_custom_app_bundle source={:?} target={} icon={:?}",
            source_app, target_name, icon_path
        ),
    );

    let base_dir = data_local_base();
    let custom_apps_dir = base_dir.join("custom_apps");
    fs::create_dir_all(&custom_apps_dir)
        .map_err(|e| format!("create custom_apps dir failed: {e}"))?;

    let target_app_path = custom_apps_dir.join(format!("{}.app", target_name));
    if target_app_path.exists() {
        fs::remove_dir_all(&target_app_path)
            .map_err(|e| format!("remove existing app failed: {e}"))?;
    }

    // 复制原始app bundle
    let output = Command::new("cp")
        .arg("-R")
        .arg(source_app)
        .arg(&target_app_path)
        .output()
        .map_err(|e| format!("cp command failed: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "cp failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // 修改应用名称
    let info_plist_path = target_app_path.join("Contents/Info.plist");
    if info_plist_path.exists() {
        let output = Command::new("plutil")
            .args(["-replace", "CFBundleName", "-string", target_name])
            .arg(&info_plist_path)
            .output()
            .map_err(|e| format!("plutil replace CFBundleName failed: {e}"))?;

        if !output.status.success() {
            write_log(
                "WARN",
                &format!(
                    "plutil replace CFBundleName failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            );
        }

        let output = Command::new("plutil")
            .args(["-replace", "CFBundleDisplayName", "-string", target_name])
            .arg(&info_plist_path)
            .output()
            .map_err(|e| format!("plutil replace CFBundleDisplayName failed: {e}"))?;

        if !output.status.success() {
            write_log(
                "WARN",
                &format!(
                    "plutil replace CFBundleDisplayName failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            );
        }
    }

    // 替换图标 - 使用ICNS格式
    let icon_resources_dir = target_app_path.join("Contents/Resources");
    if icon_resources_dir.exists() {
        // 删除现有图标文件
        for entry in fs::read_dir(&icon_resources_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if let Some(filename) = path.file_name() {
                let filename_str = filename.to_string_lossy();
                if filename_str.starts_with("AppIcon") || filename_str.contains(".icns") {
                    let _ = fs::remove_file(&path);
                }
            }
        }

        // 将PNG转换为ICNS格式
        let icns_icon_path = icon_resources_dir.join("AppIcon.icns");
        match convert_png_to_icns(icon_path, &icns_icon_path) {
            Ok(()) => {
                write_log(
                    "INFO",
                    &format!(
                        "Successfully converted PNG to ICNS: {}",
                        icns_icon_path.to_string_lossy()
                    ),
                );

                // 更新图标引用为ICNS文件
                let output = Command::new("plutil")
                    .args(["-replace", "CFBundleIconFile", "-string", "AppIcon.icns"])
                    .arg(&info_plist_path)
                    .output()
                    .map_err(|e| format!("plutil replace CFBundleIconFile failed: {e}"))?;

                if !output.status.success() {
                    write_log(
                        "WARN",
                        &format!(
                            "plutil replace CFBundleIconFile failed: {}",
                            String::from_utf8_lossy(&output.stderr)
                        ),
                    );
                }
            }
            Err(e) => {
                write_log(
                    "WARN",
                    &format!("Failed to convert PNG to ICNS: {}, falling back to PNG", e),
                );

                // 回退到PNG格式
                let target_icon_path = icon_resources_dir.join("AppIcon.png");
                fs::copy(icon_path, &target_icon_path)
                    .map_err(|e| format!("copy icon failed: {e}"))?;

                // 更新图标引用
                let output = Command::new("plutil")
                    .args(["-replace", "CFBundleIconFile", "-string", "AppIcon.png"])
                    .arg(&info_plist_path)
                    .output()
                    .map_err(|e| format!("plutil replace CFBundleIconFile failed: {e}"))?;

                if !output.status.success() {
                    write_log(
                        "WARN",
                        &format!(
                            "plutil replace CFBundleIconFile failed: {}",
                            String::from_utf8_lossy(&output.stderr)
                        ),
                    );
                }
            }
        }
    }

    write_log(
        "INFO",
        &format!(
            "Created custom app bundle: {}",
            target_app_path.to_string_lossy()
        ),
    );
    Ok(target_app_path)
}

#[cfg(target_os = "macos")]
#[allow(dead_code)]
fn refresh_icon_cache(app_path: &Path) -> Result<(), String> {
    write_log(
        "INFO",
        &format!("Refreshing icon cache for: {}", app_path.to_string_lossy()),
    );

    // 方法1: 使用touch命令修改应用bundle的修改时间
    let output = Command::new("touch")
        .arg(app_path)
        .output()
        .map_err(|e| format!("touch command failed: {e}"))?;

    if !output.status.success() {
        write_log(
            "WARN",
            &format!("touch failed: {}", String::from_utf8_lossy(&output.stderr)),
        );
    }

    // 方法2: 清理系统图标缓存（可能需要管理员权限，所以我们只记录警告）
    let output = Command::new("sudo")
        .args([
            "find",
            "/private/var/folders/",
            "-name",
            "com.apple.iconservices",
            "-exec",
            "rm",
            "-rf",
            "{}",
            ";",
        ])
        .output();

    match output {
        Ok(_) => write_log("INFO", "Successfully cleared icon cache"),
        Err(e) => write_log(
            "WARN",
            &format!(
                "Failed to clear icon cache (this is normal without sudo): {}",
                e
            ),
        ),
    }

    // 方法3: 重启Dock相关服务
    let output = Command::new("killall").args(["Dock"]).output();

    match output {
        Ok(_) => write_log("INFO", "Successfully restarted Dock"),
        Err(e) => write_log("WARN", &format!("Failed to restart Dock: {}", e)),
    }

    // 方法4: 使用系统工具注册应用
    let output = Command::new("open").arg("-R").arg(app_path).output();

    match output {
        Ok(_) => write_log("INFO", "Successfully revealed app in Finder"),
        Err(e) => write_log("WARN", &format!("Failed to reveal app in Finder: {}", e)),
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn convert_png_to_icns(png_path: &Path, icns_path: &Path) -> Result<(), String> {
    write_log(
        "INFO",
        &format!(
            "Converting PNG to ICNS: {} -> {}",
            png_path.to_string_lossy(),
            icns_path.to_string_lossy()
        ),
    );

    // 使用macOS的sips工具将PNG转换为ICNS
    let output = Command::new("sips")
        .args([
            "-s",
            "format",
            "icns",
            png_path.to_string_lossy().as_ref(),
            "--out",
            icns_path.to_string_lossy().as_ref(),
        ])
        .output()
        .map_err(|e| format!("sips command failed: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "sips failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // 检查输出文件是否存在
    if !icns_path.exists() {
        return Err("ICNS file was not created".into());
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn create_custom_app_bundle(
    _source_app: &Path,
    _target_name: &str,
    _icon_path: &Path,
) -> Result<PathBuf, String> {
    Err("Custom app bundles are only supported on macOS".into())
}

fn find_engine_binary_for_version(version: &str) -> Option<PathBuf> {
    let mut root = engines_dir_path();
    root.push(version);
    if !root.exists() {
        return None;
    }
    // search typical patterns inside version folder
    let candidates = [
        "Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing",
        "Chromium.app/Contents/MacOS/Chromium",
        "Google Chrome.app/Contents/MacOS/Google Chrome",
    ];
    // search two levels deep
    if let Ok(entries) = fs::read_dir(&root) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                if let Ok(subs) = fs::read_dir(&p) {
                    for s in subs.flatten() {
                        let sp = s.path();
                        for rel in &candidates {
                            let f = sp.join(rel);
                            if f.is_file() {
                                write_log(
                                    "INFO",
                                    &format!(
                                        "find_engine_binary_for_version({version}) hit {}",
                                        f.to_string_lossy()
                                    ),
                                );
                                return Some(f);
                            }
                        }
                    }
                }
                for rel in &candidates {
                    let f = p.join(rel);
                    if f.is_file() {
                        write_log(
                            "INFO",
                            &format!(
                                "find_engine_binary_for_version({version}) hit {}",
                                f.to_string_lossy()
                            ),
                        );
                        return Some(f);
                    }
                }
            }
        }
    }
    for rel in &candidates {
        let f = root.join(rel);
        if f.is_file() {
            write_log(
                "INFO",
                &format!(
                    "find_engine_binary_for_version({version}) hit {}",
                    f.to_string_lossy()
                ),
            );
            return Some(f);
        }
    }
    write_log(
        "INFO",
        &format!(
            "find_engine_binary_for_version({version}) not found under {}",
            root.to_string_lossy()
        ),
    );
    None
}

#[cfg(target_os = "macos")]
fn fix_macos_exec_and_quarantine(root: &PathBuf) {
    fn walk(dir: &Path) {
        if let Ok(rd) = fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p);
                } else if p.is_file() && path_contains_macos_exec_dir(&p) {
                    if let Ok(meta) = fs::metadata(&p) {
                        #[cfg(unix)]
                        {
                            let mode = meta.permissions();
                            let new_mode =
                                PermissionsExt::from_mode(PermissionsExt::mode(&mode) | 0o111);
                            let _ = fs::set_permissions(&p, new_mode);
                        }
                    }
                }
            }
        }
    }
    walk(root);
    let _ = Command::new("xattr")
        .args(["-dr", "com.apple.quarantine"])
        .arg(root)
        .status();
    write_log(
        "INFO",
        &format!(
            "fix_macos_exec_and_quarantine done for {}",
            root.to_string_lossy()
        ),
    );
}

#[cfg(not(target_os = "macos"))]
fn fix_macos_exec_and_quarantine(_root: &PathBuf) {}

#[tauri::command]
fn browser_open(
    app: AppHandle,
    label: &str,
    url: Option<&str>,
    version: Option<&str>,
    window_title: Option<&str>,
    browser_name: Option<&str>,
) -> Result<Option<u32>, String> {
    let lbl = format!("browser-{}", label);
    let default_title = format!("Libre Browser - {}", label);
    let display_title = window_title.unwrap_or(&default_title);
    let display_name = browser_name.unwrap_or(label);
    write_log(
        "INFO",
        &format!(
            "browser_open label={} version={:?} url={:?} title={:?} name={:?}",
            label, version, url, display_title, display_name
        ),
    );
    // If we already spawned a process for this label and it's running, do nothing
    if let Ok(mut m) = proc_map().lock() {
        if let Some(child) = m.get_mut(label) {
            if child.try_wait().map_err(|e| e.to_string())?.is_none() {
                write_log(
                    "INFO",
                    &format!("browser_open label={} already running", label),
                );
                return Ok(Some(child.id()));
            }
        }
    }
    let want_restore = url.is_none();
    // Prefer launching external engine if available
    let engine_bin = if let Some(v) = version {
        find_engine_binary_for_version(v)
    } else {
        find_engine_binary()
    };
    if engine_bin.is_none() {
        write_log(
            "WARN",
            &format!(
                "no engine binary found for version={:?}; fallback to webview",
                version
            ),
        );
    }
    if let Some(bin) = engine_bin {
        // 生成自定义图标和App Bundle（仅在macOS上）
        #[allow(unused_variables)]
        let (custom_app, app_dir) = if cfg!(target_os = "macos") {
            // 生成唯一索引，基于标签哈希
            let mut hasher = Sha256::new();
            hasher.update(label.as_bytes());
            let hash = hasher.finalize();
            let icon_index = (hash[0] as usize) % 1000; // 0-999的索引

            match generate_browser_icon(display_name, icon_index) {
                Ok(icon_path) => {
                    // 创建自定义App Bundle
                    match bin
                        .parent()
                        .and_then(|p| p.parent())
                        .and_then(|p| p.parent())
                    {
                        Some(source_app) => {
                            match create_custom_app_bundle(source_app, display_name, &icon_path) {
                                Ok(custom_app) => {
                                    // 获取自定义app目录
                                    let app_dir = custom_app.parent().map(|p| p.to_path_buf());
                                    match app_dir {
                                        Some(custom_app_dir) => {
                                            write_log(
                                                "INFO",
                                                &format!(
                                                    "Using custom app bundle: {:?}",
                                                    custom_app
                                                ),
                                            );
                                            (Some(custom_app), Some(custom_app_dir))
                                        }
                                        None => {
                                            write_log("WARN", "无法获取自定义app目录");
                                            (
                                                Some(custom_app),
                                                bin.parent()
                                                    .and_then(|p| p.parent())
                                                    .and_then(|p| p.parent())
                                                    .map(|p| p.to_path_buf()),
                                            )
                                        }
                                    }
                                }
                                Err(e) => {
                                    write_log("WARN", &format!("Failed to create custom app bundle: {}, using original", e));
                                    (
                                        None,
                                        bin.parent()
                                            .and_then(|p| p.parent())
                                            .and_then(|p| p.parent())
                                            .map(|p| p.to_path_buf()),
                                    )
                                }
                            }
                        }
                        None => {
                            write_log("WARN", "无法找到源app目录");
                            (
                                None,
                                bin.parent()
                                    .and_then(|p| p.parent())
                                    .and_then(|p| p.parent())
                                    .map(|p| p.to_path_buf()),
                            )
                        }
                    }
                }
                Err(e) => {
                    write_log(
                        "WARN",
                        &format!("Failed to generate icon: {}, using original", e),
                    );
                    (
                        None,
                        bin.parent()
                            .and_then(|p| p.parent())
                            .and_then(|p| p.parent())
                            .map(|p| p.to_path_buf()),
                    )
                }
            }
        } else {
            (
                None,
                bin.parent()
                    .and_then(|p| p.parent())
                    .and_then(|p| p.parent())
                    .map(|p| p.to_path_buf()),
            )
        };

        // Best-effort: fix permissions/quarantine for existing installs before spawn
        #[cfg(target_os = "macos")]
        {
            if let Some(v) = version {
                let mut verdir = engines_dir_path();
                verdir.push(v);
                fix_macos_exec_and_quarantine(&verdir);
            } else if let Some(ref app_dir) = app_dir {
                fix_macos_exec_and_quarantine(&app_dir.to_path_buf());
            }
        }
        let profile_dir = profile_dir_path(label);
        let _ = fs::create_dir_all(&profile_dir);
        let crash_dir = profile_dir.join("crashes");
        ensure_dir(&crash_dir);
        let log_file = chrome_log_path(label);
        let mut args = vec![
            format!("--user-data-dir={}", profile_dir.to_string_lossy()),
            "--no-first-run".to_string(),
            "--no-default-browser-check".to_string(),
        ];
        if want_restore {
            args.push("--restore-last-session".into());
        } else if let Some(u) = url {
            args.push("--new-window".into());
            args.push(u.to_string());
        }
        #[cfg(target_os = "macos")]
        {
            // 避免触发钥匙串访问提示
            args.push("--password-store=basic".into());
            args.push("--use-mock-keychain".into());
        }
        // 减少提示条
        args.push("--test-type".into());
        args.push("--disable-infobars".into());
        // 设置窗口标题（Chromium 支持）
        args.push(format!("--window-name={}", display_title));
        // 打开 Chromium 日志
        args.push("--enable-logging=1".into());
        args.push("--v=1".into());
        args.push(format!("--log-file={}", log_file.to_string_lossy()));
        args.push(format!("--crash-dumps-dir={}", crash_dir.to_string_lossy()));
        #[cfg(target_os = "macos")]
        {
            // 优先通过 open 打开自定义或原始 .app，避免 GUI 激活问题
            let app_to_use = if let Some(ref custom_app) = custom_app.as_ref() {
                custom_app
            } else if let Some(app_dir) = bin
                .parent()
                .and_then(|p| p.parent())
                .and_then(|p| p.parent())
            {
                app_dir
            } else {
                write_log("ERROR", "无法找到应用目录");
                return Err("无法找到应用目录".into());
            };

            write_log(
                "INFO",
                &format!("open app {:?} with args={:?}", app_to_use, args),
            );
            let status = Command::new("open")
                .arg("-n")
                .arg(app_to_use)
                .arg("--args")
                .args(&args)
                .status();
            match status {
                Ok(st) if st.success() => {
                    // 轮询查找 PID
                    let pid = try_find_pid_by_profile(&profile_dir, 3000);
                    if let Some(pid) = pid {
                        write_log("INFO", &format!("open ok pid={}", pid));
                        // 先写入 pid 文件与监控，避免后续激活卡住影响前端状态
                        let pid_file = profile_dir_path(label).join("pid");
                        match File::create(&pid_file) {
                            Ok(mut f) => {
                                let _ = write!(f, "{}", pid);
                                let _ = f.flush();
                                write_log(
                                    "INFO",
                                    &format!("wrote pid file {}", pid_file.to_string_lossy()),
                                );
                            }
                            Err(e) => {
                                write_log(
                                    "ERROR",
                                    &format!(
                                        "create pid file {} failed: {}",
                                        pid_file.to_string_lossy(),
                                        e
                                    ),
                                );
                            }
                        }
                        // 监控进程早退，采集 Chromium 日志
                        let lbl = label.to_string();
                        let pdir = profile_dir.clone();
                        thread::spawn(move || {
                            let start = std::time::Instant::now();
                            loop {
                                if !pid_alive_unix(pid) {
                                    let ms = start.elapsed().as_millis();
                                    if ms < 5000 {
                                        // 5s 内退出认为异常
                                        write_log(
                                            "ERROR",
                                            &format!(
                                                "engine pid={} exited quickly ({}ms)",
                                                pid, ms
                                            ),
                                        );
                                        let logp = pdir.join("chrome_debug.log");
                                        if let Some(tail) = tail_file_lines(&logp, 100, 64 * 1024) {
                                            for line in tail.lines() {
                                                write_log(
                                                    "ERROR",
                                                    &format!("[ChromeLog][{}] {}", lbl, line),
                                                );
                                            }
                                        } else {
                                            write_log(
                                                "WARN",
                                                &format!("no chrome_debug.log for {}", lbl),
                                            );
                                        }
                                    }
                                    break;
                                }
                                thread::sleep(Duration::from_millis(200));
                            }
                        });
                        // 尝试前置激活窗口（异步，不阻塞主流程）
                        if let Some(app_name_os) = app_to_use.file_name() {
                            let mut app_name = app_name_os.to_string_lossy().to_string();
                            if let Some(stripped) = app_name.strip_suffix(".app") {
                                app_name = stripped.to_string();
                            }
                            match Command::new("osascript")
                                .args([
                                    "-e",
                                    &format!("tell application \"{}\" to activate", app_name),
                                ])
                                .spawn()
                            {
                                Ok(_) => write_log(
                                    "INFO",
                                    &format!("osascript activate spawned for {}", app_name),
                                ),
                                Err(e) => {
                                    write_log(
                                        "ERROR",
                                        &format!(
                                            "osascript spawn error: {} — fallback to direct spawn",
                                            e
                                        ),
                                    );
                                    let _ = Command::new(&bin)
                                        .args(&args)
                                        .stdin(Stdio::null())
                                        .stdout(Stdio::null())
                                        .stderr(Stdio::null())
                                        .status()
                                        .map(|s| {
                                            write_log(
                                                "INFO",
                                                &format!("fallback direct spawn status={}", s),
                                            )
                                        })
                                        .map_err(|e| {
                                            write_log(
                                                "ERROR",
                                                &format!("fallback direct spawn error: {}", e),
                                            )
                                        });
                                }
                            }
                        }
                        return Ok(Some(pid));
                    } else {
                        write_log("WARN", "open succeeded but pid not found within timeout");
                        return Ok(None);
                    }
                }
                Ok(st) => {
                    write_log("ERROR", &format!("open returned status {}", st));
                }
                Err(e) => {
                    write_log("ERROR", &format!("open failed: {}", e));
                }
            }
        }
        // 非 macOS 或回退：直接执行二进制
        write_log("INFO", &format!("spawn engine: {:?} args={:?}", bin, args));
        let child = Command::new(bin)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| {
                let msg = format!("spawn failed: {e}");
                write_log("ERROR", &msg);
                msg
            })?;
        let pid = child.id();
        write_log("INFO", &format!("spawn ok pid={}", pid));
        if let Ok(mut m) = proc_map().lock() {
            m.insert(label.to_string(), child);
        }
        let pid_file = profile_dir_path(label).join("pid");
        match File::create(&pid_file) {
            Ok(mut f) => {
                let _ = write!(f, "{}", pid);
                let _ = f.flush();
                write_log(
                    "INFO",
                    &format!("wrote pid file {}", pid_file.to_string_lossy()),
                );
            }
            Err(e) => {
                write_log(
                    "ERROR",
                    &format!(
                        "create pid file {} failed: {}",
                        pid_file.to_string_lossy(),
                        e
                    ),
                );
            }
        }
        // 监控进程早退，采集 Chromium 日志
        let lbl = label.to_string();
        let pdir = profile_dir.clone();
        thread::spawn(move || {
            let start = std::time::Instant::now();
            loop {
                if !pid_alive_unix(pid) {
                    let ms = start.elapsed().as_millis();
                    if ms < 5000 {
                        write_log(
                            "ERROR",
                            &format!("engine pid={} exited quickly ({}ms)", pid, ms),
                        );
                        let logp = pdir.join("chrome_debug.log");
                        if let Some(tail) = tail_file_lines(&logp, 100, 64 * 1024) {
                            for line in tail.lines() {
                                write_log("ERROR", &format!("[ChromeLog][{}] {}", lbl, line));
                            }
                        } else {
                            write_log("WARN", &format!("no chrome_debug.log for {}", lbl));
                        }
                    }
                    break;
                }
                thread::sleep(Duration::from_millis(200));
            }
        });
        return Ok(Some(pid));
    }
    // Fallback: open internal webview
    if app.get_webview_window(&lbl).is_some() {
        return Ok(None);
    }
    let to_url = url.unwrap_or("https://example.com");
    let parsed: url::Url = to_url.parse().map_err(|e: url::ParseError| e.to_string())?;
    WebviewWindowBuilder::new(&app, lbl, WebviewUrl::External(parsed))
        .title("Libre Browser Tab")
        .build()
        .map_err(|e| {
            let msg = format!("webview build failed: {e}");
            write_log("ERROR", &msg);
            msg
        })?;
    Ok(None)
}

#[tauri::command]
fn browser_close(app: AppHandle, label: &str) -> Result<(), String> {
    // Close spawned engine process if any
    if let Ok(mut m) = proc_map().lock() {
        if let Some(mut child) = m.remove(label) {
            let _ = child.kill();
            let _ = child.wait();
            let _ = fs::remove_file(profile_dir_path(label).join("pid"));
            return Ok(());
        }
    }
    // Kill by pid file fallback
    let pid_path = profile_dir_path(label).join("pid");
    if let Ok(mut f) = File::open(&pid_path) {
        let mut s = String::new();
        if f.read_to_string(&mut s).is_ok() {
            if let Ok(pid) = s.trim().parse::<u32>() {
                write_log(
                    "INFO",
                    &format!("browser_close killing pid={} for label={}", pid, label),
                );
                // try graceful then force
                let _ = Command::new("kill")
                    .args(["-TERM", &pid.to_string()])
                    .status();
                // wait up to 2s
                let start = std::time::Instant::now();
                loop {
                    if !pid_alive_unix(pid) {
                        break;
                    }
                    if start.elapsed() > Duration::from_millis(2000) {
                        break;
                    }
                    thread::sleep(Duration::from_millis(100));
                }
                if pid_alive_unix(pid) {
                    let _ = Command::new("kill")
                        .args(["-KILL", &pid.to_string()])
                        .status();
                }
                let _ = fs::remove_file(&pid_path);
                write_log("INFO", &format!("browser_close label={} done", label));
                return Ok(());
            }
        }
    }
    let lbl = format!("browser-{}", label);
    if let Some(w) = app.get_webview_window(&lbl) {
        let _ = w.close();
        write_log(
            "INFO",
            &format!("browser_close closed webview label={}", label),
        );
    }
    Ok(())
}

#[tauri::command]
fn browser_exists(app: AppHandle, label: &str) -> Result<bool, String> {
    // Check spawned process
    if let Ok(mut m) = proc_map().lock() {
        if let Some(child) = m.get_mut(label) {
            if child.try_wait().map_err(|e| e.to_string())?.is_none() {
                return Ok(true);
            }
        }
    }
    // Check internal webview window
    let lbl = format!("browser-{}", label);
    Ok(app.get_webview_window(&lbl).is_some())
}

#[allow(unused_variables)]
fn pid_alive_unix(pid: u32) -> bool {
    #[cfg(target_family = "unix")]
    {
        // Prefer `kill -0 <pid>` which returns success if process exists
        if let Ok(status) = Command::new("kill").args(["-0", &pid.to_string()]).status() {
            return status.success();
        }
        // Fallback: ps output non-empty
        if let Ok(out) = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "pid="])
            .output()
        {
            return !out.stdout.is_empty();
        }
    }
    false
}

#[tauri::command]
fn browser_running(label: &str) -> Option<u32> {
    let pid_file = profile_dir_path(label).join("pid");
    if let Ok(mut f) = File::open(&pid_file) {
        let mut s = String::new();
        if f.read_to_string(&mut s).is_ok() {
            if let Ok(pid) = s.trim().parse::<u32>() {
                if pid_alive_unix(pid) {
                    return Some(pid);
                }
            }
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn try_find_pid_by_profile(profile_dir: &Path, timeout_ms: u64) -> Option<u32> {
    let end = std::time::Instant::now() + Duration::from_millis(timeout_ms);
    while std::time::Instant::now() < end {
        // Use ps to search processes containing the profile dir path
        if let Ok(out) = Command::new("ps").args(["-axo", "pid,command"]).output() {
            if out.status.success() {
                if let Ok(text) = String::from_utf8(out.stdout) {
                    let prof = profile_dir.to_string_lossy();
                    for line in text.lines() {
                        if line.contains(&*prof)
                            && (line.contains("Google Chrome for Testing")
                                || line.contains("Chromium"))
                        {
                            let pid_str = line.split_whitespace().next().unwrap_or("");
                            if let Ok(pid) = pid_str.parse::<u32>() {
                                return Some(pid);
                            }
                        }
                    }
                }
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
    None
}

#[cfg(not(target_os = "macos"))]
#[allow(dead_code)]
fn try_find_pid_by_profile(_profile_dir: &Path, _timeout_ms: u64) -> Option<u32> {
    None
}

#[tauri::command]
fn read_logs_tail(lines: usize) -> Result<String, String> {
    let dir = logs_dir_path();
    let file = dir.join("current.log");
    let mut content = String::new();
    if let Ok(mut f) = File::open(&file) {
        use std::io::Read;
        let _ = f.read_to_string(&mut content);
    } else {
        return Err("log file not found".into());
    }
    let v: Vec<&str> = content.lines().collect();
    let n = lines.min(v.len());
    Ok(v[v.len() - n..].join("\n"))
}

#[cfg(test)]
mod test_icon;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // prepare logging
    let _ = rotate_logs();
    write_log("INFO", "应用启动");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_http::init())
        .invoke_handler(tauri::generate_handler![
            logs_dir,
            log_info,
            engines_dir,
            list_installed_engines,
            download_engine,
            download_engine_archive,
            extract_engine_archive,
            browser_open,
            browser_close,
            browser_exists,
            browser_running,
            read_logs_tail
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
