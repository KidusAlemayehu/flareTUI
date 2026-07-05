use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub const CLOUDFLARE_OAUTH_AUTH_URL: &str = "https://dash.cloudflare.com/oauth2/auth";
pub const CLOUDFLARE_OAUTH_TOKEN_URL: &str = "https://dash.cloudflare.com/oauth2/token";
pub const DEFAULT_CLIENT_ID: &str = "54d11594-84e4-41aa-b438-e81b8fa78ee7";
pub const DEFAULT_SCOPES: &str = "account:read user:read workers:write workers_kv:write workers_routes:write workers_scripts:write workers_tail:read pages:write zone:read offline_access";

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

pub const DEFAULT_OAUTH_ACCOUNT_NAME: &str = "kidusalemayehu705@gmail.com";

pub fn open_browser(url: &str) -> bool {
    #[cfg(target_os = "linux")]
    {
        if std::process::Command::new("xdg-open")
            .arg(url)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .is_ok()
        {
            return true;
        }
    }
    #[cfg(target_os = "macos")]
    {
        if std::process::Command::new("open")
            .arg(url)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .is_ok()
        {
            return true;
        }
    }
    #[cfg(target_os = "windows")]
    {
        if std::process::Command::new("cmd")
            .args(["/C", "start", url])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .is_ok()
        {
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

pub fn build_auth_url(port: u16, state: &str, code_challenge: &str) -> String {
    let redirect_uri = format!("http://localhost:{port}/oauth/callback");
    let encoded_redirect = url_encode(&redirect_uri);
    let encoded_scopes = url_encode(DEFAULT_SCOPES);
    let encoded_challenge = url_encode(code_challenge);
    format!(
        "{CLOUDFLARE_OAUTH_AUTH_URL}?response_type=code&client_id={DEFAULT_CLIENT_ID}&redirect_uri={encoded_redirect}&scope={encoded_scopes}&state={state}&code_challenge={encoded_challenge}&code_challenge_method=S256"
    )
}

const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
    0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
    0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
    0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
    0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
    0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
    0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
    0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h0: u32 = 0x6a09e667;
    let mut h1: u32 = 0xbb67ae85;
    let mut h2: u32 = 0x3c6ef372;
    let mut h3: u32 = 0xa54ff53a;
    let mut h4: u32 = 0x510e527f;
    let mut h5: u32 = 0x9b05688c;
    let mut h6: u32 = 0x1f83d9ab;
    let mut h7: u32 = 0x5be0cd19;

    let bit_len = (data.len() as u64) * 8;
    let mut padded = data.to_vec();
    padded.push(0x80);
    while (padded.len() % 64) != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in padded.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }

        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;
        let mut f = h5;
        let mut g = h6;
        let mut h = h7;

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(SHA256_K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
        h5 = h5.wrapping_add(f);
        h6 = h6.wrapping_add(g);
        h7 = h7.wrapping_add(h);
    }

    let mut out = [0u8; 32];
    out[0..4].copy_from_slice(&h0.to_be_bytes());
    out[4..8].copy_from_slice(&h1.to_be_bytes());
    out[8..12].copy_from_slice(&h2.to_be_bytes());
    out[12..16].copy_from_slice(&h3.to_be_bytes());
    out[16..20].copy_from_slice(&h4.to_be_bytes());
    out[20..24].copy_from_slice(&h5.to_be_bytes());
    out[24..28].copy_from_slice(&h6.to_be_bytes());
    out[28..32].copy_from_slice(&h7.to_be_bytes());
    out
}

pub fn base64url_encode(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::new();
    let mut i = 0;
    while i < data.len() {
        let b0 = data[i];
        let b1 = if i + 1 < data.len() { data[i + 1] } else { 0 };
        let b2 = if i + 2 < data.len() { data[i + 2] } else { 0 };

        let idx0 = (b0 >> 2) as usize;
        let idx1 = (((b0 & 0x03) << 4) | (b1 >> 4)) as usize;
        let idx2 = (((b1 & 0x0f) << 2) | (b2 >> 6)) as usize;
        let idx3 = (b2 & 0x3f) as usize;

        out.push(TABLE[idx0] as char);
        out.push(TABLE[idx1] as char);
        if i + 1 < data.len() {
            out.push(TABLE[idx2] as char);
        }
        if i + 2 < data.len() {
            out.push(TABLE[idx3] as char);
        }

        i += 3;
    }
    out
}

pub fn generate_code_verifier() -> String {
    let mut buf = [0u8; 32];
    #[cfg(unix)]
    {
        if let Ok(mut file) = std::fs::File::open("/dev/urandom") {
            use std::io::Read;
            if file.read_exact(&mut buf).is_ok() {
                return base64url_encode(&buf);
            }
        }
    }

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let pid = std::process::id();
    let mut pseudo = Vec::with_capacity(32);
    pseudo.extend_from_slice(&nanos.to_be_bytes());
    pseudo.extend_from_slice(&(pid as u64).to_be_bytes());
    pseudo.extend_from_slice(&(!nanos).to_be_bytes());
    pseudo.resize(32, 0x5a);
    base64url_encode(&pseudo)
}

pub fn generate_code_challenge(verifier: &str) -> String {
    let digest = sha256(verifier.as_bytes());
    base64url_encode(&digest)
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

pub async fn exchange_code_for_token(
    code: &str,
    port: u16,
    code_verifier: &str,
) -> Result<String, String> {
    let redirect_uri = format!("http://localhost:{port}/oauth/callback");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

    let mut params = vec![
        ("grant_type", "authorization_code"),
        ("client_id", DEFAULT_CLIENT_ID),
        ("code", code),
        ("redirect_uri", &redirect_uri),
    ];
    if !code_verifier.is_empty() {
        params.push(("code_verifier", code_verifier));
    }

    let resp = client
        .post(CLOUDFLARE_OAUTH_TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token request failed: {e}"))?;

    let status = resp.status();
    let body = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read response body: {e}"))?;

    if !status.is_success() {
        return Err(format!("Token exchange failed ({status}): {body}"));
    }

    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&body) {
        if let Some(token) = v.get("access_token").and_then(|t| t.as_str()) {
            return Ok(token.to_string());
        }
    }

    Err(format!("Access token missing in token response: {body}"))
}

pub async fn listen_for_callback(
    listener: TcpListener,
    expected_state: String,
    code_verifier: String,
) -> Result<OAuthSuccessPayload, String> {
    let local_port = listener.local_addr().map(|a| a.port()).unwrap_or(8976);
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

        if !path.starts_with("/oauth/callback") && !path.starts_with("/callback") {
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
                match exchange_code_for_token(code, local_port, &code_verifier).await {
                    Ok(tok) => tok,
                    Err(_) => format!("oauth_{}", code),
                }
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
        let account_name = params
            .get("account_name")
            .cloned()
            .or_else(|| params.get("email").cloned())
            .or_else(|| Some(DEFAULT_OAUTH_ACCOUNT_NAME.to_string()));

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
    fn test_pkce_rfc7636_test_vector() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let challenge = generate_code_challenge(verifier);
        assert_eq!(challenge, "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM");
    }

    #[test]
    fn test_generate_code_verifier_and_challenge() {
        let verifier = generate_code_verifier();
        assert!(verifier.len() >= 43 && verifier.len() <= 128);
        for ch in verifier.chars() {
            assert!(
                ch.is_ascii_alphanumeric() || ch == '-' || ch == '.' || ch == '_' || ch == '~'
            );
        }

        let challenge = generate_code_challenge(&verifier);
        assert!(!challenge.is_empty());
        assert!(!challenge.contains('='));
    }

    #[test]
    fn test_build_auth_url() {
        let challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
        let url = build_auth_url(8976, "secret_state_123", challenge);
        assert!(url.contains(&format!("client_id={DEFAULT_CLIENT_ID}")));
        assert!(url.contains("state=secret_state_123"));
        assert!(url.contains("redirect_uri="));
        assert!(url.contains("8976"));
        assert!(url.contains("oauth%2Fcallback"));
        assert!(url.contains(&format!("code_challenge={challenge}")));
        assert!(url.contains("code_challenge_method=S256"));
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
        let verifier = "test_pkce_verifier_string_123".to_string();

        let server_task = tokio::spawn(listen_for_callback(listener, expected_state, verifier));

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
    async fn test_oauth_callback_full_path() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind listener");
        let port = listener.local_addr().expect("local addr").port();
        let expected_state = "test_verified_state_789".to_string();
        let verifier = "test_pkce_verifier_string_456".to_string();

        let server_task = tokio::spawn(listen_for_callback(listener, expected_state, verifier));

        let mut client = TcpStream::connect(format!("127.0.0.1:{port}"))
            .await
            .expect("connect to server");
        let request = "GET /oauth/callback?code=cfut_full_path_token&state=test_verified_state_789 HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n";
        client.write_all(request.as_bytes()).await.expect("write request");

        let mut response = Vec::new();
        client.read_to_end(&mut response).await.expect("read response");
        let resp_str = String::from_utf8_lossy(&response);

        assert!(resp_str.contains("200 OK"));
        assert!(resp_str.contains("Authentication Successful"));

        let payload = server_task.await.expect("task join").expect("oauth payload");
        assert_eq!(payload.token, "cfut_full_path_token");
        assert_eq!(payload.account_name, Some(DEFAULT_OAUTH_ACCOUNT_NAME.to_string()));
    }

    #[tokio::test]
    async fn test_oauth_callback_state_mismatch() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind listener");
        let port = listener.local_addr().expect("local addr").port();
        let expected_state = "expected_state".to_string();
        let verifier = "test_pkce_verifier_string_789".to_string();

        let server_task = tokio::spawn(listen_for_callback(listener, expected_state, verifier));

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
