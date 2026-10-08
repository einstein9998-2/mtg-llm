# RFC 0011: pay generic from lands, keep floating mana

Status: applied to the live engine 2026-10-08 (Brady: it matters a lot in the Alurentell matchup because Carpet of Flowers makes floating mana); patch written by the match thread, reviewed and tested here
Author: match thread (patch), "Alternate UR Cutter list" thread (review, tests)

## Problem
When mana floats in the pool and a spell costs generic mana, `plan_payment` spends the pool first. With Carpet of Flowers (black mana for the whole main phase) the player often wants to pay generic costs from lands and keep the floating mana for later spells, for example three Acererak casts from Carpet mana (CR 601.2h: the player chooses how to pay).

## Change
`mtg-core/src/mana.rs` only:
- `plan_payment` now calls `plan_payment_with(.., lands_first)`, where `lands_first` is a process-wide flag (`set_lands_first(bool)`, off by default).
- With `lands_first` the colored part of a cost still uses floating mana, and the generic part is paid from sources first. If the sources alone cannot pay, the default plan (pool first) is used, so a cast that was payable stays payable.
- `plan_payment_inner(.., pool_generic)` is the old body; with `pool_generic == true` it is unchanged line for line, so the default policy, hashes and goldens are untouched.
- Two unit tests call `plan_payment_with` directly (no global is touched in tests).

The flag is a harness hook: the game harness (`llmgame.rs`, match thread) offers each castable spell an extra option "(pay generic from lands, keep floating mana)" when mana floats, turns the flag on for that cast, and records the choice in the action list as `idx + 1000` so a replay sets the flag the same way. Nothing in the engine, the fuzzers or the tests sets it.

## Limits and risks
- The flag is process-wide, not game state: it is not hashed and not in `GameRecord`. A recorded game that used the option replays correctly only through the harness (which reads the `+1000` marker), not through plain `replay`. Games that never use it are unaffected.
- A harness that runs several games on threads in one process would share the flag; `llmgame` runs one game per process.
- A cleaner design would make it a per-cast choice carried in the decision. That is a larger change (decision shape, hashes, records) and was not needed for the harness.

## Tests
- Unit: default spends the pool first; lands-first taps the sources and keeps the pool; colored pip still paid from the pool; falls back when sources are short; unpayable stays unpayable.
- Spec (all earlier scenarios), workspace tests and real/test goldens unchanged (flag off).
- Fuzz of the Legacy pool with the flag off (as before) and with the flag forced on for every cast (temporary local edit, not committed): invariants and replay/fork checks hold.
