# Game 1 with one Veil of Summer swapped for Emrakul, against the blue decks

Brady's suspicion: Emrakul is good, but it is no better than any non-Veil card, and probably no better than Veil against blue decks.
Test: unboarded game 1, Alurentell main with 1 Veil of Summer and 1 Emrakul, the Aeons Torn in place of the second Veil (arm `v`), against the same main with 2 Veil and no Emrakul (control `c`).
12 games per arm (UR Cutter, UB Dimir, UW Phelia; 4 games each, 2 on the play and 2 on the draw), 24 LLM games in all. Arm and control games with the same number share a shuffle seed and a seat. Net-vs-net simulations of the same swap ran alongside.

## Result in one line
Nothing here shows Emrakul is worse than the second Veil against blue decks. The numbers lean slightly toward Emrakul but are inside the noise, and the LLM games saw Emrakul in only 2 of 12.

## LLM games (Alurentell win/loss)

| Matchup | Emrakul instead of 2nd Veil | Plain main (2 Veil) |
|---|---|---|
| UR Cutter | 4/4 | 3/4 |
| UB Dimir | 2/4 | 2/4 |
| UW Phelia | 4/4 | 2/4 |
| **Total** | **10/12** | **7/12** |

The 95% interval on a 12-game difference is about ±37 points, so +25 is not a finding.

Emrakul was drawn in 2 of 12 arm games (v1 UR, v3 UB, both in the opening seven) and both were won, though the second game was also won by the Veil it kept.
- v1: Show and Tell put Omniscience in, Emrakul was then cast free from hand, the extra turn plus annihilator 6 wiped the board, and the game ended on turn 4. Daze, Force and burn could not touch it (uncounterable, protection from colored spells).
- v3: Veil in response to Force of Will made Show and Tell uncounterable, Emrakul came in and won in two attacks. Dimir's removal is all colored and could not target it.

In the other 10 arm games the card did nothing, as expected: it needs Omniscience or Show and Tell to be live, and in those games the Aluren/Acererak loop or the other cards won or lost on their own.

Veil (control arm, two copies): drawn in c1, c3, c5, c6, c7, c12 (not drawn in the other six).
- Won the game when cast: c1 and c7 (cast before Show and Tell, so the Force of Will / Daze fights did not matter).
- Dead draw in c3, c5, c6 and c12 (no usable mana at all), mostly where the combo turn used all the mana and there was no spare green for Veil (Veil plus Aluren needs five mana and a green source). The c6 loss was a Show and Tell into Force of Negation that Veil would have beaten, had there been mana for it.
- The single Veil in the arm was drawn in v3 (won it) and v12 (not needed).

Losses in the arm (v4, v10) and control (c2, c4, c6, c9, c12) were mostly the opponent's early clock (Moonshadow, Murktide plus burn, Phelia) or a stripped hand, not the Veil/Emrakul slot. v4 and c4 (both UB, on the draw) were lost to the same Moonshadow start.

What the players said (the arm players were asked "would you rather have a second Veil than Emrakul against this opponent?"): 9 of 12 said they would rather have the second Veil, but all 9 had not drawn Emrakul, so this is a guess from people who never held the card. The two who did draw it (v1, v3) both said keep it, cutting a Veil or the fourth Acererak/Stock Up/Petal for it. v12 called Veil the safer card without drawing either.

## Net simulations (same swap, unboarded, both seats, 6 batches of 400)

| Matchup | base | Emrakul for 1 Veil | change (95%) |
|---|---|---|---|
| UR Cutter | 55.8% (2400) | 58.5% (2400) | +2.7 (±2.8) |
| UB Dimir | 46.1% (2400) | 49.4% (2400) | +3.3 (±2.8) |
| UW Phelia | 39.4% (2400) | 45.4% (2400) | +6.0 (±2.8) |

Pooled across the three, about +4 (±1.6) points. This agrees in direction with the first Emrakul swap table (round 2, 400 per side): Emrakul-for-Veil was one of the better swaps there too. A caveat: the nets play Veil only roughly (when to hold mana for it), so its value as protection is the part they are least able to price.

## What to take from it
- No evidence for "Emrakul is worse than Veil against blue decks": LLM games 10/12 vs 7/12 and sims +3 to +6 all point the other way, none of it conclusive on its own.
- The sims and games agree that the second Veil is not doing much: in the control games Veil was a dead draw as often as a game-winner, mostly because casting it next to the combo needs mana the combo turn does not have.
- Emrakul is a narrow card (live only with Omniscience or Show and Tell) that rarely matters, but when it was drawn it ended the game against decks whose removal is all colored.
- Cuts: the first Emrakul report's candidates (Ponder, Stock Up) remain the likelier cuts if Brady wants Emrakul without giving up a Veil; this test only says 2nd Veil is not clearly better to keep.

Files: `llm/emrakul-games/reports/v*g1[ab].md, c*g1[ab].md` (the `a` file is Alurentell's report, with `EMRAKUL:` and `VEIL:` lines), `tasks/`, `decks-base-emrv/alurentell.txt` (the arm's 75), `mkv.sh` (launch helper), `mk2.sh`/`mkgame2.sh` (now with ub/uw matchups and `BASE_A`/`NOTE` overrides).
