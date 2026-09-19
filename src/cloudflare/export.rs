use color_eyre::Result;
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

use crate::db::DnsRecord;

pub struct ZoneExporter;

impl ZoneExporter {
    pub fn export_bind_zone(domain: &str, records: &[DnsRecord]) -> Result<PathBuf> {
        let mut content = String::new();
        content.push_str("; BIND Zone File exported by flareTUI\n");
        content.push_str(&format!("; Zone: {domain}\n"));
        content.push_str(&format!("$ORIGIN {domain}.\n"));
        content.push_str("$TTL 300\n\n");

        for r in records {
            let name = if r.name == domain {
                "@"
            } else if r.name.ends_with(domain) {
                let trimmed = r.name.trim_end_matches(domain).trim_end_matches('.');
                if trimmed.is_empty() {
                    "@"
                } else {
                    trimmed
                }
            } else {
                r.name.as_str()
            };

            let ttl = if r.ttl == 1 { 300 } else { r.ttl };
            let target = match r.record_type.as_str() {
                "CNAME" | "NS" | "MX" => {
                    if r.content.ends_with('.') {
                        r.content.clone()
                    } else {
                        format!("{}.", r.content)
                    }
                }
                "TXT" => format!("\"{}\"", r.content),
                _ => r.content.clone(),
            };

            content.push_str(&format!(
                "{:<20} {:<6} IN  {:<6} {}\n",
                name, ttl, r.record_type, target
            ));
        }

        let export_dir = if let Some(dirs) = ProjectDirs::from("com", "flare", "flareTUI") {
            let dir = dirs.config_dir().join("exports");
            let _ = fs::create_dir_all(&dir);
            dir
        } else {
            PathBuf::from(".")
        };

        let file_path = export_dir.join(format!("{domain}.zone"));
        fs::write(&file_path, content)?;
        Ok(file_path)
    }
}
