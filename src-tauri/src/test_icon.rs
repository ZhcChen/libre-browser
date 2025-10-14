// 测试图标生成功能
use crate::{data_local_base, generate_browser_icon};
use std::path::Path;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_generation() {
        let test_name = "TestBrowser";
        let test_index = 42;

        println!("测试图标生成功能...");

        match generate_browser_icon(test_name, test_index) {
            Ok(icon_path) => {
                println!("✓ 图标生成成功: {}", icon_path.to_string_lossy());

                // 检查文件是否存在
                assert!(icon_path.exists(), "图标文件应该存在");

                // 检查文件大小
                let metadata = std::fs::metadata(&icon_path).expect("无法读取图标文件元数据");
                assert!(metadata.len() > 0, "图标文件大小应该大于0");

                println!("✓ 图标文件大小: {} bytes", metadata.len());

                // 清理测试文件
                let _ = std::fs::remove_file(&icon_path);
                println!("✓ 测试文件已清理");
            }
            Err(e) => {
                panic!("图标生成失败: {}", e);
            }
        }
    }

    #[test]
    fn test_custom_apps_directory_creation() {
        let base_dir = data_local_base();
        let custom_apps_dir = base_dir.join("custom_apps");

        println!("测试自定义应用目录创建...");

        // 尝试创建目录
        match std::fs::create_dir_all(&custom_apps_dir) {
            Ok(()) => {
                println!(
                    "✓ 自定义应用目录创建成功: {}",
                    custom_apps_dir.to_string_lossy()
                );
                assert!(custom_apps_dir.exists(), "目录应该存在");
            }
            Err(e) => {
                panic!("目录创建失败: {}", e);
            }
        }
    }
}

// 手动测试函数，不是正式测试
pub fn manual_test_icon_generation() -> Result<(), String> {
    println!("开始手动测试图标生成功能...");

    let test_names = vec![
        ("Work Browser", 1),
        ("Personal Browser", 2),
        ("Development", 3),
        ("Social Media", 4),
        ("Research", 5),
    ];

    for (name, index) in test_names {
        println!("正在为 '{}' 生成图标...", name);
        match generate_browser_icon(name, index) {
            Ok(icon_path) => {
                println!("✓ 成功生成: {}", icon_path.to_string_lossy());
            }
            Err(e) => {
                println!("✗ 生成失败: {}", e);
                return Err(e);
            }
        }
    }

    println!("所有图标生成测试完成！");
    Ok(())
}
