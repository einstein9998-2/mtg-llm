# RFC 0002: Last-known-information filters test subtypes and the other characteristics (E7); several Quarries in the payment planner (E6)

Status: in review (gates green; rules review by the spec thread requested)
Author: Rust engine core thread  Reviewers (two independent, one a rules reviewer): spec thread (rules), hidden-information reviewer (not needed, see Impact)

## Problem
E7 (lockstep differential, repros/e7-ajani-pariah-triggers-on-non-cat.scn): Ajani, Nacatl Pariah's "whenever one or more other Cats you control die" trigger went on the stack when a permanent that is not a Cat left the battlefield (a fetchland cracked, Wasteland sacrificed), and a yes flipped Ajani with no Cat dead. Root cause: a zone-change trigger on a permanent that has left the battlefield is matched against its last-known information (CR 603.10a), and `lki_matches` (eval.rs) checked only types, colors, controller, owner, token and names. The snapshot did not store subtypes or supertypes, so every subtype, supertype, keyword, mana value, power and toughness condition in a "dies" or "leaves the battlefield" filter passed. The same hole applied to any dies/leaves trigger with such a filter, not just Ajani.

E6 (repros/e6-two-quarries-one-victim-cast-offer.scn): with two Lazotep Quarries and one creature, a two-mana spell was not offered. RFC 0001's fallback turned every Quarry into a sacrifice source and kept only as many as there are victims, dropping the second Quarry instead of leaving it as a colorless source.

## Proposed change
`mtg-core` only.
- E7: `Lki` gains `supertypes` and `subtypes` (captured from the derived characteristics when the snapshot is taken). `lki_matches` is split into `lki_match_one` plus the `alt` alternatives, and `lki_match_one` now applies every test of the live matcher that a snapshot can answer: types, supertypes, subtypes (any/not), colors (any/not, colorless), keywords, mana value, power and toughness, controller, owner, token, names. Tests that need live state (tapped, attacking, counters on a live object, entered this turn) are not part of a snapshot and remain unchecked, as before.
- E6: `plan_cost` plans with 0, then 1, 2, ... of the sacrifice-capable permanents in sacrifice mode (fewest sacrifices first); the others keep their ordinary modes. Sacrifice sources beyond the number of available victims are dropped for that attempt.

## Rules basis
CR 603.10a (leaves-the-battlefield triggers look back in time) and 700.4 (dies = put into a graveyard from the battlefield); the snapshot must contain the characteristics the trigger condition reads. CR 605.1a and 118.3 for the Quarry (unchanged from RFC 0001).

## Impact
- State layout and `hash_rules` / `hash_full`: the hash changes. `Lki` rides inside hashed pending triggers and stack entries (`Captured.lki`), so the two new fields change the hash of any state holding one. `HASH_SCHEMA` 2 to 3 and `ENGINE_CORE_VERSION` 3 to 4 (this also covers the rules behavior changes of RFC 0001 and E7/E6).
- Determinism and golden replays: all 36 goldens (20 test-pool, 16 real-pool) were re-recorded because of the hash change only. The recorded action sequences are identical to the old ones in every file (checked), so the games themselves did not change; only the stored version, hash schema and checkpoint hashes did. The real-pool card snapshot is unchanged.
- Hidden information: none (battlefield characteristics are public; no new view or fork state).
- Effect VM / IR: none.

## Test plan
Own scenarios: Wasteland sacrifice and fetchland crack do not trigger Ajani (both fail before the fix), a Cat dying still does (existing `own-ajani-transforms-when-a-cat-dies`), two Quarries and one creature pay Amped Raptor. Plus spec visible 806, sealed holdout by the spec thread, workspace suite, 1000-game legacyfuzz, differential rerun by the differential thread.

## Verdicts on the other lockstep findings
- E4 (Cavern of Souls restricted mana needs a manual tap before the cast is offered; Brady accepted as a documented limit 2026-10-01, kd-0010): not a rules error and not a deliberate design choice. `simple_mana_ability` excludes the restricted ability from automatic payment because the restriction (creature spell of the chosen type, and the spell can't be countered) depends on the spell being cast, and automatic payment does not carry the can't-be-countered rider. Nothing is unreachable: the Cavern's explicit mana option exists, and mana floated that way is used correctly. Recommendation: accept as a documented limit for now (Brady's decision, kd-0010); a proper fix passes the spell to the planner and sets the uncounterable flag from the payment, a separate RFC if search quality shows it matters.
- E5 (Brady accepted as a documented deviation 2026-10-01, kd-0012; Animate Dead's leave-the-battlefield sentence is a separate trigger in the engine, a delayed trigger in Forge): low impact (one empty stack object and priority window when the Aura leaves before its enters trigger resolves). Not fixed here; recommend accepting as a deviation, fixable by creating the leaves trigger as a delayed trigger from the enters ability.
