//! `kurultai webui` — reach the Brain UI without memorizing
//! `daemon --port 8421` + `/ui/`. If a daemon already serves the port, print
//! its URL; otherwise spawn one, wait for readiness, then print the URL.

use crate::error::{KurultaiError, Result};
use std::io::IsTerminal;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const READY_TIMEOUT: Duration = Duration::from_secs(10);
const POLL_EVERY: Duration = Duration::from_millis(150);

pub struct WebuiOptions {
    pub port: u16,
    /// Open the browser (`--open`; default when stdout is a TTY).
    pub open: bool,
    /// `--print-url`: print and exit — for headless agents.
    pub print_url: bool,
    /// Extra args forwarded to `daemon` (e.g. `--bind tailscale`).
    pub daemon_args: Vec<String>,
}

/// Entry point: ensure a daemon, print the URL, maybe open a browser.
pub async fn run(opts: WebuiOptions) -> Result<()> {
    let base = format!("http://127.0.0.1:{}", opts.port);
    let ui = format!("{base}/ui/");

    let mut spawned: Option<Child> = None;
    if !probe(&base).await {
        spawned = Some(spawn_daemon(&opts)?);
        wait_ready(&base).await?;
    }

    println!("{ui}");

    let should_open = opts.open && !opts.print_url && std::io::stdout().is_terminal();
    if should_open {
        open_browser(&ui);
    }
    drop(spawned);
    Ok(())
}

/// `GET /api/status` — true when a daemon answers 200.
async fn probe(base: &str) -> bool {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(1))
        .build();
    let Ok(client) = client else { return false };
    client
        .get(format!("{base}/api/status"))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

fn spawn_daemon(opts: &WebuiOptions) -> Result<Child> {
    let exe = std::env::current_exe()
        .map_err(|e| KurultaiError::config(format!("current_exe for daemon spawn: {e}")))?;
    let mut cmd = Command::new(exe);
    cmd.arg("daemon")
        .arg("--port")
        .arg(opts.port.to_string())
        .args(&opts.daemon_args)
        .stdin(Stdio::null())
        .stdout(Stdio::null());
    // Keep daemon logs on stderr so `webui` stdout stays URL-only.
    cmd.spawn()
        .map_err(|e| KurultaiError::config(format!("spawn `kurultai daemon`: {e}")))
}

async fn wait_ready(base: &str) -> Result<()> {
    let start = Instant::now();
    while start.elapsed() < READY_TIMEOUT {
        if probe(base).await {
            return Ok(());
        }
        tokio::time::sleep(POLL_EVERY).await;
    }
    Err(KurultaiError::config(format!(
        "daemon did not answer {base}/api/status within {}s — run `kurultai daemon --port {}` and check logs",
        READY_TIMEOUT.as_secs(),
        base.rsplit(':').next().unwrap_or(""),
    )))
}

pub(crate) fn open_browser(url: &str) {
    #[cfg(target_os = "macos")]
    let opener = "open";
    #[cfg(all(unix, not(target_os = "macos")))]
    let opener = "xdg-open";
    #[cfg(windows)]
    let opener = "explorer";
    if let Err(e) = Command::new(opener)
        .arg(url)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        eprintln!("could not open browser ({opener}: {e}) — open {url} manually");
    }
}
