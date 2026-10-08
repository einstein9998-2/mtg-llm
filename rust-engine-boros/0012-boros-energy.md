# RFC 0012: Boros Energy (Static Prison, Forth Eorlingas!, Samut, Sunbaked Canyon, Mindbreak Trap)

Status: applied to the live engine at Brady's instruction ("add the missing cards for boros", 2026-10-08); independent reviewers pending
Author: "Sideboard plans vs Alurentell" thread  Reviewers (two independent, one a rules reviewer): pending

## Problem
Brady replaced the Boros list with his Boros Energy 75 (`decks/boros-energy-v2.txt`). Five cards in the main deck or sideboard were not defined: Static Prison, Forth Eorlingas!, Samut, Hazoret's Champion, Sunbaked Canyon (main) and Mindbreak Trap (side, 2). Hexing Squelcher was already defined (Storm package). Without them the list could not be played in the engine and the sideboard plans against it could not be tested.

## Proposed change
No `mtg-core` change. Card data only, `crates/mtg-cards/cards/boros.cards.ron` (fifth card source in `legacy.rs`), built from existing IR:
1. **Static Prison**: enters trigger `ExileTargetUntilLeaves` on a nonland permanent an opponent controls (Portable Hole's mechanism), then `GainEnergy 2`; a trigger at the beginning of the controller's first main phase (`BeginStep Main1`) uses `MayElse`: pay one energy, otherwise sacrifice. With 0 energy there is no decision and the Prison is sacrificed.
2. **Forth Eorlingas!**: `{X}{R}{W}` sorcery, creates X Human Knight tokens (2/2 red, trample, haste; new token definition, subtypes Human and Knight already exist).
3. **Samut, Hazoret's Champion**: 2/2, static `GrantKeywords(HASTE)` to creatures its controller controls (layer 6, itself included).
4. **Sunbaked Canyon**: two mana abilities (R, W) each costing `{T}` and 1 life, plus `{1},{T}, sacrifice: draw a card`.
5. **Mindbreak Trap**: `{2}{U}{U}` instant; alternative cost `three_spells` (free if an opponent cast three or more spells this turn, `SpellsCastThisTurn(Opp) >= 3`); exiles up to four target spells.

Test-only adapter change (`mtg-spec`, `runner.rs`, `expect.rs`): a card's two mana abilities that share a cost shape (Sunbaked Canyon's R and W) are treated as one Oracle ability ("{T}, pay 1 life: add {R} or {W}"), so scenarios address it as one ability and pick the colour with `choose_color`. No effect on older scenarios.

## Documented limits (not modelled)
- **Monarch**: Forth Eorlingas! also makes you the monarch if you are not, and the monarch draws at the end step and loses the crown to combat damage. There is no monarch state in the engine; adding it needs a new state field and a view encoding change. Forth Eorlingas! here only makes tokens. This understates Boros (no card draw engine); treat Boros results as a floor for this card.
- **Mindbreak Trap targets**: "any number of target spells" is modelled as up to four optional distinct spell target slots (not itself). More than four spells on the stack at once is rare in this pool.
- **Mindbreak Trap subtype** (Trap) is not defined (no subtype slot spent; no card in the pool cares).
- **Samut** Oracle text and cost come from web sources; the card is not in the Savecraft database. Text: "Creatures you control have haste." Mana cost {1}{R} (one source). Re-check if a reviewer can reach a better source.
- Static Prison's payment is a yes/no choice (one energy), not a number.

## Rules basis
Oracle text from Savecraft card search (Scryfall) 2026-10-08 for the four cards it holds; Samut as above. CR 610.3 (exile until leaves), 614/611 continuous effects, 118.3 and 119.4 (payment limits), 702.40 and 608.2h for the storm/Trap interaction scenarios, 601.2/118.9 alternative costs. Writers' verified CR excerpts: `cr-excerpts-boros-A.md`, `cr-excerpts-boros-B.md`.

## Impact
- State layout and hashes: no new state field, no IR change. The card database hash and `n_defs` change because cards were added: nets and game records made against the previous live pool need regenerating (as after RFC 0007/0008/0009).
- Determinism and golden replays: test-pool and real-pool goldens replay bit-identically (`cargo test` passes unchanged).
- Hidden information: none new.
- `ENGINE_CORE_VERSION` bump: no.

## Test plan and results
Scenarios were written by two separate agents from Oracle text and the CR only (the implementer did not write or see them first). Defects found in them (an inconsistent stack check in one Trap scenario) were sent back to their writer.
- 48 Boros scenarios (`rust-engine-boros/scenarios/`): group A 24 (Static Prison 12, Sunbaked Canyon 7, Samut 5), group B 24 (Forth Eorlingas! 8, Mindbreak Trap 16). All pass. Four Canyon scenarios first failed because of the separate R/W abilities; fixed in the test adapter.
- Visible spec plus the Chant, Tron, Storm, UR and Boros packages: 1280 / 1280.
- `cargo test --release --workspace`: all pass, goldens bit-identical.
- Fuzz (`legacyfuzz`, invariants and fork/replay checks at every decision, deep checks every 100): 3000 games over `alurentell`, `boros-energy-v2`, `ur-cutter`, `ur-cutter-alt`, `uwx-control`, `dimir-tempo`: 0 violations, every deck card cast or entered.
- Not run: the sealed holdout and Forge differential testing.
