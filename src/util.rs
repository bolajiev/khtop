pub fn wei_to_eth(wei: &str) -> Option<String> {
    let v: f64 = wei.trim().parse().ok()?;
    Some(format_eth(v / 1e18))
}

pub fn format_eth(eth: f64) -> String {
    if eth >= 1.0 {
        format!("{:.4}", eth)
    } else if eth >= 1e-6 {
        format!("{:.6}", eth)
    } else if eth > 0.0 {
        format!("{:.2e}", eth)
    } else {
        "0".to_string()
    }
}

pub fn short_hash(h: &str) -> String {
    if h.len() > 14 {
        format!("{}…{}", &h[..8], &h[h.len() - 6..])
    } else {
        h.to_string()
    }
}

pub fn fmt_time(iso: &str) -> String {
    match chrono::DateTime::parse_from_rfc3339(iso) {
        Ok(dt) => dt.format("%m-%d %H:%M:%S").to_string(),
        Err(_) => iso.chars().take(19).collect(),
    }
}

pub fn terminal_status(s: &str) -> bool {
    matches!(
        s,
        "success" | "error" | "cancelled" | "completed" | "failed"
    )
}
