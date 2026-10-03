# RFC 0001: Sacrifice-cost mana abilities count in the castability planner (Lazotep Quarry)

Status: accepted (post-freeze fix for differential finding E3; gates green, rules review by the spec thread requested)
Author: Rust engine core thread  Reviewers (two independent, one a rules reviewer): spec thread (rules), hidden-information reviewer (pending)

## Problem
Forge casts Goblin Bombardment ({1}{R}) from Plains + Lazotep Quarry ("{T}, Sacrifice a creature: Add one mana of any color") when a creature is available. The engine offered no cast: automatic payment only knew "simple" mana abilities (tap, optionally sacrifice self), so the Quarry's second ability was only an explicit mana action. Repro: differential-tests/repros/e3-quarry-sacrifice-mana-cast.scn (differential-tests/REPORT.md, E3).

## Proposed change
`mtg-core/src/legal.rs` only.
- `sac_mana_ability(ad)`: an activated mana ability whose cost is exactly `{T}` plus one `Sacrifice(filter)` and whose effect is one mana of any color.
- `plan_cost` first plans with the ordinary sources (unchanged). Only if that cannot pay, and the seat controls an untapped permanent with such an ability, it plans again with `mana_sources_ex(.., allow_sac = true)`, where such a permanent offers any color for one sacrifice instead of its other modes (one tap per permanent). Sacrifice sources are kept only up to the number of victims that are not themselves sources of the payment.
- Applying the payment sacrifices a canonical victim: tokens first, then lowest mana value, then the normal option order (`sac_pick`). A player who wants a different victim activates the Quarry explicitly before casting; the explicit option list (one option per victim) is unchanged.

## Rules basis
CR 605.1a / 605.3b (mana abilities are activated while paying a cost; the player chooses what to sacrifice, 118.3 and 601.2h). The engine's canonical victim is a simplification of the player's choice; the explicit activation is the full-choice path. Verified against Oracle text pinned in rust-engine-spec/reference/oracle.json (Lazotep Quarry); the Forge behaviour is the differential evidence.

## Impact
- State layout and `hash_rules` / `hash_full`: none (no new state; `SrcInfo` is a local planning struct).
- Determinism and golden replays: all goldens unchanged (suite green).
- Hidden information: sacrifices only the actor's own battlefield permanents, which are public; no new view or fork state.
- Effect VM / IR: none.
- `ENGINE_CORE_VERSION` bump: no (no layout or hash change). Action sets differ from M3g only at positions that need the Quarry to pay.

## Test plan
- Own scenarios `own-quarry-sacrifice-mana-makes-a-spell-castable` (cheapest creature is sacrificed, the cast succeeds) and `own-quarry-not-sacrificed-when-plain-mana-pays` (no sacrifice when ordinary mana suffices).
- Spec visible 806/806, own 25/25, workspace suite, 1000-game legacyfuzz and the real-pool noninterference run (results in MILESTONES.md).
- Differential rerun of the E3 repro by the differential thread.

## Known limit (from the spec thread's review, to fold into the next RFC)
With `pay: auto` the canonical victim does not avoid the spell's own target: Swords to Plowshares on the only creature can have that creature sacrificed to the Quarry and fizzle. A player who cares activates the Quarry explicitly first. The fix is to skip targets of the spell being paid for when choosing the victim (low severity; not done in RFC 0001).
