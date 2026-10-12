# Emrakul, the Aeons Torn (engine work for the Alurentell experiments)

Developed on the local merged engine tree (storm, tron, ur, boros cards together), not on this repo's `main`, so it is delivered as files and not applied here.

- `emrakul.cards.ron`: the card (flying, uncounterable, protection from colored spells, cast trigger extra turn, annihilator 6, graveyard shuffle from hand/battlefield/library). Registered last in `legacy.rs` so no CardDefId shifts (nets need `netremap`: `nets-emr/` = 238 defs).
- `emrakul_load.rs`: load test.
- `emrakul-engine-files.tgz`: the changed source files of that tree for reference: `mtg-core` `types.rs` (keyword `PROT_COLORED_SPELLS`), `ops.rs` (damage prevention), `legal.rs` (targeting; see the fix below), `ir.rs`/`resolve.rs`/`lint.rs` (`Effect::ExtraTurn`), `mtg-cards` `legacy.rs`, and `own.yaml` (scenarios `own-emrakul-*`).
- Fix found by the simulations: a colored spell must not be offered when its only target is Emrakul. The cast-time check ran with the card still in hand, so `PROT_COLORED_SPELLS` was ignored and `ChooseTarget` later had zero options (a panic in the search for 2 to 3% of UW games). `legal.rs` now treats a castable card in hand, graveyard, exile or library like a spell on the stack (`is_castable_card`).
