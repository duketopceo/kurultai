//! Device authorization login for the Kurultai CLI.
//!
//! Starts a device flow against a hosted Kurultai instance, opens the approval
//! page in a browser, polls for completion, and stores the resulting agent token
//! in the local keyring (`omaseal`/`oma-ring` when available) or a restrictive
//! `~/.config/kurultai/credentials.toml` file as a fallback.

use crate::error::KurultaiError;
use crate::Result;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct LoginOptions {
    pub base_url: String,
    pub codename: String,
    pub no_browser: bool,
    pub account: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DeviceTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}

#[derive(Debug, Deserialize)]
struct DeviceTokenError {
    error: String,
}

pub async fn run(opts: LoginOptions) -> Result<()> {
    let client = reqwest::Client::new();
    let base = opts.base_url.trim_end_matches('/').to_string();

    let code_resp = client
        .post(format!("{base}/auth/device/code"))
        .json(&serde_json::json!({
            "client_id": "kurultai-cli",
            "codename": &opts.codename,
        }))
        .send()
        .await
        .map_err(|e| KurultaiError::Other(anyhow::anyhow!("device code request failed: {e}")))?
        .json::<DeviceCodeResponse>()
        .await
        .map_err(|e| {
            KurultaiError::Other(anyhow::anyhow!("device code response parse failed: {e}"))
        })?;

    println!("User code: {}", code_resp.user_code);
    println!("Approve at: {}", code_resp.verification_uri);

    if !opts.no_browser {
        open_browser(&code_resp.verification_uri);
    }

    let interval = std::cmp::max(code_resp.interval, 2);
    let deadline = Instant::now() + Duration::from_secs(code_resp.expires_in);

    while Instant::now() < deadline {
        tokio::time::sleep(Duration::from_secs(interval)).await;

        let res = client
            .post(format!("{base}/auth/device/token"))
            .json(&serde_json::json!({
                "grant_type": "urn:ietf:params:oauth:grant-type:device_code",
                "device_code": &code_resp.device_code,
                "client_id": "kurultai-cli",
            }))
            .send()
            .await
            .map_err(|e| KurultaiError::Other(anyhow::anyhow!("token poll failed: {e}")))?;

        if res.status().is_success() {
            let token = res.json::<DeviceTokenResponse>().await.map_err(|e| {
                KurultaiError::Other(anyhow::anyhow!("token response parse failed: {e}"))
            })?;

            let account = opts
                .account
                .unwrap_or_else(|| format!("{}-agent-token", opts.codename));
            save_credential(&account, &token.access_token)?;

            println!("\nLogin succeeded for '{}'.", opts.codename);
            println!("Token saved. Use it as KURULTAI_API_KEY or in an MCP Authorization header.");
            return Ok(());
        }

        if res.status().as_u16() == 400 {
            let text = res.text().await.unwrap_or_default();
            if text.contains("authorization_pending") {
                eprint!(".");
                continue;
            }
            if let Ok(err) = serde_json::from_str::<DeviceTokenError>(&text) {
                return Err(KurultaiError::Other(anyhow::anyhow!(
                    "login failed: {}",
                    err.error
                )));
            }
            return Err(KurultaiError::Other(anyhow::anyhow!(
                "login failed: {text}"
            )));
        }

        return Err(KurultaiError::Other(anyhow::anyhow!(
            "token poll returned {}",
            res.status()
        )));
    }

    Err(KurultaiError::Other(anyhow::anyhow!(
        "device code expired before approval"
    )))
}

fn open_browser(url: &str) {
    let cmd = if cfg!(target_os = "macos") {
        Some(("open", vec![url.to_string()]))
    } else if cfg!(target_os = "linux") {
        Some(("xdg-open", vec![url.to_string()]))
    } else if cfg!(target_os = "windows") {
        Some((
            "cmd",
            vec!["/c".to_string(), "start".to_string(), url.to_string()],
        ))
    } else {
        None
    };

    if let Some((bin, args)) = cmd {
        let _ = Command::new(bin)
            .args(args)
            .stderr(std::process::Stdio::null())
            .spawn();
    }
}

fn binary_available(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .is_ok()
}

fn save_credential(account: &str, token: &str) -> Result<()> {
    // Prefer omaseal / oma-ring when installed locally.
    for bin in ["omaseal", "oma-ring"] {
        if binary_available(bin) {
            let mut child = Command::new(bin)
                .args(["set", "kurultai", account])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::null())
                .spawn()
                .map_err(|e| KurultaiError::Other(anyhow::anyhow!("{bin} spawn: {e}")))?;
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(token.as_bytes());
            }
            let status = child
                .wait()
                .map_err(|e| KurultaiError::Other(anyhow::anyhow!("{bin} wait: {e}")))?;
            if status.success() {
                return Ok(());
            }
        }
    }

    // Fallback: write to a 0600 file in the config directory.
    let dir = dirs::config_dir()
        .ok_or_else(|| KurultaiError::Other(anyhow::anyhow!("cannot find config directory")))?
        .join("kurultai");
    std::fs::create_dir_all(&dir)
        .map_err(|e| KurultaiError::Other(anyhow::anyhow!("create config dir: {e}")))?;
    let path = dir.join("credentials.toml");
    let mut doc = load_credentials(&path)?;
    doc.insert(account.to_string(), toml::Value::String(token.to_string()));
    let content = toml::to_string(&doc)
        .map_err(|e| KurultaiError::Other(anyhow::anyhow!("serialize credentials: {e}")))?;
    std::fs::write(&path, content)
        .map_err(|e| KurultaiError::Other(anyhow::anyhow!("write credentials: {e}")))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&path)
            .map_err(|e| KurultaiError::Other(anyhow::anyhow!("metadata: {e}")))?
            .permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(&path, perms)
            .map_err(|e| KurultaiError::Other(anyhow::anyhow!("set permissions: {e}")))?;
    }

    Ok(())
}

fn load_credentials(path: &PathBuf) -> Result<BTreeMap<String, toml::Value>> {
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let text = std::fs::read_to_string(path)
        .map_err(|e| KurultaiError::Other(anyhow::anyhow!("read credentials: {e}")))?;
    let doc: BTreeMap<String, toml::Value> = toml::from_str(&text)
        .map_err(|e| KurultaiError::Other(anyhow::anyhow!("parse credentials: {e}")))?;
    Ok(doc)
}
