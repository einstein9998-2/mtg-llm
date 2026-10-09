# RFC 0015: A named payment source no longer pulls in unnamed lands while floating mana could pay

Status: draft; implemented and tested in a scratch copy of the live engine; independent review pending
Author: Chant and Petal thread (Brady: "fix the tomb bug", 2026-10-09)

## Problem
In the LLM-game harness the player can name lands to pay with (`pay=Name1,Name2`). If the named lands alone cannot pay the whole cost, the old planner still planned from ALL sources with the floating mana removed, so the unnamed lands were tapped for the rest and the pool was left unspent. In ts1 a player floating {G}{G} cast Show and Tell ({2}{U}) naming Tropical Island: the engine tapped Tropical Island for {U} and Ancient Tomb for {C}{C} (2 damage), and the {G}{G} stayed in the pool. This cost 2 life in a game lost at 3 life, and it happened with both the normal option and "pay generic from lands, keep floating mana" because both carried the same named source.

Cause: `plan_cost_with` (legal.rs) built the "hinted" plan from `ms`, the list of every source, instead of only the named ones. The comment already said "plan from the named sources alone, and fall back to the pool only if they cannot pay".

## Change
`mtg-core/src/legal.rs`, `plan_cost_with`: the hinted attempt uses only sources in `pay_hint`. If they cannot pay alone, the normal plan runs (pool first for generic, named sources preferred by `pref`, unnamed only if still needed), so the example now taps Tropical Island for {U} and spends {G}{G} on the generic part. One function, 3 lines.

Ordering: the patch is made on top of RFC 0014 (tron sideboard, #21): the named-source attempt goes through `pay_plan_lat`, so Mycosynth Lattice's "any color" still applies to a named-source plan.

## Impact
- State layout, IR, card database, goldens: none. `ENGINE_CORE_VERSION` unchanged.
- Behavior changes only when a payment source is named (the harness's `pay=` and spec scenarios with `pay: {tap: [...]}`), and the named sources cannot pay the full cost alone. Automatic payment without a named source is unchanged.
- Not changed, and a separate question: automatic payment itself never avoids Ancient Tomb's life loss. It orders sources by `(pref, number of colors, tag)`, and Tomb has the same `pref` as any nonbasic land, so it is tapped for generic after basics but before other nonbasics with the same color count. Say so if it should be ranked last.

## Test results (scratch copy of live + this patch)
- New scenario `ancient-tomb-named-land-floating-mana.yaml` fails on the untouched live engine (both Tombs tapped, so the 2 damage was dealt) and passes with the patch (Tomb untapped, life 20).
- Visible spec set 838/838 pass.
- `cargo test --workspace --release`: all pass except `legacy_decks_random_games_hold_invariants` (expects 8 decks, finds 9), which fails identically on the untouched live engine.
