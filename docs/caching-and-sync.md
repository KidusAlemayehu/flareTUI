# Caching & Fast Sync Workflow

flareTUI utilizes an offline-first storage model backed by SQLite (`cache.db`).

## Cache Invalidation & Refresh

- **Cold Boot**: flareTUI immediately loads data from SQLite before dispatching background network queries.
- **Fast Sync**: Detects remote Cloudflare changes by comparing entity ETags and revision tokens, avoiding redundant payload transfers.
- **Manual Invalidation**: Press `r` at any time to force a full re-fetch from Cloudflare APIs and refresh local cache tables.
