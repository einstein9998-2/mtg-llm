# Storm (Beseech Storm, Brady's 75) vs the Rust engine: what is missing

Prepared 2026-10-06 for `decks/storm.txt`. Engine checked: `/mnt/project-files/rust-engine/` at core-frozen-m5, `crates/mtg-cards/cards/legacy.cards.ron`. Oracle text is from the Savecraft card search (Scryfall data); Giant's Boulder is not in that database, so its text comes from three store/price pages that agree (echomtg, wizardtower, tcgsync). Classification is from reading `ir.rs` and the RON file, not from running anything: the implementer must confirm each row.

**Verified names:** all 30 distinct cards resolve (24 in the main deck, 6 more only in the sideboard). The engine has 12 of them (Badlands, Underground Sea, Scalding Tarn, Verdant Catacombs, Raucous Theater, Bloodstained Mire, Lotus Petal, Lion's Eye Diamond, Dark Ritual, Veil of Summer, Thoughtseize, Boseiju) and lacks 18.

Storm already works (Flusterstorm: `CopySpell` with `EventAmount`), so Tendrils and Empty the Warrens are easy.

## Data only, probably (10, plus the search half of Gamble)

| Card | Where | Oracle (Scryfall) and notes |
|---|---|---|
| Taiga | main 1 | Plain Mountain Forest dual. |
| Tendrils of Agony | main 1, side 1 | Target player loses 2, you gain 2, storm. |
| Empty the Warrens | side 1 | Two 1/1 red Goblin tokens (`Goblin Token` exists), storm. |
| Peer into the Abyss | side 1 | Target player draws half their library (round up) and loses half their life (round up). `HalfUp`, `CardsInLibrary`, `Life` exist. |
| Boomerang Basics | side 4 | Sorcery, Lesson. Bounce target nonland permanent; if you controlled it, draw. `Cond::ControlledBy` exists. Lesson is not in the closed subtype table (types.rs). |
| Haywire Mite | side 1 | 1/1 Insect artifact creature; dies: gain 2; {G}, sacrifice: exile target noncreature artifact or enchantment. Insect exists. |
| Mox Opal | main 4 | Legendary artifact, metalcraft: {T}: add one mana of any color, only with three or more artifacts. `ActivatedDef` has no activation condition; use a `Restrict::CantActivate` with a cond, or an `If` that wastes the tap. The legend rule exists (`sba.rs`). |
| Runehorn Hellkite | main 1 | {5}{R} 5/5 flying Dragon. {5}{R}, exile this card from your graveyard: each player discards their hand, then draws seven. Needs an activated ability from the graveyard that exiles itself as a cost: `ActivatedDef.zone` exists, a self-exile cost has not been confirmed. |
| Echo of Eons | main 3, side 1 | {4}{U}{U} sorcery: each player shuffles their hand and graveyard into their library, then draws seven. Flashback {2}{U}. Alt cost with `zone: Graveyard` exists (Faithless Looting does it). `MoveTo` into the library has not been seen in the RON; confirm. |
| Giant's Boulder | main 4 | {1} artifact. ETB scry 2 (`Scry` exists). {1},{T}: add one mana of any color. {7},{T}, sacrifice: destroy target permanent. No mana ability with a mana cost exists in the RON yet, so confirm that a `Mana` cost is allowed inside `is_mana: true`. |
| Gamble | main 4 | {R} sorcery: search for a card, put it in hand, discard a card at random, shuffle. `Discard` is the player's choice, there is no random discard (see core table). |

Gamble is in this table for its search; the random discard is the core item below.

## Needs core, so an RFC for Brady's sign-off (8 items, 7 cards plus Gamble's discard)

| Card | Where | What is new |
|---|---|---|
| Burning Wish | main 4 | Reveal a sorcery card you own from outside the game, put it into your hand, exile Wish. `PlayerState.sideboard` exists and `fork.rs` already hides the opponent's, so this is one new effect (`Wish { filter }`) plus exile-self. The revealed card is public. Sorcery targets in this sideboard: Beseech the Mirror, Echo of Eons, Tendrils, Empty the Warrens, Peer into the Abyss, Boomerang Basics, Thoughtseize. |
| Gamble (random discard) | main 4 | A discard effect that picks from the hand with the engine RNG (state must stay deterministic for goldens and forks). One small new effect. |
| Beseech the Mirror | main 3, side 1 | {1}{B}{B}{B} sorcery, bargain (sacrifice an artifact, enchantment or token as you cast it; optional `AddCost` exists). Search for a card, exile it face down, shuffle; if bargained, you may cast it free if its mana value is 4 or less, otherwise put it in hand. New: a "cast the exiled card free" effect and a condition for "was bargained". |
| Gaea's Will | main 1 | No mana cost: castable only by suspend 4 ({G}) or free casts (Beseech). Needs suspend (time counters exist, no suspend), a turn-long "play lands and cast spells from your graveyard" permission, and "cards going to your graveyard this turn are exiled instead" (`replace.rs` only has enters replacements). The largest item. |
| Chrome Mox | main 4 | Imprint: as it enters, exile a nonartifact, nonland card from your hand; tap for a color of the exiled card. New: linked-exile mana. |
| Urza's Saga | main 4 | Saga rules (lore counter on entering and after the draw step, chapter triggers, sacrifice after chapter III), chapters I and II grant activated abilities ({T}: add {C}; {2},{T}: create a 0/0 Construct with +1/+1 per artifact you control), chapter III searches for an artifact with mana value 0 or 1 and puts it onto the battlefield. `CounterKind::Other(u8)` can hold lore counters; chapter triggers and the sacrifice SBA are new. Saga is not a subtype in the table; Construct is. |
| Hexing Squelcher | main 4 | 2/2 Goblin Sorcerer for {1}{R}. Can't be countered (flag exists). "Spells you control can't be countered" as a static (only a turn-limited `PlayerFx::SpellsUncounterable` exists), ward-pay-2-life on itself and on your other creatures (`CounterEventUnless` only takes mana, and granting a triggered ability is new). Sorcerer is not in the subtype table. |
| Song of Creation | main 1 | {1}{G}{U}{R} enchantment. Whenever you cast a spell, draw two (easy); at your end step, discard your hand (easy); you may play an additional land each turn: the land-drop limit is hard-coded (`lands_played < 1` in `priority.rs`), so a land-drop allowance is new core. |

Song of Creation (a one-of) and Gaea's Will (a one-of) are the natural candidates to ship as documented limits, the way E4 and E5 were; Hexing Squelcher is a four-of, so its ward and static need real support or a documented simplification.

## Suggested order

1. Brady signs off the list (done, he supplied it) and the approach.
2. RFC 0006 with the cheap core effects first: Wish, random discard, free cast from exile (Beseech), imprint mana (Chrome Mox). These are small and make the deck's shell playable.
3. Second RFC (or documented limits, Brady's call) for the large items: Saga, Gaea's Will, Hexing Squelcher, Song of Creation's extra land.
4. Data-only cards can be written now, after the subtypes (Lesson, Saga, Sorcerer) are decided, since the subtype table is core.
5. A different agent writes the spec scenarios from Oracle text and the CR only, as for Orim's Chant (RFC 0005).

The Storm sideboard plan for Alurentell is not written here; Brady and the coordinator own that.
