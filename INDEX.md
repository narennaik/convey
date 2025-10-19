# Convey Documentation Index

Welcome to the Convey documentation! This guide will help you navigate all available documentation based on your needs.

## 📚 Documentation Map

### For Users

**Start Here**: [README.md](README.md)
- What is Convey?
- Installation instructions
- Basic usage
- Quick start guide

**Troubleshooting**: [README.md#troubleshooting](README.md#troubleshooting)
- Common issues and solutions
- Permission problems
- Audio/transcription issues

### For Developers

**Getting Started**: [BUILD.md](BUILD.md)
- Prerequisites and setup
- Development workflow
- Build instructions
- Testing

**Deep Dive**: [ARCHITECTURE.md](ARCHITECTURE.md)
- System architecture
- Design decisions and rationale
- Component breakdown
- Data flow diagrams
- Performance characteristics

**Distribution**: [DISTRIBUTION.md](DISTRIBUTION.md)
- Creating releases
- Code signing
- macOS notarization
- DMG creation

### For Contributors

**Code Quality**: [BUILD.md#code-quality](BUILD.md#code-quality)
- Formatting standards
- Linting rules
- Testing requirements

**Architecture**: [ARCHITECTURE.md](ARCHITECTURE.md)
- Understand the codebase
- Design patterns used
- Why certain decisions were made

### Quick Reference

**Project Overview**: [SUMMARY.md](SUMMARY.md)
- High-level summary
- Key achievements
- Technical highlights
- Quick reference commands

## 📖 Documentation Files

| File | Purpose | Audience |
|------|---------|----------|
| [README.md](README.md) | User-facing introduction, installation, usage | End users |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Technical deep-dive into design and implementation | Developers |
| [BUILD.md](BUILD.md) | Build instructions, development guide | Developers/Contributors |
| [DISTRIBUTION.md](DISTRIBUTION.md) | Release and distribution process | Maintainers |
| [SUMMARY.md](SUMMARY.md) | High-level project summary | Everyone |
| [INDEX.md](INDEX.md) | This file - documentation navigation | Everyone |

## 🎯 Common Tasks

### I want to...

**...install and use Convey**
→ [README.md#installation](README.md#installation)

**...understand how it works**
→ [ARCHITECTURE.md](ARCHITECTURE.md)

**...build from source**
→ [BUILD.md#quick-start](BUILD.md#quick-start)

**...contribute code**
→ [BUILD.md#development](BUILD.md#development) + [ARCHITECTURE.md](ARCHITECTURE.md)

**...create a release**
→ [BUILD.md#distribution](BUILD.md#distribution) + [DISTRIBUTION.md](DISTRIBUTION.md)

**...customize the icon**
→ [BUILD.md#regenerate-icons](BUILD.md#regenerate-icons)

**...use a different Whisper model**
→ [BUILD.md#custom-whisper-model](BUILD.md#custom-whisper-model)

**...understand design decisions**
→ [ARCHITECTURE.md#design-decisions](ARCHITECTURE.md#design-decisions)

**...get a quick overview**
→ [SUMMARY.md](SUMMARY.md)

## 🔍 Find Information By Topic

### Installation & Setup
- [User installation](README.md#for-users)
- [Developer setup](README.md#for-developers)
- [Prerequisites](BUILD.md#prerequisites)

### Usage
- [Basic usage](README.md#usage)
- [Settings](README.md#settings)
- [Voice commands](README.md#voice-commands)

### Development
- [Running locally](BUILD.md#run-in-development-mode)
- [Testing](BUILD.md#testing)
- [Code quality](BUILD.md#code-quality)

### Architecture
- [Technology stack](ARCHITECTURE.md#technology-stack)
- [Component overview](ARCHITECTURE.md#key-components)
- [Data flow](ARCHITECTURE.md#data-flow)
- [Performance](ARCHITECTURE.md#performance-characteristics)

### Building
- [Quick build](BUILD.md#quick-start)
- [Clean build](BUILD.md#clean-build)
- [Advanced options](BUILD.md#advanced)

### Distribution
- [Creating releases](DISTRIBUTION.md#building-for-distribution)
- [Code signing](DISTRIBUTION.md#code-signing-optional-but-recommended)
- [DMG creation](BUILD.md#create-dmg-for-distribution)

### Troubleshooting
- [Build errors](BUILD.md#build-errors)
- [Runtime issues](BUILD.md#runtime-issues)
- [Common problems](README.md#troubleshooting)

## 🏗️ Project Structure

Quick reference to important directories:

```
convey/
├── README.md              ← Start here (users)
├── ARCHITECTURE.md        ← Technical details (developers)
├── BUILD.md               ← Build guide (developers)
├── DISTRIBUTION.md        ← Release process (maintainers)
├── SUMMARY.md             ← Quick overview (everyone)
├── INDEX.md               ← This file
│
├── Cargo.toml             ← Rust configuration
├── .gitignore             ← Git exclusions
│
├── assets/                ← Design assets
│   └── icon.svg          ← Source icon (Solarized colors)
│
├── build/                 ← Build outputs (gitignored)
│
├── resources/             ← Bundled resources
│   └── models/
│       └── ggml-base.bin ← Whisper model
│
├── scripts/               ← Build scripts
│   ├── build_app.sh      ← Build .app
│   ├── make_dmg.sh       ← Create DMG
│   └── generate_icons.sh ← Generate icons
│
└── src/                   ← Rust source code
    ├── main.rs           ← Entry point
    ├── ui/               ← iced GUI
    ├── services/         ← Business logic
    └── ...               ← Other modules
```

## 🚀 Quick Start Paths

### Path 1: User
1. Read [README.md](README.md)
2. Follow [installation steps](README.md#for-users)
3. Check [usage guide](README.md#usage)
4. If issues: [troubleshooting](README.md#troubleshooting)

### Path 2: Developer (First Time)
1. Skim [README.md](README.md) for context
2. Review [ARCHITECTURE.md](ARCHITECTURE.md) to understand design
3. Follow [BUILD.md#prerequisites](BUILD.md#prerequisites)
4. Run `cargo run --release`
5. Read [BUILD.md#development](BUILD.md#development)

### Path 3: Contributor
1. Read [ARCHITECTURE.md](ARCHITECTURE.md) fully
2. Set up dev environment: [BUILD.md#development](BUILD.md#development)
3. Review code quality standards: [BUILD.md#code-quality](BUILD.md#code-quality)
4. Make changes
5. Test: `cargo test && cargo clippy`
6. Submit PR

### Path 4: Maintainer (Release)
1. Review [DISTRIBUTION.md](DISTRIBUTION.md)
2. Run `./scripts/build_app.sh`
3. Test thoroughly
4. Run `./scripts/make_dmg.sh`
5. Sign and notarize (see [DISTRIBUTION.md](DISTRIBUTION.md))
6. Create GitHub release

## 💡 Tips

### For Reading Documentation

- **Start broad, go deep**: Begin with README, then SUMMARY, then ARCHITECTURE
- **Use search**: All docs are markdown - search for keywords
- **Follow links**: Documents cross-reference each other extensively
- **Check examples**: Most docs include code examples and commands

### For Contributing

- **Understand the architecture first**: Read ARCHITECTURE.md before coding
- **Follow conventions**: Check existing code for patterns
- **Document changes**: Update relevant docs when changing code
- **Test thoroughly**: Run all checks before submitting

## 📝 Documentation Standards

All Convey documentation follows these principles:

1. **Clear purpose**: Each doc has a specific audience and goal
2. **No redundancy**: Information appears in exactly one place
3. **Cross-referenced**: Related info is linked, not duplicated
4. **Examples included**: Commands and code snippets throughout
5. **Up-to-date**: Docs updated alongside code changes

## 🔗 External Resources

- [iced documentation](https://docs.rs/iced/)
- [whisper.cpp repository](https://github.com/ggerganov/whisper.cpp)
- [Solarized color scheme](https://ethanschoonover.com/solarized/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [macOS IOKit](https://developer.apple.com/documentation/iokit)

## 📬 Getting Help

- **Questions about usage**: Check [README.md#troubleshooting](README.md#troubleshooting)
- **Technical questions**: Read [ARCHITECTURE.md](ARCHITECTURE.md)
- **Build issues**: See [BUILD.md#troubleshooting](BUILD.md#troubleshooting)
- **Still stuck**: Open an issue on GitHub

---

**Made with ❤️ by Naren Laxmidas**

*Last updated: 2025-01-20*
