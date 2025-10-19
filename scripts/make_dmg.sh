#!/bin/bash
set -e

VERSION="0.1.0"

# Clean up
rm -rf dmg_source build/Convey-${VERSION}.dmg

# Create source folder
mkdir dmg_source
cp -r build/Convey.app dmg_source/
ln -s /Applications dmg_source/Applications

# Create DMG directly
hdiutil create -volname "Convey" -srcfolder dmg_source -ov -format UDZO build/Convey-${VERSION}.dmg

# Clean up
rm -rf dmg_source

echo "✅ DMG created: build/Convey-${VERSION}.dmg"
ls -lh build/Convey-${VERSION}.dmg
