# Named payment sources and floating mana (RFC 0014)

Made 2026-10-09 for Brady's "fix the tomb bug" (Ancient Tomb tapped for generic while {G}{G} floated, ts1 of the Tail Swipe games).

## Files
- `0014-named-payment-keeps-floating-mana.md`: the RFC (cause, change, impact, tests).
- `paypick-core.patch`: `patch -p1` inside `rust-engine/` (one hunk in `crates/mtg-core/src/legal.rs`). Dry-run clean against the live `/mnt/project-files/rust-engine` and the repo's `rust-engine/`.
- `scenarios/cards/ancient-tomb-named-land-floating-mana.yaml`: the repro, written by the implementer (not a separate spec writer).

## Status
Not applied to the live engine by this PR.
