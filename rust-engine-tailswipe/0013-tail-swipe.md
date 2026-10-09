# RFC 0013: Tail Swipe (fight, and "if you cast this spell during your main phase")

Status: draft; implemented and tested in a scratch copy of the live engine; independent review pending
Author: Tail Swipe thread (Brady asked for it in the "Chant and Petal" thread, 2026-10-09)

## Problem
Brady wants to test Tail Swipe ({G} instant, Dominaria United) in the Alurentell postboard deck: with Atraxa (lifelink) it gains life at instant speed, including in response to Pyroblast or Boomerang Basics. The pool has no fight, and `Effect::Damage` deals damage from the spell, not from a creature, so lifelink and deathtouch would not apply.

Oracle text (card sites, 2026-10-09): "Choose target creature you control and target creature you don't control. If you cast this spell during your main phase, the creature you control gets +1/+1 until end of turn. Then those creatures fight each other."

## Proposed change
`mtg-core` (IR only, both cases appended at the end of their enums so existing discriminants and the card database hash of existing pools are unchanged):
1. `Effect::Fight { a, b }`: the creatures in target slots `a` and `b` each deal damage equal to their power to the other (CR 701.12). Source of each damage event is the creature, so lifelink, deathtouch and protection work. Nothing happens unless both targets are still legal creatures on the battlefield (701.12b) and they are different objects. Powers are read before damage is dealt, so the exchange is simultaneous in effect.
2. `Cond::InMainPhase`: the current step is Main1 or Main2. Tail Swipe uses `And([IsActive(You), InMainPhase])` for "during your main phase"; it is evaluated at resolution, which is the same step as casting because a step does not end while the stack is non-empty.
3. `lint.rs`: `max_target_used` reads the two slots.
Card data: `Tail Swipe` in `legacy.cards.ron` (green section).

## Rules basis
CR 701.12 (fight), 702.15 (lifelink: damage dealt by the creature gains that much life), 601.2c/608.2b (targets checked on resolution). The +1/+1 is a layer 7c effect until end of turn.

## Impact
- State layout, `hash_rules` / `hash_full`: none. `Effect` and `Cond` gain one case each at the end. Test-pool goldens replay bit-identically.
- The live card database hash changes because a card was added (as for any card addition): nets and game records made against the previous live pool must be regenerated.
- `ENGINE_CORE_VERSION`: unchanged.

## Test results (scratch copy of live + this patch)
- Visible spec set 838/838 pass; 5 new scenarios pass (main phase Atraxa fight with lifelink gains 8, opponent's turn gives no pump, combat phase gives no pump, pump keeps a 5/5 Acererak alive against a 5/5 Murktide, both targets must differ in controller).
- `cargo test --workspace --release`: all pass except `legacy_decks_random_games_hold_invariants` (expects 8 decks, finds 9); it fails identically on the untouched live engine, so it is not caused by this change.
- Limits: the fight is not offered as a modal "bite"; opponent-controlled lifelink or protection edge cases beyond Atraxa's were not tested.
