//! The pool must contain no layered-effect pair that timestamp-only evaluation could get wrong
//! without a recorded review (doc 01 section 10.4).
#[test]
fn every_flagged_layer_pair_is_reviewed() {
    let decks = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../decks");
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_depsscan")).arg(decks).output().expect("run depsscan");
    assert!(out.status.success(), "depsscan failed:\n{}", String::from_utf8_lossy(&out.stdout));
}
