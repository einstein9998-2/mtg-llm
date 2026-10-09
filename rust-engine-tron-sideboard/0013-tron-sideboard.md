# RFC 0013: Colorless Tron sideboard (Eldrazi Confluence, Argentum Masticore, Mycosynth Lattice, Summon: Bahamut)

Status: proposed (this PR). Not applied to the live engine.
Author: Tron sideboard thread. Scenarios: two separate agents wrote them from Oracle text and the CR only (notes in `SCENARIO-NOTES-A.md`, `-B.md`).

## Why
Brady's Colorless Tron has four sideboard cards the engine did not define, so the deck loaded with an 11-card sideboard (RFC 0008 listed them as documented limits). With this change `colorless-tron.txt` loads as 60 + 15.

## Changes (`tron-sideboard-core.patch`, `patch -p1` inside `rust-engine/`, 17 files)
Card data, `tron-sideboard.cards.ron` (second source after `tron.cards.ron`):
- **Eldrazi Confluence** `{2}{C}{C}` instant. "Choose three, you may choose the same mode more than once" is modeled as ten modes, one per multiset of three of the printed modes, each with one target slot per +3/-3 or exile-and-return instance (700.2d), resolving in printed order. The rules outcome is exact; only the shape of the choice differs (one mode pick from ten instead of three picks). No core change.
- **Summon: Bahamut** `{9}` Enchantment Creature, Saga Dragon, 9/9 flying. Uses the Saga support from RFC 0007 with `last: 4`. New `Expr::SumCmc(filter)` (total mana value of the matching permanents) for chapter IV.
- **Mycosynth Lattice** `{6}`. "All permanents are artifacts" is a layer-4 static `AddTypes`. The other two lines are new `Restriction::ColorlessEverything` and `Restriction::ManaAsAnyColor`, read through a cache in `State::lattice` (recomputed with the derived characteristics, not hashed): `chars()` returns no colors for every object while the first is on, and the payment planner (`pay_plan_lat`) lets any source pay a colored symbol and floating {C} pay a colored symbol. A {C} symbol still needs colorless mana.
- **Argentum Masticore** `{5}` 5/5 first strike. New keyword `PROT_MULTICOLORED` (702.16b targeting in `can_target_from`, 702.16e damage in `commit_damage`, 702.16f blocking in `can_block_pair`). Enchanting and equipping by multicolored permanents is not modeled. Upkeep "sacrifice unless you discard": `MayElse` (discard when able, else sacrifice) around the new `Effect::DiscardReflexive`, which asks for the card (new `CardsPurpose::DiscardReflexive`), discards it and queues the reflexive trigger with the discarded card's mana value as its captured amount; the trigger's target filter uses `cmc_x_max`, and `stabilize` now sets `cur_x` from the captured amount while it collects that trigger's targets.

Other fixes:
- Automatic payment treated every creature that entered this turn as unable to make mana. Summoning sickness (302.6) only stops abilities with {T} in the cost, so Eldrazi Scion and Spawn tokens made this turn could not be sacrificed for mana in auto-pay. Found by a Confluence scenario. (This changes which plans auto-pay finds in games with those tokens; goldens replay bit-identically.)
- Test adapter: `choose_target` with `answer: null` declines an "up to" target.

## Not modeled / documented limits
- Confluence: the ten-mode representation above.
- Masticore: no protection from multicolored for enchanting or equipping.
- Lattice: the colorless rule applies to cards in all zones through `chars()`; last-known information of a permanent that left is whatever it had when it left. Layer 5 is folded into the end of the layer pass (no separate layer in the engine).
- Not in this change: Karn -2, Ugin -11, Extinguisher Battleship station, Mishra's Research Desk unearth (RFC 0008 limits; Urza's Saga came with RFC 0007).

## Impact
- The card database hash and `n_defs` change (four new definitions): nets and game records made against the previous live pool need regeneration, as after RFCs 0008, 0007 and 0009.
- `ENGINE_CORE_VERSION` is unchanged; `State::lattice` is a cache and not part of the rules hash; the goldens replay bit-identically.
- Hashed card-definition types gain variants (`Expr::SumCmc`, `Effect::DiscardReflexive`, two `Restriction`s, one `Keywords` bit); existing cards do not use them.

## Results
See `README.md`.
