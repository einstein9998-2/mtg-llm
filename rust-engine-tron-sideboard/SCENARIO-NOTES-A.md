# Group A scenarios (Eldrazi Confluence and Summon: Bahamut): notes for the implementer

33 scenarios, one file each, `scenarios/tron2-A-<topic>.yaml` (21 Confluence, 12 Bahamut). Written from the Oracle text in SCENARIO-BRIEF.md (re-checked with Savecraft `card_search`, exact name, 2026-10-09) and the Comprehensive Rules (2025-11-14, Savecraft `rules_search`; rules 714, 400.7, 608.2b, 700.2, 704.4, 704.5m, 306.5b, 122.2, 509.1a, 301.5c retrieved fresh, the rest copied from earlier packages' excerpt files that came from the same module). No engine source, patch or `*.cards.ron` was read, and no `mcp__hearthbot__*` tool was called.

## Delivery and validation

- `oracle-additions-A.json`: `cards` Eldrazi Confluence and Summon: Bahamut. The Eldrazi Scion Token already exists (Tron additions C).
- `cr-excerpts-tron2-A.md`: every CR rule a `verified: true` source cites.
- Validation (scratch `oracle.json` = reference file plus every package's additions plus mine; refs dir holding only that and my excerpts):
  `python3 /mnt/project-files/rust-engine-spec/tools/validate.py --oracle <scratch>/refs/oracle.json --refs <scratch>/refs <work>/rust-engine-tron-sideboard/scenarios/tron2-A-*.yaml` -> `33 scenario(s), 0 error(s), 0 warning(s)`.
- Run: `spec2json.py` on a directory holding only my files, then `/home/claude/target-tron/release/specrun out.json -v tron2-A`: **30 pass, 2 fail, 1 unsupported** on the writer's first run; after the implementer's follow-ups (below) all 32 shipped scenarios pass.
- One `verified: false` entry: Karn's printed starting loyalty (5), in `confluence-blink-karn-loyalty-resets-to-printed-number` (the Oracle data has no loyalty). Everything else follows from Oracle text plus CR.

## How the Confluence multisets are written

`modes: [k+1]` in the cast action (the schema's `modes` are 1-based), where k is the brief's index: 1 = pump x3, 2 = pump x2 + blink, 3 = pump x2 + Scion, 4 = pump + blink x2, 5 = pump + blink + Scion, 6 = pump + Scion x2, 7 = blink x3, 8 = blink x2 + Scion, 9 = blink + Scion x2, 10 = Scion x3 (so brief index 0 is `modes: [1]`, 9 is `modes: [10]`). Targets are listed in the brief's slot order: every pump target first, then every blink target (Scions have none: `targets: []` or just the other slots). The engine accepted this reading in all passing scenarios (a wrong multiset would have shown as a different board).

## Scenario list

| file (tron2-A-...) | asserts |
|---|---|
| confluence-pump-three-times-one-creature-kills-it | pump x3 on one 7/7: -9 toughness, dies |
| confluence-pump-three-different-creatures-toughness-3-dies-4-lives | 1/3 and 1/1 die, 3/4 becomes 6/1 and lives; targets per instance |
| confluence-pump-wears-off-at-cleanup | 6/1 at p0's end step, 3/4 again in p1's main phase; the two Scions stay |
| confluence-blink-own-etb-creature-redraws-returns-tapped-and-sick | Quantum Riddler returns tapped, sick, enters trigger draws again; 2 Scions |
| confluence-blink-two-permanents-counters-lost-equipment-falls-off | +1/+1 counter gone, Equipment unattached (stays), both tapped and sick |
| confluence-blink-karn-loyalty-resets-to-printed-number | same with numbers: 2 loyalty -> 5 (recalled) -> 6 after a second +1 |
| confluence-blink-only-blocker-returns-tapped-attacker-unblocked | cast in declare attackers: blinked blocker is tapped, no block decision, 5 lifelink damage |
| confluence-blink-attacker-is-removed-from-combat | opponent's attacker blinked in their declare attackers step: no damage |
| confluence-blink-trinisphere-returns-tapped-and-tax-ends | Bolt illegal with one Mountain while Confluence is on the stack, castable after Trinisphere returns tapped |
| confluence-blink-cannot-target-a-land | Island, Ancient Tomb, Urza's Saga (all lands) are illegal targets of the blink mode; Voltaic Key legal |
| confluence-two-blinks-on-one-permanent-second-does-nothing | blink x2 on the same Riddler: one trigger, one draw (608.2b/400.7) |
| confluence-pump-then-blink-same-creature-survives-zero-toughness | 1/3 pumped to 4/0 then blinked in one resolution: SBAs not checked in between (704.4) |
| confluence-scion-three-times-makes-three-scions | three 1/1 colorless Eldrazi Scion creatures, sick, untapped |
| confluence-scion-three-times-sacrificed-pay-for-trinisphere | all lands tapped, the three Scions' sacrifice pays {3} (see Run result) |
| confluence-cost-needs-two-colorless-mana | four Islands: cast not offered; after Ancient Tomb is played it is |
| confluence-force-of-will-counters-it-nothing-happens | pitched Force of Will: no pump, no token |
| confluence-one-pump-target-gone-rest-resolves | Swords exiles one target: the other pump and the Scion still happen (608.2b) |
| confluence-all-targets-illegal-fizzles-no-scion | Swords exiles the only target: the spell does not resolve, the untargeted Scions are NOT made |
| confluence-instant-speed-pump-attacker-scion-blocks-and-kills | in p1's declare attackers: attacker 6/1, new Scion blocks and kills it |
| confluence-blink-in-response-to-swords-fizzles-it | Swords on the Riddler; Confluence blink in response: Swords fizzles, no life |
| bahamut-cast-enters-with-lore-counter-chapter-one-destroys | {9} cast, 9/9 flying Enchantment Creature - Saga Dragon, lore 1, sick; chapter I target set: nonland only (no lands), Bahamut itself is a candidate |
| bahamut-chapter-one-zero-targets-allowed | "up to one": decline the target, nothing destroyed (see Run result) |
| bahamut-chapter-two-after-draw-step-destroys-then-attacks-in-the-air | lore 2 only at precombat main, chapter II destroys, 9 flying damage past a ground creature |
| bahamut-chapter-three-draws-two-no-counter-on-opponents-turn | lore 2 through p1's turn and p0's draw step, 3 in main phase, two cards drawn |
| bahamut-chapter-four-damage-equals-other-permanents-mana-value | 3+4+1+0+0 = 8 (not 17), p1 12, then sacrificed; Saga stays while IV is on the stack |
| bahamut-chapter-four-permanent-destroyed-in-response-not-counted | Abrade on Grim Monolith in response: 5 damage, not 7 |
| bahamut-chapter-four-bahamut-exiled-in-response-damage-still-dealt | Swords on Bahamut (p0 gains 9): chapter still deals 6 |
| bahamut-chapter-four-no-other-mana-value-deals-zero | lands and a token only: 0 damage, still sacrificed |
| bahamut-countered-by-force-of-will-no-lore-counter | countered: graveyard, no chapter trigger |
| bahamut-swords-in-response-to-chapter-one-trigger-still-destroys | chapter I on the stack, Swords exiles the Saga: the chapter still destroys its target |
| bahamut-with-trinisphere-costs-exactly-nine | eight lands: not castable; land drop makes nine: castable (no extra tax) |
| bahamut-blocks-and-kills-an-attacker-as-a-creature | the Saga is a creature that blocks (3 damage taken, 9 dealt) |

## Run result (engine build at /home/claude/target-tron/release/specrun)

30 pass. Not passing:

- **FAIL `bahamut-chapter-one-zero-targets-allowed`**: step `{decision: choose_target, slot: 0, answer: null}` is rejected with "bad target null" (`answer: []` and omitting `answer` are rejected the same way: "bad target []", "bad target null"). The scenario is right: "up to one" allows zero targets even with legal targets (115.4) and the Oracle text says "up to one target". Either the adapter has no shape for declining an optional trigger target (the pool only has `targets: []` on activations) or the engine does not offer a "no target" choice for this chapter. The implementer should say which shape declines; the expected end state is nothing destroyed, lore 1, trigger on the stack with `targets: []`.
- **FAIL `confluence-scion-three-times-sacrificed-pay-for-trinisphere`**: with every land tapped and three untapped Scions, `cast Trinisphere, pay: auto` is "not offered" (the engine offers a single `Mana` action for the Scions, apparently de-duplicated as interchangeable). Casting with mana abilities activated during payment is legal (601.2g, 605.3b), and sacrificing has no {T} so summoning sickness does not matter (302.6). The explicit form (activate each Scion, then cast) cannot be written either: the three identical created tokens cannot be named (`bind` needs exactly one match and the handle `p0:Eldrazi Scion Token#k` is "unknown alias" for non-setup objects). Wanted step: `activate` one of several identical created tokens, or `pay: auto` that may sacrifice mana-ability sources.
- **UNSUPPORTED `confluence-blink-karn-fresh-loyalty-and-new-activation`**: "activate ability 1 of Karn, the Great Creator" (the -2: "You may reveal an artifact card you own from outside the game or choose a face-up artifact card you own in exile. Put that card into your hand."). The scenario wants `expect_illegal` of the -2 at 1 loyalty and then both abilities offered for the blinked Karn. The +1-only twin (`...loyalty-resets-to-printed-number`) passes.

## Rulings I was unsure about

- Bahamut's chapter IV with Bahamut exiled in response (`...bahamut-exiled-in-response-damage-still-dealt`): I rely on 113.7a (the ability exists independently of its source) and 608.2h (last known information). I am confident the damage is still dealt; no official ruling was retrieved.
- Blink then pump versus pump then blink on one creature (`...pump-then-blink-same-creature-survives-zero-toughness`): the outcome (creature survives as a tapped, sick 1/3) is the same whichever instance is performed first, which is why the scenario does not need an ordering ruling; 700.2d says the modes are treated as if written in sequence, the card's printed order is pump, blink, Scion.
- The brief says "blinking a land puts it back tapped"; the Oracle text says "nonland permanent", so lands cannot be targeted. `confluence-blink-cannot-target-a-land` asserts that instead.

## Not covered

- Pump on a hexproof/protected target, Confluence copied (storm, etc.), Aura falling off (no suitable Aura in the setup syntax; the Equipment case covers the attachment rule), Scion tokens sacrificed for a spell on the stack, Bahamut destroyed by its own chapter, Bahamut put onto the battlefield by Show and Tell (the lore counter is a replacement-style "as enters": covered for Urza's Saga in the Storm package).


## Implementer follow-up (2026-10-09)
- `bahamut-chapter-one-zero-targets-allowed`: the engine did offer a decline (`Done`) for the optional trigger target; the test adapter had no shape for it. `choose_target` with `answer: null` now declines. Scenario unchanged, passes.
- `confluence-scion-three-times-sacrificed-pay-for-trinisphere`: a real engine bug found by the scenario. Automatic payment skipped any creature made this turn as a mana source, even for an ability with no {T} in its cost (an Eldrazi Scion's "Sacrifice this token: Add {C}"). Summoning sickness (302.6) only stops abilities that cost {T}; fixed in `mana_sources_ex`. Scenario unchanged, passes.
- `confluence-blink-karn-fresh-loyalty-and-new-activation`: dropped from the package. It needs Karn's -2, which is not in the engine (a known Tron gap from RFC 0008, unrelated to the sideboard); its +1-only twin stays. The 33 written scenarios are therefore 32 shipped.
