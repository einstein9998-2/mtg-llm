# Post-board list: 2 Petal, 3 Chant, 0 Prismatic Ending, 3 Aluren (2026-10-14/15)

Brady's request: try "2 petal 3 chant 0 prismatic ending 3 aluren" against the three tempo decks (UR Cutter, UB Dimir, UW Phelia). Post-board game 2, 12 LLM-vs-LLM games (4 per opponent, 2 on the play and 2 on the draw), Sonnet agents on both seats, same playbook, rulings and menus as the Plains wave. The opponents had boarded with their usual plan. No same-build control arm was run.

## The list
I read the numbers as absolute counts in the 60. Versus the usual sideboard plan for each matchup that means: keep 2 Lotus Petal (the usual plan boards all 4 out), 3 Orim's Chant (usual: 2), 0 Prismatic Ending (usual: 2 in), 3 Aluren. The usual plans (`plan-pe-*.txt` here are the ones that were played) otherwise stay: 4 Carpet of Flowers and 2 Veil of Summer in.

To stay at 60 the numbers needed extra cuts on two matchups, which I took from Ponder (my call, not Brady's): UB balanced exactly, UR needed 1 extra cut (Ponder 3), UW needed 2 (Ponder 2, Aluren 3). Final 60s: 2 Petal, 3 Chant, 0 Ending, 3 Aluren, 4 Veil, 4 Carpet, and Ponder 3 (UR) / 4 (UB) / 2 (UW).

## Result
| Opponent | Alurentell |
|---|---|
| UR Cutter | 3-1 |
| UB Dimir | 2-2 |
| UW Phelia | 3-1 |
| **Total** | **8-4** |

Twelve games cannot give a win rate (95% interval about ±28 to 37 points). For scale, earlier waves with the usual plan on other builds were about 63% against UR; there is no UB or UW baseline on this build. So 8-4 is not different from the usual plan, and the data cannot say whether the list is better or worse.

## What the changed cards did
- **Orim's Chant (3):** drawn in 7 of 12 games, cast in 5.
  - Decided the game in two: e11 (UW) and e12 (UW). Both times Chant went first into a tapped-out or one-counter hand, then Aluren or Show and Tell resolved and Acererak looped; in e12 Chant plus Veil beat a Force of Will and Stifle hand ("Chant alone loses to the Force, Veil alone loses to Swords or Stifle on the Acererak trigger").
  - Worked as bait in e1 and e2 (UR): Dazed in e1, Forced in e2 (pitching Daze), then Show and Tell met Pyroblast anyway.
  - Resolved but did not save the game in e10 (UW, at 3 life behind Lavinia).
  - Dead in hand in e5 (put back) and e7 (stuck on two lands, no white source). The white-source problem is the same as in the earlier Chant work: Chant needs Savannah, Tundra or Carpet white.
  - Players who drew it: keep 3 (e12 wanted a third over the second Petal), but e7 and e5 would play fewer.
- **Lotus Petal (2):** drawn in 5 games. Useful as mana in e2 and e3 (Stock Up before Wasteland, Daze insurance on the kill turn), dead in e7, Consigned for nothing in e11 (and cost the fourth mana that turn), and in e12 it only let Carpet resolve into Prismatic Ending. e11's player wanted 0.
- **Aluren (3):** all eight wins were the Aluren plus Acererak loop. Players who drew them wanted three or four; the two games where none showed up were losses (e7, e10), and e11 only won because a second Aluren was in hand after the first was Forced. Nobody said three was too many.
- **No Prismatic Ending:** e10 (UW) was the only game where its absence cost something: Lavinia hit for 2 a turn from turn 3 to 10 and Ending is the clean answer. e11's player also said he would not cut it entirely against Phelia and Tamiyo, but that game was won. In e12 and e8 it was not missed.

## Wins and losses
- Wins (8): Aluren plus Acererak in e1, e3, e5, e8, e9, e11, e12; e2 got there through Atraxa's reveal after Show and Tell ate a Pyroblast.
- Losses (4): e4 (UR) Show and Tell Forced, then Murktide as an 8/8 on their turn 4 (life was the margin, 7 paid to Ancient Tomb and fetches); e6 (UB) Force of Negation on Show and Tell, then Consign on Acererak's enter trigger, Bilbo, Nethergoyf and Kaito; e7 (UB) a one-land keep with three Brainstorms, no second land, dead on turn 5 to exactly 10 damage; e10 (UW) Wasteland on the only blue source, no land drop for four turns, Lavinia.

## Notes from the games
- **Consign to Memory on Acererak's own enter trigger** (e6, and again in the Eureka games f9 and f12): Acererak stays on the battlefield without bouncing, the loop is dead, and the legend rule stops a recast. The playbook only covers Consign as a spell counter. Stifle does the same.
- **Carpet of Flowers** was good when they controlled Islands (it paid for GG, Stock Up, Aluren), but Daze or Wasteland-style plays that return or remove their only Island turn it off (h7 shows this in the Eureka wave).
- Wasteland on the only nonbasic or Daze on the one-mana play that has no spare mana (Carpet, Stock Up) cost several games; players repeat the old rule: do not cast the turn-2 spell into open Daze mana without a spare.
- **Engine and menus:** no engine bugs. Carpet asks "Do it?" and a colour every main phase (4 prompts a turn with two Carpets). Brainstorm and Ponder order prompts do not say which pick goes on top; it is the first pick. The odds tool does not know the opponent's sideboard (it reports 0 unseen Pyroblast).

## Read
8-4 against three tempo decks is in line with the usual plan. Within the list: keep Chant, Aluren is not the card to cut, Petal was the weakest of the changed cards, and Ending's absence only showed once (Lavinia). Nothing here is strong enough to change a list. The next step if Brady wants it is a same-build control arm with the usual plan.

Files: `pe-games/` (24 reports `eNg2a.md` Alurentell and `eNg2b.md` opponent, the three plans, `mkpe.sh`, `BRIEF-post-pe.md`).
