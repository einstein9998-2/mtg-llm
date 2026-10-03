#!/usr/bin/env python3
"""Join Rust positions (posgen) with Forge probe output and report action-set and injection-fidelity differences.
Usage: compare.py positions.jsonl forge.jsonl [out-prefix]"""
import json, sys, collections
pos = {}
for l in open(sys.argv[1]):
    if l.strip():
        o = json.loads(l); pos[o['id']] = o
fo = {}
for l in open(sys.argv[2]):
    if l.strip():
        o = json.loads(l); fo[o['id']] = o
prefix = sys.argv[3] if len(sys.argv) > 3 else None
# known-divergence registry (doc 04 section 9): kd-NNNN.toml files whose match block suppresses an action-set difference
import os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kdlib
kds = kdlib.load(sys.argv[4] if len(sys.argv) > 4 else None)
feature_tags = lambda p: kdlib.feature_tags(p['state'])
kd_for = lambda side, key, tags, active: kdlib.match(kds, side, key, tags, active)
suppressed = collections.Counter()
n = 0; same = 0; errs = []; inj = []; act = []; stack = 0
for i, p in pos.items():
    f = fo.get(i)
    if f is None: continue
    n += 1
    if 'error' in f: errs.append((i, f['error'])); continue
    if f['stack'] != 0: stack += 1; continue
    want_pl = 'P1' if p.get('priority', 0) == 0 else 'P2'
    if f.get('priority') and f['priority'] != want_pl or f.get('phase') != p['step']:
        errs.append((i, f"window mismatch: forge {f.get('priority')} in {f.get('phase')}, rust {want_pl} in {p['step']}")); continue
    # summoning sickness is compared through the action sets (haste makes Forge's flag differ from the raw fact)
    rs = [x.strip().replace('|SummonSick', '') for x in p['summary']]
    fs = [x.strip().replace('|SummonSick', '') for x in f['forge_summary']]
    # library size: Forge draws nothing at injection; compare everything
    if rs != fs:
        inj.append((i, rs, fs)); continue
    ra, fa = set(x.replace('playback|', 'play|') for x in p['rust_actions']), set(f['forge_actions'])
    # per-turn activation limits (loyalty abilities, once-per-turn abilities) depend on history Forge cannot be given
    act_names = set(p.get('activated', []))
    ra = set(k for k in ra if not (k.startswith('act|') and k.split('|')[2] in act_names))
    fa = set(k for k in fa if not (k.startswith('act|') and k.split('|')[2] in act_names))
    tags = feature_tags(p)
    ro, fo_l = [], []
    for k in sorted(ra - fa):
        kd = kd_for('RUST_ONLY', k, tags, p['priority'])
        if kd: suppressed[kd] += 1
        else: ro.append(k)
    for k in sorted(fa - ra):
        kd = kd_for('FORGE_ONLY', k, tags, p['priority'])
        if kd: suppressed[kd] += 1
        else: fo_l.append(k)
    if not ro and not fo_l: same += 1
    else: act.append((i, ro, fo_l))
print(f"positions {n}  identical-action-sets {same}  action-diffs {len(act)}  injection-mismatch {len(inj)}  forge-nonempty-stack {stack}  forge-errors {len(errs)}")
cnt = collections.Counter()
for i, ro, fo_ in act:
    for k in ro: cnt[('RUST_ONLY', k.split('|')[0] + '|' + k.split('|')[2] if k.count('|') >= 2 else k)] += 1
    for k in fo_: cnt[('FORGE_ONLY', k.split('|')[0] + '|' + k.split('|')[2] if k.count('|') >= 2 else k)] += 1
print("suppressed by known divergences:", dict(suppressed))
print("\naction differences by (side, kind|card):")
for (s, k), c in cnt.most_common(60): print(f"  {c:4d}  {s:10s} {k}")
ic = collections.Counter()
for i, rs, fs in inj:
    for a, b in zip(rs, fs):
        if a != b:
            sa, sb = set(a.split(' ')), set(b.split(' '))
            ic[(tuple(sorted(sa - sb))[:2], tuple(sorted(sb - sa))[:2])] += 1
print("\ninjection mismatches (first 15 distinct):")
for k, c in ic.most_common(15): print(f"  {c:4d} {k}")
print("\nforge errors:")
for k, c in collections.Counter(e for _, e in errs).most_common(10): print(f"  {c:4d} {k}")
if prefix:
    json.dump({'actions': act, 'inject': [(i, a, b) for i, a, b in inj], 'errors': errs}, open(prefix + '.diffs.json', 'w'), indent=1)
