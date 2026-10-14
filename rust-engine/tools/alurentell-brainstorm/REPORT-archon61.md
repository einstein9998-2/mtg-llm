# Archon of Cruelty as a 61st card, game 1 (2026-10-14)

Brady's follow-up to the Ponder-swap test: keep all four Ponder and add one Archon of Cruelty as a 61st card. Same setup as `REPORT-archon.md`: unboarded game 1, 12 LLM-vs-LLM games (4 each against UR Cutter, Dimir Tempo and UW Phelia, 2 on the play and 2 on the draw), Sonnet agents on both seats, same playbook, rulings and menus. The engine took the 61-card deck without trouble (library 54 after a 7-card hand). The Alurentell brief says Show and Tell lets both players put a land in, so land choices are not a bug. No control arm was run on this build.

## Result
| Opponent | Alurentell with Archon as the 61st card |
|---|---|
| UR Cutter | 3-1 |
| UB Dimir | 2-2 |
| UW Phelia | 1-3 |
| **Total** | **6-6** |

Twelve games cannot give a win rate (a 95% interval is about ±37 points), so this is neither for nor against the card. The Ponder-swap version was 5-7, the earlier plain-main control (different build) 7/12.

## What Archon did
Nothing. It was never cast and never put onto the battlefield in 12 games.
- **In hand and dead (2 games):** b2 (UR, lost; Omniscience and Archon in hand, no Show and Tell, no black mana) and b10 (UW, lost; drawn twice, never found Show and Tell).
- **Seen but not taken (3 games):** b1 and b6 saw it in an Atraxa reveal and took Acererak instead (a creature slot has one pick, and Aluren + Acererak or Omniscience + Acererak won the same turn); b9 saw it with Ponder and shuffled it away.
- **Never seen:** the other seven games (b3, b4, b5, b7, b8, b11, b12), four of them wins.
- The wins that happened (b1, b3, b4, b6, b8, b11) came from Atraxa, Aluren + Acererak or Omniscience + Acererak. The losses came from the same causes as in the other tests: Show and Tell or Aluren countered (b9, b10, b12 against Force of Will, Force of Negation, Stifle and a Tamiyo ultimate), Daze or Wasteland on a thin mana base (b5, b7), and a keep with no Show and Tell (b2).

## What the players said
All 12 Alurentell players said to leave it out as a 61st card (b5 and b2 would cut an Atraxa or the second Omniscience only if the slot had to be used). Their reasons, most of them from players who never held it, so treat them as opinion:
- It only works through Show and Tell or Omniscience, so it competes with Atraxa for the same put-in and does nothing in an opening hand.
- Atraxa's reveal fetches the rest of the combo even when she is answered, blocks and gains 7; Archon's enter trigger (edict, discard, lose 3, draw, gain 3) is slower and tutors nothing.
- Swords, bounce, Stifle on the trigger and Force of Will on the Show and Tell all answer it (Fatal Push cannot reach MV 8 and Snuff Out cannot target a black creature, but that leaves the white and blue answers), and the opponent picks which creature to sacrifice (a Monk token).
- Where it looked good: a lone big blocker or a hand of Force and Daze, which is the b1 situation of the earlier test.

## Show and Tell lands (Brady's correction)
Brady pointed out that Show and Tell lets the opponent put a land in too, and that it matters because a tapped-out opponent gets an untapped land. The reports back this up:
- b6 and b11: the opponent put in Wasteland (colourless), so Stifle on the loop was impossible; a blue land would have let them Stifle the Acererak trigger. b11's player called this the real danger to the loop.
- b6: an untapped Island came with the Show and Tell and the opponent also Wastelanded the Alurentell player's only nonbasic.
So the choice of land is a real decision for the opponent, and an Alurentell player casting Show and Tell into a tapped-out opponent with Stifle or a Consign-style answer in the deck gives them a mana source to use it. My earlier "bug" note on this was wrong.

## Read
No evidence that a 61st Archon helps, and 12 of 12 players would not play it. It never mattered in a game, so the 6-6 mostly reflects the base deck. Both Archon tests (swap for Ponder 5-7, 61st card 6-6) say it is not an upgrade over the 60-card list.

## Caveats
- Opponents had no playbook (Dimir plays as an aggro deck, as in the earlier wave).
- Some Alurentell players noted the `pay=` flag does not handle Daze or Boseiju's comma in its name (b3 used a v-id); no other engine bugs.
- b7: the Alurentell player kept a no-cantrip, no-Show-and-Tell seven; that is a play mistake, not Archon's effect.

Files: `archon61-games/` (24 reports `bNg1a.md` Alurentell and `bNg1b.md` opponent, the 61-card list `alurentell-61.txt`, the brief and launcher).
