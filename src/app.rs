use crate::client::models::*;
use crate::client::{ApiError, KhClient, SimulateOutcome, TransferRequest, TransferResponse};
use crate::util;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use serde_json::Value;
use std::io;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

pub const REFRESH_SECS: u64 = 5;
pub const LOG_POLL_SECS: u64 = 2;

pub struct Dashboard {
    pub runs: Vec<Run>,
    pub workflows: Vec<Workflow>,
    pub summary: Option<AnalyticsSummary>,
    pub spend: Option<SpendCap>,
    pub chains: Vec<Chain>,
    pub analytics_ok: bool,
}

pub struct LogRow {
    pub time: String,
    pub node_name: String,
    pub node_type: String,
    pub status: String,
    pub gas_eth: Option<String>,
    pub tx_hash: Option<String>,
    pub tx_link: Option<String>,
    pub message: Option<String>,
}

impl LogRow {
    fn from_log_entry(e: &LogEntry) -> LogRow {
        let out = &e.output;
        let failed = out.get("success").and_then(|s| s.as_bool()) == Some(false);
        let message = if failed {
            out.get("error")
                .and_then(|x| x.as_str())
                .or(e.error.as_deref())
                .map(String::from)
        } else {
            e.error.clone()
        };
        LogRow {
            time: e
                .started_at
                .as_deref()
                .map(util::fmt_time)
                .unwrap_or_default(),
            node_name: e
                .node_name
                .clone()
                .unwrap_or_else(|| e.node_id.clone().unwrap_or_default()),
            node_type: e.node_type.clone().unwrap_or_default(),
            status: e.status.clone(),
            gas_eth: out
                .get("gasUsed")
                .and_then(|x| x.as_str())
                .and_then(util::wei_to_eth),
            tx_hash: out
                .get("transactionHash")
                .and_then(|x| x.as_str())
                .map(String::from),
            tx_link: out
                .get("transactionLink")
                .and_then(|x| x.as_str())
                .map(String::from),
            message,
        }
    }

    fn from_step(s: &StepEntry) -> LogRow {
        let out = &s.output;
        let failed = out.get("success").and_then(|x| x.as_bool()) == Some(false);
        LogRow {
            time: s
                .timestamp
                .as_deref()
                .map(util::fmt_time)
                .unwrap_or_default(),
            node_name: s
                .node_name
                .clone()
                .unwrap_or_else(|| s.node_id.clone().unwrap_or_default()),
            node_type: "direct".to_string(),
            status: s.status.clone(),
            gas_eth: out
                .get("gasUsed")
                .and_then(|x| x.as_str())
                .and_then(util::wei_to_eth),
            tx_hash: out
                .get("transactionHash")
                .and_then(|x| x.as_str())
                .map(String::from),
            tx_link: out
                .get("transactionLink")
                .and_then(|x| x.as_str())
                .map(String::from),
            message: if failed {
                out.get("error").and_then(|x| x.as_str()).map(String::from)
            } else {
                None
            },
        }
    }
}

pub struct LogsData {
    pub rows: Vec<LogRow>,
    pub execution: Option<Value>,
}

pub enum Msg {
    Dashboard(Result<Dashboard, String>),
    Logs(Result<LogsData, String>),
    Simulate(Result<SimulateOutcome, String>),
    Broadcast(Result<TransferResponse, String>),
    RunStarted(Result<WorkflowExecuteResponse, String>),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Runs,
    Workflows,
    Logs,
    Gas,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Filter,
    TransferAmount,
    TransferConfirm,
    Help,
}

pub struct TransferState {
    pub amount: String,
    pub sim: Option<SimulateOutcome>,
}

pub struct App {
    pub client: KhClient,
    pub runs: Vec<Run>,
    pub workflows: Vec<Workflow>,
    pub summary: Option<AnalyticsSummary>,
    pub spend: Option<SpendCap>,
    pub chains: Vec<Chain>,
    pub wallet_from: Option<String>,
    pub logs: Vec<LogRow>,
    pub log_execution: Option<Value>,
    pub log_run_id: Option<String>,
    pub active_poll: Option<String>,
    pub selected_run: Option<String>,
    pub selected_wf: Option<String>,
    pub focus: Pane,
    pub mode: Mode,
    pub filter: String,
    pub transfer: Option<TransferState>,
    pub toast: Option<(String, Instant)>,
    pub error: Option<String>,
    pub last_refresh: Option<Instant>,
    pub log_scroll: u16,
    pub analytics_ok: bool,
}

impl App {
    pub fn new(client: KhClient) -> Self {
        App {
            client,
            runs: Vec::new(),
            workflows: Vec::new(),
            summary: None,
            spend: None,
            chains: Vec::new(),
            wallet_from: None,
            logs: Vec::new(),
            log_execution: None,
            log_run_id: None,
            active_poll: None,
            selected_run: None,
            selected_wf: None,
            focus: Pane::Runs,
            mode: Mode::Normal,
            filter: String::new(),
            transfer: None,
            toast: None,
            error: None,
            last_refresh: None,
            log_scroll: 0,
            analytics_ok: true,
        }
    }

    pub fn filtered_runs(&self) -> Vec<&Run> {
        let f = self.filter.trim().to_lowercase();
        self.runs
            .iter()
            .filter(|r| {
                f.is_empty()
                    || r.id.to_lowercase().contains(&f)
                    || r.status.to_lowercase().contains(&f)
                    || r.source.to_lowercase().contains(&f)
                    || r.workflow_name
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&f)
            })
            .collect()
    }

    pub fn selected_run_index(&self) -> Option<usize> {
        self.selected_run
            .as_ref()
            .and_then(|id| self.filtered_runs().iter().position(|r| &r.id == id))
    }

    pub fn selected_workflow_index(&self) -> Option<usize> {
        self.selected_wf
            .as_ref()
            .and_then(|id| self.workflows.iter().position(|w| &w.id == id))
    }

    pub fn set_toast(&mut self, msg: String) {
        self.toast = Some((msg, Instant::now()));
    }

    pub async fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> anyhow::Result<()> {
        let (input_tx, mut input_rx) = mpsc::channel(64);
        crate::events::spawn_input(input_tx);
        let (msg_tx, mut msg_rx) = mpsc::channel(64);

        let mut ticker = tokio::time::interval(Duration::from_secs(REFRESH_SECS));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut log_ticker = tokio::time::interval(Duration::from_secs(LOG_POLL_SECS));
        log_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        self.refresh_dashboard(msg_tx.clone());

        loop {
            tokio::select! {
                _ = ticker.tick() => self.refresh_dashboard(msg_tx.clone()),
                _ = log_ticker.tick() => self.poll_logs(msg_tx.clone()),
                Some(msg) = msg_rx.recv() => self.on_msg(msg),
                Some(ev) = input_rx.recv() => {
                    if !self.on_event(ev, msg_tx.clone()) {
                        break;
                    }
                }
            }
            terminal.draw(|f| crate::ui::draw(f, self))?;
            if let Some((_, at)) = self.toast {
                if at.elapsed() > Duration::from_secs(6) {
                    self.toast = None;
                }
            }
        }
        Ok(())
    }

    fn refresh_dashboard(&mut self, mtx: mpsc::Sender<Msg>) {
        let client = self.client.clone();
        let need_chains = self.chains.is_empty();
        let analytics_ok = self.analytics_ok;
        tokio::spawn(async move {
            let res = async {
                let chains = if need_chains {
                    client.chains().await.ok()
                } else {
                    None
                };

                let (runs, analytics_ok) = if analytics_ok {
                    match client.analytics_runs(50).await {
                        Ok(page) => (page.runs, true),
                        Err(e) if crate::client::is_scope_error(&e) => {
                            (client.fallback_runs().await, false)
                        }
                        Err(e) => return Err(e),
                    }
                } else {
                    (client.fallback_runs().await, false)
                };

                let spend = if analytics_ok {
                    client.spend_cap().await.ok()
                } else {
                    None
                };
                let summary = if analytics_ok {
                    client.analytics_summary().await.ok()
                } else {
                    None
                };
                let workflows = client.list_workflows().await?;
                Ok::<Dashboard, ApiError>(Dashboard {
                    runs,
                    workflows,
                    spend,
                    summary,
                    chains: chains.unwrap_or_default(),
                    analytics_ok,
                })
            }
            .await;
            let _ = mtx
                .send(Msg::Dashboard(res.map_err(|e| e.to_string())))
                .await;
        });
    }

    fn on_dashboard(&mut self, d: Dashboard) {
        self.analytics_ok = d.analytics_ok;
        self.runs = d.runs;
        if !d.workflows.is_empty() {
            self.workflows = d.workflows;
        }
        self.spend = d.spend;
        self.summary = d.summary;
        if !d.chains.is_empty() {
            self.chains = d.chains;
        }
        self.error = None;
        self.last_refresh = Some(Instant::now());
        if let Some(wf) = self.selected_wf.clone() {
            if !self.workflows.iter().any(|w| w.id == wf) {
                self.selected_wf = self.workflows.first().map(|w| w.id.clone());
            }
        }
        if let Some(run) = self.selected_run.clone() {
            if !self.runs.iter().any(|r| r.id == run) {
                self.selected_run = self.runs.first().map(|r| r.id.clone());
            }
        }
    }

    fn poll_logs(&mut self, mtx: mpsc::Sender<Msg>) {
        let Some(id) = self.active_poll.clone() else {
            return;
        };
        let Some(run) = self.runs.iter().find(|r| r.id == id) else {
            return;
        };
        let terminal = util::terminal_status(&run.status);
        if terminal {
            self.active_poll = None;
            return;
        }
        let client = self.client.clone();
        let source = run.source.clone();
        tokio::spawn(async move {
            let res: Result<LogsData, String> = if source == "direct" {
                client
                    .step_logs(&id)
                    .await
                    .map(|s| LogsData {
                        rows: s.steps.iter().map(LogRow::from_step).collect(),
                        execution: None,
                    })
                    .map_err(|e| e.to_string())
            } else {
                client
                    .execution_logs(&id)
                    .await
                    .map(|l| LogsData {
                        rows: l.logs.iter().map(LogRow::from_log_entry).collect(),
                        execution: l.execution,
                    })
                    .map_err(|e| e.to_string())
            };
            let _ = mtx.send(Msg::Logs(res)).await;
        });
    }

    fn fetch_logs_now(&mut self, run_id: &str, mtx: &mpsc::Sender<Msg>) {
        self.log_run_id = Some(run_id.to_string());
        self.logs.clear();
        self.log_execution = None;
        self.log_scroll = 0;
        if let Some(run) = self.runs.iter().find(|r| r.id == run_id) {
            if !util::terminal_status(&run.status) {
                self.active_poll = Some(run_id.to_string());
            } else {
                self.active_poll = None;
            }
        }
        self.poll_logs(mtx.clone());
    }

    fn on_msg(&mut self, msg: Msg) {
        match msg {
            Msg::Dashboard(res) => match res {
                Ok(d) => self.on_dashboard(d),
                Err(e) => self.error = Some(e),
            },
            Msg::Logs(res) => {
                let id = self.log_run_id.clone();
                match res {
                    Ok(l) => {
                        if let Some(exec) = l.execution.clone() {
                            if let Some(from) = extract_from_address(&exec) {
                                self.wallet_from = Some(from);
                            }
                        }
                        self.logs = l.rows;
                        self.log_execution = l.execution;
                        if let Some(id) = id {
                            if let Some(run) = self.runs.iter().find(|r| r.id == id) {
                                if util::terminal_status(&run.status) {
                                    self.active_poll = None;
                                }
                            }
                        }
                    }
                    Err(e) => self.error = Some(e),
                }
            }
            Msg::Simulate(res) => match res {
                Ok(sim) => {
                    if let Some(t) = self.transfer.as_mut() {
                        t.sim = Some(sim.clone());
                    }
                    if let Some(from) = sim.from.clone() {
                        self.wallet_from = Some(from);
                    }
                    if sim.would_revert {
                        self.mode = Mode::Normal;
                        self.set_toast(format!(
                            "simulation: WOULD REVERT{}",
                            sim.revert_reason
                                .as_ref()
                                .map(|r| format!(" — {r}"))
                                .unwrap_or_default()
                        ));
                    } else {
                        self.mode = Mode::TransferConfirm;
                    }
                }
                Err(e) => {
                    self.mode = Mode::TransferAmount;
                    self.set_toast(format!("simulation failed: {e}"));
                }
            },
            Msg::Broadcast(res) => match res {
                Ok(r) => {
                    self.mode = Mode::Normal;
                    self.transfer = None;
                    self.set_toast(format!(
                        "broadcast {} ({}) — tx incoming, watch the audit tail",
                        r.execution_id, r.status
                    ));
                    self.refresh_dashboard_raw();
                }
                Err(e) => {
                    self.mode = Mode::TransferConfirm;
                    self.set_toast(format!("broadcast failed: {e}"));
                }
            },
            Msg::RunStarted(res) => match res {
                Ok(r) => {
                    self.set_toast(format!(
                        "workflow run started: {} ({})",
                        r.execution_id, r.status
                    ));
                    self.refresh_dashboard_raw();
                }
                Err(e) => self.set_toast(format!("run failed: {e}")),
            },
        }
    }

    fn refresh_dashboard_raw(&mut self) {
        let (mtx, _) = mpsc::channel(64);
        self.refresh_dashboard(mtx);
    }

    fn on_event(&mut self, ev: Event, mtx: mpsc::Sender<Msg>) -> bool {
        let Event::Key(key) = ev else { return true };
        if key.kind == crossterm::event::KeyEventKind::Release {
            return true;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return false;
        }
        match self.mode {
            Mode::Filter => self.key_filter(key),
            Mode::TransferAmount => self.key_transfer_amount(key, &mtx),
            Mode::TransferConfirm => self.key_transfer_confirm(key, &mtx),
            Mode::Help => {
                if matches!(key.code, KeyCode::Esc | KeyCode::Char('?')) {
                    self.mode = Mode::Normal;
                }
            }
            Mode::Normal => {
                if !self.key_normal(key, &mtx) {
                    return false;
                }
            }
        }
        true
    }

    fn key_filter(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.filter.clear();
                self.mode = Mode::Normal;
            }
            KeyCode::Enter => self.mode = Mode::Normal,
            KeyCode::Backspace => {
                self.filter.pop();
            }
            KeyCode::Char(c) => self.filter.push(c),
            _ => {}
        }
    }

    fn key_transfer_amount(&mut self, key: KeyEvent, mtx: &mpsc::Sender<Msg>) {
        match key.code {
            KeyCode::Esc => {
                self.transfer = None;
                self.mode = Mode::Normal;
            }
            KeyCode::Enter => {
                let Some(t) = self.transfer.as_ref() else {
                    return;
                };
                let Ok(v) = t.amount.trim().parse::<f64>() else {
                    self.set_toast("invalid amount".into());
                    return;
                };
                if v <= 0.0 {
                    self.set_toast("amount must be > 0".into());
                    return;
                }
                self.start_simulate(mtx);
            }
            KeyCode::Backspace => {
                if let Some(t) = self.transfer.as_mut() {
                    t.amount.pop();
                }
            }
            KeyCode::Char(c) if c.is_ascii_digit() || c == '.' => {
                if let Some(t) = self.transfer.as_mut() {
                    if t.amount.contains('.') && c == '.' {
                        return;
                    }
                    t.amount.push(c);
                }
            }
            _ => {}
        }
    }

    fn key_transfer_confirm(&mut self, key: KeyEvent, mtx: &mpsc::Sender<Msg>) {
        match key.code {
            KeyCode::Esc => {
                if let Some(t) = self.transfer.as_mut() {
                    t.sim = None;
                }
                self.mode = Mode::TransferAmount;
            }
            KeyCode::Enter => {
                let Some(t) = self.transfer.as_ref() else {
                    return;
                };
                let Some(_sim) = t.sim.as_ref() else { return };
                let req = TransferRequest {
                    chain_id: demo_chain_id(),
                    recipient_address: demo_recipient(),
                    amount: t.amount.clone(),
                };
                let client = self.client.clone();
                let key = uuid::Uuid::new_v4().to_string();
                let tx = mtx.clone();
                tokio::spawn(async move {
                    let res = client.broadcast_transfer(&req, &key).await;
                    let _ = tx
                        .send(Msg::Broadcast(res.map_err(|e| e.to_string())))
                        .await;
                });
            }
            _ => {}
        }
    }

    fn start_simulate(&mut self, mtx: &mpsc::Sender<Msg>) {
        let client = self.client.clone();
        let tx = mtx.clone();
        let req = TransferRequest {
            chain_id: demo_chain_id(),
            recipient_address: demo_recipient(),
            amount: self.transfer.as_ref().unwrap().amount.clone(),
        };
        tokio::spawn(async move {
            let res = client.simulate_transfer(&req).await;
            let _ = tx.send(Msg::Simulate(res.map_err(|e| e.to_string()))).await;
        });
    }

    fn key_normal(&mut self, key: KeyEvent, mtx: &mpsc::Sender<Msg>) -> bool {
        match key.code {
            KeyCode::Char('q') => return false,
            KeyCode::Char('?') => self.mode = Mode::Help,
            KeyCode::Char('/') => {
                self.filter.clear();
                self.mode = Mode::Filter;
            }
            KeyCode::Tab => {
                self.focus = match self.focus {
                    Pane::Runs => Pane::Workflows,
                    Pane::Workflows => Pane::Logs,
                    Pane::Logs => Pane::Gas,
                    Pane::Gas => Pane::Runs,
                };
            }
            KeyCode::Char('r') => {
                let wf_id = match self.focus {
                    Pane::Runs => self.selected_run.as_ref().and_then(|id| {
                        self.runs
                            .iter()
                            .find(|r| &r.id == id)
                            .and_then(|r| r.workflow_id.clone())
                    }),
                    _ => self.selected_wf.clone(),
                };
                if let Some(wf_id) = wf_id {
                    let client = self.client.clone();
                    let tx = mtx.clone();
                    tokio::spawn(async move {
                        let res = client.execute_workflow(&wf_id).await;
                        let _ = tx
                            .send(Msg::RunStarted(res.map_err(|e| e.to_string())))
                            .await;
                    });
                } else {
                    self.set_toast(
                        "no workflow selected to run — move to the Workflows pane first".into(),
                    );
                }
            }
            KeyCode::Char('t') => {
                if demo_recipient().is_empty() {
                    self.set_toast("transfer needs KH_DEMO_RECIPIENT set in .env".into());
                } else {
                    self.transfer = Some(TransferState {
                        amount: demo_amount(),
                        sim: None,
                    });
                    self.mode = Mode::TransferAmount;
                }
            }
            KeyCode::Enter => {
                if self.focus == Pane::Runs {
                    if let Some(id) = self.selected_run.clone() {
                        self.fetch_logs_now(&id, mtx);
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.focus == Pane::Runs {
                    let n = self.filtered_runs().len();
                    if let Some(i) = self.selected_run_index() {
                        if i + 1 < n {
                            self.selected_run = self.filtered_runs()[i + 1].id.clone().into();
                        }
                    } else if n > 0 {
                        self.selected_run = self.filtered_runs()[0].id.clone().into();
                    }
                } else if self.focus == Pane::Workflows {
                    let n = self.workflows.len();
                    if let Some(i) = self.selected_workflow_index() {
                        if i + 1 < n {
                            self.selected_wf = self.workflows[i + 1].id.clone().into();
                        }
                    } else if n > 0 {
                        self.selected_wf = self.workflows[0].id.clone().into();
                    }
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.focus == Pane::Runs {
                    if let Some(i) = self.selected_run_index() {
                        if i > 0 {
                            self.selected_run = self.filtered_runs()[i - 1].id.clone().into();
                        }
                    }
                } else if self.focus == Pane::Workflows {
                    if let Some(i) = self.selected_workflow_index() {
                        if i > 0 {
                            self.selected_wf = self.workflows[i - 1].id.clone().into();
                        }
                    }
                }
            }
            KeyCode::Home | KeyCode::Char('g') => {
                if self.focus == Pane::Runs && !self.filtered_runs().is_empty() {
                    self.selected_run = self.filtered_runs()[0].id.clone().into();
                } else if self.focus == Pane::Workflows && !self.workflows.is_empty() {
                    self.selected_wf = self.workflows[0].id.clone().into();
                }
            }
            KeyCode::End | KeyCode::Char('G') => {
                let runs = self.filtered_runs();
                if self.focus == Pane::Runs && !runs.is_empty() {
                    self.selected_run = runs[runs.len() - 1].id.clone().into();
                } else if self.focus == Pane::Workflows && !self.workflows.is_empty() {
                    self.selected_wf = self.workflows[self.workflows.len() - 1].id.clone().into();
                }
            }
            KeyCode::PageDown => self.log_scroll = self.log_scroll.saturating_add(5),
            KeyCode::PageUp => self.log_scroll = self.log_scroll.saturating_sub(5),
            _ => {}
        }
        true
    }
}

fn extract_from_address(exec: &Value) -> Option<String> {
    for k in ["from", "fromAddress"] {
        if let Some(s) = exec.get(k).and_then(|v| v.as_str()) {
            return Some(s.to_string());
        }
    }
    None
}

fn demo_chain_id() -> String {
    std::env::var("KH_DEMO_CHAIN_ID").unwrap_or_else(|_| "11155111".to_string())
}

fn demo_recipient() -> String {
    std::env::var("KH_DEMO_RECIPIENT").unwrap_or_default()
}

fn demo_amount() -> String {
    std::env::var("KH_DEMO_AMOUNT").unwrap_or_else(|_| "0.0001".to_string())
}

pub async fn run_once(client: &KhClient) -> anyhow::Result<()> {
    let workflows = client.list_workflows().await?;
    let chains = client.chains().await?;
    let (runs, analytics_ok) = match client.analytics_runs(50).await {
        Ok(page) => (page.runs, true),
        Err(e) if crate::client::is_scope_error(&e) => (client.fallback_runs().await, false),
        Err(e) => return Err(e.into()),
    };
    let spend = if analytics_ok {
        client.spend_cap().await.ok()
    } else {
        None
    };
    let summary = if analytics_ok {
        client.analytics_summary().await.ok()
    } else {
        None
    };
    let out = serde_json::json!({
        "analytics_in_scope": analytics_ok,
        "workflows": workflows.len(),
        "workflow_names": workflows.iter().map(|w| w.name.clone()).collect::<Vec<_>>(),
        "runs": runs.len(),
        "recent_runs": runs.iter().take(5).map(|r| serde_json::json!({
            "id": r.id, "source": r.source, "status": r.status, "tx": r.transaction_hash,
        })).collect::<Vec<_>>(),
        "spend_cap": spend,
        "summary": summary,
        "chains_enabled": chains.iter().filter(|c| c.is_enabled).map(|c| format!("{} (testnet:{})", c.name, c.is_testnet)).collect::<Vec<_>>(),
        "rate_limit_remaining": client.rate_remaining(),
    });
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}

pub async fn spike_transfer(client: &KhClient, simulate_only: bool) -> anyhow::Result<()> {
    let req = TransferRequest {
        chain_id: demo_chain_id(),
        recipient_address: demo_recipient(),
        amount: demo_amount(),
    };
    if req.recipient_address.is_empty() {
        anyhow::bail!("KH_DEMO_RECIPIENT is not set (0x address to send to)");
    }
    println!(
        "simulating transfer on chain {} to {} amount {} ETH …",
        req.chain_id, req.recipient_address, req.amount
    );
    let sim = client
        .simulate_transfer(&req)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    println!(
        "simulate: success={} would_revert={} gas_estimate={} from={} to={} value={}{}",
        sim.success,
        sim.would_revert,
        sim.gas_estimate.as_deref().unwrap_or("?"),
        sim.from.as_deref().unwrap_or("?"),
        sim.to.as_deref().unwrap_or("?"),
        sim.value.as_deref().unwrap_or("?"),
        sim.revert_reason
            .as_ref()
            .map(|r| format!(" revert: {r}"))
            .unwrap_or_default(),
    );
    if sim.would_revert {
        anyhow::bail!("simulation reverted — not broadcasting");
    }
    if simulate_only {
        return Ok(());
    }
    let key = uuid::Uuid::new_v4().to_string();
    println!("simulation OK — broadcasting with Idempotency-Key {key} …");
    let r = client
        .broadcast_transfer(&req, &key)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    println!(
        "broadcast: execution_id={} status={}",
        r.execution_id, r.status
    );
    let status = client
        .direct_status(&r.execution_id)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    println!(
        "status: {} tx={} gas={} link={}",
        status.status,
        status.transaction_hash.as_deref().unwrap_or("pending"),
        status.gas_used_wei.as_deref().unwrap_or("?"),
        status.transaction_link.as_deref().unwrap_or("")
    );
    Ok(())
}
