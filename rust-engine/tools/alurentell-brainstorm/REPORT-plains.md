# Plains instead of Island, post-board games (2026-10-14)

Brady's thread: white mana is the limit on Orim's Chant and Prismatic Ending (two white lands, Savannah and Tundra, both nonbasic and Wastelandable). Test: one Plains in place of the one Island in the base 75, then the standard sideboard plan (2 Chant, 2 Ending, 4 Carpet, 2 more Veil) against UR Cutter, Dimir Tempo (UB) and UW Phelia. Three of the seven fetches are Flooded Strands, so only those find a Plains; Wasteland cannot hit it. 12 post-board LLM-vs-LLM games (4 per opponent, 2 on the play, 2 on the draw), Sonnet agents on both seats, no control arm on this build.

## Result
| Opponent | Alurentell with Plains |
|---|---|
| UR Cutter | 2-2 |
| UB Dimir | 3-1 |
| UW Phelia | 2-2 |
| **Total** | **7-5** |

For scale, earlier post-board LLM waves with the plain plan won about 63% vs UR main (31 of 49) and 59% vs UR alt (29 of 49); there is no matching UB or UW baseline. Twelve games cannot separate these (about ±37 points).

## What the Plains did
Each Alurentell player reported on the Plains in a `WHITE:` line. In 12 games:
- **Drawn or fetched and played in 6** (p5, p8, p9, p10, p11, p12). It mattered in 2: p5 (Plains paid the first Prismatic Ending and Savannah the second, so both Endings and a held Veil fit in one turn; won) and p11 (a Wasteland-proof W for Ending when Savannah and Tundra were Wastelander targets; won). In p8, p9, p10 and p12 it was a land drop that did nothing special; p10 noted it as the only land that survived Wasteland and Stifle for three turns.
- **Never seen in 6** (p1, p2, p3, p4, p6, p7), so those games say nothing.
- **Never a liability:** no player reported missing blue or green because the Island became a Plains. The Island was the weakest land in the sim tests too (round 2: cutting the Island was free).
- **Carpet of Flowers carried the white in most wins**, not the Plains. p6 and p7 cast Chant (and in p7 Ending) with W made by Carpet (needs opposing Islands; p7 had no white land at all after Tundra was Wastelanded and still won on Carpet mana); p3 used Savannah for Chant and Carpet for the green. Carpet only adds mana once a turn (its text says "if you haven't added mana with this ability this turn"), so plans counting on a second batch in main 2 are wrong (p11).
- **What cost games was not colour:** Wasteland and Stifle on fetches (p10: Stifle on two fetch activations, left on one land), Lavinia (p10, p12: counters every free spell, shuts off Aluren and Force), Pyroblast and Daze on the combo turn, mulligans and one-landers, and fast Dimir starts (Nethergoyf, Bilbo, Bowmasters in p8).
- **Wanted more white:** p7 ("one more clean W source, Savannah or Plains level"), p9 (one more, no evidence). **Wanted no more:** p3, p4, p11, p12. p10 asked for more Wasteland-proof basics rather than more white specifically.

## Chant and Ending in these games (side data)
- Chant cast first on the go-off turn won p1 (soaked up both Dazes), p3, p6 and p7; kicked Chant stopped an attack in p12 but the game was still lost. Cast unkicked from one W each time.
- Ending exiled a key creature in p5 (Moonshadow, Nethergoyf), p11 (Phelia, answered by Force of Negation after a Veil fight) and p12 (Phelia); held for Lavinia in p10 but never castable (needs two colours and X).
- Veil was the weakest sideboard card in several UW games (p12: their answers are white spells and abilities) but won the Force of Negation fight in p5 and p7.

## Read
The Plains is free: no sign it hurts, it is a Wasteland-proof white source when it shows up, and the result (7-5) is in line with earlier post-board play. Nothing here proves it helps; it was drawn in half the games and decided at most two. A second Plains in place of another land would be the next step if Brady wants to push white access further, but the sims say land cuts beyond one are not free, and Carpet already supplies white against decks with Islands. A better test of "more white" would use the games where white was missing (the 48 games in the report-mining tally had 19 wins).

## Caveats
- No control arm; the baseline numbers come from earlier waves on other builds.
- Opponents had no playbook. Dimir played as a pure aggro deck.
- Engine items seen: Show and Tell offers lands as put-in choices (a1b, a2b, a12b, p9b), which the real card does not allow; Bauble's look result is not shown to the player; a bare land name in `pay=` can match the wrong land.

Files: `plains-games/` (24 reports, `pNg2a.md` Alurentell and `pNg2b.md` opponent, the brief); launcher `archon-games/mkx.sh` (also runs this arm).
