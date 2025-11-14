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
    #[serde(default)]
    pub zone_id: Option<String>,
    #[serde(rename = "type")]
    pub record_type: String,
    pub name: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub proxied: bool,
    #[serde(default)]
    pub ttl: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct PatchDnsRecord {
    pub proxied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewDnsRecord {
    #[serde(rename = "type")]
    pub record_type: String,
    pub name: String,
    pub content: String,
    pub ttl: u32,
    pub proxied: bool,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct ZoneSettings {
    pub ssl: Option<SettingValue<String>>,
    pub development_mode: Option<SettingValue<String>>,
    pub security_level: Option<SettingValue<String>>,
    pub min_tls_version: Option<SettingValue<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SettingValue<T> {
    pub value: T,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SettingUpdate<T> {
    pub value: T,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PurgeCacheRequest {
    pub purge_everything: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkerScriptResult {
    pub id: String,
    pub tag: Option<String>,
    pub routes: Option<Vec<WorkerRoute>>,
    pub modified_on: Option<String>,
    pub created_on: Option<String>,
    pub compatibility_date: Option<String>,
    pub compatibility_flags: Option<Vec<String>>,
    pub usage_model: Option<String>,
    pub handlers: Option<Vec<String>>,
    pub last_deployed_from: Option<String>,
    pub logpush: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct WorkerBinding {
    pub name: String,
    #[serde(rename = "type")]
    pub binding_type: String,
    pub database_id: Option<String>,
    pub id: Option<String>,
    pub namespace_id: Option<String>,
    pub bucket_name: Option<String>,
    pub queue_name: Option<String>,
    pub text: Option<String>,
    pub service: Option<String>,
    pub environment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct WorkerRouteDetail {
    pub id: Option<String>,
    pub pattern: String,
    pub zone_id: Option<String>,
    pub zone_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PagesDeployment {
    pub latest_stage: Option<PagesStage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PagesStage {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PagesDeploymentResult {
    pub id: String,
    pub short_id: Option<String>,
    pub project_id: Option<String>,
    pub project_name: Option<String>,
    pub environment: Option<String>,
    pub url: Option<String>,
    pub created_on: Option<String>,
    pub modified_on: Option<String>,
    pub latest_stage: Option<PagesStage>,
    pub deployment_trigger: Option<PagesDeploymentTrigger>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PagesDeploymentTrigger {
    #[serde(rename = "type")]
    pub trigger_type: Option<String>,
    pub metadata: Option<PagesTriggerMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PagesTriggerMetadata {
    pub branch: Option<String>,
    pub commit_hash: Option<String>,
    pub commit_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PagesBuildLogResult {
    pub total: Option<usize>,
    pub data: Option<Vec<PagesBuildLogEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PagesBuildLogEntry {
    pub ts: Option<String>,
    pub line: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PagesDomainResult {
    pub id: String,
    pub name: String,
    pub status: Option<String>,
    pub ssl: Option<PagesDomainSsl>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PagesDomainSsl {
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct D1DatabaseResult {
    pub uuid: String,
    pub name: String,
    pub version: Option<String>,
    pub created_at: Option<String>,
    pub num_tables: Option<usize>,
    pub file_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct D1QueryRequest {
    pub sql: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct D1QueryResult {
    pub results: Option<Vec<serde_json::Value>>,
    pub success: Option<bool>,
    pub meta: Option<D1QueryMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct D1QueryMeta {
    pub duration: Option<f64>,
    pub rows_read: Option<u64>,
    pub rows_written: Option<u64>,
    pub changes: Option<u64>,
    pub last_row_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct KvNamespaceResult {
    pub id: String,
    pub title: String,
    pub supports_url_encoding: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KvKeyResult {
    pub name: String,
    pub expiration: Option<u64>,
    pub metadata: Option<serde_json::Value>,
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
        if !resp.success {
            let msg = resp.errors.first().map(|e| e.message.clone()).unwrap_or_else(|| "Failed to fetch zones".to_string());
            color_eyre::eyre::bail!("{msg}");
        }
        Ok(resp.result.unwrap_or_default())
    }

    pub async fn get_dns_records(&self, zone_id: &str) -> Result<Vec<DnsRecordResult>> {
        let url = format!("{}/zones/{zone_id}/dns_records?per_page=100", self.base_url);
        let resp = self.client.get(&url).send().await?.json::<ApiResponse<Vec<DnsRecordResult>>>().await?;
        if !resp.success {
            let msg = resp.errors.first().map(|e| e.message.clone()).unwrap_or_else(|| "Failed to fetch DNS records".to_string());
            color_eyre::eyre::bail!("{msg}");
        }
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
        if !resp.success {
            let msg = resp.errors.first().map(|e| e.message.clone()).unwrap_or_else(|| "Failed to fetch workers".to_string());
            color_eyre::eyre::bail!("{msg}");
        }
        Ok(resp.result.unwrap_or_default())
    }

    pub async fn get_pages(&self, account_id: &str) -> Result<Vec<PagesProjectResult>> {
        let url = format!("{}/accounts/{account_id}/pages/projects", self.base_url);
        let resp = self.client.get(&url).send().await?.json::<ApiResponse<Vec<PagesProjectResult>>>().await?;
        if !resp.success {
            let msg = resp.errors.first().map(|e| e.message.clone()).unwrap_or_else(|| "Failed to fetch pages".to_string());
            color_eyre::eyre::bail!("{msg}");
        }
        Ok(resp.result.unwrap_or_default())
    }

    pub async fn get_pages_deployments(&self, account_id: &str, project_name: &str) -> Result<Vec<PagesDeploymentResult>> {
        let url = format!("{}/accounts/{account_id}/pages/projects/{project_name}/deployments?per_page=25", self.base_url);
        let resp = self.client.get(&url).send().await?.json::<ApiResponse<Vec<PagesDeploymentResult>>>().await?;
        Ok(resp.result.unwrap_or_default())
    }

    pub async fn rollback_pages_deployment(&self, account_id: &str, project_name: &str, deployment_id: &str) -> Result<PagesDeploymentResult> {
        let url = format!("{}/accounts/{account_id}/pages/projects/{project_name}/deployments/{deployment_id}/rollback", self.base_url);
        let resp = self.client.post(&url).send().await?.json::<ApiResponse<PagesDeploymentResult>>().await?;
        if resp.success {
            if let Some(res) = resp.result {
                return Ok(res);
            }
        }
        let msg = resp.errors.first().map(|e| e.message.clone()).unwrap_or_else(|| "Failed to rollback deployment".to_string());
        color_eyre::eyre::bail!("{msg}")
    }

    pub async fn get_pages_build_logs(&self, account_id: &str, project_name: &str, deployment_id: &str) -> Result<Vec<PagesBuildLogEntry>> {
        let url = format!("{}/accounts/{account_id}/pages/projects/{project_name}/deployments/{deployment_id}/history/logs", self.base_url);
        let resp = self.client.get(&url).send().await?.json::<ApiResponse<PagesBuildLogResult>>().await?;
        Ok(resp.result.and_then(|r| r.data).unwrap_or_default())
    }

    pub async fn get_pages_domains(&self, account_id: &str, project_name: &str) -> Result<Vec<PagesDomainResult>> {
        let url = format!("{}/accounts/{account_id}/pages/projects/{project_name}/domains", self.base_url);
        let resp = self.client.get(&url).send().await?.json::<ApiResponse<Vec<PagesDomainResult>>>().await?;
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

    pub async fn delete_worker_tail(&self, account_id: &str, script_name: &str, tail_id: &str) -> Result<()> {
        let url = format!("{}/accounts/{account_id}/workers/scripts/{script_name}/tails/{tail_id}", self.base_url);
        let resp = self.client.delete(&url).send().await?.json::<ApiResponse<Option<serde_json::Value>>>().await?;
        if !resp.success {
            let msg = resp.errors.first().map(|e| e.message.clone()).unwrap_or_else(|| "Failed to delete tail session".to_string());
            color_eyre::eyre::bail!("{msg}");
        }
        Ok(())
    }

    pub async fn get_worker_bindings(&self, account_id: &str, script_name: &str) -> Result<Vec<WorkerBinding>> {
        let url = format!("{}/accounts/{account_id}/workers/scripts/{script_name}/bindings", self.base_url);
        let resp = self.client.get(&url).send().await?.json::<ApiResponse<Vec<WorkerBinding>>>().await?;
        if !resp.success {
            let msg = resp.errors.first().map(|e| e.message.clone()).unwrap_or_else(|| "Failed to fetch worker bindings".to_string());
            color_eyre::eyre::bail!("{msg}");
        }
        Ok(resp.result.unwrap_or_default())
    }

    pub async fn get_worker_routes(&self, account_id: &str, script_name: &str) -> Result<Vec<WorkerRouteDetail>> {
        let url = format!("{}/accounts/{account_id}/workers/scripts/{script_name}/routes", self.base_url);
        let resp = self.client.get(&url).send().await?.json::<ApiResponse<Vec<WorkerRouteDetail>>>().await?;
        if !resp.success {
            let msg = resp.errors.first().map(|e| e.message.clone()).unwrap_or_else(|| "Failed to fetch worker routes".to_string());
            color_eyre::eyre::bail!("{msg}");
        }
        Ok(resp.result.unwrap_or_default())
    }

    pub async fn get_zone_settings(&self, zone_id: &str) -> Result<ZoneSettings> {
        let url = format!("{}/zones/{zone_id}/settings", self.base_url);
        let resp = self.client.get(&url).send().await?.json::<ApiResponse<Vec<serde_json::Value>>>().await?;
        let mut settings = ZoneSettings::default();

        if let Some(items) = resp.result {
            for item in items {
                if let (Some(id), Some(val)) = (item.get("id").and_then(|v| v.as_str()), item.get("value")) {
                    match id {
                        "ssl" => {
                            if let Some(s) = val.as_str() {
                                settings.ssl = Some(SettingValue { value: s.to_string() });
                            }
                        }
                        "development_mode" => {
                            if let Some(s) = val.as_str() {
                                settings.development_mode = Some(SettingValue { value: s.to_string() });
                            }
                        }
                        "security_level" => {
                            if let Some(s) = val.as_str() {
                                settings.security_level = Some(SettingValue { value: s.to_string() });
                            }
                        }
                        "min_tls_version" => {
                            if let Some(s) = val.as_str() {
                                settings.min_tls_version = Some(SettingValue { value: s.to_string() });
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(settings)
    }

    pub async fn set_zone_ssl(&self, zone_id: &str, ssl_mode: &str) -> Result<()> {
        let url = format!("{}/zones/{zone_id}/settings/ssl", self.base_url);
        let body = SettingUpdate { value: ssl_mode };
        let resp = self.client.patch(&url).json(&body).send().await?.json::<ApiResponse<serde_json::Value>>().await?;
        if resp.success {
            Ok(())
        } else {
            let msg = resp.errors.first().map(|e| e.message.clone()).unwrap_or_else(|| "Failed to update SSL".to_string());
            color_eyre::eyre::bail!("{msg}")
        }
    }

    pub async fn set_zone_dev_mode(&self, zone_id: &str, enabled: bool) -> Result<()> {
        let url = format!("{}/zones/{zone_id}/settings/development_mode", self.base_url);
        let body = SettingUpdate { value: if enabled { "on" } else { "off" } };
        let resp = self.client.patch(&url).json(&body).send().await?.json::<ApiResponse<serde_json::Value>>().await?;
        if resp.success {
            Ok(())
        } else {
            let msg = resp.errors.first().map(|e| e.message.clone()).unwrap_or_else(|| "Failed to update Dev Mode".to_string());
            color_eyre::eyre::bail!("{msg}")
        }
    }

    pub async fn set_zone_security_level(&self, zone_id: &str, level: &str) -> Result<()> {
        let url = format!("{}/zones/{zone_id}/settings/security_level", self.base_url);
        let body = SettingUpdate { value: level };
        let resp = self.client.patch(&url).json(&body).send().await?.json::<ApiResponse<serde_json::Value>>().await?;
        if resp.success {
            Ok(())
        } else {
            let msg = resp.errors.first().map(|e| e.message.clone()).unwrap_or_else(|| "Failed to update Security Level".to_string());
            color_eyre::eyre::bail!("{msg}")
        }
    }

    pub async fn purge_zone_cache(&self, zone_id: &str) -> Result<()> {
        let url = format!("{}/zones/{zone_id}/purge_cache", self.base_url);
        let body = PurgeCacheRequest { purge_everything: true };
        let resp = self.client.post(&url).json(&body).send().await?.json::<ApiResponse<serde_json::Value>>().await?;
        if resp.success {
            Ok(())
        } else {
            let msg = resp.errors.first().map(|e| e.message.clone()).unwrap_or_else(|| "Failed to purge cache".to_string());
            color_eyre::eyre::bail!("{msg}")
        }
    }

    pub async fn create_dns_record(&self, zone_id: &str, record: &NewDnsRecord) -> Result<DnsRecordResult> {
        let url = format!("{}/zones/{zone_id}/dns_records", self.base_url);
        let resp = self.client.post(&url).json(record).send().await?.json::<ApiResponse<DnsRecordResult>>().await?;
        if resp.success {
            if let Some(res) = resp.result {
                return Ok(res);
            }
        }
        let msg = resp.errors.first().map(|e| e.message.clone()).unwrap_or_else(|| "Failed to create DNS record".to_string());
        color_eyre::eyre::bail!("{msg}")
    }

    pub async fn update_dns_record(&self, zone_id: &str, record_id: &str, record: &NewDnsRecord) -> Result<DnsRecordResult> {
        let url = format!("{}/zones/{zone_id}/dns_records/{record_id}", self.base_url);
        let resp = self.client.put(&url).json(record).send().await?.json::<ApiResponse<DnsRecordResult>>().await?;
        if resp.success {
            if let Some(res) = resp.result {
                return Ok(res);
            }
        }
        let msg = resp.errors.first().map(|e| e.message.clone()).unwrap_or_else(|| "Failed to update DNS record".to_string());
        color_eyre::eyre::bail!("{msg}")
    }

    pub async fn delete_dns_record(&self, zone_id: &str, record_id: &str) -> Result<()> {
        let url = format!("{}/zones/{zone_id}/dns_records/{record_id}", self.base_url);
        let resp = self.client.delete(&url).send().await?.json::<ApiResponse<serde_json::Value>>().await?;
        if resp.success {
            Ok(())
        } else {
            let msg = resp.errors.first().map(|e| e.message.clone()).unwrap_or_else(|| "Failed to delete DNS record".to_string());
            color_eyre::eyre::bail!("{msg}")
        }
    }

    pub async fn get_d1_databases(&self, account_id: &str) -> Result<Vec<D1DatabaseResult>> {
        let url = format!("{}/accounts/{account_id}/d1/database", self.base_url);
        let resp = self.client.get(&url).send().await?.json::<ApiResponse<Vec<D1DatabaseResult>>>().await?;
        if !resp.success {
            let msg = resp.errors.first().map(|e| e.message.clone()).unwrap_or_else(|| "Failed to fetch D1 databases".to_string());
            color_eyre::eyre::bail!("{msg}");
        }
        Ok(resp.result.unwrap_or_default())
    }

    pub async fn execute_d1_query(&self, account_id: &str, database_id: &str, sql: &str) -> Result<Vec<D1QueryResult>> {
        let url = format!("{}/accounts/{account_id}/d1/database/{database_id}/query", self.base_url);
        let body = D1QueryRequest {
            sql: sql.to_string(),
            params: None,
        };
        let resp = self.client.post(&url).json(&body).send().await?.json::<ApiResponse<Vec<D1QueryResult>>>().await?;
        if !resp.success {
            let msg = resp.errors.first().map(|e| e.message.clone()).unwrap_or_else(|| "Failed to execute D1 query".to_string());
            color_eyre::eyre::bail!("{msg}");
        }
        Ok(resp.result.unwrap_or_default())
    }

    pub async fn get_kv_namespaces(&self, account_id: &str) -> Result<Vec<KvNamespaceResult>> {
        let url = format!("{}/accounts/{account_id}/storage/kv/namespaces?per_page=100", self.base_url);
        let resp = self.client.get(&url).send().await?.json::<ApiResponse<Vec<KvNamespaceResult>>>().await?;
        if !resp.success {
            let msg = resp.errors.first().map(|e| e.message.clone()).unwrap_or_else(|| "Failed to fetch KV namespaces".to_string());
            color_eyre::eyre::bail!("{msg}");
        }
        Ok(resp.result.unwrap_or_default())
    }

    pub async fn get_kv_keys(&self, account_id: &str, namespace_id: &str, limit: Option<usize>, prefix: Option<&str>) -> Result<Vec<KvKeyResult>> {
        let mut url = format!("{}/accounts/{account_id}/storage/kv/namespaces/{namespace_id}/keys?limit={}", self.base_url, limit.unwrap_or(100));
        if let Some(p) = prefix {
            if !p.is_empty() {
                url.push_str(&format!("&prefix={p}"));
            }
        }
        let resp = self.client.get(&url).send().await?.json::<ApiResponse<Vec<KvKeyResult>>>().await?;
        if !resp.success {
            let msg = resp.errors.first().map(|e| e.message.clone()).unwrap_or_else(|| "Failed to fetch KV keys".to_string());
            color_eyre::eyre::bail!("{msg}");
        }
        Ok(resp.result.unwrap_or_default())
    }

    pub async fn get_kv_value(&self, account_id: &str, namespace_id: &str, key_name: &str) -> Result<String> {
        let url = format!("{}/accounts/{account_id}/storage/kv/namespaces/{namespace_id}/values/{key_name}", self.base_url);
        let resp = self.client.get(&url).send().await?;
        if resp.status().is_success() {
            let text = resp.text().await?;
            Ok(text)
        } else {
            color_eyre::eyre::bail!("Failed to fetch KV value: HTTP {}", resp.status());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_script_deserialization() {
        let json = r#"{
            "id": "my-worker",
            "tag": "abc123tag",
            "modified_on": "2025-10-29T10:00:00Z",
            "compatibility_date": "2024-09-01",
            "compatibility_flags": ["nodejs_compat"],
            "usage_model": "standard",
            "handlers": ["fetch", "scheduled"],
            "last_deployed_from": "wrangler",
            "logpush": false
        }"#;

        let script: WorkerScriptResult = serde_json::from_str(json).expect("deserialize worker");
        assert_eq!(script.id, "my-worker");
        assert_eq!(script.compatibility_date.as_deref(), Some("2024-09-01"));
        assert_eq!(script.compatibility_flags, Some(vec!["nodejs_compat".to_string()]));
        assert_eq!(script.usage_model.as_deref(), Some("standard"));
        assert_eq!(script.handlers, Some(vec!["fetch".to_string(), "scheduled".to_string()]));
    }

    #[test]
    fn test_worker_bindings_deserialization() {
        let json = r#"[
            {"name": "BOT_TOKEN", "type": "secret_text"},
            {"name": "DB", "type": "d1", "database_id": "76632ad7-a168-49f6-b81a-90306079a5fe"},
            {"name": "CACHE_KV", "type": "kv_namespace", "namespace_id": "kv123"}
        ]"#;

        let bindings: Vec<WorkerBinding> = serde_json::from_str(json).expect("deserialize bindings");
        assert_eq!(bindings.len(), 3);
        assert_eq!(bindings[0].name, "BOT_TOKEN");
        assert_eq!(bindings[0].binding_type, "secret_text");
        assert_eq!(bindings[1].binding_type, "d1");
        assert_eq!(bindings[1].database_id.as_deref(), Some("76632ad7-a168-49f6-b81a-90306079a5fe"));
        assert_eq!(bindings[2].binding_type, "kv_namespace");
        assert_eq!(bindings[2].namespace_id.as_deref(), Some("kv123"));
    }

    #[test]
    fn test_dns_record_deserialization() {
        let json = r#"{
            "id": "f61bab825bf5e32aef5307b4cf0f994a",
            "name": "iamkidus.et",
            "type": "CAA",
            "content": "0 issue \"letsencrypt.org\"",
            "proxiable": false,
            "proxied": false,
            "ttl": 1
        }"#;

        let record: DnsRecordResult = serde_json::from_str(json).expect("deserialize dns record");
        assert_eq!(record.id, "f61bab825bf5e32aef5307b4cf0f994a");
        assert_eq!(record.name, "iamkidus.et");
        assert_eq!(record.record_type, "CAA");
        assert_eq!(record.content, "0 issue \"letsencrypt.org\"");
        assert_eq!(record.zone_id, None);
        assert!(!record.proxied);
        assert_eq!(record.ttl, 1);
    }

    #[test]
    fn test_d1_database_deserialization() {
        let json = r#"{
            "uuid": "76632ad7-a168-49f6-b81a-90306079a5fe",
            "name": "prod-users-db",
            "version": "production",
            "created_at": "2025-10-01T08:00:00Z",
            "num_tables": 6,
            "file_size": 1048576
        }"#;

        let db: D1DatabaseResult = serde_json::from_str(json).expect("deserialize d1 db");
        assert_eq!(db.uuid, "76632ad7-a168-49f6-b81a-90306079a5fe");
        assert_eq!(db.name, "prod-users-db");
        assert_eq!(db.num_tables, Some(6));
        assert_eq!(db.file_size, Some(1048576));
    }

    #[test]
    fn test_d1_query_result_deserialization() {
        let json = r#"{
            "results": [
                {"id": 1, "username": "alice", "active": 1},
                {"id": 2, "username": "bob", "active": 0}
            ],
            "success": true,
            "meta": {
                "duration": 0.0154,
                "rows_read": 2,
                "rows_written": 0,
                "changes": 0,
                "last_row_id": 0
            }
        }"#;

        let query_res: D1QueryResult = serde_json::from_str(json).expect("deserialize d1 query");
        assert_eq!(query_res.success, Some(true));
        let results = query_res.results.expect("results");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0]["username"], "alice");
        let meta = query_res.meta.expect("meta");
        assert_eq!(meta.rows_read, Some(2));
        assert!(meta.duration.unwrap() > 0.0);
    }

    #[test]
    fn test_kv_namespace_and_keys_deserialization() {
        let ns_json = r#"{
            "id": "0f2b23456789abcdef0123456789abcd",
            "title": "SESSION_CACHE",
            "supports_url_encoding": true
        }"#;

        let ns: KvNamespaceResult = serde_json::from_str(ns_json).expect("deserialize kv ns");
        assert_eq!(ns.id, "0f2b23456789abcdef0123456789abcd");
        assert_eq!(ns.title, "SESSION_CACHE");
        assert_eq!(ns.supports_url_encoding, Some(true));

        let key_json = r#"{
            "name": "sess_user_9918",
            "expiration": 1799999999,
            "metadata": {"role": "admin"}
        }"#;

        let key: KvKeyResult = serde_json::from_str(key_json).expect("deserialize kv key");
        assert_eq!(key.name, "sess_user_9918");
        assert_eq!(key.expiration, Some(1799999999));
        assert_eq!(key.metadata.unwrap()["role"], "admin");
    }
}
