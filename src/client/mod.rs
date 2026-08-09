pub mod models;

use anyhow::Result;
pub use models::*;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

pub const BASE_URL: &str = "https://app.keeperhub.com/api";

#[derive(Debug, Clone)]
pub enum ApiError {
    Transport(String),
    Api {
        status: u16,
        code: String,
        detail: String,
        hint: Option<String>,
        request_id: Option<String>,
    },
    Parse {
        status: u16,
        body: String,
    },
    Http {
        status: u16,
        body: String,
    },
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Transport(e) => write!(f, "network error: {e}"),
            ApiError::Api {
                status,
                code,
                detail,
                hint,
                request_id,
            } => {
                write!(f, "[{status}] {code}: {detail}")?;
                if let Some(h) = hint {
                    write!(f, " (hint: {h})")?;
                }
                if let Some(r) = request_id {
                    write!(f, " (request_id: {r})")?;
                }
                Ok(())
            }
            ApiError::Parse { status, body } => {
                write!(
                    f,
                    "[{status}] could not parse response: {}",
                    truncate(body, 200)
                )
            }
            ApiError::Http { status, body } => write!(f, "[{status}] {}", truncate(body, 200)),
        }
    }
}

impl std::error::Error for ApiError {}

/// True when the failure means this API key lacks permission for the endpoint
/// (401 or a scope-flavoured code). Callers may degrade to fallback sources.
pub fn is_scope_error(e: &ApiError) -> bool {
    match e {
        ApiError::Api { status, code, .. } => {
            *status == 401
                || code.to_lowercase().contains("scope")
                || code.to_lowercase().contains("auth")
        }
        ApiError::Http { status: 401, .. } => true,
        _ => false,
    }
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(n).collect::<String>())
    }
}

#[derive(Clone)]
pub struct KhClient {
    inner: reqwest::Client,
    api_key: Arc<String>,
    base: Arc<String>,
    rate_remaining: Arc<AtomicU32>,
}

impl KhClient {
    pub fn new(api_key: Arc<String>) -> Self {
        Self::with_base(api_key, BASE_URL.to_string())
    }

    pub fn with_base(api_key: Arc<String>, base: String) -> Self {
        let inner = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(5))
            .timeout(std::time::Duration::from_secs(15))
            .user_agent(concat!("khtop/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("reqwest client build");
        Self {
            inner,
            api_key,
            base: Arc::new(base),
            rate_remaining: Arc::new(AtomicU32::new(0)),
        }
    }

    pub fn rate_remaining(&self) -> u32 {
        self.rate_remaining.load(Ordering::Relaxed)
    }

    fn capture_rate_headers(&self, resp: &reqwest::Response) {
        if let Some(v) = resp.headers().get("x-ratelimit-remaining") {
            if let Ok(s) = v.to_str() {
                if let Ok(n) = s.parse::<u32>() {
                    self.rate_remaining.store(n, Ordering::Relaxed);
                }
            }
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base, path)
    }

    async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        let resp = self
            .inner
            .get(self.url(path))
            .bearer_auth(self.api_key.as_str())
            .send()
            .await
            .map_err(|e| ApiError::Transport(e.to_string()))?;
        self.capture_rate_headers(&resp);
        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| ApiError::Transport(e.to_string()))?;
        if status.is_success() {
            serde_json::from_str(&text).map_err(|_e| ApiError::Parse {
                status: status.as_u16(),
                body: text,
            })
        } else {
            Err(parse_error(status, &text))
        }
    }

    async fn post_raw(
        &self,
        path: &str,
        body: &Value,
        idempotency_key: Option<&str>,
    ) -> Result<(u16, Value), ApiError> {
        let mut builder = self
            .inner
            .post(self.url(path))
            .bearer_auth(self.api_key.as_str())
            .json(body);
        if let Some(k) = idempotency_key {
            builder = builder.header("Idempotency-Key", k);
        }
        let resp = builder
            .send()
            .await
            .map_err(|e| ApiError::Transport(e.to_string()))?;
        self.capture_rate_headers(&resp);
        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| ApiError::Transport(e.to_string()))?;
        let value: Value = serde_json::from_str(&text).map_err(|_| ApiError::Parse {
            status: status.as_u16(),
            body: text.clone(),
        })?;
        Ok((status.as_u16(), value))
    }

    pub async fn list_workflows(&self) -> Result<Vec<Workflow>, ApiError> {
        self.get_json("/workflows").await
    }

    pub async fn execute_workflow(
        &self,
        workflow_id: &str,
    ) -> Result<WorkflowExecuteResponse, ApiError> {
        let (_, value) = self
            .post_raw(
                &format!("/workflows/{workflow_id}/execute"),
                &serde_json::json!({ "input": {} }),
                None,
            )
            .await?;
        serde_json::from_value(value).map_err(|e| ApiError::Parse {
            status: 0,
            body: e.to_string(),
        })
    }

    pub async fn workflow_executions(
        &self,
        workflow_id: &str,
    ) -> Result<Vec<WorkflowExecution>, ApiError> {
        self.get_json(&format!("/workflows/{workflow_id}/executions"))
            .await
    }

    pub async fn execution_logs(&self, execution_id: &str) -> Result<LogsResponse, ApiError> {
        self.get_json(&format!("/workflows/executions/{execution_id}/logs"))
            .await
    }

    pub async fn analytics_runs(&self, limit: u64) -> Result<RunsPage, ApiError> {
        self.get_json(&format!("/analytics/runs?limit={limit}"))
            .await
    }

    /// Fallback when the analytics endpoint is outside the key's scope:
    /// aggregate per-workflow execution history instead.
    pub async fn fallback_runs(&self) -> Vec<Run> {
        let Ok(workflows) = self.list_workflows().await else {
            return Vec::new();
        };
        let mut runs: Vec<Run> = Vec::new();
        for wf in workflows.iter().take(10) {
            let Ok(execs) = self.workflow_executions(&wf.id).await else {
                continue;
            };
            for e in execs {
                runs.push(Run {
                    id: e.id,
                    source: "workflow".into(),
                    workflow_id: Some(wf.id.clone()),
                    workflow_name: Some(wf.name.clone()),
                    status: e.status,
                    created_at: e.started_at,
                    completed_at: e.completed_at,
                    duration_ms: None,
                    r#type: None,
                    network: e.transaction_hashes.first().and_then(|t| t.network.clone()),
                    transaction_hash: e.transaction_hashes.first().map(|t| t.hash.clone()),
                    gas_used_wei: None,
                });
            }
        }
        runs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        runs
    }

    pub async fn analytics_summary(&self) -> Result<AnalyticsSummary, ApiError> {
        self.get_json("/analytics/summary").await
    }

    pub async fn spend_cap(&self) -> Result<SpendCap, ApiError> {
        self.get_json("/analytics/spend-cap").await
    }

    pub async fn chains(&self) -> Result<Vec<Chain>, ApiError> {
        self.get_json("/chains").await
    }

    pub async fn direct_status(&self, execution_id: &str) -> Result<DirectStatus, ApiError> {
        self.get_json(&format!("/execute/{execution_id}/status"))
            .await
    }

    pub async fn step_logs(&self, execution_id: &str) -> Result<StepsResponse, ApiError> {
        self.get_json(&format!("/analytics/runs/{execution_id}/steps"))
            .await
    }

    fn transfer_body(&self, req: &TransferRequest, simulate: bool) -> Value {
        serde_json::json!({
            "chainId": req.chain_id,
            "recipientAddress": req.recipient_address,
            "amount": req.amount,
            "simulate": simulate,
        })
    }

    pub async fn simulate_transfer(
        &self,
        req: &TransferRequest,
    ) -> Result<SimulateOutcome, ApiError> {
        let (status, value) = self
            .post_raw("/execute/transfer", &self.transfer_body(req, true), None)
            .await?;
        SimulateOutcome::parse(status, &value)
    }

    pub async fn broadcast_transfer(
        &self,
        req: &TransferRequest,
        idempotency_key: &str,
    ) -> Result<TransferResponse, ApiError> {
        let (_, value) = self
            .post_raw(
                "/execute/transfer",
                &self.transfer_body(req, false),
                Some(idempotency_key),
            )
            .await?;
        serde_json::from_value(value).map_err(|e| ApiError::Parse {
            status: 0,
            body: e.to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct SimulateOutcome {
    pub success: bool,
    pub would_revert: bool,
    pub gas_estimate: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub value: Option<String>,
    pub revert_reason: Option<String>,
}

impl SimulateOutcome {
    pub fn parse(status: u16, v: &Value) -> Result<Self, ApiError> {
        let success = v
            .get("success")
            .and_then(|x| x.as_bool())
            .unwrap_or(status == 202 || status == 200);
        let would_revert = v
            .get("wouldRevert")
            .and_then(|x| x.as_bool())
            .unwrap_or(false);
        Ok(SimulateOutcome {
            success,
            would_revert,
            gas_estimate: v
                .get("gasEstimate")
                .and_then(|x| x.as_str())
                .map(String::from),
            from: v.get("from").and_then(|x| x.as_str()).map(String::from),
            to: v.get("to").and_then(|x| x.as_str()).map(String::from),
            value: v.get("value").and_then(|x| x.as_str()).map(String::from),
            revert_reason: v
                .get("revertReason")
                .and_then(|x| x.as_str())
                .map(String::from),
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferResponse {
    pub execution_id: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
struct ErrorEnvelope {
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    detail: Option<String>,
    #[serde(default)]
    hint: Option<String>,
    #[serde(default)]
    request_id: Option<String>,
}

fn parse_error(status: StatusCode, text: &str) -> ApiError {
    match serde_json::from_str::<ErrorEnvelope>(text) {
        Ok(e) if e.error.is_some() => ApiError::Api {
            status: status.as_u16(),
            code: e.error.unwrap_or_default(),
            detail: e.detail.unwrap_or_default(),
            hint: e.hint,
            request_id: e.request_id,
        },
        _ => ApiError::Http {
            status: status.as_u16(),
            body: text.to_string(),
        },
    }
}
