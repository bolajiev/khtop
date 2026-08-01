use khtop::client::{KhClient, TransferRequest};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

fn serve(
    routes: Arc<Mutex<Vec<(String, String)>>>,
    expected_keys: Arc<Mutex<Vec<String>>>,
) -> (String, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut buf = Vec::new();
            let mut chunk = [0u8; 4096];
            loop {
                match stream.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(n) => {
                        buf.extend_from_slice(&chunk[..n]);
                        if buf.windows(4).any(|w| w == b"\r\n\r\n") {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            let req = String::from_utf8_lossy(&buf);
            let req_line = req.lines().next().unwrap_or_default().to_string();
            let parts: Vec<&str> = req_line.split_whitespace().collect();
            if parts.len() >= 2 {
                let path = parts[1];
                let method = parts[0];
                let auth = req.lines().any(|l| {
                    l.to_lowercase()
                        .starts_with("authorization: bearer kh_test")
                });
                expected_keys
                    .lock()
                    .unwrap()
                    .push(format!("{method} {path} auth={auth}"));
                let route_key = format!("{method} {path}");
                let body = routes
                    .lock()
                    .unwrap()
                    .iter()
                    .find(|(r, _)| r == &route_key)
                    .map(|(_, b)| b.clone());
                let (status, resp) = match body {
                    Some(b) => ("200 OK".to_string(), b),
                    None => ("404 Not Found".to_string(), serde_json::json!({"error": "not_found", "detail": "no mock route", "request_id": "mock-1"}).to_string()),
                };
                let mut headers = format!("HTTP/1.1 {status}\r\nContent-Type: application/json\r\nX-RateLimit-Remaining: 97\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", resp.len());
                if !resp.is_empty() {
                    headers.push_str(&resp);
                }
                let _ = stream.write_all(headers.as_bytes());
                let _ = stream.flush();
            }
        }
    });
    (format!("http://{addr}"), handle)
}

fn routes() -> Vec<(String, String)> {
    let wf = serde_json::json!([{
        "id": "wf_123", "name": "Auto-compounder", "description": "x",
        "visibility": "private", "nodes": [{
            "id": "trigger-1", "type": "trigger",
            "data": {"label": "Schedule", "config": {"triggerType": "Schedule", "scheduleCron": "*/30 * * * *"}}
        }], "edges": [],
        "createdAt": "2026-07-01T00:00:00Z", "updatedAt": "2026-07-28T10:00:00Z"
    }]);
    let runs = serde_json::json!({"runs": [
        {"id": "direct_789", "source": "direct", "type": "transfer", "network": "ethereum",
         "status": "completed", "createdAt": "2026-07-30T12:00:00Z", "completedAt": "2026-07-30T12:00:15Z",
         "transactionHash": "0xca19a1f0c1d48e6d8a4c1f9d1f3d7a1c9f3b4d5e6f7a8b9c0d1e2f3a4b5c6d7e8",
         "gasUsedWei": "21000000000000"},
        {"id": "exec_123", "source": "workflow", "workflowId": "wf_123", "workflowName": "Auto-compounder",
         "status": "running", "createdAt": "2026-07-30T12:05:00Z", "durationMs": 4200}
    ], "nextCursor": null});
    let spend = serde_json::json!({"dailyCapWei": "100000000000000000", "spentTodayWei": "25000000000000000",
                                   "remainingWei": "75000000000000000", "percentUsed": 25.0});
    let summary = serde_json::json!({"totalRuns": 1250, "successfulRuns": 1180, "failedRuns": 70,
                                     "successRate": 94.4, "totalGasUsedWei": "15000000000000000",
                                     "avgExecutionTimeMs": 2340});
    let chains = serde_json::json!([
        {"id": "chain_1", "chainId": 1, "name": "Ethereum Mainnet", "symbol": "ETH", "chainType": "evm",
         "explorerUrl": "https://etherscan.io", "explorerAddressPath": "/address/", "isTestnet": false,
         "isEnabled": true, "usePrivateMempoolRpc": false},
        {"id": "chain_2", "chainId": 11155111, "name": "Sepolia", "symbol": "ETH", "chainType": "evm",
         "explorerUrl": "https://sepolia.etherscan.io", "isTestnet": true, "isEnabled": true,
         "usePrivateMempoolRpc": false}
    ]);
    let logs = serde_json::json!({
        "execution": {"id": "exec_123", "workflowId": "wf_456", "status": "running"},
        "logs": [{
            "id": "log_001", "executionId": "exec_123", "nodeId": "transfer-1", "nodeName": "First transfer",
            "nodeType": "web3/transfer-funds", "status": "running",
            "input": {"amount": "0.1"}, "output": {}, "error": null, "duration": "1850",
            "startedAt": "2026-07-30T12:05:01Z", "completedAt": null, "iterationIndex": null,
            "forEachNodeId": null
        }]
    });
    let steps = serde_json::json!({"steps": [{
        "nodeId": "direct_789", "nodeName": "Transfer", "status": "completed",
        "output": {"success": true, "transactionHash": "0xabc123", "gasUsed": "21000000000000",
                   "gasUsedUnits": "21000", "effectiveGasPrice": "1000000042",
                   "transactionLink": "https://etherscan.io/tx/0xabc123"},
        "durationMs": 15000, "timestamp": "2026-07-30T12:00:15Z"
    }]});
    let exec_status = serde_json::json!({"executionId": "direct_789", "status": "completed", "type": "transfer",
        "transactionHash": "0xabc123",
        "transactionLink": "https://etherscan.io/tx/0xabc123",
        "gasUsedWei": "21000000000000", "result": {"success": true}, "error": null,
        "createdAt": "2026-07-30T12:00:00Z", "completedAt": "2026-07-30T12:00:15Z"});
    vec![
        ("GET /workflows".into(), wf.to_string()),
        ("GET /analytics/runs?limit=50".into(), runs.to_string()),
        ("GET /analytics/spend-cap".into(), spend.to_string()),
        ("GET /analytics/summary".into(), summary.to_string()),
        ("GET /chains".into(), chains.to_string()),
        (
            "GET /workflows/executions/exec_123/logs".into(),
            logs.to_string(),
        ),
        (
            "GET /analytics/runs/direct_789/steps".into(),
            steps.to_string(),
        ),
        (
            "GET /execute/direct_789/status".into(),
            exec_status.to_string(),
        ),
        (
            "POST /workflows/wf_123/execute".into(),
            serde_json::json!({"executionId": "exec_999", "status": "running"}).to_string(),
        ),
        (
            "POST /execute/transfer".into(),
            serde_json::json!({"executionId": "direct_999", "status": "completed"}).to_string(),
        ),
    ]
}

fn client(
    route_map: Vec<(String, String)>,
    expected_keys: Arc<Mutex<Vec<String>>>,
) -> (KhClient, std::thread::JoinHandle<()>) {
    let (addr, handle) = serve(Arc::new(Mutex::new(route_map)), expected_keys);
    (
        KhClient::with_base(Arc::new("kh_test_key".into()), addr),
        handle,
    )
}

#[tokio::test]
async fn dashboard_shapes_parse() {
    let expected = Arc::new(Mutex::new(Vec::new()));
    let (client, _server) = client(routes(), expected.clone());

    let workflows = client.list_workflows().await.expect("workflows parse");
    assert_eq!(workflows.len(), 1);
    assert_eq!(workflows[0].id, "wf_123");
    assert_eq!(workflows[0].name, "Auto-compounder");
    assert_eq!(
        workflows[0].updated_at.as_deref(),
        Some("2026-07-28T10:00:00Z")
    );

    let runs = client.analytics_runs(50).await.expect("runs parse");
    assert_eq!(runs.runs.len(), 2);
    let direct = runs.runs.iter().find(|r| r.source == "direct").unwrap();
    assert_eq!(direct.status, "completed");
    assert_eq!(
        direct.transaction_hash.as_deref(),
        Some("0xca19a1f0c1d48e6d8a4c1f9d1f3d7a1c9f3b4d5e6f7a8b9c0d1e2f3a4b5c6d7e8")
    );
    assert_eq!(direct.gas_used_wei.as_deref(), Some("21000000000000"));
    assert_eq!(direct.network.as_deref(), Some("ethereum"));
    let wf_run = runs.runs.iter().find(|r| r.source == "workflow").unwrap();
    assert_eq!(wf_run.workflow_id.as_deref(), Some("wf_123"));
    assert_eq!(wf_run.workflow_name.as_deref(), Some("Auto-compounder"));

    let spend = client.spend_cap().await.expect("spend parse");
    assert_eq!(spend.percent_used, 25.0);
    assert_eq!(spend.daily_cap_wei, "100000000000000000");

    let summary = client.analytics_summary().await.expect("summary parse");
    assert_eq!(summary.total_runs, 1250);
    assert_eq!(summary.success_rate, 94.4);

    let chains = client.chains().await.expect("chains parse");
    assert!(chains.iter().any(|c| c.is_testnet && c.is_enabled));
    assert_eq!(chains[0].chain_id, 1);

    let keys = expected.lock().unwrap().clone();
    assert!(
        keys.iter().all(|k| k.contains("auth=true")),
        "all requests must be bearer-auth'd: {keys:?}"
    );
}

#[tokio::test]
async fn logs_and_steps_parse() {
    let expected = Arc::new(Mutex::new(Vec::new()));
    let (client, _server) = client(routes(), expected.clone());

    let logs = client.execution_logs("exec_123").await.expect("logs parse");
    assert_eq!(logs.logs.len(), 1);
    let l = &logs.logs[0];
    assert_eq!(l.node_name.as_deref(), Some("First transfer"));
    assert_eq!(l.node_type.as_deref(), Some("web3/transfer-funds"));
    assert_eq!(l.status, "running");
    assert_eq!(l.started_at.as_deref(), Some("2026-07-30T12:05:01Z"));
    assert!(l.iteration_index.is_none());
    assert!(l.for_each_node_id.is_none());
    assert!(logs.execution.is_some());

    let steps = client.step_logs("direct_789").await.expect("steps parse");
    assert_eq!(steps.steps.len(), 1);
    let s = &steps.steps[0];
    assert_eq!(s.status, "completed");
    assert_eq!(s.output["transactionHash"], "0xabc123");
    assert_eq!(s.output["gasUsed"], "21000000000000");
}

#[tokio::test]
async fn execute_and_workflow_trigger() {
    let expected = Arc::new(Mutex::new(Vec::new()));
    let (client, _server) = client(routes(), expected.clone());

    let wf = client
        .execute_workflow("wf_123")
        .await
        .expect("execute workflow");
    assert_eq!(wf.execution_id, "exec_999");
    assert_eq!(wf.status, "running");

    let req = TransferRequest {
        chain_id: "1".into(),
        recipient_address: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".into(),
        amount: "0.1".into(),
    };
    let sim = client.simulate_transfer(&req).await.expect("simulate");
    assert!(sim.success);
    let resp = client
        .broadcast_transfer(&req, "idem-1")
        .await
        .expect("broadcast");
    assert_eq!(resp.execution_id, "direct_999");

    let status = client
        .direct_status("direct_789")
        .await
        .expect("direct status");
    assert_eq!(status.status, "completed");
    assert_eq!(
        status.transaction_link.as_deref(),
        Some("https://etherscan.io/tx/0xabc123")
    );
    assert_eq!(status.gas_used_wei.as_deref(), Some("21000000000000"));
}

#[tokio::test]
async fn simulate_revert_is_parsed_not_error() {
    let mut route_map: Vec<(String, String)> = routes()
        .into_iter()
        .filter(|(r, _)| r != "POST /execute/transfer")
        .collect();
    route_map.push((
        "POST /execute/transfer".into(),
        serde_json::json!({"success": false, "status": "simulated", "from": "0xorg",
                           "to": "0xtarget", "value": "0", "wouldRevert": true,
                           "revertReason": "Error(ERC20: transfer amount exceeds balance)",
                           "error": "Error(ERC20: transfer amount exceeds balance)"})
        .to_string(),
    ));
    let expected = Arc::new(Mutex::new(Vec::new()));
    let (client, _server) = client(route_map, expected.clone());
    let req = TransferRequest {
        chain_id: "1".into(),
        recipient_address: "0xdead".into(),
        amount: "9999".into(),
    };
    let sim = client
        .simulate_transfer(&req)
        .await
        .expect("revert is a parseable response, not an error");
    assert!(sim.would_revert);
    assert!(!sim.success);
    assert_eq!(
        sim.revert_reason.as_deref(),
        Some("Error(ERC20: transfer amount exceeds balance)")
    );
}

#[tokio::test]
async fn api_errors_surface_code_detail_request_id() {
    let expected = Arc::new(Mutex::new(Vec::new()));
    let (client, _server) = client(Vec::new(), expected.clone());
    let err = client.list_workflows().await.expect_err("must error");
    let msg = err.to_string();
    assert!(msg.contains("not_found"), "error code in message: {msg}");
    assert!(msg.contains("request_id"), "request id in message: {msg}");
    assert!(msg.contains("mock-1"), "request id value: {msg}");
}
