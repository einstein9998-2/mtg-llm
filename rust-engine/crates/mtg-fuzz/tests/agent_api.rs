//! The agent-facing API of `mtg-view` (doc 02 section 3): built without `diff-harness`, raw
//! engine access must not compile, and the seat-bound handle must. This builds two tiny crates
//! outside the workspace (so feature unification from the harness crates cannot leak in).

use std::path::PathBuf;
use std::process::Command;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn check(name: &str, body: &str) -> (bool, String) {
    // One build at a time: the probes share a target directory.
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let ws = root();
    let dir = std::env::temp_dir().join(format!("mtg-agent-api-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        format!(
            "[package]\nname = \"agent-probe\"\nversion = \"0.0.0\"\nedition = \"2021\"\n[workspace]\n[dependencies]\nmtg-view = {{ path = \"{}/crates/mtg-view\" }}\nmtg-core = {{ path = \"{}/crates/mtg-core\" }}\n",
            ws.display(),
            ws.display()
        ),
    )
    .unwrap();
    // Reuse the workspace lockfile so the offline build resolves the same dependency versions.
    std::fs::copy(ws.join("Cargo.lock"), dir.join("Cargo.lock")).unwrap();
    std::fs::write(dir.join("src/main.rs"), format!("#![allow(unused)]\nuse mtg_view::*;\nfn main() {{}}\n{body}\n")).unwrap();
    let target = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| ws.join("target").display().to_string());
    let out = Command::new(env!("CARGO"))
        .args(["check", "--offline", "--manifest-path"])
        .arg(dir.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", format!("{target}/agent-api-check"))
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stderr).to_string();
    (out.status.success(), text)
}

#[test]
fn seat_bound_handle_compiles_without_the_harness_feature() {
    let (ok, text) = check("ok", "fn f(g: &Game, m: &dyn BeliefModel) { let v = g.seat_view(mtg_core::ids::Seat(0)); let _ = v.observe(); let _ = v.decision(); let _ = v.fork(1, m); let _ = g.result(); }");
    assert!(ok, "{text}");
}

#[test]
fn raw_engine_access_does_not_compile_without_the_harness_feature() {
    for (name, body, what) in [
        ("pending", "fn f(g: &Game) { let _ = g.pending(); }", "pending"),
        ("observe", "fn f(g: &Game) { let _ = g.observe(mtg_core::ids::Seat(1)); }", "observe"),
        ("hash", "fn f(g: &Game) { let _ = g.state_hash(); }", "state_hash"),
        ("events", "fn f(g: &mut Game) { let _ = g.take_events(); }", "take_events"),
        ("raw", "fn f(g: &Game) { let _ = g.raw_state(); }", "raw_state"),
    ] {
        let (ok, text) = check(name, body);
        assert!(!ok && text.contains(&format!("no method named `{what}`")), "{what} must not be reachable by an agent crate: {text}");
    }
}
