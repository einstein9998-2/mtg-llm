# Named payment sources and floating mana (RFC 0015)

Made 2026-10-09 for Brady's "fix the tomb bug" (Ancient Tomb tapped for generic while {G}{G} floated, ts1 of the Tail Swipe games).

## Files
- `0015-named-payment-keeps-floating-mana.md`: the RFC (cause, change, impact, tests).
- `paypick-core.patch`: `patch -p1` inside `rust-engine/` (one hunk in `crates/mtg-core/src/legal.rs`). Apply it after the tron-sideboard package (#21): hunk 2 edits the line that package changed to `pay_plan_lat` (Mycosynth Lattice). It does not apply to the live engine without #21's legal.rs changes. Checked by applying #21's `legal.rs` hunks to the untouched live `legal.rs` and then this patch (clean, offset 3).
- `scenarios/cards/ancient-tomb-named-land-floating-mana.yaml`: the repro, written by the implementer (not a separate spec writer).

## Status
Not applied to the live engine by this PR.
