use color_eyre::Result;
use crate::cloudflare::client::CloudflareClient;
use crate::db::{CacheDb, DnsRecord, PagesProject, WorkerScript, Zone};

pub struct SyncReport {
    pub zones_count: usize,
    pub records_count: usize,
    pub workers_count: usize,
    pub pages_count: usize,
    pub d1_count: usize,
    pub kv_count: usize,
    pub security_events_count: usize,
    pub waf_rules_count: usize,
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
            d1_count: 0,
            kv_count: 0,
            security_events_count: 0,
            waf_rules_count: 0,
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
                            zone_id: r.zone_id.unwrap_or_else(|| z.id.clone()),
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
                    let err_str = e.to_string();
                    let hint = if err_str.to_lowercase().contains("authentication")
                        || err_str.to_lowercase().contains("forbidden")
                        || err_str.contains("10000")
                    {
                        "Token lacks 'Zone -> DNS' permission. Create a User API token with 'Edit zone DNS' template."
                    } else {
                        "Check network connection or token permissions."
                    };
                    report.warning = Some(format!("DNS sync error: {e}. {hint}"));
                }
            }

            // 3. Sync Security Events for zone
            if let Ok(sec_events) = client.get_security_events(&z.id, Some(50)).await {
                report.security_events_count += sec_events.len();
                let cached: Vec<crate::db::CachedSecurityEvent> = sec_events
                    .into_iter()
                    .map(|e| crate::db::CachedSecurityEvent {
                        ray_id: e.ray_id,
                        zone_id: z.id.clone(),
                        timestamp: e.timestamp,
                        action: e.action,
                        source: e.source,
                        rule_id: e.rule_id.unwrap_or_default(),
                        rule_description: e.rule_description.unwrap_or_default(),
                        client_ip: e.client_ip,
                        client_country: e.client_country.unwrap_or_else(|| "XX".to_string()),
                        client_asn: e
                            .client_asn
                            .as_deref()
                            .and_then(|s| s.trim_start_matches("AS").parse::<u32>().ok())
                            .unwrap_or(0),
                        client_as_org: e.client_as_org.unwrap_or_default(),
                        http_method: e.http_method.unwrap_or_else(|| "GET".to_string()),
                        http_host: e.http_host.unwrap_or_else(|| z.name.clone()),
                        http_path: e.http_path.unwrap_or_else(|| "/".to_string()),
                        user_agent: e.user_agent.unwrap_or_default(),
                    })
                    .collect();
                let _ = db.replace_security_events(&z.id, &cached);
            }

            // 4. Sync WAF Rules for zone
            if let Ok(waf_rules) = client.get_waf_rules(&z.id).await {
                report.waf_rules_count += waf_rules.len();
                let cached: Vec<crate::db::CachedWafRule> = waf_rules
                    .into_iter()
                    .map(|r| crate::db::CachedWafRule {
                        id: r.id,
                        zone_id: z.id.clone(),
                        description: r.description,
                        action: r.action,
                        expression: r.expression,
                        paused: r.paused,
                        priority: r.priority.unwrap_or(0) as u32,
                        ref_code: r.reference.unwrap_or_default(),
                    })
                    .collect();
                let _ = db.replace_waf_rules(&z.id, &cached);
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

                // D1 Databases
                if let Ok(cf_d1) = client.get_d1_databases(&account.id).await {
                    report.d1_count = cf_d1.len();
                    let dbs: Vec<crate::db::CachedD1Database> = cf_d1
                        .into_iter()
                        .map(|d| crate::db::CachedD1Database {
                            uuid: d.uuid,
                            name: d.name,
                            version: d.version.unwrap_or_else(|| "production".to_string()),
                            num_tables: d.num_tables.unwrap_or(0),
                            file_size: d.file_size.unwrap_or(0),
                            created_at: d.created_at.unwrap_or_else(|| "Active".to_string()),
                        })
                        .collect();
                    let _ = db.replace_d1_databases(&dbs);
                }

                // KV Namespaces
                if let Ok(cf_kv) = client.get_kv_namespaces(&account.id).await {
                    report.kv_count = cf_kv.len();
                    let nss: Vec<crate::db::CachedKvNamespace> = cf_kv
                        .into_iter()
                        .map(|k| crate::db::CachedKvNamespace {
                            id: k.id,
                            title: k.title,
                            supports_url_encoding: k.supports_url_encoding.unwrap_or(false),
                        })
                        .collect();
                    let _ = db.replace_kv_namespaces(&nss);
                }
            }
        }

        Ok(report)
    }
}
