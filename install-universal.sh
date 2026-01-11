#!/bin/bash

# 三术 MCP 工具安装脚本 - 支持 macOS、Linux
# 只需要构建和安装两个CLI工具即可运行MCP

set -e

echo "🚀 开始安装 三术 MCP 工具..."

# 检测操作系统
OS="unknown"
case "$OSTYPE" in
    darwin*)  OS="macos" ;;
    linux*)   OS="linux" ;;
    msys*|cygwin*|mingw*) OS="windows" ;;
    *)        echo "❌ 不支持的操作系统: $OSTYPE"; exit 1 ;;
esac

echo "🔍 检测到操作系统: $OS"

# 检查必要的工具
check_command() {
    if ! command -v "$1" &> /dev/null; then
        echo "❌ 错误: 未找到 $1 命令"
        echo "请先安装 $1"
        exit 1
    fi
}

echo "🔧 检查必要工具..."
check_command "cargo"
check_command "pnpm"

# 构建MCP CLI工具
echo "🔨 构建 MCP CLI 工具..."
if ! cargo tauri --version >/dev/null 2>&1; then
    cargo install tauri-cli --locked --version 2.9.1
fi

cargo tauri build --no-bundle
cargo build --release --bin sanshu-mcp

# 检查构建结果
if [[ ! -f "target/release/sanshu-ui" ]] || [[ ! -f "target/release/sanshu-mcp" ]]; then
    echo "❌ CLI 工具构建失败"
    echo "请检查构建错误并重试"
    exit 1
fi

echo "✅ CLI 工具构建成功"

# 根据操作系统安装CLI工具
if [[ "$OS" == "macos" ]]; then
    echo "🍎 macOS 安装模式..."

    # 安装到 /usr/local/bin
    INSTALL_DIR="/usr/local/bin"

    echo "📋 安装 CLI 工具到 $INSTALL_DIR..."
    sudo cp "target/release/sanshu-ui" "$INSTALL_DIR/sanshu-ui"
    sudo cp "target/release/sanshu-mcp" "$INSTALL_DIR/sanshu-mcp"
    sudo chmod +x "$INSTALL_DIR/sanshu-ui"
    sudo chmod +x "$INSTALL_DIR/sanshu-mcp"

    echo "✅ CLI 工具已安装到 $INSTALL_DIR"

elif [[ "$OS" == "linux" ]]; then
    echo "🐧 Linux 安装模式..."

    # 创建用户本地目录
    LOCAL_DIR="$HOME/.local"
    BIN_DIR="$LOCAL_DIR/bin"

    mkdir -p "$BIN_DIR"

    # 复制CLI工具
    cp "target/release/sanshu-ui" "$BIN_DIR/sanshu-ui"
    cp "target/release/sanshu-mcp" "$BIN_DIR/sanshu-mcp"
    chmod +x "$BIN_DIR/sanshu-ui"
    chmod +x "$BIN_DIR/sanshu-mcp"

    echo "✅ CLI 工具已安装到 $BIN_DIR"

    # 检查PATH
    if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
        echo ""
        echo "💡 请将以下内容添加到您的 shell 配置文件中 (~/.bashrc 或 ~/.zshrc):"
        echo "export PATH=\"\$PATH:$BIN_DIR\""
        echo ""
        echo "然后运行: source ~/.bashrc (或 source ~/.zshrc)"
    fi

else
    echo "❌ Windows 平台请使用 Windows 专用安装程序"
    exit 1
fi

echo ""
echo "🎉 三术 MCP 工具安装完成！"
echo ""
echo "📋 使用方法："
echo "  💻 MCP 服务器模式:"
echo "    sanshu-mcp                       - 启动 MCP 服务器"
echo ""
echo "  🎨 弹窗界面模式:"
echo "    sanshu-ui                        - 启动设置界面"
echo "    sanshu-ui --mcp-request file     - MCP 弹窗模式"
echo ""
echo "📝 配置 MCP 客户端："
echo "将以下内容添加到您的 MCP 客户端配置中："
echo ""
cat << 'EOF'
{
  "mcpServers": {
    "sanshu": {
      "command": "sanshu-mcp"
    }
  }
}
EOF
echo ""
echo "💡 重要说明："
echo "  • 两个CLI工具必须在同一目录下才能正常工作"
echo "  • 'sanshu-mcp' 是MCP服务器，'sanshu-ui' 是弹窗界面"
echo "  • 无需安装完整应用，只需要这两个CLI工具即可"
echo ""

if [[ "$OS" == "macos" ]]; then
    echo "🔗 CLI 工具已安装到 /usr/local/bin/"
elif [[ "$OS" == "linux" ]]; then
    echo "🔗 CLI 工具已安装到 $BIN_DIR"
fi
