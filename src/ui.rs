use crate::app::{App, Mode, Pane};
use crate::util;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Clear, Gauge, List, ListDirection, ListItem, ListState, Paragraph, Wrap,
};
use ratatui::Frame;

fn status_color(s: &str) -> Color {
    match s {
        "pending" => Color::Yellow,
        "running" => Color::Cyan,
        "success" | "completed" => Color::Green,
        "error" | "failed" => Color::Red,
        "cancelled" => Color::DarkGray,
        _ => Color::Gray,
    }
}

fn status_span(s: &str) -> Span<'static> {
    Span::styled(
        format!("{s:>9}"),
        Style::default()
            .fg(status_color(s))
            .add_modifier(Modifier::BOLD),
    )
}

fn focus_border(app: &App, pane: Pane) -> Style {
    if app.focus == pane {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Percentage(38),
        Constraint::Min(0),
        Constraint::Length(8),
        Constraint::Length(1),
    ])
    .split(area);

    draw_header(f, app, chunks[0]);
    draw_top(f, app, chunks[1]);
    draw_logs(f, app, chunks[2]);
    draw_gas(f, app, chunks[3]);
    draw_footer(f, app, chunks[4]);

    match app.mode {
        Mode::Help => draw_help(f),
        Mode::TransferAmount => draw_transfer_dialog(f, app, false),
        Mode::TransferConfirm => draw_transfer_dialog(f, app, true),
        _ => {}
    }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let left = format!(
        " khtop — KeeperHub dashboard  ◉ {}  last refresh {}{}{}",
        if app.error.is_some() {
            "degraded".to_string()
        } else if app.runs.is_empty() && app.last_refresh.is_none() {
            "connecting…".to_string()
        } else {
            "live".to_string()
        },
        app.last_refresh
            .map(|t| format!(
                "{:02}:{:02}:{:02}",
                t.elapsed().as_secs() / 60,
                (t.elapsed().as_secs() % 60) / 60 % 60,
                t.elapsed().as_secs() % 60
            ))
            .unwrap_or_default(),
        "s ago",
        if app.client.rate_remaining() > 0 {
            format!("  ·  rate-limit rem {}", app.client.rate_remaining())
        } else {
            String::new()
        },
    );
    let right = format!(
        "{} runs · {} workflows  ",
        app.runs.len(),
        app.workflows.len()
    );
    let line = Line::from(vec![
        Span::styled(left, Style::default().fg(Color::Cyan)),
        Span::raw(" "),
        Span::styled(right, Style::default().fg(Color::DarkGray)),
    ]);
    f.render_widget(Paragraph::new(line), area);
}

fn run_list_item(r: &crate::client::models::Run) -> ListItem<'static> {
    let gas = r
        .gas_used_wei
        .as_deref()
        .and_then(util::wei_to_eth)
        .map(|g| format!("{g} ETH"))
        .unwrap_or_default();
    let wf = r.workflow_name.as_deref().unwrap_or("—");
    let wf = truncate(wf, 26);
    let time = r
        .created_at
        .as_deref()
        .map(util::fmt_time)
        .unwrap_or_default();
    ListItem::new(Line::from(vec![
        status_span(&r.status),
        Span::styled(
            format!(" {:>6} ", r.source),
            Style::default().fg(Color::Magenta),
        ),
        Span::styled(
            format!("{} ", truncate(&r.id, 28)),
            Style::default().fg(Color::White),
        ),
        Span::styled(wf, Style::default().fg(Color::Gray)),
        Span::styled(format!(" {gas:>12}"), Style::default().fg(Color::DarkGray)),
        Span::styled(format!(" {time}"), Style::default().fg(Color::DarkGray)),
    ]))
}

fn workflow_trigger(wf: &crate::client::models::Workflow) -> String {
    if let Some(nodes) = wf.nodes.as_array() {
        for n in nodes {
            let is_trigger = n.get("type").and_then(|v| v.as_str()) == Some("trigger")
                || n.get("data")
                    .and_then(|d| d.get("type"))
                    .and_then(|v| v.as_str())
                    == Some("trigger");
            if is_trigger {
                if let Some(ct) = n
                    .get("data")
                    .and_then(|d| d.get("config"))
                    .and_then(|c| c.get("triggerType"))
                    .and_then(|v| v.as_str())
                {
                    return ct.to_string();
                }
            }
        }
    }
    "—".to_string()
}

fn workflow_list_item(w: &crate::client::models::Workflow) -> ListItem<'static> {
    let dot = if w.enabled.unwrap_or(true) {
        "■"
    } else {
        "□"
    };
    let trigger = workflow_trigger(w);
    let updated = w
        .updated_at
        .as_deref()
        .map(util::fmt_time)
        .unwrap_or_default();
    ListItem::new(Line::from(vec![
        Span::styled(
            format!("{dot} "),
            Style::default().fg(if w.enabled.unwrap_or(true) {
                Color::Green
            } else {
                Color::DarkGray
            }),
        ),
        Span::styled(truncate(&w.name, 30), Style::default().fg(Color::White)),
        Span::styled(
            format!(" {:>12} ", trigger),
            Style::default().fg(Color::Yellow),
        ),
        Span::styled(truncate(&w.id, 26), Style::default().fg(Color::Gray)),
        Span::styled(format!(" {updated}"), Style::default().fg(Color::DarkGray)),
    ]))
}

fn draw_top(f: &mut Frame, app: &App, area: Rect) {
    let cols =
        Layout::horizontal([Constraint::Percentage(58), Constraint::Percentage(42)]).split(area);
    let runs_block = Block::bordered()
        .title(format!(
            " Runs  ({} / filter: {})",
            app.filtered_runs().len(),
            if app.filter.is_empty() {
                "-"
            } else {
                &app.filter
            }
        ))
        .border_style(focus_border(app, Pane::Runs));
    let runs_items: Vec<ListItem> = app
        .filtered_runs()
        .iter()
        .map(|r| run_list_item(r))
        .collect();
    let mut rs = ListState::default();
    rs.select(app.selected_run_index());
    f.render_stateful_widget(
        List::new(runs_items).block(runs_block).highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
        cols[0],
        &mut rs,
    );

    let wf_block = Block::bordered()
        .title(format!(" Workflows  ({})", app.workflows.len()))
        .border_style(focus_border(app, Pane::Workflows));
    let wf_items: Vec<ListItem> = app.workflows.iter().map(workflow_list_item).collect();
    let mut ws = ListState::default();
    ws.select(app.selected_workflow_index());
    f.render_stateful_widget(
        List::new(wf_items).block(wf_block).highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
        cols[1],
        &mut ws,
    );
}

fn log_row_item(row: &crate::app::LogRow) -> ListItem<'static> {
    let mut spans = vec![
        Span::styled(
            format!("{} ", row.time),
            Style::default().fg(Color::DarkGray),
        ),
        status_span(&row.status),
        Span::styled(
            format!(" {}", truncate(&row.node_name, 22)),
            Style::default().fg(Color::White),
        ),
        Span::styled(
            format!(" ({})", truncate(&row.node_type, 18)),
            Style::default().fg(Color::DarkGray),
        ),
    ];
    if let Some(g) = &row.gas_eth {
        spans.push(Span::styled(
            format!(" gas {g}"),
            Style::default().fg(Color::Yellow),
        ));
    }
    if let Some(h) = &row.tx_hash {
        spans.push(Span::styled(
            format!(" tx {}", util::short_hash(h)),
            Style::default().fg(Color::Cyan),
        ));
    }
    if let Some(l) = &row.tx_link {
        spans.push(Span::styled(
            format!(" {}", truncate(l, 60)),
            Style::default().fg(Color::Blue),
        ));
    }
    if let Some(m) = &row.message {
        spans.push(Span::styled(
            format!(" ✗ {m}"),
            Style::default().fg(Color::Red),
        ));
    }
    ListItem::new(Line::from(spans))
}

fn draw_logs(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::bordered()
        .title(format!(
            " Audit tail  {}",
            app.log_run_id
                .as_deref()
                .map(|id| format!("({id})"))
                .unwrap_or_default()
        ))
        .border_style(focus_border(app, Pane::Logs));
    let items: Vec<ListItem> = app.logs.iter().map(log_row_item).collect();
    let mut state = ListState::default();
    *state.offset_mut() = app.log_scroll as usize;
    f.render_stateful_widget(
        List::new(items)
            .block(block)
            .direction(ListDirection::BottomToTop)
            .highlight_symbol("▎"),
        area,
        &mut state,
    );
}

fn draw_gas(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::bordered()
        .title(" Wallet / gas ")
        .border_style(focus_border(app, Pane::Gas));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let parts =
        Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)]).split(inner);
    let mut lines: Vec<Line> = Vec::new();
    if let Some(sp) = &app.spend {
        let pct = sp.percent_used.clamp(0.0, 100.0) as u16;
        let spent = util::wei_to_eth(&sp.spent_today_wei).unwrap_or_default();
        let cap = util::wei_to_eth(&sp.daily_cap_wei).unwrap_or_default();
        lines.push(Line::from(Span::styled(
            format!(" spend cap   {pct:>5.1}%  ({spent} / {cap} ETH today)"),
            Style::default().fg(Color::Yellow),
        )));
        f.render_widget(
            Gauge::default()
                .block(
                    Block::default()
                        .title(" daily spend cap ")
                        .borders(Borders::ALL),
                )
                .gauge_style(Style::default().fg(if pct > 90 { Color::Red } else { Color::Green }))
                .percent(pct),
            parts[1],
        );
    } else {
        lines.push(Line::from(Span::styled(
            " spend cap: n/a",
            Style::default().fg(Color::DarkGray),
        )));
    }
    if let Some(s) = &app.summary {
        lines.push(Line::from(format!(
            " summary: {} runs · success {:.1}% · avg {:.0} ms · total gas {} ETH",
            s.total_runs,
            s.success_rate,
            s.avg_execution_time_ms,
            util::wei_to_eth(&s.total_gas_used_wei).unwrap_or_default()
        )));
    }
    match &app.wallet_from {
        Some(addr) => lines.push(Line::from(format!(" wallet: {addr}"))),
        None => lines.push(Line::from(Span::styled(
            " wallet: unknown — run a simulate or an execution to discover the org wallet",
            Style::default().fg(Color::DarkGray),
        ))),
    }
    let enabled = app.chains.iter().filter(|c| c.is_enabled).count();
    let testnets = app
        .chains
        .iter()
        .filter(|c| c.is_enabled && c.is_testnet)
        .count();
    lines.push(Line::from(format!(
        " chains: {enabled} enabled ({testnets} testnet) · gas sponsorship: ETH/Base/Polygon/Arbitrum + testnets"
    )));
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), parts[0]);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let (msg, err): (String, bool) = if app.mode == Mode::Normal {
        match (&app.toast, &app.error) {
            (Some((t, _)), _) => (format!(" {t}"), false),
            (None, Some(e)) => (format!(" ⚠ {e}"), true),
            (None, None) => (
                " j/k ↑↓ select · Enter logs · Tab focus · r run workflow · t transfer · / filter · PgUp/PgDn scroll logs · ? help · q quit".to_string(),
                false,
            ),
        }
    } else {
        ("".to_string(), false)
    };
    let (style, text): (Style, String) = if err {
        (Style::default().fg(Color::Red), msg)
    } else if app.mode == Mode::Normal {
        (Style::default().fg(Color::DarkGray), msg)
    } else {
        (Style::default(), String::new())
    };
    f.render_widget(Paragraph::new(Span::styled(text, style)), area);
}

fn popup_rect(area: Rect) -> Rect {
    let w = area.width.min(78) / 2;
    let h = area.height.min(18) / 2;
    let x = (area.width - w) / 2;
    let y = (area.height - h) / 2;
    Rect::new(x, y, w, h)
}

fn draw_transfer_dialog(f: &mut Frame, app: &App, confirm: bool) {
    let area = popup_rect(f.area());
    let Some(t) = &app.transfer else { return };
    let mut lines: Vec<Line> = vec![];
    lines.push(Line::from(Span::styled(
        if confirm {
            " CONFIRM TRANSFER "
        } else {
            " TRANSFER (native token) "
        },
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(format!(
        " chain:   {}   recipient: {}",
        std::env::var("KH_DEMO_CHAIN_ID").unwrap_or_else(|_| "11155111".into()),
        std::env::var("KH_DEMO_RECIPIENT").unwrap_or_default()
    )));
    let cursor = if app.mode == Mode::TransferAmount {
        "_"
    } else {
        ""
    };
    lines.push(Line::from(format!(" amount:  {} ETH {cursor}", t.amount)));
    if let Some(sim) = &t.sim {
        lines.push(Line::from(""));
        lines.push(Line::from(format!(
            " simulate: gasEstimate={} from={}",
            sim.gas_estimate.as_deref().unwrap_or("?"),
            sim.from.as_deref().unwrap_or("?")
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        if confirm {
            " Enter = broadcast (spends funds) · Esc = back to amount"
        } else {
            " Enter = simulate (dry run, no tx) · Esc = cancel"
        },
        Style::default().fg(Color::DarkGray),
    )));
    let block = Block::bordered()
        .title(" khtop action ")
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(Clear, area);
    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn draw_help(f: &mut Frame) {
    let area = popup_rect(f.area());
    let lines: Vec<Line> = vec![
        Line::from(Span::styled(
            " khtop keys ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(" j/k or ↑/↓      move selection"),
        Line::from(" Enter            load audit logs for selected run"),
        Line::from(" Tab              switch pane (runs → workflows → logs → gas)"),
        Line::from(" r                trigger a run of the selected workflow"),
        Line::from(" t                direct transfer (simulate → broadcast)"),
        Line::from(" /                filter runs (by id, status, name)"),
        Line::from(" PgUp / PgDn      scroll the audit tail"),
        Line::from(" ? or Esc         close this help"),
        Line::from(" q or Ctrl-C      quit"),
        Line::from(""),
        Line::from(Span::styled(
            " data source: KeeperHub REST API (app.keeperhub.com/api)",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            " demo transfer config: KH_DEMO_CHAIN_ID / KH_DEMO_RECIPIENT / KH_DEMO_AMOUNT",
            Style::default().fg(Color::DarkGray),
        )),
    ];
    let block = Block::bordered().border_style(Style::default().fg(Color::Cyan));
    f.render_widget(Clear, area);
    f.render_widget(
        Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Left),
        area,
    );
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}
