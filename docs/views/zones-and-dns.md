# View 1: Zones & DNS Management

The Zones & DNS view provides comprehensive control over all Cloudflare-hosted domains and their DNS records.

## Layout

- **Left Pane**: List of active zones with plan tier badges (Enterprise, Business, Pro, Free) and status indicators.
- **Right Pane**: Tabular list of DNS records for the selected zone, showing Type, Name, Content/Target, Proxy Status, and TTL.

## Shortcuts

- `s` or `Tab`: Toggle active focus between the Zones pane and DNS Records pane.
- `p`: Instantly toggle Cloudflare Proxy status (`Proxied` vs `DNS Only`).
- `n`: Create a new DNS record.
- `e`: Edit selected DNS record target, TTL, or proxy status.
- `d`: Delete record (guarded by confirmation dialog).
- `i`: Open Zone Inspector (SSL configuration, Dev mode, Security level, Edge cache purge).
- `x`: Export DNS records to standard RFC BIND format (`.zone`).
- `/`: Live fuzzy filter records.
