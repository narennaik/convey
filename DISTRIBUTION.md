# Convey - Distribution Guide

## Building for Distribution

### Prerequisites
- Rust toolchain installed
- macOS 10.13+ (for building)
- `whisper-cli` installed (users will need this too)

### Build Steps

1. **Build the app bundle:**
   ```bash
   cd src-tauri
   ./build_app.sh
   ```

   This will:
   - Build the release binary
   - Create `Convey.app` with the proper bundle structure
   - Include the Whisper model (141MB) in the app bundle
   - Set up proper permissions and Info.plist

2. **Test the app:**
   ```bash
   open Convey.app
   ```

3. **Create a DMG for distribution:**
   ```bash
   hdiutil create -volname Convey -srcfolder Convey.app -ov -format UDZO Convey-0.1.0.dmg
   ```

## App Structure

```
Convey.app/
├── Contents/
│   ├── MacOS/
│   │   └── Convey              # Main executable
│   ├── Resources/
│   │   └── resources/
│   │       └── models/
│   │           └── ggml-base.bin  # Whisper model (141MB)
│   ├── Info.plist              # App metadata
│   └── PkgInfo
```

## What's Included

- **Whisper Model**: The `ggml-base.bin` model is bundled with the app (~141MB)
- **Dependencies**: All Rust dependencies are statically linked
- **Resources**: Any additional resources in the `resources/` folder

## Requirements for Users

Users need to install:
1. **whisper-cli**: The app looks for it in:
   - `/opt/homebrew/bin/whisper-cli` (Homebrew Apple Silicon)
   - `/usr/local/bin/whisper-cli` (Homebrew Intel)
   - `/usr/bin/whisper-cli`
   - Or in `$PATH`

   Install via: `brew install whisper-cpp`

2. **System Permissions**:
   - Microphone access (requested on first use)
   - Accessibility access (for keyboard simulation/paste)

## Code Signing (Optional but Recommended)

For distribution outside the Mac App Store:

```bash
# Sign the app
codesign --deep --force --verify --verbose --sign "Developer ID Application: Your Name" Convey.app

# Notarize the app (requires Apple Developer account)
xcrun notarytool submit Convey-0.1.0.dmg --keychain-profile "notary-profile" --wait

# Staple the notarization ticket
xcrun stapler staple Convey.app
xcrun stapler staple Convey-0.1.0.dmg
```

## Distribution

- **DMG**: Best for direct download distribution
- **ZIP**: Alternative for simpler downloads
- **Homebrew Cask**: For package manager distribution

## File Sizes

- Binary: ~10-15MB (release build)
- Whisper Model: 141MB
- **Total App Size**: ~155MB
