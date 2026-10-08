# Group A scenarios (Stormchaser's Talent and the Otter token): notes for the implementer

25 scenarios, one file each, `scenarios/ur-A-<topic>.yaml`. Written from the Oracle text in SCENARIO-BRIEF.md (re-checked with Savecraft `card_search` on 2026-10-08, exact name) and the Comprehensive Rules (2025-11-14, Savecraft `rules_search`: rule 716 and 702.108 retrieved fresh; the other rules are copied from the existing `rust-engine-spec/reference/cr-excerpts*.md`, which were retrieved from the same module). No engine source, patch, `ur.cards.ron` or `/home/claude/work/` was read, and no `mcp__hearthbot__*` tool was called.

## Delivery and validation

- `oracle-additions-A.json`: `cards` Stormchaser's Talent and Boomerang Basics (the latter is also a group B card; the entries are identical to group B's), `tokens` "Otter Token" (1/1, colors U and R, prowess; from `card_search` token "Otter", set TBLB). Scenarios write the token as `{token: Otter Token}`.
- `cr-excerpts-ur-A.md`: every CR rule a `verified: true` source cites.
- Validation: scratch `oracle.json` (reference file plus the additions) and a refs dir holding only `oracle.json` and `cr-excerpts-ur-A.md`:
  `python3 /mnt/project-files/rust-engine-spec/tools/validate.py --oracle <scratch>/refs/oracle.json --refs <scratch>/refs /mnt/project-files/rust-engine-ur/scenarios/ur-A-*.yaml` -> `25 scenario(s), 0 error(s), 0 warning(s)`.
- No `ruling` sources and no `verified: false` entries: every expectation follows from Oracle text plus CR.

## Scenario list (file prefix ur-A-)

| file | asserts |
|---|---|
| talent-enters-level-1-creates-otter-with-prowess | enters trigger goes on the stack, then exactly one Otter: token, Creature, subtype Otter, colors U+R, 1/1, prowess, summoning sick; Class at level 1 |
| talent-cast-triggers-prowess-on-existing-otter | the Talent is an enchantment spell = noncreature: old Otter 2/2, new Otter 1/1 (did not exist at cast) |
| level-2-returns-instant-or-sorcery-from-your-graveyard | Level 2 is an activated ability on the stack (level still 1 until it resolves); trigger target chosen as it goes on the stack: only instants/sorceries in YOUR graveyard offered (creature, land, opponent's card excluded) |
| level-2-no-instant-or-sorcery-in-graveyard-no-trigger | no legal target: level 2 reached, trigger removed (603.3d), no decision; opponent's Bolt not eligible |
| level-2-trigger-target-exiled-in-response-returns-nothing | Faerie Macabre exiles the chosen target; trigger does not resolve, the other legal card is not substituted, level stays 2 |
| level-up-is-sorcery-speed-only | not with a spell on the stack, not at beginning of combat, offered in main 2, not on the opponent's turn (p0 holds priority in p1's main phase) |
| levels-are-gained-one-at-a-time-never-skipped-never-repeated | Level 3 not offered at level 1; Level 2 not offered again at level 2; neither offered while the level 2 trigger is on the stack; neither at level 3 |
| level-3-instant-cast-creates-otter-and-pumps-existing-otter | Talent trigger plus the old Otter's prowess; p0 orders them; old Otter 2/2, new Otter 1/1 |
| level-3-creature-and-artifact-spells-make-no-otter | creature spell: no triggers at all; artifact spell (Lotus Petal): prowess only |
| level-3-opponents-spells-trigger-nothing | "you cast": opponent's Bolt triggers neither the Talent nor prowess |
| level-3-trigger-resolves-even-if-the-spell-is-countered | Force of Will counters the Bolt; the cast trigger still makes the Otter |
| level-3-storm-copy-is-not-cast-so-one-otter | Flusterstorm with one storm copy: one Otter; copy retarget prompt; copy then original resolve |
| level-3-spell-cast-in-response-to-the-level-up-makes-no-otter | level applies on resolution: Bolt cast with the Level 3 ability on the stack triggers nothing; the next Bolt does |
| omniscience-free-casts-still-trigger-level-3 | two free Bolts (no lands) = two Otters, `spells_cast_this_turn: 2` |
| aluren-creature-free-cast-no-otter-instant-not-free | Aluren's free cast is creature spells only (free Bolt is illegal), the free creature triggers nothing, the paid Bolt makes an Otter |
| opponents-aluren-creature-answered-by-bolt-makes-otter | p1 casts a creature free with Aluren, p0 responds with Bolt: trigger, Bolt, creature resolve in order; priority back to p1 each time |
| force-of-will-counters-the-talent-no-otter | pitched Force of Will; Talent to graveyard, no Otter |
| daze-paid-talent-resolves-and-makes-otter / daze-declined-talent-countered-no-otter | Daze on the Talent, pay and decline |
| stifle-the-enters-trigger-no-otter | trigger targeted with `{trigger_of: "@talent"}` |
| stifle-the-level-up-activation-level-stays-one | activation targeted with `{ability_of: ...}`; mana not refunded; level stays 1; Level 2 offered again and works |
| stifle-the-level-2-trigger-level-stays-two-card-stays | level 2 already reached; Bolt stays; Level 2 not offered again |
| wasteland-cannot-target-the-talent | Talent is not a land; Wasteland ability 1 (destroy) is not offered with the Talent as target but is with Volcanic Island |
| boomerang-own-talent-draws-and-recast-resets-to-level-1 | sorcery cast at level 3 makes an Otter; bouncing own Talent draws; recast = new object at level 1 (new Otter, old Otter pumped by the enchantment spell); Level 2 offered, Level 3 not; Level 2 then returns Boomerang Basics |
| show-and-tell-puts-talent-onto-battlefield-otter-without-cast | put, not cast: still level 1 and still makes the Otter |

## Spellings I used (the adapter owns exact shapes)

- Class level: `counters: {level: N}` in setup and in assertions. I assert `counters: {level: 1}` for a level 1 Class (the brief says level 1 is the minimum and settable). **If the engine stores no counter at level 1, those assertions are the first place it will differ**; the behavioural checks (which level-up is offered) are independent of that.
- Level-up: `{t: activate, source: "@talent", ability: 0|1, pay: ...}`; the level 2 trigger's target is `{decision: choose_target, slot: 0, answer: ...}` raised after the activation resolves, before priority. With a single candidate it is still raised (one-candidate choices are presented). With none, nothing is raised.
- `pay: {tap: [...]}` is used on activations where an auto-payment might spend a Mountain meant for a later Bolt, or to keep specific Islands untapped.
- `order_triggers` with plain aliases of the two sources, `["@talent", "@otter"]` (first listed goes on the stack first). It is raised because the Talent trigger and the Otter's prowess are different abilities of different sources controlled by one player. The final state does not depend on the order chosen. Used in level-3-instant-cast-... and level-3-storm-copy-... (there `["@flu", "@talent"]`, the Flusterstorm card alias naming its storm trigger).
- Storm copy: after the storm trigger resolves, `yes_no, purpose: change_targets` for the copy's controller (same shape as the existing `flusterstorm.yaml`). p1 cannot pay {1} (tapped out), so no payment prompt is raised (the pool's "no prompt when payment is impossible" convention).
- Daze: `alt_cost: {return_island: "@j1"}`; the payment prompt is `yes_no, purpose: pay_for_daze` for the spell's controller.
- Stifle targets: `{trigger_of: "@talent", nth: 0}` and `{ability_of: "@talent", nth: 0}` (only one ability of the Talent is on the stack each time, so no `text_contains` is needed).
- A trigger on the stack without a stated source (the prowess trigger of an unaliased Otter) is written `{kind: triggered, controller: p0}` in the Boomerang scenario.
- Show and Tell: `simultaneous_secret_choice`; p1 has no eligible card so none is raised for p1.

## Rulings I was unsure about

- None rests on a recalled official ruling. The two places where the CR leaves the shape to the adapter: whether the engine raises `order_triggers` for the Talent trigger plus prowess (I say yes, different sources; an engine that auto-orders because every order gives the same result will fail the scenarios that include the step), and the `change_targets` prompt for the storm copy.
- Level 3 and a spell cast in response to the level-up (level-3-spell-cast-in-response...): relies on the level changing when the activated ability resolves (716.2a: "This Class's level becomes N"), not on activation. I am confident of this reading.
- Boomerang on own Talent: relies on 400.7 (new object) plus 716.2d (no level = level 1). 716.2b says a permanent retains its level even if it stops being a Class, but that concerns a permanent that stays on the battlefield; a bounced Class is a new object.

## Not covered (by the brief's exclusions or by design)

- Veil of Summer (brief: irrelevant); counter removal/proliferate on the Class; "becomes the target" triggers; Pyrokinesis (group B).
- A creature-token-specific scenario for Otter bounce/ceasing to exist (group B: Boomerang Basics on a token).
- Prowess expiry at end of turn is not asserted (the existing Cori-Steel Cutter/Monk scenarios cover the mechanism).
- Class copied or turned into a creature (levels are not copiable, 716.2b); the pool has no such effect.

## Schema gaps

- No assertion key for a Class level other than the `level` counter convention; nothing for "level of a permanent that is not a Class".
- Stack entries cannot name a token source without an alias; setup tokens can be aliased, created tokens need `bind` (not needed here).
