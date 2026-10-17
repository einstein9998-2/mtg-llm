# Two Orim's Chant or two Veil of Summer in the main deck (2026-10-17)

Brady's league result (4-1) came from swapping the two main-deck Veil of Summer for the two Orim's Chant, with the four Veil in the sideboard and one Faerie Macabre turned into a basic Forest. He drew Chant once late against UB and it made his Show and Tell resolve, and thought Veil would have done the same. Test: game 1 (nobody sideboarded), 12 LLM-vs-LLM games per list against UR Cutter, Dimir/UB and UW Phelia (4 each, 2 on the play and 2 on the draw), Sonnet agents on both seats, same seed and same play/draw for game n in both lists.

- **Veil list** (`chantveil-games/arms/cur/alurentell.txt`): the base list, `decks/alurentell.txt`. Main has 2 Veil of Summer; sideboard has 2 Chant, 2 Veil.
- **Chant list** (`chantveil-games/arms/brady/alurentell.txt`): main 2 Orim's Chant instead of the 2 Veil; sideboard has 4 Veil, no Chant, 1 Faerie Macabre, 1 Forest. The other 58 main cards are identical.

## Result
| Opponent | Veil main | Chant main |
|---|---|---|
| UR Cutter | 1-3 | 3-1 |
| UB Dimir | 3-1 | 3-1 |
| UW Phelia | 4-0 | 1-3 |
| **Total** | **8-4** | **7-5** |

One game apart over 24 games, and the opponent splits go in opposite directions (Chant ahead against UR by two games, Veil ahead against UW by three). Twelve games per list cannot separate the two (about ±28 points each). Read it as "no difference found".

## What the cards did
**Chant list: drawn in 5 of 12, cast in 2.**
- Cast and won: g1 (UR; unkicked off Lotus Petal right after Aluren resolved, shut off Force of Will and Daze for the whole loop turn) and g8 (UB; free under Omniscience in response to the Bowmasters put-in trigger, then the loop ran unopposed).
- Drawn and dead, both losses: g6 (UB; Tropical Island was the only coloured land and was Wastelanded on their turn 2) and g12 (UW; no white source all game, four Wastelands).
- Drawn only inside the finished loop: g7. Also seen in an Atraxa reveal (g10) and passed over for Force of Will because no white was available.
- Record when drawn 3-2, not drawn 4-3.

**Veil list: drawn in 6 of 12, cast in 2.**
- c1 (UR, loss): the one real use. Their Force of Will on Murktide Regent was answered by Force of Will, they Forced back, and Veil in response made the first Force uncounterable and drew a card. Murktide was countered; the game was still lost to Unholy Heat on Acererak and two Dragon's Rage Channelers.
- c10 (UW, win): cast free under Omniscience before Aluren; the opponent never held a counter, so it was untested.
- Drawn and never cast: c2 (UR, loss, no lands), c6 (UB, win, Aluren needed both green sources so Veil could not be paid the same turn), c8 (UB, loss, dead against a creature clock), c11 (UW, win, drawn inside the loop).
- Record when drawn 3-3, not drawn 5-1 (c4, a UR loss).

## What the players said
Each Alurentell player was asked whether they would rather have had the other card.

| | Prefer the card they did not have | Keep what they had | No view |
|---|---|---|---|
| Veil list (12) | Chant: c1, c5, c9, c12 | c3, c4, c6, c10, c11 | c2, c7, c8 |
| Chant list (12) | Veil: b4, b5, b6, b10, b11, b12 | b1, b2, b8 | b3, b7, b9 |

- **Chant's argument** (c9, c12, b1, b8): one W on the combo turn stops everything, including the cards Veil cannot touch: Swords to Plowshares and Stifle on Acererak's trigger (UW), Unholy Heat and Lightning Bolt (UR), Fatal Push (UB). It also covers Daze when you are tapped out. Against UW it was the card the Veil players wanted.
- **Veil's argument** (b4, b5, b6, b10, b11, b12): it needs G, which the deck can make from Tropical Island, Hedge Maze, Carpet and Petal, while Chant needs W from Savannah or Tundra only, both nonbasic and Wasteland targets. Both dead Chants (b6, b12) were Wasteland games, and b4 and b11 wanted Veil to beat Daze or Force of Will on Aluren or Show and Tell. Veil also draws a card when they counter, per the players.
- **Both are weak against a creature clock** (c8, c4, b2, b6): nothing to protect when Show and Tell never shows up.
- **Kicked Chant** (WW more) is not mentioned as cast in any report; nobody had the white for it.

So the card that wins the game depends on what the opponent holds, and the colour-access cost is the real difference: Chant is better when white survives, Veil is cheaper to cast. This matches the earlier Petal/Chant waves ("Chant wins when white survives, dead card when white is gone").

## Mistakes worth knowing before reading the numbers
- **Two Chant-list losses came from the same menu misread.** In g9 (UW) and g11 (UW) the player had Force of Will plus a pitch card with a counterspell on the stack against their Show and Tell, picked index 0 (Pass priority) instead of the Force of Will option, and lost the spell. Both were losses on a cast the player meant to protect; neither is about Chant or Veil, but they are two of the five Chant-list losses.
- g2 (Chant list, UR): the player played a fetchland with City of Traitors untapped and lost its mana, then lost to Murktide and Bolt.
- Force of Will with no blue card left to pitch came up repeatedly in both lists (b2, b9, c1, c4 in the UR/UW losses, and the UW and UB opponents' own reports).
- Wasteland on Alurentell's newest nonbasic land is in the loss reports on both sides (b6, b12, c1, c2), so mana access, not the Chant/Veil slot, decided those games.

## Not done: post-board games
The request was to include post-board games because the sideboards differ. They do not change anything here: after sideboarding against these three decks the plan puts in all 4 Veil and 2 Chant (`sideboard-plans/alurentell.txt`: +2 Veil, +2 Chant), and the Chant list already has 2 Chant main and 4 Veil in the sideboard, so both lists reach the same 60 cards. The Faerie Macabre and Forest only matter in the sideboard against decks that are not these. Post-board games would just be a second run of one list. If the question is a different post-board layout for the Chant list (for example only 2 of the 4 Veil), say which cards give way and I'll run it.

## Caveats
- No test beyond 12 games per list; Wasteland and mulligans decided a good share of the games on both sides.
- Play and draw and the seed are shared per game number, but the opening hands are not: the lists differ, so the shuffles diverge.
- The Alurentell player saw its own deck file and the report asked for a `PROTECT:` line (drawn, cast, what it did, would you rather have the other card).
- Opponents have no playbook. One UB report (c7) called Show and Tell offering a land to the opponent an engine bug; it is correct per Brady's earlier ruling.
- Engine and menu items seen again: a `;` or `'` in a pick reason runs as shell text (the pick still goes through); delve and fetch-land menus do not show remaining counts.

## Files
`chantveil-games/`: 48 reports (`cvcurNg1a` = Veil list Alurentell player, `cvcurNg1b` = opponent, `cvbradyN...` for the Chant list; N = 1-4 UR, 5-8 UB, 9-12 UW), the two list files in `arms/`, `BRIEF-chantveil.md` (player brief) and `mkcv.sh <cur|brady> <n> <A|B>` (launcher; it expects the `llm2` working directory from earlier waves).
