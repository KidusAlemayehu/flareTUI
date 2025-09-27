use color_eyre::Result;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub errors: Vec<ApiError>,
    pub result: Option<T>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiError {
    pub code: i64,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenVerifyResult {
    pub id: String,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Account {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ZoneResult {
    pub id: String,
    pub name: String,
    pub status: String,
    pub plan: Option<ZonePlan>,
    pub created_on: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ZonePlan {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DnsRecordResult {
    pub id: String,
    pub zone_id: String,
    #[serde(rename = "type")]
    pub record_type: String,
    pub name: String,
    pub content: String,
    pub proxied: bool,
    pub ttl: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct PatchDnsRecord {
    pub proxied: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkerScriptResult {
    pub id: String,
    pub routes: Option<Vec<WorkerRoute>>,
    pub modified_on: Option<String>,
    pub created_on: Option<String>,
    pub compatibility_date: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkerRoute {
    pub pattern: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PagesProjectResult {
    pub id: String,
    pub name: String,
    pub subdomain: String,
    pub production_branch: Option<String>,
    pub latest_deployment: Option<PagesDeployment>,
    pub updated_on: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PagesDeployment {
    pub latest_stage: Option<PagesStage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PagesStage {
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TailSessionResult {
    pub id: String,
    pub url: String,
    pub expires_at: Option<String>,
}

#[derive(Clone)]
pub struct CloudflareClient {
    client: Client,
    token: String,
    base_url: String,
}

impl CloudflareClient {
    pub fn new(token: String) -> Self {
        let mut headers = HeaderMap::new();
        let auth_val = format!("Bearer {token}");
        if let Ok(mut val) = HeaderValue::from_str(&auth_val) {
            val.set_sensitive(true);
            headers.insert(AUTHORIZATION, val);
        }
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .unwrap_or_default();

        Self {
            client,
            token,
            base_url: "https://api.cloudflare.com/client/v4".to_string(),
        }
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub async fn verify_token(&self) -> Result<TokenVerifyResult> {
        // 1. Try standard /user/tokens/verify endpoint (User API Tokens)
        let url = format!("{}/user/tokens/verify", self.base_url);
        if let Ok(resp) = self.client.get(&url).send().await {
            if let Ok(json) = resp.json::<ApiResponse<TokenVerifyResult>>().await {
                if json.success {
                    if let Some(res) = json.result {
                        return Ok(res);
                    }
                }
            }
        }

        // 2. Fallback: Test /zones or /accounts (Account API Tokens)
        let zones_url = format!("{}/zones?per_page=1", self.base_url);
        if let Ok(resp) = self.client.get(&zones_url).send().await {
            if let Ok(json) = resp.json::<ApiResponse<Vec<ZoneResult>>>().await {
                if json.success {
                    return Ok(TokenVerifyResult {
                        id: "account_token".to_string(),
                        status: "active".to_string(),
                    });
                }
            }
        }

        let accounts_url = format!("{}/accounts?per_page=1", self.base_url);
        if let Ok(resp) = self.client.get(&accounts_url).send().await {
            if let Ok(json) = resp.json::<ApiResponse<Vec<Account>>>().await {
                if json.success {
                    return Ok(TokenVerifyResult {
                        id: "account_token".to_string(),
                        status: "active".to_string(),
                    });
                }
            }
        }

        color_eyre::eyre::bail!("Token verification failed: Invalid API Token or missing permissions")
    }

    pub async fn get_accounts(&self) -> Result<Vec<Account>> {
        let url = format!("{}/accounts", self.base_url);
        let resp = self.client.get(&url).send().await?.json::<ApiResponse<Vec<Account>>>().await?;
        Ok(resp.result.unwrap_or_default())
    }

    pub async fn get_zones(&self) -> Result<Vec<ZoneResult>> {
        let url = format!("{}/zones?per_page=50", self.base_url);
        let resp = self.client.get(&url).send().await?.json::<ApiResponse<Vec<ZoneResult>>>().await?;
        Ok(resp.result.unwrap_or_default())
    }

    pub async fn get_dns_records(&self, zone_id: &str) -> Result<Vec<DnsRecordResult>> {
        let url = format!("{}/zones/{zone_id}/dns_records?per_page=100", self.base_url);
        let resp = self.client.get(&url).send().await?.json::<ApiResponse<Vec<DnsRecordResult>>>().await?;
        Ok(resp.result.unwrap_or_default())
    }

    pub async fn toggle_dns_proxy(&self, zone_id: &str, record_id: &str, current_proxied: bool) -> Result<bool> {
        let url = format!("{}/zones/{zone_id}/dns_records/{record_id}", self.base_url);
        let new_proxied = !current_proxied;
        let body = PatchDnsRecord { proxied: new_proxied };
        let resp = self.client.patch(&url).json(&body).send().await?.json::<ApiResponse<DnsRecordResult>>().await?;
        if resp.success {
            Ok(new_proxied)
        } else {
            let msg = resp.errors.first().map(|e| e.message.clone()).unwrap_or_else(|| "Failed to update proxy".to_string());
            color_eyre::eyre::bail!("{msg}")
        }
    }

    pub async fn get_workers(&self, account_id: &str) -> Result<Vec<WorkerScriptResult>> {
        let url = format!("{}/accounts/{account_id}/workers/scripts", self.base_url);
        let resp = self.client.get(&url).send().await?.json::<ApiResponse<Vec<WorkerScriptResult>>>().await?;
        Ok(resp.result.unwrap_or_default())
    }

    pub async fn get_pages(&self, account_id: &str) -> Result<Vec<PagesProjectResult>> {
        let url = format!("{}/accounts/{account_id}/pages/projects", self.base_url);
        let resp = self.client.get(&url).send().await?.json::<ApiResponse<Vec<PagesProjectResult>>>().await?;
        Ok(resp.result.unwrap_or_default())
    }

    pub async fn create_worker_tail(&self, account_id: &str, script_name: &str) -> Result<TailSessionResult> {
        let url = format!("{}/accounts/{account_id}/workers/scripts/{script_name}/tails", self.base_url);
        let resp = self.client.post(&url).send().await?.json::<ApiResponse<TailSessionResult>>().await?;
        if resp.success {
            if let Some(res) = resp.result {
                return Ok(res);
            }
        }
        let msg = resp.errors.first().map(|e| e.message.clone()).unwrap_or_else(|| "Failed to create tail session".to_string());
        color_eyre::eyre::bail!("{msg}")
    }
}
