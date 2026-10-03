#!/usr/bin/env python3
"""Action-set differential at stack-response windows.
Usage: resp_actions.py positions.jsonl step-diff.jsonl [known-divergences dir]
Reads the `resp_rust` / `resp_forge` key sets stepdiff recorded at the opponent's first window with the position's action on the stack."""
import json, sys, collections, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kdlib

pos = {}
for l in open(sys.argv[1]):
    if l.strip():
        o = json.loads(l); pos[o['id']] = o
kds = kdlib.load(sys.argv[3] if len(sys.argv) > 3 else None)
n = same = 0
diffs = []
suppressed = collections.Counter()
for l in open(sys.argv[2]):
    if not l.strip(): continue
    r = json.loads(l)
    if not r.get('resp_rust') or not r.get('resp_forge'): continue
    p = pos[r['id']]
    n += 1
    ra = set(k.replace('playback|', 'play|') for k in r['resp_rust'])
    fa = set(r['resp_forge'])
    act_names = set(p.get('activated', []))
    ra = set(k for k in ra if not (k.startswith('act|') and k.split('|')[2] in act_names))
    fa = set(k for k in fa if not (k.startswith('act|') and k.split('|')[2] in act_names))
    tags = kdlib.feature_tags(p['state'])
    actor = 1 - p['priority']  # the responder is the other seat
    ro, fo = [], []
    for k in sorted(ra - fa):
        kd = kdlib.match(kds, 'RUST_ONLY', k, tags, actor)
        if kd: suppressed[kd] += 1
        else: ro.append(k)
    for k in sorted(fa - ra):
        kd = kdlib.match(kds, 'FORGE_ONLY', k, tags, actor)
        if kd: suppressed[kd] += 1
        else: fo.append(k)
    if not ro and not fo: same += 1
    else: diffs.append((r['id'], p['action'], ro, fo))
print(f"response windows {n}  identical-action-sets {same}  action-diffs {len(diffs)}")
print("suppressed by known divergences:", dict(suppressed))
cnt = collections.Counter()
for i, a, ro, fo in diffs:
    for k in ro: cnt[('RUST_ONLY', k)] += 1
    for k in fo: cnt[('FORGE_ONLY', k)] += 1
for (s, k), c in cnt.most_common(40): print(f"  {c:4d} {s:10s} {k}")
for i, a, ro, fo in diffs[:40]: print('  ', i, '|', a, '| rust-only', ro, '| forge-only', fo)
