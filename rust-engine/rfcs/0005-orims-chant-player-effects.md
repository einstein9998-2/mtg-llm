# RFC 0005: Orim's Chant ("can't cast spells this turn", "creatures can't attack this turn", kicker)

Status: applied to the live engine 2026-10-06 at Brady's request ("do both"); independent reviewers still pending
Author: Orim's Chant thread  Reviewers (two independent, one a rules reviewer): pending

## Problem
Brady's Alurentell 75 (2026-10-06) has 2x Orim's Chant in the sideboard. The pool has no card that stops a player from casting spells for one turn, and none that stops all attacks for one turn:
- `Restriction::CantCast` exists only as a static ability of a permanent (Deafening Silence, Voice of Victory). A resolved spell has no way to leave a turn-long "can't cast" behind.
- `ContEffect::CantAttack` applies to a fixed set of objects at resolution, so it would miss a creature that enters later in the turn (Chant says "creatures can't attack this turn", which covers them).
- Kicker has no representation. `AddCost(optional: true)` lets the caster choose an extra cost, but the choice is not kept anywhere a resolving spell can read ("if this spell was kicked").

Oracle text (Scryfall data via Savecraft, 2026-10-06): "Kicker {W} (You may pay an additional {W} as you cast this spell.) Target player can't cast spells this turn. If this spell was kicked, creatures can't attack this turn." Instant, {W}.

Savannah, the other card in the request, was already in the pool (Boros Aggro runs it), so it needs nothing.

## Proposed change
`mtg-core` (IR enum cases only, plus two read sites):
1. `PlayerFx::CantCast`, new case appended to the enum (existing cases keep their discriminants). `Effect::PlayerFx { who, fx: CantCast, until: EndOfTurn }` records it in `State::player_fx` as the existing `SpellsUncounterable` and `HexproofFrom` do. `Cx::cast_restricted_x` (`restrict.rs`) returns true first when the seat has the effect and the spell is not already on the stack (`x` is `None`). That function is already the single gate for the enumerator (`priority.rs`, including alternative-cost and virtual ways such as Force of Will's pitch and Aluren/Omniscience permits) and for every effect-driven cast (`resolve.rs`: Bilbo, energy casts, miracle, free casts), so all of them are blocked. Activated abilities, special actions and land plays are untouched, as the rules require.
2. `PlayerFx::CreaturesCantAttack`, new case appended. `Cx::attack_candidates` (`combat.rs`) returns no attackers for a seat that has it, so there is no declaration decision at all (consistent with "no decision when the rules give no choice"). Kicked Chant applies it to both players. Because it is read when attackers are declared, it also covers creatures that enter after Chant resolves. It does not affect creatures put onto the battlefield attacking (CR 508.4; not exercised by any scenario, no pool card does this on the Chant turn).
3. Kicker is NOT added to the core. Card data models it as an alternative way to cast the spell: `Alt(label: "kicked", key: "kicked", cost: [Mana("{W}{W}")])`, and the kicked branch reads `Cond::WasCast(key: Some("kicked"))`, an existing condition. Payment is identical to paying {W} plus the kicker {W}, and the enumerator offers both casts when {W}{W} is available and only the unkicked one when it is not.
4. `Cond::WasCast` (`eval.rs`) now also works for a spell that is still on the stack: it reads `from` and `way` from the stack entry's `CastInfo` (already stored there) instead of from the object, which only records them when a permanent enters. Before this, the condition was always false for an instant or sorcery. No state change; behavior changes only for conditions evaluated on a resolving spell, and no existing card does that (the visible spec set is unchanged at 806/806).
5. Test adapter only (`mtg-spec`, not core): `kicked: true|false` on a cast action and on `legal` patterns maps to the "kicked" way (a spelling the test writer chose; SCHEMA.md has no kicker syntax yet).

Card data: `Orim's Chant` in `legacy.cards.ron` (white section).

## Rules basis
- CR 101.2 ("can't" beats "can") and 601.2a-b (a spell that can't be cast isn't offered): checked against the CR excerpts pinned in `rust-engine-spec/reference/cr-excerpts.md` and the Savecraft `rules_search` results used by the test writer (see `rust-engine-spec/scenarios/cards/orims-chant-*.yaml` NOTES for the cited rules).
- CR 702.33 (kicker) is the one deviation: the engine treats the kicked cast as an alternative cost. A rules-faithful kicker would pay the base cost plus the kicker as an additional cost, so the only observable differences are (a) a free cast of Chant (Omniscience, Aluren-style permits, Force-of-Will-like alternative costs) cannot also be kicked, and (b) effects that care about alternative versus additional costs. Neither occurs in the closed pool except (a) through Omniscience, which would let the Alurentell player cast Chant for free and then pay {W} to kick it. That line is unavailable in this model (inferred from how the engine's cast model works, not from a ruling).
- Chant on a spell already on the stack: a spell cast before Chant resolved stays legal (it was cast legally), so the check applies only at announcement.

## Impact
- State layout and `hash_rules` / `hash_full`: none. `player_fx` is already in both hashes; `PlayerFx` gains two cases at the end. `CastInfo` is untouched (this is why kicker is not a core change).
- Determinism and golden replays: no golden changes (the test-pool and real-pool golden suites pass unchanged, see Test plan results).
- Hidden information: the effects are public (they sit in `player_fx` as the existing player effects do); no new view or fork state.
- Effect VM / IR: two new enum cases on `PlayerFx` (core-adjacent, doc 01 section 14.1). No new `Effect`, `Cond` or `Expr` case.
- `ENGINE_CORE_VERSION` bump: no, since goldens replay bit-identically (a behavior change only for Chant, which no existing record contains). The card database hash changes because a card was added, as for any card addition.

## Alternative considered
Faithful kicker: record the chosen additional cost on `CastInfo` (or `Object::cast_way`), add `Cond::Kicked`, and keep `AddCost(optional)` for the cost. This is correct for free casts, but it changes `CastInfo`/object layout, so the state hash changes at every position with a spell on the stack, all goldens must be regenerated and every older record is invalid. Rejected for one card whose only observable difference is the Omniscience free-cast line. If Brady wants that line, this is the follow-up RFC.

## Test plan
- Scenario tests written by a separate agent from the card text and CR only (the implementer did not write them): `rust-engine-spec/scenarios/cards/orims-chant-*.yaml`, run with `spec2json.py` and `specrun`.
- Visible spec set (803) must still pass; own scenarios; `cargo test --workspace --release`; goldens (test pool and real pool) unchanged; `legacyfuzz` with Chant in the sideboard-aware decks (see MILESTONES.md for the run).
- Known not covered: sideboarded games with Chant in the 8-deck fuzz (the fuzz plays main decks only), and the Omniscience free-cast kicker line (unavailable by design, above).
