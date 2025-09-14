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

        // Seed with demo initial data if empty
        db.seed_if_empty()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(schema::CREATE_TABLES_SQL)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.seed_if_empty()?;
        Ok(db)
    }

    fn seed_if_empty(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT count(*) FROM zones", [], |row| row.get(0))?;

        if count == 0 {
            // Seed sample zones
            conn.execute(
                "INSERT INTO zones (id, name, status, plan, created_on) VALUES 
                ('z1', 'example.com', 'active', 'Enterprise', '2023-01-15'),
                ('z2', 'cloud-infra.dev', 'active', 'Pro', '2023-06-20'),
                ('z3', 'personal-blog.org', 'active', 'Free', '2024-02-10')",
                [],
            )?;

            // Seed sample DNS records
            conn.execute(
                "INSERT INTO dns_records (id, zone_id, record_type, name, content, proxied, ttl) VALUES
                ('d1', 'z1', 'A', 'example.com', '192.0.2.1', 1, 1),
                ('d2', 'z1', 'AAAA', 'example.com', '2001:db8::1', 1, 1),
                ('d3', 'z1', 'CNAME', 'api.example.com', 'workers.dev', 1, 1),
                ('d4', 'z1', 'CNAME', 'status.example.com', 'statuspage.io', 0, 300),
                ('d5', 'z1', 'TXT', 'example.com', 'v=spf1 include:_spf.mx.cloudflare.net ~all', 0, 3600),
                ('d6', 'z2', 'A', 'cloud-infra.dev', '198.51.100.24', 1, 1),
                ('d7', 'z2', 'CNAME', 'auth.cloud-infra.dev', 'auth0.com', 0, 300),
                ('d8', 'z3', 'CNAME', 'personal-blog.org', 'kidus.pages.dev', 1, 1)",
                [],
            )?;

            // Seed sample workers
            conn.execute(
                "INSERT INTO workers (id, name, routes, last_deployed, compatibility_date, status) VALUES
                ('w1', 'edge-auth-guard', 'api.example.com/v1/auth/*', '2025-08-30 14:22', '2024-09-23', 'Healthy'),
                ('w2', 'image-resizer', 'images.example.com/*', '2025-09-02 09:10', '2024-11-01', 'Healthy'),
                ('w3', 'rate-limiter-proxy', 'api.example.com/*', '2025-09-10 18:45', '2025-01-15', 'Healthy'),
                ('w4', 'telemetry-collector', 'logs.cloud-infra.dev/*', '2025-09-11 22:05', '2025-04-01', 'Warning')",
                [],
            )?;

            // Seed sample pages
            conn.execute(
                "INSERT INTO pages (id, name, subdomain, production_branch, latest_deployment_status, updated_on) VALUES
                ('p1', 'main-portal', 'main-portal.pages.dev', 'main', 'Success', '2025-09-11 16:30'),
                ('p2', 'developer-docs', 'developer-docs.pages.dev', 'main', 'Success', '2025-09-08 11:20'),
                ('p3', 'admin-console', 'admin-console.pages.dev', 'production', 'Building', '2025-09-12 08:45')",
                [],
            )?;
        }
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
}
