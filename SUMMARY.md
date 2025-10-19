# Convey - Project Summary

## What is Convey?

Convey is a lightweight, native macOS voice-to-text application built entirely in Rust. It allows users to press and hold the Globe/Fn key to record audio, which is then automatically transcribed using Whisper.cpp and pasted into any application.

**Core Philosophy**: Fast, offline, native. No Electron, no cloud dependencies, no subscriptions.

## Key Achievements

### 1. Pure Rust Implementation
- **No Tauri**: Despite the misleading directory name, this is a pure Rust + iced application
- **No Electron**: Native performance with ~30MB memory footprint vs ~100MB+ for Electron
- **No web technologies**: Direct system integration via Objective-C runtime bindings

### 2. Native macOS Integration
- Globe/Fn key monitoring via IOKit HID-level event tap
- macOS Keychain for secure API key storage
- NSPanel/NSWindow for native overlays
- Accessibility API integration for auto-paste

### 3. Offline AI
- Whisper base model (141MB) bundled with the app
- No internet required for transcription
- Privacy-focused: audio never leaves the device

### 4. User Experience
- One-key operation: Press Globe/Fn to record
- Auto-paste: Text automatically typed into active app
- Voice commands: Say "and press enter" to simulate Enter key
- Persistent history: All transcriptions saved locally in SQLite

## Technical Highlights

### Architecture

```
┌─────────────── Pure Rust Stack ──────────────────┐
│                                                   │
│  iced GUI (Elm architecture)                     │
│         │                                         │
│         ├──→ Services (Arc<Mutex<T>>)            │
│         │         │                               │
│         │         ├──→ SQLite (rusqlite)         │
│         │         └──→ Keychain (security-framework) │
│         │                                         │
│         ├──→ Workflow State Machine              │
│         │                                         │
│         └──→ Native Integration                  │
│                 ├──→ Audio (cpal)                │
│                 ├──→ Whisper CLI (subprocess)    │
│                 ├──→ Clipboard (enigo)           │
│                 └──→ Fn Key (objc runtime)       │
│                                                   │
└───────────────────────────────────────────────────┘
```

### Design Decisions

| Decision | Rationale |
|----------|-----------|
| **iced over egui/gtk** | Better styling, Elm architecture, pure Rust |
| **Whisper CLI over bindings** | Simpler build, user controls version, easy to upgrade |
| **Bundled model** | Zero-friction UX, works offline immediately |
| **Solarized theme** | Professional, reduces eye strain, warm colors |
| **SQLite for history** | Embedded, ACID guarantees, easy backup |
| **Globe/Fn key** | Native macOS gesture, non-intrusive |

### Performance Metrics (M1 MacBook Air)

| Metric | Value |
|--------|-------|
| Cold start | ~0.5s |
| Memory (idle) | ~30MB |
| Memory (recording) | ~50MB |
| Transcription (30s audio) | ~1.5s |
| Binary size | ~15MB |
| App bundle size | ~157MB (includes 141MB model) |
| DMG size | ~138MB (compressed) |

## Project Structure

### Clean Organization

```
convey/
├── Cargo.toml          # Rust configuration
├── README.md           # User-facing documentation
├── ARCHITECTURE.md     # Technical deep-dive
├── BUILD.md            # Build instructions
├── DISTRIBUTION.md     # Distribution guide
├── SUMMARY.md          # This file
│
├── assets/             # Design assets
│   └── icon.svg       # Solarized colors (#268bd2, #2aa198, #fdf6e3)
│
├── build/              # Build outputs (gitignored)
│   ├── Convey.app     # macOS app bundle
│   └── Convey-0.1.0.dmg
│
├── resources/          # Bundled resources
│   └── models/
│       └── ggml-base.bin  # Whisper model
│
├── scripts/            # Build automation
│   ├── build_app.sh   # Creates .app bundle
│   ├── make_dmg.sh    # Creates DMG
│   └── generate_icons.sh # SVG → PNG/ICNS pipeline
│
└── src/                # Rust source
    ├── main.rs        # Entry point
    ├── audio.rs       # cpal audio capture
    ├── clipboard.rs   # enigo keyboard simulation
    ├── database.rs    # SQLite history
    ├── fn_key_monitor.rs # IOKit event tap
    ├── notch.rs       # NSPanel overlay
    ├── services/      # Thread-safe service layer
    ├── storage.rs     # Settings + Keychain
    ├── ui/
    │   ├── mod.rs    # iced application
    │   └── theme.rs  # Solarized palette
    ├── whisper.rs     # CLI integration
    └── workflow.rs    # State machine
```

## Documentation

### Complete Documentation Set

1. **README.md** - User-facing quick start, installation, usage
2. **ARCHITECTURE.md** - Technical architecture, design decisions, data flow
3. **BUILD.md** - Build instructions, development guide, troubleshooting
4. **DISTRIBUTION.md** - Code signing, notarization, DMG creation
5. **SUMMARY.md** - This file, high-level overview

### Key Documentation Features

- **Design rationale**: Why each technology choice was made
- **Performance metrics**: Real-world measurements
- **Troubleshooting**: Common issues and solutions
- **Advanced topics**: Custom models, profiling, optimization
- **No redundancy**: Each doc has a clear purpose

## Security & Privacy

### Privacy-First Design

- **No telemetry**: Zero phone-home, no analytics
- **Offline-first**: No internet required for core functionality
- **Local storage**: Everything stays on device
- **Secure secrets**: API keys in macOS Keychain (AES-256)
- **Temporary audio**: Deleted immediately after transcription

### Required Permissions

| Permission | Purpose | Why? |
|-----------|---------|------|
| Microphone | Record audio | Core feature |
| Accessibility | Globe key + auto-paste | UX convenience |

**NOT Required**: Network, Full Disk Access, Camera

## Build System

### Automated Pipeline

```bash
# Icon generation (SVG → PNG → ICNS)
./scripts/generate_icons.sh

# App build (Rust → .app bundle)
./scripts/build_app.sh

# DMG creation (.app → DMG with Applications symlink)
./scripts/make_dmg.sh
```

### Build Outputs

| File | Size | Purpose |
|------|------|---------|
| `target/release/convey` | ~15MB | Executable |
| `build/Convey.app` | ~157MB | App bundle (includes model) |
| `build/Convey-0.1.0.dmg` | ~138MB | Compressed distribution |

## UI/UX

### Solarized Theme

All colors from the official Solarized palette:

```rust
// Background
BASE03: #002b36  // Deep blue-gray
BASE02: #073642  // Dark blue-gray

// Content
BASE0:  #839496  // Body text
BASE1:  #93a1a1  // Optional emphasis

// Accent
BLUE:   #268bd2  // Primary accent
CYAN:   #2aa198  // Secondary accent
```

### Features

- **Clean, minimal UI**: Focus on the recording button
- **Persistent history**: Recent transcriptions always visible
- **Settings inline**: No separate preferences window
- **Error handling**: Clear error messages
- **Attribution footer**: "Made with ❤️ by Naren Laxmidas"

## Workflow

### User Flow

```
1. User presses Globe/Fn
   ↓
2. Audio recording starts (visual indicator)
   ↓
3. User releases Globe/Fn
   ↓
4. Recording stops → saved as WAV
   ↓
5. Whisper CLI processes audio
   ↓
6. Transcription complete
   ↓
7. Text auto-pasted (if enabled)
   ↓
8. Saved to history
   ↓
9. Ready for next recording
```

### State Machine

```
Idle ←──────────────────────────┐
 │                               │
 │ (Globe pressed)               │
 ↓                               │
Recording                        │
 │                               │
 │ (Globe released)              │
 ↓                               │
Transcribing                     │
 │                               │
 │ (Whisper complete)            │
 ↓                               │
Done ────────────────────────────┘
     (auto-transition)
```

## Dependencies

### Core Dependencies

```toml
iced = "0.12"              # GUI framework
cpal = "0.15"              # Audio I/O
hound = "3.5"              # WAV encoding
rusqlite = "0.31"          # SQLite
enigo = "0.2"              # Keyboard simulation
objc = "0.2"               # macOS integration
keyring = "2.3"            # Keychain access
```

### External Dependencies

- **whisper-cli**: User installs via Homebrew
- **ggml-base.bin**: Bundled in app (141MB)

## Future Enhancements

### Potential Features
- [ ] Hotkey customization
- [ ] Multiple model support (tiny/small/medium/large)
- [ ] Language override
- [ ] Export history (text/JSON)
- [ ] Custom voice commands
- [ ] Local LLM post-processing
- [ ] CoreML acceleration

### Technical Improvements
- [ ] Binary size reduction (LTO, strip)
- [ ] Streaming transcription (real-time)
- [ ] Better error recovery
- [ ] CI/CD automation

## Lessons Learned

### What Went Well

1. **Pure Rust pays off**: No build complexity, fast binaries, good macOS support
2. **iced is mature**: Elm architecture made state management clean
3. **CLI integration simple**: Whisper.cpp via subprocess works great
4. **Bundled model**: Users love zero-setup experience

### Challenges Overcome

1. **Globe/Fn key detection**: Required HID-level event tap
2. **NSPanel visibility**: Notch overlay has known visibility issues
3. **Auto-paste reliability**: Character-by-character typing more reliable than clipboard
4. **Model bundling**: 141MB addition, but worth it for UX

### Design Evolution

- Started considering Tauri → chose pure Rust
- Started with separate settings window → moved to inline
- Started with clipboard paste → switched to typing
- Started with complex overlay → simplified to transparent waveform

## Attribution

**Creator**: Naren Laxmidas
**GitHub**: [@narennaik](https://github.com/narennaik)
**License**: MIT

Made with ❤️ for the macOS community.

## Quick Reference

### For Users

```bash
# Install
brew install whisper-cpp
# Drag Convey.app to Applications
# Press Globe/Fn to start!
```

### For Developers

```bash
# Clone
git clone <repo>
cd convey

# Build
./scripts/build_app.sh

# Run
cargo run --release
```

### For Contributors

```bash
# Format
cargo fmt

# Lint
cargo clippy -- -D warnings

# Test
cargo test

# Build docs
cargo doc --open
```

---

**Built with Rust 🦀 | Native macOS Performance | Privacy-Focused Design**
