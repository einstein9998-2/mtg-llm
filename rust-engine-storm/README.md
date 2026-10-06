# Legacy Storm for the Rust engine (RFC 0007, not applied to the live engine)

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
The two patches both append to the same enums and match lists, so applying the second one by hand needs a merge: dry-run of this patch on top of `tron-core.patch` rejects 14 hunks in 8 files (`legacy.rs`, `card.rs`, `decision.rs`, `ir.rs`, `lint.rs`, `types.rs`, `expect.rs`, `runner.rs`), all at append points. No logic conflicts were seen. Keep one `Saga` subtype and put `Lesson`, `Sorcerer` after Tron's names. A combined patch with a joint test run is a follow-up once the order is chosen. Until then Tron's 4 Urza's Saga cannot be cast on a Tron-only engine, and Storm's Kozilek's Command scenario is unsupported on a Storm-only engine.

## Not in the engine (documented limits)
See the RFC: suspend's free cast cannot be declined; Urza's Saga's gained abilities are marker counters; Hexing Squelcher's ward is two triggers; suspend shows up as an activation.
