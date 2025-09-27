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
}

