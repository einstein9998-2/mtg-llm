# Group B scenarios (Chrome Mox, Mox Opal, Giant's Boulder, Urza's Saga, Haywire Mite, Taiga, Runehorn Hellkite): notes for the implementer

58 scenarios in `scenarios/storm-B-*.yaml`, one per file. Written only from the Oracle text in SCENARIO-BRIEF.md and the Comprehensive Rules (2025-11-14, retrieved through Savecraft `rules_search`). No engine source, patch, Tron material or gap document was read. Oracle texts were re-retrieved with `card_search` and match the brief (Giant's Boulder is not on Scryfall: copied from the brief, every Boulder quote is `verified: false`).

## Files and validation

- `oracle-additions-B.json`: the seven cards of this group, Kozilek's Command (used only as a colorless nonartifact nonland card to imprint) and the `Construct Token` token (merge into `reference/oracle.json` `cards{}` and `tokens{}`).
- `cr-excerpts-storm-B.md`: verbatim CR text. 105.2c, 106.1b, 106.5, 107.4e, 202.2-202.2d, 205.4a, 207.2c, 305.7, 607.2a, 701.9a, 714.2b, 714.3a, 714.3b, 714.4 were retrieved fresh; the rest were copied unchanged from existing excerpt files (same Savecraft module). Append to `reference/`.
- Command (scratch `oracle.json` = pool + additions; scratch refs dir holds only my excerpts file):
  `python3 /mnt/project-files/rust-engine-spec/tools/validate.py --oracle <scratch>/oracle.json --refs <scratch refs dir> /mnt/project-files/rust-engine-storm/scenarios/storm-B-*.yaml`
  Result: `58 scenario(s), 0 error(s), 0 warning(s)`.
- All CR citations are `verified: true`. Oracle quotes are verified except Giant's Boulder. There is no ruling source: every expectation follows from Oracle text plus CR. The one place an expectation rests on a recalled official ruling is Chrome Mox with nothing (or a colorless card) imprinted producing no mana (see "Unsure").

## Scenario map

Counts: Chrome Mox 9, Mox Opal 7, Giant's Boulder 8, Urza's Saga 12, Haywire Mite 7, Taiga 6, Runehorn Hellkite 9.

| file (storm-B-...) | what it pins |
|---|---|
| chrome-mox-imprint-blue-card-taps-for-blue-only | cast for {0} counts as a spell; trigger on the stack; imprint candidates exclude artifacts (Lotus Petal) and lands (Island); blue imprint taps for {U}, not R/G |
| chrome-mox-imprint-declined-produces-no-mana | "may" declined (empty answer); Mox taps for nothing |
| chrome-mox-imprint-multicolored-card-either-color | Prismari Charm {U}{R}: U or R at activation, not G/W/B |
| chrome-mox-imprint-hybrid-card-is-both-colors | Spider-Woman {1}{W/U}: W or U |
| chrome-mox-imprint-colorless-card-no-mana | Kozilek's Command ({X}{C}{C}) imprintable, Mox adds no mana (not {C}) |
| chrome-mox-leaves-battlefield-imprinted-card-stays-exiled | Haywire Mite exiles the Mox; imprinted card stays exiled; {U} made in response stays in the pool; dies trigger order |
| chrome-mox-imprint-trigger-stifled-no-imprint | the imprint is a triggered ability: Stifle on it, card stays in hand |
| chrome-mox-put-by-show-and-tell-still-imprints | Mox put (not cast) still imprints; taps at once; storm count 1 |
| chrome-mox-no-eligible-card-raises-no-imprint-choice | only an artifact and a land in hand: no decision raised |
| mox-opal-two-artifacts-off-third-artifact-on | Opal counts itself; Opal+Petal off, cast Bauble: on |
| mox-opal-artifact-creature-counts-no-summoning-sickness | Haywire Mite counts; Opal entered this turn taps at once |
| mox-opal-opponents-artifacts-do-not-count | "you control" |
| mox-opal-petal-sacrificed-first-turns-metalcraft-off | restriction re-evaluated per activation |
| mox-opal-tapped-first-then-petal-gives-both-mana | mana stays when an artifact leaves afterwards |
| mox-opal-second-copy-legend-rule | legal: `legend_rule` decision, keep the untapped new copy |
| mox-opal-cast-as-third-artifact-taps-at-once | Opal itself is the third artifact |
| giants-boulder-etb-scry-two-bottom-one | scry looks at exactly two cards; one to the bottom; untapped; costs {1} |
| giants-boulder-etb-scry-two-keep-and-reorder | keep both, new order |
| giants-boulder-etb-trigger-countered-by-consign-to-memory | trigger can be countered; no scry decision |
| giants-boulder-filters-one-pool-mana-to-any-color | pool {C:2} -> {C:1, U:1} (net one mana), sick artifact usable, tapped afterwards |
| giants-boulder-needs-pool-mana-and-seven-for-destroy | no mana anywhere: neither ability offered |
| giants-boulder-seven-destroys-target-permanent | exactly 7 Islands; sacrifice is a cost; destroys Aluren |
| giants-boulder-six-lands-not-enough-then-seven-after-land-drop | 6 not enough, 7 after land drop; targets an opponent's artifact |
| giants-boulder-destroy-ability-stifled-boulder-already-gone | Stifle: target survives, Boulder still gone |
| urzas-saga-enters-with-lore-counter-mana-only-after-chapter-one | lore counter on entering; chapter I on stack; no mana until it resolves; land drop counted |
| urzas-saga-lore-counter-after-draw-step-not-on-opponents-turn | still 1 on p1's turn and in p0's draw step; 2 at main phase; chapter II on stack; Construct ability not offered until it resolves; chapter I's mana ability persists |
| urzas-saga-construct-token-alone-is-one-one | Construct counts itself (1/1); costs {2} + Saga's {T} |
| urzas-saga-construct-counts-artifacts-you-control-and-shrinks | 3/3 with two other own artifacts, opponent's artifacts ignored, 2/2 after one leaves |
| urzas-saga-construct-ability-needs-two-other-mana | Saga's own {C} cannot pay for its own {T} ability; one other land not enough |
| urzas-saga-tapped-for-mana-cannot-make-construct | tap for {C} first and the Construct ability is gone for the turn |
| urzas-saga-chapter-three-finds-zero-or-one-cost-artifact-then-sacrificed | search mask (mana cost {0} or {1} only; Haywire Mite included; mana value 2 artifacts, Cutter, nonartifacts excluded); Saga still on battlefield and tappable while the trigger is on the stack; found card put (no spell cast); Saga sacrificed after |
| urzas-saga-chapter-three-fetches-chrome-mox-imprint-after-sacrifice | SBA sacrifice happens before the imprint trigger is put on the stack |
| urzas-saga-chapter-three-trigger-countered-saga-still-sacrificed | Consign to Memory on chapter III: no search, Saga sacrificed afterwards |
| urzas-saga-wasteland-in-response-to-chapter-three-search-still-happens | ability exists independently of its source (113.7a) |
| urzas-saga-wasteland-in-response-to-chapter-one-no-mana-ever | cannot tap in response; chapter I fizzles into nothing |
| urzas-saga-put-by-show-and-tell-gets-lore-counter-not-a-land-play | enters via Show and Tell: lore counter, chapter I, land drop still available |
| haywire-mite-sacrifice-exiles-artifact-and-gains-two | exile not destroy; dies trigger resolves first; {G} must come from Taiga, Mountain stays untapped |
| haywire-mite-targets-noncreature-artifact-or-enchantment-only | Urza's Saga (Enchantment Land) is a legal target; Taiga, other Mite, Construct token, itself are not |
| haywire-mite-ability-in-response-to-bolt-fizzles-bolt | sacrifice in response to Bolt: Bolt countered on resolution, 2 life once |
| haywire-mite-ability-stifled-target-survives-life-still-gained | Stifle picks the exile ability (text_contains) not the dies trigger |
| haywire-mite-needs-green-mana | Mountain and Island cannot pay {G} |
| haywire-mite-cast-and-sacrificed-the-same-turn | no {T} in the cost: no summoning sickness |
| haywire-mite-countered-by-force-of-will-no-life | countered on the stack is not "dies" |
| taiga-taps-for-red-or-green-only | intrinsic abilities from the two land types; no U/B/W |
| taiga-wasteland-can-target-nonbasic-taiga-not-basic-mountain | nonbasic; mana made in response stays |
| taiga-scalding-tarn-finds-it-as-a-mountain / -windswept-heath-finds-it-as-a-forest / -polluted-delta-cannot-find-it | search by land subtype |
| taiga-under-magus-of-the-moon-is-a-plain-mountain | layer 4: only {R}, no Forest |
| runehorn-hellkite-hard-cast-needs-red-and-six-mana | 5/5 flying Dragon, Taiga makes the {R} |
| runehorn-hellkite-no-red-neither-cast-nor-graveyard-ability | six Islands: nothing offered |
| runehorn-hellkite-graveyard-ability-discards-hands-draws-seven | exiled as a cost (visible while ability on stack); both players discard hand then draw seven |
| runehorn-hellkite-activated-at-instant-speed-in-opponents-turn | in response to Bolt on p1's turn; Bolt still resolves |
| runehorn-hellkite-ability-stifled-hellkite-still-exiled | Stifle: hands unchanged, Hellkite stays exiled |
| runehorn-hellkite-exiled-as-cost-no-graveyard-hate-response | Faerie Macabre cannot target the exiled Hellkite; Force of Will and Consign to Memory cannot target the ability |
| runehorn-hellkite-library-shorter-than-seven-loses | 5-card library: draws 5, loses (104.3c, 704.5b) |
| runehorn-hellkite-ability-only-works-from-the-graveyard | not offered from hand or battlefield |
| runehorn-hellkite-wheeled-hellkite-is-reusable-same-turn | second Hellkite discarded by the first wheel is activatable at once |

## New action and assertion spellings (the validator does not check them; the runner adapter must learn them)

1. **Imprint choice**: `{decision: choose_cards, purpose: imprint, answer: ["@card"]}`; the empty answer `[]` declines the "may". `expect_options` lists the eligible hand cards (nonartifact, nonland). When no hand card is eligible NO decision is raised (scenario `chrome-mox-no-eligible-card-raises-no-imprint-choice`); an adapter that raises a `yes_no` first should skip it for the declined case only if it does the same everywhere (the rules fix the outcome, not the shape).
2. **Mana of an exiled card's colours**: Chrome Mox's mana ability is `{t: activate, source: "@mox", ability: 0, choose_color: X}`; with exactly one colour imprinted the scripts omit `choose_color` (no choice). With several colours (Prismari Charm, Spider-Woman) the colour is chosen at activation, and `legal:` patterns with `choose_color` include/exclude say which colours are offered. With a colorless or no imprint, the activation (if offered) produces no mana.
3. **Ability index conventions** (activated and mana abilities in Oracle order, triggered/static not counted): Chrome Mox 0 = tap for mana; Mox Opal 0; Giant's Boulder 0 = `{1},{T}` filter, 1 = `{7},{T}`, sacrifice; Urza's Saga: gained abilities count as if printed in chapter order, 0 = `{T}: Add {C}` (chapter I), 1 = `{2},{T}: Create a Construct` (chapter II); Haywire Mite 0; Runehorn Hellkite 0; Wasteland 0 = mana, 1 = destroy; Taiga 0 = its mana ability with `choose_color: R|G` (SCHEMA addendum for duals); Faerie Macabre 0 (via `activate_from_hand`).
4. **Activating an ability from the graveyard**: `{t: activate, source: "@hk", ability: 0}` where `@hk` is a card in the graveyard (no new `t` value; `activate_from_graveyard` was not invented). The negative case for a Hellkite in hand uses the existing `activate_from_hand`.
5. **Counters**: `counters: {lore: N}` on Urza's Saga, in descriptors and checks (new counter name).
6. **Token**: `Construct Token` (0/0 colorless artifact creature token with `+1/+1 for each artifact you control`) in `oracle-additions-B.json`; descriptors use `{token: Construct Token, pt: [..], types: [Artifact, Creature], subtypes: [Construct]}`; `bind: {find: {name: "Construct Token", controller: p0}}`.
7. **Stifle / Consign targets**: `{ability_of: "@src"}` for activated abilities, `{trigger_of: "@src"}` for triggers, with `text_contains` where one source has several abilities on the stack (Haywire Mite).
8. **Mana pool in setup**: `mana_pool: {C: 2}` is used once (Giant's Boulder filter scenario), so the floating mana does not depend on how mana was produced.

## Reliance on runner behaviours to confirm

- **`advance` stopping with a chapter trigger on the stack.** The Urza's Saga scenarios cross turns with `advance: {to: main1, of: p1}` then `advance: {to: main1, of: p0}`. When the lore counter is added at the start of p0's precombat main phase the chapter ability triggers and is put on the stack before p0 gets priority; the scripts assume `advance` stops at that first priority with the trigger on the stack (same reliance as the Tron package). If the runner reads SCHEMA 3.3 more strictly, replace by explicit passes. To keep `advance` legal on the way, no scenario gives p1 an Aether Vial (its upkeep "may" would be a decision), and p0/p1 have no creatures able to attack.
- **`advance: {to: draw, of: p0}`** is used once (lore counter not yet added in the draw step).
- **Triggers from `resolve_top`**: scripts use `resolve_top` after a cast, then check the trigger on the stack, then `resolve_top` again; the decision from the resolving trigger (imprint, scry, search) follows.
- **Shuffles**: Urza's Saga chapter III and the fetchlands shuffle; scenarios assert only `library_count` (random: any).
- **Library padding**: Hellkite scenarios pad libraries with Islands, so "draw seven" asserts a hand of seven Islands whether or not the engine shuffles.

## Rulings and assumptions I was unsure about

1. **Chrome Mox with no imprint or a colorless imprint produces no mana.** The Oracle text names no fallback; I recall the official ruling (no mana) but could not retrieve it (no egress), and CR 106.5 (mana of an undefined type produces no mana) is cited by analogy, so those two scenarios are the least certain Mox expectations. They also assert that the activation is OFFERED (it can be activated for no effect), as is true of any ability whose cost can be paid. If the runner's mask hides a useless activation, tell the spec-writer rather than editing.
2. **Haywire Mite targets Urza's Saga.** Urza's Saga is an Enchantment Land, not a creature, so it is a "noncreature enchantment" (scenario `haywire-mite-targets-...`). I did not retrieve a ruling; it follows from the type line.
3. **Imprint with no eligible card raises no decision.** I believe the rules give the player no choice there; adapters that always raise a `yes_no` for a "may" would differ.
4. **Giant's Boulder**: per the brief the engine pays `{1}` only from the pool. In the real rules a player may also tap a land (or any mana source) during the cost payment of the filter ability. I wrote NO scenario that relies on a land being unable to pay; the only negative scenarios (`needs-pool-mana-and-seven-for-destroy`) give the player no mana source at all, which is illegal in the real rules too. The `{7}` destroy ability is paid normally from lands.
5. **Runehorn Hellkite does not shuffle.** The brief says Echo of Eons and Runehorn Hellkite shuffle libraries; Oracle text for the Hellkite says nothing about a shuffle. I followed the brief's instruction to assert counts and zones only, but because the library padding in my scenarios is all Islands, none of my Hellkite assertions depends on whether a shuffle happens. A scenario with `random: []` asserting "no shuffle" could be added if the implementer wants one.
6. **Magus of the Moon and Taiga** (layer 4 / 305.7) is my own extension beyond the listed topics; Magus is in the engine pool. If the engine does not model Magus, drop that scenario.
7. **`spells_cast_this_turn`** is read per seat (as in earlier packages): Chrome Mox and Mox Opal cast for {0} each count as one spell; a permanent put onto the battlefield by Show and Tell or by chapter III does not.
8. **Priority**: the scripts rely on the active player receiving priority after a resolution (117.3b) and on a player who casts or activates keeping priority (117.3c).
9. **Turn structure of the Saga scripts**: chapter I triggers on entering (turn 3), chapter II in p0's precombat main phase of turn 5, chapter III of turn 7 (p1's turns 4 and 6 pass with no lore counter).

## Things I could not express

- **Which abilities a Saga has gained** cannot be put in a setup (`counters: {lore: N}` alone does not say chapter I/II have resolved), so every Saga scenario that needs a gained ability plays the Saga through the real turns instead of starting mid-way. This makes several scripts long.
- **The imprinted card's link to the Mox** cannot be set up either; Chrome Mox is always cast or put in the script.
- **Giant's Boulder's chosen mana colour spent** is deliberately unasserted (brief).
- **"No decision pending"** has no explicit step: the "no eligible imprint card", "Consign counters scry trigger" and "Stifle on the imprint trigger" scenarios rely on the next script step failing if an unexpected decision were raised.
- **Faerie Macabre's "up to two" targets** is written with one target; zero or two targets are not tested.
