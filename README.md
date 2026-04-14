# BG3 HonourMode Save Manager

A desktop app for managing Baldur's Gate 3 HonourMode saves — back up, restore, and label runs, with a built-in profile editor to clear failed Honour Mode flags without losing your progress.

**Platforms:** Windows · Linux · Steam Deck

## Features

- Browse all HonourMode runs and their backups
- Create labelled backups at any point
- Restore any backup (with an automatic safety backup before overwriting)
- View save screenshots and metadata
- Profile editor: clear failed Honour Mode flags from `profile8.lsf` without starting over

## Installation

Download the latest release for your platform from the [Releases](../../releases/latest) page.

- **Windows:** Run the `.exe` installer
- **Linux / Steam Deck:** Download the `.AppImage`, make it executable (`chmod +x`), and run it

## Building from source

**Prerequisites:** [Rust](https://rustup.rs), [Node.js](https://nodejs.org) 20+, and the [Tauri system dependencies](https://v2.tauri.app/start/prerequisites/) for your platform.

```bash
npm install
npm run tauri build
```

## License

[GNU GPL v3](LICENSE) — open source, but derivative works must remain open source under the same license.
