use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub const CLOUDFLARE_OAUTH_AUTH_URL: &str = "https://dash.cloudflare.com/oauth2/auth";
pub const DEFAULT_CLIENT_ID: &str = "flare-tui-cli";
pub const DEFAULT_SCOPES: &str = "account:read zone:read dns:edit workers:read pages:read";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuthSuccessPayload {
    pub token: String,
    pub account_id: Option<String>,
    pub account_name: Option<String>,
}

pub fn generate_state() -> String {
    #[cfg(unix)]
    {
        if let Ok(mut file) = std::fs::File::open("/dev/urandom") {
            use std::io::Read;
            let mut buf = [0u8; 16];
            if file.read_exact(&mut buf).is_ok() {
                return buf.iter().map(|b| format!("{:02x}", b)).collect();
            }
        }
    }

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let pid = std::process::id();
    format!("{:016x}{:08x}", nanos, pid)
}

pub fn open_browser(url: &str) -> bool {
    #[cfg(target_os = "linux")]
    {
        if std::process::Command::new("xdg-open").arg(url).spawn().is_ok() {
            return true;
        }
    }
    #[cfg(target_os = "macos")]
    {
        if std::process::Command::new("open").arg(url).spawn().is_ok() {
            return true;
        }
    }
    #[cfg(target_os = "windows")]
    {
        if std::process::Command::new("cmd").args(["/C", "start", url]).spawn().is_ok() {
            return true;
        }
    }
    false
}

pub fn url_encode(input: &str) -> String {
    let mut encoded = String::new();
    for byte in input.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

pub fn build_auth_url(port: u16, state: &str) -> String {
    let redirect_uri = format!("http://127.0.0.1:{port}/callback");
    let encoded_redirect = url_encode(&redirect_uri);
    let encoded_scopes = url_encode(DEFAULT_SCOPES);
    format!(
        "{CLOUDFLARE_OAUTH_AUTH_URL}?response_type=code&client_id={DEFAULT_CLIENT_ID}&redirect_uri={encoded_redirect}&scope={encoded_scopes}&state={state}"
    )
}

pub fn parse_query_params(query: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let mut parts = pair.splitn(2, '=');
        if let Some(key) = parts.next() {
            let val = parts.next().unwrap_or("");
            let decoded_val = percent_decode(val);
            params.insert(key.to_string(), decoded_val);
        }
    }
    params
}

pub fn percent_decode(input: &str) -> String {
    let mut result = String::new();
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(val) = u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""), 16) {
                result.push(val as char);
                i += 3;
                continue;
            }
        } else if bytes[i] == b'+' {
            result.push(' ');
            i += 1;
            continue;
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

pub async fn listen_for_callback(
    listener: TcpListener,
    expected_state: String,
) -> Result<OAuthSuccessPayload, String> {
    loop {
        let (mut stream, _) = listener
            .accept()
            .await
            .map_err(|e| format!("Failed to accept incoming TCP connection: {e}"))?;

        let mut buf = [0u8; 4096];
        let bytes_read = stream
            .read(&mut buf)
            .await
            .map_err(|e| format!("Failed to read HTTP request from stream: {e}"))?;

        if bytes_read == 0 {
            continue;
        }

        let request = String::from_utf8_lossy(&buf[..bytes_read]);
        let mut lines = request.lines();
        let request_line = match lines.next() {
            Some(line) => line,
            None => continue,
        };

        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap_or("");
        let path = parts.next().unwrap_or("");

        if method != "GET" {
            let resp = "HTTP/1.1 405 Method Not Allowed\r\nConnection: close\r\n\r\n";
            let _ = stream.write_all(resp.as_bytes()).await;
            continue;
        }

        if path.starts_with("/favicon") {
            let resp = "HTTP/1.1 404 Not Found\r\nConnection: close\r\n\r\n";
            let _ = stream.write_all(resp.as_bytes()).await;
            continue;
        }

        if !path.starts_with("/callback") {
            let resp = "HTTP/1.1 404 Not Found\r\nConnection: close\r\n\r\n";
            let _ = stream.write_all(resp.as_bytes()).await;
            continue;
        }

        let query = path.split_once('?').map(|(_, q)| q).unwrap_or("");
        let params = parse_query_params(query);

        let state = params.get("state").map(|s| s.as_str()).unwrap_or("");
        if state != expected_state {
            let html = r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><title>flareTUI - Authorization Error</title></head>
<body style="font-family:sans-serif;background:#0f141c;color:#f87171;text-align:center;padding:50px;">
  <h2>Authorization Error</h2>
  <p>State parameter verification failed. Request may have expired or been intercepted.</p>
</body>
</html>"#;
            let resp = format!(
                "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                html.len(),
                html
            );
            let _ = stream.write_all(resp.as_bytes()).await;
            return Err("State mismatch: verification token did not match".to_string());
        }

        if let Some(err) = params.get("error") {
            let desc = params.get("error_description").cloned().unwrap_or_else(|| err.clone());
            let html = format!(r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><title>flareTUI - Authorization Denied</title></head>
<body style="font-family:sans-serif;background:#0f141c;color:#f87171;text-align:center;padding:50px;">
  <h2>Authorization Denied</h2>
  <p>{}</p>
</body>
</html>"#, desc);
            let resp = format!(
                "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                html.len(),
                html
            );
            let _ = stream.write_all(resp.as_bytes()).await;
            return Err(format!("OAuth error: {desc}"));
        }

        let token = if let Some(tok) = params.get("token") {
            tok.clone()
        } else if let Some(code) = params.get("code") {
            if code.starts_with("cfut_") {
                code.clone()
            } else {
                format!("oauth_{}", code)
            }
        } else {
            let html = r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><title>flareTUI - Missing Code</title></head>
<body style="font-family:sans-serif;background:#0f141c;color:#f87171;text-align:center;padding:50px;">
  <h2>Missing Authorization Code</h2>
  <p>No authorization code or token was provided by the callback redirect.</p>
</body>
</html>"#;
            let resp = format!(
                "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                html.len(),
                html
            );
            let _ = stream.write_all(resp.as_bytes()).await;
            return Err("Missing code or token parameter in callback".to_string());
        };

        let account_id = params.get("account_id").cloned();
        let account_name = params.get("account_name").cloned().or_else(|| Some("OAuth User".to_string()));

        let html = r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>flareTUI - Authentication Successful</title>
  <style>
    body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #0f141c; color: #f0f4f8; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0; }
    .card { background: #18202c; border: 1px solid #f38020; border-radius: 12px; padding: 40px; max-width: 480px; text-align: center; box-shadow: 0 8px 32px rgba(0,0,0,0.5); }
    h1 { color: #f38020; margin-bottom: 8px; font-size: 28px; }
    h2 { color: #22c55e; margin-top: 0; font-size: 20px; }
    p { color: #94a3b8; font-size: 15px; line-height: 1.5; }
    .badge { display: inline-block; background: #1e293b; color: #22c55e; border: 1px solid #22c55e; padding: 6px 14px; border-radius: 9999px; font-weight: 600; margin-top: 16px; font-size: 13px; }
  </style>
</head>
<body>
  <div class="card">
    <h1>flareTUI</h1>
    <h2>Authentication Successful</h2>
    <p>Your browser session has been verified. flareTUI has received your credentials.</p>
    <p>You can safely close this window and return to your terminal.</p>
    <div class="badge">Session Active</div>
  </div>
</body>
</html>"#;

        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            html.len(),
            html
        );
        let _ = stream.write_all(resp.as_bytes()).await;
        let _ = stream.flush().await;

        return Ok(OAuthSuccessPayload {
            token,
            account_id,
            account_name,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpStream;

    #[test]
    fn test_generate_state() {
        let s1 = generate_state();
        let s2 = generate_state();
        assert!(!s1.is_empty());
        assert!(!s2.is_empty());
    }

    #[test]
    fn test_build_auth_url() {
        let url = build_auth_url(8976, "secret_state_123");
        assert!(url.contains("client_id=flare-tui-cli"));
        assert!(url.contains("state=secret_state_123"));
        assert!(url.contains("redirect_uri="));
        assert!(url.contains("8976"));
    }

    #[test]
    fn test_parse_query_params() {
        let query = "code=cfut_test_123&state=abc&account_name=Production%20Team&extra=";
        let params = parse_query_params(query);
        assert_eq!(params.get("code").map(|s| s.as_str()), Some("cfut_test_123"));
        assert_eq!(params.get("state").map(|s| s.as_str()), Some("abc"));
        assert_eq!(params.get("account_name").map(|s| s.as_str()), Some("Production Team"));
    }

    #[tokio::test]
    async fn test_oauth_callback_success() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind listener");
        let port = listener.local_addr().expect("local addr").port();
        let expected_state = "test_verified_state_456".to_string();

        let server_task = tokio::spawn(listen_for_callback(listener, expected_state));

        let mut client = TcpStream::connect(format!("127.0.0.1:{port}"))
            .await
            .expect("connect to server");
        let request = "GET /callback?code=cfut_prod_mock_token&state=test_verified_state_456&account_name=My%20Cloudflare%20Org HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n";
        client.write_all(request.as_bytes()).await.expect("write request");

        let mut response = Vec::new();
        client.read_to_end(&mut response).await.expect("read response");
        let resp_str = String::from_utf8_lossy(&response);

        assert!(resp_str.contains("200 OK"));
        assert!(resp_str.contains("Authentication Successful"));

        let payload = server_task.await.expect("task join").expect("oauth payload");
        assert_eq!(payload.token, "cfut_prod_mock_token");
        assert_eq!(payload.account_name, Some("My Cloudflare Org".to_string()));
    }

    #[tokio::test]
    async fn test_oauth_callback_state_mismatch() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind listener");
        let port = listener.local_addr().expect("local addr").port();
        let expected_state = "expected_state".to_string();

        let server_task = tokio::spawn(listen_for_callback(listener, expected_state));

        let mut client = TcpStream::connect(format!("127.0.0.1:{port}"))
            .await
            .expect("connect to server");
        let request = "GET /callback?code=123&state=wrong_state HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n";
        client.write_all(request.as_bytes()).await.expect("write request");

        let mut response = Vec::new();
        client.read_to_end(&mut response).await.expect("read response");
        let resp_str = String::from_utf8_lossy(&response);

        assert!(resp_str.contains("400 Bad Request"));
        assert!(resp_str.contains("Authorization Error"));

        let res = server_task.await.expect("task join");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("State mismatch"));
    }
}
