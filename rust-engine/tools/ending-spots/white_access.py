#!/usr/bin/env python3
"""How often Alurentell has white mana for Orim's Chant post-board, by Lotus Petal count.

Exact hypergeometric over the 60-card deck: 2 white lands (Savannah, Tundra), 7 fetches that can
find them (3 Flooded Strand, 4 Misty Rainforest), p Lotus Petal, c Orim's Chant, rest other.
White sources a hand can make = white lands drawn + fetches that still have a white land to
find (capped by land drops available) + Petals. A Wasteland variant removes one land source.
"""
from math import comb
import sys

def table(c, p, t, extra, wasteland):
    """P over hands of n cards: returns (P(chant & W>=1), P(chant & W>=2), P(W>=1), P(W>=2))"""
    n = 7 + (t - 1) + extra            # on the play
    other = 60 - 2 - 7 - p - c
    tot = comb(60, n)
    a = b = w1 = w2 = 0.0
    for w in range(3):
        for f in range(8):
            for k in range(p + 1):
                for ch in range(c + 1):
                    r = n - w - f - k - ch
                    if r < 0 or r > other:
                        continue
                    pr = comb(2, w) * comb(7, f) * comb(p, k) * comb(c, ch) * comb(other, r) / tot
                    lands = min(w + min(f, 2 - w), t)
                    if wasteland:
                        lands = max(lands - 1, 0)
                    s = lands + k
                    if s >= 1: w1 += pr
                    if s >= 2: w2 += pr
                    if ch >= 1 and s >= 1: a += pr
                    if ch >= 1 and s >= 2: b += pr
    return a, b, w1, w2

def main():
    out = []
    for c in (3, 4):
        for wl, label in ((False, "no Wasteland"), (True, "after one Wasteland on a land")):
            for extra, elabel in ((0, "no cantrips"), (3, "+3 cards from cantrips")):
                out.append(f"\n### {c} Chant, {label}, {elabel} (on the play, % of games)\n")
                out.append("| Petals kept | metric | T2 | T3 | T4 | T5 | T6 |")
                out.append("|---|---|---|---|---|---|---|")
                for p in (0, 1, 2, 4):
                    rows = [table(c, p, t, extra, wl) for t in (2, 3, 4, 5, 6)]
                    for i, m in enumerate(("W available", "WW available", "Chant in hand and W", "Chant in hand and WW")):
                        idx = (2, 3, 0, 1)[i]
                        out.append(f"| {p} | {m} | " + " | ".join(f"{100*r[idx]:.1f}" for r in rows) + " |")
    print("\n".join(out))

main()
