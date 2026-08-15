# flareTUI Documentation

Welcome to the official documentation for **flareTUI**, a fast, keyboard-centric, production-grade Terminal User Interface for Cloudflare.

flareTUI brings the power of Cloudflare's global edge network directly into your terminal. Designed for DevOps engineers, site reliability teams, and edge application developers, flareTUI eliminates browser context switching while delivering instant cold boot times and sub-millisecond query responses through an embedded SQLite caching architecture.

## Key Capabilities

- **Offline-First & Instant Startup**: Cold starts in milliseconds using an embedded SQLite cache (`~/.cache/flaretui/cache.db`).
- **Interactive DNS & Zone Management**: Instant proxy toggling (`p`), inline record creation and modification, and standard RFC BIND exports (`x`).
- **Edge Compute (Workers & Live Tail)**: Inspect worker metadata, routes, bindings, and stream live execution logs over WebSockets with exception decoding.
- **Pages & Instant Rollback**: Inspect deployments and build logs across preview and production environments with zero-latency rollback dispatching.
- **Distributed Storage (D1, KV, R2)**: Interactive D1 SQL console with query presets, KV key-value browser with JSON syntax viewer, and 3-pane R2 object browser.
- **Security & WAF Analytics**: Manage firewall rulesets live, inspect security events, calculate threat scores, and analyze attacks by ASN and country.
- **Cloudflare Registrar**: Inspect registered domains, WHOIS records, DNSSEC validation, and manage transfer locks and auto-renewal.
- **Command Palette (`Ctrl+P`)**: Blazing-fast fuzzy search powered by `nucleo-matcher` for navigating views, executing tools, and triggering operations.
- **Multi-Account Profiles (`Ctrl+U`)**: Hot-swap between distinct Cloudflare accounts and API profiles at runtime.

## Documentation Index

- [Getting Started](getting-started.md)
- [Authentication](authentication.md)
- [Configuration](configuration.md)
- [Architecture](architecture.md)
- [Keybindings Reference](keybindings.md)
- [Views Overview](views/zones-and-dns.md)
