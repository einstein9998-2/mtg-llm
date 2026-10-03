# Proposed Legacy deck set for the MTG LLM project

_Recommendation for Brady to edit, not a decision. Prepared 2026-10-01. Banlist as of 2026-08-10. Forge master as of 2026-09-30 (commit fd5c996)._

## Recommendation

| # | Deck | Type | Role | Non-basic cards | Forge-AI-flagged cards in main |
|---|------|------|------|-----------------|-------------------------------|
| 1 | Mono-White Death and Taxes | Aggro / taxes (non-combo) | Pipeline pair A | 27 | 0 |
| 2 | Jund Midrange | Midrange (non-combo) | Pipeline pair A | 31 | 0 |
| 3 | Izzet Delver | Tempo (non-combo, stack-heavy) | Pipeline pair B | 28 | 0 |
| 4 | Jeskai Control | Control (non-combo) | Pipeline pair B | 34 | 0 |
| 5 | Rakdos Reanimator | Combo (graveyard) | Combo set | 28 | 4 |
| 6 | Sneak and Show | Combo (big-creature cheat) | Combo set | 29 | 4 |
| 7 | Doomsday | Combo (pile / Oracle) | Combo set | 31 | 3 |
| 8 | Ad Nauseam Tendrils (ANT) | Combo (storm), stretch tier | Stretch (highest engine risk) | 28 | 7 |

**Combined pool: 135 unique non-basic cards (140 names with basics).** 50 of those appear in two or more decks, mostly fetches, duals, Brainstorm, Ponder, Force of Will, Thoughtseize and Lightning Bolt. That is below your 200 to 300 estimate. A smaller pool is cheaper to verify, but it also means fewer cards to test the engine against, so there is room to add decks or splash cards if you want a bigger test surface.

**First pipeline matchup: Death and Taxes vs Jund.** Neither deck has a stack-heavy plan, and neither has any card in its main deck that Forge's AI is flagged as unable to play. The two are an aggro-taxes deck and a midrange deck, so combat, removal and card advantage all get exercised.

**Second matchup: Izzet Delver vs Jeskai Control.** It adds Force of Will, Daze, Counterspell, Brainstorm and Snapcaster, which is where the stack and priority handling gets tested. Neither main deck has an AI-flagged card.

The four combo decks are meant to be piloted by your players against those four non-combo decks. They should not be the Forge AI's deck: Forge marks 24 cards in this pool (12 `AI:RemoveDeck:All`, 12 `AI:RemoveDeck:Random`) as cards its AI will not build around, and they cluster in the combo lists (ANT 7, Reanimator 4, Sneak and Show 4, Doomsday 3).

## Why these eight

- **Coverage of your brief's stress list:** Force of Will, Daze and Brainstorm (Delver, Control, Sneak and Show, Doomsday); long combo sequences (ANT, Doomsday); graveyard and stack tricks (Reanimator, Sneak and Show); bluffing spots (Control vs combo). Stifle is not in any list, so add it to Delver if you want that interaction tested.
- **Chosen for archetype spread, not meta share.** The two meta trackers I could read disagree (MTGGoldfish leads with Dimir Tempo at 10.3%, then Eldrazi and Boros Energy; MTGDecks leads with Reanimator at 14.4%). I would not rely on either for deck choice.
- **ANT is the stretch deck.** It has the highest engine risk (Lion's Eye Diamond, Past in Flames, Ad Nauseam, storm count) and all seven of its flagged cards are in the main deck. Drop it first if you want seven decks.
- **These are my own builds, not copied tournament lists.** They are legal and complete (60/15, four-of limit respected), but their strength is untested. Tune them against current results before using them as a win-rate benchmark.

## Legality check

Every card was checked against the Legacy banlist from Wizards' banned and restricted page and cross-checked with ScrollVault. Both agree on 73 named cards as of 2026-08-10 (The Fantasticar was the latest ban). No card in any list is banned. The banlist has moved a lot, which shapes the lists: Entomb, Sensei's Top, Survival of the Fittest, Deathrite Shaman, Psychic Frog, Ragavan, Oko and Wrenn and Six are all banned, so Reanimator has no Entomb, Control has no Top, and Jund has no Deathrite. I read the banlist through a fetch tool that summarizes pages, so spot-check it against the official page before committing. Scryfall was blocked in this environment, so I could not check legality per card there.

## Forge check

**Implemented:** all 135 cards have a card script in Forge master (`forge-gui/res/cardsfolder`). Per-card details are in [forge-card-coverage.csv](/mnt/project-files/decks/forge-card-coverage.csv).

**Not verified:** a script existing does not mean the card behaves correctly. I did not build or run Forge. I read the scripts of about 25 of the riskiest cards against their Oracle text (from memory) and found no mismatch in Sneak Attack, Show and Tell, Doomsday, Thassa's Oracle, Daze, Force of Will, Misdirection, Lion's Eye Diamond, Past in Flames, Ad Nauseam, Intuition, Gamble, Animate Dead, Reanimate, Exhume, Emrakul, Murktide Regent, Cavern of Souls, Solitude, Tendrils, Cabal Ritual, Infernal Tutor, Council's Judgment, Cabal Therapy. That is a weak signal. Forge's own unit tests name only 25 of the 135 cards, and mostly incidentally.

**Known Forge bugs: I could not get a list.** Forge's issue tracker was not reachable (GitHub search is blocked by robots.txt for the fetch tool, and the API is not enabled for this session), and web search returned nothing card-specific. So "no known bugs" would be wrong; the honest status is "unchecked".

**Useful findings from the code:**
- Forge has a headless `sim` mode (`forge-gui-desktop/.../view/SimulateMatch.java`, reached through `Main.java`), which is the natural base for the Phase 1 batched runner. JDK 21 is installed here; the build needs JDK 17+. I did not measure games per second.
- The hidden-choice resolution in `ChangeZoneEffect.java` collects every player's choice before any card moves, which matters for Show and Tell and Exhume. I read this, I did not run it.
- Forge has a `GameSimulationTest` and `SpellAbilityPickerSimulationTest` that script boards and play out turns. They are a ready-made pattern for the "known interactions" suite in your brief.

**Interactions I would test first:** Daze returning an Island (including a dual with the Island type) with Force of Will in response; Sneak Attack and Show and Tell with Griselbrand and Emrakul, including removal in response to the sacrifice trigger; Show and Tell when the opponent also puts a permanent in; Doomsday into Thassa's Oracle with a small library; Lion's Eye Diamond with Past in Flames and Ad Nauseam; Animate Dead with the target exiled in response; Cavern of Souls making a spell uncounterable; Murktide Regent's delve count; Solitude evoke; Council's Judgment voting; Cabal Therapy flashback; Emrakul's extra turn and graveyard shuffle; Karakas bouncing a legendary creature.

## Easy swaps if you want a different mix

Dimir Tempo (Orcish Bowmasters, Thoughtseize, Tamiyo) is the top deck on MTGGoldfish and would replace Delver or Jund. Boros Energy would replace Death and Taxes. Tron and Lands are good low-interaction decks if you want a prison or ramp archetype. I stayed away from cards from the newest sets that I could not confirm exist in Forge.

## Decklists

Also saved one per file in this folder, in `N Card Name` format with the sideboard after a blank line.

### 1. Mono-White Death and Taxes (Aggro / taxes (non-combo))

```
Main (60)
4 Mother of Runes
4 Stoneforge Mystic
4 Thalia, Guardian of Thraben
3 Flickerwisp
2 Recruiter of the Guard
2 Phyrexian Revoker
2 Mirran Crusader
1 Brimaz, King of Oreskos
4 Aether Vial
4 Swords to Plowshares
2 Batterskull
1 Umezawa's Jitte
1 Sword of Fire and Ice
1 Council's Judgment
1 Oblivion Ring
1 Armageddon
4 Wasteland
1 Karakas
18 Plains

Sideboard (15)
2 Rest in Peace
2 Containment Priest
2 Kataki, War's Wage
2 Pithing Needle
2 Wear // Tear
2 Ethersworn Canonist
1 Elspeth, Sun's Champion
1 Disenchant
1 Mana Tithe
```

### 2. Jund Midrange (Midrange (non-combo))

```
Main (60)
4 Tarmogoyf
3 Orcish Bowmasters
3 Bloodbraid Elf
2 Scavenging Ooze
2 Dark Confidant
2 Grim Lavamancer
4 Lightning Bolt
3 Thoughtseize
2 Inquisition of Kozilek
3 Liliana of the Veil
3 Kolaghan's Command
2 Abrupt Decay
2 Fatal Push
1 Maelstrom Pulse
4 Verdant Catacombs
3 Bloodstained Mire
3 Wooded Foothills
3 Badlands
3 Bayou
2 Taiga
2 Wasteland
2 Forest
1 Swamp
1 Mountain

Sideboard (15)
2 Collective Brutality
2 Surgical Extraction
2 Pyrokinesis
2 Toxic Deluge
2 Obstinate Baloth
1 Pernicious Deed
1 Ancient Grudge
1 Pithing Needle
1 Hymn to Tourach
1 Liliana, the Last Hope
```

### 3. Izzet Delver (Tempo (non-combo, stack-heavy))

```
Main (60)
4 Delver of Secrets
4 Dragon's Rage Channeler
3 Brazen Borrower
3 Murktide Regent
1 Snapcaster Mage
4 Brainstorm
3 Ponder
3 Preordain
4 Lightning Bolt
3 Force of Will
3 Daze
2 Counterspell
2 Chain Lightning
2 Spell Pierce
4 Scalding Tarn
3 Misty Rainforest
4 Volcanic Island
4 Island
1 Mountain
3 Wasteland

Sideboard (15)
2 Pyroblast
2 Red Elemental Blast
2 Flusterstorm
2 Surgical Extraction
2 Abrade
1 Pyroclasm
1 Tormod's Crypt
1 Fire // Ice
1 Vendilion Clique
1 Smash to Smithereens
```

### 4. Jeskai Control (Control (non-combo))

```
Main (60)
3 Snapcaster Mage
2 Solitude
1 Vendilion Clique
1 Torrential Gearhulk
4 Brainstorm
3 Ponder
4 Swords to Plowshares
3 Force of Will
3 Counterspell
2 Mana Leak
2 Supreme Verdict
2 Teferi, Time Raveler
2 Jace, the Mind Sculptor
2 Lightning Bolt
1 Council's Judgment
1 Terminus
1 Memory Lapse
4 Flooded Strand
2 Arid Mesa
2 Scalding Tarn
4 Tundra
3 Volcanic Island
1 Plateau
3 Island
2 Plains
1 Mountain
1 Karakas

Sideboard (15)
2 Rest in Peace
2 Flusterstorm
2 Red Elemental Blast
2 Surgical Extraction
2 Wear // Tear
1 Pithing Needle
1 Containment Priest
1 Elspeth, Sun's Champion
1 Disenchant
1 Pyroclasm
```

### 5. Rakdos Reanimator (Combo (graveyard))

```
Main (60)
4 Reanimate
3 Animate Dead
2 Exhume
4 Faithless Looting
3 Thoughtseize
2 Inquisition of Kozilek
4 Dark Ritual
4 Lotus Petal
2 Cabal Therapy
2 Unearth
3 Griselbrand
2 Archon of Cruelty
2 Atraxa, Grand Unifier
2 Bloodghast
2 Putrid Imp
4 Bloodstained Mire
3 Badlands
3 Blood Crypt
4 Swamp
3 Mountain
2 Cavern of Souls

Sideboard (15)
2 Duress
2 Collective Brutality
2 Surgical Extraction
2 Hymn to Tourach
2 Fatal Push
1 Grave Titan
1 Cabal Therapy
1 Pyrokinesis
1 Kolaghan's Command
1 Pithing Needle
```

### 6. Sneak and Show (Combo (big-creature cheat))

```
Main (60)
3 Griselbrand
3 Emrakul, the Aeons Torn
1 Archon of Cruelty
1 Atraxa, Grand Unifier
1 Omniscience
4 Show and Tell
4 Sneak Attack
4 Brainstorm
3 Ponder
3 Preordain
4 Force of Will
3 Daze
3 Spell Pierce
2 Lotus Petal
1 Misdirection
1 Intuition
4 Scalding Tarn
3 Misty Rainforest
4 Volcanic Island
5 Island
2 Mountain
1 Wasteland

Sideboard (15)
2 Pyroblast
2 Red Elemental Blast
2 Flusterstorm
2 Surgical Extraction
2 Lightning Bolt
1 Gamble
1 Engineered Explosives
1 Tormod's Crypt
1 Wasteland
1 Spell Snare
```

### 7. Doomsday (Combo (pile / Oracle))

```
Main (60)
4 Doomsday
1 Thassa's Oracle
1 Laboratory Maniac
4 Dark Ritual
3 Lotus Petal
4 Brainstorm
4 Ponder
2 Preordain
4 Thoughtseize
4 Force of Will
2 Daze
2 Spell Pierce
2 Dark Confidant
2 Cabal Therapy
2 Inquisition of Kozilek
1 Snuff Out
4 Polluted Delta
2 Misty Rainforest
4 Underground Sea
3 Island
2 Swamp
1 Wasteland
2 Mana Confluence

Sideboard (15)
2 Surgical Extraction
2 Flusterstorm
2 Duress
2 Hymn to Tourach
2 Fatal Push
1 Echoing Truth
1 Pithing Needle
1 Engineered Explosives
1 Tormod's Crypt
1 Orcish Bowmasters
```

### 8. Ad Nauseam Tendrils (ANT) (Combo (storm), stretch tier)

```
Main (60)
4 Tendrils of Agony
3 Ad Nauseam
2 Past in Flames
3 Lion's Eye Diamond
4 Lotus Petal
4 Dark Ritual
2 Cabal Ritual
4 Infernal Tutor
4 Brainstorm
3 Ponder
2 Preordain
3 Thoughtseize
2 Duress
1 Manamorphose
4 Polluted Delta
3 Bloodstained Mire
4 Underground Sea
3 Badlands
2 Volcanic Island
2 Swamp
1 Island

Sideboard (15)
2 Flusterstorm
2 Pyroblast
2 Red Elemental Blast
2 Surgical Extraction
2 Hymn to Tourach
2 Inquisition of Kozilek
1 Pithing Needle
1 Empty the Warrens
1 Tormod's Crypt
```
