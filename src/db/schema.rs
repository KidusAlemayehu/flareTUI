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
"#;
