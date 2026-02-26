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
Libre Browser 编译脚本

用法:
  ./scripts/build.sh [tauri build 参数...]

示例:
  ./scripts/build.sh
  ./scripts/build.sh --debug
  ./scripts/build.sh --target aarch64-apple-darwin
  ./scripts/build.sh --bundles app

说明:
  该脚本会将所有参数透传给 `pnpm run tauri:build`。
  后续可在此脚本内扩展不同平台/架构的构建流程。
EOF
}

if [ "${1:-}" = "help" ] || [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; then
  show_help
  exit 0
fi

ensure_command "pnpm" "未检测到 pnpm，请先安装 pnpm"
ensure_command "cargo" "未检测到 Rust/cargo，请先安装 Rust 工具链"

if [ ! -d "node_modules" ]; then
  print_info "检测到未安装前端依赖，正在执行 pnpm install..."
  pnpm install
fi

if [ "$#" -gt 0 ]; then
  print_info "开始编译（附加参数: $*）..."
  exec pnpm run tauri:build -- "$@"
fi

print_info "开始编译生产版本..."
exec pnpm run tauri:build
