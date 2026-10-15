# Playing around Wasteland vs color requirements, game 1 (2026-10-15)

Brady's question: how much does playing around Wasteland get in the way of the deck's color requirements? Test: the stock Alurentell main with no white, then one Island swapped for a third Tropical Island, then a second Island swapped for an eighth fetch. Game 1 only (60-card mains), 12 LLM-vs-LLM games per list (Sonnet on both seats, the usual menus and playbook), against UR Cutter, Dimir Tempo and UW Phelia (4 each, two on the play and two on the draw). Same seed per game number in every list, so the three lists see the same opponent, seat and shuffle seed (the in-slot land swaps keep slot order, so openings are often identical but diverge once a fetch or a different draw changes the library).

All three opponents run 4 Wasteland and Daze or Force of Will; UW also has Stifle. The players answered a fixed "MANA" section after each game (first blue, first two green sources, Wasteland seen/played around/cost, any color-blocked turn, would a different mix help), and I pulled the same facts out of the engine transcripts (`landstats.py`).

## The three lists
| | Basics | Duals | Fetches | Rest |
|---|---|---|---|---|
| **3 basics** | Forest, Island, Island | 2 Tropical, 2 Hedge Maze | 7: 4 Misty, Delta, Strand, Vista | 4 Tomb, City, Boseiju |
| **Third Trop** | Forest, Island | **3 Tropical**, 2 Maze | 7 | same |
| **Eighth fetch** | Forest | 3 Tropical, 2 Maze | **8**: adds a 2nd Strand | same |

20 lands each, 40 other cards identical. Aluren needs {2}{G}{G}; blue comes from Island, Tropical, Maze; there is no black source, so Acererak is only ever cast free.

**Deck note:** `decks/alurentell.txt` in the repo currently is the white version (3 Strand, Savannah, Tundra). I read "stock list, no white, 3 basics, 7 fetches, 2 Trop" as the older Freqing-list build used in the earlier tests (`analysis/city-vs-tomb-old-build/llm-games/decks-4t1c/alurentell.txt`, comments stripped), which has exactly those counts. If you meant something else, this is the thing to redo. Also my choices: both cuts come out of the Islands (Forest stays, since green is the harder color) and the eighth fetch is a second Flooded Strand (Island-typed Tundra/Savannah aren't in this build).

## Result
| Opponent | 3 basics | Third Trop | Eighth fetch |
|---|---|---|---|
| UR Cutter | 2-2 | 3-1 | 2-2 |
| UB Dimir | 1-3 | 4-0 | 1-3 |
| UW Phelia | 3-1 | 2-2 | 4-0 |
| **Total** | **6-6** | **9-3** | **7-5** |

Twelve games cannot separate these (95% interval is about ±28 points per list, and 9-3 vs 6-6 is within noise). Games that ended on Alurentell's turn 3 or earlier are all wins by Show and Tell combo before Wasteland matters: 4 / 6 / 3 of them. The remaining games, where the mana actually got played, went **2-6 / 3-3 / 4-5**, which is the part to look at, and it is also too small to rank anything.

## What Wasteland actually did
| | 3 basics | Third Trop | Eighth fetch |
|---|---|---|---|
| Wastelands the opponent played | 14 (in 8 games) | 11 (8) | 12 (7) |
| Wastelands that hit one of my lands | 10 | 7 | 11 |
| Targets | 5 Trop, 4 Tomb, 1 Maze | 3 Tomb, 2 Trop, 1 Maze, 1 Boseiju | 4 Trop, 3 Tomb, 3 Maze, 1 Boseiju |
| Of those, a land that makes U or G | 6 | 4 | 8 |
| Turns with no land drop (any reason) / my turns played | 6 / 56 | 3 / 50 | 15 / 57 |
| Games with GG never or only from turn 5 (of games lasting 4+ turns) | 2 of 8 | 2 of 6 | 2 of 9 |
| Average turn of first GG (games that got it) | 3.8 | 3.6 | 3.1 |

- Wasteland is mostly a **mana** problem here, not a color one: 10 of the 28 hits took an Ancient Tomb (colorless), and the player answers name a missing color as the blocker in only about 5 of 36 games (my tally from their MANA text: 1 / 2 / 2 across the lists, e.g. 3 basics game 1, Third Trop games 10 and 11, Eighth fetch games 1 and 6). In nearly all the others the line is "total lands" (a Tomb or Trop died and the deck was left on two lands), which Wasteland causes in any list.
- Tropical Island is the land the opponents pick: 11 of 28 hits. That comes with having a third copy, but the opponent also simply kills the first nonbasic it sees (Maze, Boseiju, Tomb).
- Games where a Wasteland hit: 10-9. Games with no hit: 12-5. Suggestive, but quick Show and Tell wins (which end before a Wasteland exists) are in the second group.
- 9 of the 14 losses had a Wasteland hit in them; the 5 that didn't were Nethergoyf / Murktide tempo starts on the other side (3 basics games 4, 7, 8; Eighth fetch game 4; Third Trop game 1).
- UW is where fetches hurt most: Stifle on a fetch trigger cost a land in 3 basics game 11 and Third Trop games 10 and 11 (three Stifles in one game), with a Wasteland on the only dual then finishing the job. This is not a Wasteland-color effect but it is the same "fetch-heavy mana is fragile" problem.

## How the players played around Wasteland
Passively, almost always. Nobody skipped a land drop in order to dodge it unless they had nothing to cast:
- **Fetches as decoys**: keep Misty/Strand/Vista uncracked through the opponent's turn and crack at their end step (cost nothing but life, and it works because Wasteland shows up as a land drop and is often spent immediately). Mentioned in 15 of the 36 Alurentell reports.
- **Holding the second Tomb or the Trop in hand** until the combo turn, or until the opponent is tapped out. Cost: usually nothing; in a few games it cost a turn of mana (for example Third Trop game 2 and Eighth fetch game 2, where Tomb was held a turn).
- **Fetch basics, not duals** once a Wasteland was on the table (Forest/Island via Misty). This is where the eighth fetch and the third Trop pull in opposite directions: Basics are Wasteland-proof, duals are not, but a fetch can find either.
- Cases where they could not play around it: a one-land hand, or the Tomb was the land needed for a turn-2 Show and Tell (3 basics 9, Third Trop 9; both fine because a Petal replaced it).
- Mistakes the players flagged themselves: tapping the last spare land into an open Wasteland as Daze money (Third Trop game 7), fetching Forest and then being unable to cast the second Brainstorm (3 basics game 10), and keeping a one-land seven against four Wastelands (Eighth fetch game 8).

Eighth fetch has the most turns with no land drop (15 vs 6 and 3) and the highest hit rate per Wasteland played (11 hits from 12), consistent with fewer basics giving Wasteland fewer dead targets. The no-drop count is raw (it also counts turns with no land in hand, and a few held-Tomb turns) so it is a hint, not a measurement, and it comes from a few games (Eighth fetch games 4, 6, 1).

## Read
- Nothing here says a basic is worth more than a Tropical Island: the Third Trop list was the best of the three on results (9-3), had the fewest Wasteland hits (7), and its color-access numbers are not worse than the other two.
- The eighth fetch does not buy anything in these games and costs life, Stifle exposure and more dead-Wasteland targets: 7-5 overall, 4-5 outside the quick wins vs 3 basics at 2-6 is not distinguishable, but its Wasteland and no-drop numbers are the worst. I would not make that swap on this evidence.
- Where Alurentell players asked for a change, it was most often more basics, specifically a second Forest over a Hedge Maze or a fetch (3 basics game 1, Third Trop game 10, Eighth fetch games 2, 8 and 10), not an extra Island. That is a handful of mentions, not a pattern.
- This is a 12-game-per-arm test: if the third Trop vs 3 basics question matters, the cheap next step is the same three lists on the draw against Dimir only, where the lists split 1-3 / 4-0 / 1-3.

## Caveats
- The 1-Petal-postboard question that was in the same message went to the Prismatic Ending vs tempo thread (merged separately as #36), not here.
- Opponents play Wasteland as an ordinary land drop and often fire it at once; no opponent Wasteland was held back for tempo plays. LLM opponents hit the first meaningful nonbasic, which probably over-targets Trop compared with a human.
- "Turns played" is Alurentell's turn count; the stat script identifies Wasteland hits from engine events (the Wasteland activation followed by one of my lands leaving the battlefield), not from the player reports.
- Engine oddities the players noted, unverified here: the menu offers Cast Acererak while no black source exists (3 basics game 11); Ponder's order prompt does not say which pick ends on top (the first pick ends on top); Acererak's "unless that player sacrifices a creature" shows as a bare "Do it?"; the block menu lists a menace attacker as blockable by one creature (Eighth fetch game 12, UW seat); a UW player noted Arid Mesa has no {1} in the engine.

## Per-game results
Each cell: result, Alurentell turns, and which of my lands Wasteland destroyed (WL).

| # | Opp | Seat | 3 basics | Third Trop | Eighth fetch |
|---|---|---|---|---|---|
| 1 | UR | play | L T8 · WL Tomb, Trop, Trop | L T7 | L T8 · WL Tomb, Maze, Trop |
| 2 | UR | draw | W T5 · WL Trop | W T4 · WL Maze | W T4 · WL Maze |
| 3 | UR | play | W T2 | W T2 | W T2 |
| 4 | UR | draw | L T6 | W T3 | L T6 |
| 5 | Dimir | play | W T1 | W T1 | W T1 |
| 6 | Dimir | draw | L T5 · WL Tomb, Trop | W T6 | L T6 · WL Tomb, Trop |
| 7 | Dimir | play | L T5 | W T7 · WL Tomb | L T6 · WL Trop |
| 8 | Dimir | draw | L T4 | W T3 · WL Tomb, Boseiju | L T5 · WL Boseiju, Trop |
| 9 | UW | play | W T3 · WL Tomb | W T3 · WL Tomb | W T3 |
| 10 | UW | draw | W T5 · WL Maze | L T5 · WL Trop | W T4 · WL Maze |
| 11 | UW | play | L T9 · WL Tomb, Trop | L T7 · WL Trop | W T6 · WL Tomb |
| 12 | UW | draw | W T3 | W T2 | W T6 |

Files: `wasteland-games/` holds the 72 player reports (`w<list><N>g1a.md` Alurentell, `...b.md` opponent; lists `b3`, `t3`, `f8`), the three decklists in `arms/`, the brief (`BRIEF-mana.md`), the launcher `mkw.sh`, `landstats.py` with its output `stats.tsv`, and gzipped Alurentell-view transcripts in `transcripts/`. Rerun the stats with `python3 landstats.py transcripts reports`. `mkw.sh` expects the same `llm2` working layout as the earlier waves.
