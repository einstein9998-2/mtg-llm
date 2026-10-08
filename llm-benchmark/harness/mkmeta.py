#!/usr/bin/env python3
"""mkmeta.py add GAME SEED A_SEAT FIRST A_DECK B_DECK | mkmeta.py results   (fills a_result/b_result from truth logs)"""
import json, sys, os, re
P = '/home/claude/work/llm2/games.json'
m = json.load(open(P)) if os.path.exists(P) else {}
if sys.argv[1] == 'add':
    g, seed, aseat, first, ad, bd = sys.argv[2:8]
    m[g] = {'seed': int(seed), 'a_seat': int(aseat), 'b_seat': 1 - int(aseat), 'first_seat': int(first), 'a_deck': ad, 'b_deck': bd}
else:
    for g, d in m.items():
        t = f'/home/claude/work/llm2/runs/{g}/truth/log.txt'
        if os.path.exists(t):
            last = open(t).read().strip().split('\n')[-1]
            if last.startswith('YOU WON'): d['a_result'], d['b_result'] = 'won', 'lost'
            elif last.startswith('YOU LOST'): d['a_result'], d['b_result'] = 'lost', 'won'
            elif last.startswith('DRAW'): d['a_result'] = d['b_result'] = 'draw'
json.dump(m, open(P, 'w'), indent=1)
