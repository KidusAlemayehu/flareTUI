# flareTUI

> A fast, keyboard-centric, production-grade Terminal User Interface for managing Cloudflare assets.

## Features
- **Zone & DNS Management**: View, filter, and edit DNS records with orange/grey cloud toggling and safety diff modals.
- **Workers & Pages**: Inspect scripts, deployments, environment variables, and live-tail log streams over SSE.
- **Storage Browser**: D1 SQL queries and KV namespace inspection.
- **Offline & Cache-First**: Local SQLite write-through caching for instantaneous startup and fuzzy navigation.
- **Vim-inspired Keybindings**: Fast navigation with `j`/`k`, `/` fuzzy search, and command palette (`Ctrl+P`).

## Architecture
- **Language**: Rust (pinned to `1.89.0` via `rust-toolchain.toml`)
- **TUI Framework**: [Ratatui](https://github.com/ratatui/ratatui) with Crossterm backend
- **Async Runtime**: [Tokio](https://tokio.rs/)
- **Local Cache**: Embedded SQLite via `rusqlite`
- **Fuzzy Matcher**: `nucleo-matcher`
