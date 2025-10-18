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
"#;
