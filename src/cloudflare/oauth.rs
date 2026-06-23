use std::time::{SystemTime, UNIX_EPOCH};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
