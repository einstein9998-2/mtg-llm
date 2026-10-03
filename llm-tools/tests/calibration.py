#!/usr/bin/env python3
"""Grade sample_opp_hands' marginals against the engine's true opponent hands over all recorded games (answer key only, never given to the tool).
usage: python3 tests/calibration.py > results/calibration.md"""
import collections, os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import recorded as R
from mtgtools import calc, decks

opp = decks.load("ur-cutter")["main"]; bins = collections.defaultdict(lambda: [0, 0.0, 0]); states = games = 0
tot_p = tot_a = 0.0; ll = base_ll = 0.0
from math import log
for run in R.runs():
    games += 1
    for r, v, truth in R.replay(run):
        if v["stale"] or v["opp_lib_n"] is None or v["opp_hand_n"] == 0: continue
        states += 1
        _, marg, info, _ = calc.sample_hands(v, opp, 0)
        for c, x in marg.items():
            p = min(max(x["p_in_hand"], 1e-6), 1 - 1e-6); a = truth.get(c, 0) > 0
            q = min(max(info["hand_size"] / max(info["unknown_pool_size"] + len(v["opp_hand_known"]), 1), 1e-6), 1 - 1e-6)   # naive baseline: same p for every card
            b = min(int(p * 10), 9); bins[b][0] += 1; bins[b][1] += p; bins[b][2] += a
            ll += -(log(p) if a else log(1 - p)); base_ll += -(log(q) if a else log(1 - q))
print(f"# sample_opp_hands calibration\n\n{games} recorded games (Alurentell vs UR Cutter), {states} decision states, one row per (state, card still unknown).\n")
print("Predicted P(card is in the opponent's hand) against how often it really was (engine ground truth, used only as the answer key).\n")
print("| predicted | n | mean predicted | actual |\n|---|---|---|---|")
for b in sorted(bins):
    c, sp, a = bins[b]; print(f"| {b / 10:.1f}-{(b + 1) / 10:.1f} | {c} | {sp / c:.3f} | {a / c:.3f} |")
print(f"\nLog loss: tool {ll:.0f}, naive same-p-for-every-card baseline {base_ll:.0f} (lower is better).")
print("\nThe sampler is uniform over hands consistent with the cards seen, so it is calibrated when the opponent plays cards independently of what they hold. The Forge AI plays cards it holds and keeps others, so deviations of a few points to ten points in either direction are expected; this is a sanity check on the accounting, not a model of the opponent.")
