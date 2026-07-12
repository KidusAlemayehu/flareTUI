pub mod schema;

use std::path::Path;
use std::sync::{Arc, Mutex};
use color_eyre::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zone {
    pub id: String,
    pub name: String,
    pub status: String,
    pub plan: String,
    pub created_on: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub id: String,
    pub zone_id: String,
    pub record_type: String,
    pub name: String,
    pub content: String,
    pub proxied: bool,
    pub ttl: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerScript {
    pub id: String,
    pub name: String,
    pub routes: String,
    pub last_deployed: String,
    pub compatibility_date: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagesProject {
    pub id: String,
    pub name: String,
    pub subdomain: String,
    pub production_branch: String,
    pub latest_deployment_status: String,
    pub updated_on: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedDeployment {
    pub id: String,
    pub project_name: String,
    pub environment: String,
    pub url: String,
    pub branch: String,
    pub commit_hash: String,
    pub commit_message: String,
    pub status: String,
    pub created_on: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CachedWorkerBinding {
    pub script_name: String,
    pub name: String,
    pub binding_type: String,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CachedD1Database {
    pub uuid: String,
    pub name: String,
    pub version: String,
    pub num_tables: usize,
    pub file_size: u64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CachedD1Table {
    pub database_id: String,
    pub table_name: String,
    pub schema_sql: String,
    pub row_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CachedKvNamespace {
    pub id: String,
    pub title: String,
    pub supports_url_encoding: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CachedSecurityEvent {
    pub ray_id: String,
    pub zone_id: String,
    pub timestamp: String,
    pub action: String,
    pub source: String,
    pub rule_id: String,
    pub rule_description: String,
    pub client_ip: String,
    pub client_country: String,
    pub client_asn: u32,
    pub client_as_org: String,
    pub http_method: String,
    pub http_host: String,
    pub http_path: String,
    pub user_agent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CachedWafRule {
    pub id: String,
    pub zone_id: String,
    pub description: String,
    pub action: String,
    pub expression: String,
    pub paused: bool,
    pub priority: u32,
    pub ref_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CachedR2Bucket {
    pub name: String,
    pub account_id: String,
    pub creation_date: String,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CachedR2Object {
    pub bucket_name: String,
    pub key: String,
    pub size: u64,
    pub etag: String,
    pub uploaded: String,
    pub storage_class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CachedRegistrarDomain {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub status: String,
    pub auto_renew: bool,
    pub expires_at: String,
    pub created_at: String,
    pub locked: bool,
    pub privacy: bool,
    pub name_servers: Vec<String>,
}

#[derive(Clone)]
pub struct CacheDb {
    conn: Arc<Mutex<Connection>>,
}

impl CacheDb {
    pub fn open_or_create<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(schema::CREATE_TABLES_SQL)?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        // Purge any legacy sample data
        db.purge_mock_data()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(schema::CREATE_TABLES_SQL)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.purge_mock_data()?;
        Ok(db)
    }

    pub fn purge_mock_data(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM dns_records WHERE zone_id IN ('z1', 'z2', 'z3') OR id IN ('d1', 'd2', 'd3', 'd4', 'd5', 'd6', 'd7', 'd8')", [])?;
        conn.execute("DELETE FROM zones WHERE id IN ('z1', 'z2', 'z3')", [])?;
        conn.execute("DELETE FROM workers WHERE id IN ('w1', 'w2', 'w3', 'w4')", [])?;
        conn.execute("DELETE FROM pages WHERE id IN ('p1', 'p2', 'p3')", [])?;
        conn.execute("DELETE FROM security_events WHERE zone_id IN ('z1', 'z2', 'z3')", [])?;
        conn.execute("DELETE FROM waf_rules WHERE zone_id IN ('z1', 'z2', 'z3')", [])?;
        Ok(())
    }

    pub fn get_zones(&self) -> Result<Vec<Zone>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, status, plan, created_on FROM zones ORDER BY name ASC")?;
        let zones = stmt
            .query_map([], |row| {
                Ok(Zone {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    status: row.get(2)?,
                    plan: row.get(3)?,
                    created_on: row.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(zones)
    }

    pub fn get_dns_records(&self, zone_id: &str) -> Result<Vec<DnsRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, zone_id, record_type, name, content, proxied, ttl FROM dns_records WHERE zone_id = ? ORDER BY record_type ASC, name ASC",
        )?;
        let records = stmt
            .query_map(params![zone_id], |row| {
                let proxied_int: i32 = row.get(5)?;
                Ok(DnsRecord {
                    id: row.get(0)?,
                    zone_id: row.get(1)?,
                    record_type: row.get(2)?,
                    name: row.get(3)?,
                    content: row.get(4)?,
                    proxied: proxied_int != 0,
                    ttl: row.get(6)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(records)
    }

    pub fn toggle_dns_proxy(&self, record_id: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let current_proxied: i32 = conn.query_row(
            "SELECT proxied FROM dns_records WHERE id = ?",
            params![record_id],
            |row| row.get(0),
        )?;
        let new_proxied = if current_proxied == 0 { 1 } else { 0 };
        conn.execute(
            "UPDATE dns_records SET proxied = ? WHERE id = ?",
            params![new_proxied, record_id],
        )?;
        Ok(new_proxied != 0)
    }

    pub fn upsert_dns_record(&self, record: &DnsRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let proxied_int = if record.proxied { 1 } else { 0 };
        conn.execute(
            "INSERT INTO dns_records (id, zone_id, record_type, name, content, proxied, ttl)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
             zone_id = excluded.zone_id,
             record_type = excluded.record_type,
             name = excluded.name,
             content = excluded.content,
             proxied = excluded.proxied,
             ttl = excluded.ttl",
            params![
                record.id,
                record.zone_id,
                record.record_type,
                record.name,
                record.content,
                proxied_int,
                record.ttl
            ],
        )?;
        Ok(())
    }

    pub fn delete_dns_record(&self, record_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM dns_records WHERE id = ?", params![record_id])?;
        Ok(())
    }

    pub fn get_workers(&self) -> Result<Vec<WorkerScript>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, routes, last_deployed, compatibility_date, status FROM workers ORDER BY name ASC")?;
        let workers = stmt
            .query_map([], |row| {
                Ok(WorkerScript {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    routes: row.get(2)?,
                    last_deployed: row.get(3)?,
                    compatibility_date: row.get(4)?,
                    status: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(workers)
    }

    pub fn get_pages(&self) -> Result<Vec<PagesProject>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, subdomain, production_branch, latest_deployment_status, updated_on FROM pages ORDER BY name ASC")?;
        let pages = stmt
            .query_map([], |row| {
                Ok(PagesProject {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    subdomain: row.get(2)?,
                    production_branch: row.get(3)?,
                    latest_deployment_status: row.get(4)?,
                    updated_on: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(pages)
    }

    pub fn upsert_zone(&self, zone: &Zone) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO zones (id, name, status, plan, created_on)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
             name = excluded.name,
             status = excluded.status,
             plan = excluded.plan,
             created_on = excluded.created_on",
            params![zone.id, zone.name, zone.status, zone.plan, zone.created_on],
        )?;
        Ok(())
    }

    pub fn replace_zone_dns_records(&self, zone_id: &str, records: &[DnsRecord]) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM dns_records WHERE zone_id = ?", params![zone_id])?;
        for r in records {
            let proxied_int = if r.proxied { 1 } else { 0 };
            tx.execute(
                "INSERT INTO dns_records (id, zone_id, record_type, name, content, proxied, ttl)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![r.id, r.zone_id, r.record_type, r.name, r.content, proxied_int, r.ttl],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn upsert_worker(&self, worker: &WorkerScript) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO workers (id, name, routes, last_deployed, compatibility_date, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
             name = excluded.name,
             routes = excluded.routes,
             last_deployed = excluded.last_deployed,
             compatibility_date = excluded.compatibility_date,
             status = excluded.status",
            params![
                worker.id,
                worker.name,
                worker.routes,
                worker.last_deployed,
                worker.compatibility_date,
                worker.status
            ],
        )?;
        Ok(())
    }

    pub fn upsert_page(&self, page: &PagesProject) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO pages (id, name, subdomain, production_branch, latest_deployment_status, updated_on)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
             name = excluded.name,
             subdomain = excluded.subdomain,
             production_branch = excluded.production_branch,
             latest_deployment_status = excluded.latest_deployment_status,
             updated_on = excluded.updated_on",
            params![
                page.id,
                page.name,
                page.subdomain,
                page.production_branch,
                page.latest_deployment_status,
                page.updated_on
            ],
        )?;
        Ok(())
    }

    pub fn get_pages_deployments(&self, project_name: &str) -> Result<Vec<CachedDeployment>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, project_name, environment, url, branch, commit_hash, commit_message, status, created_on
             FROM pages_deployments WHERE project_name = ? ORDER BY created_on DESC",
        )?;
        let deps = stmt
            .query_map(params![project_name], |row| {
                Ok(CachedDeployment {
                    id: row.get(0)?,
                    project_name: row.get(1)?,
                    environment: row.get(2)?,
                    url: row.get(3)?,
                    branch: row.get(4)?,
                    commit_hash: row.get(5)?,
                    commit_message: row.get(6)?,
                    status: row.get(7)?,
                    created_on: row.get(8)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(deps)
    }

    pub fn replace_pages_deployments(&self, project_name: &str, deployments: &[CachedDeployment]) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM pages_deployments WHERE project_name = ?", params![project_name])?;
        for d in deployments {
            tx.execute(
                "INSERT INTO pages_deployments (id, project_name, environment, url, branch, commit_hash, commit_message, status, created_on)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    d.id,
                    d.project_name,
                    d.environment,
                    d.url,
                    d.branch,
                    d.commit_hash,
                    d.commit_message,
                    d.status,
                    d.created_on
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn get_worker_bindings(&self, script_name: &str) -> Result<Vec<CachedWorkerBinding>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT script_name, name, binding_type, details FROM worker_bindings WHERE script_name = ? ORDER BY name ASC",
        )?;
        let bindings = stmt
            .query_map(params![script_name], |row| {
                Ok(CachedWorkerBinding {
                    script_name: row.get(0)?,
                    name: row.get(1)?,
                    binding_type: row.get(2)?,
                    details: row.get(3)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(bindings)
    }

    pub fn replace_worker_bindings(&self, script_name: &str, bindings: &[CachedWorkerBinding]) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM worker_bindings WHERE script_name = ?", params![script_name])?;
        for b in bindings {
            tx.execute(
                "INSERT INTO worker_bindings (script_name, name, binding_type, details)
                 VALUES (?1, ?2, ?3, ?4)",
                params![b.script_name, b.name, b.binding_type, b.details],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn get_d1_databases(&self) -> Result<Vec<CachedD1Database>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT uuid, name, version, num_tables, file_size, created_at FROM d1_databases ORDER BY name ASC",
        )?;
        let dbs = stmt
            .query_map([], |row| {
                let num_tables_i64: i64 = row.get(3)?;
                let file_size_i64: i64 = row.get(4)?;
                Ok(CachedD1Database {
                    uuid: row.get(0)?,
                    name: row.get(1)?,
                    version: row.get(2).unwrap_or_else(|_| "production".to_string()),
                    num_tables: num_tables_i64 as usize,
                    file_size: file_size_i64 as u64,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(dbs)
    }

    pub fn upsert_d1_database(&self, db: &CachedD1Database) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO d1_databases (uuid, name, version, num_tables, file_size, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(uuid) DO UPDATE SET
             name = excluded.name,
             version = excluded.version,
             num_tables = excluded.num_tables,
             file_size = excluded.file_size,
             created_at = excluded.created_at",
            params![
                db.uuid,
                db.name,
                db.version,
                db.num_tables as i64,
                db.file_size as i64,
                db.created_at
            ],
        )?;
        Ok(())
    }

    pub fn replace_d1_databases(&self, dbs: &[CachedD1Database]) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM d1_databases", [])?;
        for db in dbs {
            tx.execute(
                "INSERT INTO d1_databases (uuid, name, version, num_tables, file_size, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    db.uuid,
                    db.name,
                    db.version,
                    db.num_tables as i64,
                    db.file_size as i64,
                    db.created_at
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn get_d1_tables(&self, database_id: &str) -> Result<Vec<CachedD1Table>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT database_id, table_name, schema_sql, row_count FROM d1_tables_cache WHERE database_id = ? ORDER BY table_name ASC",
        )?;
        let tables = stmt
            .query_map(params![database_id], |row| {
                let row_count_i64: i64 = row.get(3)?;
                Ok(CachedD1Table {
                    database_id: row.get(0)?,
                    table_name: row.get(1)?,
                    schema_sql: row.get(2)?,
                    row_count: row_count_i64 as usize,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(tables)
    }

    pub fn replace_d1_tables(&self, database_id: &str, tables: &[CachedD1Table]) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM d1_tables_cache WHERE database_id = ?", params![database_id])?;
        for t in tables {
            tx.execute(
                "INSERT INTO d1_tables_cache (database_id, table_name, schema_sql, row_count)
                 VALUES (?1, ?2, ?3, ?4)",
                params![t.database_id, t.table_name, t.schema_sql, t.row_count as i64],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn get_kv_namespaces(&self) -> Result<Vec<CachedKvNamespace>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, supports_url_encoding FROM kv_namespaces ORDER BY title ASC",
        )?;
        let namespaces = stmt
            .query_map([], |row| {
                let supports_int: i32 = row.get(2)?;
                Ok(CachedKvNamespace {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    supports_url_encoding: supports_int != 0,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(namespaces)
    }

    pub fn upsert_kv_namespace(&self, ns: &CachedKvNamespace) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let supports_int = if ns.supports_url_encoding { 1 } else { 0 };
        conn.execute(
            "INSERT INTO kv_namespaces (id, title, supports_url_encoding)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(id) DO UPDATE SET
             title = excluded.title,
             supports_url_encoding = excluded.supports_url_encoding",
            params![ns.id, ns.title, supports_int],
        )?;
        Ok(())
    }

    pub fn replace_kv_namespaces(&self, namespaces: &[CachedKvNamespace]) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM kv_namespaces", [])?;
        for ns in namespaces {
            let supports_int = if ns.supports_url_encoding { 1 } else { 0 };
            tx.execute(
                "INSERT INTO kv_namespaces (id, title, supports_url_encoding)
                 VALUES (?1, ?2, ?3)",
                params![ns.id, ns.title, supports_int],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn replace_security_events(&self, zone_id: &str, events: &[CachedSecurityEvent]) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM security_events WHERE zone_id = ?1", params![zone_id])?;
        for ev in events {
            tx.execute(
                "INSERT INTO security_events (
                    ray_id, zone_id, timestamp, action, source,
                    rule_id, rule_description, client_ip, client_country,
                    client_asn, client_as_org, http_method, http_host, http_path, user_agent
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                params![
                    ev.ray_id,
                    ev.zone_id,
                    ev.timestamp,
                    ev.action,
                    ev.source,
                    ev.rule_id,
                    ev.rule_description,
                    ev.client_ip,
                    ev.client_country,
                    ev.client_asn,
                    ev.client_as_org,
                    ev.http_method,
                    ev.http_host,
                    ev.http_path,
                    ev.user_agent,
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn get_security_events(&self, zone_id: &str) -> Result<Vec<CachedSecurityEvent>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT ray_id, zone_id, timestamp, action, source,
                    rule_id, rule_description, client_ip, client_country,
                    client_asn, client_as_org, http_method, http_host, http_path, user_agent
             FROM security_events
             WHERE zone_id = ?1
             ORDER BY timestamp DESC",
        )?;
        let rows = stmt
            .query_map(params![zone_id], |row| {
                Ok(CachedSecurityEvent {
                    ray_id: row.get(0)?,
                    zone_id: row.get(1)?,
                    timestamp: row.get(2)?,
                    action: row.get(3)?,
                    source: row.get(4)?,
                    rule_id: row.get(5)?,
                    rule_description: row.get(6)?,
                    client_ip: row.get(7)?,
                    client_country: row.get(8)?,
                    client_asn: row.get(9)?,
                    client_as_org: row.get(10)?,
                    http_method: row.get(11)?,
                    http_host: row.get(12)?,
                    http_path: row.get(13)?,
                    user_agent: row.get(14)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn replace_waf_rules(&self, zone_id: &str, rules: &[CachedWafRule]) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM waf_rules WHERE zone_id = ?1", params![zone_id])?;
        for r in rules {
            let paused_int = if r.paused { 1 } else { 0 };
            tx.execute(
                "INSERT INTO waf_rules (
                    id, zone_id, description, action, expression, paused, priority, ref_code
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    r.id,
                    r.zone_id,
                    r.description,
                    r.action,
                    r.expression,
                    paused_int,
                    r.priority,
                    r.ref_code,
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn get_waf_rules(&self, zone_id: &str) -> Result<Vec<CachedWafRule>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, zone_id, description, action, expression, paused, priority, ref_code
             FROM waf_rules
             WHERE zone_id = ?1
             ORDER BY priority ASC, id ASC",
        )?;
        let rows = stmt
            .query_map(params![zone_id], |row| {
                let paused_int: i32 = row.get(5)?;
                Ok(CachedWafRule {
                    id: row.get(0)?,
                    zone_id: row.get(1)?,
                    description: row.get(2)?,
                    action: row.get(3)?,
                    expression: row.get(4)?,
                    paused: paused_int != 0,
                    priority: row.get(6)?,
                    ref_code: row.get(7)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn update_waf_rule_paused(&self, rule_id: &str, paused: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE waf_rules SET paused = ?1 WHERE id = ?2",
            params![if paused { 1 } else { 0 }, rule_id],
        )?;
        Ok(())
    }

    pub fn update_waf_rule_priority(&self, rule_id: &str, priority: u32) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE waf_rules SET priority = ?1 WHERE id = ?2",
            params![priority, rule_id],
        )?;
        Ok(())
    }

    pub fn replace_r2_buckets(&self, account_id: &str, buckets: &[CachedR2Bucket]) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM r2_buckets WHERE account_id = ?1", params![account_id])?;
        for b in buckets {
            tx.execute(
                "INSERT INTO r2_buckets (name, account_id, creation_date, location)
                 VALUES (?1, ?2, ?3, ?4)",
                params![b.name, b.account_id, b.creation_date, b.location],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn get_r2_buckets(&self, account_id: &str) -> Result<Vec<CachedR2Bucket>> {
        let conn = self.conn.lock().unwrap();
        if account_id.is_empty() {
            let mut stmt = conn.prepare(
                "SELECT name, account_id, creation_date, location
                 FROM r2_buckets
                 ORDER BY name ASC",
            )?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(CachedR2Bucket {
                        name: row.get(0)?,
                        account_id: row.get(1)?,
                        creation_date: row.get(2)?,
                        location: row.get(3)?,
                    })
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            return Ok(rows);
        }
        let mut stmt = conn.prepare(
            "SELECT name, account_id, creation_date, location
             FROM r2_buckets
             WHERE account_id = ?1
             ORDER BY name ASC",
        )?;
        let rows = stmt
            .query_map(params![account_id], |row| {
                Ok(CachedR2Bucket {
                    name: row.get(0)?,
                    account_id: row.get(1)?,
                    creation_date: row.get(2)?,
                    location: row.get(3)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn replace_r2_objects(&self, bucket_name: &str, objects: &[CachedR2Object]) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM r2_objects WHERE bucket_name = ?1", params![bucket_name])?;
        for obj in objects {
            tx.execute(
                "INSERT INTO r2_objects (bucket_name, key, size, etag, uploaded, storage_class)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    obj.bucket_name,
                    obj.key,
                    obj.size as i64,
                    obj.etag,
                    obj.uploaded,
                    obj.storage_class,
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn get_r2_objects(&self, bucket_name: &str) -> Result<Vec<CachedR2Object>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT bucket_name, key, size, etag, uploaded, storage_class
             FROM r2_objects
             WHERE bucket_name = ?1
             ORDER BY key ASC",
        )?;
        let rows = stmt
            .query_map(params![bucket_name], |row| {
                let size_i64: i64 = row.get(2)?;
                Ok(CachedR2Object {
                    bucket_name: row.get(0)?,
                    key: row.get(1)?,
                    size: size_i64.max(0) as u64,
                    etag: row.get(3)?,
                    uploaded: row.get(4)?,
                    storage_class: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_r2_objects_by_prefix(&self, bucket_name: &str, prefix: &str) -> Result<Vec<CachedR2Object>> {
        let conn = self.conn.lock().unwrap();
        let pattern = format!("{}%", prefix);
        let mut stmt = conn.prepare(
            "SELECT bucket_name, key, size, etag, uploaded, storage_class
             FROM r2_objects
             WHERE bucket_name = ?1 AND key LIKE ?2
             ORDER BY key ASC",
        )?;
        let rows = stmt
            .query_map(params![bucket_name, pattern], |row| {
                let size_i64: i64 = row.get(2)?;
                Ok(CachedR2Object {
                    bucket_name: row.get(0)?,
                    key: row.get(1)?,
                    size: size_i64.max(0) as u64,
                    etag: row.get(3)?,
                    uploaded: row.get(4)?,
                    storage_class: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn replace_registrar_domains(&self, account_id: &str, domains: &[CachedRegistrarDomain]) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        if account_id.is_empty() {
            tx.execute("DELETE FROM registrar_domains", [])?;
        } else {
            tx.execute("DELETE FROM registrar_domains WHERE account_id = ?1", params![account_id])?;
        }
        for d in domains {
            let ns_json = serde_json::to_string(&d.name_servers).unwrap_or_default();
            tx.execute(
                "INSERT INTO registrar_domains (id, account_id, name, status, auto_renew, expires_at, created_at, locked, privacy, name_servers)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    d.id,
                    d.account_id,
                    d.name,
                    d.status,
                    d.auto_renew as i32,
                    d.expires_at,
                    d.created_at,
                    d.locked as i32,
                    d.privacy as i32,
                    ns_json,
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn get_registrar_domains(&self, account_id: &str) -> Result<Vec<CachedRegistrarDomain>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = if account_id.is_empty() {
            conn.prepare(
                "SELECT id, account_id, name, status, auto_renew, expires_at, created_at, locked, privacy, name_servers
                 FROM registrar_domains
                 ORDER BY name ASC",
            )?
        } else {
            conn.prepare(
                "SELECT id, account_id, name, status, auto_renew, expires_at, created_at, locked, privacy, name_servers
                 FROM registrar_domains
                 WHERE account_id = ?1
                 ORDER BY name ASC",
            )?
        };

        let map_row = |row: &rusqlite::Row| {
            let auto_renew_int: i32 = row.get(4)?;
            let locked_int: i32 = row.get(7)?;
            let privacy_int: i32 = row.get(8)?;
            let ns_str: String = row.get(9)?;
            let name_servers: Vec<String> = serde_json::from_str(&ns_str).unwrap_or_default();

            Ok(CachedRegistrarDomain {
                id: row.get(0)?,
                account_id: row.get(1)?,
                name: row.get(2)?,
                status: row.get(3)?,
                auto_renew: auto_renew_int != 0,
                expires_at: row.get(5)?,
                created_at: row.get(6)?,
                locked: locked_int != 0,
                privacy: privacy_int != 0,
                name_servers,
            })
        };

        let rows = if account_id.is_empty() {
            stmt.query_map([], map_row)?.collect::<std::result::Result<Vec<_>, _>>()?
        } else {
            stmt.query_map(params![account_id], map_row)?.collect::<std::result::Result<Vec<_>, _>>()?
        };

        Ok(rows)
    }

    pub fn update_registrar_domain_auto_renew(&self, domain_name: &str, auto_renew: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE registrar_domains SET auto_renew = ?1 WHERE name = ?2",
            params![auto_renew as i32, domain_name],
        )?;
        Ok(())
    }

    pub fn update_registrar_domain_locked(&self, domain_name: &str, locked: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE registrar_domains SET locked = ?1 WHERE name = ?2",
            params![locked as i32, domain_name],
        )?;
        Ok(())
    }

    pub fn update_registrar_domain_privacy(&self, domain_name: &str, privacy: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE registrar_domains SET privacy = ?1 WHERE name = ?2",
            params![privacy as i32, domain_name],
        )?;
        Ok(())
    }

    pub fn insert_registrar_domain(&self, domain: &CachedRegistrarDomain) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let ns_json = serde_json::to_string(&domain.name_servers).unwrap_or_default();
        conn.execute(
            "INSERT OR REPLACE INTO registrar_domains (id, account_id, name, status, auto_renew, expires_at, created_at, locked, privacy, name_servers)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                domain.id,
                domain.account_id,
                domain.name,
                domain.status,
                domain.auto_renew as i32,
                domain.expires_at,
                domain.created_at,
                domain.locked as i32,
                domain.privacy as i32,
                ns_json,
            ],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_bindings_cache() {
        let db = CacheDb::open_in_memory().expect("open db");
        let script = "test-worker";
        let bindings = vec![
            CachedWorkerBinding {
                script_name: script.to_string(),
                name: "KV_STORE".to_string(),
                binding_type: "kv_namespace".to_string(),
                details: "KV Namespace (kv123)".to_string(),
            },
            CachedWorkerBinding {
                script_name: script.to_string(),
                name: "DB".to_string(),
                binding_type: "d1".to_string(),
                details: "D1 DB (d1-uuid)".to_string(),
            },
        ];

        db.replace_worker_bindings(script, &bindings).expect("save bindings");
        let retrieved = db.get_worker_bindings(script).expect("get bindings");
        assert_eq!(retrieved.len(), 2);
        assert_eq!(retrieved[0].name, "DB");
        assert_eq!(retrieved[1].name, "KV_STORE");
    }

    #[test]
    fn test_d1_databases_and_tables_cache() {
        let db = CacheDb::open_in_memory().expect("open db");
        let dbs = vec![
            CachedD1Database {
                uuid: "d1-uuid-1".to_string(),
                name: "auth-db".to_string(),
                version: "production".to_string(),
                num_tables: 4,
                file_size: 204800,
                created_at: "2025-10-01".to_string(),
            },
            CachedD1Database {
                uuid: "d1-uuid-2".to_string(),
                name: "analytics-db".to_string(),
                version: "production".to_string(),
                num_tables: 8,
                file_size: 4096000,
                created_at: "2025-10-15".to_string(),
            },
        ];

        db.replace_d1_databases(&dbs).expect("replace d1 dbs");
        let loaded = db.get_d1_databases().expect("get d1 dbs");
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].name, "analytics-db");
        assert_eq!(loaded[1].name, "auth-db");

        let tables = vec![
            CachedD1Table {
                database_id: "d1-uuid-1".to_string(),
                table_name: "users".to_string(),
                schema_sql: "CREATE TABLE users (id INT, email TEXT)".to_string(),
                row_count: 1420,
            },
            CachedD1Table {
                database_id: "d1-uuid-1".to_string(),
                table_name: "tokens".to_string(),
                schema_sql: "CREATE TABLE tokens (token_id TEXT, user_id INT)".to_string(),
                row_count: 580,
            },
        ];
        db.replace_d1_tables("d1-uuid-1", &tables).expect("replace tables");
        let loaded_tables = db.get_d1_tables("d1-uuid-1").expect("get tables");
        assert_eq!(loaded_tables.len(), 2);
        assert_eq!(loaded_tables[0].table_name, "tokens");
        assert_eq!(loaded_tables[1].table_name, "users");
    }

    #[test]
    fn test_kv_namespaces_cache() {
        let db = CacheDb::open_in_memory().expect("open db");
        let namespaces = vec![
            CachedKvNamespace {
                id: "kv-1".to_string(),
                title: "CONFIG_KV".to_string(),
                supports_url_encoding: true,
            },
            CachedKvNamespace {
                id: "kv-2".to_string(),
                title: "CACHE_KV".to_string(),
                supports_url_encoding: false,
            },
        ];

        db.replace_kv_namespaces(&namespaces).expect("replace kv namespaces");
        let loaded = db.get_kv_namespaces().expect("get kv namespaces");
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].title, "CACHE_KV");
        assert_eq!(loaded[1].title, "CONFIG_KV");
    }

    #[test]
    fn test_security_events_cache() {
        let db = CacheDb::open_in_memory().expect("open db");
        let zone_id = "zone-test-sec-1";
        db.upsert_zone(&Zone {
            id: zone_id.to_string(),
            name: "example.com".to_string(),
            status: "active".to_string(),
            plan: "Free".to_string(),
            created_on: "2026-01-01".to_string(),
        })
        .expect("upsert zone");

        let events = vec![
            CachedSecurityEvent {
                ray_id: "8472a1bf001".to_string(),
                zone_id: zone_id.to_string(),
                timestamp: "2026-01-08T14:22:00Z".to_string(),
                action: "block".to_string(),
                source: "firewallCustom".to_string(),
                rule_id: "rule-sql-inj".to_string(),
                rule_description: "Block SQL injection attempts".to_string(),
                client_ip: "198.51.100.42".to_string(),
                client_country: "US".to_string(),
                client_asn: 13335,
                client_as_org: "Cloudflare Inc".to_string(),
                http_method: "POST".to_string(),
                http_host: "api.example.com".to_string(),
                http_path: "/login".to_string(),
                user_agent: "sqlmap/1.7".to_string(),
            },
            CachedSecurityEvent {
                ray_id: "8472a1bf002".to_string(),
                zone_id: zone_id.to_string(),
                timestamp: "2026-01-08T14:25:00Z".to_string(),
                action: "managed_challenge".to_string(),
                source: "waf".to_string(),
                rule_id: "rule-bot".to_string(),
                rule_description: "Challenge automated bots".to_string(),
                client_ip: "203.0.113.19".to_string(),
                client_country: "DE".to_string(),
                client_asn: 24940,
                client_as_org: "Hetzner Online GmbH".to_string(),
                http_method: "GET".to_string(),
                http_host: "api.example.com".to_string(),
                http_path: "/catalog".to_string(),
                user_agent: "Mozilla/5.0 Bot".to_string(),
            },
        ];

        db.replace_security_events(zone_id, &events).expect("replace security events");
        let retrieved = db.get_security_events(zone_id).expect("get security events");
        assert_eq!(retrieved.len(), 2);
        // Ordered by timestamp DESC
        assert_eq!(retrieved[0].ray_id, "8472a1bf002");
        assert_eq!(retrieved[0].client_ip, "203.0.113.19");
        assert_eq!(retrieved[1].ray_id, "8472a1bf001");
        assert_eq!(retrieved[1].action, "block");
    }

    #[test]
    fn test_waf_rules_cache() {
        let db = CacheDb::open_in_memory().expect("open db");
        let zone_id = "zone-test-waf-1";
        db.upsert_zone(&Zone {
            id: zone_id.to_string(),
            name: "example.com".to_string(),
            status: "active".to_string(),
            plan: "Free".to_string(),
            created_on: "2026-01-01".to_string(),
        })
        .expect("upsert zone");

        let rules = vec![
            CachedWafRule {
                id: "rule-2".to_string(),
                zone_id: zone_id.to_string(),
                description: "Challenge suspicious bots".to_string(),
                action: "managed_challenge".to_string(),
                expression: "(cf.client.bot)".to_string(),
                paused: false,
                priority: 2,
                ref_code: "RULE_BOT_2".to_string(),
            },
            CachedWafRule {
                id: "rule-1".to_string(),
                zone_id: zone_id.to_string(),
                description: "Block sensitive admin access".to_string(),
                action: "block".to_string(),
                expression: "(http.request.uri.path contains \"/admin\")".to_string(),
                paused: false,
                priority: 1,
                ref_code: "RULE_ADMIN_1".to_string(),
            },
        ];

        db.replace_waf_rules(zone_id, &rules).expect("replace waf rules");
        let retrieved = db.get_waf_rules(zone_id).expect("get waf rules");
        assert_eq!(retrieved.len(), 2);
        // Ordered by priority ASC
        assert_eq!(retrieved[0].id, "rule-1");
        assert_eq!(retrieved[0].priority, 1);
        assert_eq!(retrieved[1].id, "rule-2");
        assert_eq!(retrieved[1].priority, 2);
    }

    #[test]
    fn test_r2_buckets_and_objects_cache() {
        let db = CacheDb::open_in_memory().expect("open db");
        let account_id = "test-account-123";

        let buckets = vec![
            CachedR2Bucket {
                name: "backup-assets".to_string(),
                account_id: account_id.to_string(),
                creation_date: "2026-03-01T10:00:00Z".to_string(),
                location: "weur".to_string(),
            },
            CachedR2Bucket {
                name: "public-cdn".to_string(),
                account_id: account_id.to_string(),
                creation_date: "2026-03-05T12:00:00Z".to_string(),
                location: "wnam".to_string(),
            },
        ];

        db.replace_r2_buckets(account_id, &buckets).expect("replace r2 buckets");
        let retrieved_buckets = db.get_r2_buckets(account_id).expect("get r2 buckets");
        assert_eq!(retrieved_buckets.len(), 2);
        assert_eq!(retrieved_buckets[0].name, "backup-assets");
        assert_eq!(retrieved_buckets[1].name, "public-cdn");

        let objects = vec![
            CachedR2Object {
                bucket_name: "public-cdn".to_string(),
                key: "static/css/style.css".to_string(),
                size: 24500,
                etag: "\"abc123etag\"".to_string(),
                uploaded: "2026-03-05T12:10:00Z".to_string(),
                storage_class: "Standard".to_string(),
            },
            CachedR2Object {
                bucket_name: "public-cdn".to_string(),
                key: "static/images/logo.svg".to_string(),
                size: 1048576,
                etag: "\"def456etag\"".to_string(),
                uploaded: "2026-03-05T12:15:00Z".to_string(),
                storage_class: "Standard".to_string(),
            },
            CachedR2Object {
                bucket_name: "public-cdn".to_string(),
                key: "index.html".to_string(),
                size: 4096,
                etag: "\"ghi789etag\"".to_string(),
                uploaded: "2026-03-05T12:20:00Z".to_string(),
                storage_class: "Standard".to_string(),
            },
        ];

        db.replace_r2_objects("public-cdn", &objects).expect("replace r2 objects");
        let all_objects = db.get_r2_objects("public-cdn").expect("get r2 objects");
        assert_eq!(all_objects.len(), 3);
        assert_eq!(all_objects[0].key, "index.html");

        let static_objects = db.get_r2_objects_by_prefix("public-cdn", "static/").expect("get prefix");
        assert_eq!(static_objects.len(), 2);
        assert_eq!(static_objects[0].key, "static/css/style.css");
        assert_eq!(static_objects[1].key, "static/images/logo.svg");
    }

    #[test]
    fn test_registrar_domains_cache() {
        let db = CacheDb::open_in_memory().expect("open db");
        let account_id = "acc_reg_test";

        let domains = vec![
            CachedRegistrarDomain {
                id: "reg-1".to_string(),
                account_id: account_id.to_string(),
                name: "daedalus-dev.com".to_string(),
                status: "active".to_string(),
                auto_renew: true,
                expires_at: "2027-08-15T12:00:00Z".to_string(),
                created_at: "2024-08-15T12:00:00Z".to_string(),
                locked: true,
                privacy: true,
                name_servers: vec!["eva.ns.cloudflare.com".to_string(), "kurt.ns.cloudflare.com".to_string()],
            },
            CachedRegistrarDomain {
                id: "reg-2".to_string(),
                account_id: account_id.to_string(),
                name: "flaretui.org".to_string(),
                status: "active".to_string(),
                auto_renew: false,
                expires_at: "2028-01-20T09:30:00Z".to_string(),
                created_at: "2025-01-20T09:30:00Z".to_string(),
                locked: false,
                privacy: true,
                name_servers: vec!["olga.ns.cloudflare.com".to_string(), "skip.ns.cloudflare.com".to_string()],
            },
        ];

        db.replace_registrar_domains(account_id, &domains).expect("replace registrar domains");
        let retrieved = db.get_registrar_domains(account_id).expect("get registrar domains");
        assert_eq!(retrieved.len(), 2);
        assert_eq!(retrieved[0].name, "daedalus-dev.com");
        assert!(retrieved[0].auto_renew);
        assert!(retrieved[0].locked);

        // Test updates
        db.update_registrar_domain_auto_renew("daedalus-dev.com", false).expect("update auto renew");
        db.update_registrar_domain_locked("daedalus-dev.com", false).expect("update locked");
        db.update_registrar_domain_privacy("daedalus-dev.com", false).expect("update privacy");

        let updated = db.get_registrar_domains(account_id).expect("get updated domains");
        assert!(!updated[0].auto_renew);
        assert!(!updated[0].locked);
        assert!(!updated[0].privacy);
    }
}


