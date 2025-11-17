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
}

