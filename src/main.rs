use anyhow::Result;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use khtop::app;
use khtop::client;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::sync::Arc;

fn main() -> Result<()> {
    load_env();
    let mut args = std::env::args().skip(1);
    let mut once = false;
    let mut simulate = false;
    let mut transfer = false;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--once" => once = true,
            "--simulate-transfer" => simulate = true,
            "--transfer" => transfer = true,
            "--recipient" => {
                let v = args.next().expect("--recipient needs a value");
                std::env::set_var("KH_DEMO_RECIPIENT", v);
            }
            "--amount" => {
                let v = args.next().expect("--amount needs a value");
                std::env::set_var("KH_DEMO_AMOUNT", v);
            }
            "--chain" => {
                let v = args.next().expect("--chain needs a value");
                std::env::set_var("KH_DEMO_CHAIN_ID", v);
            }
            other => {
                eprintln!("khtop: unknown argument {other}");
                print_usage();
                std::process::exit(2);
            }
        }
    }

    let api_key = std::env::var("KH_API_KEY").unwrap_or_default();
    if api_key.is_empty() || api_key == "kh_your_api_key" {
        eprintln!("khtop: KH_API_KEY is not set");
        eprintln!("  Copy .env.example to .env and add your key, or export KH_API_KEY.");
        eprintln!("  Create a key at app.keeperhub.com -> Settings -> API Keys -> Organisation tab (kh_ prefix).");
        std::process::exit(1);
    }

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let client = client::KhClient::new(Arc::new(api_key));
    rt.block_on(async move {
        if once {
            return app::run_once(&client).await;
        }
        if simulate || transfer {
            return app::spike_transfer(&client, simulate).await;
        }
        run_tui(client).await
    })
}

fn print_usage() {
    eprintln!("usage: khtop [--once] [--simulate-transfer|--transfer [--recipient A --amount X --chain ID]]");
}

/// Load .env from the current directory first (dev/repo convention), then from
/// ~/.config/khtop/.env or ~/.khtop.env so `khtop` works from any directory.
fn load_env() {
    dotenvy::dotenv().ok();
    if std::env::var("KH_API_KEY").is_ok() {
        return;
    }
    for p in ["~/.config/khtop/.env", "~/.khtop.env"] {
        if let Some(expanded) = expand_home(p) {
            let _ = dotenvy::from_path(&expanded);
            if std::env::var("KH_API_KEY").is_ok() {
                return;
            }
        }
    }
}

fn expand_home(p: &str) -> Option<std::path::PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(std::path::PathBuf::from(p.replacen("~", &home, 1)))
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(io::stdout()))?)
}

fn teardown_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

async fn run_tui(client: client::KhClient) -> Result<()> {
    let mut terminal = setup_terminal()?;
    let res = app::App::new(client).run(&mut terminal).await;
    teardown_terminal(&mut terminal)?;
    res
}
