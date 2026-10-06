# Colorless Tron vs the Rust engine: what is missing

Prepared 2026-10-06. Deck: `colorless-tron.txt` (Brady's own 75, 60 main / 15 side, not a tournament list). Engine checked: `/mnt/project-files/rust-engine/` at core-frozen-m5, `crates/mtg-cards/cards/legacy.cards.ron` (196 card names). All 32 distinct card names and Oracle texts come from the Savecraft card search (Scryfall data), none from memory, except the loyalty numbers (the search does not return loyalty): Tezzeret 4 and Ugin 7 are from scrydex.com card pages, **Karn 5 is not confirmed** (I could not load its page; 5 is the printed value as I recall it, so Brady or the implementer should check it).

Nothing in the live engine was changed and nothing is pushed to GitHub. A scratch copy (`rust-engine-tron/`) holds the card data that needs no new core code.

## Summary

- Already in the engine (5 of 32): Ancient Tomb, Karakas, Boseiju Who Endures, Disruptor Flute, Grafdigger's Cage.
- Missing: 27 cards. 9 are plain card data, 11 need one small core addition each, 7 are larger.
- The data parts of 15 cards (the 9 below, plus the data parts of Karn, Manifold Key, Kozilek's Command, The One Ring, Planar Nexus and Ugin) were written as RON and **load and validate** in a scratch copy of the engine (`CardDb::validate` passes). That checks syntax and lints, not behavior; the scenario writer still has to test them.
- The engine's subtype table needs 12 more names: Karn, Ugin, Tezzeret, Saga, Tower, Mine, Power-Plant, Spawn, Scion, Masticore, Spacecraft, Robot. It is an append-only table with 97 of 128 slots used, so this does not move any existing bit. The draft patch includes it. (Storm needs Saga too, plus Lesson and Sorcerer: 3 more, so 112 of 128 for both decks.)
- Tokens needed (data): Eldrazi Spawn, Eldrazi Scion, Construct (power and toughness equal to the artifacts you control), and Tezzeret's emblem.

## Data only (9)

| Card | Zone | Notes |
|---|---|---|
| Urza's Tower | main 4 | `If` on two `Controls` checks for a Mine and a Power-Plant subtype. |
| Urza's Workshop | main 4 | The Oracle text has two mana abilities. `ActivatedDef` has no activation condition, so the draft is one ability: add {C}, or with three or more artifacts add {C} for each Urza's land (`Repeat` over `Count`). The second ability always gives at least as much, so the dropped alternative is never better: no change in outcomes, one fewer option listed. |
| Expedition Map | main 1 | `Search` land to hand, `SacrificeSelf` cost. |
| Pithing Needle | main 1 | Same pieces as Disruptor Flute (`EntersChoice`, `Restrict CantActivate`). Needs a scenario that a named planeswalker's loyalty abilities are blocked too. |
| Voltaic Key | main 1 | `Untap`. |
| Tormod's Crypt | side 1 | `ExileGraveyard` with a player target. |
| Liquimetal Coating | side 1 | `Continuous AddTypes ARTIFACT` until end of turn on any permanent. |
| Warping Wail | side 3 | Three modes; "power or toughness 1 or less" is two filters joined with `alt`. Needs `{C}` pips in costs (`ManaCost` has a colorless pip slot and the draft loads). |
| Tezzeret, Cruel Captain | main 4 | Loyalty 4, artifact-enters trigger, 0 / -3 / -7. The -7 emblem fires at the beginning of combat from the command zone, with new subtype Robot. |

## Data plus one small core addition each (11)

| Card | Zone | What is new |
|---|---|---|
| Karn, the Great Creator | main 4 | Static and +1 are data (`Restrict CantActivate` for opposing artifacts, `SetPTX` with `CmcOf`). The -2 fetches an artifact from outside the game or from exile: needs the **Wish** effect (shared with Storm's Burning Wish, see below), plus a choice that also offers the face-up artifacts you own in exile. |
| Manifold Key | main 2 | First ability is data. "{3},{T}: target creature can't be blocked" needs an unblockable keyword (`Keywords` has 12 free bits; read in `combat.rs`). |
| Kozilek's Command | main 4 | Modes 1, 2, 4 are data (the draft restricts the two "target player" modes to yourself, since choosing the opponent is never better). "Exile target creature with mana value X **or less**": `ObjFilter.cmc_x` means "equals X" only, so it needs a "or less" form. Check that `x_count` plus `optional` gives "up to X targets" for the graveyard mode. |
| The One Ring | main 4 | Everything is data except "you gain protection from everything until your next turn": a new `PlayerFx` case (damage to the player prevented, the player can't be targeted), read at damage and at `legal_targets`. |
| Grim Monolith | main 4 | "Doesn't untap during your untap step": `turn.rs` untaps every permanent unconditionally. One keyword bit read in the untap loop. Tron's untap engine (Voltaic and Manifold Key, Tezzeret, {4} untap) needs this to be right. |
| Trinisphere | main 4 | A cost **floor** ("each spell that would cost less than three mana costs three"): `CostMod` only adds or subtracts generic mana. Must apply last, to every way of casting: Force of Will's pitch cost, Aluren and Omniscience free casts, Lotus Petal, Orim's Chant. This is the card that hurts Alurentell most, so the free-cast path needs its own scenarios. |
| Portable Hole | main 1 + side 1 | "Exile target nonland permanent an opponent controls with mana value 2 or less until this leaves": the engine has `until_links` (used by Cloak and Dagger) but the effect that fills it is specific to that card. One new effect, `ExileTargetUntilLeaves`. |
| Mishra's Research Desk | main 2 | Exile the top two, choose one, play only that one until the end of your next turn. `ExilePlay` lets you play every exiled card with no expiry. New effect `ExileChoosePlay`; the expiry would change the hashed state (`exile_plays`), so the draft proposes no expiry (the chosen card stays playable while it is exiled): slightly generous, documented. The Oracle's unearth ({1}{R}) is not modeled: it needs a leave-the-battlefield replacement the engine lacks, and red mana only comes from Planar Nexus. |
| Planar Nexus | main 4 | "Every nonbasic land type" is data (listing Desert, Urza's, Tower, Mine, Power-Plant, which are the only nonbasic land types any pool card can see). "{1},{T}: add one mana of any color" is a **mana ability with a mana cost** (a filter land); the payment planner has no such source. This matters: it is Tron's only way to make green (a second Boseiju already on the battlefield also taps for {G}) and, besides Karakas, white, so without it Boseiju's channel ability ({1}{G}) is almost never usable, and Boseiju is Tron's answer to Aluren. Without it, Nexus is a colorless land that is a Mine and a Power-Plant. |
| Torpor Orb | side 1 | "Creatures entering don't cause abilities to trigger": a new `Restriction` read where enter triggers are collected (`trigger.rs`). Shuts off Acererak, Atraxa, Cloak and Dagger, Tezzeret's own artifact-creature triggers. |
| Ensnaring Bridge | side 1 | "Creatures with power greater than the number of cards in your hand can't attack": a new `Restriction` read in `attack_candidates` (`combat.rs`), the same function RFC 0005 edits for Orim's Chant. Hand size is the Bridge controller's. |

## Larger (7)

| Card | Zone | What is new |
|---|---|---|
| Urza's Saga | main 4 | Saga rules: a lore counter as the permanent enters and again after your draw step (a turn-based action at the start of the precombat main phase), chapter triggers, sacrifice once the last chapter has resolved. Chapter I and II give abilities ("{T}: add {C}", "{2},{T}: create a Construct"): needs an **activation condition on `ActivatedDef`** (a counter check), which Storm's Mox Opal also needs. Chapter III search is data (artifact with mana value 0 or 1). Shared with Storm. |
| Summon: Bahamut | side 1 | The same Saga support (here on a 9/9 flying Dragon creature, four chapters), plus an expression for "total mana value of other permanents you control" (chapter IV damage). |
| Ugin, Eye of the Storms | main 4 | Everything but the -11 is data (cast trigger from the stack, colorless-spell trigger, +2, 0). The -11 searches for any number of colorless nonland cards, exiles them and lets you cast them free this turn: a multi-card search and a turn-limited free cast of exiled cards (the same free-cast-from-exile effect Storm needs for Beseech the Mirror). Proposal: ship Ugin without the ultimate first (documented limit); it takes two +2 activations from 7 loyalty, so it rarely matters. |
| Extinguisher Battleship | side 1 | The enters trigger (destroy a noncreature permanent, 4 damage to each creature) is data. Station (tap another creature you control, put charge counters equal to its power; at 5 counters it is a 10/10 flying trampling artifact creature) needs a "tap another creature" cost that records the creature's power. Proposal: ship the enters trigger only. |
| Argentum Masticore | side 2 | Protection from multicolored (no protection exists in the engine at all), and an upkeep "sacrifice unless you discard; when you discard, destroy a nonland permanent with mana value at most the discarded card's" (needs the discarded card's mana value as a filter bound). Proposal: documented limit, or defer. |
| Mycosynth Lattice | side 1 | Cards outside the battlefield turn colorless (statics only apply on the battlefield), and mana of any color spending. Proposal: documented limit, or defer. |
| Eldrazi Confluence | side 1 | "Choose three, you may choose the same mode more than once": the mode chooser takes distinct modes only. Proposal: ship distinct modes only (each of the three modes once), documented. |

## Overlap with the Storm thread

`storm-engine-gap.md` lists five items that Tron also needs. I suggest one shared design for each so the two decks do not diverge:

| Shared item | Storm card | Tron card |
|---|---|---|
| Wish: a fetch from outside the game (`PlayerState.sideboard`, hidden for the opponent) | Burning Wish | Karn -2 |
| Free cast of cards just exiled, this turn | Beseech the Mirror | Ugin -11 (if not deferred) |
| Saga rules | Urza's Saga | Urza's Saga, Summon: Bahamut |
| Activation condition on `ActivatedDef` | Mox Opal | Urza's Saga chapter abilities (Workshop works without it) |
| Subtypes Saga (and the append-only table) | Saga, Lesson, Sorcerer | Saga and 11 others |

## What I would do

1. Treat Tron as RFC 0007 and the Storm RFC as 0006, with the five shared items specified once in 0006 and 0007 referring to it. Items that only Tron needs: Trinisphere floor, Grim Monolith no-untap, The One Ring protection, Portable Hole, Mishra's Research Desk, Torpor Orb, Ensnaring Bridge, "or less" X filter, unblockable keyword, Planar Nexus filter ability.
2. Land the data-only cards and the 12 subtypes first (draft in `rust-engine-tron/`, hash-neutral for existing states; it changes the card-database hash and `n_defs`, so any net trained before it needs the new pool).
3. Put the large items (Masticore, Lattice, Confluence modes, Battleship station, Ugin ultimate) in `known-divergences` as documented limits, the way E4 and E5 were, and revisit if Brady wants them.
4. A different agent writes the spec scenarios from Oracle text and the CR only, as for Orim's Chant. Priority scenarios for the matchup: Trinisphere against Force of Will, Aluren and Omniscience casts; Torpor Orb against Acererak and Atraxa; Karn's static against Lotus Petal; The One Ring's protection against Force of Will targets and damage; Boseiju channel against Aluren.

The Alurentell sideboard plan for this matchup (Brady's, in the thread root) belongs to the "Add sideboarding" thread and is not planned here.

## Status 2026-10-06 (after implementation)

RFC 0008 (`rust-engine-tron/0008-colorless-tron.md`) and the patch (`rust-engine-tron/tron-core.patch`) are done in a scratch copy and not applied to the live engine. Of the 27 missing cards, 20 are fully modeled (the 9 data cards and the 11 small core items; Planar Nexus only through automatic payment, Kozilek's Command player modes only target you). Not modeled: Urza's Saga and Summon: Bahamut (RFC 0007), Karn -2, Ugin -11, Argentum Masticore, Mycosynth Lattice, Eldrazi Confluence (distinct modes only), Extinguisher Battleship station, Mishra's Research Desk unearth/expiry. Results: 156 new scenarios pass, visible spec 838/838, workspace tests and goldens clean, 3000-game fuzz clean.
