#!/usr/bin/env python3
"""Summarise sideboard-variant runs: win rates with error bars, card-level evidence, example games.

Usage: sbvariants.py <dir with one sub-directory per variant, each holding features.jsonl (sbtrace output)>
       [--base base] [--examples 3] [--min-aturns 5]
Seat 0 is deck A (Alurentell). `winner` is 0 for A. Game 1 is never boarded and is identical in every
variant (same seeds), so only games 2+ and whole matches differ.
"""
import json, math, os, sys
from collections import defaultdict

def load(path):
    return [json.loads(l) for l in open(path) if l.strip()]

def wilson(w, n, z=1.96):
    if n == 0:
        return (0.0, 0.0, 0.0)
    p = w / n
    d = 1 + z * z / n
    c = (p + z * z / (2 * n)) / d
    h = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / d
    return (p, c - h, c + h)

def pct(w, n):
    p, lo, hi = wilson(w, n)
    return f"{100*p:.1f}% [{100*lo:.1f}, {100*hi:.1f}] ({w}/{n})"

def matches(games):
    by = defaultdict(list)
    for g in games:
        by[g["match"]].append(g)
    wins = [0, 0, 0]  # A, B, undecided
    for m, gs in by.items():
        gs.sort(key=lambda g: g["game"])
        a = sum(1 for g in gs if g["winner"] == 0)
        b = sum(1 for g in gs if g["winner"] == 1)
        wins[0 if a >= 2 else 1 if b >= 2 else 2] += 1
    return wins

def a_won(g):
    return g["winner"] == 0

def seen_by(g, card, turn):
    return any(t <= turn for t in g["drawn"].get(card, []))

def main():
    root = sys.argv[1]
    flag = lambda n, d: sys.argv[sys.argv.index(n) + 1] if n in sys.argv else d
    base = flag("--base", "base")
    nex = int(flag("--examples", "3"))
    min_at = int(flag("--min-aturns", "5"))
    variants = sorted(d for d in os.listdir(root) if os.path.exists(os.path.join(root, d, "features.jsonl")))
    data = {v: load(os.path.join(root, v, "features.jsonl")) for v in variants}
    out = []
    out.append("## Win rates (95% Wilson intervals)\n")
    out.append("| Variant | Matches won by Alurentell | Games 2+ | Games 2+ on the play | Games 2+ on the draw |")
    out.append("|---|---|---|---|---|")
    summary = {}
    for v in variants:
        gs = data[v]
        w = matches(gs)
        post = [g for g in gs if g["game"] >= 2]
        onp = [g for g in post if g["first"] == 0]
        ond = [g for g in post if g["first"] == 1]
        summary[v] = (w, post)
        out.append(f"| {v} | {pct(w[0], w[0]+w[1]+w[2])} | {pct(sum(map(a_won, post)), len(post))} | {pct(sum(map(a_won, onp)), len(onp))} | {pct(sum(map(a_won, ond)), len(ond))} |")
    if base in summary:
        out.append(f"\n## Difference from `{base}` (games 2+, normal approximation; matches in brackets)\n")
        out.append("| Variant | Games 2+ difference | z | Matches difference |")
        out.append("|---|---|---|---|")
        bw, bpost = summary[base]
        bn, bk = len(bpost), sum(map(a_won, bpost))
        bm = bw[0] + bw[1] + bw[2]
        for v in variants:
            if v == base:
                continue
            w, post = summary[v]
            n, k = len(post), sum(map(a_won, post))
            p1, p2 = bk / bn, k / n
            se = math.sqrt(p1 * (1 - p1) / bn + p2 * (1 - p2) / n)
            m = w[0] + w[1] + w[2]
            q1, q2 = bw[0] / bm, w[0] / m
            sem = math.sqrt(q1 * (1 - q1) / bm + q2 * (1 - q2) / m)
            out.append(f"| {v} | {100*(p2-p1):+.1f} points (SE {100*se:.1f}) | {(p2-p1)/se:+.1f} | {100*(q2-q1):+.1f} points (SE {100*sem:.1f}) |")
    out.append("\n## Card evidence (games 2+; observational, not causal)\n")
    out.append("Win rate of Alurentell when the card was seen (in the opening hand or drawn) by its own turn 4 versus not; the card has to be in that variant's deck to be seen.\n")
    out.append("| Variant | Card | Seen by turn 4 | Not seen by turn 4 | Games where cast | Won, cast within the last 2 own turns | Lost with it stuck in hand |")
    out.append("|---|---|---|---|---|---|---|")
    for v in variants:
        post = [g for g in data[v] if g["game"] >= 2]
        for card, usekey in (("Lotus Petal", "used"), ("Stock Up", "cast"), ("Acererak the Archlich", "cast")):
            s = [g for g in post if seen_by(g, card, 4)]
            ns = [g for g in post if not seen_by(g, card, 4)]
            if not any(card in g["drawn"] for g in post):
                continue
            used = [g for g in post if g[usekey].get(card)]
            late = [g for g in post if a_won(g) and any(t >= g["a_turns"] - 1 for t in g[usekey].get(card, []))]
            stuck = [g for g in post if not a_won(g) and g["hand_end"].get(card)]
            out.append(f"| {v} | {card} | {pct(sum(map(a_won, s)), len(s))} | {pct(sum(map(a_won, ns)), len(ns))} | {len(used)} | {len(late)} | {len(stuck)} |")
    out.append("\n## Example games\n")
    out.append("Chosen by rule, first matches in file order (match, game). `T<n>` is Alurentell's own turn number.\n")
    def show(title, picks):
        out.append(f"**{title}**\n")
        if not picks:
            out.append("none\n")
        for g in picks[:nex]:
            tl = "; ".join(g["timeline"][-14:])
            out.append(f"- match {g['match']} game {g['game']} ({'on the play' if g['first']==0 else 'on the draw'}, {g['a_turns']} own turns, {'won' if a_won(g) else 'lost'}): {tl}")
        out.append("")
    for v in variants:
        post = [g for g in data[v] if g["game"] >= 2]
        out.append(f"### {v}\n")
        show("Won, with Lotus Petal used in the last two own turns", [g for g in post if a_won(g) and any(t >= g["a_turns"] - 1 for t in g["used"].get("Lotus Petal", []))])
        show("Won, with a Stock Up cast in the last three own turns", [g for g in post if a_won(g) and any(t >= g["a_turns"] - 2 for t in g["cast"].get("Stock Up", []))])
        show(f"Lost after at least {min_at} own turns with a Stock Up stuck in hand", [g for g in post if not a_won(g) and g["a_turns"] >= min_at and g["hand_end"].get("Stock Up")])
        show(f"Lost after at least {min_at} own turns without ever seeing a Stock Up", [g for g in post if not a_won(g) and g["a_turns"] >= min_at and "Stock Up" not in g["drawn"]])
    print("\n".join(out))

main()
