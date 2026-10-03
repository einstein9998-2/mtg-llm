# sample_opp_hands calibration

19 recorded games (Alurentell vs UR Cutter), 2051 decision states, one row per (state, card still unknown).

Predicted P(card is in the opponent's hand) against how often it really was (engine ground truth, used only as the answer key).

| predicted | n | mean predicted | actual |
|---|---|---|---|
| 0.0-0.1 | 12331 | 0.064 | 0.042 |
| 0.1-0.2 | 12674 | 0.151 | 0.170 |
| 0.2-0.3 | 8537 | 0.249 | 0.187 |
| 0.3-0.4 | 4619 | 0.345 | 0.332 |
| 0.4-0.5 | 1025 | 0.412 | 0.521 |
| 0.9-1.0 | 6 | 1.000 | 1.000 |

Log loss: tool 15799, naive same-p-for-every-card baseline 18482 (lower is better).

The sampler is uniform over hands consistent with the cards seen, so it is calibrated when the opponent plays cards independently of what they hold. The Forge AI plays cards it holds and keeps others, so deviations of a few points to ten points in either direction are expected; this is a sanity check on the accounting, not a model of the opponent.
