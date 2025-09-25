use color_eyre::Result;
use futures_util::StreamExt;
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::connect_async;
use reqwest::Url;

use crate::components::views::tail_view::LogEntry;

pub struct TailStream;

impl TailStream {
    pub async fn start(ws_url: String, log_tx: UnboundedSender<LogEntry>) -> Result<()> {
        let (ws_stream, _) = connect_async(ws_url.as_str()).await?;
        let (_, mut read) = ws_stream.split();

        tokio::spawn(async move {
            while let Some(msg_result) = read.next().await {
                if let Ok(msg) = msg_result {
                    if let Ok(text) = msg.to_text() {
                        if let Ok(json) = serde_json::from_str::<Value>(text) {
                            if let Some(entry) = Self::parse_cloudflare_tail_message(&json) {
                                if log_tx.send(entry).is_err() {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    fn parse_cloudflare_tail_message(json: &Value) -> Option<LogEntry> {
        let request = json.get("event")?.get("request")?;
        let method = request.get("method")?.as_str()?.to_string();
        let url_str = request.get("url")?.as_str()?.to_string();
        let parsed_path = Url::parse(&url_str)
            .map(|u| u.path().to_string())
            .unwrap_or(url_str);

        let outcome = json.get("outcome").and_then(|o| o.as_str()).unwrap_or("ok");
        let status = match outcome {
            "ok" => 200,
            "exception" => 500,
            "exceededCpu" => 503,
            _ => 200,
        };

        let mut log_msgs = Vec::new();
        if let Some(logs) = json.get("logs").and_then(|l| l.as_array()) {
            for l in logs {
                if let Some(msg_arr) = l.get("message").and_then(|m| m.as_array()) {
                    let combined = msg_arr
                        .iter()
                        .map(|v| v.as_str().unwrap_or(&v.to_string()).to_string())
                        .collect::<Vec<_>>()
                        .join(" ");
                    log_msgs.push(combined);
                }
            }
        }

        let message = if log_msgs.is_empty() {
            format!("Worker execution finished [outcome: {outcome}]")
        } else {
            log_msgs.join(" | ")
        };

        // Timestamp
        let timestamp = chrono_or_simple_time();

        Some(LogEntry {
            timestamp,
            method,
            url: parsed_path,
            status,
            duration_ms: 18,
            message,
        })
    }
}

fn chrono_or_simple_time() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let seconds = (now / 1000) % 60;
    let minutes = (now / 60000) % 60;
    let hours = (now / 3600000) % 24;
    let millis = now % 1000;
    format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis)
}
