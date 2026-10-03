Reviewer probes for core-frozen-m5 (hidden information and determinism), 2026-10-02.
Nothing here is part of the engine; they ran on a scratch copy of the mirror (cp -r of rust-engine, release build,
pinned rustc 1.97.0), the mirror itself was not touched.

Copy the four files into crates/mtg-fuzz/tests/ of a workspace copy, then:
  cargo test --release -p mtg-fuzz --test m5_probe -- --nocapture      # Quarry-focused non-interference (Q_GAMES=300)
  cargo test --release -p mtg-fuzz --test det_probe -- --nocapture     # determinism digest (DET_GAMES=300, DET_FROM=0)
  cargo test --release -p mtg-fuzz --test st_probe -- --nocapture      # Show and Tell fork rate stratified by opponent hand size (ST_GAMES=2500)

ni_copy.rs is mtg-debug/src/noninterference.rs with two additions: raw events are kept and the Lazotep Quarry taps
(and Quarry taps in a step where a permanent also went battlefield -> graveyard) are counted in two atomics. The
comparison logic is unchanged. m5_probe.rs includes it with #[path].

Results (release build): see "Review round M5" in rust-engine/CORE-FREEZE-REVIEW.md.
