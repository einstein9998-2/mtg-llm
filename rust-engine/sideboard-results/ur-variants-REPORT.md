> Rerun 2026-10-06 after fixing the belief leak (PR #4 review): opponent beliefs come from the public plan (`--public sideboard-variants/base`). The first run, where the opponent saw the variant list, is superseded.

## Win rates (95% Wilson intervals)

| Variant | Matches won by Alurentell | Games 2+ | Games 2+ on the play | Games 2+ on the draw |
|---|---|---|---|---|
| a1 | 52.3% [49.2, 55.4] (523/1000) | 50.2% [47.7, 52.7] (747/1488) | 48.9% [45.3, 52.6] (345/705) | 51.3% [47.8, 54.8] (402/783) |
| a2 | 52.9% [49.8, 56.0] (529/1000) | 51.0% [48.5, 53.6] (761/1491) | 50.5% [46.8, 54.2] (354/701) | 51.5% [48.0, 55.0] (407/790) |
| b | 53.2% [50.1, 56.3] (532/1000) | 49.7% [47.1, 52.2] (740/1490) | 48.5% [44.8, 52.1] (350/722) | 50.8% [47.3, 54.3] (390/768) |
| base | 52.4% [49.3, 55.5] (524/1000) | 49.6% [47.1, 52.1] (749/1511) | 49.0% [45.3, 52.6] (356/727) | 50.1% [46.6, 53.6] (393/784) |
| c1 | 51.4% [48.3, 54.5] (514/1000) | 48.9% [46.4, 51.4] (738/1510) | 48.0% [44.4, 51.6] (349/727) | 49.7% [46.2, 53.2] (389/783) |
| c2 | 53.0% [49.9, 56.1] (530/1000) | 50.7% [48.2, 53.2] (755/1489) | 49.4% [45.7, 53.0] (348/705) | 51.9% [48.4, 55.4] (407/784) |
| d1 | 52.3% [49.2, 55.4] (523/1000) | 49.3% [46.8, 51.9] (741/1502) | 49.3% [45.7, 52.9] (358/726) | 49.4% [45.8, 52.9] (383/776) |
| d2 | 55.7% [52.6, 58.8] (557/1000) | 52.5% [50.0, 55.1] (788/1500) | 53.0% [49.3, 56.7] (377/711) | 52.1% [48.6, 55.6] (411/789) |
| d3 | 54.2% [51.1, 57.3] (542/1000) | 52.1% [49.5, 54.6] (778/1494) | 51.4% [47.7, 55.1] (359/698) | 52.6% [49.2, 56.1] (419/796) |

## Difference from `base` (games 2+, normal approximation; matches in brackets)

| Variant | Games 2+ difference | z | Matches difference |
|---|---|---|---|
| a1 | +0.6 points (SE 1.8) | +0.3 | -0.1 points (SE 2.2) |
| a2 | +1.5 points (SE 1.8) | +0.8 | +0.5 points (SE 2.2) |
| b | +0.1 points (SE 1.8) | +0.1 | +0.8 points (SE 2.2) |
| c1 | -0.7 points (SE 1.8) | -0.4 | -1.0 points (SE 2.2) |
| c2 | +1.1 points (SE 1.8) | +0.6 | +0.6 points (SE 2.2) |
| d1 | -0.2 points (SE 1.8) | -0.1 | -0.1 points (SE 2.2) |
| d2 | +3.0 points (SE 1.8) | +1.6 | +3.3 points (SE 2.2) |
| d3 | +2.5 points (SE 1.8) | +1.4 | +1.8 points (SE 2.2) |

## Card evidence (games 2+; observational, not causal)

Win rate of Alurentell when the card was seen (in the opening hand or drawn) by its own turn 4 versus not; the card has to be in that variant's deck to be seen.

| Variant | Card | Seen by turn 4 | Not seen by turn 4 | Games where cast | Won, cast within the last 2 own turns | Lost with it stuck in hand |
|---|---|---|---|---|---|---|
| a1 | Lotus Petal | 51.7% [46.0, 57.4] (152/294) | 49.8% [47.0, 52.7] (595/1194) | 393 | 27 | 9 |
| a1 | Stock Up | 52.1% [48.5, 55.6] (393/755) | 48.3% [44.7, 51.9] (354/733) | 554 | 123 | 10 |
| a1 | Acererak the Archlich | 51.2% [47.9, 54.4] (459/897) | 48.7% [44.7, 52.8] (288/591) | 636 | 250 | 147 |
| a2 | Lotus Petal | 49.9% [45.7, 54.1] (270/541) | 51.7% [48.5, 54.8] (491/950) | 688 | 71 | 29 |
| a2 | Stock Up | 51.5% [47.3, 55.6] (280/544) | 50.8% [47.6, 54.0] (481/947) | 446 | 104 | 11 |
| a2 | Acererak the Archlich | 54.0% [50.7, 57.2] (485/898) | 46.5% [42.6, 50.6] (276/593) | 716 | 285 | 173 |
| b | Lotus Petal | 50.5% [44.8, 56.2] (147/291) | 49.5% [46.6, 52.3] (593/1199) | 459 | 44 | 13 |
| b | Stock Up | 49.2% [46.0, 52.4] (457/929) | 50.4% [46.3, 54.6] (283/561) | 690 | 148 | 26 |
| b | Acererak the Archlich | 52.0% [48.4, 55.6] (386/742) | 47.3% [43.8, 50.9] (354/748) | 566 | 229 | 144 |
| base | Stock Up | 49.4% [46.2, 52.6] (460/932) | 49.9% [45.9, 54.0] (289/579) | 658 | 166 | 20 |
| base | Acererak the Archlich | 51.2% [48.0, 54.5] (458/894) | 47.2% [43.3, 51.1] (291/617) | 606 | 261 | 141 |
| c1 | Lotus Petal | 51.8% [43.6, 59.9] (73/141) | 48.6% [45.9, 51.2] (665/1369) | 189 | 16 | 5 |
| c1 | Stock Up | 49.6% [46.2, 52.9] (423/853) | 47.9% [44.1, 51.8] (315/657) | 626 | 152 | 17 |
| c1 | Acererak the Archlich | 51.2% [47.9, 54.4] (461/901) | 45.5% [41.6, 49.5] (277/609) | 629 | 267 | 153 |
| c2 | Lotus Petal | 51.6% [43.8, 59.3] (81/157) | 50.6% [47.9, 53.3] (674/1332) | 216 | 15 | 4 |
| c2 | Stock Up | 50.9% [47.5, 54.3] (424/833) | 50.5% [46.6, 54.3] (331/656) | 596 | 137 | 13 |
| c2 | Acererak the Archlich | 50.4% [47.1, 53.7] (450/893) | 51.2% [47.2, 55.2] (305/596) | 628 | 249 | 138 |
| d1 | Stock Up | 50.6% [47.4, 53.8] (467/923) | 47.3% [43.3, 51.4] (274/579) | 706 | 161 | 24 |
| d1 | Acererak the Archlich | 51.5% [48.2, 54.8] (449/872) | 46.3% [42.5, 50.3] (292/630) | 551 | 217 | 154 |
| d2 | Stock Up | 51.7% [48.5, 54.9] (476/921) | 53.9% [49.8, 57.9] (312/579) | 696 | 180 | 25 |
| d2 | Acererak the Archlich | 55.5% [52.2, 58.8] (488/879) | 48.3% [44.4, 52.2] (300/621) | 628 | 264 | 148 |
| d3 | Stock Up | 51.2% [48.0, 54.4] (481/939) | 53.5% [49.4, 57.6] (297/555) | 663 | 181 | 18 |
| d3 | Acererak the Archlich | 52.8% [49.6, 56.1] (473/895) | 50.9% [46.9, 54.9] (305/599) | 654 | 283 | 139 |

## Example games

Chosen by rule, first matches in file order (match, game). `T<n>` is Alurentell's own turn number.

### a1

**Won, with Lotus Petal used in the last two own turns**

- match 115 game 2 (on the draw, 10 own turns, won): T1 puts Tropical Island onto the battlefield; T2 puts Hedge Maze onto the battlefield; T3 cast Lotus Petal; T5 cast Brainstorm; T5 puts Hedge Maze onto the battlefield; T6 puts Savannah onto the battlefield; T7 cast Carpet of Flowers; T7 puts Ancient Tomb onto the battlefield; T7 cast Show and Tell; T7 puts Atraxa, Grand Unifier onto the battlefield; T8 cast Prismatic Ending; T9 uses Lotus Petal; T9 cast Orim's Chant
- match 168 game 3 (on the draw, 8 own turns, won): T7 cast Acererak the Archlich; T7 cast Acererak the Archlich; T7 cast Acererak the Archlich; T7 puts The Atropal onto the battlefield; T7 cast Atraxa, Grand Unifier; T7 cast Veil of Summer; T7 puts Treasure Token onto the battlefield; T7 puts Treasure Token onto the battlefield; T7 puts City of Traitors onto the battlefield; T7 cast Acererak the Archlich; T7 cast Brainstorm; T7 cast Force of Will; T8 cast Veil of Summer; T8 cast Show and Tell
- match 271 game 2 (on the draw, 9 own turns, won): T7 cast Stock Up; T7 cast Carpet of Flowers; T7 puts Ancient Tomb onto the battlefield; T8 cast Lotus Petal; T8 cast Aluren; T8 puts Ancient Tomb onto the battlefield; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 puts The Atropal onto the battlefield; T8 uses Lotus Petal; T9 cast Carpet of Flowers

**Won, with a Stock Up cast in the last three own turns**

- match 4 game 3 (on the play, 13 own turns, won): T10 puts Skeleton Token onto the battlefield; T10 cast Acererak the Archlich; T10 cast Acererak the Archlich; T10 cast Acererak the Archlich; T10 cast Veil of Summer; T10 cast Acererak the Archlich; T10 cast Acererak the Archlich; T10 puts The Atropal onto the battlefield; T10 cast Acererak the Archlich; T10 cast Acererak the Archlich; T10 cast Carpet of Flowers; T11 cast Lotus Petal; T11 cast Ponder; T11 cast Stock Up
- match 11 game 3 (on the play, 7 own turns, won): T5 cast Acererak the Archlich; T5 cast Show and Tell; T5 puts Carpet of Flowers onto the battlefield; T5 cast Acererak the Archlich; T5 cast Acererak the Archlich; T5 cast Acererak the Archlich; T5 puts The Atropal onto the battlefield; T5 puts Treasure Token onto the battlefield; T5 cast Orim's Chant; T5 puts Flooded Strand onto the battlefield; T5 puts Island onto the battlefield; T5 cast Force of Will; T5 cast Acererak the Archlich; T6 cast Force of Will
- match 14 game 2 (on the draw, 11 own turns, won): T5 puts Ancient Tomb onto the battlefield; T6 cast Carpet of Flowers; T7 cast Stock Up; T7 puts Savannah onto the battlefield; T7 cast Show and Tell; T7 puts Atraxa, Grand Unifier onto the battlefield; T8 cast Orim's Chant; T8 cast Force of Will; T9 cast Ponder; T9 cast Stock Up; T9 cast Ponder; T9 cast Carpet of Flowers; T9 puts Misty Rainforest onto the battlefield; T10 puts Ancient Tomb onto the battlefield

**Lost after at least 5 own turns with a Stock Up stuck in hand**

- match 15 game 2 (on the play, 9 own turns, lost): T4 cast Stock Up; T5 cast Aluren; T6 cast Carpet of Flowers; T6 cast Stock Up; T6 puts Ancient Tomb onto the battlefield; T6 cast Show and Tell; T6 puts Atraxa, Grand Unifier onto the battlefield; T7 cast Atraxa, Grand Unifier; T8 cast Veil of Summer; T8 cast Ponder; T8 cast Prismatic Ending; T9 cast Acererak the Archlich; T9 cast Carpet of Flowers; T9 puts City of Traitors onto the battlefield
- match 74 game 3 (on the play, 10 own turns, lost): T7 puts Atraxa, Grand Unifier onto the battlefield; T7 puts Ancient Tomb onto the battlefield; T8 puts Tropical Island onto the battlefield; T8 cast Carpet of Flowers; T8 cast Stock Up; T9 cast Aluren; T9 cast Acererak the Archlich; T9 cast Acererak the Archlich; T9 cast Acererak the Archlich; T9 puts The Atropal onto the battlefield; T9 cast Acererak the Archlich; T9 cast Acererak the Archlich; T10 cast Acererak the Archlich; T10 cast Carpet of Flowers
- match 112 game 3 (on the draw, 8 own turns, lost): T1 puts Flooded Strand onto the battlefield; T1 puts Island onto the battlefield; T1 cast Lotus Petal; T2 puts Hedge Maze onto the battlefield; T3 uses Lotus Petal; T3 cast Show and Tell; T4 cast Veil of Summer; T5 cast Veil of Summer; T5 puts Ancient Tomb onto the battlefield; T7 puts Ancient Tomb onto the battlefield; T8 cast Stock Up; T8 cast Stock Up

**Lost after at least 5 own turns without ever seeing a Stock Up**

- match 0 game 3 (on the draw, 7 own turns, lost): T1 puts Island onto the battlefield; T2 cast Lotus Petal; T2 puts City of Traitors onto the battlefield; T2 uses Lotus Petal; T2 cast Orim's Chant; T3 puts Ancient Tomb onto the battlefield; T6 puts Misty Rainforest onto the battlefield; T6 puts Hedge Maze onto the battlefield; T7 puts Misty Rainforest onto the battlefield; T7 puts Savannah onto the battlefield; T7 cast Orim's Chant
- match 1 game 3 (on the draw, 6 own turns, lost): T3 puts Misty Rainforest onto the battlefield; T3 puts Island onto the battlefield
- match 9 game 2 (on the play, 13 own turns, lost): T1 puts Island onto the battlefield; T2 cast Ponder; T2 puts Hedge Maze onto the battlefield; T2 cast Force of Will; T3 cast Veil of Summer; T3 puts Ancient Tomb onto the battlefield; T8 puts Tropical Island onto the battlefield; T9 cast Brainstorm; T9 cast Veil of Summer; T12 cast Ponder; T12 puts Ancient Tomb onto the battlefield; T12 cast Ponder; T13 puts Ancient Tomb onto the battlefield

### a2

**Won, with Lotus Petal used in the last two own turns**

- match 11 game 3 (on the play, 7 own turns, won): T5 cast Acererak the Archlich; T5 cast Acererak the Archlich; T5 puts The Atropal onto the battlefield; T5 cast Acererak the Archlich; T5 cast Ponder; T5 puts Treasure Token onto the battlefield; T5 cast Ponder; T5 cast Orim's Chant; T5 cast Lotus Petal; T6 uses Lotus Petal; T6 cast Acererak the Archlich; T6 puts Zombie Token onto the battlefield; T6 cast Force of Will; T7 puts Zombie Token onto the battlefield
- match 13 game 2 (on the draw, 9 own turns, won): T8 cast Lotus Petal; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 puts The Atropal onto the battlefield; T9 cast Stock Up; T9 uses Lotus Petal; T9 cast Orim's Chant; T9 cast Force of Will; T9 puts Zombie Token onto the battlefield
- match 15 game 3 (on the draw, 11 own turns, won): T4 puts Flooded Strand onto the battlefield; T4 puts Island onto the battlefield; T5 cast Brainstorm; T6 cast Stock Up; T6 cast Lotus Petal; T6 uses Lotus Petal; T6 cast Prismatic Ending; T7 cast Show and Tell; T7 puts Atraxa, Grand Unifier onto the battlefield; T9 cast Brainstorm; T10 cast Lotus Petal; T10 uses Lotus Petal; T10 cast Prismatic Ending; T11 puts Misty Rainforest onto the battlefield

**Won, with a Stock Up cast in the last three own turns**

- match 6 game 2 (on the draw, 9 own turns, won): T5 puts Flooded Strand onto the battlefield; T5 puts Hedge Maze onto the battlefield; T6 cast Atraxa, Grand Unifier; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 puts The Atropal onto the battlefield; T8 cast Stock Up; T8 cast Acererak the Archlich; T8 puts Flooded Strand onto the battlefield; T9 puts Zombie Token onto the battlefield
- match 10 game 2 (on the draw, 10 own turns, won): T5 puts Goblin Token onto the battlefield; T6 puts Misty Rainforest onto the battlefield; T6 cast Ponder; T8 cast Show and Tell; T8 puts Atraxa, Grand Unifier onto the battlefield; T8 puts Misty Rainforest onto the battlefield; T9 cast Force of Will; T9 puts Tropical Island onto the battlefield; T9 puts Tundra onto the battlefield; T9 puts Ancient Tomb onto the battlefield; T10 puts Ancient Tomb onto the battlefield; T10 cast Aluren; T10 cast Stock Up; T10 cast Acererak the Archlich
- match 12 game 2 (on the play, 11 own turns, won): T10 cast Acererak the Archlich; T10 cast Acererak the Archlich; T10 cast Acererak the Archlich; T10 puts The Atropal onto the battlefield; T10 cast Acererak the Archlich; T10 cast Acererak the Archlich; T10 puts Goblin Token onto the battlefield; T10 cast Atraxa, Grand Unifier; T10 cast Veil of Summer; T10 cast Acererak the Archlich; T10 puts Treasure Token onto the battlefield; T10 puts Treasure Token onto the battlefield; T11 cast Lotus Petal; T11 puts Zombie Token onto the battlefield

**Lost after at least 5 own turns with a Stock Up stuck in hand**

- match 31 game 3 (on the draw, 7 own turns, lost): T7 cast Acererak the Archlich; T7 puts Treasure Token onto the battlefield; T7 cast Acererak the Archlich; T7 cast Acererak the Archlich; T7 puts Skeleton Token onto the battlefield; T7 puts Skeleton Token onto the battlefield; T7 cast Acererak the Archlich; T7 cast Acererak the Archlich; T7 cast Brainstorm; T7 cast Acererak the Archlich; T7 cast Acererak the Archlich; T7 uses Lotus Petal; T7 cast Show and Tell; T7 puts Hedge Maze onto the battlefield
- match 74 game 3 (on the play, 11 own turns, lost): T11 cast Acererak the Archlich; T11 cast Acererak the Archlich; T11 cast Acererak the Archlich; T11 cast Acererak the Archlich; T11 cast Acererak the Archlich; T11 cast Acererak the Archlich; T11 cast Acererak the Archlich; T11 cast Acererak the Archlich; T11 cast Acererak the Archlich; T11 puts The Atropal onto the battlefield; T11 cast Acererak the Archlich; T11 cast Acererak the Archlich; T11 cast Ponder; T11 cast Acererak the Archlich
- match 86 game 3 (on the play, 10 own turns, lost): T8 cast Acererak the Archlich; T8 puts The Atropal onto the battlefield; T8 cast Acererak the Archlich; T8 puts Skeleton Token onto the battlefield; T8 puts Skeleton Token onto the battlefield; T8 puts Treasure Token onto the battlefield; T8 cast Carpet of Flowers; T8 cast Ponder; T8 cast Acererak the Archlich; T9 cast Stock Up; T9 puts Zombie Token onto the battlefield; T10 cast Veil of Summer; T10 cast Prismatic Ending; T10 cast Force of Will

**Lost after at least 5 own turns without ever seeing a Stock Up**

- match 0 game 3 (on the draw, 8 own turns, lost): T2 puts City of Traitors onto the battlefield; T2 uses Lotus Petal; T2 cast Orim's Chant; T3 puts Ancient Tomb onto the battlefield; T3 cast Lotus Petal; T5 uses Lotus Petal; T5 cast Orim's Chant; T5 puts Ancient Tomb onto the battlefield; T6 puts Misty Rainforest onto the battlefield; T6 puts Hedge Maze onto the battlefield; T7 puts Misty Rainforest onto the battlefield; T7 puts Hedge Maze onto the battlefield; T8 cast Ponder; T8 cast Show and Tell
- match 11 game 2 (on the draw, 9 own turns, lost): T1 puts Misty Rainforest onto the battlefield; T1 puts Island onto the battlefield; T8 puts Tundra onto the battlefield
- match 28 game 2 (on the play, 8 own turns, lost): T1 puts Ancient Tomb onto the battlefield; T2 puts Ancient Tomb onto the battlefield; T4 puts City of Traitors onto the battlefield; T7 puts Savannah onto the battlefield; T7 cast Carpet of Flowers; T7 cast Show and Tell; T8 cast Orim's Chant; T8 cast Show and Tell; T8 puts Aluren onto the battlefield; T8 cast Lotus Petal; T8 uses Lotus Petal; T8 cast Carpet of Flowers

### b

**Won, with Lotus Petal used in the last two own turns**

- match 23 game 2 (on the draw, 12 own turns, won): T11 cast Acererak the Archlich; T11 cast Acererak the Archlich; T11 puts Goblin Token onto the battlefield; T11 cast Acererak the Archlich; T11 cast Acererak the Archlich; T11 cast Acererak the Archlich; T11 cast Acererak the Archlich; T11 cast Acererak the Archlich; T11 puts The Atropal onto the battlefield; T11 cast Acererak the Archlich; T11 uses Lotus Petal; T11 cast Acererak the Archlich; T12 cast Show and Tell; T12 puts Hedge Maze onto the battlefield
- match 48 game 3 (on the draw, 11 own turns, won): T10 cast Acererak the Archlich; T10 cast Acererak the Archlich; T10 cast Acererak the Archlich; T10 cast Acererak the Archlich; T10 cast Acererak the Archlich; T10 cast Acererak the Archlich; T10 cast Acererak the Archlich; T10 puts The Atropal onto the battlefield; T10 cast Acererak the Archlich; T10 puts Goblin Token onto the battlefield; T10 puts Treasure Token onto the battlefield; T10 cast Carpet of Flowers; T11 cast Veil of Summer; T11 puts Zombie Token onto the battlefield
- match 77 game 2 (on the draw, 23 own turns, won): T11 puts Hedge Maze onto the battlefield; T14 cast Veil of Summer; T17 puts Flooded Strand onto the battlefield; T18 cast Show and Tell; T18 puts Island onto the battlefield; T18 puts Atraxa, Grand Unifier onto the battlefield; T19 cast Brainstorm; T19 cast Orim's Chant; T20 cast Show and Tell; T20 puts Atraxa, Grand Unifier onto the battlefield; T22 cast Lotus Petal; T22 uses Lotus Petal; T22 cast Acererak the Archlich; T22 puts Misty Rainforest onto the battlefield

**Won, with a Stock Up cast in the last three own turns**

- match 1 game 2 (on the play, 20 own turns, won): T14 cast Acererak the Archlich; T16 cast Acererak the Archlich; T16 cast Acererak the Archlich; T16 cast Acererak the Archlich; T16 puts The Atropal onto the battlefield; T17 cast Acererak the Archlich; T17 cast Orim's Chant; T18 puts Savannah onto the battlefield; T18 puts Zombie Token onto the battlefield; T18 cast Stock Up; T18 cast Acererak the Archlich; T19 puts Misty Rainforest onto the battlefield; T20 puts Flooded Strand onto the battlefield; T20 cast Stock Up
- match 3 game 2 (on the draw, 6 own turns, won): T0 cast Force of Will; T1 puts Savannah onto the battlefield; T3 cast Carpet of Flowers; T4 puts Misty Rainforest onto the battlefield; T4 puts Island onto the battlefield; T4 cast Show and Tell; T4 puts Atraxa, Grand Unifier onto the battlefield; T4 cast Orim's Chant; T5 cast Stock Up
- match 4 game 3 (on the play, 14 own turns, won): T2 cast Brainstorm; T5 puts Savannah onto the battlefield; T6 puts Misty Rainforest onto the battlefield; T7 puts Tropical Island onto the battlefield; T9 puts Island onto the battlefield; T9 cast Show and Tell; T9 puts Atraxa, Grand Unifier onto the battlefield; T11 cast Show and Tell; T11 puts Atraxa, Grand Unifier onto the battlefield; T11 puts Tropical Island onto the battlefield; T12 cast Stock Up; T13 cast Prismatic Ending; T13 cast Force of Will; T14 cast Prismatic Ending

**Lost after at least 5 own turns with a Stock Up stuck in hand**

- match 0 game 2 (on the play, 6 own turns, lost): T4 puts Skeleton Token onto the battlefield; T4 puts Treasure Token onto the battlefield; T4 puts Hedge Maze onto the battlefield; T4 cast Brainstorm; T5 puts Misty Rainforest onto the battlefield; T5 cast Lotus Petal; T5 puts Hedge Maze onto the battlefield; T5 cast Stock Up; T5 puts Zombie Token onto the battlefield; T6 cast Ponder; T6 puts Flooded Strand onto the battlefield; T6 puts Zombie Token onto the battlefield; T6 uses Lotus Petal; T6 cast Stock Up
- match 8 game 2 (on the play, 9 own turns, lost): T9 cast Acererak the Archlich; T9 cast Acererak the Archlich; T9 cast Acererak the Archlich; T9 cast Acererak the Archlich; T9 cast Acererak the Archlich; T9 puts Treasure Token onto the battlefield; T9 cast Acererak the Archlich; T9 cast Acererak the Archlich; T9 cast Acererak the Archlich; T9 cast Acererak the Archlich; T9 cast Acererak the Archlich; T9 cast Acererak the Archlich; T9 puts The Atropal onto the battlefield; T9 puts Treasure Token onto the battlefield
- match 40 game 2 (on the draw, 6 own turns, lost): T1 puts Flooded Strand onto the battlefield; T1 puts Island onto the battlefield; T1 cast Force of Will; T2 puts Hedge Maze onto the battlefield; T3 cast Veil of Summer; T5 puts Savannah onto the battlefield; T5 cast Show and Tell; T5 puts Atraxa, Grand Unifier onto the battlefield; T6 cast Show and Tell; T6 puts Atraxa, Grand Unifier onto the battlefield; T6 puts Ancient Tomb onto the battlefield

**Lost after at least 5 own turns without ever seeing a Stock Up**

- match 7 game 3 (on the play, 6 own turns, lost): T3 puts Ancient Tomb onto the battlefield; T3 cast Carpet of Flowers; T4 cast Carpet of Flowers; T4 puts Savannah onto the battlefield; T4 cast Orim's Chant; T5 cast Brainstorm; T5 cast Acererak the Archlich; T5 cast Acererak the Archlich; T5 puts Treasure Token onto the battlefield; T6 cast Veil of Summer; T6 cast Acererak the Archlich; T6 cast Acererak the Archlich; T6 puts Tundra onto the battlefield; T6 cast Prismatic Ending
- match 9 game 2 (on the play, 10 own turns, lost): T1 puts Island onto the battlefield; T1 cast Ponder; T2 cast Ponder; T2 puts Ancient Tomb onto the battlefield; T3 puts Hedge Maze onto the battlefield; T4 cast Veil of Summer; T5 cast Brainstorm; T6 cast Brainstorm
- match 19 game 2 (on the draw, 13 own turns, lost): T1 puts Hedge Maze onto the battlefield; T5 cast Veil of Summer; T5 puts Ancient Tomb onto the battlefield; T7 cast Show and Tell; T8 cast Ponder; T9 cast Lotus Petal; T9 puts Ancient Tomb onto the battlefield; T10 puts Ancient Tomb onto the battlefield; T11 cast Force of Will; T13 puts Tundra onto the battlefield; T13 cast Orim's Chant

### base

**Won, with Lotus Petal used in the last two own turns**

none


**Won, with a Stock Up cast in the last three own turns**

- match 0 game 3 (on the draw, 15 own turns, won): T14 cast Acererak the Archlich; T14 puts The Atropal onto the battlefield; T14 cast Acererak the Archlich; T14 cast Stock Up; T14 cast Acererak the Archlich; T14 cast Acererak the Archlich; T14 puts Skeleton Token onto the battlefield; T14 puts Skeleton Token onto the battlefield; T14 puts Ancient Tomb onto the battlefield; T14 cast Stock Up; T14 cast Show and Tell; T14 puts Flooded Strand onto the battlefield; T14 puts Tropical Island onto the battlefield; T15 puts Zombie Token onto the battlefield
- match 4 game 3 (on the play, 9 own turns, won): T7 puts Tropical Island onto the battlefield; T8 puts Misty Rainforest onto the battlefield; T8 cast Stock Up; T8 cast Carpet of Flowers; T8 puts Hedge Maze onto the battlefield; T9 cast Show and Tell; T9 puts Atraxa, Grand Unifier onto the battlefield; T9 cast Carpet of Flowers; T9 cast Stock Up; T9 cast Stock Up; T9 puts Flooded Strand onto the battlefield; T9 puts Tundra onto the battlefield; T9 cast Prismatic Ending; T9 cast Acererak the Archlich
- match 6 game 2 (on the draw, 7 own turns, won): T4 cast Acererak the Archlich; T4 puts The Atropal onto the battlefield; T5 puts Misty Rainforest onto the battlefield; T5 puts Tundra onto the battlefield; T5 cast Brainstorm; T5 puts Zombie Token onto the battlefield; T6 puts Flooded Strand onto the battlefield; T6 puts Island onto the battlefield; T6 cast Atraxa, Grand Unifier; T7 cast Prismatic Ending; T7 puts Misty Rainforest onto the battlefield; T7 puts Hedge Maze onto the battlefield; T7 cast Stock Up; T7 cast Acererak the Archlich

**Lost after at least 5 own turns with a Stock Up stuck in hand**

- match 9 game 2 (on the play, 23 own turns, lost): T8 puts Tropical Island onto the battlefield; T9 cast Brainstorm; T11 cast Veil of Summer; T21 cast Stock Up; T21 cast Carpet of Flowers; T21 puts Ancient Tomb onto the battlefield; T22 cast Veil of Summer; T22 cast Show and Tell; T22 puts Hedge Maze onto the battlefield; T22 puts City of Traitors onto the battlefield; T23 puts Flooded Strand onto the battlefield; T23 cast Stock Up; T23 puts Tropical Island onto the battlefield; T23 cast Stock Up
- match 37 game 2 (on the draw, 5 own turns, lost): T3 cast Show and Tell; T3 cast Veil of Summer; T3 puts Atraxa, Grand Unifier onto the battlefield; T4 cast Veil of Summer; T4 cast Veil of Summer; T4 puts Flooded Strand onto the battlefield; T4 puts Island onto the battlefield; T4 cast Stock Up; T5 cast Prismatic Ending; T5 cast Show and Tell; T5 puts Atraxa, Grand Unifier onto the battlefield; T5 puts Misty Rainforest onto the battlefield; T5 puts Tropical Island onto the battlefield; T5 cast Veil of Summer
- match 108 game 2 (on the draw, 6 own turns, lost): T0 cast Force of Will; T1 puts Tropical Island onto the battlefield; T2 puts Island onto the battlefield; T4 cast Brainstorm; T4 cast Brainstorm; T4 puts Hedge Maze onto the battlefield; T5 cast Stock Up; T5 puts Ancient Tomb onto the battlefield; T6 cast Aluren

**Lost after at least 5 own turns without ever seeing a Stock Up**

- match 24 game 3 (on the play, 8 own turns, lost): T1 puts Tropical Island onto the battlefield; T1 cast Veil of Summer; T2 cast Brainstorm; T2 puts Tundra onto the battlefield; T3 cast Veil of Summer; T3 puts Ancient Tomb onto the battlefield; T4 cast Brainstorm; T4 cast Show and Tell; T4 puts Aluren onto the battlefield; T6 cast Prismatic Ending; T8 cast Carpet of Flowers
- match 28 game 2 (on the play, 8 own turns, lost): T1 puts Ancient Tomb onto the battlefield; T3 puts Ancient Tomb onto the battlefield; T7 puts Savannah onto the battlefield; T7 cast Carpet of Flowers; T8 cast Orim's Chant
- match 31 game 3 (on the draw, 5 own turns, lost): T1 puts Ancient Tomb onto the battlefield; T2 puts Flooded Strand onto the battlefield; T2 puts Hedge Maze onto the battlefield; T3 cast Brainstorm; T3 puts Ancient Tomb onto the battlefield; T4 cast Show and Tell; T4 puts Tundra onto the battlefield; T4 cast Prismatic Ending; T4 puts Ancient Tomb onto the battlefield; T5 cast Orim's Chant; T5 cast Carpet of Flowers; T5 puts Flooded Strand onto the battlefield; T5 puts Savannah onto the battlefield; T5 cast Acererak the Archlich

### c1

**Won, with Lotus Petal used in the last two own turns**

- match 310 game 2 (on the play, 8 own turns, won): T5 cast Acererak the Archlich; T5 cast Acererak the Archlich; T5 puts The Atropal onto the battlefield; T5 cast Acererak the Archlich; T5 cast Acererak the Archlich; T5 puts Skeleton Token onto the battlefield; T5 puts Skeleton Token onto the battlefield; T5 puts Treasure Token onto the battlefield; T7 cast Lotus Petal; T7 uses Lotus Petal; T7 cast Atraxa, Grand Unifier; T7 puts Zombie Token onto the battlefield; T7 cast Acererak the Archlich; T8 puts Zombie Token onto the battlefield
- match 320 game 2 (on the play, 8 own turns, won): T4 puts Hedge Maze onto the battlefield; T4 cast Acererak the Archlich; T5 cast Atraxa, Grand Unifier; T5 puts Misty Rainforest onto the battlefield; T5 puts Tropical Island onto the battlefield; T6 cast Force of Will; T7 cast Brainstorm; T7 cast Ponder; T7 puts Tropical Island onto the battlefield; T7 cast Lotus Petal; T7 uses Lotus Petal; T7 cast Atraxa, Grand Unifier; T8 cast Orim's Chant; T8 puts Flooded Strand onto the battlefield
- match 390 game 3 (on the play, 8 own turns, won): T7 puts Tropical Island onto the battlefield; T7 cast Lotus Petal; T7 uses Lotus Petal; T7 cast Aluren; T7 cast Acererak the Archlich; T7 cast Acererak the Archlich; T7 cast Acererak the Archlich; T7 cast Acererak the Archlich; T7 cast Acererak the Archlich; T7 cast Acererak the Archlich; T7 puts The Atropal onto the battlefield; T8 cast Acererak the Archlich; T8 cast Stock Up; T8 cast Veil of Summer

**Won, with a Stock Up cast in the last three own turns**

- match 0 game 3 (on the draw, 15 own turns, won): T14 cast Acererak the Archlich; T14 puts The Atropal onto the battlefield; T14 cast Acererak the Archlich; T14 cast Stock Up; T14 cast Acererak the Archlich; T14 cast Acererak the Archlich; T14 puts Skeleton Token onto the battlefield; T14 puts Skeleton Token onto the battlefield; T14 puts Ancient Tomb onto the battlefield; T14 cast Stock Up; T14 cast Show and Tell; T14 puts Flooded Strand onto the battlefield; T14 puts Tropical Island onto the battlefield; T15 puts Zombie Token onto the battlefield
- match 4 game 3 (on the play, 13 own turns, won): T10 puts Skeleton Token onto the battlefield; T10 cast Acererak the Archlich; T10 cast Acererak the Archlich; T10 cast Acererak the Archlich; T10 cast Veil of Summer; T10 cast Acererak the Archlich; T10 cast Acererak the Archlich; T10 puts The Atropal onto the battlefield; T10 cast Acererak the Archlich; T10 cast Acererak the Archlich; T10 cast Carpet of Flowers; T11 cast Lotus Petal; T11 cast Ponder; T11 cast Stock Up
- match 6 game 2 (on the draw, 7 own turns, won): T4 cast Acererak the Archlich; T4 puts The Atropal onto the battlefield; T5 puts Misty Rainforest onto the battlefield; T5 puts Tundra onto the battlefield; T5 cast Brainstorm; T5 puts Zombie Token onto the battlefield; T6 puts Flooded Strand onto the battlefield; T6 puts Island onto the battlefield; T6 cast Atraxa, Grand Unifier; T7 cast Prismatic Ending; T7 puts Misty Rainforest onto the battlefield; T7 puts Hedge Maze onto the battlefield; T7 cast Stock Up; T7 cast Acererak the Archlich

**Lost after at least 5 own turns with a Stock Up stuck in hand**

- match 15 game 2 (on the play, 9 own turns, lost): T4 cast Stock Up; T5 cast Aluren; T6 cast Carpet of Flowers; T6 cast Stock Up; T6 puts Ancient Tomb onto the battlefield; T6 cast Show and Tell; T6 puts Atraxa, Grand Unifier onto the battlefield; T7 cast Atraxa, Grand Unifier; T8 cast Veil of Summer; T8 cast Ponder; T8 cast Prismatic Ending; T9 cast Acererak the Archlich; T9 cast Carpet of Flowers; T9 puts City of Traitors onto the battlefield
- match 37 game 2 (on the draw, 5 own turns, lost): T3 cast Show and Tell; T3 cast Veil of Summer; T3 puts Atraxa, Grand Unifier onto the battlefield; T4 cast Veil of Summer; T4 cast Veil of Summer; T4 puts Flooded Strand onto the battlefield; T4 puts Island onto the battlefield; T4 cast Stock Up; T5 cast Prismatic Ending; T5 cast Show and Tell; T5 puts Atraxa, Grand Unifier onto the battlefield; T5 puts Misty Rainforest onto the battlefield; T5 puts Tropical Island onto the battlefield; T5 cast Veil of Summer
- match 74 game 3 (on the play, 10 own turns, lost): T7 puts Atraxa, Grand Unifier onto the battlefield; T7 puts Ancient Tomb onto the battlefield; T8 puts Tropical Island onto the battlefield; T8 cast Carpet of Flowers; T8 cast Stock Up; T9 cast Aluren; T9 cast Acererak the Archlich; T9 cast Acererak the Archlich; T9 cast Acererak the Archlich; T9 puts The Atropal onto the battlefield; T9 cast Acererak the Archlich; T9 cast Acererak the Archlich; T10 cast Acererak the Archlich; T10 cast Carpet of Flowers

**Lost after at least 5 own turns without ever seeing a Stock Up**

- match 2 game 3 (on the play, 8 own turns, lost): T1 puts Tropical Island onto the battlefield; T1 cast Brainstorm; T1 cast Force of Will; T2 cast Carpet of Flowers; T2 puts Flooded Strand onto the battlefield; T2 puts Hedge Maze onto the battlefield; T3 puts Tundra onto the battlefield; T3 cast Show and Tell; T3 puts Aluren onto the battlefield; T6 puts Misty Rainforest onto the battlefield; T6 puts Island onto the battlefield; T6 cast Show and Tell; T7 puts Hedge Maze onto the battlefield; T8 cast Aluren
- match 9 game 2 (on the play, 13 own turns, lost): T1 puts Island onto the battlefield; T2 cast Ponder; T2 puts Hedge Maze onto the battlefield; T2 cast Force of Will; T3 cast Veil of Summer; T3 puts Ancient Tomb onto the battlefield; T8 puts Tropical Island onto the battlefield; T9 cast Brainstorm; T9 cast Veil of Summer; T12 cast Ponder; T12 puts Ancient Tomb onto the battlefield; T12 cast Ponder; T13 puts Ancient Tomb onto the battlefield
- match 24 game 3 (on the play, 6 own turns, lost): T2 cast Lotus Petal; T2 puts Tropical Island onto the battlefield; T2 cast Veil of Summer; T3 cast Brainstorm; T3 puts Ancient Tomb onto the battlefield; T4 cast Brainstorm; T4 puts Tundra onto the battlefield; T4 cast Show and Tell; T4 puts Aluren onto the battlefield; T5 cast Veil of Summer; T6 cast Prismatic Ending; T6 puts Misty Rainforest onto the battlefield; T6 puts Island onto the battlefield

### c2

**Won, with Lotus Petal used in the last two own turns**

- match 115 game 2 (on the draw, 10 own turns, won): T1 puts Tropical Island onto the battlefield; T2 puts Hedge Maze onto the battlefield; T3 cast Lotus Petal; T5 cast Brainstorm; T5 puts Hedge Maze onto the battlefield; T6 puts Savannah onto the battlefield; T7 cast Carpet of Flowers; T7 puts Ancient Tomb onto the battlefield; T7 cast Show and Tell; T7 puts Atraxa, Grand Unifier onto the battlefield; T8 cast Prismatic Ending; T9 uses Lotus Petal; T9 cast Orim's Chant
- match 168 game 3 (on the draw, 8 own turns, won): T7 cast Acererak the Archlich; T7 cast Acererak the Archlich; T7 cast Acererak the Archlich; T7 puts The Atropal onto the battlefield; T7 cast Atraxa, Grand Unifier; T7 cast Veil of Summer; T7 puts Treasure Token onto the battlefield; T7 puts Treasure Token onto the battlefield; T7 puts City of Traitors onto the battlefield; T7 cast Acererak the Archlich; T7 cast Brainstorm; T7 cast Force of Will; T8 cast Veil of Summer; T8 cast Show and Tell
- match 271 game 2 (on the draw, 9 own turns, won): T7 cast Stock Up; T7 cast Carpet of Flowers; T7 puts Ancient Tomb onto the battlefield; T8 cast Lotus Petal; T8 cast Aluren; T8 puts Ancient Tomb onto the battlefield; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 puts The Atropal onto the battlefield; T8 uses Lotus Petal; T9 cast Carpet of Flowers

**Won, with a Stock Up cast in the last three own turns**

- match 4 game 3 (on the play, 9 own turns, won): T7 puts Tropical Island onto the battlefield; T8 puts Misty Rainforest onto the battlefield; T8 cast Stock Up; T8 cast Carpet of Flowers; T8 puts Hedge Maze onto the battlefield; T9 cast Show and Tell; T9 puts Atraxa, Grand Unifier onto the battlefield; T9 cast Carpet of Flowers; T9 cast Stock Up; T9 cast Stock Up; T9 puts Flooded Strand onto the battlefield; T9 puts Tundra onto the battlefield; T9 cast Prismatic Ending; T9 cast Acererak the Archlich
- match 14 game 2 (on the draw, 11 own turns, won): T5 puts Ancient Tomb onto the battlefield; T6 cast Carpet of Flowers; T7 cast Stock Up; T7 puts Savannah onto the battlefield; T7 cast Show and Tell; T7 puts Atraxa, Grand Unifier onto the battlefield; T8 cast Orim's Chant; T8 cast Force of Will; T9 cast Ponder; T9 cast Stock Up; T9 cast Ponder; T9 cast Carpet of Flowers; T9 puts Misty Rainforest onto the battlefield; T10 puts Ancient Tomb onto the battlefield
- match 18 game 2 (on the draw, 14 own turns, won): T4 puts Savannah onto the battlefield; T4 cast Orim's Chant; T5 puts Tropical Island onto the battlefield; T5 cast Carpet of Flowers; T6 cast Atraxa, Grand Unifier; T6 puts Tropical Island onto the battlefield; T8 puts Misty Rainforest onto the battlefield; T10 cast Veil of Summer; T11 cast Show and Tell; T11 puts Atraxa, Grand Unifier onto the battlefield; T12 cast Force of Will; T13 cast Stock Up; T13 puts Misty Rainforest onto the battlefield; T13 cast Show and Tell

**Lost after at least 5 own turns with a Stock Up stuck in hand**

- match 9 game 2 (on the play, 23 own turns, lost): T8 puts Tropical Island onto the battlefield; T9 cast Brainstorm; T11 cast Veil of Summer; T21 cast Stock Up; T21 cast Carpet of Flowers; T21 puts Ancient Tomb onto the battlefield; T22 cast Veil of Summer; T22 cast Show and Tell; T22 puts Hedge Maze onto the battlefield; T22 puts City of Traitors onto the battlefield; T23 puts Flooded Strand onto the battlefield; T23 cast Stock Up; T23 puts Tropical Island onto the battlefield; T23 cast Stock Up
- match 112 game 3 (on the draw, 8 own turns, lost): T1 puts Flooded Strand onto the battlefield; T1 puts Island onto the battlefield; T1 cast Lotus Petal; T2 puts Hedge Maze onto the battlefield; T3 uses Lotus Petal; T3 cast Show and Tell; T4 cast Veil of Summer; T5 cast Veil of Summer; T5 puts Ancient Tomb onto the battlefield; T7 puts Ancient Tomb onto the battlefield; T8 cast Stock Up; T8 cast Stock Up
- match 215 game 3 (on the play, 6 own turns, lost): T6 cast Acererak the Archlich; T6 cast Acererak the Archlich; T6 cast Acererak the Archlich; T6 cast Acererak the Archlich; T6 cast Acererak the Archlich; T6 puts Goblin Token onto the battlefield; T6 cast Acererak the Archlich; T6 cast Veil of Summer; T6 cast Acererak the Archlich; T6 cast Acererak the Archlich; T6 puts The Atropal onto the battlefield; T6 cast Acererak the Archlich; T6 puts Goblin Token onto the battlefield; T6 cast Acererak the Archlich

**Lost after at least 5 own turns without ever seeing a Stock Up**

- match 0 game 3 (on the draw, 7 own turns, lost): T1 puts Island onto the battlefield; T2 cast Lotus Petal; T2 puts City of Traitors onto the battlefield; T2 uses Lotus Petal; T2 cast Orim's Chant; T3 puts Ancient Tomb onto the battlefield; T6 puts Misty Rainforest onto the battlefield; T6 puts Hedge Maze onto the battlefield; T7 puts Misty Rainforest onto the battlefield; T7 puts Savannah onto the battlefield; T7 cast Orim's Chant
- match 28 game 2 (on the play, 8 own turns, lost): T1 puts Ancient Tomb onto the battlefield; T3 puts Ancient Tomb onto the battlefield; T7 puts Savannah onto the battlefield; T7 cast Carpet of Flowers; T8 cast Orim's Chant
- match 35 game 2 (on the play, 13 own turns, lost): T5 cast Carpet of Flowers; T5 puts Hedge Maze onto the battlefield; T6 puts Flooded Strand onto the battlefield; T6 puts Hedge Maze onto the battlefield; T6 cast Prismatic Ending; T7 cast Prismatic Ending; T7 cast Aluren; T7 cast Force of Will; T9 cast Carpet of Flowers; T9 puts Ancient Tomb onto the battlefield; T10 puts Ancient Tomb onto the battlefield; T11 cast Ponder; T12 cast Acererak the Archlich; T13 cast Show and Tell

### d1

**Won, with Lotus Petal used in the last two own turns**

none


**Won, with a Stock Up cast in the last three own turns**

- match 13 game 3 (on the play, 21 own turns, won): T10 cast Acererak the Archlich; T11 cast Acererak the Archlich; T11 cast Carpet of Flowers; T12 cast Acererak the Archlich; T15 cast Brainstorm; T15 cast Force of Will; T15 puts Ancient Tomb onto the battlefield; T16 cast Acererak the Archlich; T16 puts The Atropal onto the battlefield; T16 puts Ancient Tomb onto the battlefield; T17 cast Ponder; T20 cast Stock Up; T20 puts Tropical Island onto the battlefield; T21 cast Brainstorm
- match 20 game 2 (on the draw, 8 own turns, won): T6 cast Acererak the Archlich; T6 cast Acererak the Archlich; T6 cast Acererak the Archlich; T6 cast Acererak the Archlich; T6 puts The Atropal onto the battlefield; T6 cast Acererak the Archlich; T6 cast Acererak the Archlich; T6 cast Stock Up; T6 cast Acererak the Archlich; T7 cast Prismatic Ending; T7 puts Flooded Strand onto the battlefield; T7 puts Tropical Island onto the battlefield; T7 puts Zombie Token onto the battlefield; T8 puts Zombie Token onto the battlefield
- match 26 game 3 (on the play, 14 own turns, won): T10 puts Savannah onto the battlefield; T10 cast Acererak the Archlich; T11 cast Aluren; T11 puts Ancient Tomb onto the battlefield; T12 cast Atraxa, Grand Unifier; T12 puts Misty Rainforest onto the battlefield; T13 cast Stock Up; T13 cast Ponder; T13 puts Misty Rainforest onto the battlefield; T14 puts Misty Rainforest onto the battlefield; T14 cast Brainstorm; T14 cast Show and Tell; T14 puts Acererak the Archlich onto the battlefield; T14 puts Goblin Token onto the battlefield

**Lost after at least 5 own turns with a Stock Up stuck in hand**

- match 22 game 2 (on the play, 15 own turns, lost): T10 cast Carpet of Flowers; T10 cast Show and Tell; T10 cast Veil of Summer; T11 cast Brainstorm; T11 cast Prismatic Ending; T11 puts Misty Rainforest onto the battlefield; T12 cast Atraxa, Grand Unifier; T12 puts Hedge Maze onto the battlefield; T13 cast Ponder; T13 cast Show and Tell; T13 puts Hedge Maze onto the battlefield; T13 puts Ancient Tomb onto the battlefield; T15 cast Veil of Summer; T15 puts City of Traitors onto the battlefield
- match 37 game 3 (on the play, 10 own turns, lost): T4 cast Carpet of Flowers; T4 cast Ponder; T5 cast Orim's Chant; T6 cast Ponder; T6 cast Carpet of Flowers; T8 cast Acererak the Archlich; T8 puts Tundra onto the battlefield; T9 puts Flooded Strand onto the battlefield; T9 puts Hedge Maze onto the battlefield; T9 cast Acererak the Archlich; T10 puts Flooded Strand onto the battlefield; T10 puts Hedge Maze onto the battlefield; T10 cast Acererak the Archlich; T10 cast Stock Up
- match 201 game 2 (on the play, 12 own turns, lost): T11 cast Acererak the Archlich; T11 cast Acererak the Archlich; T11 cast Acererak the Archlich; T11 puts The Atropal onto the battlefield; T11 puts Treasure Token onto the battlefield; T11 cast Carpet of Flowers; T11 cast Brainstorm; T11 cast Acererak the Archlich; T11 cast Show and Tell; T11 puts Atraxa, Grand Unifier onto the battlefield; T11 cast Stock Up; T11 cast Stock Up; T12 puts Zombie Token onto the battlefield; T12 puts Misty Rainforest onto the battlefield

**Lost after at least 5 own turns without ever seeing a Stock Up**

- match 0 game 2 (on the play, 8 own turns, lost): T1 puts Hedge Maze onto the battlefield; T2 puts Tropical Island onto the battlefield; T2 cast Force of Will; T3 puts Misty Rainforest onto the battlefield; T3 puts Hedge Maze onto the battlefield; T5 puts Tundra onto the battlefield; T6 cast Brainstorm; T7 cast Show and Tell; T8 puts Savannah onto the battlefield; T8 cast Aluren
- match 10 game 2 (on the draw, 7 own turns, lost): T3 cast Veil of Summer; T3 puts Tundra onto the battlefield; T3 cast Show and Tell; T3 puts Atraxa, Grand Unifier onto the battlefield; T4 cast Veil of Summer; T4 cast Prismatic Ending; T4 puts Misty Rainforest onto the battlefield; T4 puts Island onto the battlefield; T5 cast Orim's Chant; T6 cast Prismatic Ending; T6 cast Acererak the Archlich; T7 cast Acererak the Archlich; T7 puts Treasure Token onto the battlefield; T7 cast Acererak the Archlich
- match 11 game 3 (on the play, 10 own turns, lost): T2 puts Ancient Tomb onto the battlefield; T3 puts Misty Rainforest onto the battlefield; T3 puts Hedge Maze onto the battlefield; T4 cast Veil of Summer; T5 cast Veil of Summer; T6 cast Brainstorm; T7 cast Carpet of Flowers; T9 puts Misty Rainforest onto the battlefield; T9 puts Island onto the battlefield; T10 cast Acererak the Archlich; T10 puts Ancient Tomb onto the battlefield

### d2

**Won, with Lotus Petal used in the last two own turns**

none


**Won, with a Stock Up cast in the last three own turns**

- match 1 game 2 (on the play, 9 own turns, won): T8 cast Veil of Summer; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 puts The Atropal onto the battlefield; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 cast Atraxa, Grand Unifier; T8 cast Carpet of Flowers; T8 cast Stock Up; T8 cast Carpet of Flowers; T9 puts Zombie Token onto the battlefield
- match 3 game 2 (on the draw, 11 own turns, won): T1 puts Misty Rainforest onto the battlefield; T1 puts Island onto the battlefield; T1 cast Brainstorm; T2 cast Brainstorm; T7 puts Boseiju, Who Endures onto the battlefield; T8 cast Carpet of Flowers; T8 cast Show and Tell; T8 puts Atraxa, Grand Unifier onto the battlefield; T9 cast Stock Up; T9 puts Misty Rainforest onto the battlefield; T9 puts Savannah onto the battlefield; T9 cast Acererak the Archlich; T11 puts Hedge Maze onto the battlefield; T11 cast Stock Up
- match 7 game 2 (on the draw, 10 own turns, won): T2 puts Ancient Tomb onto the battlefield; T2 cast Force of Will; T5 cast Show and Tell; T6 puts Misty Rainforest onto the battlefield; T6 puts Hedge Maze onto the battlefield; T7 cast Show and Tell; T7 puts Atraxa, Grand Unifier onto the battlefield; T7 cast Carpet of Flowers; T7 cast Orim's Chant; T8 cast Carpet of Flowers; T8 cast Brainstorm; T9 cast Stock Up; T9 cast Carpet of Flowers; T10 cast Carpet of Flowers

**Lost after at least 5 own turns with a Stock Up stuck in hand**

- match 37 game 2 (on the draw, 6 own turns, lost): T2 puts Flooded Strand onto the battlefield; T2 puts Island onto the battlefield; T2 cast Brainstorm; T3 cast Stock Up; T3 puts Hedge Maze onto the battlefield; T4 cast Veil of Summer; T4 cast Ponder; T5 cast Stock Up; T5 puts Flooded Strand onto the battlefield; T5 puts Tropical Island onto the battlefield; T6 cast Ponder; T6 cast Stock Up; T6 puts City of Traitors onto the battlefield; T6 cast Show and Tell
- match 37 game 3 (on the play, 10 own turns, lost): T3 puts Savannah onto the battlefield; T4 cast Carpet of Flowers; T4 cast Ponder; T5 cast Stock Up; T5 cast Carpet of Flowers; T6 cast Carpet of Flowers; T7 cast Ponder; T7 cast Prismatic Ending; T8 cast Acererak the Archlich; T9 cast Veil of Summer; T9 cast Show and Tell; T9 puts Acererak the Archlich onto the battlefield; T10 cast Stock Up; T10 puts Hedge Maze onto the battlefield
- match 154 game 3 (on the play, 20 own turns, lost): T8 cast Ponder; T9 cast Show and Tell; T9 puts Atraxa, Grand Unifier onto the battlefield; T9 puts Tropical Island onto the battlefield; T9 cast Veil of Summer; T11 cast Brainstorm; T14 cast Force of Will; T15 cast Stock Up; T15 cast Show and Tell; T15 puts Atraxa, Grand Unifier onto the battlefield; T18 cast Stock Up; T19 cast Show and Tell; T19 puts Flooded Strand onto the battlefield; T20 puts Hedge Maze onto the battlefield

**Lost after at least 5 own turns without ever seeing a Stock Up**

- match 0 game 2 (on the play, 8 own turns, lost): T1 puts Hedge Maze onto the battlefield; T2 puts Tropical Island onto the battlefield; T2 cast Force of Will; T3 puts Misty Rainforest onto the battlefield; T3 puts Hedge Maze onto the battlefield; T5 puts Tundra onto the battlefield; T6 cast Brainstorm; T7 cast Show and Tell; T8 puts Savannah onto the battlefield; T8 cast Aluren
- match 24 game 2 (on the draw, 7 own turns, lost): T1 puts City of Traitors onto the battlefield; T6 puts Ancient Tomb onto the battlefield
- match 29 game 3 (on the play, 7 own turns, lost): T1 puts Island onto the battlefield; T2 cast Ponder; T2 puts Ancient Tomb onto the battlefield; T3 cast Brainstorm; T3 puts Savannah onto the battlefield; T3 cast Prismatic Ending; T4 cast Veil of Summer; T5 cast Veil of Summer; T6 cast Carpet of Flowers; T7 cast Acererak the Archlich; T7 puts Misty Rainforest onto the battlefield; T7 puts Hedge Maze onto the battlefield

### d3

**Won, with Lotus Petal used in the last two own turns**

none


**Won, with a Stock Up cast in the last three own turns**

- match 4 game 3 (on the play, 8 own turns, won): T8 puts Goblin Token onto the battlefield; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 cast Acererak the Archlich; T8 puts Goblin Token onto the battlefield; T8 cast Acererak the Archlich
- match 8 game 2 (on the play, 19 own turns, won): T2 puts Misty Rainforest onto the battlefield; T2 puts Tropical Island onto the battlefield; T9 cast Ponder; T15 puts Ancient Tomb onto the battlefield; T15 cast Show and Tell; T15 puts Atraxa, Grand Unifier onto the battlefield; T17 cast Show and Tell; T17 puts Acererak the Archlich onto the battlefield; T18 cast Show and Tell; T18 puts Acererak the Archlich onto the battlefield; T19 cast Stock Up; T19 cast Ponder; T19 puts Tundra onto the battlefield; T19 cast Ponder
- match 10 game 2 (on the draw, 11 own turns, won): T10 puts Ancient Tomb onto the battlefield; T10 cast Aluren; T10 cast Acererak the Archlich; T10 cast Acererak the Archlich; T10 puts Treasure Token onto the battlefield; T10 cast Acererak the Archlich; T10 cast Acererak the Archlich; T10 cast Acererak the Archlich; T10 cast Acererak the Archlich; T10 cast Acererak the Archlich; T10 puts The Atropal onto the battlefield; T10 cast Acererak the Archlich; T10 cast Acererak the Archlich; T11 puts Zombie Token onto the battlefield

**Lost after at least 5 own turns with a Stock Up stuck in hand**

- match 69 game 2 (on the play, 13 own turns, lost): T9 cast Acererak the Archlich; T9 cast Acererak the Archlich; T9 cast Acererak the Archlich; T9 puts The Atropal onto the battlefield; T9 cast Acererak the Archlich; T10 cast Show and Tell; T10 puts Ancient Tomb onto the battlefield; T10 puts Zombie Token onto the battlefield; T11 cast Veil of Summer; T11 puts Flooded Strand onto the battlefield; T11 puts Hedge Maze onto the battlefield; T13 puts Flooded Strand onto the battlefield; T13 puts Tundra onto the battlefield; T13 cast Show and Tell
- match 85 game 4 (on the play, 12 own turns, lost): T11 puts Ancient Tomb onto the battlefield; T11 cast Ponder; T11 cast Carpet of Flowers; T11 cast Veil of Summer; T11 cast Acererak the Archlich; T11 cast Stock Up; T11 cast Show and Tell; T11 puts Misty Rainforest onto the battlefield; T11 puts Tropical Island onto the battlefield; T11 cast Ponder; T11 cast Acererak the Archlich; T12 cast Show and Tell; T12 puts Misty Rainforest onto the battlefield; T12 cast Carpet of Flowers
- match 276 game 2 (on the draw, 10 own turns, lost): T8 cast Ponder; T8 cast Ponder; T8 cast Aluren; T8 puts Ancient Tomb onto the battlefield; T9 cast Veil of Summer; T9 cast Atraxa, Grand Unifier; T9 puts Misty Rainforest onto the battlefield; T9 puts Tundra onto the battlefield; T9 cast Brainstorm; T10 puts Hedge Maze onto the battlefield; T10 cast Atraxa, Grand Unifier; T10 cast Show and Tell; T10 puts Atraxa, Grand Unifier onto the battlefield; T10 cast Brainstorm

**Lost after at least 5 own turns without ever seeing a Stock Up**

- match 19 game 3 (on the play, 11 own turns, lost): T1 puts Flooded Strand onto the battlefield; T1 puts Tropical Island onto the battlefield; T2 puts Ancient Tomb onto the battlefield; T4 puts Ancient Tomb onto the battlefield; T5 puts Hedge Maze onto the battlefield; T6 puts City of Traitors onto the battlefield; T6 cast Show and Tell; T7 cast Veil of Summer; T10 puts Island onto the battlefield; T11 cast Brainstorm; T11 cast Force of Will
- match 24 game 3 (on the play, 7 own turns, lost): T2 cast Veil of Summer; T2 puts Ancient Tomb onto the battlefield; T3 puts Tundra onto the battlefield; T3 cast Carpet of Flowers; T3 cast Prismatic Ending; T4 cast Brainstorm; T4 puts Misty Rainforest onto the battlefield; T4 puts Hedge Maze onto the battlefield; T5 cast Show and Tell; T5 puts Carpet of Flowers onto the battlefield; T6 puts Savannah onto the battlefield; T6 cast Force of Will; T7 cast Show and Tell; T7 puts Flooded Strand onto the battlefield
- match 28 game 2 (on the play, 8 own turns, lost): T1 puts Ancient Tomb onto the battlefield; T2 puts Ancient Tomb onto the battlefield; T5 puts City of Traitors onto the battlefield; T7 puts Savannah onto the battlefield

