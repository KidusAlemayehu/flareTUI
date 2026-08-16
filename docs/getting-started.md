# Getting Started

flareTUI runs natively on Linux, macOS, and Windows. It requires minimal system resources and provides immediate responsiveness.

## Prerequisites

- **Rust**: Version 1.80 or higher (pinned to 1.89.0).
- **Terminal Emulator**: Any modern terminal with TrueColor and Unicode support (e.g., Alacritty, Kitty, WezTerm, Ghostty, Tmux, Windows Terminal).

## Installation

### From Source

```bash
git clone https://github.com/KidusAlemayehu/flareTUI.git
cd flareTUI
cargo build --release
./target/release/flareTUI
```

### Via Cargo

```bash
cargo install --git https://github.com/KidusAlemayehu/flareTUI
flareTUI
```

## Directory Structure

flareTUI follows standard OS directory specifications:

- **Configuration**: `~/.config/flaretui/config.toml` (Stores profiles, active tokens, and theme settings).
- **Cache Database**: `~/.cache/flaretui/cache.db` (Local SQLite database caching zones, DNS records, workers, and domains).
