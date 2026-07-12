pub const CREATE_TABLES_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS zones (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    status TEXT NOT NULL,
    plan TEXT NOT NULL,
    created_on TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS dns_records (
    id TEXT PRIMARY KEY,
    zone_id TEXT NOT NULL,
    record_type TEXT NOT NULL,
    name TEXT NOT NULL,
    content TEXT NOT NULL,
    proxied INTEGER NOT NULL,
    ttl INTEGER NOT NULL,
    FOREIGN KEY(zone_id) REFERENCES zones(id)
);

CREATE TABLE IF NOT EXISTS workers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    routes TEXT NOT NULL,
    last_deployed TEXT NOT NULL,
    compatibility_date TEXT NOT NULL,
    status TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS pages (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    subdomain TEXT NOT NULL,
    production_branch TEXT NOT NULL,
    latest_deployment_status TEXT NOT NULL,
    updated_on TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS pages_deployments (
    id TEXT PRIMARY KEY,
    project_name TEXT NOT NULL,
    environment TEXT NOT NULL,
    url TEXT NOT NULL,
    branch TEXT NOT NULL,
    commit_hash TEXT NOT NULL,
    commit_message TEXT NOT NULL,
    status TEXT NOT NULL,
    created_on TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS worker_bindings (
    script_name TEXT NOT NULL,
    name TEXT NOT NULL,
    binding_type TEXT NOT NULL,
    details TEXT NOT NULL,
    PRIMARY KEY (script_name, name)
);

CREATE TABLE IF NOT EXISTS d1_databases (
    uuid TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT,
    num_tables INTEGER NOT NULL DEFAULT 0,
    file_size INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS d1_tables_cache (
    database_id TEXT NOT NULL,
    table_name TEXT NOT NULL,
    schema_sql TEXT NOT NULL DEFAULT '',
    row_count INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (database_id, table_name)
);

CREATE TABLE IF NOT EXISTS kv_namespaces (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    supports_url_encoding INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS security_events (
    ray_id TEXT PRIMARY KEY,
    zone_id TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    action TEXT NOT NULL,
    source TEXT NOT NULL,
    rule_id TEXT NOT NULL DEFAULT '',
    rule_description TEXT NOT NULL DEFAULT '',
    client_ip TEXT NOT NULL,
    client_country TEXT NOT NULL DEFAULT '',
    client_asn INTEGER NOT NULL DEFAULT 0,
    client_as_org TEXT NOT NULL DEFAULT '',
    http_method TEXT NOT NULL,
    http_host TEXT NOT NULL,
    http_path TEXT NOT NULL,
    user_agent TEXT NOT NULL DEFAULT '',
    FOREIGN KEY(zone_id) REFERENCES zones(id)
);

CREATE TABLE IF NOT EXISTS waf_rules (
    id TEXT PRIMARY KEY,
    zone_id TEXT NOT NULL,
    description TEXT NOT NULL,
    action TEXT NOT NULL,
    expression TEXT NOT NULL,
    paused INTEGER NOT NULL DEFAULT 0,
    priority INTEGER NOT NULL DEFAULT 0,
    ref_code TEXT NOT NULL DEFAULT '',
    FOREIGN KEY(zone_id) REFERENCES zones(id)
);

CREATE TABLE IF NOT EXISTS r2_buckets (
    name TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    creation_date TEXT NOT NULL,
    location TEXT NOT NULL DEFAULT 'auto'
);

CREATE TABLE IF NOT EXISTS r2_objects (
    bucket_name TEXT NOT NULL,
    key TEXT NOT NULL,
    size INTEGER NOT NULL DEFAULT 0,
    etag TEXT NOT NULL DEFAULT '',
    uploaded TEXT NOT NULL DEFAULT '',
    storage_class TEXT NOT NULL DEFAULT 'Standard',
    PRIMARY KEY (bucket_name, key),
    FOREIGN KEY(bucket_name) REFERENCES r2_buckets(name) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS registrar_domains (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    name TEXT NOT NULL,
    status TEXT NOT NULL,
    auto_renew INTEGER NOT NULL DEFAULT 1,
    expires_at TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT '',
    locked INTEGER NOT NULL DEFAULT 1,
    privacy INTEGER NOT NULL DEFAULT 1,
    name_servers TEXT NOT NULL DEFAULT ''
);
"#;
