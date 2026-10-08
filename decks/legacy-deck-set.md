# Legacy deck set: final eight, built from real tournament lists

_Prepared 2026-10-01 for Brady to edit. Banlist as of 2026-08-10. Forge master as of 2026-09-30. Earlier drafts are in `superseded-v1/` (my own builds, including Jund) and `superseded-v2/` (the version with UR Delver and Omni-Show)._

## The set

| # | Deck | Type | Meta evidence | Source list | Forge-AI-unsupported cards in main |
|---|------|------|---------------|-------------|------|
| 1 | UR Cutter (Murktide / Cori-Steel Cutter tempo) | Tempo (non-combo) | UR Tempo 4% on mtgtop8 2-month and 5.6% on AetherHub; Izzet Delver 4.2% on MTGGoldfish | [hejljud, 4th, MTGO League, 2026-09-22](https://mtgtop8.com/event?e=91149&d=892306&f=LE) | Mishra's Bauble (4) |
| 2 | Alurentell (Aluren + Show and Tell + Omniscience) | Combo (Aluren / cheat big permanents) | Aluren 4% on mtgtop8 2-month (all recent Aluren lists are this hybrid); Aluren 3.9% on MTGGoldfish | Brady's personal list, 2026-10-06 (was Freqing, 1st, MTGO League, 2026-09-27) | City of Traitors (1) |
| 3 | Boros Aggro (Energy) | Aggro (non-combo) | Boros Energy #3 on MTGGoldfish (6.3%); Ocelot Aggro 9.7% on MTGO tier list; Boros Aggro 6% on mtgtop8 2-month | [Fenrir18, 13th, MTGO Challenge 32, 2026-09-27 (65 players)](https://mtgtop8.com/event?e=91385&d=894044&f=LE) | Goblin Bombardment (3) |
| 4 | BW Death and Taxes | Midrange / taxes (non-combo) | Death and Taxes (Yorion) BW #4 on MTGGoldfish (5.3%); BW D&T 3.4% AetherHub; D&T 4% mtgtop8 2-month | [Federodi, 5th-8th, 1 Tappa Tigullio League 2026/27, 2026-09-27 (23 players)](https://mtgtop8.com/event?e=91360&d=893865&f=LE) | none |
| 5 | Dimir Tempo | Tempo (non-combo) | #1 on MTGGoldfish (10.3%), MTGO tier list (10.1%), AetherHub (7.5%), mtgtop8 2-month (11%) | [_GhostWalking_, 14th, MTGO Challenge 32, 2026-09-27 (65 players)](https://mtgtop8.com/event?e=91385&d=894054&f=LE) | Nethergoyf (4), Mishra's Bauble (3) |
| 6 | UW Phelia (Phelia / Riddler / Stifle; formerly "UWx Control", file stem `uwx-control`) | Control-tempo (non-combo) | UWx Control 9% on mtgtop8 2-month (#2); Uwx Phelia #3 on AetherHub (6.25%); Azorius Tempo 2.2% on MTGGoldfish | [HJ_Kaiser, 5th-8th, MTGO Challenge 32, 2026-09-27 (65 players)](https://mtgtop8.com/event?e=91385&d=894042&f=LE) | Prismatic Ending (2), Stifle (4) |
| 7 | Doomsday | Combo (pile / Oracle) | #5 on MTGGoldfish (5.1%); 6.5% and best win rate (65.8%) on MTGO tier list; 4% mtgtop8 2-month | [thescuba96, 2nd, MTGO Challenge 32, 2026-09-27 (65 players)](https://mtgtop8.com/event?e=91385&d=894043&f=LE) | Doomsday (4), Edge of Autumn (2), Lion's Eye Diamond (1) |
| 8 | Reanimator | Combo (graveyard) | Reanimator 4% on mtgtop8 2-month; Rakdos Reanimator 3.1% on MTGGoldfish; 4.1% on MTGO tier list | [pacoelflaco, 2nd, MTGO Challenge 32, 2026-09-26](https://mtgtop8.com/event?e=91314&d=893501&f=LE) | Cabal Therapy (1), Faithless Looting (4), Unmask (4) |
| 9 | Storm (Beseech Storm), added 2026-10-06 | Combo (LED / Wish storm) | Storm 3% on mtgtop8 2-month; 14 distinct Storm-family entries on mtgtop8 between 2026-09-22 and 2026-10-04, 9 of them this Beseech Storm variant (2 were TES) | Brady's own 75 (2026-10-06); main identical to [Otazz, 9th, MTGO Challenge 32, 2026-10-02 (53 players)](https://mtgtop8.com/event?e=91634&d=895876&f=LE) | not checked yet |
| 10 | Colorless Tron (Karn / Ugin / Tezzeret / The One Ring + Trinisphere), added 2026-10-06 | Prison-ramp (colorless Tron lands) | Not a meta pick: Brady's own list, added as an Alurentell opponent | Brady's personal list, 2026-10-06 (not a tournament list) | not checked (Forge does not pilot it); 27 of its 32 distinct cards are missing from the Rust engine, see `tron-engine-gap.md` |

**Pool:** 144 unique non-basic cards, 45 of them shared by two or more decks. That is below your 200 to 300 estimate. Each list is one real tournament list, not an average, so it reflects one pilot's choices; tune them if you want a more typical build.

**Spread:** five non-combo decks (aggro, taxes-midrange, two tempo decks, tempo-control) and three combo decks (Alurentell, Doomsday, Reanimator). There is no pure draw-go control deck: the UW Phelia list is a tempo-control deck built on Phelia, Quantum Riddler, Tamiyo and Stifle, which is what UW Phelia currently means in Legacy. Storm (Beseech Storm) was added as a ninth deck on 2026-10-06 at Brady's request, as an Alurentell opponent, using his own 75; it needs new engine cards, see `storm-engine-gap.md`. A TES list is kept in `storm-tes-alt.txt`. Colorless Tron was added as a tenth deck on 2026-10-06, also at Brady's request as an Alurentell opponent, using his own 75 (checked card by card; see `tron-engine-gap.md` for the engine gap). The 144-card pool count above predates decks 9 and 10: Tron adds 27 unique non-basic cards the engine lacks and shares 5 with the pool (Ancient Tomb, Karakas, Boseiju, Disruptor Flute, Grafdigger's Cage).

## Matchups

1. **UR Cutter vs Alurentell (your pick).** Do not let the Forge AI pilot Alurentell: it has no logic for the Aluren or Show and Tell combo, so a Forge-AI Alurentell would mostly sit there and flatter any win rate. For a Forge baseline, put the AI on the UR Cutter side and have our player pilot Alurentell.
2. **Boros Aggro vs BW Death and Taxes.** The cleanest pipeline test for the Forge-AI milestone: both are creature-combat decks with little stack interaction, BW D&T has no AI-unsupported card in its main deck, and Boros has one (Goblin Bombardment).
3. **Dimir Tempo vs UW Phelia.** The stack and priority stress test (Force of Will, Daze, Stifle, Brainstorm, Bowmasters). Nethergoyf, Mishra's Bauble, Stifle and Prismatic Ending are the AI-unsupported cards in play.

The other combo decks, Doomsday and Reanimator, are for our players to pilot against the non-combo decks, not for the Forge AI.

## UR Cutter vs Alurentell notes

- **Alurentell is my reading of your name:** the Aluren and Show and Tell hybrid. Every Aluren list mtgtop8 shows for 2026-09-22 to 09-28 (Freqing, Zavadil, Minoue Hiroyuki, Kheldarion, Pikolay) is this same shell, and two web results use the name "Aluren-Tell".
- **UR Cutter:** hejljud's list has no Delver of Secrets. Bruno Lorenzato's UR Cutter list ([source](https://mtgtop8.com/event?e=91364&d=893897&f=LE), 24 players, 2026-09-27) differs by running Tamiyo and Flow State instead of Preordain and Prismari Charm.
- **Interactions to test first:** Force of Will and Daze on both sides; Wasteland on Ancient Tomb and City of Traitors; Show and Tell with both players putting a permanent in (choices hidden from each other); Aluren usable by any player, so UR Cutter can cast Dragon's Rage Channeler for free at instant speed; Acererak looping with Aluren through Tomb of Annihilation (Forge has dungeon support in `VentureEffect.java`, untested here); Atraxa's reveal-ten trigger; Omniscience free casts; Murktide Regent's delve; Cori-Steel Cutter's second-spell Monk trigger; Stock Up.

## Meta behind the picks

| Source | Window | Top of the format |
|---|---|---|
| MTGGoldfish, paper and MTGO | last 30 days | Dimir Tempo 10.3%, Eldrazi 6.8%, Boros Energy 6.3%, BW Death and Taxes 5.3%, Doomsday 5.1%, Tron 5.0%, Izzet Delver 4.2%, Sewer-Cam 4.1%, Aluren 3.9% |
| mtgtop8, 481 decks | last 2 months | Dimir Tempo 11%, UWx Control 9%, Boros Aggro 6%, Tron 5%, Eldrazi 4%, UR Tempo 4%, D&T 4%, Show and Tell 4%, Doomsday 4%, Aluren 4%, Reanimator 4%, Lands 4%, Storm 3% |
| AetherHub | last 180 days | Dimir Tempo 7.5%, Karn Forge 6.5%, UWx Phelia 6.3%, UR Tempo 5.6%, Artifacts Blue 5.0%, Doomsday 5.0% |
| decklistdata, MTGO | last 14 days | Dimir Tempo 10.1%, Ocelot Aggro 9.7%, Doomsday 6.5% (win rate 65.8%), Eldrazi 6.5%, OmniTell 6.0% |

One other tracker I read earlier (MTGDecks) showed Reanimator at 14% and Goblins in the top three, which none of these four support, so I treated it as an outlier.

## Legality

No card in any main deck or sideboard is on the Legacy banlist (Wizards' list as of 2026-08-10, 73 named cards, cross-checked with ScrollVault). Every deck is exactly 60 main and 15 side with no more than four copies of any card. All card names match a Forge card script, which is a useful extra check that my transcription did not invent names. I read the lists through a page-summarizing fetch tool, not a raw export, so a miscopied quantity that still totals 60 is possible; spot-check against the linked pages before treating any list as exact.

## Forge check

**Implemented:** all 144 non-basic cards have a Forge script (`forge-gui/res/cardsfolder`, master as of 2026-09-30). Per-card details are in [forge-card-coverage.csv](/mnt/project-files/decks/forge-card-coverage.csv).

**AI flags:** Forge's docs say `AI:RemoveDeck:All` marks cards the AI cannot use or can only use badly, while `Random` only means the card is too narrow for random decks. In this pool 16 cards carry `All` and 18 carry `Random`. Only `All` matters for AI strength.

**Not verified:** a script existing is not proof of correct behavior. I did not build or run Forge, and Forge's own unit tests name only 15 of these 144 cards. I read the scripts of about 25 of the riskiest cards (Stifle, Nethergoyf, Barrowgoyf, Moonshadow, Flow State, Cunning Wish, Omniscience, Aluren, Acererak, Atraxa, Stock Up, Phelia, Quantum Riddler, Shallow Grave, Stronghold Gambit, Unmask, Personal Tutor, Cori-Steel Cutter, Raph & Mikey, Dragon's Rage Channeler, Unholy Heat, Prismari Charm, City of Traitors) and saw no clear mismatch with their text. That is a weak signal.

**Known Forge bugs:** I could not get a list. Forge's GitHub issue search is blocked to my fetch tool by robots.txt and the API is not enabled for this session. Treat bug status as unchecked.

**Risks specific to this set:**
- **Aluren is "any player" and lets either player cast creatures with mana value 3 or less for free at instant speed.** Forge's script extends the permission to several zones with a flag meant to avoid granting new zone permissions; worth a scripted test.
- **Acererak ventures into a dungeon.** The action API has to expose dungeon room choices.
- **Stifle (UW Phelia) targets triggered and activated abilities.** The legal-action enumerator has to expose triggers on the stack as targets.
- **Newer and Universes Beyond cards** (Spider-Woman, Raph & Mikey, Bilbo, Samwise, Kaito, Cloak and Dagger) are the least battle-tested scripts. Most are one-of or two-of cards, but they are a good place to look for rules bugs.
- **Forge has a headless `sim` mode** (`forge-gui-desktop/.../view/SimulateMatch.java`) for the Phase 1 runner; JDK 21 is installed here. Games per second not measured.

## Not included, in case you want to swap

Eldrazi Aggro (#2 on MTGGoldfish), Tron and Karn Forge, Artifacts Blue (won MTGO Challenge 32 on 2026-09-27), Lands, Sewer-Cam Combo, Hogaak. I did not pull lists for them. Artifacts Blue and Lands are the two I would look at first for a colorless or prison-style deck.

## Decklists

Also saved one per file in this folder, with source and link in the header.

### 1. UR Cutter (Murktide / Cori-Steel Cutter tempo)

Source: [hejljud, 4th, MTGO League, 2026-09-22](https://mtgtop8.com/event?e=91149&d=892306&f=LE)

```
Main (60)
2 Flooded Strand
1 Island
2 Misty Rainforest
2 Polluted Delta
2 Scalding Tarn
1 Thundering Falls
4 Volcanic Island
4 Wasteland
4 Dragon's Rage Channeler
4 Murktide Regent
4 Brainstorm
4 Daze
4 Force of Will
4 Lightning Bolt
4 Ponder
3 Preordain
1 Prismari Charm
2 Unholy Heat
4 Cori-Steel Cutter
4 Mishra's Bauble

Sideboard (15)
2 Hydroblast
2 Meltdown
2 Null Rod
4 Pyroblast
2 Pyroclasm
3 Surgical Extraction
```

### 1b. UR Cutter alternate (Brady's list, added 2026-10-07)

Source: Brady's own list, posted 2026-10-07 (not a tournament list; `ur-cutter-alt.txt`). Rarely played; Brady thinks it is better than the main list above. It is an alternate opponent only: `ur-cutter.txt` stays the default UR deck (deck 1). Brady (2026-10-07): a comparison between the two UR lists (differential tests) comes later, once LLM vs LLM testing is proven working. Differences from the main list: no Murktide Regent, Preordain or Prismari Charm; adds 4 Stormchaser's Talent, 3 Boomerang Basics, 4 Flow State, 1 Mountain, 1 Bloodstained Mire, 1 Flooded Strand; sideboard swaps in Torpor Orb, Consign to Memory, Force of Negation, Price of Progress, Tormod's Crypt, Pyrokinesis. The three cards the engine lacked (Stormchaser's Talent, Pyrokinesis, Price of Progress) were added by RFC 0009 on 2026-10-08, so the deck loads and plays in the live engine (package: `rust-engine-ur/`).

```
Main (60)
4 Dragon's Rage Channeler
1 Thundering Falls
1 Flooded Strand
4 Force of Will
4 Lightning Bolt
4 Stormchaser's Talent
1 Misty Rainforest
4 Scalding Tarn
3 Boomerang Basics
4 Ponder
4 Wasteland
4 Flow State
1 Mountain
1 Island
1 Polluted Delta
4 Mishra's Bauble
4 Brainstorm
3 Cori-Steel Cutter
2 Daze
1 Unholy Heat
4 Volcanic Island
1 Bloodstained Mire

Sideboard (15)
2 Hydroblast
1 Torpor Orb
2 Pyroblast
1 Consign to Memory
1 Force of Negation
2 Price of Progress
2 Tormod's Crypt
2 Pyrokinesis
2 Unholy Heat
```

### 2. Alurentell (Aluren + Show and Tell + Omniscience)

Source: Brady's personal list, 2026-10-06 (not a tournament list; replaces the Freqing 1st-place list of 2026-09-27, kept in `superseded-v3/`). The engine needs Orim's Chant for the sideboard (RFC 0005); Savannah was already in the pool.

```
Main (60)
4 Ancient Tomb
1 Boseiju, Who Endures
1 City of Traitors
3 Flooded Strand
2 Hedge Maze
1 Island
4 Misty Rainforest
1 Savannah
1 Tundra
2 Tropical Island
4 Acererak the Archlich
4 Atraxa, Grand Unifier
4 Brainstorm
4 Force of Will
4 Ponder
4 Show and Tell
4 Stock Up
2 Veil of Summer
4 Aluren
4 Lotus Petal
2 Omniscience

Sideboard (15)
4 Carpet of Flowers
1 Force of Negation
2 Consign to Memory
2 Faerie Macabre
2 Orim's Chant
2 Prismatic Ending
2 Veil of Summer
```

### 3. Boros Aggro (Energy)

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

### 4. BW Death and Taxes

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

### 5. Dimir Tempo

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

### 6. UW Phelia (Phelia / Riddler / Stifle; file stem `uwx-control`)

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

### 7. Doomsday

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

### 9. Storm (Beseech Storm)

Source: Brady's own list, 2026-10-06; main matches [Otazz, 9th, MTGO Challenge 32, 2026-10-02](https://mtgtop8.com/event?e=91634&d=895876&f=LE) and yonthan910, 1st, MTGO League, 2026-09-23. Only the sideboard differs (Veil of Summer instead of Carpet of Flowers). Not yet in the Rust engine (see `storm-engine-gap.md`).

```
Main (60)
1 Badlands
1 Bloodstained Mire
1 Raucous Theater
1 Scalding Tarn
1 Taiga
1 Underground Sea
4 Urza's Saga
2 Verdant Catacombs
4 Hexing Squelcher
1 Runehorn Hellkite
3 Beseech the Mirror
4 Burning Wish
4 Dark Ritual
3 Echo of Eons
1 Gaea's Will
4 Gamble
1 Tendrils of Agony
2 Veil of Summer
4 Chrome Mox
4 Giant's Boulder
4 Lion's Eye Diamond
4 Lotus Petal
4 Mox Opal
1 Song of Creation

Sideboard (15)
1 Beseech the Mirror
4 Boomerang Basics
2 Boseiju, Who Endures
1 Echo of Eons
1 Empty the Warrens
1 Haywire Mite
1 Peer into the Abyss
1 Thoughtseize
1 Tendrils of Agony
2 Veil of Summer
```

### 10. Colorless Tron

Source: Brady's own list, 2026-10-06 (not a tournament list). Mostly not in the Rust engine yet (see `tron-engine-gap.md`).

```
Main (60)
4 Ancient Tomb
1 Karakas
2 Boseiju, Who Endures
4 Planar Nexus
4 Urza's Saga
4 Urza's Tower
4 Urza's Workshop
4 Karn, the Great Creator
4 Ugin, Eye of the Storms
4 Tezzeret, Cruel Captain
4 The One Ring
4 Trinisphere
4 Kozilek's Command
4 Grim Monolith
2 Manifold Key
1 Voltaic Key
2 Mishra's Research Desk
1 Pithing Needle
1 Disruptor Flute
1 Portable Hole
1 Expedition Map

Sideboard (15)
1 Portable Hole
1 Tormod's Crypt
1 Grafdigger's Cage
1 Eldrazi Confluence
1 Liquimetal Coating
1 Torpor Orb
3 Warping Wail
1 Ensnaring Bridge
2 Argentum Masticore
1 Mycosynth Lattice
1 Extinguisher Battleship
1 Summon: Bahamut
```
