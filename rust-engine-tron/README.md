# Colorless Tron for the Rust engine (RFC 0008, not applied to the live engine)

Made 2026-10-06 on top of the live `/mnt/project-files/rust-engine` (core-frozen-m5 plus RFCs 0005 and 0006). The live engine is NOT changed. Read `0008-colorless-tron.md` first.

## Files
- `0008-colorless-tron.md`: the RFC (changes, rules basis, impact, documented limits, results).
- `tron-core.patch`: `patch -p1` inside `rust-engine/`. 22 files: `mtg-core` (cost floor, no-untap, unblockable, protection from everything, static restrictions, new effects, filter-land payment, subtypes), `mtg-cards` (`tron.cards.ron`, second source in `legacy.rs`, load test, one `deps-reviewed.txt` line), `mtg-spec` (test adapter). Dry-run applies cleanly to the live tree.
- `tron.cards.ron`: the card data (28 top-level definitions: cards, tokens and Tezzeret's emblem); already inside the patch, kept here for reading.
- `scenarios/`: 156 spec scenarios (A 42: Trinisphere, Monolith, Ring, Keys, Karn; B 51: Hole, Desk, Orb, Bridge and others; C 63: lands, Kozilek's Command, Tezzeret, Ugin). Written by three separate agents from Oracle text and the CR, not by the implementer. Notes in `SCENARIO-NOTES-*.md`, extra Oracle text in `oracle-additions-*.json`, rule quotes in `cr-excerpts-tron-*.md`, the writers' instructions in `SCENARIO-BRIEF.md`.

## Results (scratch copy with the patch)
- Tron scenarios: 156 / 156 pass (0 unsupported).
- Visible spec: 838 / 838 (adapter changes did not move any older scenario).
- `cargo test --release --workspace --no-fail-fast`: 81 passed, 0 failed (includes the test-pool and real-pool goldens, bit-identical, so no `ENGINE_CORE_VERSION` bump is needed).
- 9-deck random fuzz (the eight decks plus Tron), 3000 games, invariants and fork/replay checks at every decision, deep checks every 100: 0 violations. The fuzz found two real bugs in Kozilek's Command (a mode offered when no X leaves a target, and "up to X" targets demanding X cards); both fixed and covered by the rerun.

## How to apply (when Brady approves)
`cd rust-engine && patch -p1 < ../rust-engine-tron/tron-core.patch`, rebuild, rerun the spec and the goldens. Applying changes the card database hash and `n_defs` (card pool grew), so nets trained earlier need the new pool and game records from older versions must be regenerated.

## Not in the engine (documented limits)
Urza's Saga and Summon: Bahamut (RFC 0007, Storm), Karn -2 (Wish), Ugin -11, Argentum Masticore, Mycosynth Lattice, Eldrazi Confluence (distinct modes only), Extinguisher Battleship station, Mishra's Research Desk unearth and end-of-next-turn expiry, Kozilek's Command player modes only target you, Planar Nexus filter only through automatic payment. Until 0007 lands, Tron's 4 Urza's Saga cannot be cast; the fuzz deck lists omit the four absent cards.
Loyalty: Tezzeret 4 and Ugin 7 confirmed from scrydex.com; Karn's 5 is not confirmed.
