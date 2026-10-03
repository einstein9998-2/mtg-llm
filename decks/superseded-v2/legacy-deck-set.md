# Legacy deck set v2: real tournament lists

_Replaces the first proposal (kept in `superseded-v1/`). Prepared 2026-10-01. Recommendation for Brady to edit. Banlist as of 2026-08-10, Forge master as of 2026-09-30._

## First matchup (Brady's pick): UR Cutter vs Alurentell

Brady chose this pairing himself, so it comes before the rest of the set. Both lists are real September 2026 tournament lists, in [ur-cutter.txt](/mnt/project-files/decks/ur-cutter.txt) and [alurentell.txt](/mnt/project-files/decks/alurentell.txt).

- **UR Cutter:** the Murktide/Cori-Steel Cutter tempo deck with no Delver of Secrets. Source: hejljud, 4th, MTGO League, 2026-09-22 ([list](https://mtgtop8.com/event?e=91149&d=892306&f=LE)). A second UR Cutter list from a 24-player event (Bruno Lorenzato, 3rd-4th, 2026-09-27, [list](https://mtgtop8.com/event?e=91364&d=893897&f=LE)) differs by running Tamiyo and Flow State instead of Preordain and Prismari Charm. The UR Delver list in the main set below (doikun) is the Delver version of the same shell.
- **Alurentell:** I read this as the Aluren and Show and Tell hybrid. That is my inference from the name, not something I could confirm with Brady. Every Aluren list mtgtop8 shows for 2026-09-22 to 09-28 (Freqing, Zavadil, Minoue Hiroyuki, Kheldarion, Pikolay) is this same shell: 4 Aluren, 4 Show and Tell, 2 Omniscience, 4 Acererak, 4 Atraxa, 4 Stock Up, Force of Will, Brainstorm, Ponder, Ancient Tomb. Source used: Freqing, 1st, MTGO League, 2026-09-27 ([list](https://mtgtop8.com/event?e=91342&d=893755&f=LE)). Two web results also use the name "Aluren-Tell".
- **Checks:** both are exactly 60 main and 15 side, no banned cards, no more than four copies, and every card has a Forge script.
- **Forge-AI-unsupported cards (`AI:RemoveDeck:All`):** UR Cutter main has Mishra's Bauble (4), sideboard has Meltdown (2). Alurentell main has City of Traitors (1), sideboard has Carpet of Flowers (3). Aluren, Omniscience, Ancient Tomb, Lotus Petal and Veil of Summer are flagged `Random`, which only means narrow.
- **Do not let the Forge AI pilot Alurentell.** It has no logic for an Aluren or Show and Tell combo, so a Forge-AI Alurentell would mostly sit there, and a win rate against it would overstate how good UR Cutter's player is. If you want a Forge AI baseline, put it on the UR Cutter side and have our player pilot Alurentell, or use a scripted combo opponent.
- **Interactions to test first in this pairing:** Force of Will and Daze on both sides (UR Cutter also has Wasteland, which hits Ancient Tomb and City of Traitors); Show and Tell with both players putting a permanent in (the choices must be hidden from each other); Aluren being usable by any player, so UR Cutter can cast Dragon's Rage Channeler for free at instant speed; Acererak looping with Aluren and venturing through Tomb of Annihilation (Forge has dungeon support in `VentureEffect.java` and a `tomb_of_annihilation` token script, but I did not run it); Atraxa's reveal-ten trigger; Omniscience free casts; Murktide Regent's delve; Cori-Steel Cutter's second-spell Monk trigger; Stock Up.

The rest of this report is the earlier eight-deck set. Two of its lists now overlap with this matchup (UR Delver with UR Cutter, Omni-Show with Alurentell), so if you want a final set of seven or eight, I would swap in the two lists above and replace the duplicates. I'd hold that until you say which decks you want.

## What was wrong with v1

Jund is not a current Legacy deck, and v1 was wrong in other places too. I built those lists from memory of older Legacy, so Jund, Jeskai Control, Ad Nauseam Tendrils, Mono-White Death and Taxes and my Sneak and Show list did not match what is winning now. v2 uses lists that actually placed in September 2026 events, read from mtgtop8. Those lists are full of cards I could not have guessed (Phelia, Nethergoyf, Moonshadow, Quantum Riddler, Cori-Steel Cutter), which is the main sign that v1 was stale.

## Current meta, from four sources that agree on the top

| Source | Window | Top of the format |
|---|---|---|
| MTGGoldfish, paper and MTGO | last 30 days | Dimir Tempo 10.3%, Eldrazi 6.8%, Boros Energy 6.3%, BW Death and Taxes 5.3%, Doomsday 5.1%, Tron 5.0%, Izzet Delver 4.2%, Sewer-Cam 4.1%, Aluren 3.9% |
| mtgtop8, 481 decks | last 2 months | Dimir Tempo 11%, UWx Control 9%, Boros Aggro 6%, Tron 5%, Eldrazi 4%, UR Tempo 4%, D&T 4%, Show and Tell 4%, Doomsday 4%, Aluren 4%, Reanimator 4%, Lands 4%, Storm 3% |
| AetherHub | last 180 days | Dimir Tempo 7.5%, Karn Forge 6.5%, UWx Phelia 6.3%, UR Tempo 5.6%, Artifacts Blue 5.0%, Doomsday 5.0% |
| decklistdata, MTGO | last 14 days | Dimir Tempo 10.1%, Ocelot Aggro 9.7%, Doomsday 6.5% (win rate 65.8%), Eldrazi 6.5%, OmniTell 6.0% |

One other tracker I read earlier (MTGDecks) showed Reanimator at 14% and Goblins in the top three, which none of these four support, so I treated it as an outlier.

## Recommended set

| # | Deck | Type | Meta evidence | Source list | Forge-AI-unsupported cards in main |
|---|------|------|---------------|-------------|------|
| 1 | Boros Aggro (Energy) | Aggro (non-combo) | Boros Energy #3 on MTGGoldfish (6.3%); Ocelot Aggro 9.7% on MTGO tier list; Boros Aggro 6% on mtgtop8 2-month | [Fenrir18, 13th, MTGO Challenge 32, 2026-09-27 (65 players)](https://mtgtop8.com/event?e=91385&d=894044&f=LE) | Goblin Bombardment (3) |
| 2 | BW Death and Taxes | Midrange / taxes (non-combo) | Death and Taxes (Yorion) BW #4 on MTGGoldfish (5.3%); BW D&T 3.4% AetherHub; D&T 4% mtgtop8 2-month | [Federodi, 5th-8th, 1 Tappa Tigullio League 2026/27, 2026-09-27 (23 players)](https://mtgtop8.com/event?e=91360&d=893865&f=LE) | none |
| 3 | Dimir Tempo | Tempo (non-combo) | #1 on MTGGoldfish (10.3%), MTGO tier list (10.1%), AetherHub (7.5%), mtgtop8 2-month (11%) | [_GhostWalking_, 14th, MTGO Challenge 32, 2026-09-27 (65 players)](https://mtgtop8.com/event?e=91385&d=894054&f=LE) | Nethergoyf (4), Mishra's Bauble (3) |
| 4 | UR Delver (Cutter) | Tempo (non-combo, stack-heavy) | Izzet Delver #7 on MTGGoldfish (4.2%); UR Tempo #4 on AetherHub (5.6%); UR Tempo 4% mtgtop8 2-month | [doikun, 2nd, MTGO League, 2026-09-28](https://mtgtop8.com/event?e=91386&d=894059&f=LE) | Mishra's Bauble (4) |
| 5 | UWx Control (Phelia / Riddler / Stifle) | Control-tempo (non-combo) | UWx Control 9% on mtgtop8 2-month (#2); Uwx Phelia #3 on AetherHub (6.25%); Azorius Tempo 2.2% on MTGGoldfish | [HJ_Kaiser, 5th-8th, MTGO Challenge 32, 2026-09-27 (65 players)](https://mtgtop8.com/event?e=91385&d=894042&f=LE) | Stifle (4), Prismatic Ending (2) |
| 6 | Doomsday | Combo (pile / Oracle) | #5 on MTGGoldfish (5.1%); 6.5% and best win rate (65.8%) on MTGO tier list; 4% mtgtop8 2-month | [thescuba96, 2nd, MTGO Challenge 32, 2026-09-27 (65 players)](https://mtgtop8.com/event?e=91385&d=894043&f=LE) | Doomsday (4), Edge of Autumn (2), Lion's Eye Diamond (1) |
| 7 | Omni-Show (Show and Tell / Omniscience) | Combo (cheat big permanents) | Show and Tell family 4% on mtgtop8 2-month; OmniTell 6.0% on MTGO tier list; Sneak and Show 2.5% / Omni-Tell 1.6% on MTGGoldfish | [Riccardo Saino, 5th-8th, 1 Tappa Tigullio League 2026/27, 2026-09-27 (23 players)](https://mtgtop8.com/event?e=91360&d=893866&f=LE) | none |
| 8 | Reanimator | Combo (graveyard) | Reanimator 4% on mtgtop8 2-month; Rakdos Reanimator 3.1% on MTGGoldfish; 4.1% on MTGO tier list | [pacoelflaco, 2nd, MTGO Challenge 32, 2026-09-26](https://mtgtop8.com/event?e=91314&d=893501&f=LE) | Faithless Looting (4), Unmask (4), Cabal Therapy (1) |

**Pool of the eight-deck set:** 153 unique non-basic cards (47 shared by two or more decks), still below your 200 to 300 estimate, but closer than v1's 135. Every list is a single real tournament list, not an average of many, so each is one pilot's choices. Tune them if you want a more typical build.

**Spread:** five non-combo decks (aggro, taxes-midrange, two tempo decks, tempo-control) and three combo decks (Doomsday, Omni-Show, Reanimator). There is no pure control deck: the UWx Control list is a tempo-control deck built on Phelia, Quantum Riddler, Tamiyo and Stifle, which is what UWx Control currently means in Legacy. Storm is 3% on mtgtop8, so it is out for now.

## Recommended first non-combo matchups

1. **Boros Aggro vs BW Death and Taxes.** Both are real top-ten decks. Both are creature-combat decks with little stack interaction, so decisions are cheap to enumerate and blunders are easy to label against engine ground truth. BW D&T has no Forge-AI-unsupported card in its main deck and Boros has one (Goblin Bombardment, 3 copies), so a Forge AI opponent can play them reasonably.
2. **Dimir Tempo vs UR Delver.** The two leading blue tempo decks, with Force of Will, Daze, Brainstorm, Ponder and Bowmasters on both sides. This is where stack and priority handling gets exercised, so run it second. Nethergoyf and Mishra's Bauble (Dimir) and Mishra's Bauble (Delver) are the Forge-AI-unsupported cards to watch.

The combo decks are meant for your players to pilot against these non-combo decks, not for the Forge AI to pilot.

## Legality

No card in any main deck or sideboard is on the Legacy banlist (Wizards' list as of 2026-08-10, 73 named cards, cross-checked with ScrollVault). Counts are exactly 60 main and 15 side for every deck, with no more than four copies of any card. All 153 card names also match a Forge card script, which is a useful extra check that my transcription did not invent names. I read the lists through a page-summarizing fetch tool, not a raw export, so a miscopied quantity that still totals 60 is possible; spot-check against the linked pages before treating any list as exact.

## Forge check

**Implemented:** all 153 cards have a Forge script (`forge-gui/res/cardsfolder`, master as of 2026-09-30). Per-card details are in [forge-card-coverage.csv](/mnt/project-files/decks/forge-card-coverage.csv).

**Correction to v1:** Forge's own docs say `AI:RemoveDeck:All` marks cards the AI cannot use or can only use badly, while `AI:RemoveDeck:Random` only means the card is too narrow for random decks. I had lumped them together. In this pool 15 cards carry `All` and 18 carry `Random`. Only `All` matters for AI strength.

**Not verified:** a script existing is not proof of correct behavior. I did not build or run Forge, and Forge's own unit tests name only 15 of these 153 cards. I read the scripts of about 20 of the riskiest new cards (Stifle, Nethergoyf, Barrowgoyf, Moonshadow, Flow State, Cunning Wish, Omniscience, Phelia, Quantum Riddler, Shallow Grave, Stronghold Gambit, Unmask, Personal Tutor, Cori-Steel Cutter, Raph & Mikey, plus the combo cards from v1) and saw no clear mismatch with their text. That is a weak signal.

**Known Forge bugs:** I still could not get a list. Forge's GitHub issue search is blocked to my fetch tool by robots.txt and the API is not enabled for this session. Treat bug status as unchecked.

**Risks that are specific to this set:**
- **Cunning Wish (Omni-Show) pulls cards from the sideboard.** The runner has to load sideboards into the match and the player has to see them, and the wish targets are hidden information about our own deck only.
- **Stifle (UWx Control) targets triggered and activated abilities.** The legal-action enumerator has to expose triggers on the stack as targets. Forge flags it as AI-unsupported.
- **Newer and Universes Beyond cards** (Spider-Woman, Raph & Mikey, Bilbo, Samwise, Kaito, Cloak and Dagger) are the least battle-tested scripts. Most are one-of or two-of cards, but they are a good place to look for rules bugs.
- **Forge has a headless `sim` mode** (`forge-gui-desktop/.../view/SimulateMatch.java`) for the Phase 1 runner; JDK 21 is installed here. Games per second not measured.

## Not included, in case you want to swap

Eldrazi Aggro (#2 on MTGGoldfish), Tron and Karn Forge, Artifacts Blue (won MTGO Challenge 32 on 2026-09-27), Lands, Aluren, Sewer-Cam Combo, Hogaak, Storm. I did not pull lists for them. Artifacts Blue and Lands are the two I would look at first if you want a colorless or prison-style deck.

## Decklists

Also saved one per file in this folder, with source and link in the header.

### 1. Boros Aggro (Energy)

Source: [Fenrir18, 13th, MTGO Challenge 32, 2026-09-27 (65 players)](https://mtgtop8.com/event?e=91385&d=894044&f=LE)

```
Main (60)
4 Arid Mesa
1 Elegant Parlor
1 Flooded Strand
2 Karakas
2 Lazotep Quarry
1 Marsh Flats
1 Mountain
2 Plains
3 Plateau
1 Savannah
4 Wasteland
2 Windswept Heath
4 Ajani, Nacatl Pariah
4 Amped Raptor
4 Guide of Souls
4 Ocelot Pride
1 Phlage, Titan of Fire's Fury
3 Sand Scout
3 Spider-Woman, Stunning Savior
4 Voice of Victory
2 Erode
4 Swords to Plowshares
3 Goblin Bombardment

Sideboard (15)
2 Abrade
2 Containment Priest
3 Deafening Silence
3 Faerie Macabre
2 Gaddock Teeg
3 Pyroblast
```

### 2. BW Death and Taxes

Source: [Federodi, 5th-8th, 1 Tappa Tigullio League 2026/27, 2026-09-27 (23 players)](https://mtgtop8.com/event?e=91360&d=893865&f=LE)

```
Main (60)
1 Flooded Strand
2 Karakas
4 Marsh Flats
3 Plains
1 Prismatic Vista
2 Scrubland
1 Shadowy Backstreet
1 Swamp
4 Wasteland
1 Boggart Trawler
1 Cloak and Dagger, Entwined
2 Flickerwisp
2 Orcish Bowmasters
3 Overlord of the Balemurk
4 Phelia, Exuberant Shepherd
3 Recruiter of the Guard
2 Samwise the Stouthearted
2 Skyclave Apparition
4 Solitude
3 White Orchid Phantom
2 Witch Enchanter
4 Swords to Plowshares
4 Thoughtseize
4 Aether Vial

Sideboard (15)
1 Containment Priest
3 Deafening Silence
2 Disruptor Flute
1 Erode
1 Faerie Macabre
1 Grafdigger's Cage
1 Loran of the Third Path
1 Path to Exile
1 Surgical Extraction
1 White Orchid Phantom
2 Wrath of the Skies
```

### 3. Dimir Tempo

Source: [_GhostWalking_, 14th, MTGO Challenge 32, 2026-09-27 (65 players)](https://mtgtop8.com/event?e=91385&d=894054&f=LE)

```
Main (60)
2 Bloodstained Mire
2 Flooded Strand
1 Island
4 Polluted Delta
1 Swamp
1 Undercity Sewers
4 Underground Sea
4 Wasteland
1 Bilbo, Thief in the Night
1 Brazen Borrower
4 Moonshadow
4 Nethergoyf
3 Orcish Bowmasters
4 Brainstorm
3 Daze
3 Fatal Push
2 Flow State
4 Force of Will
3 Ponder
1 Snuff Out
3 Thoughtseize
2 Kaito, Bane of Nightmares
3 Mishra's Bauble

Sideboard (15)
2 Barrowgoyf
3 Consign to Memory
1 Fatal Push
2 Force of Negation
1 Hydroblast
2 Massacre
2 Nihil Spellbomb
1 Null Rod
1 Snuff Out
```

### 4. UR Delver (Cutter)

Source: [doikun, 2nd, MTGO League, 2026-09-28](https://mtgtop8.com/event?e=91386&d=894059&f=LE)

```
Main (60)
3 Flooded Strand
1 Island
2 Misty Rainforest
2 Polluted Delta
2 Scalding Tarn
1 Thundering Falls
4 Volcanic Island
4 Wasteland
1 Brazen Borrower
3 Delver of Secrets
4 Dragon's Rage Channeler
2 Murktide Regent
4 Brainstorm
3 Daze
3 Flow State
4 Force of Will
4 Lightning Bolt
4 Ponder
1 Preordain
1 Unholy Heat
3 Cori-Steel Cutter
4 Mishra's Bauble

Sideboard (15)
1 Abrade
3 Consign to Memory
1 Fire Magic
1 Force of Negation
1 Hydroblast
2 Meltdown
1 Null Rod
2 Pyroblast
1 Rough // Tumble
1 Surgical Extraction
1 Unholy Heat
```

### 5. UWx Control (Phelia / Riddler / Stifle)

Source: [HJ_Kaiser, 5th-8th, MTGO Challenge 32, 2026-09-27 (65 players)](https://mtgtop8.com/event?e=91385&d=894042&f=LE)

```
Main (60)
2 Arid Mesa
4 Flooded Strand
1 Meticulous Archive
3 Polluted Delta
1 Snow-Covered Island
1 Snow-Covered Plains
3 Tundra
1 Volcanic Island
4 Wasteland
3 Murktide Regent
4 Phelia, Exuberant Shepherd
4 Quantum Riddler
4 Tamiyo, Inquisitive Student
4 Brainstorm
1 Consign to Memory
2 Force of Negation
4 Force of Will
4 Ponder
2 Prismatic Ending
4 Stifle
4 Swords to Plowshares

Sideboard (15)
2 Consign to Memory
2 Containment Priest
2 Erode
1 Hydroblast
2 Lavinia, Azorius Renegade
2 Pyroblast
2 Triumph of Saint Katherine
2 Wrath of the Skies
```

### 6. Doomsday

Source: [thescuba96, 2nd, MTGO Challenge 32, 2026-09-27 (65 players)](https://mtgtop8.com/event?e=91385&d=894043&f=LE)

```
Main (60)
1 Cavern of Souls
1 Flooded Strand
1 Misty Rainforest
4 Polluted Delta
1 Scalding Tarn
1 Snow-Covered Island
1 Snow-Covered Swamp
1 Undercity Sewers
4 Underground Sea
1 Verdant Catacombs
4 Street Wraith
1 Thassa's Oracle
4 Brainstorm
1 Consider
4 Dark Ritual
3 Daze
4 Doomsday
2 Edge of Autumn
2 Flow State
4 Force of Will
3 Personal Tutor
4 Ponder
3 Thoughtseize
1 Jace, Wielder of Mysteries
1 Lion's Eye Diamond
3 Lotus Petal

Sideboard (15)
4 Barrowgoyf
1 Bitter Triumph
1 Duress
2 Fatal Push
1 Flusterstorm
2 Force of Negation
1 Hide on the Ceiling
3 Murktide Regent
```

### 7. Omni-Show (Show and Tell / Omniscience)

Source: [Riccardo Saino, 5th-8th, 1 Tappa Tigullio League 2026/27, 2026-09-27 (23 players)](https://mtgtop8.com/event?e=91360&d=893866&f=LE)

```
Main (60)
4 Ancient Tomb
1 Forest
1 Hedge Maze
3 Island
1 Mistrise Village
4 Misty Rainforest
2 Scalding Tarn
2 Tropical Island
1 Atraxa, Grand Unifier
2 Emrakul, the Aeons Torn
1 Auroral Procession
4 Brainstorm
3 Cunning Wish
4 Force of Will
1 Intuition
1 Planar Genesis
4 Ponder
4 Show and Tell
1 Sink into Stupor
2 Spell Pierce
4 Stock Up
4 Veil of Summer
2 Lotus Petal
4 Omniscience

Sideboard (15)
3 Carpet of Flowers
2 Consign to Memory
2 Disruptor Flute
1 Dress Down
1 Echoing Truth
1 Firemind's Foresight
1 Grafdigger's Cage
1 Heritage Reclamation
1 Shared Summons
1 Sublime Epiphany
1 Surgical Extraction
```

### 8. Reanimator

Source: [pacoelflaco, 2nd, MTGO Challenge 32, 2026-09-26](https://mtgtop8.com/event?e=91314&d=893501&f=LE)

```
Main (60)
2 Badlands
1 Bloodstained Mire
2 Marsh Flats
2 Polluted Delta
1 Raucous Theater
2 Swamp
1 Undercity Sewers
1 Underground Sea
2 Verdant Catacombs
2 Archon of Cruelty
3 Atraxa, Grand Unifier
3 Griselbrand
1 Koma, World-Eater
2 Raph & Mikey, Troublemakers
1 Cabal Therapy
2 Collective Brutality
4 Dark Ritual
4 Faithless Looting
4 Reanimate
4 Shallow Grave
4 Thoughtseize
4 Unmask
4 Animate Dead
4 Lotus Petal

Sideboard (15)
1 Collective Brutality
1 Koma, World-Eater
1 Magus of the Moon
2 Massacre
4 Show and Tell
4 Stronghold Gambit
2 Surgical Extraction
```
