use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub visibility: Option<String>,
    #[serde(default)]
    pub nodes: Value,
    #[serde(default)]
    pub edges: Value,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LogEntry {
    pub id: String,
    #[serde(default)]
    pub execution_id: Option<String>,
    #[serde(default)]
    pub node_id: Option<String>,
    #[serde(default)]
    pub node_name: Option<String>,
    #[serde(default)]
    pub node_type: Option<String>,
    pub status: String,
    #[serde(default)]
    pub input: Value,
    #[serde(default)]
    pub output: Value,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub duration: Option<String>,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub completed_at: Option<String>,
    #[serde(default)]
    pub iteration_index: Option<u64>,
    #[serde(default)]
    pub for_each_node_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LogsResponse {
    #[serde(default)]
    pub execution: Option<Value>,
    #[serde(default)]
    pub logs: Vec<LogEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StepEntry {
    #[serde(default)]
    pub node_id: Option<String>,
    #[serde(default)]
    pub node_name: Option<String>,
    pub status: String,
    #[serde(default)]
    pub input: Value,
    #[serde(default)]
    pub output: Value,
    #[serde(default)]
    pub duration_ms: Option<u64>,
    #[serde(default)]
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StepsResponse {
    #[serde(default)]
    pub steps: Vec<StepEntry>,
}

#[derive(Debug, Clone, Default)]
pub struct TransferRequest {
    pub chain_id: String,
    pub recipient_address: String,
    pub amount: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DirectStatus {
    pub execution_id: String,
    pub status: String,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub transaction_hash: Option<String>,
    #[serde(default)]
    pub transaction_link: Option<String>,
    #[serde(default)]
    pub gas_used_wei: Option<String>,
    #[serde(default)]
    pub result: Option<Value>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnalyticsSummary {
    pub total_runs: u64,
    #[serde(default)]
    pub successful_runs: u64,
    #[serde(default)]
    pub failed_runs: u64,
    #[serde(default)]
    pub success_rate: f64,
    #[serde(default)]
    pub total_gas_used_wei: String,
    #[serde(default)]
    pub avg_execution_time_ms: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Run {
    pub id: String,
    pub source: String,
    #[serde(default)]
    pub workflow_id: Option<String>,
    #[serde(default)]
    pub workflow_name: Option<String>,
    pub status: String,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub completed_at: Option<String>,
    #[serde(default)]
    pub duration_ms: Option<u64>,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub network: Option<String>,
    #[serde(default)]
    pub transaction_hash: Option<String>,
    #[serde(default)]
    pub gas_used_wei: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RunsPage {
    #[serde(default)]
    pub runs: Vec<Run>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpendCap {
    #[serde(default)]
    pub daily_cap_wei: String,
    #[serde(default)]
    pub spent_today_wei: String,
    #[serde(default)]
    pub remaining_wei: String,
    #[serde(default)]
    pub percent_used: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Chain {
    pub id: String,
    pub chain_id: u64,
    pub name: String,
    pub symbol: String,
    #[serde(default)]
    pub chain_type: String,
    #[serde(default)]
    pub explorer_url: Option<String>,
    #[serde(default)]
    pub explorer_address_path: Option<String>,
    #[serde(default)]
    pub is_testnet: bool,
    #[serde(default)]
    pub is_enabled: bool,
    #[serde(default)]
    pub use_private_mempool_rpc: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkflowExecuteResponse {
    pub execution_id: String,
    pub status: String,
}
