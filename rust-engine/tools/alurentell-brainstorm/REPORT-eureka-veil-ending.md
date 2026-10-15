# Eureka in place of Veil (game 1) and in place of Prismatic Ending (post-board) (2026-10-15)

Brady's follow-up to the first Eureka test (`REPORT-eureka.md`, 2 Eureka for 2 Ponder): "replace veil instead for preboard and replace ending postboard vs tempo (for eureka)" and "so still 4 veil postboard make sure". Same card scratch implementation as before (`eureka-engine/`), same 12-game shape per wave: 4 games against each of UR Cutter, UB Dimir and UW Phelia, 2 on the play and 2 on the draw, Sonnet agents on both seats. No same-build control arm was run.

## What was played (my reading of Brady's words)
- **Game 1 (`eureka-veil-g1-games/`):** the unboarded 60 with the 2 main-deck Veil of Summer replaced by 2 Eureka. Ponder stays at 4. The two sideboard Veil are still in the sideboard (`alurentell-eureka-for-veil-75.txt`).
- **Post-board (`eureka-ending-post-games/`):** each matchup's usual sideboard plan, with the 2 Eureka coming in instead of the 2 Prismatic Ending the plan normally brings in (`plan-eur3-*.txt`). Veil is 4 (2 main + 2 boarded in) and Ponder is 4. I checked the final 60 of the first game: 4 Ponder, 4 Veil, 2 Eureka, 2 Chant, no Ending.

## Result
| Opponent | Game 1 (2 Eureka for 2 Veil) | Post-board (2 Eureka for 2 Ending) |
|---|---|---|
| UR Cutter | 4-0 | 3-1 |
| UB Dimir | 2-2 | 1-3 |
| UW Phelia | 2-2 | 0-4 |
| **Total** | **8-4** | **4-8** |

Twelve games cannot give a win rate (95% interval about ±28 to 37 points). For scale, the first Eureka test (2 Eureka for 2 Ponder) was 10-2 game 1 and 7-5 post-board, so these two waves are not different from it in any way the data can show. The post-board 4-8 against 7-5 is the largest gap, and it is still inside the noise.

## How often Eureka showed up and what it did
| | In hand (drawn, kept, or in the opening seven) | Cast | Resolved | Game result when cast |
|---|---|---|---|---|
| Game 1 | 4 of 12 (k1, k3, k9, k12; k2 saw one in a Stock Up and bottomed it) | 1 (k3) | 1 | won |
| Post-board | 6 of 12 (m2, m5, m7, m10, m11, m12; m4 only saw one in a Ponder look) | 2 (m5, m12) | 2 | lost, lost |

Three casts in 24 games, all three resolved, no one countered it this time.

- **k3 (UR, game 1, won) is the only game it did what it is for.** Turn 3, the opponent tapped out with five cards in hand. Hard cast with exactly 4 mana ({2}{G}{G} from Hedge Maze, a fetched Tropical Island, Ancient Tomb) and a free Force of Will as the backup. Rounds: Omniscience for a second Dragon's Rage Channeler, then Acererak for a Wasteland, then Misty Rainforest for Polluted Delta and Scalding Tarn (their hand went from 5 to 1), then nobody. Under Omniscience the player cast Stock Up twice, found Aluren, and the Acererak Lost Mine loop killed from 20. The opponent got two Channelers, a Wasteland and two fetches and had one card left to use them. The player would keep 2.
- **m5 (UB, post-board, lost).** Hard cast on turn 5 main 2 with three floating green from Carpet of Flowers plus Tundra. The player held no Atraxa or Acererak (Thoughtseize had taken the Atraxa), so it put in Savannah and a second Carpet and declined the third round; the opponent put in nothing. They died the next turn to four attackers. The UB player on the other seat had taken Atraxa with Thoughtseize and said Eureka was stranded.
- **m12 (UW, post-board, lost).** Both Eureka and Acererak were in the opening seven. Lavinia, Azorius Renegade on their turn 3 taxed every noncreature spell by the lands the Alurentell player controlled, so the first Eureka was cast on turn 8, at 2 life: Force of Will pitching Tamiyo, answered by Veil of Summer, so it resolved. It put in Hedge Maze and Acererak (which bounced itself and made a Goblin that Phelia exiled). The opponent put in nothing. Lost the next turn. The player noted that Eureka dodges Lavinia's "counter free spells" line, because it puts cards in without casting them.
- **Not cast:** k1 put it back with a Brainstorm because Show and Tell was the better 3-mana play, and later declined a free cast under Omniscience to avoid giving the opponent permanents while already winning. k9 (UW) and k12 (UW) never had two green sources after Wasteland took the Savannah, the Tropical Island and the Tomb (k9) or the Savannah and the Hedge Maze (k12). m7 (UB) kept one land. m10 (UW) and m11 (UW) never had a green land after Wasteland and Stifle, and m10 discarded it to hand size. m2 mulliganed it away in the opening seven.
- **What the opponent got out of it:** in the three casts the opponent put in permanents only in k3 (two Channelers, a Wasteland, two fetches). In m5 and m12 they had nothing to put in.
- **Opponents who saw it:** the UB and UW players on the other seat saw Eureka in the Alurentell hand with Thoughtseize (m5, m7, m10) and said it was stranded or dead. The m5 UB player said a 1-of is risky because it needs a fatty and a fast clock punishes it. Nobody on the opponent side called it dangerous, unlike the first wave's h6.

## What the players said
Nearly every Alurentell player who gave a view would keep 0 or at most 1 Eureka (I did not tally all 24 line by line). The k3 player is the one clear "keep 2"; the k1 player kept Show and Tell over it. The reasons repeat the first wave's:
- **GG plus two is hard on this mana base.** The deck has about 9 real green sources, and Wasteland, Stifle on a fetch and Thoughtseize strip them. In k9, k12, m10 and m11 the card was uncastable all game because every green source was killed.
- **It costs one more than Show and Tell** and is the same kind of target for Force of Will, Force of Negation and Daze. Nobody found a spot where Eureka was a better first choice than Show and Tell, except the k3 line (Omniscience plus Acererak for a single spell).
- **It needs permanents in hand.** m5 and m12 had one or none, and two Eureka in the opening seven (m12) was a dead hand.
- **Carpet of Flowers is what makes it castable** (m5 used three Carpet green; the m11 player had planned Carpet, Chant, Veil and Eureka as a one-turn kill and said that was five or six specific cards).
- Asked what they would cut it for, the post-board players mostly named the Prismatic Ending (m5, m6, m7, m9, m12 and others), the k9 and k12 players named the second Veil of Summer, and a few named a third Show and Tell.

## Veil out of the main deck (game 1)
8-4 with the two main Veil replaced, against the first test's 10-2 with the two Ponder replaced. In the game-1 reports Veil's absence was not named as the reason for any loss: k9 and k12 lost to Wasteland on the green sources, and k5 and k6 lost with no Eureka in sight. The k9 and k12 players said they would put a second Veil back before Eureka, as the better protection for Show and Tell against Force of Will. I cannot separate Veil's absence from the rest at 12 games. The two sideboard Veil were available post-board.

## Ending out of the sideboard plan (post-board)
Four-and-eight against the first test's seven-and-five is the weaker result, and it is worst against UW (0-4). Against UW the Alurentell players in m9 and m12 said, unprompted, that the missing Prismatic Ending was the hole: the UW decks board Lavinia, Azorius Renegade (m9, m10, m12), Erode and Pyroblast, and Lavinia stops Show and Tell, Aluren and any free cast while the Alurentell side has few lands, and Ending is the one-mana answer to her and to Phelia and Tamiyo. m10's player lost to the same Lavinia lock with one land and said he would put the Ending back in place of the Eureka. With 4 games per opponent that is a pattern, not a measurement. (In the first Eureka test all three resolved post-board casts were wins; here both post-board casts were losses.)

## Read
No evidence for Eureka in either slot. Game 1 (8-4) is not different from the earlier result. Post-board, giving up the two Prismatic Ending for two Eureka looks worse than the usual plan against UW, where the players asked for the Ending back. The one good game (k3) shows the real ceiling: with the opponent tapped out and Omniscience in hand, 4 mana puts Omniscience and Acererak in at once. That is a win Show and Tell also gives on 3 mana for one card, and it costs GG that the deck often cannot make. I did not test one Eureka; the players' answer, over both Eureka waves, is that one at most.

## Notes
- **Engine: nothing new that looks like a bug.** Same menu notes as the earlier waves: a `;` in a pick reason runs as a shell command (harmless), `pay=` by name fails on "Boseiju, Who Endures", Carpet of Flowers asks twice per main phase, and Brainstorm/Ponder order prompts do not say the first pick goes on top. m9's UW opponent flagged Pyroblast not countering a Stock Up while the opponent's Veil of Summer was on the stack as a possible bug; Veil makes spells uncounterable that turn, so that is correct.
- **A wording slip in the game brief repeated here.** The Eureka text in these task files again said "every card enters at once", which was already wrong in the first wave (cards enter one at a time, as chosen, and triggers wait until Eureka has finished; see `eureka-engine/README.md`). I did not fix it before launching; it did not change any play that the reports show, and m12's player noticed the prompt order.
- **Carpet of Flowers counts the opponent's Island-typed lands, including the Alurentell side's own fetch targets** (m12's UW player held Tundra and Meticulous Archive back for that reason and fetched Snow-Covered Plains).

Files: `eureka-veil-g1-games/` (24 reports `kNg1a.md` Alurentell and `kNg1b.md` opponent, the 75-card list, `mkeu2.sh`), `eureka-ending-post-games/` (24 reports `mNg2a.md`/`mNg2b.md`, the three plans, the 75-card list, `mkeu2.sh`). The briefs are `eureka-g1-games/BRIEF-g1-eur.md` and `eureka-g1-games/BRIEF-post-eur.md` from the first wave.
