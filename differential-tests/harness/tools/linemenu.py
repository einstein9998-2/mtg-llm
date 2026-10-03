#!/usr/bin/env python3
"""linemenu.py <out.jsonl> <known-divergences dir> [--all]: menu (action-set) differences found at lockstep windows, with known-divergence suppression.
A window's tags come from its Forge summary (battlefields). Prints the unexplained differences."""
import json, sys, collections, os
sys.path.insert(0, os.path.dirname(__file__))
import kdlib

def tags_of(summary):
    bf = []
    life = []
    for s in summary:
        who = 'human' if s.startswith('human') else 'ai'
        i = s.find(' bf='); j = s.find(' gy=')
        names = [t.split('|')[0].replace('^L', '').replace('^T', '') for t in s[i + 4:j].split(';') if t]
        bf.append(f'{who}battlefield=' + ';'.join(names))
        life.append(f"{who}life=" + s.split(' life=')[1].split(' ')[0])
    return kdlib.feature_tags(bf + life)

rows = [json.loads(l) for l in open(sys.argv[1])]
# loyalty and once-per-turn abilities used earlier in the turn (history Forge cannot be given): Forge lists them again
acts = {}
pp = os.path.join(os.path.dirname(os.path.abspath(sys.argv[1])), 'pos.jsonl')
if os.path.exists(pp):
    for l in open(pp):
        q = json.loads(l)
        if q.get('activated'): acts[q['id']] = set(q['activated'])
kds = kdlib.load(sys.argv[2])
total = 0; suppressed = collections.Counter(); unexplained = collections.Counter(); ex = {}
windows_with = 0
for r in rows:
    for d in r.get('diffs', []):
        if d['kind'] != 'menu': continue
        windows_with += 1
        tags = tags_of(d['summary'])
        for item in d['detail']:
            side, key = item.split(' ', 1)
            total += 1
            kd = kdlib.match(kds, side, key, tags, d['seat'])
            if not kd and side == 'FORGE_ONLY' and key.startswith('act|') and key.split('|')[2] in acts.get(r['id'], ()): kd = 'history-activated'
            if kd: suppressed[kd] += 1
            else:
                k = (side, key, d['seat'], tuple(sorted(tags))); unexplained[k] += 1; ex.setdefault(k, (r['id'], d['window']))
print(f'windows with menu differences {windows_with}; differing keys {total}; suppressed {dict(suppressed)}; unexplained {sum(unexplained.values())}')
for k, n in unexplained.most_common(60):
    print(f'{n:5} {k[0]:10} {k[1]:55} seat{k[2]} {list(k[3])}  e.g. {ex[k][0][-55:]} w{ex[k][1]}')
