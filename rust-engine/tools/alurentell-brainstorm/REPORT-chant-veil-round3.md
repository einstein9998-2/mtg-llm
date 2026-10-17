# Two Orim's Chant or two Veil of Summer in the main deck, another round against blue and non-blue decks (2026-10-17)

Third round of the Chant-main vs Veil-main comparison (`REPORT-chant-veil.md` was UR, UB, UW; `REPORT-chant-veil-nonblue.md` was Boros, Storm, colorless Tron). Brady's note after round 2: Karn stopping Lotus Petal is correct, run some more tests against blue and non-blue, and build the most popular Tron deck, Blue Tron, and play Chant main against it. Same lists, same layout as before: game 1 (nobody sideboarded), Sonnet agents on both seats, same seed and same play/draw for game n in both lists. This round is 2 games per opponent per list (12 per list, 24 games), six opponents, new seeds (9301-9312, none reused from rounds 1 and 2), one game on the play and one on the draw per opponent.

- **Veil list** (`chantveil-games/arms/cur/alurentell.txt`): 2 Veil of Summer main.
- **Chant list** (`chantveil-games/arms/brady/alurentell.txt`): 2 Orim's Chant main instead of the Veil, 4 Veil in the sideboard, 1 Faerie Macabre turned into a Forest.
- Opponents: `ur-cutter`, `dimir-tempo`, `uwx-control`, `boros-aggro`, `storm`, and the new `blue-tron` (below).

## Blue Tron list
Source: fujimori noriki, Mono Blue Tron, 1st of 18, 21st Legacy KAMUI Decision Battle, 2026-10-11, via metagame.info (the same page lists Mono Blue Tron at several other events between 10-04 and 10-13). The engine does not know four of its cards, so the list in `chantveil-round3-games/decks/blue-tron.txt` differs from the real one:

| Real list | In the engine |
|---|---|
| 3 Sink into Stupor | 3 Consign to Memory (4 in the 75) |
| 4 Petrified Hamlet, 1 Abstergo Entertainment | 5 more Island (6 total) |
| Sideboard Firespout | Sideboard Dismember |

Everything else is as played: 4 Karn the Great Creator, 3 Ugin Eye of the Storms, 4 Force of Will, 4 Force of Negation, 4 Kozilek's Command, 2 Eldrazi Confluence, 4 Stock Up, 4 The One Ring, 3 Expedition Map, 3 Grim Monolith, 2 Giant's Boulder, 4 Planar Nexus, 4 Tower, 3 Mine, 3 Power Plant. Planar Nexus is the only way it makes blue besides the Islands. Whether it is "by far the most popular Tron deck" I took from Brady; the event list above is mono-blue Tron-heavy but I did not count the field.

## Result
| Opponent | Veil main | Chant main |
|---|---|---|
| UR Cutter | 0-2 | 2-0 |
| UB Dimir | 2-0 | 1-1 |
| UW Phelia | 2-0 | 2-0 |
| Boros | 1-1 | 0-2 |
| Storm | 2-0 | 2-0 |
| Blue Tron | 2-0 | 2-0 |
| **Total** | **9-3** | **9-3** |

Level. By blue and non-blue (UR, UB, UW and Blue Tron are the 8 blue games per list; Boros and Storm the 4 non-blue): Veil 6-2 and 3-1, Chant 7-1 and 2-2.

All three rounds (36 games per list, 72 in all):

| Opponent | Veil main | Chant main |
|---|---|---|
| UR Cutter (rounds 1, 3) | 1-5 | 5-1 |
| UB Dimir (1, 3) | 5-1 | 4-2 |
| UW Phelia (1, 3) | 6-0 | 3-3 |
| Boros (2, 3) | 4-2 | 4-2 |
| Storm (2, 3) | 6-0 | 6-0 |
| Colorless Tron (2) | 2-2 | 3-1 |
| Blue Tron (3) | 2-0 | 2-0 |
| **Total** | **26-10** | **27-9** |

One game apart over 36 per list is still "no difference found". The opponent splits keep pointing opposite ways (UR for Chant by four games, UW for Veil by three) and they are small samples with different play/draw each round, so I would not read them as real.

## What the cards did
**Chant list: drawn into hand in 4 of 12, cast in 2, record 1-3 when drawn and 8-0 when not.**
- b7 (Boros, loss) and b8 (Boros, loss): cast kicked on the opponent's turn 4 as a fog while they attacked. In b7 it stopped the attack and their spells that turn, but the player says it should have been held for the combo turn: Swords to Plowshares exiled the Acererak on turn 5. In b8 it stopped about 10 damage, bought one turn and the player had no blocker or combo turn to use it on.
- b4 (UB, loss): in the opening seven, never cast. Awkward all game: WW kicked, a lone Tundra for white (Wasteland took Savannah), and the combo turn was one mana short of Aluren plus Chant.
- b9 (Storm, win): drawn off a Ponder, held with Savannah open, never needed.
- Seen in a reveal, a Ponder or a Stock Up but not drawn: b1, b2, b3, b11, b12.
- The round-2 casts (b1, b3, both Boros wins) were unkicked before Aluren with the opponent tapped out, which stopped Swords and the free creatures. This round's two casts were the other use, a fog on the opponent's turn, and both games were lost. The cast is better held for the combo turn.

**Veil list: drawn into hand in 6 of 12, cast in 2, record 5-1 when drawn and 4-2 when not.**
- c6 (UW, win): cast as a cantrip in response to their Stifle on the player's own Flooded Strand, which drew a card (a second Acererak). It protected nothing.
- c11 (Blue Tron, win): cast before Aluren with exactly five mana; resolved, and drew no card because they cast no blue spell into it.
- c7 (Boros, win) and c8 (Boros, loss): in the opening seven, a blank against Boros, held or put back with Brainstorm. c4 (UB, win) and c10 (Storm, win): in hand, never cast; c10 bottomed it on a mulligan.
- Not in hand: c1, c2, c3, c5, c9, c12 (c9 saw one at the bottom of Stock Up).
- Veil never stopped anything that would have beaten the combo in these 12 games.

## What the players said
Each Alurentell player was asked whether they would rather have the other card.

| | Prefer the other card | Keep what they had | No difference |
|---|---|---|---|
| Veil list, blue (8) | c1, c11 (Chant) | c2, c3, c4, c5, c6, c12 (Veil) | - |
| Veil list, Boros and Storm (4) | c7, c9, c10 (Chant) | - | c8 |
| Chant list, blue (8) | b1, b3, b5, b6, b11, b12 (Veil) | b2 (Chant) | b4 |
| Chant list, Boros and Storm (4) | - | b7, b8, b9, b10 (Chant) | - |

It splits cleanly on the opponent. Against the four blue decks 12 of the 16 players want Veil (six Chant-list players asked for it and six Veil-list players kept it), 3 want Chant and 1 sees no difference. Against Boros and Storm every player who held a view wants Chant (7 of 8; c8 sees no difference).
- **Veil against blue (UR, UB, UW, Blue Tron):** it is green, so it comes off Tropical, Hedge Maze, Carpet or Petal; it beats Force of Will, Force of Negation, Daze and Consign when cast first; and it draws a card when the opponent counters it. Chant needs W from Savannah or Tundra, and white was missing or Wastelanded in b1, b3, b4, b5 and b12.
- **Chant against Boros and Storm:** Veil is a blank in both (c7, c8, c9, c10, b7 to b10). A kicked Chant on their turn or in response to a ritual is the only tool the Storm players named. Against Boros it stops Swords and the free creatures under Aluren, but not Goblin Bombardment, Karakas or Quarry, which are abilities (b7).
- **Blue Tron:** Veil beat the free counters (b11, b12 said so outright), and no player lost to a Chant or Veil decision. The player who picked Chant (c11) wanted it for Ugin's cast trigger and Eldrazi Confluence, neither of which Veil stops.

## Losses
- c1 (UR): long game against Murktide, Dragon's Rage Channeler and Prismari Charm; their Force of Will answered the late Show and Tell. Veil never drawn.
- c2 (UR): mulligan to six, Wasteland on the tapped Hedge Maze, Daze and Force of Will on the turn-4 Show and Tell. Veil never drawn.
- c8 and b8 (Boros): one-land keeps on the draw against the playbook (OH5), Ponder and Brainstorm for two turns while Ajani, Cats and Guide of Souls built up. Neither card stops that.
- b7 (Boros): the fog Chant on turn 4, covered above.
- b4 (UB): Chant dead, no mana, Kaito and Moonshadow.

## Caveats
- 12 games per list, 2 per opponent; Wasteland, mulligans and one-land keeps decided several games on both sides. The Blue Tron list is a substitute as above and its pilot was Sonnet with no playbook: in c11 it tapped out for Ugin and left an Ugin cast trigger unused on an Eldrazi Confluence, and in b11 it tapped out for Ugin at 10 life with The One Ring at three burden counters and was drained by a second Aluren.
- Only the first game with the unboarded decks, as before. Both lists reach the same 60 after sideboarding against these decks.
- Storm and Blue Tron were mostly decided before a protection card could matter (Storm never went off in any of the 4 games, and in the Blue Tron games the free counters were handled with Force of Will).

## Engine and menu items
- **Engine card pool.** Probing the engine binary with one-card decks (2026-10-17) found these unknown: Sink into Stupor, Petrified Hamlet, Abstergo Entertainment, Firespout, Counterspell, Mana Drain, Memory Lapse, Mana Leak, Spell Pierce, Annul, Mulldrifter, Thought Monitor, Mystic Sanctuary, Walking Ballista, Wurmcoil Engine, Treasure Cruise, Thirst for Knowledge, Mind Stone, Chromatic Star, Sylvan Scrying, Ancient Stirrings, Karn Liberated, Oblivion Stone, Myr Battlesphere. That rules out most of the other Tron lists and any blue deck built on classic counters.
- **Windows.** b9 on the Storm side lost its Main 2 after Gamble (no Main 2 stop, same pattern as the Lion's Eye Diamond turns in round 2). b9's own Alurentell player had no upkeep stop on the opponent's turn, so a "Chant in their upkeep" plan is not playable; the only windows came once a Saga ability or Gamble was on the stack.
- **Menus.** Legend-rule prompts list both copies with identical labels (seen again with The One Ring); Kaito's menu indices shift between prompts and one pick used the wrong ability (b4 opponent); Ugin tapped all four lands when the player named three (b11 opponent, one report); `;` and `&` in a pick reason run as shell text (the pick still goes through).
- **Not bugs, checked:** the second Daze on Show and Tell in b1 was paid (the Alurentell player paid both), and Karn stopping Petal is correct per Brady.

## Files
`chantveil-round3-games/`: 48 reports (`cxcurNg1a` = Veil list Alurentell player, `cxcurNg1b` = opponent, `cxbradyN...` for the Chant list; N = 1-2 UR, 3-4 UB, 5-6 UW, 7-8 Boros, 9-10 Storm, 11-12 Blue Tron), `decks/` (the Blue Tron, Storm and colorless Tron lists used), and `mkcx.sh <cur|brady> <n> <A|B>` (launcher; expects the `llm2` working directory and reuses the lists and brief in `chantveil-games/`).
