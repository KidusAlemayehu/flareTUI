# View 4: Storage & Edge Databases

Unified interface for managing D1 SQL databases, Workers KV key-value stores, and R2 Object Storage buckets.

## D1 SQL Console

- Press `Enter` or `i` on any D1 database to launch the interactive SQL Console.
- Schema sidebar shows tables, columns, and data types.
- Quick query presets:
  - `1`: Full table scan (`SELECT * FROM table LIMIT 50`)
  - `2`: Row count (`SELECT COUNT(*) FROM table`)
  - `3`: Table info (`PRAGMA table_info(table)`)
  - `4`: SQLite schema (`SELECT sql FROM sqlite_schema`)
- Scroll tabular results using Vim motions (`h`, `j`, `k`, `l`, `PgUp`, `PgDn`).
- Query history buffer navigation with `Up` and `Down`.

## Workers KV Inspector

- Browse KV namespaces and keys with real-time prefix filtering (`/`).
- Press `Enter` to fetch and display values with automated JSON pretty-printing.

## R2 Object Storage

- Traverse virtual directory hierarchies within R2 buckets.
- Inspect object sizes, ETags, and upload timestamps.
