# Build Instructions

## Prerequisites

- **Rust toolchain** (1.76+): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **whisper-cli**: `brew install whisper-cpp`
- **macOS 10.13+** (for building)

## Project Structure

```
convey/
├── Cargo.toml           # Rust project configuration
├── assets/              # Design assets
│   └── icon.svg        # Source SVG icon (Solarized colors)
├── build/               # Build outputs (gitignored)
│   ├── Convey.app      # macOS app bundle
│   └── Convey-0.1.0.dmg # Distribution DMG
├── resources/           # Bundled resources
│   └── models/
│       └── ggml-base.bin # Whisper base model (141MB)
├── scripts/             # Build and utility scripts
│   ├── build_app.sh    # Build .app bundle
│   ├── make_dmg.sh     # Create DMG
│   └── generate_icons.sh # Generate icons from SVG
└── src/                 # Rust source code
```

## Quick Start

### Build the App

```bash
./scripts/build_app.sh
```

Creates `build/Convey.app` with:
- Release-optimized binary (~15MB)
- Bundled Whisper model (141MB)
- Solarized app icon
- macOS permissions configured (microphone, accessibility)

### Create DMG for Distribution

```bash
./scripts/make_dmg.sh
```

Creates `build/Convey-0.1.0.dmg` (~138MB compressed) with:
- The app bundle
- Applications folder symlink for drag-to-install

### Regenerate Icons

If you modify `assets/icon.svg`:

```bash
./scripts/generate_icons.sh
```

This regenerates all PNG sizes (16×16 to 1024×1024) and the ICNS file.

## Development

### Run in Development Mode

```bash
# Run with logging
RUST_LOG=info cargo run --release

# Watch for changes (requires cargo-watch)
cargo watch -x 'run --release'
```

### Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Test specific module
cargo test database::tests
```

### Code Quality

```bash
# Format code
cargo fmt

# Check for errors
cargo check

# Lint with Clippy
cargo clippy -- -D warnings
```

## Clean Build

```bash
# Remove build artifacts
rm -rf build/

# Clean Cargo build cache
cargo clean

# Full clean (including downloaded dependencies)
cargo clean && rm -rf target/
```

## Distribution

### For Public Release

1. **Build the app**:
   ```bash
   ./scripts/build_app.sh
   ```

2. **Test thoroughly**:
   ```bash
   open build/Convey.app
   # Test: recording, transcription, auto-paste, settings
   ```

3. **Create DMG**:
   ```bash
   ./scripts/make_dmg.sh
   ```

4. **Sign and Notarize** (requires Apple Developer account):
   ```bash
   # Sign the app
   codesign --deep --force --verify --verbose \
     --sign "Developer ID Application: Your Name" \
     build/Convey.app

   # Create DMG
   ./scripts/make_dmg.sh

   # Notarize
   xcrun notarytool submit build/Convey-0.1.0.dmg \
     --keychain-profile "notary-profile" \
     --wait

   # Staple
   xcrun stapler staple build/Convey.app
   xcrun stapler staple build/Convey-0.1.0.dmg
   ```

See [DISTRIBUTION.md](DISTRIBUTION.md) for complete distribution instructions.

## Troubleshooting

### Build Errors

**"Unable to find whisper-cli"**
→ Install: `brew install whisper-cpp`

**Linker errors on macOS**
→ Install Xcode Command Line Tools: `xcode-select --install`

**Rust version too old**
→ Update: `rustup update stable`

### Runtime Issues

**App doesn't start**
→ Check permissions: System Settings → Privacy & Security → Accessibility

**No audio captured**
→ Grant microphone permission: System Settings → Privacy & Security → Microphone

**Transcription fails**
→ Check model exists: `ls resources/models/ggml-base.bin`

## Advanced

### Custom Whisper Model

Replace `resources/models/ggml-base.bin` with another model:

```bash
# Download larger model for better accuracy
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin \
  -O resources/models/ggml-medium.bin

# Update whisper.rs to use new model
# Then rebuild
./scripts/build_app.sh
```

### Development with Different Model

```bash
# Override model path
WHISPER_MODEL=/path/to/custom-model.bin cargo run --release
```

### Profiling

```bash
# CPU profiling with instruments (macOS)
cargo build --release
xcrun xctrace record --template 'Time Profiler' \
  --launch target/release/convey

# Memory profiling
cargo build --release
leaks --atExit -- target/release/convey
```

### Binary Size Optimization

Add to `Cargo.toml`:

```toml
[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link-time optimization
codegen-units = 1   # Better optimization
strip = true        # Strip symbols
```

Then rebuild:
```bash
cargo build --release
# Binary shrinks from ~15MB to ~8MB
```
