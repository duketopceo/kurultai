//! Chaos tests — failure modes JS harnesses can't reach.
//!
//! Spawns the real `kurultai daemon` subprocess on a scratch store and
//! exercises kill-mid-write recovery and cold-start latency. All hermetic;
//! scratch state lives in tempdirs, daemons get killed at scope end.

use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Stdio};
use std::time::{Duration, Instant};

/// Minimal HTTP/1.1 client — status line + body. The test needs POST + GET
/// over loopback only; pulling in reqwest for that isn't worth a dev-dep.
fn http(port: u16, method: &str, path: &str, body: Option<&str>) -> (u16, String) {
    let mut stream = match TcpStream::connect(("127.0.0.1", port)) {
        Ok(s) => s,
        Err(_) => return (0, String::new()),
    };
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let req = match body {
        Some(b) => format!(
            "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{b}",
            b.len()
        ),
        None => format!("{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"),
    };
    stream.write_all(req.as_bytes()).unwrap();
    let mut buf = Vec::new();
    let _ = stream.read_to_end(&mut buf);
    let text = String::from_utf8_lossy(&buf);
    let status = text
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let body_out = text.split("\r\n\r\n").nth(1).unwrap_or("").to_string();
    (status, body_out)
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn fixture_config(tmp: &tempfile::TempDir) -> PathBuf {
    let vault = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vault");
    let db = tmp.path().join("store.db");
    let cfg = tmp.path().join("config.toml");
    fs::write(
        &cfg,
        format!(
            "environment = \"dev\"\n\n[storage]\npath = \"{db}\"\n\n[embed]\nmodel = \"openai/text-embedding-3-large\"\ndimension = 4\n\n[runtime]\npoll_interval_secs = 300\n\n[sources.notes]\nkind = \"markdown\"\nenabled = false\nroot_path = \"{vault}\"\n",
            db = db.display(),
            vault = vault.display(),
        ),
    )
    .unwrap();
    cfg
}

fn spawn_daemon(cfg: &PathBuf, port: u16) -> Child {
    std::process::Command::new(env!("CARGO_BIN_EXE_kurultai"))
        .arg("--config")
        .arg(cfg)
        .args(["daemon", "--port", &port.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn daemon")
}

fn wait_healthy(port: u16, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let (200, _) = http(port, "GET", "/health", None) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(150));
    }
    false
}

fn kill9(child: &mut Child) {
    // assert_cmd wraps std Child; id() gives the pid for SIGKILL.
    #[cfg(unix)]
    unsafe {
        libc::kill(child.id() as i32, libc::SIGKILL);
    }
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn kill_mid_write_recovers() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = fixture_config(&tmp);
    let port = free_port();

    let mut daemon = spawn_daemon(&cfg, port);
    assert!(wait_healthy(port, Duration::from_secs(15)), "daemon up");

    // Create a thread, then write in a tight loop and SIGKILL mid-stream.
    let (s, _) = http(
        port,
        "POST",
        "/api/hey/threads",
        Some(r#"{"name":"chaos-kill","turn_cap":10000}"#),
    );
    assert_eq!(s, 200, "thread create");
    for i in 0..300 {
        let body = format!(r#"{{"content":"chaos write {i}","instance_id":"chaos"}}"#);
        http(
            port,
            "POST",
            "/api/hey/threads/chaos-kill/messages",
            Some(&body),
        );
        if i == 50 {
            kill9(&mut daemon);
            break;
        }
    }

    // Restart on the same store — WAL must recover, not corrupt.
    let mut daemon2 = spawn_daemon(&cfg, port);
    assert!(
        wait_healthy(port, Duration::from_secs(15)),
        "daemon recovered after SIGKILL mid-write"
    );
    let (s, body) = http(
        port,
        "GET",
        "/api/hey/threads/chaos-kill/messages?limit=500",
        None,
    );
    assert_eq!(s, 200, "messages readable post-kill");
    let landed = body.matches("\"id\"").count();
    assert!(landed > 0, "at least one pre-kill write survived");
    kill9(&mut daemon2);
}

#[test]
fn cold_start_first_requests_bounded() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = fixture_config(&tmp);
    let port = free_port();

    let mut daemon = spawn_daemon(&cfg, port);
    let t = Instant::now();
    assert!(wait_healthy(port, Duration::from_secs(15)), "daemon up");
    let boot_ms = t.elapsed().as_millis();

    let t = Instant::now();
    let (s1, _) = http(port, "GET", "/api/graph", None);
    let graph_ms = t.elapsed().as_millis();
    let t = Instant::now();
    let (s2, _) = http(port, "GET", "/api/search?q=test&limit=20", None);
    let search_ms = t.elapsed().as_millis();
    kill9(&mut daemon);

    assert_eq!(s1, 200);
    assert_eq!(s2, 200);
    // Generous ceilings — the point is a regression tripwire, not microbenches.
    assert!(graph_ms < 5000, "cold /api/graph took {graph_ms}ms");
    assert!(search_ms < 5000, "cold /api/search took {search_ms}ms");
    eprintln!("cold_start: boot={boot_ms}ms graph={graph_ms}ms search={search_ms}ms");
}

/// Disk-pressure: point the store at a tiny tmpfs and confirm writes fail
/// loudly rather than corrupting. Requires a small writable mount; ignored
/// by default — run with `--ignored` on Linux boxes that have /dev/shm.
#[test]
#[ignore]
fn disk_pressure_fails_loudly() {
    // /dev/shm sized mounts aren't portable enough to automate here; this is
    // the manual recipe:
    //   1. `sudo mount -t tmpfs -o size=4M tmpfs /tmp/kurultai-tiny`
    //   2. point [storage].path at /tmp/kurultai-tiny/store.db
    //   3. hammer writes; expect 5xx errors, never silent success
    //   4. unmount; store on the real disk must still open clean
    unimplemented!("manual procedure — see doc comment");
}
