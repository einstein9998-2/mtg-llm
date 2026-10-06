# Group C scenarios (Urza's lands, Planar Nexus, Kozilek's Command, Warping Wail, Eldrazi tokens, Tezzeret, Ugin): notes for the implementer

63 scenarios, one file each, `scenarios/tron-C-<topic>.yaml`. Written from the Oracle text in SCENARIO-BRIEF.md and the Comprehensive Rules (2025-11-14, retrieved with Savecraft `rules_search`). No engine source, patch, `tron.cards.ron` or gap document was read, and no `mcp__hearthbot__*` tool was called.

## Delivery and validation

- `oracle-additions-C.json`: `cards` (Urza's Tower, Urza's Workshop, Planar Nexus, Kozilek's Command, Warping Wail, Tezzeret, Ugin, Liquimetal Coating, plus Urza's Mine and Urza's Power Plant) and `tokens` (Eldrazi Spawn Token, Eldrazi Scion Token). Brief cards are verbatim from the brief. Urza's Mine, Urza's Power Plant and the two tokens come from Savecraft `card_search` (exact name). Liquimetal Coating is a group B card; my entry is the brief's text, so the two groups' entries are identical. The Ugin -11 line is the brief's placeholder; no scenario quotes it.
- `cr-excerpts-tron-C.md`: every CR rule a `verified: true` source cites, copied verbatim from `rules_search`.
- Validation (scratch copy of `reference/oracle.json` with the additions, my excerpts file as the only cr-excerpts in the refs dir):
  `python3 /mnt/project-files/rust-engine-spec/tools/validate.py --oracle <scratch>/refs/oracle.json --refs <scratch>/refs /mnt/project-files/rust-engine-tron/scenarios/tron-C-*.yaml` -> `63 scenario(s), 0 error(s), 0 warning(s)`.
- No `ruling` sources and no `verified: false` entries. Every expectation follows from Oracle text plus CR.

## Scenario list (what each asserts that an engine could get wrong)

| group | file (tron-C-...) | asserts |
|---|---|---|
| Urza's Tower | tower-alone-taps-for-one-colorless | no Mine/Power-Plant: {C}; Urza's and Tower land types |
| | tower-with-mine-only-taps-for-one | needs BOTH other lands |
| | tower-full-tron-taps-for-three | Tower 3, Mine 2, Power Plant 2 = 7; tapped lands still count |
| | tower-loses-tron-when-mine-destroyed-by-wasteland | condition checked at activation, not cached |
| | tower-under-magus-of-the-moon-taps-for-red | Magus: lands become Mountains, lose Tron, {R} only |
| Urza's Workshop | workshop-no-metalcraft-taps-for-one | two own artifacts (opponent's three do not count): only {C} |
| | workshop-metalcraft-alone-one-mana | counts itself once (not per artifact) |
| | workshop-metalcraft-counts-all-urzas-lands-tapped-or-not | Workshop + Tower + Mine + Power Plant = 4, tapped Mine counts |
| | workshop-metalcraft-counts-planar-nexus | Nexus is an Urza's land (2) |
| | workshop-metalcraft-lost-when-artifact-leaves | activation restriction rechecked after Lotus Petal is sacrificed |
| Planar Nexus | nexus-has-every-nonbasic-land-type | the ten Oracle-listed nonbasic types, no basic types |
| | nexus-one-and-tap-adds-any-color | {1},{T}: any color, {1} from another land |
| | nexus-pays-colored-spell-with-help | Nexus + Mountain pay Brainstorm's {U} mid-cast (explicit `pay: {tap: [...]}`) |
| | nexus-alone-cannot-make-colored-mana | a lone Nexus cannot pay its own {1} |
| | nexus-with-tower-makes-three / nexus-with-mine-counts-as-power-plant-and-tower | one Nexus satisfies both halves of a Tron condition (Tower 3, Mine 2) |
| | nexus-under-magus-is-just-a-mountain | only {R}, no {C}, no any-color |
| Kozilek's Command | command-cost-needs-two-colorless-mana | {C}{C} not payable with Islands, Petal, a lone Tower |
| | command-full-tron-x5-tokens-and-exile | X = 5 with Tron, modes 1+3 |
| | command-x0-tokens-and-draw | X = 0 (Ancient Tomb pays {C}{C}): no tokens, scry 0 is no event (no decision), draw |
| | command-x2-tokens-scry-then-draw | two Spawn, scry 2 THEN draw |
| | command-exile-creature-mana-value-x-or-less | MV vs announced X (Acererak 3, Atraxa 7, Phelia 2) |
| | command-x0-exile-token-only-and-draw | token MV 0; token ceases to exist |
| | command-exile-up-to-x-graveyard-cards | up to X, both graveyards, X cap |
| | command-choose-exactly-two-distinct-modes | one, three, or the same mode twice are not offered |
| | command-defense-grid-taxes-on-opponents-turn / -no-tax-on-own-turn | {3} tax on X spell; "its controller" is the spell's controller |
| | command-partial-fizzle-one-mode-target-gone | one mode's target illegal, other mode still resolves (608.2b) |
| | command-omniscience-free-cast-x-is-zero | 107.3b: free cast forces X = 0 |
| Warping Wail | wail-cost-needs-colorless | {C} not payable with colored mana |
| | wail-exile-power-or-toughness-one-or-less / wail-exile-toughness-one-token | OR, not AND (Guide of Souls 1/2; 2/1 Cat Warrior token) |
| | wail-counters-show-and-tell | counter a sorcery on the opponent's turn |
| | wail-counter-cannot-target-instant | mode 2 needs a sorcery spell; unusable modes not offered |
| | wail-scion-token-and-sacrifice | 1/1 Scion, summoning-sick sacrifice for {C} |
| | wail-choose-exactly-one-mode | two modes not offered |
| Eldrazi tokens | eldrazi-spawn-sacrifice-pays-colorless-cost | sacrifice is a mana ability, pays {C} of Wail |
| | eldrazi-spawn-summoning-sick-sacrifice | sacrifice has no {T}: summoning sickness irrelevant |
| | eldrazi-tokens-ignore-null-rod | token creatures are not artifacts; Petal is blocked, Spawn is not |
| Tezzeret | tezzeret-enters-with-four-loyalty | printed 4 (brief); not an artifact so no self-trigger |
| | tezzeret-artifact-enters-adds-loyalty | counter on ENTER (trigger on stack, loyalty still 4 until it resolves) |
| | tezzeret-opponents-artifact-does-not-trigger | "you control" |
| | tezzeret-show-and-tell-artifact-enters-once | a put (not a cast) triggers; only the own artifact (5 not 6) |
| | tezzeret-zero-untaps-artifact-no-counter / -creature-no-counter / -artifact-creature-with-counter | the artifact-creature rider, checked on resolution (Liquimetal Coating makes the Scion an artifact) |
| | tezzeret-minus-three-searches-artifact-mana-value-one-or-less | MV 0/1 legal, Null Rod/Cori-Steel Cutter (MV 2) and Island not |
| | tezzeret-minus-three-at-three-loyalty-dies-but-resolves | 704.5i then 113.7a |
| | tezzeret-loyalty-timing-and-limits | empty stack only; one loyalty activation per turn; trigger counter is not one |
| | tezzeret-minus-three-not-offered-at-two-loyalty | 606.6 for -3 and -7 |
| | tezzeret-minus-seven-emblem-robot-artifact / tezzeret-emblem-existing-artifact-creature-stays-itself | emblem trigger at beginning of combat, own artifact only; becomes a 0/0 Robot only if not a creature |
| Ugin | ugin-cast-trigger-exiles-colored-permanent | cast trigger above the spell; colorless permanents not targetable; one exile only |
| | ugin-cast-trigger-no-colored-permanent-no-target | "up to one" with no candidate: Ugin still resolves |
| | ugin-cast-trigger-survives-force-of-will | trigger independent of the countered spell |
| | ugin-colorless-spell-trigger-lotus-petal / -on-warping-wail / -on-kozileks-command | {0}, {1}{C}, {X}{C}{C} are colorless spells |
| | ugin-colored-spell-and-opponents-colorless-spell-do-not-trigger | Bolt and the opponent's Wail trigger nothing |
| | ugin-plus-two-gains-three-and-draws / ugin-zero-adds-three-colorless-via-stack | +2 effects; the 0 is a stack ability (not a mana ability), once per turn |
| | ugin-plus-two-resolves-after-ugin-is-bolted | loyalty-0 death while the ability is on the stack |
| | ugin-trigger-ignores-veil-of-summer-hexproof | hexproof from blue/black does not stop a colorless source |

## Spellings I used (the adapter owns exact shapes)

- Loyalty ability indexes follow Oracle order counting only activated abilities: Tezzeret 0 = the 0, 1 = -3, 2 = -7; Ugin 0 = +2, 1 = the 0 (-11 would be 2).
- `x: N` and `modes: [..]` (1-based printed order) on a cast; `targets:` in mode order (a player for modes 1/2 of Command, then the creature, or then the graveyard cards).
- Cast triggers with a target (Ugin): the target is a `choose_target, slot: 0` decision raised immediately after the cast action, before priority (601.2i). A cast of a colorless spell that finds no colored permanent raises no decision.
- `pay: {tap: [...]}` on an ACTIVATION (Planar Nexus {1},{T}). The Nexus scenarios name the payment explicitly. The coordinator says automatic payment now handles the Nexus; those two scenarios would also pass with `pay: auto`.
- `bind: {as: scion, find: {name: "Eldrazi Scion Token", controller: p0}}` for a single created token. Tokens in setup/expect use `{token: Eldrazi Spawn Token}` and `{token: Eldrazi Scion Token}`.
- Emblems: `command: [{emblem: "put three +1/+1 counters"}]`.
- `search` with `expect_options` include/exclude for the Tezzeret -3.
- Land activation in Magus scenarios: `ability: 0, choose_color: R`, as the addenda for basic-land-type lands.

## Rulings I was less sure of (all derived from CR plus Oracle; none from a recalled ruling)

1. **Planar Nexus satisfies both halves of Tower's and Mine's condition** ("an Urza's Mine and an Urza's Power-Plant"): one permanent can be both because the Nexus has all land types (205.3i) and the card text has no "other" or "different". Tower + Nexus = {C}{C}{C}, Mine + Nexus = {C}{C}, Workshop counts the Nexus. I believe this matches the official rulings but could not retrieve them. If you disagree, the two `nexus-with-...` scenarios are the ones to challenge.
2. **Urza's Mine / Power Plant text** was retrieved with `card_search`, the card name is "Urza's Power Plant" (hyphen only in the land type and in the Oracle text of the other lands). If your card data names it "Urza's Power-Plant", change the name in four scenarios.
3. **Urza's Workshop** is two Oracle abilities (indexes 0 and 1), but the brief says the engine models them as one. The metalcraft scenarios use `ability: 1` for the metalcraft mana and `ability: 0` for the plain {C}; the adapter should map both indices to the single ability and the metalcraft ones must still produce N mana (N = Urza's lands) when metalcraft holds, one {C} when the player has the basic mode. I wrote no scenario that depends on choosing the {C} mode while metalcraft is available.
4. **Ugin cast trigger with no colored permanent anywhere** (`ugin-cast-trigger-no-colored-permanent-no-target`): I assume the trigger goes on the stack with zero targets, as in the existing Loran scenario for "up to one", because 603.3d removes a trigger only when a choice is *required*. If the engine drops it instead, that scenario's two stack checks differ; the outcome (Ugin resolves, nothing exiled) is the same. I did not write the "candidates exist but the player chooses zero" case because the decision shape for declining an "up to one" trigger target is not in the schema.
5. **Kozilek's Command mode 4 with nothing to target / X = 0**: not tested (I could not decide from the CR text whether an "up to X" mode with zero possible targets may be chosen alone; likely yes, 115.6 and 601.2c, but unverified). Mode 4 is tested with X = 3 and cards in graveyards.
6. **Defense Grid**: "except during its controller's turn" is read as the SPELL's controller's turn (the only sensible reading). Two scenarios (tax on the opponent's turn, none on the own turn).
7. **Ugin's colorless-spell trigger and Kozilek's Command / Warping Wail**: colorless because no colored mana symbol (202.2b, 107.4c). Kindred is not a color question.
8. **Veil of Summer vs Ugin** relies on 702.11d only (hexproof from [quality] matches the SOURCE's quality) and on Ugin being colorless.
9. **Tezzeret -7 emblem becomes a Robot permanently** (no duration stated, so it lasts; asserted only at the same step, never later).
10. **Starting loyalty**: Tezzeret 4 and Ugin 7 come from the brief. Other scenarios set loyalty directly in setup; `ugin-plus-two-resolves-after-ugin-is-bolted` uses an artificial loyalty of 1 on purpose.
11. **Cat Warrior Token** P/T 2/1 comes from Ajani's Oracle text in `oracle.json` ("create a 2/1 white Cat Warrior creature token"); the token entry itself pre-exists in `oracle.json`.

## Schema gaps and things I did not express

- No assertion that a spell was cast "with X = n" beyond the stack entry's `x` field (`stack: [{kind: spell, x: 5, modes: [1, 3]}]` is used; STACK_KEYS accepts both).
- No key for "was this a mana ability" or "did the stack get used"; I assert `stack: []` plus pool contents instead.
- No `subtypes_exclude`, so "Nexus has no basic land type" is shown by its mana (no colored mana) rather than by a subtype assertion.
- The open priority question (SCHEMA addenda): I avoided scenarios that depend on whether the opponent regains priority after a mana ability in response (the Spawn/Scion sacrifice in response to Kozilek's Command was replaced by a Swords to Plowshares response).
- Not covered on purpose: Kozilek's Command targeting the opponent with its player modes, Workshop's two abilities as separate abilities, Ugin -11, Urza's Saga.
- Dependencies between groups: Liquimetal Coating (group B card, same oracle entry) is used in three Tezzeret scenarios to make a creature an artifact creature; no pool creature is an artifact.
