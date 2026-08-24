# Master Keybindings Reference

flareTUI is designed for high-efficiency keyboard workflows.

## Global Shortcuts

| Shortcut | Description |
| :--- | :--- |
| `Ctrl+P` | Open Universal Command Palette |
| `Ctrl+U` | Switch Account Profile |
| `Ctrl+L` | Log out active profile |
| `Ctrl+A` | Open Authentication dialog |
| `1` .. `7` | Switch active tab |
| `j` / `k` or `Down` / `Up` | Navigate rows |
| `Tab` / `Shift+Tab` | Cycle focus between panes |
| `r` | Refresh data from Cloudflare API |
| `?` | Show interactive Help modal |
| `Esc` / `q` | Close modal or exit application |

## View-Specific Shortcuts

- **Zones (`1`)**: `s` (switch pane), `p` (proxy toggle), `n` (new record), `e` (edit), `d` (delete), `x` (export BIND).
- **Workers (`2`)**: `i` (inspector), `l` (attach live tail).
- **Pages (`3`)**: `d` (deployments), `l` (logs), `b` (rollback).
- **Storage (`4`)**: `s` (switch D1/KV), `Enter` (launch console/inspector), `1-4` (D1 query presets).
- **Live Tail (`5`)**: `f` (follow mode), `c` (clear buffer), `Space` (pause/resume), `/` (search filter).
- **Security (`6`)**: `t` (cycle sub-tabs), `f` (cycle action filter), `+/-` (reorder rules), `c` (copy IP), `r` (copy Ray ID).
- **Domains (`7`)**: `t` (transfer lock), `a` (auto-renew), `n` (register domain), `i` (domain inspector).
