# Differential testing: Rust engine vs Forge

Phase 2 gate work (doc 04, `engine-design/04-differential-testing-spec.md`). Everything here compares the Rust engine in
`rust-engine/` (M3e mirror, `ENGINE_CORE_VERSION = 2`) against Forge (checkout `153b545f`, headless). The engine source was
not modified; the harness lives next to it and uses the engine's `diff-harness` feature accessors.

Results and verdicts: [REPORT.md](REPORT.md) (single-step positions, M3f) and [LOCKSTEP-REPORT.md](LOCKSTEP-REPORT.md) (multi-step lines, core-frozen-m3h; harness in `harness/lines/`, `linegen`/`linediff` binaries, results in `results/lines/`). Known-divergence registry: [known-divergences/](known-divergences/).
Minimal repros: [repros/](repros/). Raw logs and per-position results: [results/](results/).

## What it does

Offline two-phase "lockstep" (a simplification of doc 04 section 5; no live JVM-in-the-loop, no randomness leader/follower):

1. **posgen** (Rust) plays random games between the eight decks and, at sampled priority windows, exports a *position*:
   the state as Forge `GameState` text lines, the canonical action set the engine offers, a canonical summary, and one
   action to play. Three kinds of action: a single priority action, a *combat run* (pass into combat, attack with a random
   subset, Forge's AI chooses blocks, run to main phase 2) and a *stack response* (cast a spell, the opponent answers it
   with a random legal action).
2. **Forge probe** (`harness/forge-probe/`) injects the same text, then either lists the menu Forge offers (menu mode, for the
   action-set comparison) or plays the same action with canonical target and cost choices (step mode). Every decision the
   Forge AI makes is logged by a generated wrapper (`LogAi.java`, derived from `PlayerControllerAi`).
3. **stepdiff** (Rust) replays the game to the position, plays the action and *follows the Forge log* (same cards chosen,
   same targets, same blocks), then compares canonical summaries (life, hand, battlefield, graveyard, exile, library size).
   A mismatch is re-played from a game built out of the injected state text ("reinjection"); if that matches Forge the
   difference came from history Forge cannot be given (durable effects, delayed triggers) and is reported as a
   `history_artifact`, not a divergence.
4. **compare.py** compares action sets and applies the known-divergence registry (suppression is counted per entry).

Statuses in `step-diff.jsonl`: `match`, `history_artifact`, `mismatch` (aligned and still different after reinjection: a real
candidate), `unaligned` (the follower could not mirror a Forge decision, so the comparison is not valid), `forge_error`
(Forge could not play the action: harness or Forge-side), `forge_play_failed`.

## Canonical rules both sides share

Target order: opponent player, own player, then objects by (name, controller); greedy up to the maximum. Cost picks (pitch,
discard, sacrifice, return) take the smallest card name. Modes: first legal. Combat identity is name plus counters
(`Name{P1P1=2}`), and object targets carry `|T` when tapped, so same-named permanents can be told apart.

## Reproducing

```
# engine build with the harness (adjust the path in harness/crates/mtg-diff/Cargo.toml to your engine copy)
cd harness && CARGO_TARGET_DIR=/tmp/target-diff cargo build --release --bins
# Forge probe classes (needs the Forge jar and the interaction-test classes, see interaction-tests/harness)
sh forge-probe/build.sh
# one batch: normal (priority actions + action sets), combat, or response
TARGET=/tmp/target-diff/release OUT=<probe classes> bash tools/run-batch.sh <out dir> <games> <seed0> <sample> [combat|response]
# the 233 interaction scenarios on the engine
/tmp/target-diff/release/scnrun interaction-tests/scenarios
```

Seeds are in REPORT.md; a position id (`<deckA>-<deckB>-s<seed>-d<decision>[-combat|-resp]`) is enough to regenerate it.

## Known-divergence entries

`known-divergences/kd-NNNN.toml` follow doc 04 section 9 with three extra `match` keys the harness uses: `sides`
(`RUST_ONLY`/`FORGE_ONLY`), `zones`, and `actor_seat`. Status is always `proposed` or `scope-cut`: per doc 04 section 6.4 a
FORGE_BUG, VERSION_DIVERGENCE or SPEC_AMBIGUITY verdict needs a human to approve it, so nothing here is `accepted`.

## Layout

- `harness/` Rust crate `mtg-diff` (`posgen`, `stepdiff`, `scnrun`, `explore`), Forge probe sources, `tools/` (compare, run-batch)
- `known-divergences/`, `repros/`, `results/`
