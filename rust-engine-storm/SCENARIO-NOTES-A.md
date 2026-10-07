# Storm group A scenarios: notes for the implementer

63 scenarios, one file each, `scenarios/storm-A-<topic>.yaml`, for Burning Wish, Gamble, Beseech the Mirror, Tendrils of Agony, Empty the Warrens, Peer into the Abyss, Boomerang Basics and Echo of Eons. Written only from the Oracle text in SCENARIO-BRIEF.md (re-checked with Savecraft `card_search`; the text matches) and the Comprehensive Rules (2025-11-14, Savecraft `rules_search`). No engine source, patch, Forge output or gap document was read. Every `verified: true` source is Oracle text in `oracle-additions-A.json` / `reference/oracle.json` or CR text in `cr-excerpts-storm-A.md`. Three sources are `verified: false` (a recalled storm ruling, a 608.2h last-known-information fragment, and the Boomerang Basics type line "Sorcery - Lesson" which the validator cannot check); no expectation rests only on them.

| card | scenarios |
|---|---|
| Burning Wish | 7 |
| Gamble | 5 |
| Beseech the Mirror | 12 |
| Tendrils of Agony | 11 |
| Empty the Warrens | 6 |
| Peer into the Abyss | 7 |
| Boomerang Basics | 7 |
| Echo of Eons | 8 |

## Files and validation

- `oracle-additions-A.json`: 9 entries (the 8 cards plus Gaea's Will, which Beseech scenario `beseech-bargained-gaeas-will-...` needs; group C ships an identical Gaea's Will entry). Merge into `reference/oracle.json` `cards{}`. Orim's Chant, Force of Will, Force of Negation, Consign to Memory, Veil of Summer, Prismatic Ending, Lotus Petal, Dark Ritual, Brainstorm, Ponder, Aluren, Faerie Macabre, the basic lands and the Goblin token are already in the pool.
- `cr-excerpts-storm-A.md`: verbatim CR text for every rule I cite (58 rules; the ones that already exist in `reference/cr-excerpts*.md` were copied unchanged). Copy it into `reference/`.
- Command: `python3 /mnt/project-files/rust-engine-spec/tools/validate.py --oracle <scratch>/oracle.json --refs <scratch refs dir> /mnt/project-files/rust-engine-storm/scenarios/storm-A-*.yaml`, with a scratch `oracle.json` = the pool plus my additions and a refs dir holding only `cr-excerpts-storm-A.md` (self-sufficient). Result: `63 scenario(s), 0 error(s), 0 warning(s)`. (Files of groups B and C sit in the same folder; I validated only `storm-A-*`.)

## New action and decision spellings (the schema has none; the validator does not check them)

1. **Sideboard**: `setup.p0.sideboard: [Card, Card, ...]` (inside the seat mapping; the validator ignores it). The validator does not register aliases on sideboard entries, so scenarios refer to a sideboard card by the doc 04 handle `"p0:Tendrils of Agony#1"`, where k counts that name in the seat's setup in the usual zone order with the sideboard counted LAST (every sideboard card in these scenarios has a unique name in its seat, so k is always 1). After the card enters the game (Burning Wish) the same handle names the object in hand, on the stack and so on. Cards in the sideboard are hidden from the opponent (hidden-info scenario `burning-wish-hidden-sideboard`).
2. **Burning Wish choice**: `{decision: choose_cards, purpose: reveal_from_sideboard, answer: ["p0:Name#1"]}`; an empty `answer: []` declines. `expect_options` lists card NAMES (`include`, `exclude`, `count`); it offers exactly the sorcery cards of the sideboard. When the sideboard holds no sorcery the player has no choice and no decision is raised (one scenario relies on that; the Show and Tell precedent).
3. **Bargain**: `{t: cast, card: "@bes", bargained: true, sacrifice: ["@petal"], pay: auto}`; `bargained: false` (or omitted) casts without the additional cost. The sacrificed permanent is a direct key `sacrifice: [...]` as in the addenda for other cost choices. `bargained: true` with an illegal sacrifice (a land, a nonartifact nontoken creature, an opponent's permanent) is `expect_illegal`.
4. **Beseech cast-free choice**: after `{decision: search, found: "@x"}` for a bargained Beseech whose found card has mana value 4 or less and is a spell: `{decision: yes_no, purpose: cast_exiled_card, answer: true|false}`, then for a targeted card `{decision: choose_target, slot: 0, answer: {player: p1}}` (or an alias). No decision is raised when the card cannot be cast (mana value 5 or more, a land, or the player cannot cast spells), or when the spell was not bargained. A free-cast X spell (Prismatic Ending) raises no `choose_number` (CR 107.3b: X is 0).
5. **Random discard**: Gamble scenarios script `random: [{kind: random_discard, player: p0, result: "@bolt"}, {kind: shuffle, player: p0, result: [...]}]`. The queue order is: discard, then shuffle (the card text order). Where the discard is forced or irrelevant the scenario uses `random: any`; `random: []` is used for "nothing random may happen" (countered Gamble, countered Echo).
6. **Targeting a copy on the stack**: `targets: [{copy_of: "@ew", nth: 0}]` (copies of the spell with that card alias, counted from the top of the stack, 0-based; used once, Force of Will on a storm copy).
7. **Control different from owner** in setup: an entry in a seat's `battlefield` with `controller: pX` (the seat lists the OWNER). Used by two Boomerang Basics scenarios (owner versus controller decides where the card goes and who draws). Documented in the scenario notes.
8. Existing spellings reused: `alt_cost: {flashback: true}`, `{exile_blue_card: "@bs", pay_life: 1}` (Force of Will), `{exile_blue_card: "@bs"}` (Force of Negation), Orim's Chant `kicked: false`, `yes_no` with `purpose: change_targets` for each storm copy (answer false unless the scenario retargets), `{trigger_of: "@tend"}` to target the storm trigger, `cards_drawn_this_turn`, `copy: true` stack entries.

## Scenario map

Burning Wish (7): `fetches-sideboard-sorcery-and-exiles-itself` (basic: hand, exile, library untouched); `offers-only-sorcery-cards` (four sorceries incl. the Lesson sorcery offered, instants/creature/land/enchantment/artifact not); `no-sorcery-in-sideboard-still-exiles`; `may-decline-and-still-exiles`; `countered-stays-in-graveyard` (Force of Will); `hidden-sideboard` (hidden-info, world_b); `fetched-tendrils-storm-counts-wish`.

Gamble (5): `fetch-then-random-discard-other-card` (scripted discard then scripted shuffle, found card not shuffled); `random-discard-can-hit-the-found-card`; `only-the-found-card-in-hand-is-discarded`; `empty-library-finds-nothing-still-discards`; `countered-does-nothing` (`random: []`).

Beseech the Mirror (12): `not-bargained-found-card-goes-to-hand`; `bargained-free-tendrils-storm-counts-beseech` (Ritual, Beseech then Tendrils free: two copies; cast during resolution); `bargained-mana-value-five-goes-to-hand` (Force of Will); `bargained-land-cannot-be-cast`; `bargained-may-decline-free-cast`; `bargained-free-bolt-countered-not-in-hand`; `bargained-prismatic-ending-x-zero-exiles-mana-value-zero` and `...-does-not-exile-mana-value-four` (X = 0 and zero colors spent); `bargained-gaeas-will-free-cast-then-recast-from-graveyard`; `bargain-needs-artifact-enchantment-or-token-you-control`; `found-card-stays-hidden` (hidden-info); `orims-chant-in-response-bars-the-free-cast`.

Tendrils of Agony (11): `storm-counts-spells-of-both-players`; `lands-and-abilities-do-not-count-for-storm`; `copies-are-not-cast-second-tendrils-counts-only-originals`; `copy-retargeted-to-self-original-unaffected`; `force-of-will-on-original-copies-still-resolve`; `consign-to-memory-counters-storm-trigger`; `veil-of-summer-hexproof-fizzles-original-and-copies`; `orims-chant-does-not-stop-storm-copies`; `force-of-negation-exiles-original-copies-resolve`; `storm-counts-countered-spells-and-the-counterspell`; `storm-kills-exactly-at-zero`.

Empty the Warrens (6): `storm-two-tokens-per-resolution` (six Goblins); `copies-not-cast-second-warrens-counts-originals` (ten); `force-of-will-counters-a-copy-only`; `veil-of-summer-draws-for-earlier-black-spell`; `veil-of-summer-no-draw-for-red-and-colorless-spells`; `orims-chant-does-not-stop-storm-copies`.

Peer into the Abyss (7): `self-library-thirty-life-twenty` (15 and 10); `odd-library-odd-life-round-up` (31 and 7: 16 and 4); `target-opponent`; `library-and-life-one-lethal`; `empty-library-draws-nothing-and-halves-life`; `countered-by-force-of-will`; `reads-library-size-on-resolution-not-on-cast` (Brainstorm in response).

Boomerang Basics (7): `own-permanent-returns-and-draws`; `opponents-permanent-returns-no-draw`; `cannot-target-lands`; `own-token-ceases-to-exist-and-draws`; `stolen-permanent-goes-to-owner-controller-draws`; `own-card-controlled-by-opponent-returns-no-draw`; `target-gone-in-response-fizzles-no-draw`.

Echo of Eons (8): `hard-cast-both-players-shuffle-and-draw-seven`; `flashback-exiles-and-does-not-shuffle-itself`; `opponent-short-library-draws-and-loses`; `empty-hand-and-graveyard-still-draws-seven`; `flashback-offered-only-with-two-generic-and-blue`; `flashback-needs-blue-mana`; `flashback-countered-still-exiled`; `hard-cast-countered-goes-to-graveyard`. All Echo scenarios assert counts and zones only, never which cards are drawn.

## Rulings and assumptions I was unsure about

1. **Countering the original storm spell does not stop the copies.** The storm trigger is its own stack object (702.40a). That the trigger still creates copies after the original is gone rests on last known information plus the well-known official storm ruling; I could not retrieve that ruling, so the source is `kind: ruling, verified: false` in `tendrils-force-of-will-on-original-copies-still-resolve` (the Force of Negation scenario relies on the same behaviour and cites no ruling).
2. **No scenario depends on a storm trigger with zero copies** being on the stack (every storm cast has at least one earlier spell), because I could not settle from the CR text whether an engine may skip it. By 702.40a it triggers and goes on the stack regardless; if you implement it, a `check` with `stack: [spell, triggered]` for storm 0 would be right.
3. **A free cast during Beseech's resolution** follows 608.2g: the cast spell goes on the stack and Beseech finishes resolving, so Beseech is in the graveyard (or, with Gaea's Will active, exile) before the free-cast spell resolves. The storm trigger of a free-cast Tendrils goes on the stack once Beseech has finished (the stack is `[Tendrils, storm trigger]` with p0 holding priority). Beseech counts as "cast before" for storm.
4. **Gaea's Will found by Beseech** (`beseech-bargained-gaeas-will-...`): a card with no mana cost cannot be hard cast, but "without paying its mana cost" may be applied to an unpayable cost (118.6a), and mana value 0 is within the limit. Gaea's Will itself is exiled when it resolves (its replacement effect exists by then), Beseech is in the graveyard from before and can be cast from it for its normal cost; the recast Beseech is exiled instead of going back. This does not touch the suspend simplification of the brief.
5. **Prismatic Ending via Beseech**: mana value 1 in the library (X = 0 off the stack, 202.3e); X is 0 (107.3b); converge counts 0 colors for a free cast, so it exiles only a mana value 0 permanent. The expectation follows from the Oracle conditional text plus those rules.
6. **Gamble with an empty library** still discards (the instructions are independent); asserted with two identical Islands so randomness does not matter.
7. **Burning Wish with no sorcery** raises no decision and exiles itself; with a sorcery available the choice is "may" (declinable). Burning Wish's fetched card is "revealed" but I assert no reveal event (no key for it); the hidden-info scenario only asserts that the other sideboard cards never appear in the opponent's observation.
8. **Veil of Summer's draw condition** looks at spells an opponent cast earlier in the turn, not only the one on the stack (Warrens scenarios use Dark Ritual as the black spell and Lotus Petal as the colorless one).
9. **Boomerang Basics with owner not equal controller**: the setup spelling `controller:` on an entry in the owner's `battlefield` list is my own use of the existing entry key. If your runner reads it differently, drop those two scenarios (`stolen-permanent-...`, `own-card-controlled-by-opponent-...`); the other five Boomerang scenarios do not need it.
10. **Priority flow assumed**: after a spell or trigger resolves the active player receives priority; the caster holds priority after casting and each `p0 pass, p1 pass` pair resolves the top object (same as the Flusterstorm/Orim's Chant precedents). Where a storm trigger is on the stack and the opponent responds, the opponent casts after p0's first pass.
11. `spells_cast_this_turn` is read per seat and does not count copies.
12. Mana: where the payment split matters (Ritual's floating BBB plus lands), `pay: {tap: [...]}` fixes the Swamp used for the first spell; the rest is `pay: auto`, with totals that only work out one way (the pool must be spent). Tapped-state assertions are limited to scenarios where every land is used.

## Things I could not express

- The sideboard contents and the "this card left the sideboard" fact (no assertion key); the Burning Wish scenarios assert the hand/exile/graveyard result only.
- Face-down exile and "revealed" (a reveal event has no spelling; `events_contain` kinds are untested).
- The count of storm copies other than by asserting the stack (`copy: true` entries) and the final life/token totals.
