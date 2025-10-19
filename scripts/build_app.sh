#!/bin/bash
set -e

APP_NAME="Convey"
VERSION="0.1.0"
BUNDLE_ID="com.narennaik.convey"

echo "Building Convey for distribution..."

# Build release binary
echo "Step 1: Building release binary..."
cargo build --release

# Create build directory
mkdir -p build

# Clean previous build
rm -rf "build/${APP_NAME}.app"

# Create app bundle structure
echo "Step 2: Creating app bundle structure..."
mkdir -p "build/${APP_NAME}.app/Contents/MacOS"
mkdir -p "build/${APP_NAME}.app/Contents/Resources"

# Copy binary
echo "Step 3: Copying binary..."
cp "target/release/convey" "build/${APP_NAME}.app/Contents/MacOS/${APP_NAME}"

# Copy resources (including the Whisper model)
echo "Step 4: Copying resources (including Whisper model)..."
cp -r "resources" "build/${APP_NAME}.app/Contents/Resources/"

# Copy icon
echo "Step 5: Copying app icon..."
cp "assets/Convey.icns" "build/${APP_NAME}.app/Contents/Resources/"

# Create Info.plist
echo "Step 6: Creating Info.plist..."
cat > "build/${APP_NAME}.app/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>${BUNDLE_ID}</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>CFBundleIconFile</key>
    <string>Convey</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSMicrophoneUsageDescription</key>
    <string>Convey needs microphone access to transcribe your voice.</string>
    <key>NSAppleEventsUsageDescription</key>
    <string>Convey needs to simulate keyboard input to paste transcribed text.</string>
</dict>
</plist>
EOF

# Create PkgInfo
echo "APPL????" > "build/${APP_NAME}.app/Contents/PkgInfo"

echo ""
echo "✅ App bundle created successfully: build/${APP_NAME}.app"
echo ""
echo "Bundle size:"
du -sh "build/${APP_NAME}.app"
echo ""
echo "To test the app, run:"
echo "  open build/${APP_NAME}.app"
echo ""
echo "To create a DMG for distribution:"
echo "  ./make_dmg.sh"
