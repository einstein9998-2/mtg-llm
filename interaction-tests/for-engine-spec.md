# Candidate scenarios for the Rust engine spec tests

Source: the Forge interaction tests (`README.md`, `FINDINGS.md`). These are rules facts the engine must get right that are easy to miss
when the enumerator and the rules code are separate pieces. Each names the scenario in `scenarios/` that already checks Forge on it,
so the expected result is the one Forge produced and the rules require.

## 1. Casting restrictions must filter the legal action list (found as a Forge enumerator gap)

Forge's `getAllPossibleAbilities(player, removeUnplayable=true)` still offered spells that a "can't cast" static forbids; only the real
play path (`checkRestrictions`) refuses them. The engine's legal-action enumeration must apply the same restrictions, otherwise a
player is offered illegal casts.

- `voice-of-victory-blocks-opponent-spells-on-my-turn` (22-boros-vs-bw-dnt): Voice of Victory on p1's side, p2 holds Solitude with
  five Plains during p1's turn. Solitude must not be castable, by hard cast or by evoke. Control: `voice-of-victory-allows-spells-on-opponents-turn`.
- `deafening-silence-one-noncreature-spell` (30): a second noncreature spell in one turn is not offered, for either player.
- `grafdigger-cage-stops-flashback` (30): Faithless Looting cannot be flashed back; `grafdigger-cage-stops-reanimate`: Reanimate still
  resolves and costs 8 life but Griselbrand stays in the graveyard.
- `containment-priest-exiles-reanimated-creature` (30): a creature put onto the battlefield without being cast is exiled; one that was
  cast (`containment-priest-allows-cast-creature`) is not.

## 2. Costs and choices the enumerator has to expand

- Escape with a collective restriction (`nethergoyf-escape`, `phlage-escape-stays`, `phlage-escape-needs-five-other-cards`): the exile set is
  chosen as a whole ("four or more card types among them"), so legality cannot be checked card by card. Nethergoyf can exile any number.
- Alternative costs offered as separate actions: Force of Will, Daze, Force of Negation (only on the opponent's turn: `force-of-negation-no-free-on-own-turn`),
  Snuff Out (needs a Swamp), Unmask, Solitude evoke, Bilbo's one-less on escape and flashback (`bilbo-escape-costs-one-less`).
- "You may cast it" effects have their own decision (`bilbo-attack-casts-instant-from-graveyard`).
- Library order choices are an explicit decision (`doomsday-pile-order`): five picks one at a time, then the order.

## 3. State-based and replacement interactions worth a holdout scenario

- Spider-Woman's enters-tapped applies to creatures put in by Aether Vial and to artifacts (`spider-woman-aether-vial-creature-enters-tapped`).
- Reanimate fizzles when the target is exiled in response, with no life loss (`reanimate-fizzles-to-surgical-extraction`, `faerie-macabre-fizzles-reanimate`).
- Thassa's Oracle wins at exact devotion and not above it (`thassa-oracle-wins-at-exact-devotion`, `thassa-oracle-no-win-with-large-library`).
- Phelia flickering a permanent resets it as a new object: counters gone, no counter for Phelia unless it returns under her controller
  (`phelia-resets-counters-on-flickered-card`, `phelia-own-permanent-gets-counter`).
- Cavern of Souls and Koma make the spell uncounterable even against a free Force of Will (`cavern-of-souls-makes-creature-uncounterable`, `koma-cannot-be-countered`).
- Seat order must not matter: Aluren in Forge is only usable by the first seat (a Forge bug, see `FINDINGS.md`). An engine test should run
  every symmetric scenario twice with the seats swapped (`swap-seats: yes` in the Forge harness).
