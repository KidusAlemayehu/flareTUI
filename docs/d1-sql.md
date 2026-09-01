# D1 SQL Console Guide

The D1 SQL Console enables direct query execution against Cloudflare's serverless edge database.

## Query Presets

- `1`: Table preview (`SELECT * FROM {table} LIMIT 50;`)
- `2`: Total record count (`SELECT COUNT(*) FROM {table};`)
- `3`: Table column metadata (`PRAGMA table_info({table});`)
- `4`: Complete table schema DDL (`SELECT sql FROM sqlite_schema WHERE name = '{table}';`)

## Navigation

- Results grid supports Vim keybindings: `h` (left), `j` (down), `k` (up), `l` (right).
- Full horizontal column scrolling for wide tables.
