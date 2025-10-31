use color_eyre::Result;
use futures_util::StreamExt;
use reqwest::Url;
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch;
use tokio_tungstenite::connect_async;

use crate::components::views::tail_view::LogEntry;

pub struct TailStream;

impl TailStream {
    pub async fn start(
        ws_url: String,
        log_tx: UnboundedSender<LogEntry>,
        mut cancel_rx: watch::Receiver<bool>,
    ) -> Result<()> {
        let (ws_stream, _) = connect_async(ws_url.as_str()).await?;
        let (_, mut read) = ws_stream.split();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel_rx.changed() => {
                        if *cancel_rx.borrow() {
                            break;
                        }
                    }
                    msg_result = read.next() => {
                        match msg_result {
                            Some(Ok(msg)) => {
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
                            Some(Err(_)) | None => break,
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub fn parse_cloudflare_tail_message(json: &Value) -> Option<LogEntry> {
        let (method, url_str) = if let Some(request) = json.get("event").and_then(|e| e.get("request")) {
            let m = request.get("method").and_then(|v| v.as_str()).unwrap_or("GET").to_string();
            let u = request.get("url").and_then(|v| v.as_str()).unwrap_or("/").to_string();
            (m, u)
        } else if let Some(sched) = json.get("event").and_then(|e| e.get("cron")) {
            ("CRON".to_string(), sched.as_str().unwrap_or("cron").to_string())
        } else if let Some(queue) = json.get("event").and_then(|e| e.get("queue")) {
            ("QUEUE".to_string(), queue.as_str().unwrap_or("queue").to_string())
        } else {
            ("EVENT".to_string(), "handler".to_string())
        };

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
        let mut log_level = None;
        if let Some(logs) = json.get("logs").and_then(|l| l.as_array()) {
            for l in logs {
                if let Some(lvl) = l.get("level").and_then(|v| v.as_str()) {
                    log_level = Some(lvl.to_string());
                }
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

        let mut exceptions = Vec::new();
        if let Some(exc_arr) = json.get("exceptions").and_then(|e| e.as_array()) {
            for exc in exc_arr {
                let name = exc.get("name").and_then(|v| v.as_str()).unwrap_or("Error");
                let message = exc.get("message").and_then(|v| v.as_str()).unwrap_or("Unknown error");
                exceptions.push(format!("{name}: {message}"));
            }
        }

        let message = if !exceptions.is_empty() {
            format!("❌ {}", exceptions.join(" | "))
        } else if log_msgs.is_empty() {
            format!("Worker execution finished [outcome: {outcome}]")
        } else {
            log_msgs.join(" | ")
        };

        let timestamp = chrono_or_simple_time();

        Some(LogEntry {
            timestamp,
            method,
            url: parsed_path,
            status,
            duration_ms: 18,
            message,
            raw_json: Some(serde_json::to_string_pretty(json).unwrap_or_default()),
            exceptions,
            log_level,
            outcome: outcome.to_string(),
        })
    }
}

pub fn chrono_or_simple_time() -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cloudflare_tail_http_message() {
        let json: Value = serde_json::json!({
            "outcome": "ok",
            "event": {
                "request": {
                    "method": "POST",
                    "url": "https://tg-naol-bot.workers.dev/webhook"
                }
            },
            "logs": [
                {
                    "level": "info",
                    "message": ["Received telegram update:", {"update_id": 12345}]
                }
            ],
            "exceptions": []
        });

        let entry = TailStream::parse_cloudflare_tail_message(&json).expect("parsed entry");
        assert_eq!(entry.method, "POST");
        assert_eq!(entry.url, "/webhook");
        assert_eq!(entry.status, 200);
        assert_eq!(entry.outcome, "ok");
        assert!(entry.message.contains("Received telegram update"));
        assert!(entry.raw_json.is_some());
    }

    #[test]
    fn test_parse_cloudflare_tail_exception_message() {
        let json: Value = serde_json::json!({
            "outcome": "exception",
            "event": {
                "request": {
                    "method": "GET",
                    "url": "https://api.example.com/crash"
                }
            },
            "logs": [],
            "exceptions": [
                {
                    "name": "TypeError",
                    "message": "Cannot read property 'id' of undefined"
                }
            ]
        });

        let entry = TailStream::parse_cloudflare_tail_message(&json).expect("parsed entry");
        assert_eq!(entry.method, "GET");
        assert_eq!(entry.status, 500);
        assert_eq!(entry.outcome, "exception");
        assert_eq!(entry.exceptions.len(), 1);
        assert!(entry.message.contains("TypeError: Cannot read property 'id' of undefined"));
    }
}
