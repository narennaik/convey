#!/bin/bash
set -e

echo "Generating app icons from SVG..."

# Clean up old files
rm -rf icon*.png Convey.iconset Convey.icns

# Generate PNGs from SVG
echo "Step 1: Generating PNG files..."
qlmanage -t -s 1024 -o . icon.svg
mv icon.svg.png icon_1024.png

sips -z 512 512 icon_1024.png --out icon_512.png
sips -z 256 256 icon_1024.png --out icon_256.png
sips -z 128 128 icon_1024.png --out icon_128.png
sips -z 64 64 icon_1024.png --out icon_64.png
sips -z 32 32 icon_1024.png --out icon_32.png
sips -z 16 16 icon_1024.png --out icon_16.png

# Create iconset
echo "Step 2: Creating iconset..."
mkdir -p Convey.iconset
cp icon_16.png Convey.iconset/icon_16x16.png
cp icon_32.png Convey.iconset/icon_16x16@2x.png
cp icon_32.png Convey.iconset/icon_32x32.png
cp icon_64.png Convey.iconset/icon_32x32@2x.png
cp icon_128.png Convey.iconset/icon_128x128.png
cp icon_256.png Convey.iconset/icon_128x128@2x.png
cp icon_256.png Convey.iconset/icon_256x256.png
cp icon_512.png Convey.iconset/icon_256x256@2x.png
cp icon_512.png Convey.iconset/icon_512x512.png
cp icon_1024.png Convey.iconset/icon_512x512@2x.png

# Generate ICNS
echo "Step 3: Generating ICNS file..."
iconutil -c icns Convey.iconset

echo "✅ Icon generation complete!"
ls -lh Convey.icns
