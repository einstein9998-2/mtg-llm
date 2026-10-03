#!/usr/bin/env python3
"""Summarise a linediff output: statuses, ending kinds, most common unaligned reasons and divergences."""
import json, sys, collections, re
rows = [json.loads(l) for l in open(sys.argv[1])]
st = collections.Counter(r['status'] for r in rows)
print(dict(st))
kinds = collections.Counter(); ex = {}
for r in rows:
    if r['status'] in ('unaligned', 'diverged', 'forge_play_failed', 'forge_error', 'rebuild_error'):
        for d in r.get('diffs', []):
            if d['kind'] == 'menu': continue
            det = (d['detail'][0] if d['detail'] else '')
            det = re.sub(r'\d+', 'N', det)[:110]
            k = (r['status'], d['kind'], det)
            kinds[k] += 1; ex.setdefault(k, r['id'])
        if r['status'] in ('forge_error', 'rebuild_error'):
            k = (r['status'], 'error', re.sub(r'\d+', 'N', str(r.get('error')))[:110]); kinds[k] += 1; ex.setdefault(k, r['id'])
for k, n in kinds.most_common(int(sys.argv[2]) if len(sys.argv) > 2 else 25):
    print(f'{n:5}  {k[0]:10} {k[1]:10} {k[2]}   e.g. {ex[k][-70:]}')
