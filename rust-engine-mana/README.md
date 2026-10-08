# RFC 0011 package: pay generic from lands, keep floating mana

`0011-pay-lands-first.md` is the RFC, `0011-pay-lands-first.patch` the change to `crates/mtg-core/src/mana.rs` (applied to the live engine in the shared folder on 2026-10-08).

The menu option that uses it ("pay generic from lands, keep floating mana") lives in the game harness `llmgame.rs` (`match-logs/tooling/` in the shared folder, written by the match thread); it calls `mtg_core::mana::set_lands_first`. That harness copy also carries other match-thread changes, so it is not part of this package.
