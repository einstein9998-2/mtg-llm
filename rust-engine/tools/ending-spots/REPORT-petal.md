# Keeping exactly one Lotus Petal post-board against tempo (2026-10-15)

Brady's question: against tempo (UR, UB, UW), keep exactly 1 Lotus Petal post-board. One minimizes how often it is too low-impact, but you can still see it after Atraxa, where it is free to take as the only artifact, and when the extra mana could matter more.

Plan tested (`llm-petal/plans/*/alurentell.txt`): the usual post-board swap but with one Petal kept. Out: 3 Lotus Petal, 2 Omniscience, 2 Force of Will (UR also 1 Boseiju, 1 Aluren; UB 4 Force of Will instead; UW also 2 Aluren). In: 4 Carpet of Flowers, 2 Orim's Chant, 1 Prismatic Ending, 2 Veil of Summer.

## Short answer

- **Nothing in 12 LLM games argues against exactly one, and nothing argues for zero.** Every Alurentell player who gave a verdict kept at least one: 7 said exactly one, 3 said one and leaned toward two (against Wasteland and Stifle), 2 said "none or at most one". Nobody wanted more than two. Nobody wanted none without an "or one" next to it.
- **The result is 5 wins and 7 losses (UR 2-2, UB 2-2, UW 1-3).** That is far too few games to say whether one Petal is better than zero, and there is no zero-Petal control with this exact plan. The earlier Chant-plan games (4-2 and 2-4 against the same decks) are not a clean baseline either: different cards were boarded.
- **The Petal showed up in 4 of 12 games, close to what the math says (16% by turn 4, 22% with cantrips).** It was used in all four and was never a cost: it made a one-land hand a keep (turn-1 Stock Up off Ancient Tomb and Petal), was the third mana for Show and Tell once Wasteland hit a land, and was a black mana for a third Acererak. It decided no game by itself. Alurentell went 2-2 in those four.
- **The "too low-impact" worry shows up exactly where Carpet of Flowers is drawn.** With two to four Carpets making 4 to 9 mana a turn, three players called the Petal redundant (p8, p9, p12). The games where the extra mana mattered were the land-light ones with Wasteland or Stifle on the lands (p7, p10, p11).
- **Atraxa reveal:** no Petal was taken from a reveal in these games. The math: a lone Petal is in the top 10 about 17-21% of the time that Atraxa resolves, so the "free pick" fires in roughly one Atraxa in five. It is a real but small bonus.

## The math

All exact hypergeometric on the 60-card deck, on the play (`white_access.py`, REPORT-white.md for the full tables).

P(the one Petal has been drawn by turn N):

| | T2 | T3 | T4 | T5 | T6 |
|---|---|---|---|---|---|
| no cantrips | 13% | 15% | 17% | 18% | 20% |
| with 3 cantrip cards | 18% | 20% | 22% | 23% | 25% |

P(that Petal is among the top 10 cards an Atraxa reveals at turns 5 to 7) is about 17-22%, given it has not been drawn yet.

White access for Orim's Chant from the one Petal (3 Chant boarded, cantrips counted, by turn 3):

| | W with 0 Petals | W with 1 Petal | WW with 0 | WW with 1 |
|---|---|---|---|---|
| no Wasteland | 89% | 91% | 58% | 65% |
| after one Wasteland on a land | 58% | 67% | 0% | 11% |

So one Petal is worth about +2 points of white by turn 3 without Wasteland and +9 with it, and it is the only way to reach WW at all once Wasteland takes a white land. Four Petals would be worth +27 points with Wasteland, but that is four more cards out of the sideboard plan.

Earlier net-only baseline games (`endingspot ... --baseline 20000`, the `end1p` plan against the base list) put one Petal at +0.7 points (UR), +0.4 (UB) and +1.3 (UW) of win rate, with a noise of about ±0.5. Nets undervalue the sideboard cards and follow Petal count, so this is a sanity check only.

## The 12 LLM games

Two Sonnet agents per game (one per seat), post-board game 2, opponent plans from `oppplans`. Tags `p1`-`p12` = three each for UR, UB and UW, alternating play and draw. Alurentell result:

| Game | Opponent | Alurentell on | Result | Petal | Verdict on one Petal |
|---|---|---|---|---|---|
| p1 | UR | play | WON | not drawn | keep one, not more |
| p2 | UR | draw | LOST | not drawn | keep one: instant-speed white to kick Chant was what it lacked |
| p3 | UR | play | WON | drawn in opener, turn-1 Stock Up off Tomb | keep one, it made a one-land hand a keep |
| p4 | UR | draw | LOST | opened in a mulligan, never drawn | keep one, not more |
| p5 | UB | play | WON | not drawn | keep one: mana was exactly tight on the Chant + Show and Tell turn |
| p6 | UB | draw | LOST | not drawn | keep one |
| p7 | UB | play | LOST | drawn turn 3, used as third Show and Tell mana and a Carpet pip | exactly one is fine, two plausible if land-light |
| p8 | UB | draw | WON | drawn off Stock Up, black mana for a third Acererak | none or at most one, redundant with four Carpets |
| p9 | UW | play | LOST | not drawn | one is fine, not more |
| p10 | UW | draw | LOST | not drawn | one, leaning two against Wasteland and Stifle |
| p11 | UW | play | LOST | drawn off Stock Up, only extra mana after Wasteland on City | one, probably two |
| p12 | UW | draw | WON | seen in a Ponder and shuffled away | none or at most one, Carpets made it redundant |

Total: 5 wins, 7 losses. With the Petal drawn: 2-2. Without: 3-5.

Other things the players kept running into:

- **Wasteland and Stifle still dominate the mana problem.** Lands were Wastelanded or fetches Stifled in p1, p7, p10 and p11, and the Petal (where drawn) was the spare mana left. Players in those games wanted a second Petal; players in the Carpet games did not.
- **Nobody used the Petal to pay for Daze**, though several named it as a likely use. It is a one-shot that costs a card slot every game it is not drawn.
- **Chant was mostly bait or protection for Show and Tell and Aluren.**
- **Engine notes from the players** (not changed here): `pay=` is sometimes ignored by the auto-payer, which tapped Ancient Tomb for life; Flooded Strand listed Meticulous Archive; Daze shows two menu entries.

## What this does not tell us

- No control group: no 12-game run of this plan with zero Petals, so the 5-7 says nothing about Petal count.
- Players only saw the Petal in 4 games, so the verdicts of the other 8 are opinion from games where it was absent.
- The Atraxa-reveal case did not occur once in 12 games, so its value rests on the math alone.
- Games are single Sonnet agents against single Sonnet agents.

## Files

- `llm-petal/p{1..12}g2{a,b}.md`: all 24 game reports (a = Alurentell player, b = opponent).
- `llm-petal/plans/{ur,ub,uw}/alurentell.txt`: the post-board plan used.
- `llm-petal/mkp.sh`, `llm-petal/BRIEF-petal.md`: the launcher and the player brief with the extra `PETAL:` and `CHANT_ENDING:` report lines.
- `white_access.py`, `REPORT-white.md`: the white-access math.
