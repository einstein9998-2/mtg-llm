# "Can't be countered" vs counter effects: new scenarios and engine run (2026-10-06)

Files: `scenarios/cards/uncounterable-*.yaml` (14 scenarios, one per file). `tools/validate.py scenarios`: 838 scenarios, 0 errors, 0 warnings.
Engine run (scratch copy of rust-engine at the M3h tree, `specrun`): 838 scenarios, **829 pass, 9 fail**. All 824 pre-existing scenarios still pass; the 9 failures are all new and all are the intended result (the engine counters a spell that can't be countered).

Rule applied, from retrieved text only (Savecraft rules_search, card_search; Daze/Veil/Flusterstorm texts from the parent and `reference/oracle.json`): CR 118.12a ("unless" = the player may pay; if they don't, counter), CR 101.2 ("can't" beats "can"), CR 701.6a (counter = cancel and remove from stack), CR 702.21a (ward = triggered "counter unless pays"), CR 611.2c (Veil's effect modifies the rules, so it also covers spells already on the stack). Expected outcome for every "unless" case: the player is still offered the optional payment (a real choice is offered only when they have mana to pay; with no mana there is no prompt, as in existing scenarios); declining does NOT counter; the spell resolves.

| id | rule basis | engine | observed on the current engine |
|---|---|---|---|
| uncounterable-daze-hardcast-after-veil-decline-resolves | 118.12a, 101.2, 701.6a | FAIL | prompt is raised, answer "no" counters the Cage (not on battlefield, in graveyard) |
| uncounterable-daze-return-island-after-veil-decline-resolves | 118.12a, 101.2 | FAIL | same, with the Island-return alternative cost |
| uncounterable-daze-veil-in-response-decline-resolves | 611.2c, 118.12a, 101.2 | FAIL | Veil resolved on top of Daze; on "no" the Cage is still countered (Veil draw works) |
| uncounterable-daze-veil-in-response-pay-spends-mana | 118.12a, 101.2 | PASS | paying {1} taps the Mountain and Cage resolves |
| uncounterable-daze-veil-cannot-pay-resolves | 118.12a, 101.2 | FAIL | tapped-out controller (no prompt): the Cage is countered on the implicit decline |
| uncounterable-flusterstorm-veil-in-response-decline-resolves | 118.12a, 101.2, 707.10 | FAIL | Veil cast above Flusterstorm and its one storm copy (storm count 1: only Bolt precedes it; Veil is cast after). First decline counters Bolt, so the second prompt never appears (script fails there; Bolt would not damage p1) |
| uncounterable-force-of-negation-veil-not-countered-not-exiled | 101.2, 701.6a | PASS | Cage not countered, not exiled, resolves |
| uncounterable-pyroblast-veil-blue-spell-resolves | 101.2, 701.6a | PASS | blue Bilbo resolves (used a creature instead of Brainstorm: no put-back decisions) |
| uncounterable-daze-cavern-creature-decline-resolves | 118.12a, 101.2 (Cavern of Souls) | FAIL | Phelia cast with Cavern's colored mana is countered on "no" (not on battlefield) |
| uncounterable-koma-ward-veil-spell-decline-resolves | 702.21a, 118.12a, 101.2 | FAIL | ward prompt is raised, "no" counters p1's Veil'd Lightning Bolt; Koma takes no damage |
| uncounterable-koma-ward-veil-spell-cannot-pay-resolves | 702.21a, 101.2 | FAIL | p1 has no mana for {4} (no prompt): Bolt countered, Koma undamaged |
| uncounterable-daze-koma-spell-decline-resolves | 118.12a, 101.2 (Koma's own "This spell can't be countered") | FAIL | Koma is countered on "no" |
| uncounterable-lavinia-trigger-veil-spell-resolves | 101.2, 603.4 | PASS | Lavinia's trigger does not counter the Veil'd zero-mana Force of Will, which counters Bolt |
| uncounterable-consign-to-memory-veil-colorless-spell-resolves | 101.2, 701.6a | PASS | colorless Cage not countered |

Summary of the engine defect: the "counter unless pays" paths (`CounterUnless` for Daze and Flusterstorm, `CounterEventUnless` for ward) ignore the uncounterable flag (Veil of Summer's `SpellsUncounterable` player effect, Cavern of Souls' restricted mana and Koma's `UNCOUNTERABLE` keyword), both on a decline and on the implicit decline when the payment is impossible. The unconditional paths (`CounterSpell`, `CounterSpellExile`, `CounterAny`, `CounterSpellOf`) honor it. When the payer pays, the prompt, payment and resolution are right.

Notes for the implementer:
- Scenarios assume the pay prompt is still raised when the spell can't be countered and the controller can pay (118.12a says the player may pay; the CR does not suppress the choice). If the engine decides to skip the prompt as a no-op, that is a design change the spec thread has to approve: the scripts would then drop the `yes_no` step, but the outcome (spell resolves, no mana spent) stays.
- Storm count 0 was not achievable: Veil must be cast this turn, and the targeted spell counts too. The Flusterstorm scenario casts Veil after Flusterstorm so exactly one copy exists.
- Cavern of Souls was feasible (pool card, `flags: {chosen_type: Dog}`, ability index 1).

Pool counter effects (grep `Counter` in `legacy.cards.ron`) and coverage:
- `CounterSpell`: Force of Will, Hydroblast (red), Pyroblast (blue). Covered: Pyroblast (new), Force of Will (existing veil/cavern/koma scenarios). Hydroblast's counter mode uses the same effect; no new scenario.
- `CounterSpellExile`: Force of Negation. Covered (new).
- `CounterUnless`: Daze, Flusterstorm. Covered (new, 7 scenarios).
- `CounterEventUnless`: Koma ward {4} (the only Ward card). Covered (new, 2).
- `CounterSpellOf`: Lavinia trigger. Covered (new).
- `CounterAny`: Consign to Memory. Covered (new; Veil'd colorless spell). The triggered-ability half is not a spell, so "can't be countered" does not apply to it.
- `CounterAbility`: Stifle. Targets abilities only; "can't be countered" on spells is not applicable. Not covered, not needed.
Not covered: Cavern of Souls with a counter other than Daze (FoW is in the existing scenario; FoN and Pyroblast are untested against Cavern), and ward with an uncounterable ability (Veil covers spells only, so only the spell side applies).
