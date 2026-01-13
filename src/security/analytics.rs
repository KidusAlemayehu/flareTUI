use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::db::{CachedSecurityEvent, CachedWafRule};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreatLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl ThreatLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThreatLevel::Low => "LOW",
            ThreatLevel::Medium => "MEDIUM",
            ThreatLevel::High => "HIGH",
            ThreatLevel::Critical => "CRITICAL",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionBreakdown {
    pub action: String,
    pub count: usize,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CountryStat {
    pub country: String,
    pub count: usize,
    pub percentage: f64,
    pub blocked_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AsnStat {
    pub asn: u32,
    pub org: String,
    pub count: usize,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PathStat {
    pub path: String,
    pub count: usize,
    pub top_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuleHitStat {
    pub rule_id: String,
    pub description: String,
    pub action: String,
    pub hits: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimeSeriesBucket {
    pub label: String,
    pub total: usize,
    pub blocked: usize,
    pub challenged: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SecurityAnalyticsReport {
    pub total_events: usize,
    pub threats_blocked: usize,
    pub threats_challenged: usize,
    pub rate_limited: usize,
    pub threat_score: u8,
    pub threat_level: ThreatLevel,
    pub action_breakdown: Vec<ActionBreakdown>,
    pub top_countries: Vec<CountryStat>,
    pub top_asns: Vec<AsnStat>,
    pub top_paths: Vec<PathStat>,
    pub rule_hits: Vec<RuleHitStat>,
    pub timeline: Vec<TimeSeriesBucket>,
}

pub struct SecurityAnalyticsEngine;

impl SecurityAnalyticsEngine {
    pub fn calculate_threat_score(events: &[CachedSecurityEvent]) -> (u8, ThreatLevel) {
        if events.is_empty() {
            return (0, ThreatLevel::Low);
        }

        let total = events.len() as f64;
        let mut blocked_count = 0usize;
        let mut rate_limited_count = 0usize;
        let mut exploit_pattern_count = 0usize;

        for ev in events {
            let act = ev.action.to_lowercase();
            if act == "block" || act.contains("drop") {
                blocked_count += 1;
            } else if act.contains("rate") || ev.source.to_lowercase().contains("rate") {
                rate_limited_count += 1;
            }

            let desc = ev.rule_description.to_lowercase();
            let path = ev.http_path.to_lowercase();
            if desc.contains("sql")
                || desc.contains("inject")
                || desc.contains("xss")
                || desc.contains("rce")
                || desc.contains("exploit")
                || desc.contains("traversal")
                || path.contains(".env")
                || path.contains("wp-admin")
                || path.contains("etc/passwd")
            {
                exploit_pattern_count += 1;
            }
        }

        // Volume score: up to 35 pts (e.g. 50 events = 35 pts)
        let volume_score = ((total / 50.0) * 35.0).min(35.0);

        // Block ratio severity: up to 35 pts
        let block_ratio = blocked_count as f64 / total;
        let block_score = block_ratio * 35.0;

        // Rate limit score: up to 15 pts
        let rate_score = ((rate_limited_count as f64 / total) * 15.0).min(15.0);

        // Exploit pattern bonus: up to 15 pts
        let exploit_score = ((exploit_pattern_count as f64 * 3.0)).min(15.0);

        let final_score = (volume_score + block_score + rate_score + exploit_score).round() as u8;
        let score = final_score.clamp(0, 100);

        let level = match score {
            0..=24 => ThreatLevel::Low,
            25..=59 => ThreatLevel::Medium,
            60..=84 => ThreatLevel::High,
            _ => ThreatLevel::Critical,
        };

        (score, level)
    }

    pub fn analyze(events: &[CachedSecurityEvent], rules: &[CachedWafRule]) -> SecurityAnalyticsReport {
        let (threat_score, threat_level) = Self::calculate_threat_score(events);
        let total_events = events.len();

        let mut threats_blocked = 0;
        let mut threats_challenged = 0;
        let mut rate_limited = 0;

        let mut action_counts: HashMap<String, usize> = HashMap::new();
        let mut country_counts: HashMap<String, (usize, usize)> = HashMap::new();
        let mut asn_counts: HashMap<(u32, String), usize> = HashMap::new();
        let mut path_counts: HashMap<String, (usize, HashMap<String, usize>)> = HashMap::new();
        let mut rule_hit_counts: HashMap<String, usize> = HashMap::new();

        for ev in events {
            let act_lower = ev.action.to_lowercase();
            *action_counts.entry(ev.action.clone()).or_insert(0) += 1;

            let is_block = act_lower == "block" || act_lower.contains("drop");
            let is_challenge = act_lower.contains("challenge");
            let is_rate = act_lower.contains("rate") || ev.source.to_lowercase().contains("rate");

            if is_block {
                threats_blocked += 1;
            }
            if is_challenge {
                threats_challenged += 1;
            }
            if is_rate {
                rate_limited += 1;
            }

            let c_entry = country_counts.entry(ev.client_country.clone()).or_insert((0, 0));
            c_entry.0 += 1;
            if is_block {
                c_entry.1 += 1;
            }

            *asn_counts.entry((ev.client_asn, ev.client_as_org.clone())).or_insert(0) += 1;

            let p_entry = path_counts.entry(ev.http_path.clone()).or_insert((0, HashMap::new()));
            p_entry.0 += 1;
            *p_entry.1.entry(ev.action.clone()).or_insert(0) += 1;

            if !ev.rule_id.is_empty() {
                *rule_hit_counts.entry(ev.rule_id.clone()).or_insert(0) += 1;
            }
        }

        // Action breakdown
        let mut action_breakdown: Vec<ActionBreakdown> = action_counts
            .into_iter()
            .map(|(action, count)| {
                let percentage = if total_events > 0 {
                    (count as f64 / total_events as f64) * 100.0
                } else {
                    0.0
                };
                ActionBreakdown {
                    action,
                    count,
                    percentage,
                }
            })
            .collect();
        action_breakdown.sort_by(|a, b| b.count.cmp(&a.count));

        // Top countries
        let mut top_countries: Vec<CountryStat> = country_counts
            .into_iter()
            .map(|(country, (count, blocked_count))| {
                let percentage = if total_events > 0 {
                    (count as f64 / total_events as f64) * 100.0
                } else {
                    0.0
                };
                CountryStat {
                    country,
                    count,
                    percentage,
                    blocked_count,
                }
            })
            .collect();
        top_countries.sort_by(|a, b| b.count.cmp(&a.count));

        // Top ASNs
        let mut top_asns: Vec<AsnStat> = asn_counts
            .into_iter()
            .map(|((asn, org), count)| {
                let percentage = if total_events > 0 {
                    (count as f64 / total_events as f64) * 100.0
                } else {
                    0.0
                };
                AsnStat {
                    asn,
                    org,
                    count,
                    percentage,
                }
            })
            .collect();
        top_asns.sort_by(|a, b| b.count.cmp(&a.count));

        // Top paths
        let mut top_paths: Vec<PathStat> = path_counts
            .into_iter()
            .map(|(path, (count, acts))| {
                let top_action = acts
                    .into_iter()
                    .max_by_key(|(_, c)| *c)
                    .map(|(a, _)| a)
                    .unwrap_or_else(|| "block".to_string());
                PathStat {
                    path,
                    count,
                    top_action,
                }
            })
            .collect();
        top_paths.sort_by(|a, b| b.count.cmp(&a.count));

        // Rule hits
        let mut rule_hits: Vec<RuleHitStat> = Vec::new();
        for rule in rules {
            let hits = rule_hit_counts.remove(&rule.id).unwrap_or(0);
            rule_hits.push(RuleHitStat {
                rule_id: rule.id.clone(),
                description: rule.description.clone(),
                action: rule.action.clone(),
                hits,
            });
        }
        for (r_id, hits) in rule_hit_counts {
            rule_hits.push(RuleHitStat {
                rule_id: r_id.clone(),
                description: format!("Rule {r_id}"),
                action: "waf".to_string(),
                hits,
            });
        }
        rule_hits.sort_by(|a, b| b.hits.cmp(&a.hits));

        // Timeline bucket grouping
        let timeline = Self::generate_timeline(events);

        SecurityAnalyticsReport {
            total_events,
            threats_blocked,
            threats_challenged,
            rate_limited,
            threat_score,
            threat_level,
            action_breakdown,
            top_countries,
            top_asns,
            top_paths,
            rule_hits,
            timeline,
        }
    }

    fn generate_timeline(events: &[CachedSecurityEvent]) -> Vec<TimeSeriesBucket> {
        if events.is_empty() {
            return Vec::new();
        }

        let mut sorted = events.to_vec();
        sorted.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        let bucket_count = 8.min(sorted.len());
        let chunk_size = (sorted.len() + bucket_count - 1) / bucket_count;
        let mut buckets = Vec::new();

        for chunk in sorted.chunks(chunk_size) {
            if let Some(first) = chunk.first() {
                let label = if first.timestamp.len() >= 16 {
                    first.timestamp[11..16].to_string()
                } else {
                    first.timestamp.clone()
                };

                let total = chunk.len();
                let blocked = chunk.iter().filter(|e| e.action.to_lowercase().contains("block")).count();
                let challenged = chunk.iter().filter(|e| e.action.to_lowercase().contains("challenge")).count();

                buckets.push(TimeSeriesBucket {
                    label,
                    total,
                    blocked,
                    challenged,
                });
            }
        }

        buckets
    }

    pub fn filter_events(
        events: &[CachedSecurityEvent],
        action_filter: Option<&str>,
        query: Option<&str>,
    ) -> Vec<CachedSecurityEvent> {
        events
            .iter()
            .filter(|e| {
                if let Some(act) = action_filter {
                    if !act.is_empty() && act != "ALL" && !e.action.eq_ignore_ascii_case(act) {
                        return false;
                    }
                }
                if let Some(q) = query {
                    let q_lower = q.trim().to_lowercase();
                    if !q_lower.is_empty() {
                        let ip_match = e.client_ip.to_lowercase().contains(&q_lower);
                        let country_match = e.client_country.to_lowercase().contains(&q_lower);
                        let path_match = e.http_path.to_lowercase().contains(&q_lower);
                        let rule_match = e.rule_description.to_lowercase().contains(&q_lower)
                            || e.rule_id.to_lowercase().contains(&q_lower);
                        let org_match = e.client_as_org.to_lowercase().contains(&q_lower);
                        let ray_match = e.ray_id.to_lowercase().contains(&q_lower);

                        if !ip_match && !country_match && !path_match && !rule_match && !org_match && !ray_match {
                            return false;
                        }
                    }
                }
                true
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_events() -> Vec<CachedSecurityEvent> {
        vec![
            CachedSecurityEvent {
                ray_id: "8472a1bf001".to_string(),
                zone_id: "zone-1".to_string(),
                timestamp: "2026-01-10T10:00:00Z".to_string(),
                action: "block".to_string(),
                source: "firewallCustom".to_string(),
                rule_id: "rule-sqli".to_string(),
                rule_description: "Block SQL injection attack".to_string(),
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
                zone_id: "zone-1".to_string(),
                timestamp: "2026-01-10T10:05:00Z".to_string(),
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
                http_path: "/products".to_string(),
                user_agent: "Mozilla/5.0 Bot".to_string(),
            },
            CachedSecurityEvent {
                ray_id: "8472a1bf003".to_string(),
                zone_id: "zone-1".to_string(),
                timestamp: "2026-01-10T10:10:00Z".to_string(),
                action: "block".to_string(),
                source: "rateLimit".to_string(),
                rule_id: "rule-rate-api".to_string(),
                rule_description: "API request rate limit exceeded".to_string(),
                client_ip: "192.0.2.88".to_string(),
                client_country: "US".to_string(),
                client_asn: 15169,
                client_as_org: "Google LLC".to_string(),
                http_method: "POST".to_string(),
                http_host: "api.example.com".to_string(),
                http_path: "/login".to_string(),
                user_agent: "curl/7.88.1".to_string(),
            },
        ]
    }

    fn sample_rules() -> Vec<CachedWafRule> {
        vec![
            CachedWafRule {
                id: "rule-sqli".to_string(),
                zone_id: "zone-1".to_string(),
                description: "Block SQL injection attack".to_string(),
                action: "block".to_string(),
                expression: "(http.request.uri.path contains \"/login\")".to_string(),
                paused: false,
                priority: 1,
                ref_code: "SQLI_1".to_string(),
            },
            CachedWafRule {
                id: "rule-bot".to_string(),
                zone_id: "zone-1".to_string(),
                description: "Challenge automated bots".to_string(),
                action: "managed_challenge".to_string(),
                expression: "(cf.client.bot)".to_string(),
                paused: false,
                priority: 2,
                ref_code: "BOT_2".to_string(),
            },
        ]
    }

    #[test]
    fn test_threat_score_calculation_empty() {
        let (score, level) = SecurityAnalyticsEngine::calculate_threat_score(&[]);
        assert_eq!(score, 0);
        assert_eq!(level, ThreatLevel::Low);
    }

    #[test]
    fn test_threat_score_calculation() {
        let events = sample_events();
        let (score, level) = SecurityAnalyticsEngine::calculate_threat_score(&events);
        assert!(score > 0);
        assert!(level == ThreatLevel::Medium || level == ThreatLevel::High);
    }

    #[test]
    fn test_security_analytics_aggregation() {
        let events = sample_events();
        let rules = sample_rules();
        let report = SecurityAnalyticsEngine::analyze(&events, &rules);

        assert_eq!(report.total_events, 3);
        assert_eq!(report.threats_blocked, 2);
        assert_eq!(report.threats_challenged, 1);
        assert_eq!(report.rate_limited, 1);

        // Action breakdown
        assert_eq!(report.action_breakdown[0].action, "block");
        assert_eq!(report.action_breakdown[0].count, 2);

        // Top countries
        assert_eq!(report.top_countries[0].country, "US");
        assert_eq!(report.top_countries[0].count, 2);
        assert_eq!(report.top_countries[0].blocked_count, 2);

        // Top paths
        assert_eq!(report.top_paths[0].path, "/login");
        assert_eq!(report.top_paths[0].count, 2);

        // Rule hits
        let sqli_hit = report.rule_hits.iter().find(|r| r.rule_id == "rule-sqli").expect("rule found");
        assert_eq!(sqli_hit.hits, 1);
    }

    #[test]
    fn test_filter_events_by_action_and_query() {
        let events = sample_events();

        // Filter by action
        let blocked = SecurityAnalyticsEngine::filter_events(&events, Some("block"), None);
        assert_eq!(blocked.len(), 2);

        // Filter by search query (IP)
        let filtered_ip = SecurityAnalyticsEngine::filter_events(&events, None, Some("198.51.100.42"));
        assert_eq!(filtered_ip.len(), 1);
        assert_eq!(filtered_ip[0].ray_id, "8472a1bf001");

        // Filter by search query (Path)
        let filtered_path = SecurityAnalyticsEngine::filter_events(&events, None, Some("/login"));
        assert_eq!(filtered_path.len(), 2);

        // Combined filter
        let combined = SecurityAnalyticsEngine::filter_events(&events, Some("managed_challenge"), Some("/products"));
        assert_eq!(combined.len(), 1);
        assert_eq!(combined[0].ray_id, "8472a1bf002");
    }

    #[test]
    fn test_timeline_generation() {
        let events = sample_events();
        let timeline = SecurityAnalyticsEngine::generate_timeline(&events);
        assert!(!timeline.is_empty());
        assert_eq!(timeline[0].label, "10:00");
    }
}
