# Convey

A lightweight, native macOS voice-to-text application built entirely in Rust. Press and hold Globe/Fn to record your voice, release to transcribe, and have the text automatically pasted into any application.

**Philosophy**: Fast, offline, native. No Electron, no cloud dependencies, no subscriptions.

## Features

- **Globe/Fn key activation** - Press and hold to record, release to transcribe
- **Offline transcription** - Powered by Whisper.cpp (base model bundled, 141MB)
- **Auto-paste** - Text automatically typed into your active application
- **Persistent history** - All transcriptions saved locally with SQLite
- **Voice commands** - Say "and press enter" to automatically press Enter after pasting
- **Solarized theme** - Carefully designed UI using the Solarized color palette
- **Native performance** - Pure Rust with iced GUI framework, no web technologies

## Installation

### For Users

1. **Download** the latest DMG from releases
2. **Install whisper-cli**:
   ```bash
   brew install whisper-cpp
   ```
3. **Drag Convey.app to Applications**
4. **Launch** and grant permissions:
   - Microphone access (for recording)
   - Accessibility access (for auto-paste)
5. **Press Globe/Fn** to start recording

### For Developers

```bash
git clone https://github.com/inmobi-marketing/convey.git
cd convey

# Download the Whisper model (141MB)
./scripts/download_model.sh

# Install whisper-cli
brew install whisper-cpp

# Build and run
cargo run --release
```

See [ARCHITECTURE.md](ARCHITECTURE.md) for technical details and [BUILD.md](BUILD.md) for build instructions.

## Usage

1. **Start Recording**: Press **Globe** or **Fn** key
2. **Stop Recording**: Release the key
3. **Result**: Text is automatically transcribed and pasted into your active app

### Settings

Access settings in the app window:
- **Auto paste**: Toggle automatic pasting of transcriptions
- **Recognize 'and press enter'**: Say "and press enter" to automatically press Enter after pasting

### Voice Commands

- Say normal text to transcribe it
- End with "and press enter" or "then press enter" to automatically press Enter after pasting

## Building

See [BUILD.md](BUILD.md) for complete build and distribution instructions.

Quick build:
```bash
./scripts/build_app.sh        # Creates build/Convey.app
./scripts/make_dmg.sh         # Creates build/Convey-0.1.0.dmg
```

## Project Structure

```
convey/
├── Cargo.toml          # Rust project configuration
├── assets/             # Design assets
│   └── icon.svg       # Source icon (Solarized colors)
├── build/              # Build outputs (gitignored)
│   ├── Convey.app     # macOS app bundle
│   └── Convey-0.1.0.dmg # Distribution DMG
├── resources/          # Bundled resources
│   └── models/
│       └── ggml-base.bin # Whisper base model (141MB)
├── scripts/            # Build and utility scripts
│   ├── build_app.sh   # Build .app bundle
│   ├── make_dmg.sh    # Create DMG
│   └── generate_icons.sh # Generate icons from SVG
└── src/                # Rust source code
    ├── main.rs        # Application entry point
    ├── audio.rs       # Audio capture (cpal + hound)
    ├── clipboard.rs   # Clipboard & auto-paste (enigo)
    ├── database.rs    # SQLite history (rusqlite)
    ├── fn_key_monitor.rs # Globe/Fn key detection (Obj-C runtime)
    ├── notch.rs       # Waveform overlay (NSPanel)
    ├── services/      # Thread-safe service layer
    ├── storage.rs     # Settings & Keychain integration
    ├── ui/            # iced GUI
    │   ├── mod.rs    # Main application UI
    │   └── theme.rs  # Solarized color palette
    ├── whisper.rs     # Whisper.cpp CLI integration
    └── workflow.rs    # Recording state machine
```

## Development

```bash
# Run with logging
RUST_LOG=info cargo run --release

# Build release binary
cargo build --release

# Format code
cargo fmt

# Check for errors
cargo check
```

## Troubleshooting

**"Unable to locate whisper-cli"**
→ Install via `brew install whisper-cpp`

**No transcription output**
→ The model should be bundled. Check `resources/models/ggml-base.bin` exists

**Overlay not showing**
→ This is a known issue - the overlay is created but may not be visible. The app still works perfectly without it!

**Auto-paste not working**
→ Grant Accessibility permission: `System Settings → Privacy & Security → Accessibility → Add Convey`

**Globe/Fn key not responding**
→ Ensure Convey is running and has Accessibility permission

## Technical Details

- **Language**: Pure Rust
- **GUI**: iced 0.12
- **Audio**: cpal + hound
- **Transcription**: whisper.cpp via CLI
- **Database**: SQLite (rusqlite)
- **Keychain**: macOS Keychain for API keys
- **Size**: ~156MB (app + model)

## License

MIT License - See LICENSE file for details

---

**Made with ❤️  by [Naren Laxmidas](https://github.com/narennaik)**

Built with Rust 🦀 for a fast, native macOS transcription experience.
