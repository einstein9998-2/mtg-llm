# Group A scenarios (Static Prison, Samut, Hazoret's Champion, Sunbaked Canyon): notes for the implementer

24 scenarios, one file each, `scenarios/boros-A-<topic>.yaml`. Written from the Oracle text in SCENARIO-BRIEF.md (Static Prison and Sunbaked Canyon re-checked with Savecraft `card_search`, exact name, 2026-10-08) and the Comprehensive Rules (2025-11-14, Savecraft `rules_search`, every cited rule retrieved on 2026-10-08, except 603.3d, 603.6a and 400.7, copied from the UR group-A excerpts, same module). No engine source, patch, `boros.cards.ron` or `/home/claude/work/` was read, and no `mcp__hearthbot__*` tool was called.

## Delivery and validation

- `oracle-additions-A.json`: `cards` Static Prison, Sunbaked Canyon, Samut, Hazoret's Champion. No tokens needed (Goblin Token is in `reference/oracle.json`).
- `cr-excerpts-boros-A.md`: every CR rule a `verified: true` source cites.
- Validation: scratch `oracle.json` (reference file plus the additions) and a refs dir holding only `oracle.json` and `cr-excerpts-boros-A.md`:
  `python3 /mnt/project-files/rust-engine-spec/tools/validate.py --oracle <scratch>/refs/oracle.json --refs <scratch> /mnt/project-files/rust-engine-boros/scenarios/boros-A-*.yaml` -> `24 scenario(s), 0 error(s), 0 warning(s)`.
- Pool cards used in setups (all in `reference/oracle.json`): Plains, Mountain, Phelia, Aether Vial, Skyclave Apparition, Flickerwisp, Prismatic Ending, Karakas, Guide of Souls, Wasteland, Lightning Bolt, Swords to Plowshares, Dragon's Rage Channeler, Goblin Token.

## Samut is unverified

Samut, Hazoret's Champion is not in the Savecraft database (`card_search` by name and by text both found nothing, 2026-10-08). Its text ("Creatures you control have haste."), type line (Legendary Creature — Human Warrior Cleric) and 2/2 body come from the brief; every Samut Oracle source in the scenarios is `verified: false` with a note. The CR citations about haste, summoning sickness and continuous effects are verified true.

## Scenario list (file prefix boros-A-)

### Static Prison

| file | asserts |
|---|---|
| static-prison-etb-exiles-target-and-gives-two-energy | enters trigger on the stack with the chosen target, energy still 0 until it resolves, then Phelia exiled and 2 energy |
| static-prison-target-restrictions-nonland-opponents-only | target options are exactly the opponent's nonland permanents (creature, artifact); a legendary land, a basic land, the controller's own creature and the Prison itself are not offered |
| static-prison-no-legal-target-trigger-removed-no-energy | the opponent has only lands: no target decision, ability removed (603.3d), NO energy, Prison stays |
| static-prison-leaves-battlefield-exiled-permanent-returns | opposing Prismatic Ending (X = 0) exiles the Prison; the prisoner returns at once under its owner's control as a new object; energy is kept |
| static-prison-exiled-token-ceases-to-exist | a token is a legal target; exile empties at once; nothing returns when the Prison leaves |
| static-prison-first-main-phase-pay-energy-keeps-it | trigger at the beginning of p0's first main phase; paying: energy 2 to 1, Prison stays |
| static-prison-first-main-phase-decline-payment-sacrificed | declining: Prison sacrificed, energy unchanged |
| static-prison-no-energy-sacrificed-without-a-choice | 0 energy: trigger still goes on the stack, no decision, Prison sacrificed automatically, energy 0 |
| static-prison-declined-payment-returns-exiled-permanent-next-turn | full cycle: exile on turn 3, p1's whole turn 4, decline on turn 5, Prison sacrificed, prisoner returns |
| static-prison-triggers-only-at-its-controllers-first-main-phase | no trigger in p0's second main phase or in either of p1's main phases (the `advance`s would fail on any decision) |
| static-prison-removed-in-response-to-etb-trigger-nothing-exiled-energy-still-gained | Aether Vial puts Skyclave Apparition in response; Skyclave exiles the Prison; the Prison trigger then resolves: the prisoner is NOT exiled (610.3b) but the controller gets {E}{E} |
| static-prison-blinked-by-flickerwisp-old-card-returns-new-prison-exiles-again | Flickerwisp blinks the Prison; the first prisoner returns at once; the Prison comes back at the end step as a new object, triggers again, exiles Flickerwisp, energy 2 + 2 = 4 |

### Sunbaked Canyon

| file | asserts |
|---|---|
| sunbaked-canyon-taps-for-red-and-pays-one-life | ability 0 with `choose_color: R`: life 20 to 19, `mana_pool {R: 1}`, nothing on the stack; Swords (W) is illegal, Bolt castable |
| sunbaked-canyon-taps-for-white-and-pays-one-life | mirror with W: Bolt illegal, Swords exiles Phelia, p1 gains 2 |
| sunbaked-canyon-pay-life-at-one-life-is-legal-and-loses | at 1 life the ability is offered (119.4); after paying, p0 is at 0 and p1 wins |
| sunbaked-canyon-sacrifice-to-draw-costs-one-mana-no-life | ability 1: costs paid on activation (Canyon already in graveyard, Mountain tapped), no life paid, ability on the stack, draw on resolution |
| sunbaked-canyon-sacrificed-in-response-to-wasteland-draws-and-fizzles-it | Wasteland targets the Canyon; the Canyon is sacrificed for a card in response; Wasteland's ability fizzles |
| sunbaked-canyon-draw-ability-needs-one-other-mana-source | only source of {1} is the Canyon itself: ability 1 is not offered (`legal` exclude and `expect_illegal`), ability 0 still is |
| sunbaked-canyon-mana-ability-in-response-uses-no-stack | in response to an opposing Bolt, the mana ability leaves the stack unchanged and keeps priority; the R pays for p0's own Bolt; final life 16 / 17 |

### Samut, Hazoret's Champion

| file | asserts |
|---|---|
| samut-cast-this-turn-gives-itself-and-an-earlier-creature-haste | Samut cast this turn attacks at once and gives haste to a creature that entered earlier this turn (effect not locked in, 611.3a); 3 damage |
| samut-gives-haste-to-creatures-that-enter-later | a creature cast with Samut already out has haste and attacks; 611.3c |
| samut-does-not-give-haste-to-opponents-creatures | the opponent's just-entered creature cannot attack (no declaration is raised, the `advance` passes through combat) |
| samut-leaving-the-battlefield-removes-haste | Bolt kills Samut; the summoning-sick creature loses haste and cannot attack |
| samut-second-copy-legend-rule | second Samut: `legend_rule` decision, old one kept (marked with 1 damage), new one to the graveyard |

## Spellings I used (the adapter owns exact shapes)

- **Static Prison payment is `yes_no, purpose: pay_energy`**, not `choose_number`. The brief says the Prison's "unless you pay {E}" is "a yes/no decision when you have energy" (SCHEMA addenda use `choose_number, purpose: pay_energy` for Wrath of the Skies, where an amount is chosen). If the adapter wants `choose_number` (1 = pay, 0 = decline) for a one-energy payment, tell the spec-writer and the three affected scenarios (pay, decline, full cycle) change in one place each. With 0 energy no decision is raised (118.3), asserted.
- Beginning-of-main-phase triggers: the scenarios start in the opponent's end step (turn 2, `phase: end`, `active: p1`) and use explicit passes (p1, p0, p0, p1, p0, p1) to reach p0's first main phase with the trigger on the stack and p0 holding priority, the same walk the Aether Vial upkeep scenario uses for upkeep. I did not use `advance: {to: main1}` for this because `advance` stops "when the active player has priority in the step" and the schema does not say whether a pending trigger on the stack counts.
- ETB trigger target: `choose_target, slot: 0` by the trigger's controller, raised as the trigger goes on the stack, after the two passes that resolve the permanent spell (same shape as the UR Level 2 trigger). With zero legal targets nothing is raised (603.3d).
- Vial put and Skyclave trigger: `choose_cards, purpose: put_onto_battlefield` then `choose_target` by p1, as in the Flickerwisp scenarios.
- `resolve_top` is used for the Flickerwisp scenario where the active player (p1) holds priority.
- Prismatic Ending with `x: 0` pays only {W} (one color), enough for the mana value 1 Prison.
- Canyon: ability 0 = the mana ability, 1 = the draw ability (Oracle order); `choose_color: R|W` picks the colour; the draw ability is paid with `pay: {tap: ["@m1"]}`.
- `sick: true` is asserted on Phelia returning on p1's own turn (new object that entered this turn). I did not assert `sick` on hasty creatures (Samut scenarios) because the descriptor is ambiguous for them (entered this turn vs. cannot attack).
- Samut's haste is asserted as `keywords: [haste]` (granted keyword, subset check) and through behaviour (offered as an attacker, or not).

## Rulings I was unsure about

- **Static Prison, removed before its enters trigger resolves (energy still gained).** The exile does nothing (610.3b, retrieved). That the controller still gets {E}{E} follows from 608.2b (the target is still legal, so the ability resolves normally) and the single-ability text; I could not retrieve an official Gatherer ruling, so the scenario is `derived_from: cr`. I am confident.
- **No target (opponent has only lands): no energy.** One triggered ability with a target that cannot be chosen is removed as a whole (603.3d); "You get {E}{E}" is not a separate ability. A player expecting the energy anyway would be wrong; flagged in the scenario notes.
- **Prison blinked by Flickerwisp.** The first prisoner returns when the Prison leaves (the Flickerwisp trigger resolution); the Prison returns at the beginning of the next end step as a new object and triggers again; relies on 400.7 and 603.6a. No recalled ruling.
- **Static Prison's first-main-phase trigger on the turn it is cast.** It is cast during the first main phase (already begun), so there is no sacrifice that turn. This is implicit in the cast scenarios (the next trigger is only reached in the full-cycle scenario); I wrote no standalone scenario.
- **Mana ability and priority.** The Canyon response scenario uses the sequence "p1 passes, p0 activates the mana ability and casts a spell with the mana", which avoids the open question in the SCHEMA addenda (whether a pass-activate-pass sequence returns priority).

## Not covered (by the brief's exclusions or by design)

- Paying life with a life total of exactly 0 cannot be constructed (a player at 0 life has already lost); the boundary tested is 1 life (legal) instead. Paying 1 life with 1 life is the only case in which the "life total greater than or equal to the payment" test is decisive.
- Samut together with other haste sources (layer ordering: excluded by the brief). Removal of Samut after attackers are declared (the attack stays legal; not worth a scenario).
- Static Prison on a permanent with counters or auras (new-object behaviour is covered with Phelia and the token); energy higher than 3 (no scenario depends on a cap).
- The Prison exiling a creature with the opponent responding by sacrificing it (nothing new rules-wise: 603.3d/608.2b covered by the trigger scenarios).

## Schema gaps

- Setup cannot express "exiled by this Static Prison" (the link between the Prison and a prisoner), so scenarios that need a prisoner build it through play. The "pay energy / decline / no energy" scenarios use a Prison in setup with no prisoner for exactly this reason, and a separate full-cycle scenario proves the return.
- No assertion key for "still has summoning sickness" separate from `sick` (entered this turn).
- `advance` semantics when a trigger is pending at the target step are unspecified (see above); I avoided it.
- Delayed trigger sources (Flickerwisp's return) are asserted as `{kind: triggered, source: "@wisp", controller: p1}`; the SCHEMA addenda only fixes this for warp, not for Flickerwisp, but the existing Flickerwisp scenarios do the same.
