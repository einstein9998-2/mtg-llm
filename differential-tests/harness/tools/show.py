#!/usr/bin/env python3
"""show.py positions.jsonl diffs.json <substring> : print the positions whose action diff mentions substring"""
import json, sys
pos = {}
for l in open(sys.argv[1]):
    o = json.loads(l); pos[o['id']] = o
d = json.load(open(sys.argv[2]))
sub = sys.argv[3]
for i, ro, fo in d['actions']:
    if not any(sub in k for k in ro + fo): continue
    p = pos[i]
    print('==', i, 'turn', p['turn'], p['step'], 'active', p['active'])
    print('  RUST_ONLY', ro, ' FORGE_ONLY', fo)
    for l in p['state']:
        if l.split('=')[0] in ('turn','activeplayer','activephase','removesummoningsickness') or l.endswith('library='): continue
        if 'library=' in l: continue
        print('   ', l[:400])
