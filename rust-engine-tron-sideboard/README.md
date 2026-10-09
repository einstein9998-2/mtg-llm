# Colorless Tron sideboard for the Rust engine (RFC 0013, not applied to the live engine)

Made 2026-10-09 for Brady's Colorless Tron (`decks/colorless-tron.txt`). Read `0013-tron-sideboard.md` first.

## Files
- `0013-tron-sideboard.md`: the RFC (changes, limits, impact).
- `tron-sideboard-core.patch`: `patch -p1` inside `rust-engine/` (17 files; dry-run applies cleanly to the live tree as of 2026-10-09, which has Tron, Storm, UR, Boros, RFC 0010 and RFC 0011).
- `tron-sideboard.cards.ron`: the card data (already inside the patch, kept here for reading).
- `scenarios/`: 66 spec scenarios by two separate agents (A 32: Eldrazi Confluence and Summon: Bahamut; B 34: Argentum Masticore and Mycosynth Lattice). Notes in `SCENARIO-NOTES-*.md`, extra Oracle text in `oracle-additions-*.json`, rule quotes in `cr-excerpts-tron2-*.md`, the writers' instructions in `SCENARIO-BRIEF.md`.

## Results (scratch copy of the live engine with the patch)
- Tron sideboard scenarios: 66 / 66 pass (0 unsupported). Two engine/adapter gaps found by the writers were fixed (see the RFC and `SCENARIO-NOTES-A.md`).
- Whole visible spec (older 838, Tron 156, Storm 163, Boros 48, UR 54, Orim's Chant 18 and the rest, plus these 66): 1346 / 1346.
- `cargo test --release --workspace`: 85 passed, 1 failed. The failure is `legacy_fuzz::legacy_decks_random_games_hold_invariants`, which asserts eight decks in `rust-engine/decks` and finds nine (`boros-energy-v2.txt`); it fails the same way on the unpatched live engine. Both golden sets (test pool and real pool) replay bit-identically (the first attempt moved the card-database hash because two new enum variants sat mid-enum; both are appended last now).
- Fuzz (`legacyfuzz` over all 14 decks in `/mnt/project-files/decks`, including `colorless-tron.txt` now loading as 60 + 15): 3000 + 12000 + 6000 games, invariants and fork/replay checks at every decision, deep checks every 100: 0 violations, every deck card cast or entered (the four new cards included).
- `depsscan`: 0 unreviewed layer pairs.

## How to apply (when Brady approves)
`cd rust-engine && patch -p1 < ../rust-engine-tron-sideboard/tron-sideboard-core.patch`, rebuild, rerun the spec and the goldens. Applying changes the card database hash and `n_defs`: nets and game records made earlier need the new pool. Scenarios: link `rust-engine-tron-sideboard/scenarios` next to the others when building the combined spec folder (`tools/spec2json.py`).
