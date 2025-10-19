# Convey Architecture

## Overview

Convey is a native macOS voice-to-text application built entirely in Rust. It uses the iced GUI framework for the interface and integrates directly with macOS system APIs via Objective-C runtime bindings.

**Key Design Principle**: Native, fast, offline. No web technologies, no cloud dependencies, no Electron.

## Technology Stack

### Core Technologies

| Component | Technology | Purpose |
|-----------|-----------|---------|
| **GUI Framework** | [iced](https://github.com/iced-rs/iced) 0.12 | Native Rust GUI with Elm-architecture |
| **Audio Capture** | [cpal](https://github.com/RustAudio/cpal) 0.15 | Cross-platform audio I/O |
| **Audio Storage** | [hound](https://github.com/ruuda/hound) 3.5 | WAV file encoding |
| **Transcription** | [whisper.cpp](https://github.com/ggerganov/whisper.cpp) | Local speech-to-text via CLI |
| **Database** | [rusqlite](https://github.com/rusqlite/rusqlite) 0.31 | SQLite for transcription history |
| **Keyboard Simulation** | [enigo](https://github.com/enigo-rs/enigo) 0.2 | Auto-paste functionality |
| **macOS Integration** | [objc](https://github.com/SSheldon/rust-objc) 0.2 | Objective-C runtime bindings |

### Why These Choices?

**iced over egui/druid/gtk-rs**
- Elm architecture provides clean state management
- Excellent styling capabilities (needed for Solarized theme)
- Active development and good macOS support
- Pure Rust with no C++ dependencies

**whisper.cpp CLI over Rust bindings**
- Simpler integration - no complex build dependencies
- Users install via Homebrew (`brew install whisper-cpp`)
- Model bundled with app, easy to swap/upgrade
- Clear separation of concerns

**Direct Objective-C over higher-level bindings**
- Fine-grained control over macOS features
- No middleware overhead
- Access to private/undocumented APIs when needed (Globe key monitoring)
- Smaller binary size

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                        Convey Application                    │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────┐         ┌───────────────┐                │
│  │   UI Layer   │         │   Services    │                │
│  │   (iced)     │◄───────►│  (Arc/Mutex)  │                │
│  └──────────────┘         └───────────────┘                │
│         │                         │                          │
│         │                         ▼                          │
│         │              ┌──────────────────────┐            │
│         │              │  Storage Services    │            │
│         │              ├──────────────────────┤            │
│         │              │ • Database (SQLite)  │            │
│         │              │ • Settings (Keychain)│            │
│         │              │ • History            │            │
│         │              └──────────────────────┘            │
│         │                                                   │
│         ▼                                                   │
│  ┌──────────────────────────────────────────┐             │
│  │          Workflow State Machine           │             │
│  │  (Idle → Recording → Transcribing → Done) │             │
│  └──────────────────────────────────────────┘             │
│         │                                                   │
│         ├──────────┬──────────┬─────────────┬────────────┐│
│         ▼          ▼          ▼             ▼            ││
│  ┌──────────┐┌──────────┐┌──────────┐┌────────────┐   ││
│  │  Audio   ││ Whisper  ││Clipboard ││  Fn Key    │   ││
│  │ Capture  ││Integration││ Manager  ││  Monitor   │   ││
│  │ (cpal)   ││(CLI exec)││ (enigo)  ││  (objc)    │   ││
│  └──────────┘└──────────┘└──────────┘└────────────┘   ││
│                                                          ││
└──────────────────────────────────────────────────────────┘│
                          │                                  │
                          ▼                                  │
            ┌──────────────────────────┐                   │
            │     macOS System APIs     │                   │
            ├──────────────────────────┤                   │
            │ • Audio (CoreAudio)      │                   │
            │ • Accessibility          │                   │
            │ • Keychain               │                   │
            │ • NSPanel/NSWindow       │                   │
            │ • IOKit (Fn key events)  │                   │
            └──────────────────────────┘                   │
```

## Key Components

### 1. UI Layer (`src/ui/`)

**Framework**: iced 0.12 with Elm architecture

**Design Philosophy**: Functional, declarative UI with immutable state updates.

```rust
// Simplified flow
Application::new()
    .subscription() // Key events, audio updates
    .update()       // Pure state transformations
    .view()         // Declarative UI rendering
```

**Theme** (`src/ui/theme.rs`)
- Full Solarized color palette implementation
- Consistent colors across all UI elements:
  - Background: Base03 (#002b36)
  - Surface: Base02 (#073642)
  - Text: Base0 (#839496)
  - Accent: Blue (#268bd2)

**Why Solarized?**
- Designed for long-term screen use
- Reduces eye strain
- Professional, cohesive look
- Warm color temperature

### 2. Workflow State Machine (`src/workflow.rs`)

**States**:
```
Idle → Recording → Transcribing → Done
  ▲                                   │
  └───────────────────────────────────┘
```

**State Transitions**:
- `Idle → Recording`: Globe/Fn key pressed
- `Recording → Transcribing`: Globe/Fn key released
- `Transcribing → Done`: Whisper completes
- `Done → Idle`: Auto-transition after paste

**Design Decision**: Explicit state machine prevents race conditions and makes behavior predictable.

### 3. Audio Capture (`src/audio.rs`)

**Technology**: cpal (Cross-Platform Audio Library)

**Flow**:
1. Request default input device
2. Configure: 16kHz, mono, f32 samples (Whisper requirements)
3. Stream to ring buffer
4. Save as WAV when recording stops

**Why 16kHz mono?**
- Whisper.cpp requirement
- Reduces file size
- Sufficient for human speech
- Faster transcription

**Error Handling**: Graceful fallback if audio device unavailable.

### 4. Transcription (`src/whisper.rs`)

**Integration**: CLI execution, not Rust bindings

**Model Selection**:
- **Bundled**: `ggml-base.bin` (141MB)
- **Accuracy**: ~95% for clear speech
- **Speed**: ~1-2s for 30s audio on M1/M2
- **Language**: Auto-detect (supports 99 languages)

**Model Search Priority**:
1. Bundled resources (`/Applications/Convey.app/Contents/Resources/resources/models/`)
2. Development path (`resources/models/`)
3. User data directory (legacy support)

**CLI Execution**:
```bash
whisper-cli \
  -m /path/to/ggml-base.bin \
  -f /tmp/recording.wav \
  --no-timestamps \
  --language auto
```

**Why CLI over bindings?**
- No complex C++ build dependencies
- Easy to upgrade whisper.cpp version
- Clear separation of concerns
- User controls whisper-cli installation

### 5. Globe/Fn Key Monitoring (`src/fn_key_monitor.rs`)

**Challenge**: macOS doesn't expose Globe/Fn key via standard APIs

**Solution**: Direct IOKit event tap using Objective-C runtime

**Implementation**:
```rust
// Create event tap at CGEventTapLocation::HID level
CGEvent::new_event_tap(
    kCGHIDEventTap,  // HID level (before key remapping)
    kCGHeadInsertEventTap,
    kCGEventTapOptionDefault,
    kCGEventMaskForAllEvents,
    callback
)
```

**Key Detection**:
- Listen for HID usage page 0x07, usage 0xE3 (Left GUI/Globe)
- Filter to only Globe key presses/releases
- Send to main thread via channel

**Why HID-level tap?**
- Only way to detect Globe key before macOS remaps it
- Requires Accessibility permission
- More reliable than NSEvent monitoring

### 6. Auto-Paste (`src/clipboard.rs`)

**Technology**: enigo for keyboard simulation

**Flow**:
1. Focus active application (preserve user context)
2. Type out transcription character-by-character
3. If "and press enter" detected, simulate Enter key

**Design Decision**: Type instead of paste
- More reliable across applications
- Works with apps that block clipboard
- No clipboard pollution
- Feels more natural

**Enter Detection**:
```rust
if text.trim_end().ends_with("and press enter")
    || text.trim_end().ends_with("then press enter") {
    // Simulate Enter after typing
}
```

### 7. History & Storage (`src/database.rs`, `src/storage.rs`)

**Database Schema**:
```sql
CREATE TABLE transcriptions (
    id INTEGER PRIMARY KEY,
    text TEXT NOT NULL,
    created_at TEXT NOT NULL,
    audio_duration_ms INTEGER
);
```

**Settings Storage**:
- User preferences: JSON file in `~/Library/Application Support/com.narennaik.convey/`
- API keys: macOS Keychain (secure, encrypted)

**Why SQLite?**
- Lightweight, embedded
- ACID guarantees
- Easy to backup/export
- No server needed

### 8. Thread-Safe Services (`src/services/`)

**Pattern**: Arc<Mutex<T>> for shared state

**Services**:
- `HistoryService`: Thread-safe access to transcription history
- `SettingsService`: Concurrent read/write of user preferences

**Design Decision**: Service layer
- Decouples UI from storage
- Enables testing without database
- Thread-safe by design
- Single source of truth

## Data Flow

### Recording → Transcription → Paste

```
1. User presses Globe/Fn key
   └─> fn_key_monitor.rs detects → sends Message::StartRecording

2. UI updates → Workflow transitions to Recording
   └─> audio.rs starts capturing to buffer

3. User releases Globe/Fn key
   └─> fn_key_monitor.rs detects → sends Message::StopRecording

4. Workflow transitions to Transcribing
   └─> audio.rs saves WAV file
   └─> whisper.rs executes CLI with model + audio file

5. Whisper completes
   └─> Sends Message::TranscriptionComplete(text)
   └─> Workflow transitions to Done

6. Auto-paste enabled?
   └─> clipboard.rs types out text
   └─> Detects "and press enter" → simulates Enter

7. Save to history
   └─> database.rs inserts record

8. Workflow transitions back to Idle
```

## Design Decisions

### 1. Pure Rust, No Electron

**Why Rust?**
- Memory safety without garbage collection
- Native performance
- Excellent concurrency primitives (Arc, Mutex, channels)
- Growing macOS ecosystem

**Why not Electron?**
- 200MB+ base size vs ~15MB for Rust
- High memory usage (~100MB idle vs ~30MB for Convey)
- Slower startup time
- Unnecessary complexity for a system tool

### 2. iced GUI Framework

**Alternatives Considered**:
- **egui**: Immediate mode, harder to style
- **gtk-rs**: Heavy C dependencies, not native-looking on macOS
- **native-windows-gui**: Windows-only
- **druid**: Development stalled

**iced Advantages**:
- Elm architecture (predictable state management)
- Excellent styling system
- Pure Rust, minimal dependencies
- Active development

### 3. Bundled Whisper Model

**Why Bundle?**
- Zero-friction user experience
- Guaranteed version compatibility
- Works offline immediately
- No download/setup step

**Trade-off**: 141MB app size, but acceptable for utility app

### 4. CLI Integration vs Bindings

**whisper.cpp Rust bindings exist, why not use them?**
- Build complexity (C++ compiler, CMake, platform-specific)
- Harder to upgrade whisper.cpp
- Larger binary
- More potential failure points

**CLI approach**:
- User installs once: `brew install whisper-cpp`
- Simple subprocess execution
- Easy to test and debug
- Model path fully controlled by app

### 5. Solarized Theme

**Why Solarized over system theme?**
- Consistent cross-platform design
- Carefully crafted for readability
- Professional appearance
- Distinct brand identity

**Implementation**: Full color palette in `ui/theme.rs`, no hardcoded colors

## Performance Characteristics

### Metrics (M1 MacBook Air)

| Operation | Time | Notes |
|-----------|------|-------|
| Cold start | ~0.5s | Binary to window visible |
| Recording start latency | <50ms | Key press to recording indicator |
| Audio buffering | Real-time | No dropped samples |
| Transcription (30s audio) | ~1.5s | Base model, M1 Neural Engine |
| Auto-paste (100 chars) | ~300ms | Character-by-character typing |
| Database write | <5ms | SQLite insert |
| Memory usage (idle) | ~30MB | Resident set size |
| Memory usage (recording) | ~50MB | Audio buffer in RAM |

### Optimization Strategies

1. **Lazy loading**: Whisper model only loaded when needed
2. **Efficient audio buffering**: Ring buffer, fixed size
3. **Async transcription**: Non-blocking UI during processing
4. **Minimal allocations**: Reuse buffers where possible
5. **SQLite WAL mode**: Concurrent reads during writes

## Security & Privacy

### Data Storage

- **Audio files**: Temporary, deleted after transcription
- **Transcriptions**: Local SQLite database, never leave device
- **API keys**: macOS Keychain (AES-256 encrypted)
- **No analytics**: Zero telemetry, no phone-home

### Permissions Required

| Permission | Purpose | Required? |
|-----------|---------|-----------|
| Microphone | Audio capture | Yes |
| Accessibility | Fn key monitoring, auto-paste | Yes |
| Full Disk Access | No | Never |
| Network | No | Never (fully offline) |

### Threat Model

**In Scope**:
- Local data protection (Keychain for sensitive data)
- No plaintext secrets in memory
- Secure defaults (auto-paste can be disabled)

**Out of Scope**:
- Protection against malicious local user
- Screen recording protection
- Anti-tamper mechanisms

## Testing Strategy

### Unit Tests
- Audio processing pipelines
- Database queries
- State machine transitions
- Text processing (e.g., "and press enter" detection)

### Integration Tests
- Whisper CLI execution
- SQLite schema migrations
- Keychain storage/retrieval

### Manual Testing
- Cross-device compatibility (Intel/Apple Silicon)
- Audio device switching
- Permission handling
- Long recording sessions

**Run tests**:
```bash
cargo test
cargo test --release  # Performance tests
```

## Build System

### Development Build
```bash
cargo run --release
```

### Production Build
```bash
./build_app.sh  # Creates build/Convey.app
./make_dmg.sh   # Creates build/Convey-0.1.0.dmg
```

### Build Artifacts

| File | Size | Purpose |
|------|------|---------|
| `target/release/convey` | ~15MB | Binary executable |
| `build/Convey.app` | ~157MB | App bundle (binary + model + resources) |
| `build/Convey-0.1.0.dmg` | ~138MB | Compressed DMG (UDZO) |

### Icon Pipeline

1. Edit `assets/icon.svg` (Solarized colors)
2. Run `cd assets && ./generate_icons.sh`
3. Generates PNG sizes (16×16 to 1024×1024)
4. Creates `Convey.icns` for macOS
5. Build script copies to app bundle

## Future Enhancements

### Potential Features
- [ ] Hotkey customization (beyond Globe/Fn)
- [ ] Multiple Whisper models (tiny/small/medium/large)
- [ ] Language selection override
- [ ] Export history to text/JSON
- [ ] Custom voice commands (e.g., "new line", "delete that")
- [ ] Local LLM post-processing for cleanup
- [ ] Snippets/text expansion

### Technical Improvements
- [ ] Reduce binary size (strip debug symbols, LTO)
- [ ] CoreML acceleration for transcription
- [ ] Streaming transcription (real-time as you speak)
- [ ] Better error recovery (retry failed transcriptions)
- [ ] Automated testing on CI

## Contributing

See [BUILD.md](BUILD.md) for development setup.

**Code Style**:
- `cargo fmt` before commit
- `cargo clippy` with zero warnings
- Descriptive commit messages

**Architecture Changes**:
- Discuss major changes in issues first
- Update this document for significant architectural shifts
- Include rationale for design decisions

## References

- [iced documentation](https://docs.rs/iced/)
- [whisper.cpp repository](https://github.com/ggerganov/whisper.cpp)
- [Solarized color scheme](https://ethanschoonover.com/solarized/)
- [macOS IOKit](https://developer.apple.com/documentation/iokit)
- [Objective-C Runtime](https://developer.apple.com/documentation/objectivec/objective-c_runtime)
