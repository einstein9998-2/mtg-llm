# How much Lotus Petals help Orim's Chant find white mana post-board (2026-10-14)

Brady's follow-up to REPORT-chant.md: the Chant games kept running into "no white source", so how much does keeping Lotus Petals in post-board improve white access? This is arithmetic, not a simulation: `white_access.py` (exact hypergeometric over the 60 cards, no engine). It is a ceiling on access, because it ignores everything that competes for the same mana.

## Short answer

- **Petals matter a lot once Wasteland is in the picture, little otherwise.** With no Wasteland, going from 0 to 4 Petals lifts "a white source by turn 3" from about 89% to 96%, which is small because Savannah and Tundra plus seven fetches already find white most of the time. With one Wasteland on a land, the same change lifts it from 58% to 85%, and makes kicked Chant (WW) possible at all: 0% with no Petals, 42% with four, because there is only one reachable white land left to count.
- **Each Petal is worth roughly +2 points of "white by turn 3" with no Wasteland and +7 with one**, and about +5 and +10 points of "WW by turn 3" respectively (rounded) (3 Chant, cantrips counted).
- **The games sit closer to the Wasteland column than the no-Wasteland one.** The no-Wasteland table says Chant should be dead roughly 5 to 25% of the time it is in hand. In the 12 LLM games it was uncastable or badly timed in at least 7. The model ignores Wasteland, Stifle on fetches, Savannah and Tundra being needed for Atraxa and Aluren colors, and fetches being cracked for other colors, so it overstates access; the truth is between the two tables.
- **Price:** the plans are 10 cards out and 10 in, so each Petal kept is one fewer sideboard card in (Carpet, Chant, Veil or Ending). That tradeoff is not priced here.

## Model

60 cards: Savannah and Tundra (2 white lands), 3 Flooded Strand and 4 Misty Rainforest (7 fetches that can find either), `p` Lotus Petal, `c` Orim's Chant, the rest other. Cards seen by turn T on the play = 7 + (T − 1), plus 3 for cantrips (Ponder, Brainstorm, Stock Up) in the tables below. White sources = white lands drawn + fetches that still have a white land to find (capped by one land drop per turn) + Petals (one-shot, usable at instant speed, so a Petal can pay for a kicked Chant on the opponent's turn). The Wasteland table removes one land source. Carpet of Flowers is not counted: it makes white only in your own main phase and only with opposing Islands. "Chant live" means at least one Chant in the cards seen and the stated white available.


**No Wasteland on a land** (3 cards from cantrips by the turn shown, on the play, % of games)

| Chant | Petals | W, T3 | WW, T3 | Chant live (W), T3 | Chant live (WW), T3 | W, T5 | WW, T5 | Chant live (W), T5 | Chant live (WW), T5 |
|---|---|---|---|---|---|---|---|---|---|
| 3 | 0 | 89 | 58 | 43 | 27 | 93 | 68 | 51 | 36 |
| 3 | 1 | 91 | 65 | 45 | 30 | 95 | 74 | 52 | 40 |
| 3 | 2 | 93 | 71 | 46 | 33 | 96 | 79 | 53 | 43 |
| 3 | 4 | 96 | 80 | 47 | 38 | 98 | 87 | 54 | 48 |
| 4 | 0 | 89 | 58 | 52 | 33 | 93 | 68 | 61 | 44 |
| 4 | 1 | 91 | 65 | 54 | 37 | 95 | 74 | 63 | 48 |
| 4 | 2 | 93 | 71 | 56 | 41 | 96 | 79 | 64 | 52 |
| 4 | 4 | 96 | 80 | 57 | 47 | 98 | 87 | 65 | 57 |

**One Wasteland on a land** (3 cards from cantrips by the turn shown, on the play, % of games)

| Chant | Petals | W, T3 | WW, T3 | Chant live (W), T3 | Chant live (WW), T3 | W, T5 | WW, T5 | Chant live (W), T5 | Chant live (WW), T5 |
|---|---|---|---|---|---|---|---|---|---|
| 3 | 0 | 58 | 0 | 27 | 0 | 68 | 0 | 36 | 0 |
| 3 | 1 | 67 | 11 | 32 | 5 | 76 | 15 | 41 | 8 |
| 3 | 2 | 75 | 22 | 35 | 9 | 82 | 29 | 45 | 15 |
| 3 | 4 | 85 | 42 | 41 | 19 | 91 | 52 | 50 | 28 |
| 4 | 0 | 58 | 0 | 33 | 0 | 68 | 0 | 44 | 0 |
| 4 | 1 | 67 | 11 | 39 | 6 | 76 | 15 | 49 | 9 |
| 4 | 2 | 75 | 22 | 43 | 12 | 82 | 29 | 54 | 18 |
| 4 | 4 | 85 | 42 | 50 | 23 | 91 | 52 | 60 | 33 |

## Reading the tables

- **Chant live (W)** is unkicked Chant on your own turn (the use that worked in the games: clear the way for Show and Tell or Aluren). 3 Chant, 2 Petals, one Wasteland: 35% by turn 3 vs 27% with none. 4 Chant, 4 Petals: 50% vs 33%.
- **Chant live (WW)** is kicked Chant, the fog. With no Petals it never happens once a Wasteland hits, and even with no Wasteland it is live in only about a third to under half of games by turn 5 (36% for 3 Chant, 44% for 4). The games never cast one.
- The cost of dead Chant in hand (Chant seen but no white), turn 3, 3 Chant, cantrips: no Wasteland 13% of Chant hands with 0 Petals, 5% with 4; with one Wasteland 45% with 0 Petals, 17% with 4.
- 3 Chant vs 4 Chant: the fourth copy adds about 10 points of "Chant in hand" at turn 3 but none to white access, so it is the white sources, not the count, that cap it.

## Limits

- Counts cards seen, not mulligans, Daze, Force of Will or whether the white source is needed for something else that turn.
- The Wasteland row takes away exactly one land source on every game; real Wasteland rates are lower (the opponent needs one in hand and the right moment) and Stifle on a fetch is not modeled.
- The "+3 cards from cantrips" is a flat guess; the no-cantrip rows (`python3 white_access.py`) are 10 to 17 points lower at turn 3.
