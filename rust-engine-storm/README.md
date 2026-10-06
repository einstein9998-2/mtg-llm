# Legacy Storm for the Rust engine (RFC 0007, not applied to the live engine)

APPLIED to the live engine 2026-10-06 (storm-on-tron.patch, after Tron; MILESTONES M3n). The text below was written before that.

Made 2026-10-06 on top of the live `/mnt/project-files/rust-engine` (core-frozen-m5 plus RFCs 0005 and 0006). The live engine is NOT changed. Read `0007-storm.md` first.

## Files
- `0007-storm.md`: the RFC (changes, rules basis, impact, documented limits, results, the three scenario defects).
- `storm-core.patch`: `patch -p1` inside `rust-engine/`. 25 files: `mtg-core` (Wish, imprint, Sagas, suspend, free casts, ward as data, Gaea's Will, Song of Creation, storm copies of a countered spell, activation condition), `mtg-cards` (`storm.cards.ron` as a second source in `legacy.rs`, a load test), `mtg-spec` (test adapter), `mtg-view` (one match arm). Dry-run applies cleanly to the live tree as of 2026-10-06 05:45Z.
- `storm.cards.ron`: the 19 definitions (17 cards, Taiga, the Construct token), already inside the patch, kept here for reading.
- `spec-validate-suspend.patch`: one word added to `rust-engine-spec/tools/validate.py` (`suspend` as an action name); needed to validate the Gaea's Will scenarios.
- `scenarios/`: 163 spec scenarios (A 63 Beseech, Gamble, Wish, Tendrils, Empty the Warrens, Peer into the Abyss; B 58 Chrome Mox, Mox Opal, Giant's Boulder, Urza's Saga, Haywire Mite, Taiga, Runehorn Hellkite; C 42 Gaea's Will, Hexing Squelcher, Song of Creation). Written by three separate agents from Oracle text and the CR, not by the implementer; none was edited. Notes in `SCENARIO-NOTES-*.md`, extra Oracle text in `oracle-additions-*.json`, rule quotes in `cr-excerpts-storm-*.md`, the writers' instructions in `SCENARIO-BRIEF.md`.

## Results (scratch copy with the patch)
- Storm scenarios: 162 pass, 1 unsupported (`storm-b-chrome-mox-imprint-colorless-card-no-mana` needs Kozilek's Command from RFC 0008). Three scenarios that were wrong by the CR (found by the implementer, confirmed by the independent review) were fixed by a separate agent; before that: 159 pass, 3 fail.
- Whole spec (1001 scenarios): 1000 pass, 1 unsupported. Of these the 838 older ones are unchanged (adapter changes did not move any older scenario).
- `cargo test --release --workspace --no-fail-fast`: green, including the test-pool and real-pool goldens bit-identical (hash additions are conditional), so no `ENGINE_CORE_VERSION` bump.
- Fuzz: the eight decks plus Storm, 3000 random games, invariants at every decision (check every 1, deep fork/replay checks every 100): 0 violations, every card of the nine decks cast or played at least once, 518 Wish choices and 249 copy-target prompts exercised. (The fuzz plays main decks only; sideboarded games and Wish from a real sideboard are covered by the scenarios.)
- The patch was also applied to a fresh copy of the live tree, rebuilt and re-run (997 / 1001 scenarios, `mtg-match` tests green).

## How to apply (when Brady approves)
`cd rust-engine && patch -p1 < ../rust-engine-storm/storm-core.patch`, rebuild, rerun the spec and the goldens. The card database hash and `n_defs` change (cards were added), so nets trained earlier need the new pool and game records from older versions must be regenerated. Put `scenarios/*.yaml` under `rust-engine-spec/scenarios/` (for example `cards/storm/`) and merge `oracle-additions-*.json` and the CR excerpts into the spec reference before running `spec2json.py` and `specrun`.

## Together with Tron (RFC 0008)
Brady chose Tron first (2026-10-06). `storm-on-tron.patch` is Storm as ONE patch on top of the live engine with the Tron patch applied (prepared 2026-10-06 against `rust-engine-tron/tron-core.patch` as of 12:30Z, sha256 prefix `b1453ef20e91f73c`). Use it INSTEAD of `storm-core.patch` once Tron is in the live engine: `cd rust-engine && patch -p1 < ../rust-engine-storm/storm-on-tron.patch`. If the live engine has no Tron, use `storm-core.patch` as above.

How it was made: live + `tron-core.patch` = `base`; `storm-core.patch` applied to a copy; the 17 rejected hunks (9 files: `legacy.rs`, `card.rs`, `decision.rs`, `ir.rs`, `legal.rs`, `lint.rs`, `types.rs`, `expect.rs`, `runner.rs`) merged by the script in `merge-onto-tron/` (re-runnable if the Tron patch changes); the patch is `diff -ruN base combo`. Most rejects are append points. Not trivial: `legal.rs` (mana sources and explicit mana options, where Tron changed the same lines for Urza's Workshop and the Planar Nexus filter), and three decisions: one `Saga` subtype (Tron's), `Lesson` and `Sorcerer` after Tron's names; the Construct token is defined only in `tron.cards.ron` (identical); Giant's Boulder ("{1},{T}: any color") now uses Tron's filter-mana mechanism instead of the pool-paid code of the standalone patch (that code is dropped in the combined patch).

Results (combined engine): whole spec 1157 of 1157 pass (838 older, 156 Tron, 163 Storm, including the Chrome Mox colorless-imprint scenario that needed Kozilek's Command); `cargo test --release --workspace`: 83 passed, 0 failed, both golden sets bit-identical; fuzz: the ten decks (the eight, Storm and Tron; Tron without the four cards neither patch defines), 3000 random games, invariants at every decision and deep fork/replay checks every 100: 0 violations, every card of the decks cast or played at least once (440 Wish choices, 179 copy-target prompts).

Earlier this README said "14 hunks in 8 files, all at append points, no logic conflicts". That was wrong: GNU patch's "reversed patch detected" had hidden the ten `legal.rs` hunks. The real count is 17 hunks in 9 files, with a real merge in `legal.rs`.

Still open for the Tron deck: "Summon: Bahamut" (a Saga creature) is not defined by either patch; Urza's Saga now works on the combined engine, so it can be added with its own scenarios.

## Not in the engine (documented limits)
See the RFC: suspend's free cast cannot be declined; Urza's Saga's gained abilities are marker counters; Hexing Squelcher's ward is two triggers; suspend shows up as an activation.
