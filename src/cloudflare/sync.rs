use color_eyre::Result;
use crate::cloudflare::client::CloudflareClient;
use crate::db::{CacheDb, DnsRecord, PagesProject, WorkerScript, Zone};

pub struct SyncReport {
    pub zones_count: usize,
    pub records_count: usize,
    pub workers_count: usize,
    pub pages_count: usize,
    pub warning: Option<String>,
}

pub struct CloudflareSync;

impl CloudflareSync {
    pub async fn sync_all(client: &CloudflareClient, db: &CacheDb) -> Result<SyncReport> {
        let mut report = SyncReport {
            zones_count: 0,
            records_count: 0,
            workers_count: 0,
            pages_count: 0,
            warning: None,
        };

        // 1. Sync Zones
        let cf_zones = client.get_zones().await?;
        report.zones_count = cf_zones.len();

        for z in &cf_zones {
            let zone = Zone {
                id: z.id.clone(),
                name: z.name.clone(),
                status: z.status.clone(),
                plan: z.plan.as_ref().map(|p| p.name.clone()).unwrap_or_else(|| "Free".to_string()),
                created_on: z.created_on.clone().unwrap_or_else(|| "2024-01-01".to_string()),
            };
            db.upsert_zone(&zone)?;

            // 2. Sync DNS records for zone
            match client.get_dns_records(&z.id).await {
                Ok(cf_records) => {
                    report.records_count += cf_records.len();
                    let records: Vec<DnsRecord> = cf_records
                        .into_iter()
                        .map(|r| DnsRecord {
                            id: r.id,
                            zone_id: r.zone_id,
                            record_type: r.record_type,
                            name: r.name,
                            content: r.content,
                            proxied: r.proxied,
                            ttl: r.ttl,
                        })
                        .collect();
                    db.replace_zone_dns_records(&z.id, &records)?;
                }
                Err(e) => {
                    report.warning = Some(format!("DNS sync error: {e}. Token needs 'Zone -> DNS' permission"));
                }
            }
        }

        // 3. Sync Accounts -> Workers & Pages
        if let Ok(accounts) = client.get_accounts().await {
            if let Some(account) = accounts.first() {
                // Workers
                if let Ok(cf_workers) = client.get_workers(&account.id).await {
                    report.workers_count = cf_workers.len();
                    for w in cf_workers {
                        let routes_str = w
                            .routes
                            .as_ref()
                            .map(|routes| {
                                routes
                                    .iter()
                                    .map(|r| r.pattern.as_str())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            })
                            .unwrap_or_else(|| "/*".to_string());

                        let script = WorkerScript {
                            id: w.id.clone(),
                            name: w.id.clone(),
                            routes: routes_str,
                            last_deployed: w.modified_on.unwrap_or_else(|| "Active".to_string()),
                            compatibility_date: w.compatibility_date.unwrap_or_else(|| "2024-09-23".to_string()),
                            status: "Healthy".to_string(),
                        };
                        db.upsert_worker(&script)?;

                        // Cache worker bindings for offline inspector
                        if let Ok(bindings) = client.get_worker_bindings(&account.id, &w.id).await {
                            let cached: Vec<crate::db::CachedWorkerBinding> = bindings
                                .into_iter()
                                .map(|b| {
                                    let details = if let Some(db_id) = b.database_id {
                                        format!("D1 DB ({db_id})")
                                    } else if let Some(ns_id) = b.namespace_id {
                                        format!("KV Namespace ({ns_id})")
                                    } else if let Some(bucket) = b.bucket_name {
                                        format!("R2 Bucket ({bucket})")
                                    } else if let Some(q) = b.queue_name {
                                        format!("Queue ({q})")
                                    } else if let Some(t) = b.text {
                                        format!("Text: {t}")
                                    } else {
                                        "Configured".to_string()
                                    };
                                    crate::db::CachedWorkerBinding {
                                        script_name: w.id.clone(),
                                        name: b.name,
                                        binding_type: b.binding_type,
                                        details,
                                    }
                                })
                                .collect();
                            let _ = db.replace_worker_bindings(&w.id, &cached);
                        }
                    }
                }

                // Pages
                if let Ok(cf_pages) = client.get_pages(&account.id).await {
                    report.pages_count = cf_pages.len();
                    for p in cf_pages {
                        let status_str = p
                            .latest_deployment
                            .and_then(|d| d.latest_stage)
                            .map(|s| s.status)
                            .unwrap_or_else(|| "Success".to_string());

                        let project = PagesProject {
                            id: p.id,
                            name: p.name,
                            subdomain: p.subdomain,
                            production_branch: p.production_branch.unwrap_or_else(|| "main".to_string()),
                            latest_deployment_status: status_str,
                            updated_on: p.updated_on.unwrap_or_else(|| "Recently".to_string()),
                        };
                        db.upsert_page(&project)?;
                    }
                }
            }
        }

        Ok(report)
    }
}
