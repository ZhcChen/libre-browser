#!/bin/bash

# Libre Browser 自动发布脚本
# 用于自动化版本管理、构建和发布流程

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印带颜色的消息
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查依赖
check_dependencies() {
    print_info "检查项目依赖..."

    # 检查 Rust
    if ! command -v cargo &> /dev/null; then
        print_error "Rust 未安装，请先安装 Rust"
        exit 1
    fi

    # 检查 Node.js
    if ! command -v node &> /dev/null; then
        print_error "Node.js 未安装，请先安装 Node.js"
        exit 1
    fi

    # 检查 npm/pnpm
    if command -v pnpm &> /dev/null; then
        PKG_MANAGER="pnpm"
    elif command -v npm &> /dev/null; then
        PKG_MANAGER="npm"
    else
        print_error "包管理器未找到，请安装 npm 或 pnpm"
        exit 1
    fi

    print_success "依赖检查通过 ($PKG_MANAGER)"
}

# 安装依赖
install_dependencies() {
    print_info "安装项目依赖..."

    # 安装前端依赖
    cd src
    $PKG_MANAGER install
    cd ..

    # 安装 Rust 依赖
    cargo check

    print_success "依赖安装完成"
}

# 运行测试
run_tests() {
    print_info "运行测试..."

    # Rust 测试
    cargo test

    # 前端测试 (如果有的话)
    if [ -f "src/package.json" ] && grep -q "test" "src/package.json"; then
        cd src
        $PKG_MANAGER test
        cd ..
    fi

    print_success "测试通过"
}

# 代码质量检查
check_code_quality() {
    print_info "代码质量检查..."

    # Rust 格式化检查
    if ! cargo fmt --all -- --check; then
        print_error "代码格式化检查失败，请运行 'cargo fmt' 修复"
        exit 1
    fi

    # Rust Clippy 检查
    if ! cargo clippy -- -D warnings; then
        print_error "Clippy 检查失败，请修复警告"
        exit 1
    fi

    print_success "代码质量检查通过"
}

# 构建项目
build_project() {
    print_info "构建项目..."

    # 构建前端
    cd src
    $PKG_MANAGER run build
    cd ..

    # 构建 Tauri 应用
    cargo tauri build

    print_success "构建完成"
}

# 版本管理
version_management() {
    local action=$1
    local version_type=$2

    case $action in
        "bump")
            print_info "升级版本号 ($version_type)..."

            # 使用 cargo-edit 升级版本
            if command -v cargo-edit &> /dev/null; then
                cargo edit set "$version_type"
            else
                # 手动修改 Cargo.toml
                current_version=$(grep '^version = ' src-tauri/Cargo.toml | sed 's/version = "//' | sed 's/["]//g')
                print_warning "请手动升级版本号: $current_version -> ?"
                print_info "编辑 src-tauri/Cargo.toml 文件中的版本号"
                read -p "按回车继续..."
            fi
            ;;
        "tag")
            local version=$(grep '^version = ' src-tauri/Cargo.toml | sed 's/version = "//' | sed 's/["]//g')
            print_info "创建 Git 标签: v$version"
            git add .
            git commit -m "chore: bump version to v$version" || true
            git tag "v$version"
            print_success "标签创建完成: v$version"
            ;;
        "push")
            print_info "推送代码和标签到远程仓库..."
            git push origin main
            git push origin --tags
            print_success "推送完成"
            ;;
    esac
}

# 清理构建文件
clean() {
    print_info "清理构建文件..."

    # 清理前端构建
    if [ -d "src/dist" ]; then
        rm -rf src/dist
    fi

    # 清理 Rust 构建缓存
    cargo clean

    print_success "清理完成"
}

# 显示帮助信息
show_help() {
    echo "Libre Browser 自动发布脚本"
    echo ""
    echo "用法: $0 [命令] [选项]"
    echo ""
    echo "命令:"
    echo "  check         检查依赖和环境"
    echo "  install       安装项目依赖"
    echo "  test          运行测试"
    echo "  quality       代码质量检查"
    echo "  build         构建项目"
    echo "  release       完整发布流程"
    echo "  clean         清理构建文件"
    echo ""
    echo "版本管理:"
    echo "  bump <type>   升级版本号 (major|minor|patch)"
    echo "  tag           创建 Git 标签"
    echo "  push          推送到远程仓库"
    echo ""
    echo "示例:"
    echo "  $0 release                    # 完整发布流程"
    echo "  $0 bump patch && $0 tag       # 升级补丁版本并打标签"
    echo "  $0 build                      # 仅构建项目"
    echo ""
}

# 主函数
main() {
    local command=${1:-"help"}

    case $command in
        "check")
            check_dependencies
            ;;
        "install")
            check_dependencies
            install_dependencies
            ;;
        "test")
            check_dependencies
            run_tests
            ;;
        "quality")
            check_dependencies
            check_code_quality
            ;;
        "build")
            check_dependencies
            build_project
            ;;
        "bump")
            if [ -z "$2" ]; then
                print_error "请指定版本类型: major|minor|patch"
                exit 1
            fi
            version_management bump "$2"
            ;;
        "tag")
            version_management tag
            ;;
        "push")
            version_management push
            ;;
        "release")
            print_info "开始完整发布流程..."
            check_dependencies
            install_dependencies
            run_tests
            check_code_quality
            build_project

            # 询问是否升级版本
            echo ""
            read -p "是否需要升级版本号? (y/n): " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                echo "选择版本类型:"
                echo "1) major - 主版本 (1.0.0 -> 2.0.0)"
                echo "2) minor - 次版本 (1.0.0 -> 1.1.0)"
                echo "3) patch - 补丁版本 (1.0.0 -> 1.0.1)"
                read -p "请选择 (1-3): " -n 1 -r
                echo

                case $REPLY in
                    1) version_management bump "major" ;;
                    2) version_management bump "minor" ;;
                    3) version_management bump "patch" ;;
                    *) print_warning "跳过版本升级" ;;
                esac

                # 创建标签
                version_management tag

                # 询问是否推送
                echo ""
                read -p "是否推送到远程仓库? (y/n): " -n 1 -r
                echo
                if [[ $REPLY =~ ^[Yy]$ ]]; then
                    version_management push
                fi
            fi

            print_success "发布流程完成!"
            print_info "构建文件位于: src-tauri/target/release/bundle/"
            ;;
        "clean")
            clean
            ;;
        "help"|"-h"|"--help")
            show_help
            ;;
        *)
            print_error "未知命令: $command"
            show_help
            exit 1
            ;;
    esac
}

# 运行主函数
main "$@"