# Rules freeze review of the M3 candidate (spec thread, 2026-10-01)

Scope: CORE-FREEZE-REVIEW.md section 4, rules reviewer: `trigger.rs`, `replace.rs`, `sba.rs`, `derive.rs`, `combat.rs`, `cast.rs` against the CR effective 2025-11-14. Done by two read-only reviewers I directed (one for trigger/replace/sba/derive, one for combat/cast), each citing CR text from the rules tool; every finding below was run on a copy of the engine with `specrun` probes (probe scenarios were kept in my sandbox, not in the sealed holdout). I re-ran the probes for findings 1 to 4 and they fail as described. I did not independently re-derive the "checked and correct" list. These are not holdout scenarios and reveal nothing from the sealed set.

**Recommendation: do not freeze yet.** Findings 1 to 5 are real rules errors in code the freeze locks; 1, 2 and 3 change the shape of the IR or event pipeline, which is exactly what an RFC would be needed for later. Brady's sign-off is still required and nothing was pushed.

## Blocking, in order of severity
1. **Every trigger condition is re-checked at resolution as if it were an intervening "if".** `resolve.rs:108-127` re-checks `TriggeredDef.cond`; the IR cannot mark a condition that belongs to the trigger event (CR 603.4: the rule "only applies to an 'if' that immediately follows a trigger condition"). Case: Tamiyo, Inquisitive Student, draw for the turn, then Brainstorm: the third card triggers her but by resolution four cards were drawn, so `CardsDrawnThisTurn == 3` is false and she never transforms. Common Legacy line. Caveat: her Oracle text is one of the five not pinned in oracle.json, so her exact wording is recalled, not verified. Moonshadow and the Ring's level conditions use the same path but are harmless today. Fix: a second trigger-condition variant in the IR.
2. **Simultaneous zone changes are processed one object at a time, with triggers matched after each single move** (`cx.rs:20`, `ops.rs:149`, `sba.rs:146`, `resolve.rs:1354`, `trigger.rs:60-65`). Rules: 603.10a leaves-the-battlefield abilities look back in time; 603.6a all permanents, including newcomers, are checked; 614.12 only already-existing effects apply to an entering permanent. Confirmed failures: Massacre kills Ajani, Nacatl Pariah and his Cat token together and Ajani's "other Cat dies" does not trigger; Hide on the Ceiling returning Guide of Souls with Ocelot Pride triggers Guide only if Guide returns first; Hide on the Ceiling returning Containment Priest with an opposing Murktide Regent exiles the Regent if the Priest returns first. The result depends on target or battlefield order. Fix: batch zone-change events, or snapshot the battlefield and replacement sources per batch.
3. **"Can't enter" is treated as an ordinary replacement** (`replace.rs:62-127`; Grafdigger's Cage `Prevent` sits in the list with Containment Priest's `Exile`). CR 614.17c: if an event can't happen it can only be replaced by a self-replacement. Case: Priest in play, then Cage, Reanimate on Griselbrand: it is exiled; it should stay in the graveyard. This also falsifies the claim in the `replace.rs` header that every pool combination gives the same result in any order, and 616.1 (the affected object's controller chooses the order) is not modelled.
4. **Combat: the second damage step uses the wrong creature list.** `combat.rs:131-139, 463-478`: `first_struck` records only creatures that dealt first-strike damage, so a first striker with 0 power in that step (the `power == 0` skip at line 363) is never recorded. CR 510.4: the regular step is only for creatures that had neither first strike nor double strike when the first step began, plus double strikers. Case: Ocelot Pride with Animate Dead (0/1 first strike) attacks unblocked; during the first-strike step Acererak is flashed in with Aluren and puts a +1/+1 counter on it; in the regular step it deals 1 damage, but the CR says it deals none. Tamiyo's +2 shrinking an attacker to 0 power gives the same case. The same flaw applies to blockers. Fix: record every first or double striker at the start of the first-strike step.
5. **Warp: a warped card can never be cast again from exile** (already in the holdout report). CR 702.185a: "Its owner may cast this card after the current turn has ended for as long as it remains exiled." Needs a permission kind with a "not this turn" gate in `resolve.rs` and `priority.rs`, which are frozen files.

## Deviations from CORE-FREEZE-REVIEW section 3 that I would not accept as listed
- Item 1, Tamiyo, Seasoned Scholar's +2 applied directly in combat code (`combat.rs:220-236`) rather than as a trigger (CR 603.2). The attacker cannot Stifle or Consign to Memory it, both of which are in the pool. Fix, or accept in writing before the freeze.
- Item 2, Raph & Mikey's new attacker copies Raph & Mikey's attack target (CR 508.4: its controller chooses). Matters when the opponent controls Kaito or Tamiyo.

## Lower priority (list under section 3 as known deviations)
- Within one layer all resolved-effect instances are applied before all statics, regardless of timestamp (`derive.rs:141-187`, CR 613.7). Only visible case: Guide of Souls' "becomes an Angel" on Kaito is wiped by his older static `SetTypes`; no pool card reads Angel.
- The legend rule runs as a separate pass after the other state-based actions (`stabilize.rs`, CR 704.3 says one simultaneous event). No observable pool case beyond finding 2.
- The Aura check (`sba.rs:87-99`) tests only for a missing object, not an illegal one (CR 704.5m). The only Aura is Animate Dead and no pool card makes its creature illegal.
- Phyrexian mana is announced after targets and delve; CR 601.2b puts it before targets (601.2c). Same outcome for Dismember and Surgical Extraction; only the order of decisions shown to an agent differs.
- Derive.rs:162 re-checks the `lost` flag in every layer, which would break 613.6 lock-in for a nonbasic land whose static spans several layers. The pool has none.
- Card data: the Sand Warrior token should have two creature types, Sand and Warrior.

## The two deps-scan pairs and counters (the section 2 and 3 asks)
- Kaito vs Magus of the Moon and Overlord of the Balemurk vs Magus of the Moon: I agree both are independent under 613.8 (Magus affects only nonbasic lands; neither Kaito nor Overlord can become a land or change whether Magus applies). No other same-layer pair in the pool has a dependency: layer 4 holds only Magus, Kaito, Overlord, the Ring-bearer becoming legendary and Guide's Angel; layers 6 and 7c are additive; 7a is only CDAs; 7b is only Kaito.
- Counters (section 3 item 8): the pool's layer 4 and layer 6 conditions read only whether a counter is present (Kaito's loyalty, Overlord's time counters), never a value derived from counters, so building +1/+1, -1/-1 and keyword counters into `derive.rs` is safe for this pool.

## Checked and found correct
Trigger ordering (APNAP, 603.3b) and removal of targetless triggers (603.3d); delayed triggers fire once; both players losing at once is a draw (704.5a); deaths from 0 toughness, lethal damage and deathtouch; 0-loyalty planeswalkers; Equipment unattaching from a non-creature (704.5n); +1/+1 and -1/-1 counter cancellation; Magus of the Moon removing all land types and keeping supertypes (305.7) with the 614.12 lookahead for surveil lands; casting order (modes, additional costs, replicate, X before targets), one alternative cost per spell, copies are not cast (707.10) and don't count toward storm; trample lethal assignment with deathtouch and marked damage (702.19b, 702.2c); no damage spill from a planeswalker (702.19f); menace and the other block restrictions; ninjutsu's attack target (702.49c); the Ring's level 3 sacrifice.
Not checked: Daze/Flusterstorm "unless pays" timing, and anything outside the six files.

## Open question (not a finding)
Whether Kaito keeps the planeswalker type while it is "a 3/4 Ninja creature": the engine replaces types (205.1a literal reading, which the second reviewer also recalls as matching the official rulings but could not verify). Left in OPEN-QUESTIONS.md.

---

# Re-review of the M3d build (spec thread, later 2026-10-01)

Scope: diff of the old candidate against the M3d build (`batch.rs`, `trigger.rs`, `replace.rs`, `sba.rs`, `resolve.rs`, `stabilize.rs`, `combat.rs`, `priority.rs`, `cx.rs`, `ops.rs`), by one more read-only reviewer plus my own probe reruns; 11 probes of its own, which I re-ran for the two new findings.

**Recommendation from the rules side: accept the freeze**, with the two items below fixed or listed in CORE-FREEZE-REVIEW section 3. Neither needs an IR or event-pipeline change. Brady's sign-off and the other reviewers (hidden information, determinism) are still required; I am not approving on their behalf. Nothing was pushed.

## The five blocking findings and the two deviations: all verified fixed
1. Event conditions are no longer re-checked as intervening "if" (`resolve.rs:110`, `event_cond`); the Tamiyo draw-step-then-Brainstorm case passes. The other conditioned triggers in the pool were checked (replicate, evoke, Carpet of Flowers, Sand Scout, Acererak, Overlord are true intervening ifs; Moonshadow's "while" and the Ring's level checks are really event conditions but are still marked intervening, which is harmless).
2. Batched simultaneous zone changes: look-back for departed objects (603.10a), newcomers see each other (603.6a), already-existing effects only for entering permanents (614.12). Massacre with Ajani and his Cat, the Hide on the Ceiling cases, and Show and Tell with Containment Priest (both already in play and entering together) all pass.
3. "Can't enter" applies before other replacements (614.17c); Cage plus Priest plus Reanimate leaves Griselbrand in the graveyard.
4. Second damage step uses every first or double striker recorded at the start of the first-strike step (`combat.rs:333`; 510.4), read and probed.
5. Warp: cast from exile for the mana cost works on the owner's later turns. The "not on the turn it was exiled" gate is missing, but nothing in the pool can reach it (Quantum Riddler has no flash and Aluren only covers mana value 3 or less).
6. Tamiyo, Seasoned Scholar's +2 is a real delayed trigger on the stack (`attack_watch`), including attacks on planeswalkers, ending at her controller's next turn; Raph & Mikey's and Mobilize's entering attackers ask for an attack target (508.4) and emit no Attacks event.

## New findings (neither is a regression: the old build behaves the same)
1. **Stronghold Gambit puts the revealed creatures onto the battlefield one at a time** (`resolve.rs:968`; 614.12). On a mana-value tie, p0 reveals Containment Priest and p1 reveals Spider-Woman: Spider-Woman is exiled; swap the reveal and the Priest enters tapped. If the owners' placements are one simultaneous event, neither happens. The rules text ("The owner of each creature card revealed this way with the lowest mana value puts it onto the battlefield") does not itself say "simultaneously", so this is partly an open question; the fix is two lines (wrap the loop in `begin_batch`/`end_batch`) and the pool has the case (Reanimator plays four Gambits, Boros has the Priest and Spider-Woman). CORE-FREEZE-REVIEW section 3 item 1 says the unbatched moves have no pool case; this one does. I will add a scenario once the engine decides.
2. **With Grafdigger's Cage out, Animate Dead stays on the battlefield** after its enters trigger fails to return the creature (`sba.rs:88-99`; 704.5m: an Aura attached to an illegal object is put into its owner's graveyard). Animate Dead loses "enchant creature card in a graveyard" and gains "enchant creature put onto the battlefield with Animate Dead", which nothing satisfies. If Cage later leaves and the creature is reanimated, a spurious sacrifice trigger goes on the stack and does nothing. Death and Taxes plays Cage and Reanimator plays four Animate Dead, so section 3 item 9's "no pool case" is wrong for this one. Fix the check or correct the note.
3. Low, nothing observable today: Ajani, Nacatl Avenger's -4 sacrifice loop, empty-hand discard and ExileGraveyard are still one object at a time; the Aura and planeswalker SBA batches take their look-back snapshot at move time. The doc comment on `TapAttackMoved` in `ir.rs` is stale.

## Checked and correct
Nested batches and depth; held-back events matched in order; departed objects keep the snapshot controller for APNAP; triggers of a source that has left do nothing on resolution; the scripted random bottom order; Doomsday auto-filling the pile when five or fewer candidates exist; the Brainstorm path; `attack_watch` is included in the state hash, and the transient `batch` field is never alive across a decision point.

---

# Re-review of the M3f build (spec thread, 2026-10-01 evening)

Scope: diff of M3d against M3f (fork/mtg-view API rework, per-seat ids, hashing, `resolve.rs`, `trigger.rs`, `ops.rs`, `libfx.rs`, `legal.rs`, `engine.rs`, `state.rs`), Stronghold Gambit and Animate Dead from M3e. One read-only reviewer plus my reruns of its probes on the M3d and M3f builds.

**Recommendation from the rules side: not yet.** Spec on M3f: visible 806 of 806, sealed 342 of 342, but there is one confirmed regression that no scenario in the visible or sealed sets covers (I wrote a fresh sealed scenario for it).

## Regressions found (probes re-run by me: pass on M3d, fail on M3f)
1. **Miracle is never offered** (`resolve.rs:989`; the new `way_feasible_x` gate also evaluates the alt cost's `cond`, and Triumph of Saint Katherine's miracle alt has `cond: Cmp(Const(0), Eq, Const(1))` to block a normal cast; `priority.rs:218-225` then rejects it every time). CR 702.94a: "You may reveal this card from your hand as you draw it if it's the first card you've drawn this turn. When you reveal this card this way, you may cast it by paying [cost] rather than its mana cost." Triumph drawn first in the draw step with two or five Plains: no choice is raised. Triumph is sideboard-only for UWx, but the bug is in frozen `resolve.rs`. Fix: check affordability only for the miracle way.
2. **Two triggers from different attached permanents are treated as interchangeable** (`trigger.rs:371-373` with `obj_sig` in `legal.rs:486`, which hashes only that a permanent is attached, not to what). CR 603.3b says the controller chooses the order. Case: two Cori-Steel Cutters, one on Containment Priest and one on Griselbrand, then a second spell: M3d asks for the Flurry trigger order, M3f orders them itself, which changes which creature loses its Cutter while the second trigger is on the stack. Fix: hash the identity of the attached-to object. (A related case with two unattached Cutters already auto-orders on M3d; fine since the triggers are identical.)

## Pre-existing, list as a known limitation
Forks ignore known top or bottom library segments: after Thassa's Oracle the observer knows which card is on the bottom, but 30 of 40 forks put it elsewhere (same on M3d and M3f); the same applies to Dig and to Raph & Mikey's revealed bottom cards, and to Triumph's pile-back. Affects agent search worlds only, not real games.

## Checked and correct
Stronghold Gambit: both placements in one batch, higher mana value or non-creature stays in hand, the Priest and Spider-Woman tie case. Animate Dead with Grafdigger's Cage goes to the graveyard (704.5m). `batch.rs`, `replace.rs`, `sba.rs` unchanged since M3d. Option dedup `obj_sig` is stricter than before so no legal option is lost (some redundant trigger-order prompts, for example a summoning-sick next to a non-sick Soul Warden). Reveal changes alter only knowledge and view-id order, not rules logic. Fork keeps hand and zone sizes and known identities, resamples Show and Tell and Gambit picks to a legal choice, and is refused unless it is the observer's own decision. Per-seat ids never feed rules logic. In 18 of 20 goldens the option sequences match M3d; the other two differ only by the extra trigger-order prompt.

---

# Review of RFC 0001 (M3h, Lazotep Quarry sacrifice mana in the castability planner), spec thread

Scope: `legal.rs` only (`sac_mana_ability`, `mana_sources_ex(allow_sac)`, `sac_pick`, `plan_cost`), read and probed on a copy of the tagged build.

**Rules verdict: accept.** Casting a spell by activating Lazotep Quarry's second ability while paying is legal (605.3a; sacrifice is a cost paid with the rest of the cost, 601.2h; the player chooses the victim, which the engine's canonical pick simplifies, with the explicit activation kept as the full-choice path). The planner uses the sacrifice mode only when the plain sources cannot pay, never uses a creature that is itself a source of the payment, and offers no cast without a victim (probes: Bombardment from Plains + Quarry + token passes and sacrifices the token; no cast with no creature; a Priest, a token and a Murktide Regent, the token goes first then the cheapest; two Quarries with one creature offers no cast).

**One low-severity note, not a blocker:** the canonical victim can be the spell's own target. With only a Quarry and one creature, Swords to Plowshares targeting that creature is offered and `pay: auto` sacrifices the target to pay (legal under 601.2c then 601.2h, the spell fizzles), so the agent sees a pointless action. Preferring a victim that is not a target of the spell when one exists, or just documenting it, would be enough.

**Coverage:** two fresh sealed scenarios (castable with a lone creature, not castable without one). The first fails on M3f and passes on M3h.

---

# Review of RFC 0002 (M4: last-known-information filters and multi-Quarry payment), spec thread

Scope: diff of the M3h and M4 builds (`eval.rs`, `event.rs`, `ops.rs`, `sba.rs`, `legal.rs`), read and probed.

**Rules verdict: accept.**
- **E7 (LKI filters):** `Lki` now stores supertypes and subtypes and `lki_match_one` applies every test of the live matcher that a snapshot can answer (types, supertypes, subtypes, colors, keywords, mana value, power and toughness, controller, owner, token, names, with `alt` alternatives), as 603.10a requires for leaves-the-battlefield triggers. Pool dies and leaves filters use only subtype, owner, controller, token and type, all now checked; the tests left unchecked (tapped, attacking, counters, entered this turn, ring-bearer, has-X) appear in no dies or leaves trigger in the pool. Probes: a Goblin token dying and an opponent's Cat dying no longer trigger Ajani (the Goblin case fails on M3h and passes on M4); a Cat dying still does. One caveat: mana value is read from the printed definition, so a transformed or copied permanent would use that face's value; no pool filter reads it.
- **E6 (several Quarries):** `plan_cost` tries 0, then 1, 2, ... sacrifice modes (fewest sacrifices first), the rest keep their ordinary modes; two Quarries and one creature now pay {1}{R} (fails on M3h, passes on M4); two Quarries with no creature offer no cast. The earlier low note stands: the canonical victim can be the spell's own target.
- **Deviation verdicts in the RFC:** E4 (Cavern of Souls restricted mana needs a manual tap) is a documented limit and not a rules error: agree. E5 (Animate Dead's leave sentence as a separate trigger instead of a delayed trigger): agree it is low impact and acceptable as a deviation.
- Hash and version change (HASH_SCHEMA 3, ENGINE_CORE_VERSION 4) covers only the added `Lki` fields; I did not re-read the goldens' re-recording.
- Spec on M4: visible 806 of 806, sealed 350 of 350 (including the five new scenarios).

## RFC 0003 / core-frozen-m5 (engine v5): rules review, 2026-10-02

Reviewer: spec thread (independent of the engine core thread). Build reviewed: the m5 mirror of `rust-engine/`, diffed against m4 (`combat.rs`, `cost.rs`, `frame.rs`, `legal.rs`, `lib.rs`, `resolve.rs`).

Results: visible 806/806 and sealed 352/352 on m5 (the sealed set now includes two scenarios added for this RFC). On m4 the 300-creature sealed scenario does not finish (killed after the timeout), which is the bug RFC 0003 fixes.

Findings:
1. Loop index and combat indices. Widening `LoopSt.idx` to u32 and the combat indices to u16 is correct. A 300-attacker combat (no blocks, and one block out of 300 possible blockers) now gives the right life total and deaths; on m4 it never finishes.
2. Quarry auto-payment. `cost.rs` builds `avoid` from the object targets of the spell on top of the stack and `sac_pick` sorts avoided objects last, so a spell's own target is sacrificed only when nothing else can pay. Probes pass both ways (other creature available: it is sacrificed and Bolt resolves; only the target available: it is sacrificed). This closes the low-severity note from my RFC 0001 review. Only auto-payment is changed; a player who pays by hand can still sacrifice the target, which the rules allow.
3. Step cap. `RESOLVE_STEP_CAP` of 4,000,000 per resolution run ends the game as a draw. Deviation to list (kd entry): the rules have no such limit, and 104.4b would let an unbreakable loop be a draw only when no player can break it, so the cap is a safety valve, not a rules result. No scenario in the pool reaches it.
4. Combat counts saturate at 255 in the `AttackersDeclared`/`BlockersDeclared` events (`.min(255) as u8`). Deviation to list: no pool card cares about more than 255 attackers, and the cap only affects the count carried by the event.

Verdict: approve. No rules defect found in the RFC.

Runner note (not an engine finding, seen on m3f through m5): with two or more identical tokens on each side, a script that blocks each attacker one-to-one makes the runner's `swap_identity` mislabel objects, and the engine then asks for a combat damage assignment as if one attacker had several blockers. A single block, or distinct creatures, work. Spec authors should avoid one-to-one blocks between identical tokens until the runner is fixed.
