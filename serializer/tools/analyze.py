#!/usr/bin/env python3
"""Prompt-size table from decisions.jsonl (needs --log-prompts true for text; chars are always logged).
Tokens are ESTIMATED as chars/3.6 (no tokenizer offline); treat as +-25%.
Usage: analyze.py <run dir> ..."""
import json, sys, statistics as st
def tok(c): return c / 3.6
for d in sys.argv[1:]:
    rows = [json.loads(l) for l in open(f"{d}/decisions.jsonl")]
    by = {}
    for r in rows: by.setdefault(r["kind"], []).append(r["chars"])
    allc = [r["chars"] for r in rows]
    games = len({r["g"] for r in rows})
    print(f"== {d}: {len(rows)} prompts in {games} games ({len(rows)/max(games,1):.1f}/game), est tokens total {tok(sum(allc)):.0f}")
    print(f"{'kind':14}{'n':>6}{'p50 tok':>9}{'p90 tok':>9}{'max tok':>9}{'tok/game':>10}")
    for k, v in sorted(by.items(), key=lambda kv: -sum(kv[1])):
        v = sorted(v)
        print(f"{k:14}{len(v):>6}{tok(v[len(v)//2]):>9.0f}{tok(v[int(len(v)*0.9)]):>9.0f}{tok(v[-1]):>9.0f}{tok(sum(v))/max(games,1):>10.0f}")
    pr = sorted(c for k, v in by.items() if k == "Priority" for c in v)
    if pr: print(f"Priority prompts with full state (turn start) are the large ones; p90 {tok(pr[int(len(pr)*0.9)]):.0f} tok, max {tok(pr[-1]):.0f}")
