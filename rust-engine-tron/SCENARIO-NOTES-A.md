# Group A scenarios (Trinisphere, Grim Monolith, The One Ring, Voltaic Key, Manifold Key, Karn static and +1): notes for the implementer

42 scenarios in `scenarios/tron-A-*.yaml`, one per file, written only from the Oracle text in SCENARIO-BRIEF.md and the Comprehensive Rules (2025-11-14, retrieved through Savecraft `rules_search`). No engine source, patch, tron.cards.ron or gap document was read. Oracle texts for the six cards were re-retrieved with `card_search` and match the brief (Karn's full text including -2 is in `oracle-additions-A.json`; -2 is never used).

## Files and validation

- `oracle-additions-A.json`: six card entries (merge into `reference/oracle.json` `cards{}`).
- `cr-excerpts-tron-A.md`: verbatim CR text (copy into `reference/`). The first block was copied unchanged from existing excerpt files; the second block was retrieved fresh.
- Command (scratch copies of `reference/` with both additions merged, as in the Orim's Chant package):
  `python3 /mnt/project-files/rust-engine-spec/tools/validate.py --oracle <scratch>/oracle.json --refs <scratch refs dir> /mnt/project-files/rust-engine-tron/scenarios/tron-A-*.yaml`
  Result: `42 scenario(s), 0 error(s), 0 warning(s)`. (The other writers' files sit in the same folder; I validated only mine.)
- All CR citations are `verified: true`. Every Oracle source is verified. There is no ruling source: all expectations follow from Oracle text plus CR. No scenario uses Karn's starting loyalty (setups put loyalty 3, 4 or 5 explicitly, only changes are asserted).

## Scenario map

Trinisphere (12, the most important for Alurentell)
| file (tron-A-trinisphere-...) | what it pins |
|---|---|
| bolt-costs-three-two-lands-not-enough | {R} becomes {2}{R}; two lands not enough, third land allows it; opponent's Trinisphere taxes the other player |
| colored-symbol-still-required | three Plains cannot pay {2}{R}; after a Mountain the payment taps Mountain + two Plains |
| force-of-will-pitch-needs-three-mana-not-offered | pitch alt cost ({1} life + blue card) is not offered with two Islands, nor is the hard cast |
| force-of-will-pitch-plus-three-mana-counters | pitch cast with three Islands pays life, exile AND three mana |
| daze-return-island-plus-three-mana | Daze alt cost (return Island) still needs {3} from other lands |
| aluren-free-cast-still-costs-three | Aluren "without paying" cast costs three (two lands: not offered) |
| omniscience-free-cast-is-generic-three | Omniscience free Bolt costs three GENERIC (three Plains pay it; no {R}) |
| prismatic-ending-x0-three-colors-exiles-trinisphere | X=0 costs {2}{W}; W+U+R spent counts three colors for converge and exiles a mana value 3 permanent |
| show-and-tell-puts-are-not-casts | Show and Tell costs exactly 3; put Atraxa and Lotus Petal cost nothing; opponent with no lands still puts Petal |
| voltaic-key-untaps-it-and-tax-returns | tapped Trinisphere is ignored; Voltaic Key untaps it and the tax returns |
| taxes-its-own-controller-monolith-chain | controller is taxed too; Monolith's CCC exactly pays a taxed Voltaic Key (pool empty afterwards) |
| does-not-tax-abilities-lands-or-cycling | activated ability, mana ability, land play and cycling are not spells |

Grim Monolith (5): taps-for-three-colorless; does-not-untap-in-untap-step; four-untap-uses-the-stack-and-nets-mana ({4} untap is not a mana ability, goes on the stack, payable with its own mana); put-by-show-and-tell-taps-at-once (no sickness for a noncreature artifact); colorless-mana-pays-trinisphere-generic (CCC pays generic only, not {R}).

The One Ring (9): cast-gives-protection-bolt-cannot-target; protection-ends-at-your-next-turn; not-cast-no-protection (Show and Tell: no trigger, Bolt legal); tap-draws-per-burden-counter (counter first, then draw 3 with 2->3); and-voltaic-key-draw-one-then-two; upkeep-lose-life-per-burden-counter (not at the opponent's upkeep); indestructible-survives-meltdown; legend-rule-indestructible-does-not-help (new Ring has no counters); protection-prevents-combat-damage-and-lifelink.

Voltaic Key (3, plus the Ring and Trinisphere and Monolith scenarios above): untaps-grim-monolith-for-net-mana; can-target-itself-but-not-tapped; targets-only-artifacts-any-controller.

Manifold Key (4): untaps-another-artifact-not-itself (and one ability per turn, both need {T}); unblockable-before-blocks-no-block-offered; after-blockers-does-not-unblock (509.1h); and-voltaic-key-untap-each-other.

Karn (9): static: opponent-artifact-abilities-cant-activate (mana abilities included, lands and own artifacts unaffected), opponent-aether-vial-tap-blocked-trigger-works, ends-when-karn-leaves; +1: grim-monolith-becomes-two-two-and-still-taps, new-artifact-is-summoning-sick-cannot-tap, up-to-one-zero-targets-once-per-turn-no-land, trinisphere-attacks-and-taps-off-the-tax, creature-is-bolt-target-on-opponents-turn, effect-ends-at-karns-next-turn.

## Spellings I used (the schema has none for these; the validator does not check them)

- Loyalty abilities: `{t: activate, source: "@karn", ability: N, targets: [...]}` with N = position among activated/loyalty abilities in Oracle order (+1 is 0, -2 would be 1; the static ability is not counted). The loyalty cost is paid on activation (4 -> 5 is visible while the ability is on the stack).
- Counters in setup and `counters:` assertions: `{burden: N}` (The One Ring), `{loyalty: N}`, `{charge: N}` (existing spelling).
- Free casts with a mana payment: `{t: cast, card: ..., alt_cost: {free: aluren|omniscience}, pay: {tap: [...]}}`. The runner must accept `pay` together with a free alt cost (the Trinisphere mana). If the adapter models it as a separate `pay_mana` decision, translate.
- X spells: `x: 0` on the cast (existing spelling).
- Floating mana is spent with `pay: auto` in two scenarios (taxes-its-own-controller-monolith-chain, grim-monolith-colorless-mana-pays-trinisphere-generic). In both the legal payment is unique, so no preference rule is needed.

## Schema gaps (things I could not assert directly; effects asserted instead)

- Protection from everything on a player has no assertion key: asserted through legality (Bolt at the player not offered, `expect_illegal`), prevented damage (life totals, lifelink) and the end of the effect (Bolt legal again).
- Colors of mana spent (converge) and the total cost paid are not assertable; the Prismatic Ending scenario asserts the exile outcome, the Trinisphere scenarios assert tapped lands and the pool.
- "Can't be blocked this turn" has no key: the scenario relies on `advance` failing if a `declare_blockers` decision is raised (a declaration with no legal blocker raises none), as in the Orim's Chant package.
- A "may" trigger of Aether Vial is written `yes_no` (existing shape).

## Rulings and assumptions I was unsure about

1. **Trinisphere with zero-cost casts.** "Each spell that would cost less than three mana costs three mana": for Aluren, Omniscience and a {0} spell the total cost is {0} (601.2f / 118.9d), so it becomes three mana. I believe the official Trinisphere rulings say the same but could not retrieve rulings (no egress); the expectation rests on the Oracle text, its reminder text and 601.2f/118.9d. Specifically, **"three generic mana, no colored symbol"** for the Omniscience Bolt (omniscience-free-cast-is-generic-three) is derived from the reminder text (only the symbols already in the cost stay), not from a ruling.
2. **Prismatic Ending X=0 spending three colors on the two extra mana.** Follows from the reminder text ("additional mana may be paid with any color") plus converge counting colors of mana actually spent; no CR text retrieved says the extra mana counts toward converge, so treat it as the least certain Trinisphere scenario (still `verified` only for the Oracle quotes).
3. **Daze alt cost and tapping the returned Island for mana first** (601.2g/h order) is deliberately not tested; the Daze scenario uses four Islands (three tapped, one returned).
4. **`advance` stopping with a trigger already on the stack.** Several scripts (the One Ring upkeep ones, Karn vs Aether Vial) use `advance: {to: upkeep, of: pN}` and then check the pending upkeep trigger on the stack. SCHEMA says advance fails if a pass would happen with a non-empty stack; I rely on it stopping at the moment the active player first has priority (trigger on the stack). If the runner reads it differently, replace by explicit passes.
5. **"Until your next turn" end moment.** The CR gives no exact text (611.2a only says "as long as stated"). Scenarios only assert at the opponent's main phase (still active) and at the controller's next upkeep (already ended); the beginning of the untap step itself is not asserted.
6. **Empty attack declaration** (`declare_attackers, attacks: []`) is used when a creature could attack but the scenario wants no attack (two Karn +1 scenarios).
7. **Karn static and cards outside the battlefield.** Only permanents are covered ("artifacts your opponents control"); no scenario for artifact cards in hand or graveyard.
8. The Ring legend-rule scenario assumes the new Ring's enters trigger (it was cast) still goes on the stack and resolves while the old Ring goes to the graveyard (legend rule is a state-based action performed before the trigger is put on the stack); it asserts only stack [] at the end and no protection effect.
9. `spells_cast_this_turn` is read per seat (as in earlier packages).
10. Vial's `yes_no` purpose label and the `legend_rule` decision keep spellings already used in the pool.
