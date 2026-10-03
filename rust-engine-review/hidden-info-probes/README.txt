Probes for hidden-info-review-2026-10-01.md. Not part of the project; written against a COPY of the workspace.

Setup (never touch /home/claude/rust-engine):
  cp -r /home/claude/rust-engine /tmp/x && rm -rf /tmp/x/target
  cd /tmp/x && patch -p0 < read-probe-twin.patch          # adds State::review_shuffle_unknown_order (diff-harness only), paths are relative: apply by hand if needed
  cp review_hidden.rs crates/mtg-fuzz/tests/
  patch crates/mtg-debug/src/noninterference.rs < noninterference-harness-triage.patch   # optional, for the real-pool NI triage
  export CARGO_TARGET_DIR=/home/claude/target-review-hidden
  cargo test --offline --release -p mtg-fuzz --test review_hidden -- --nocapture --test-threads=1 <test name>

Tests (original code, numbers quoted in the report):
  poc_fork_decodes_opponent_hand        51/51 states: exact opponent hand recovered from forks
  poc_fork_decodes_library_order        3549/3549 library positions (69 states) recovered, top five 345/345
  poc_forks_differ_for_library_order_twins   119/119 observer-indistinguishable twins give different forks (needs read-probe-twin.patch)
  poc_raw_pending_exposes_slots         17/17 opponent hand cards identified from Game::pending() slots + decklist order
  poc_decision_id_leaks_opponent_scry_choice  6/30 opponent scries: observer's next DecisionId differs with the opponent's hidden choice
  poc_show_and_tell_fork_depends_on_true_pick P(fork shows opp entering something | truth nothing)=0/5256, | truth something = 6294/9336
  poc_oracle_random_bottom_knowledge    27/286 Thassa's Oracle resolutions leave the owner a "known" bottom run (stale pos_known_to)
  poc_fork_keeps_opponent_private_knowledge  opponent-only knowledge bits survive the fork (counts)
  ni_on_real_pool                       the project's own NI harness on the 8 real decks (env NI_ONLY=seed,observer,own to isolate; REVIEW_* env vars need the harness patch)
  poc_miracle_forced_reveal             Triumph of Saint Katherine (miracle) forced reveal; also hits an engine panic (see report, incidental)

fork-position-dealing-prototype.patch: prototype fix for finding 1 (deal sampled identities by position within zone, re-place known-identity
library cards at random free positions). With it: library decode 0/3549, hand decode 12/197 (random-guess baseline 46), sampled library def order
identical in 119/119 twins, and the six tests in crates/mtg-fuzz/tests/fork.rs still pass. It does NOT fix findings 2-4.
