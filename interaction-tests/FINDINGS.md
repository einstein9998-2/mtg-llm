# Forge rules check for the eight-deck pool: findings

Date: 2026-10-01. Forge: Card-Forge/forge HEAD 7345bf03 (shallow clone). Harness and format: `README.md`.

## Result

233 scripted scenarios across all eight decks: 230 pass, 3 expected failures (below), 0 unexplained. 119 of the 141 distinct
non-basic cards in the deck files appear in at least one scenario, and every main-deck card is covered (basic lands aside).
Every failure during the work was classified as a harness or test error or a real Forge deviation before being kept; the harness
errors that mattered are listed under "Gaps in the shortcuts the AI seat takes" because they affect the player thread too.

| file | decks | scenarios |
|---|---|---|
| `00-smoke` | sanity | 2 |
| `10`-`13` cutter-vs-alurentell a to d | UR Cutter vs Alurentell (priority matchup): Force of Will, Daze, Veil of Summer, Show and Tell, Aluren, Acererak and Lost Mine, Tomb of Annihilation, Atraxa, Omniscience, Murktide, DRC, Cutter flurry, Unholy Heat, Prismari Charm, Wasteland, Ancient Tomb, City of Traitors | 67 |
| `20` dimir-vs-uwx | Stifle on fetchland and Bowmasters triggers, Consign to Memory (trigger, colorless, replicate), Force of Negation, Daze, Bowmasters draw triggers, Nethergoyf and Barrowgoyf, escape, Moonshadow, Fatal Push revolt, Snuff Out, Thoughtseize | 27 |
| `21` uwx-and-dimir-cards | Swords, Prismatic Ending converge, Murktide delve, Quantum Riddler (warp, extra draw), Tamiyo flip, Phelia flicker, Petty Theft and adventure, Flow State, Kaito ninjutsu and creature-only-on-own-turn, Wasteland vs basics and Snow-Covered | 21 |
| `22` boros-vs-bw-dnt | Spider-Woman (creatures, artifacts, Vial), Aether Vial, Phlage (hard cast, escape), Goblin Bombardment, Voice of Victory, Ocelot Pride with city's blessing, Ajani flip, Sand Scout, Guide of Souls, Amped Raptor, Karakas, Solitude (hard cast, evoke), Skyclave Apparition, Flickerwisp, Cloak and Dagger, Overlord impending, Samwise, White Orchid Phantom, Recruiter, Witch Enchanter, Boggart Trawler | 49 |
| `23` doomsday-and-reanimator | Doomsday pile, order and life, Thassa's Oracle (win, no win, exact devotion), Cavern of Souls, Jace, LED, Dark Ritual, Street Wraith, Edge of Autumn, Personal Tutor, Consider, Reanimate (including fizzle), Animate Dead, Shallow Grave, Unmask, Faithless Looting flashback, Cabal Therapy, Griselbrand, Archon of Cruelty, Raph and Mikey, Stronghold Gambit, Collective Brutality | 37 |
| `30` manabase-and-sideboard | all fetchlands and the dual lands they find, Prismatic Vista, six surveil lands, Preordain, Erode, Bilbo, Koma (uncounterable, ward), Pyroblast, Grafdigger's Cage, Containment Priest, Faerie Macabre, Surgical Extraction, Deafening Silence | 30 |

## What Forge gets wrong

1. **Aluren only works for the first player in the game's player list.** Aluren says any player may cast creatures with mana value 3
   or less for free at instant speed. Forge's script sets `MayPlayPlayer$ Player`, and the engine
   (`StaticAbilityContinuous`, the MayPlay grant) takes `.get(0)` of the defined players, so only the first seat gets the permission.
   Scenarios `aluren-opponent-may-cast-free` and `aluren-seat-order-second-seat-controller` show both symptoms: the second seat cannot
   use Aluren, whether it controls Aluren or not. In a real game the seat order decides whether the Alurentell player can combo at
   all, so any Alurentell result is corrupted until this is fixed. A two-static fix (`MayPlayPlayer$ You` plus `MayPlayPlayer$ Opponent`)
   is in `fixes/aluren-any-player.patch`. I applied it to a scratch copy: both scenarios then pass (XPASS) and nothing else changes. It is not applied to the
   Forge checkout the other thread uses; say if you want it applied there.
2. **The Forge AI will not tap Ancient Tomb when the 2 damage would kill it** (`ancient-tomb-lethal-tap`). Rules-wise the tap is legal.
   This is the AI's payment code (`ComputerUtilMana`), not the engine, but it also hits the shared auto-payer, so the LLM seat
   cannot be offered that line through the auto-payer. Low impact: only matters at 2 life or less.

Everything else tested behaved as the rules require, including the places where I expected Forge to be weak: Stifle on fetchland and
trigger abilities, Consign to Memory, Force of Negation's turn restriction, Daze returning an Island as a cost, Nethergoyf escape,
Snuff Out and Fatal Push conditions, Skyclave Apparition and Flickerwisp rulings, Phelia flicker resetting counters, Samwise's
"from the battlefield this turn", Animate Dead sacrifice on leave, Reanimate fizzling and its life loss, Doomsday ordering,
Thassa's Oracle devotion, Cavern of Souls, Koma's ward and uncounterability, Grafdigger's Cage and Containment Priest, and the
Spider-Woman enters-tapped replacement for Vial-put creatures.

## Gaps in the shortcuts the AI seat takes (matters for the LLM-player thread)

These are not rules bugs in Forge's real play path, but any code that drives Forge through the AI controller shortcuts hits them.
I found each of them because my first harness version had the same hole.

- **Casting restrictions are not enforced by `getAllPossibleAbilities(player, true)` or by the AI-style play sequence.**
  The real human play path (`PlaySpellAbility`) calls `ability.checkRestrictions(player)`, which applies "can't cast" statics.
  Without that call Forge happily let the opponent cast Solitude during Voice of Victory's controller's turn. An action
  enumerator must filter with `checkRestrictions` (or play through `PlaySpellAbility`) or it will offer illegal spells.
  My harness now calls it; the shipped serializer and enumerator should be checked for the same hole.
- **Triggered abilities of an AI-controlled seat choose their targets in `AiController.doTrigger`, never in `chooseTargetsFor`**
  (`PlayerControllerAi.orderAndPlaySimultaneousSa`). Overriding only `chooseTargetsFor` leaves trigger targets (Bowmasters, Phelia,
  Skyclave Apparition, White Orchid Phantom) to the AI. The harness overrides `orderAndPlaySimultaneousSa`.
- **"You may cast it" effects (Bilbo, Amped Raptor, Show and Tell style casts) are decided by the AI's `canPlayFromEffectAI`.**
  The AI declined to cast Flow State off Bilbo's trigger. The harness overrides `playSaFromPlayEffect` (`pick play` / `pick skip`).
- **Collective choices with a collective restriction are not enumerated by stock code**: escape costs such as Nethergoyf's
  ("four or more card types among them", `withTypesGE4`) fail a per-card validity check, so a naive enumerator sees zero legal cards.
- **Library orders are returned bottom-first**: `orderMoveToZoneList` results are moved to position 0 one at a time, so the last
  element ends on top.
- The AI payer chooses which lands to tap and how many replicate payments to make, so scenarios give exactly the lands needed.

## Not covered

- 22 sideboard-only cards have no scenario: Abrade, Bitter Triumph, Carpet of Flowers, Defense Grid, Dismember, Disruptor Flute,
  Duress, Flusterstorm, Gaddock Teeg, Hide on the Ceiling, Hydroblast, Lavinia, Loran of the Third Path, Magus of the Moon, Massacre,
  Meltdown, Nihil Spellbomb, Null Rod, Path to Exile, Pyroclasm, Triumph of Saint Katherine, Wrath of the Skies. Add them if
  those matchups get played with sideboards.
- Combat is only tested through Voice of Victory, Raph and Mikey, Archon, Bilbo and Phelia triggers. Multi-blocker damage assignment,
  first strike plus lifelink ordering and similar are not covered.
- Multi-step turn sequences (a whole turn of a real list) are not covered; these tests check single rules interactions.
- Hidden information was not tested: the harness sees everything.
- Layer interactions beyond Nethergoyf, Barrowgoyf, Kaito, Moonshadow and Cavern are not covered.
