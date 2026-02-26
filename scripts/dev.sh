#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

print_info() {
  echo "[INFO] $1"
}

print_error() {
  echo "[ERROR] $1" >&2
}

ensure_command() {
  local cmd="$1"
  local tip="$2"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    print_error "$tip"
    exit 1
  fi
}

show_help() {
  cat <<'EOF'
Libre Browser 启动脚本

用法:
  ./scripts/dev.sh [mode]

模式:
  tauri   启动 Tauri 开发环境（默认）
  web     仅启动前端 Vite 开发服务
  help    显示帮助
EOF
}

MODE="${1:-tauri}"

case "$MODE" in
  help|-h|--help)
    show_help
    exit 0
    ;;
  tauri|web)
    ;;
  *)
    print_error "不支持的模式: $MODE"
    show_help
    exit 1
    ;;
esac

ensure_command "pnpm" "未检测到 pnpm，请先安装 pnpm"
ensure_command "cargo" "未检测到 Rust/cargo，请先安装 Rust 工具链"

if [ ! -d "node_modules" ]; then
  print_info "检测到未安装前端依赖，正在执行 pnpm install..."
  pnpm install
fi

if [ "$MODE" = "web" ]; then
  print_info "启动前端开发服务..."
  exec pnpm run dev
fi

print_info "启动 Tauri 开发环境..."
exec pnpm run tauri:dev
