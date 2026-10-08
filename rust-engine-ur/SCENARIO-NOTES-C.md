# Group C scenarios (Beseech the Mirror: the failed search moves nothing to hand): notes for the implementer

3 scenarios, one file each, `scenarios/ur-C-<topic>.yaml`. Written from the Oracle text (Savecraft `card_search`, 2026-10-08) and the Comprehensive Rules (2025-11-14, Savecraft `rules_search`). No engine source, patch, `*.cards.ron` or gap document was read, and no `mcp__hearthbot__*` tool was called. Every source is `kind: oracle` or `kind: cr` with `verified: true`; no ruling is cited.

## The rule

Beseech the Mirror puts into hand only the card that its own search exiled ("Put the exiled card into your hand if it wasn't cast this way"; 400.7j: other parts of an effect find the object that effect moved). When the search finds no card, there is no exiled card and nothing at all goes to a hand. The engine once put an object produced by an EARLIER effect into hand in that case; these scenarios fail on such an engine.

## Two corrections to the brief that shaped the scenarios

1. **A search for "a card" cannot be failed while the library has cards.** CR 701.23d: searching simply for a quantity of cards ("a card") means the player must find that many (701.23b, the "may find nothing" permission, applies only to a stated quality such as a type or name). So "the player chooses to find nothing" is not a legal answer for Beseech with a nonempty library. The two failed-search scenarios therefore empty the library (no `library_pad`, `library:` omitted or exhausted by an earlier draw); the `search` decision is still raised and answered `found: null` (SCHEMA addenda: the search decision exists even when nothing can be found). If the engine instead lets a player answer `found: null` with cards in the library, that is a separate divergence; it is not tested here.
2. **Beseech the Mirror costs {1}{B}{B}{B}** (mana value 4), not {1}{B}{B}; Savecraft and `oracle-additions-A.json` of the Storm package agree. The scenarios pay four Swamps.

## Files and validation

- `oracle-additions-C.json`: `cards` (Beseech the Mirror, Warping Wail, Boomerang Basics, Tendrils of Agony; identical to the Storm A / Tron C entries) and `tokens` (`Eldrazi Scion Token`; text is the Scryfall token text "Sacrifice this creature: Add {C}.", the Tron package has "this token"; no scenario quotes it).
- `cr-excerpts-ur-C.md`: verbatim CR text for 101.3, 111.7, 400.7j, 701.23d, 701.24b, 702.166a/b, 704.5b.
- Validation (scratch copy of `reference/oracle.json` with my additions merged; my excerpts file as the only cr-excerpts in the refs dir):
  `python3 /mnt/project-files/rust-engine-spec/tools/validate.py --oracle <scratch>/refs/oracle.json --refs <scratch>/refs /mnt/project-files/rust-engine-ur/scenarios/ur-C-*.yaml` -> `3 scenario(s), 0 error(s), 0 warning(s)`.

## Scenario list

| file (ur-C-...) | asserts |
|---|---|
| beseech-failed-search-leaves-earlier-token | Warping Wail mode 3 (paid with Ancient Tomb: life 18) makes an Eldrazi Scion; then a BARGAINED Beseech (Lotus Petal sacrificed) with an empty library: search `found: null`, no `yes_no` free-cast prompt. Hand empty; the Scion is still a 1/1 on the battlefield; the Petal stays in the graveyard; nothing in exile; game not over. |
| beseech-failed-search-leaves-earlier-bounced-card-in-hand | Boomerang Basics bounces p0's own Lotus Petal and draws Ponder (library now empty); an unbargained Beseech then finds nothing. Hand is exactly Lotus Petal + Ponder (no duplicate, no extra), graveyard exactly the two sorceries, battlefield exactly the five lands, library 0, not a loss. |
| beseech-successful-search-still-puts-found-card-in-hand | Control: same setup as the token scenario but the library is Island, Tendrils of Agony, Island. Bargained Beseech finds Tendrils (mana value 4), p0 declines the free cast (`yes_no`, false): hand is exactly Tendrils, the Scion stays on the battlefield, library keeps two Islands, nothing in exile, Tendrils not cast (2 spells this turn). Fails an engine that "fixed" the bug by moving nothing to hand. |

## Decision shapes and flow used (all existing spellings)

- `{t: cast, card: "@ww", modes: [3], pay: {tap: ["@tomb"]}}` (Wail's third mode, as in the Tron package; Ancient Tomb's {C}{C} pays {1}{C} exactly).
- `{t: cast, card: "@bes", bargained: true, sacrifice: ["@petal"], pay: auto}` / `bargained: false` (Storm A spelling).
- `search` then, only when a card was found, `yes_no` with `purpose: cast_exiled_card`; no `yes_no` is raised for `found: null`, since there is no exiled card (nothing to cast). Per the SCHEMA the search decision is raised for a failed search and answered `found: null`.
- `pay: {tap: ["@isl"]}` for Boomerang Basics so the {U} cannot be paid with the Petal.

## Rulings and assumptions I was unsure about

1. **Shuffling an empty library** (the "then shuffle" instruction after a failed search): it is not impossible (701.24e treats shuffles of zero or one cards as shuffles), but it changes nothing; the scenarios use `random: any` and do not say whether the engine draws a random event for it. If the runner adapter requires `random: []` or a scripted `shuffle` event for an empty library, that is a runner convention, not a rules point.
2. **Whether the free-cast `yes_no` is raised after a failed bargained search**: the Storm package raises it only when the exiled card has mana value 4 or less and can be cast; with no exiled card there is nothing to ask, so I assume none (the token scenario would otherwise need an extra step). An engine that raises it with an empty exile should fail on the next step anyway.
3. **Ancient Tomb damage as part of the cast payment**: life 18 after paying Wail with the Tomb, as in the Tron package scenarios.
4. The Scion is summoning sick (created this turn); the scenarios assert neither sick nor tapped state for it.

## Things I could not express

- "Face-down exile" / "the library was shuffled" as events: the library emptiness is asserted through `library_count: 0` and the (not-)asserted shuffle only; there is no spelling for "a shuffle happened".
