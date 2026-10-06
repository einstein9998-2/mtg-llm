# RFC 0007: Legacy Storm (Wish, Imprint, Sagas, suspend and the effects Tron shares)

Status: implemented on a scratch copy of the live engine, NOT applied to `/mnt/project-files/rust-engine`; waiting for Brady's go-ahead and independent review
Author: Storm thread  Reviewers (two independent, one a rules reviewer): pending

## Problem
Brady's own Beseech Storm 75 (`decks/storm.txt`, deck 9) is an Alurentell opponent. 18 of its 30 distinct cards were missing from the engine (`decks/storm-engine-gap.md`). Nineteen new definitions (17 cards, one token, Taiga) fix that. Eight things in them need core support, and four of those are shared with the Colorless Tron list (RFC 0008, which defers them to this RFC: Urza's Saga, Wish for Karn -2, free cast of an exiled card for Ugin -11, the activation condition on `ActivatedDef`).

Oracle text came from Savecraft card search (Scryfall data). Giant's Boulder (Hobbit set) is not in Savecraft; its text came from three store pages.

## Proposed change
Cheap, local items first.

1. **Subtypes** `Saga`, `Lesson`, `Sorcerer` appended to `SUBTYPE_NAMES` (`types.rs`). RFC 0008 appends its own names, including `Saga`, first; when both are applied keep one `Saga` and put `Lesson`, `Sorcerer` after Tron's names. `CounterKind::Lore` appended.
2. **`Effect::DiscardRandom { who, n }`** (Gamble). A new `RandKind::DiscardAtRandom` so scenarios can script which card goes.
3. **`Effect::SearchNoShuffle { who, filter, dest }`** (Gamble, Beseech the Mirror: the shuffle is a separate instruction). A search that puts a card into exile hides it from the opponent: `MoveOpts::face_down_to` makes the move event carry a view id for the searcher only. The found card is left in `moved`.
4. **`Effect::Wish { filter }`** (Burning Wish; Karn -2 in RFC 0008). The controller picks a card of the filter from their sideboard (`DecisionKind::ChooseSideboard`, shown to the view layer as a name choice). The card is put into the hand revealed to both players (`put_outside_card_in_hand`). `PlayerSetup.sideboard` and `State::new` carry the sideboards; `fork.rs` already clears the opponent's.
5. **Free cast from exile**: `Effect::CastMovedFree { max_cmc }` (Beseech the Mirror: "if bargained, you may cast it if its mana value is 4 or less") and `Effect::CastFree(ORef)` (suspend; Ugin -11 in RFC 0008). Both put a `CastFrame` on top of the resolving frame with the free way (`restrict::FREE_WAY`). `CastMovedFree` asks the "may" question; `CastFree` always casts when it can (see Limits). Orim's Chant stops both through `cast_restricted`.
6. **`ActivatedDef.cond: Option<Cond>`** ("Activate only if ..."): checked where abilities are offered and where mana sources are collected (Mox Opal "metalcraft", Urza's Saga's gained abilities). The field is hashed only when present, so the card-database hash of every pool that does not use it does not move.
7. **Imprint** (Chrome Mox): `Effect::Imprint { filter }` exiles a card from hand into a `Link` that remembers its colors (`Link.colors`, hashed only when non-zero). `Effect::AddManaImprinted` adds one mana of any imprinted color. With nothing imprinted the tap is still offered and adds no mana (the Oracle text names no fallback; CR 106.5 by analogy). `CardsPurpose::Imprint` is the decision. Because it is a triggered ability it can be Stifled.
8. **Sagas**: `AbilityDef::Saga { last }`. A Saga enters with a lore counter (CR 714.3a, `commit_move`) and gets another after the draw step (`turn.rs`, main phase 1 of the controller). `EventPat::Chapter(n)` triggers when the counter count crosses `n` (714.2b). State-based actions sacrifice a Saga with at least `last` counters when no chapter ability of it is on the stack (714.4, `sba.rs`). Urza's Saga's "gains" chapters are modelled as marker counters `Other(0)` and `Other(1)` that gate the gained abilities through `ActivatedDef.cond`.
9. **Ward as data** (Hexing Squelcher): `Effect::CounterEventUnlessLife { n }` with the existing `BecomesTarget(by: Opp)` pattern, plus `EventPat::OtherBecomesTarget { by, filter }` for "other creatures you control have ward". `AbilityDef::SpellsUncounterable` is a static checked in `can_be_countered` ("this spell can't be countered" for the player's spells).
10. **Gaea's Will**: `PlayerFx::PlayFromGraveyard` (lands, spells, graveyard-zone activated abilities such as Runehorn Hellkite's) and `PlayerFx::ExileInsteadOfGraveyard` (hook in `commit_move`), both appended. `CostItem::SuspendSelf(n)` and `CostItem::ExileSelf` are appended. Suspend is an activated ability from the hand with sorcery timing; once its cost is paid it is removed from the stack instead of resolving, so it is a special action in effect (CR 116.2f; nothing for the opponent to respond to). `EventPat::LastCounterRemoved(kind)` is the second suspend trigger ("when the last is removed, cast it"). Suspend is offered only when the card could be cast (702.62a, Orim's Chant).
11. **Song of Creation**: `AbilityDef::ExtraLandDrop`, counted by `land_drops()`.
12. **Storm copies of a spell that left the stack** (Tendrils of Agony countered while its storm trigger is on the stack, CR 702.40a): the stack entry of such a spell is kept in a new `State::spell_lki` until no trigger of it is on the stack, and `CopySpell` reads it. The field is hashed only when non-empty.
13. `Effect::CopySpell` for `ORef::This` now uses the raw source reference (the spell may be gone). `Effect::Exile`/`MoveTo` read their objects before clearing `moved` so that `Moved(0)` works.
14. **Derived expressions**: `static_expr` accepts `Expr::Count` over the battlefield (Construct token: "gets +1/+1 for each artifact you control", counted from characteristics derived so far).
15. **Test adapter** (`mtg-spec`, not core): `pay: {tap: [...]}` now also applies to activations and to `suspend`; named sources are strict for suspend; scenario `subtypes: ["Urza's Saga"]` is aliased to `Saga`; many descriptor and action spellings used by the Storm scenarios (see the README).

Card data: `crates/mtg-cards/cards/storm.cards.ron`, a second source for `legacy::build()` (the same way RFC 0008 adds `tron.cards.ron`). `legacy.cards.ron` and the pinned real-pool snapshot are untouched.

## Rules basis
CR 106.5, 116.2f, 118.6, 601.2, 614, 702.40 (storm), 702.62 (suspend), 702.21 (ward by analogy for the life variant), 714 (Sagas), 701.23e (a search without a reveal instruction reveals nothing). Quotes are in `cr-excerpts-storm-A/B/C.md`. Rulings that were recalled and not retrieved are marked `verified: false` in the scenarios.

## Impact
- State layout and hashes: no existing field changes. New: `State::spell_lki` (hashed only when non-empty), `Link.colors` (hashed only when non-zero), `ActivatedDef.cond` (hashed only when set). New enum cases are appended everywhere (`Effect`, `AbilityDef`, `EventPat`, `PlayerFx`, `CostItem`, `DecisionKind`, `CardsPurpose`, `RandKind`, `CounterKind`), so discriminants of existing cases do not change.
- Goldens: test-pool and real-pool goldens replay bit-identically with the patch; no `ENGINE_CORE_VERSION` bump needed. The card database hash and `n_defs` of the live pool change because cards were added, so nets trained earlier need the new pool and game records from older versions must be regenerated (as for any card addition).
- Hidden information: a Beseech-found card is exiled with a view id for the searcher only (`MoveOpts::face_down_to`), later moves of it show as hidden moves to the opponent; a Wished card is revealed to both. The `noninterference` and fuzz hidden-information checks pass.
- Performance: not measured. The new per-move and per-turn checks are behind `CardDb` flags (`has_saga`, `has_extra_land`, `has_uncounterable_static`), so games without these cards should skip them.

## Documented limits (simplifications)
- Beseech the Mirror's bargain is modelled as an alternative way to cast it (key "bargained"), but in the rules it is an additional cost (CR 702.166a). A Beseech that is cast for free (found by another Beseech) therefore can never be bargained, which removes a real chain line (the same shape as the kicker limit of RFC 0005).
- Urza's Saga lacks the "Urza's" land subtype, and chapter III uses `cmc <= 1` instead of "mana cost {0} or {1}" (identical for this library). The Construct token's +1/+1 is a layer 7a set rather than 7c (same value unless another power/toughness effect applies). "Discard your hand" is discard 99.
- In a search world the opponent's Burning Wish finds no card (the fork forgets the sideboard), so a searching bot undervalues a Storm opponent's Wish line.
- Giant's Boulder's Oracle text is from three store pages (not in Savecraft); Brady confirmed it is the card's text (2026-10-06).
- The free cast from suspend cannot be declined (the card has no targets, and a "may" would cost a decision per game). Beseech's free cast keeps its "may".
- Suspend is "an activated ability that skips the stack", not a separate action kind. Views show it as an activation.
- Urza's Saga's gained abilities are marker counters, so a copy or a "loses all abilities" effect would not interact correctly (no such effect in the pool).
- Chrome Mox with a colorless imprint taps for no mana; the one scenario for that uses Kozilek's Command, which only exists once RFC 0008 is applied.
- Hexing Squelcher's ward is two triggers (on itself, on others) rather than a keyword.
- Gaea's Will lets the player play lands from the graveyard, but "play" with a land drop obeys the normal land-drop count (Song of Creation raises it).
- The ability `activate`/`mana` indexes in scenarios count activated abilities in definition order; an intrinsic land mana ability is ability 0 (engine id 255).

## Test plan and results (scratch copy with the patch)
- 163 Storm scenarios written by three separate agents (A: Beseech, Gamble, Wish, Tendrils and Empty the Warrens; B: Chrome Mox, Mox Opal, Boulder, Urza's Saga, Mite, Taiga, Hellkite; C: Gaea's Will, Squelcher, Song of Creation). The implementer did not write or edit them: 162 pass, 1 is unsupported until RFC 0008 is applied (Kozilek's Command). Three scenario defects found by the implementer were fixed by a separate agent (below); before that fix the count was 159 pass and 3 fail.
- Visible spec: 838 / 838, unchanged.
- `cargo test --release --workspace --no-fail-fast`: all green, including both golden suites.
- Random fuzz over the eight decks plus Storm: see README for the run and its counts.

### The three scenarios that were wrong (fixed)
The implementer found three failures that were defects in the scenarios, and an independent reviewer checked each against the CR (PR 6 review). A separate agent (not the implementer) then corrected the scenario files:
1. `storm-b-urzas-saga-chapter-three-trigger-countered-saga-still-sacrificed`: p1 draws on turns 4 and 6 (CR 504.1), so the hand ends as two Islands, not empty.
2. `storm-b-urzas-saga-wasteland-in-response-to-chapter-one-no-mana-ever`: once p1 has passed and p0 passes, the top of the stack resolves (117.4) and p0 gets priority (117.3b); the extra `p1: pass` was removed.
3. `storm-b-runehorn-hellkite-exiled-as-cost-no-graveyard-hate-response`: same priority pattern; the trailing `p0: pass` was removed.

## Alternatives considered
- Suspend as a new `Opt` kind (a real special action). It is cleaner in the views but changes the option enum every consumer matches on; the stackless activation behaves the same in every scenario.
- Imprint as a replacement effect "as it enters". Rejected: the Oracle text is a triggered ability and the scenarios require that it can be Stifled.
- Putting the Storm cards in `legacy.cards.ron`. That would move the pinned real-pool snapshot; a second source does not.
