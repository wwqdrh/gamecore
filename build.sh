#!/usr/bin/env bash
# core GDExtension 构建脚本
# 用法: ./build.sh [debug|release]
# 默认构建 debug 版本

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR"
CRATE_NAME="core"
LIB_NAME="libgamecore"
BIN_DIR="$PROJECT_ROOT/addons/gamecore/bin"

BUILD_TYPE="${1:-debug}"

if [[ "$BUILD_TYPE" != "debug" && "$BUILD_TYPE" != "release" ]]; then
    echo "错误: 参数必须是 debug 或 release"
    echo "用法: $0 [debug|release]"
    exit 1
fi

CARGO_FLAG=""
if [[ "$BUILD_TYPE" == "release" ]]; then
    CARGO_FLAG="--release"
fi

install_macos() {
    local FRAMEWORK_NAME="${LIB_NAME}.macos.template_${BUILD_TYPE}.framework"
    local FRAMEWORK_DIR="$BIN_DIR/macos/$FRAMEWORK_NAME"
    local FRAMEWORK_RESOURCES="$FRAMEWORK_DIR/Resources"
    local EXECUTABLE_NAME="${LIB_NAME}.macos.template_${BUILD_TYPE}"

    echo "创建 framework: $FRAMEWORK_NAME"

    rm -rf "$FRAMEWORK_DIR"
    mkdir -p "$FRAMEWORK_DIR"
    mkdir -p "$FRAMEWORK_RESOURCES"

    cp "$DYLIB" "$FRAMEWORK_DIR/$EXECUTABLE_NAME"
    chmod +x "$FRAMEWORK_DIR/$EXECUTABLE_NAME"

    install_name_tool -id "@executable_path/../Frameworks/$FRAMEWORK_NAME/$EXECUTABLE_NAME" "$FRAMEWORK_DIR/$EXECUTABLE_NAME" 2>/dev/null || true

    cat > "$FRAMEWORK_RESOURCES/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleInfoDictionaryVersion</key>
	<string>6.0</string>
	<key>CFBundleDevelopmentRegion</key>
	<string>en</string>
	<key>CFBundleExecutable</key>
	<string>$EXECUTABLE_NAME</string>
	<key>CFBundleName</key>
	<string>GameKit Core</string>
	<key>CFBundleDisplayName</key>
	<string>GameKit Core</string>
	<key>CFBundleIdentifier</key>
	<string>com.gamekit.core</string>
	<key>NSHumanReadableCopyright</key>
	<string>Copyright 2026 gamekit</string>
	<key>CFBundleVersion</key>
	<string>1.0.0</string>
	<key>CFBundleShortVersionString</key>
	<string>1.0.0</string>
	<key>CFBundlePackageType</key>
	<string>FMWK</string>
	<key>CSResourcesFileMapped</key>
	<true/>
	<key>DTPlatformName</key>
	<string>macosx</string>
	<key>LSMinimumSystemVersion</key>
	<string>10.12</string>
</dict>
</plist>
EOF

    echo "已安装: $FRAMEWORK_DIR"
}

install_linux() {
    local SO_NAME="${LIB_NAME}.linux.template_${BUILD_TYPE}.x86_64.so"
    local DEST_DIR="$BIN_DIR/linux"

    mkdir -p "$DEST_DIR"
    cp "$DYLIB" "$DEST_DIR/$SO_NAME"
    chmod +x "$DEST_DIR/$SO_NAME"

    echo "已安装: $DEST_DIR/$SO_NAME"
}

install_windows() {
    local DLL_NAME="${LIB_NAME}.windows.template_${BUILD_TYPE}.x86_64.dll"
    local DEST_DIR="$BIN_DIR/windows"

    mkdir -p "$DEST_DIR"
    cp "$DYLIB" "$DEST_DIR/$DLL_NAME"

    echo "已安装: $DEST_DIR/$DLL_NAME"
}

echo "========================================="
echo " 构建 core GDExtension ($BUILD_TYPE)"
echo "========================================="

echo ""
echo "[1/3] 编译 Rust 扩展..."
cd "$PROJECT_ROOT"
cargo build -p "$CRATE_NAME" $CARGO_FLAG

echo ""
echo "[2/3] 查找构建产物..."
if [[ "$BUILD_TYPE" == "debug" ]]; then
    TARGET_DIR="$PROJECT_ROOT/target/debug"
else
    TARGET_DIR="$PROJECT_ROOT/target/release"
fi

if [[ "$(uname)" == "Darwin" ]]; then
    DYLIB="$TARGET_DIR/lib${CRATE_NAME}.dylib"
    PLATFORM="macos"
elif [[ "$(uname)" == "Linux" ]]; then
    DYLIB="$TARGET_DIR/lib${CRATE_NAME}.so"
    PLATFORM="linux"
else
    DYLIB="$TARGET_DIR/${CRATE_NAME}.dll"
    PLATFORM="windows"
fi

if [[ ! -f "$DYLIB" ]]; then
    echo "错误: 找不到构建产物 $DYLIB"
    exit 1
fi

echo "找到产物: $DYLIB"

echo ""
echo "[3/3] 安装到 bin 目录..."

if [[ "$PLATFORM" == "macos" ]]; then
    install_macos
elif [[ "$PLATFORM" == "linux" ]]; then
    install_linux
else
    install_windows
fi

echo ""
echo "========================================="
echo " 构建完成! ($BUILD_TYPE)"
echo "========================================="
