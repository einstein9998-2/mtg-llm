# Game 1: Emrakul for a land, or as a 61st card

Brady's ask: try swapping a land for Emrakul, the Aeons Torn, or having it be the 61st card. Same setup as the Veil swap test: unboarded game 1 against the plain main.

## Net simulations
Alurentell with Emrakul added, unboarded, 400 games per side (play and draw) per cell, 800 per cell unless noted. Change against 2400 games of the plain main (1600 for Tron and Storm). 95% interval is about ±4 for 800 games and ±3 for 1600.

| Variant | UR Cutter | UW Phelia | UB Dimir | Tron | Storm |
|---|---|---|---|---|---|
| Island out | -4.3 | +5.9 | +2.3 | -1.4 | +1.6 |
| Hedge Maze out | -3.5 | +0.8 | +2.3 | -1.0 | +1.4 |
| Flooded Strand out | -1.1 | +5.9 | +0.9 | -2.9 | +2.6 |
| City of Traitors out (1600 on blue) | 0.0 | +2.1 | +1.9 | +0.8 | +0.9 |
| Boseiju out | -1.7 | +5.0 | +5.0 | -4.6 | +2.2 |
| Savannah out (1600 on blue) | -1.0 | +1.8 | +3.3 | -2.0 | +3.2 |
| 61st card (1600 on blue) | -0.3 | +2.6 | -0.6 | +3.3 | +1.5 |

Averaged over the three blue decks: City +1.3, Savannah +1.3, 61st card +0.6, Island +1.3, Strand +1.9, Boseiju +2.8, Hedge Maze -0.2 (the 800-game cells have about ±2.3 on that average, the 1600-game cells about ±1.8). Nothing is outside the noise and no variant is clearly worse than the plain main.

## LLM games (12 per arm: UR, UB, UW; 4 each, 2 on the play, 2 on the draw; same seeds and seats in every arm)

| Arm | UR Cutter | UB Dimir | UW Phelia | Total |
|---|---|---|---|---|
| Plain main (20 lands, 2 Veil, no Emrakul) | 3/4 | 2/4 | 2/4 | **7/12** |
| Emrakul for 1 Veil (earlier test) | 4/4 | 2/4 | 4/4 | **10/12** |
| Emrakul for City of Traitors (19 lands) | 4/4 | 1/4 | 4/4 | **9/12** |
| Emrakul as a 61st card (20 lands) | 1/4 | 1/4 | 3/4 | **5/12** |

With 12 games a side a difference needs to be about 35-40 points to mean anything, so none of these gaps is a finding.

What happened with Emrakul itself:
- City arm: held in 3 of 12 games and drawn into via Atraxa in 2 more. Won l6 (Show and Tell put-in, annihilator wiped UW's board), won l8 (free cast under Omniscience, extra turn, lethal), lost l10 (Thoughtseize on Show and Tell, then Forced; stayed in hand). The remaining nine games never needed it.
- 61st card arm: drawn in 4 of 12. s6 won (Show and Tell put-in, annihilator), s12 won (free cast under Omniscience, though Stifle hit the extra-turn trigger), s2 lost (dead in the opening seven, UR player's Murktide plus burn won), s11 lost (put in with Show and Tell, then Phelia exiled it and Stifle on the delayed return trigger kept it away for good).
- Phelia plus Stifle is a real answer to a resolved Emrakul (s11, and the UW player in s12 used Stifle on its cast trigger).

Mana in the City arm (Alurentell players' own reading): 7 of 12 said the 19th land was fine. l1, l5 and l12 said they would rather have City back: l5 and l12 lost Hedge Maze or Tomb to Wasteland and were short of coloured sources, l1 stumbled on an early land. l10 (mulligan to six) did not miss it. City of Traitors was also called a trap in several control games last round (a later land drop sacrifices it).

The 61st-card arm lost 7 of 12, to fast clocks (Cutter, Murktide, Moonshadow, Bowmasters in s1, s2, s3, s4, s10), a grindy Wasteland game (s7) and Phelia plus Stifle (s11), rather than to the extra card; the players mostly noticed it only when it was a dead draw (s2). The sims do not show a cost for the 61st card (+0.6 on blue), so the LLM result is most likely noise.

## Read
- Taking City of Traitors out for Emrakul looks as good as any other swap: sims +1.3 on blue decks, LLM games 9/12 against 7/12. The players' complaint about City is that it sacrifices to the next land.
- The 61st card has no clear cost in the sims, but the LLM games were the weakest of the four arms. If Brady wants the extra card without testing more, a land swap is the safer reading.
- Savannah and Boseiju swaps also tested fine against blue decks; Boseiju hurts Tron by about 4.6 (±4.2).
- None of this distinguishes Emrakul from the cards it replaces by more than the noise, consistent with the earlier reports.

Files: `llm/emrakul-games/reports/l*g1[ab].md` (City arm) and `s*g1[ab].md` (61st card arm) (`a` is Alurentell's report), `tasks/`, `decks-base-emrl/`, `decks-base-emrs/`, `mkl.sh`; sim code `gen4.py` in this folder.
