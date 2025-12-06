<p align="center">
  <img src="docs/logo.svg" alt="Thunderlink Logo" width="120" height="120">
</p>

<h1 align="center">⚡ Thunderlink</h1>

<p align="center">
  <strong>Lightning-fast file transfers over Thunderbolt & USB-C</strong>
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#installation">Installation</a> •
  <a href="#usage">Usage</a> •
  <a href="#development">Development</a> •
  <a href="#contributing">Contributing</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/platforms-macOS%20%7C%20Windows%20%7C%20Linux-blue?style=flat-square" alt="Platforms">
  <img src="https://img.shields.io/badge/license-MIT-green?style=flat-square" alt="License">
  <img src="https://img.shields.io/badge/rust-1.70%2B-orange?style=flat-square" alt="Rust">
  <img src="https://img.shields.io/badge/tauri-2.0-blueviolet?style=flat-square" alt="Tauri">
</p>

---

**Thunderlink** transforms your Thunderbolt or USB-C cable into a high-speed data highway between two computers. No cloud, no network configuration, no hassle – just connect and transfer at speeds up to **40 Gbps**.

<p align="center">
  <img src="docs/preview.png" alt="Thunderlink Screenshot" width="700">
</p>

## ✨ Features

| Feature | Description |
|---------|-------------|
| 🚀 **Blazing Fast** | Direct peer-to-peer transfer utilizing full Thunderbolt/USB-C bandwidth |
| 🔍 **Auto-Discovery** | Automatically detects connected devices – no IP configuration needed |
| 🖥️ **Cross-Platform** | Native apps for macOS, Windows, and Linux |
| 📁 **Drag & Drop** | Intuitive interface – just drag files to send |
| 🔒 **Secure** | SHA-256 checksum verification for every transfer |
| 📊 **Real-time Stats** | Live progress, speed, and ETA tracking |
| 🎨 **Modern UI** | Beautiful, dark-themed interface built for productivity |

## 📦 Installation

### Download

| Platform | Download |
|----------|----------|
| macOS | [Thunderlink.dmg](https://github.com/H0mire/thunderlink/releases/latest) |
| Windows | [Thunderlink.exe](https://github.com/H0mire/thunderlink/releases/latest) |
| Linux | [Thunderlink.AppImage](https://github.com/H0mire/thunderlink/releases/latest) |

### Requirements

- **Hardware**: Thunderbolt 3/4 or USB-C cable connecting two computers
- **macOS**: 10.15 (Catalina) or later
- **Windows**: 10 or later  
- **Linux**: Ubuntu 20.04 or equivalent

## 🚀 Usage

1. **Connect** two computers via Thunderbolt or USB-C cable
2. **Launch** Thunderlink on both machines
3. **Select** the discovered peer from the sidebar
4. **Drag & drop** files or click "Send Files"
5. **Done!** Files transfer at maximum cable speed

### Network Setup

<details>
<summary><strong>macOS</strong></summary>

Thunderbolt Bridge is built-in. Just connect the cable and it works automatically.
If needed: System Preferences → Network → Thunderbolt Bridge → DHCP

</details>

<details>
<summary><strong>Windows</strong></summary>

Requires Intel Thunderbolt drivers. For USB-C, use a compatible link cable.

</details>

<details>
<summary><strong>Linux</strong></summary>

```bash
sudo apt install thunderbolt-tools
boltctl list  # Verify connection
```

</details>

## 🛠️ Development

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) 1.70+
- Platform-specific dependencies (see below)

### Quick Start

```bash
# Clone the repository
git clone https://github.com/H0mire/thunderlink.git
cd thunderlink

# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

### Platform Dependencies

<details>
<summary><strong>macOS</strong></summary>

```bash
xcode-select --install
```

</details>

<details>
<summary><strong>Linux (Ubuntu/Debian)</strong></summary>

```bash
sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

</details>

<details>
<summary><strong>Windows</strong></summary>

Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with C++ workload.

</details>

### Project Structure

```
thunderlink/
├── src/                    # Frontend (Vanilla JS + CSS)
│   ├── main.js            # Application logic & UI
│   └── styles.css         # Cyber-themed styling
├── src-tauri/             # Backend (Rust)
│   └── src/
│       ├── network.rs     # Interface detection (Thunderbolt/USB)
│       ├── discovery.rs   # UDP peer discovery
│       ├── transfer.rs    # TCP file streaming
│       └── state.rs       # Application state management
└── .github/workflows/     # CI/CD for all platforms
```

## 🤝 Contributing

We love contributions! Thunderlink is built by the community, for the community.

### How to Contribute

1. **Fork** the repository
2. **Create** a feature branch: `git checkout -b feature/amazing-feature`
3. **Commit** your changes: `git commit -m 'Add amazing feature'`
4. **Push** to the branch: `git push origin feature/amazing-feature`
5. **Open** a Pull Request

### Development Guidelines

- **Code Style**: Run `cargo fmt` for Rust and use consistent JS formatting
- **Commits**: Use [Conventional Commits](https://conventionalcommits.org/) (`feat:`, `fix:`, `docs:`, etc.)
- **Tests**: Add tests for new functionality where applicable
- **Documentation**: Update README and code comments for significant changes

### Areas We Need Help

| Area | Description |
|------|-------------|
| 🐛 **Bug Reports** | Found a bug? [Open an issue](https://github.com/H0mire/thunderlink/issues) |
| 💡 **Feature Ideas** | Have an idea? We'd love to hear it! |
| 🌍 **Translations** | Help us reach more users worldwide |
| 📖 **Documentation** | Improve guides and examples |
| 🧪 **Testing** | Test on different hardware/OS combinations |

### Code of Conduct

Be respectful, inclusive, and constructive. We're all here to build something great together.

## 📄 License

MIT License – see [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- Built with [Tauri](https://tauri.app/) – lightweight, secure, cross-platform
- Powered by [Rust](https://www.rust-lang.org/) – fast, reliable, productive

---

<p align="center">
  <strong>Made with ⚡ for the fast lane</strong>
</p>

<p align="center">
  <a href="https://github.com/H0mire/thunderlink/stargazers">⭐ Star us on GitHub</a>
</p>

