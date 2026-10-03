# RFC 0003: Effect-loop indices wide enough for large boards, a resolution step cap, Quarry auto-payment avoids the spell's target

Status: in review (gates below; rules review requested from the spec thread, rerun requested from the differential thread)
Author: Rust engine core thread  Reviewers (two independent, one a rules reviewer): spec thread (rules), differential thread (rerun)

## Problem
Report: /mnt/project-files/ENGINE-BUG-ajani-runaway.md (first self-play run, Boros Aggro mirror, pool game 1449, seed 100). Resolving the +2 of Ajani, Nacatl Avenger ("put a +1/+1 counter on each Cat you control") never finished, emitted counter events until the process aborted on allocation (about 30 GB), and could not be caught by `catch_unwind`.

Root cause: the effect VM's loop state `LoopSt.idx` was a `u8`. `ForEach` snapshots the matching objects and walks the list with that index. With more than 255 matching objects (Ocelot Pride's token copying produces thousands of Cat tokens under random play) the index wrapped from 255 to 0 in release builds and the loop restarted forever. The same pattern existed in combat: the attacker and blocker positions of declaration and the attacker and blocker positions of damage assignment were `u8`, so a board of more than 255 attackers or blockers had the same hazard. Not specific to Ajani: any `ForEach` over more than 255 objects.

Also open from RFC 0001's review: automatic payment through Lazotep Quarry could sacrifice the spell's own target (Lightning Bolt aimed at a Cat token, the token sacrificed to pay, the spell fizzles).

## Proposed change
`mtg-core` only.
- `LoopSt.idx` `u8` to `u32`; combat `Attackers.i`, `Blockers.i`, `Damage.attacker_idx`, `Damage.blocker_idx` `u8` to `u16`; `AttackersDeclared` and `BlockersDeclared` counts saturate at 255 instead of truncating.
- Resolution step cap: one run of a resolution script stops after `RESOLVE_STEP_CAP` (4,000,000) instructions and the game is declared a draw, the same treatment as the 60,000-live-object arena cap of M3g. A legal script loops over at most the objects in the game (at most 60,000 before the arena cap) with a few instructions per object, so the cap is never reached in a real game and exists so that any future runaway ends the game instead of the process.
- Quarry auto-payment: when the spell or ability being paid for has object targets (top of the stack), the canonical sacrifice victim is never one of them unless nothing else can be sacrificed. The castability check cannot know targets (they are chosen later), so it stays as in RFC 0001; the choice only changes which creature is sacrificed.

## Rules basis
None: rules-neutral robustness fixes. The draw on a runaway is an engine deviation, recorded with the other two in CORE-FREEZE-REVIEW.md. The Quarry change follows CR 601.2c/601.2h (targets are chosen before costs are paid, and a sensible player does not sacrifice the target to pay); the explicit Quarry activation still lets a player choose any victim.

## Impact
- State layout and `hash_rules` / `hash_full`: correction after the hidden-info/determinism review: widening the indices does not change any hash value (the checkpoint hashes in all goldens are identical between m4 and m5; only the stored version and schema lines differ). `HASH_SCHEMA` 3 to 4 and `ENGINE_CORE_VERSION` 4 to 5 were bumped anyway, which is harmless; goldens were re-recorded for the headers.
- Determinism and golden replays: the recorded action sequences and checkpoint hashes of every golden are unchanged; only the stored version and hash schema lines change.
- Hidden information: none.
- Effect VM / IR: no new instructions or variants.

## Test plan
- Own scenario `own-ajani-avenger-plus-two-with-three-hundred-cats-terminates` (300 Cat tokens; the +2 resolves and puts a counter on each). With the index narrowed back to `u8` and the cap in place the scenario fails (the cap ends the game as a draw), which shows the wrap was the cause; without the cap that is the reported hang.
- Own scenario `own-quarry-auto-payment-does-not-sacrifice-the-spells-target`.
- Spec visible 806, own, workspace suite, 1000-game legacyfuzz, real-pool noninterference (in the suite); differential rerun and spec sealed rerun requested.

## Not covered
No test constructs more than 255 attackers or blockers; the combat widening follows the same pattern and is covered by the existing combat scenarios not changing. The step cap has no direct test because no legal script reaches it; it is exercised only through the narrowed-index experiment above.

## Items from the hidden-info and determinism review of m5 (for the next RFC, none reachable in the pool today)
1. Document that on the battlefield a permanent's view ids order the same for both seats (entry order); `sac_pick`, `attack_candidates`, `obj_key` and `target_key` depend on it.
2. `sac_candidates` should call `self.refresh()` first (with two Quarries it can read a stale derived cache; the pool is saved only because a ZoneChange refreshes when any card has a triggered ability).
3. `pay_plan` should add `ctx.picks` and the source permanent to the `avoid` list, so a Quarry cannot sacrifice a creature that is also the cost's chosen pick.
4. Stratify the Show and Tell fork test by opponent hand size (the unstratified rates differ by 0.13 against a 0.15 bound, the stratified ones agree).
