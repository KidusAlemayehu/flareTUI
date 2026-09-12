# SQLite Database Schema

The local cache database schema (`~/.cache/flaretui/cache.db`) stores normalized Cloudflare resources:

- `zones`: Zone ID, name, status, plan tier, nameservers, paused flag.
- `dns_records`: Record ID, zone ID, name, type, content, proxied, ttl.
- `workers`: Script ID, name, modified_on, routes, bindings.
- `d1_databases`: Database ID, name, num_tables, file_size.
- `kv_namespaces`: Namespace ID, title, supports_url_encoding.
- `r2_buckets`: Bucket name, creation_date, location.
- `registrar_domains`: Domain name, status, expires_on, auto_renew, locked.
