# Boros Energy for the Rust engine (RFC 0012, APPLIED to the live engine)

Made 2026-10-08 for Brady's Boros Energy 75 (`decks/boros-energy-v2.txt`). Read `0012-boros-energy.md` first.

## Files
- `0012-boros-energy.md`: the RFC (changes, limits, rules basis, impact, results).
- `boros-core.patch`: `patch -p1` inside `rust-engine/` (cards file, `legacy.rs` registration, test-only spec adapter change).
- `boros.cards.ron`: the card data, already inside the patch, kept here for reading.
- `scenarios/`: 48 spec scenarios by two separate agents (A 24: Static Prison, Sunbaked Canyon, Samut; B 24: Forth Eorlingas!, Mindbreak Trap). Notes in `SCENARIO-NOTES-*.md`, extra Oracle text in `oracle-additions-*.json`, rule quotes in `cr-excerpts-boros-*.md`, the writers' instructions in `SCENARIO-BRIEF.md`.

## Applied
Applied to `/mnt/project-files/rust-engine` on 2026-10-08. The card database hash and card count changed: nets and game records made against the previous live pool must be regenerated. Goldens replay bit-identically; `ENGINE_CORE_VERSION` is unchanged.

## Limits
No monarch (Forth Eorlingas! makes tokens only); Mindbreak Trap targets up to four spells; Samut text from web sources. See the RFC.
