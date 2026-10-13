# Alurentell deck construction, round 2 (2026-10-13)

Follow-up to [REPORT.md](REPORT.md) after Brady's feedback: Petal/Force should be tested against slower cards, keep testing Chant and Ending, keep testing land cuts, explain "FoN barely matters", and try Emrakul in place of any nonland card for game 1.

Method is the same as round 1: net-vs-net single games, one post-board game per cell, both sides boarded (game 1 for the Emrakul swaps), 350 to 400 games per side per cell, play and draw reported pooled. "vs basetr" is the paired win-rate difference against an unchanged copy of Brady's plan run on the same shuffles (95% interval). Differences under about 4 points are noise. The nets were trained on the main deck only, so cards that were sideboarded in the training data are undervalued; the trace metrics (line turn, T4 mana, life at the first line) are there to cross-check the win rates.

Full tables: `tables-round2-A.md` (Petal/Force), `-B.md` (Chant/Ending), `-C.md` (lands), `-E.md` (Emrakul game 1), `tables-counters.md`, `tables-chant-ending-fate.md`.

## Short version

1. **Petal over Stock Up speeds the combo a little but doesn't win more.** One or two Petals over Stock Up moves the UR line by T4 from 42% to 44 to 46%, the mean first-line turn from 4.59 to 4.39, and T4 mana from 3.5 to 3.9, yet the win rate change vs UR, UW and UB is −3.5 to +3.0 (noise). Only UB looks positive (+3.0 / +2.6).
2. **Don't cut Aluren for speed cards.** Aluren is a second combo, not a slow card: two Aluren out for Petal costs 16 points vs UW, and out for Force 17 (UW) and 10 (UR). Stock Up is the right cut.
3. **Extra Force of Will over Stock Up:** one is a wash, two is worse (UR −5.6, UW −6.1).
4. **Chant and Ending:** against UB, replacing Chant with Petal is +6 points (the only clear change). A third copy of either is no better (UR −3, UW −4 to −5). Chant is drawn in 52 to 64% of games vs the tempo decks and Ending in a similar share. Each is uncastable (no white source) in about a fifth of those draws, where you win 15 to 27%.
5. **Lands:** cutting one land (Island or a Hedge Maze) is free everywhere. Cutting two hurts Tron (−4.1 for two Hedge Mazes).
6. **Emrakul for any one nonland card in game 1 is a small gain** (average +2.5 over 11 swaps and five matchups), best over Ponder (+4.6), Veil (+4.0), Force of Will (+3.6), Stock Up (+2.8); worst over Show and Tell (+0.2) and Aluren (+0.7).
7. **FoN:** the net casts it but mostly spends it on cheap pieces, see below.

## A. Petal and Force of Will over slower cards (vs UR, UW, UB)

Config names: pk = Petal over Stock Up (1 or 2 copies), pu = Petal over Aluren, fk = Force over Stock Up, fu = Force over Aluren, pfk = one Petal and one Force over two Stock Up, pfm = one Petal and one Force over one Stock Up and one Aluren.

| Change | UR | UW | UB |
|---|---|---|---|
| +1 Petal −1 Stock Up | −2.5 | −1.0 | +3.0 |
| +2 Petal −2 Stock Up | −3.5 | −2.9 | +2.6 |
| +1 Petal −1 Aluren | −5.1 | −6.1 | +0.1 |
| +2 Petal −2 Aluren | −5.8 | −15.8 | +1.0 |
| +1 Force −1 Stock Up | −1.1 | −1.5 | +1.8 |
| +2 Force −2 Stock Up | −5.6 | −6.1 | −1.4 |
| +1 Force −1 Aluren | −4.4 | −6.9 | +1.1 |
| +2 Force −2 Aluren | −9.9 | −17.2 | −0.6 |
| +1 Petal +1 Force −2 Stock Up | −5.0 | −3.4 | +0.6 |
| +1 Petal +1 Force −1 Stock Up −1 Aluren | −4.8 | −5.0 | +0.5 |

(Win-rate change in points vs the unchanged plan; each ±3.4 to ±4.5 at 95%. UW and UR post-board counts: Force 2, Aluren 2 (UW) and 3 (UR), Stock Up 4, Petal 0. UB: Force 0, Aluren 4.)

Trace metrics for the Petal swaps (UR): line by T4 42% → 44% / 46%, mean first-line turn 4.59 → 4.44 / 4.39, T4 mana 3.49 → 3.74 / 3.94, life at the first line 15.5 → 15.9 / 16.0. So each turn of speed is worth only a few tenths of a life point in this sim, and the nets don't turn that into wins.

## B. Chant and Ending counts

Config names: ch/en 1 or 0 = number left of 2, p = replaced with Petal, f = with Force of Will; ch3k / en3k = a third copy over Stock Up (sideboard has the Macabre slot as the spare). Storm and mirror: ch1/ch0 replaced with Stock Up/Atraxa/Acererak and Petal/Show and Tell.

| Change | UR | UW | UB |
|---|---|---|---|
| Chant −1, +1 Petal | −1.4 | +2.3 | +6.2 |
| Chant −2, +2 Petal | +0.6 | +0.5 | +6.1 |
| Chant −2, +2 Force | −4.1 | −2.0 | +2.2 |
| Chant +1 (3 total) over Stock Up | −2.9 | −5.0 | −1.0 |
| Ending −1, +1 Petal | +0.5 | +2.4 | +2.6 |
| Ending −2, +2 Petal | +1.9 | +0.8 | +1.6 |
| Ending −2, +2 Force | −6.2 | −1.0 | −1.2 |
| Ending +1 (3 total) over Stock Up | −3.2 | −4.0 | +0.5 |

Storm: Chant −1 +3.1, Chant −2 +2.5. Mirror: Chant −1 +0.1, −2 +1.8. All noise.

What happens to the cards once drawn (basetr, 800 games per matchup; win rate in each bucket):

| Card, matchup | cast | held with a white source and mana | no white source | drawn on the last turns | never drawn |
|---|---|---|---|---|---|
| Chant vs UR | 172 games, 55% | 109, 70% | 99, 27% | 51, 59% | 369, 46% |
| Chant vs UW | 331, 43% | 102, 36% | 50, 16% | 27, 59% | 290, 39% |
| Chant vs UB | 202, 44% | 83, 42% | 88, 25% | 45, 40% | 382, 39% |
| Ending vs UR | 174, 52% | 120, 68% | 103, 27% | 52, 54% | 351, 48% |
| Ending vs UB | 135, 49% | 137, 47% | 96, 17% | 50, 50% | 382, 37% |

The "no white source" bucket is games where no Tundra, Savannah, Petal or Carpet was ever available after the draw; it is partly a bad-draw signal as well as a dead-card signal.

## C. Land trims (one or two lands out, freed slot filled with a slow card)

Win-rate change in points vs the unchanged plan, 350 games per side (±4.2 to ±5.2).

| Cut | UR | UW | UB | Mirror | Storm | Tron |
|---|---|---|---|---|---|---|
| Island | −1.3 | −0.1 | −2.7 | +1.4 | n/a | n/a |
| 1 Hedge Maze | −0.9 | +3.1 | −0.3 | +2.4 | +0.9 | +1.7 |
| 1 Flooded Strand | −3.7 | +3.3 | −3.4 | 0.0 | −0.6 | +2.6 |
| Island + Hedge Maze | −0.9 | −1.1 | −2.6 | −0.1 | n/a | n/a |
| 2 Hedge Maze | +0.3 | −1.3 | −1.4 | 0.0 | −2.3 | −4.1 |
| Hedge Maze + Strand | n/a | n/a | n/a | n/a | +0.4 | +0.6 |
| 2 Strand | n/a | n/a | n/a | n/a | −0.1 | +0.1 |

Replacements: Aluren then Petal (UR, UW); Petal, Force, Omniscience (UB); Show and Tell, Petal (mirror); Stock Up, Atraxa, Acererak (Storm); Omniscience, Veil (Tron). Island is already out of the Storm, Tron and Reanimator boards.

## D. What "FoN barely matters" meant

It meant "removing it changes the sim win rate by less than the noise" (Storm +2.9 ±3.6, Tron −0.5 ±3.9), not that it is a bad card. `sbtrace` now records every counterspell: which card countered which. Across 600 games per matchup (base plan):

| Matchup | FoN drawn | cast | countered something | main hits | win cast / never drawn |
|---|---|---|---|---|---|
| Tron | 239 | 110 | 98 | Trinisphere 36, Ugin 11, Kozilek's Command 10 | 45% / 45% |
| Storm | 201 | 106 | 66 | Chrome Mox 21, Mox Opal 15, Petal 13 | 56% / 59% |
| Mirror | 415 | 91 | 74 | Brainstorm 21, Veil 17, Carpet 12 | 53% / 49% |
| Reanimator (rollout) | 295 | 92 | 68 | Animate Dead 13, Dark Ritual 9, Shallow Grave 9 | 42% / 38% |

Force of Will hits mostly the same kind of cheap permanents. Consign to Memory only shows spell hits (it counters abilities silently, which the tracker doesn't see), e.g. 35 Storm hits and 45 Tron hits. `tables-counters.md` has all of it, including what the opponents counter of ours.

## E. Emrakul swapped in for one nonland card, game 1 (unboarded, 400 per side)

Win-rate change in points vs the unchanged 60 (`emrg1-base`), in the order UR, UW, UB, Tron, Storm, then the average.

| Card out | UR | UW | UB | Tron | Storm | Avg |
|---|---|---|---|---|---|---|
| Ponder | +2.5 | +6.6 | +3.8 | +4.0 | +6.2 | **+4.6** |
| Veil of Summer | +3.4 | +7.1 | +4.0 | +2.0 | +3.5 | **+4.0** |
| Force of Will | −0.5 | +8.5 | +3.0 | +2.0 | +5.1 | **+3.6** |
| Stock Up | −1.8 | +5.1 | +1.8 | +5.4 | +3.4 | +2.8 |
| Omniscience | −0.1 | +5.0 | +1.0 | +2.6 | +4.6 | +2.6 |
| Brainstorm | −0.6 | +4.2 | +2.2 | +4.2 | +2.9 | +2.6 |
| Acererak | −1.2 | +4.6 | +2.8 | −0.3 | +4.8 | +2.1 |
| Atraxa | −1.0 | +3.8 | +3.4 | +2.1 | +2.4 | +2.1 |
| Lotus Petal | −1.9 | +1.2 | +1.9 | +3.0 | +6.0 | +2.0 |
| Aluren | −1.5 | +2.6 | +1.1 | −0.5 | +1.6 | +0.7 |
| Show and Tell | −4.5 | +2.0 | −1.2 | +3.2 | +1.4 | +0.2 |

Each cell is ±4.0 to 4.7 at 95%; the five-matchup average is about ±3.7. A second plain-main run (batch 2) differed from the first by 0.8 to 2.5 points per matchup, which is the size of base noise. Emrakul is hard cast or put in with Show and Tell in only about 5 to 15% of the games where it is drawn (UR 19/260, UW 45/303, UB 13/242, Tron 52/318, Storm 34/279) but then wins 87 to 100%; when it stays in hand the win rate is the same as the baseline. The line-turn column in `tables-round2-E.md` counts Emrakul as a payoff, so it is biased towards the Emrakul decks; the win rates are not.

Not done: Boros and Reanimator (rollout opponent, about 10 minutes per cell), the mirror (the engine reads one deck file for both seats), Emrakul as a 61st card in game 1, and any LLM games for these swaps.

## Reproducing

`gen3.py` writes the plans and decks and the queue files; `run1.sh` and `queue.sh` run them (configs named `emrg1-*` run unboarded with the Emrakul build); `round2.py <A|B|C|E>` prints the tables above; `counterstats.py` prints the counterspell hits. `sbtrace.rs` now records counterspell hits (`counters`) and life totals.
