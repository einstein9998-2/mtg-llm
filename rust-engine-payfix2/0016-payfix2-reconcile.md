# RFC 0016: Bring main in line with the engine the Tron and Alurentell matches run on (payfix2)

Status: draft; implemented and tested in a scratch copy of live + RFC 0013, 0014 and 0015; independent review pending
Author: Complete the Tron sideboard thread (Brady agreed on 2026-10-10 to a follow-up PR). The three engine changes below were made by the "Match game log for review" thread; this RFC takes them into the reviewed stack.

## Problem
The LLM match builds (`llmgame-*-payfix2`) run on an engine that has three changes no RFC on main contains, and one of them edits the same lines as RFC 0015:

1. **`pay=` with floating mana.** RFC 0015 (merged, #22) and the match thread's `pay-named-sources-then-pool.patch` fix the same bug (named lands plus floating mana tapped an unnamed Ancient Tomb) in the same few lines of `plan_cost_with` and do it differently.
2. **Sunbaked Canyon was not counted as a mana source.** Its cost was `[TapSelf, PayLife(1)]`; the planner only treats a land as a source when the cost is `TapSelf` or `SacrificeSelf`, so a cheap spell was not offered with Canyon as the only red or white source (`sunbaked-canyon-mana-ability.patch`).
3. **Painless lands first.** Automatic payment prefers sources whose mana ability has no side effect (Ancient Tomb's damage, now Canyon's life loss). This was in the match thread's tree before payfix2 and in no RFC. RFC 0015 listed it as an open question ("automatic payment itself never avoids Ancient Tomb's life loss").

A Tron match on main plus the Tron sideboard would otherwise behave differently from the same match on payfix2.

## Change
`mtg-core/src/legal.rs`, `plan_cost_with`:
- Named payment sources (`pay_hint`): plan in this order. (1) the named sources alone with only restricted mana (a fully named payment still leaves the pool, the old `own-named-land-is-tapped-before-floating-mana` rule); (2) the named sources plus the whole pool; (3) the default plan (pool first, every source, named ones preferred). An unnamed source is never tapped while the pool could pay instead. Both plans go through `pay_plan_lat`, so Mycosynth Lattice still applies. This replaces RFC 0015's single named-only attempt; its scenario still passes.
- Source preference: a source whose mana ability is a `Seq` with `Damage` or `LoseLife` gets `+6` in `pref`, so it is tapped after painless sources of the same kind.

`mtg-cards/cards/boros.cards.ron`: both Sunbaked Canyon mana abilities now cost `[TapSelf]` with effect `Seq([AddMana(color, 1), LoseLife(You, 1)])`, the way Ancient Tomb is modelled. The life is lost when the mana ability resolves instead of as a separate cost step; the amount and the timing (no stack) are the same.

`mtg-spec/src/runner.rs`, `continues_mana_group`: the spec runner counts "{T}, Pay 1 life: Add {R}" and "...Add {W}" as one Oracle ability (RFC 0012). It recognised the pair by an identical cost of more than one item; Canyon's cost is now a bare `{T}`, so four of the seven independent Canyon scenarios (`rust-engine-boros`) failed on the match thread's tree. A pair of mana abilities with an identical cost whose effect is a `Seq` now also groups. Test runner only; no engine behavior.

`own-scenarios/scenarios/own.yaml`: the seven scenarios from the match thread (four for `pay=` with floating mana, three for Canyon), also in `scenarios/`.

## Impact
- The card database hash changes (Canyon text); `n_defs` does not. Nets and game records made before need regenerating. State layout, IR, `ENGINE_CORE_VERSION` and `HASH_SCHEMA`: unchanged.
- Automatic payment changes whenever Ancient Tomb or Sunbaked Canyon competes with another source. Two real-pool goldens depended on the old order and are re-recorded: `r00-alurentell-vs-dimir-tempo.rec` (first 369 of 844 actions unchanged, 870 after) and `r07-dimir-tempo-vs-alurentell.rec` (first 326 of 472 actions unchanged, 560 after). Same decks, seed, pinned card snapshot, engine and hash headers; only the action list and checkpoints are new. The other 14 replay bit-identically. Without the painless-land preference both replay unchanged.
- Replays of old Canyon or `pay=`-with-floating-mana games diverge; keep the `-set4` builds for them.

## Tests
Scratch copy of the live engine + RFC 0013, 0014 and 0015 + this patch and the two re-recorded goldens: spec, all packages together, 1352 of 1352; own scenarios 39 of 39; `cargo test --workspace --release` all pass except `legacy_decks_random_games_hold_invariants` (expects 8 decks, finds 9, fails identically without this patch); 6000-game Legacy fuzz over the 9 decks, 0 violations, every deck card cast or entered. The Ancient Tomb scenario from RFC 0015 passes unchanged.
