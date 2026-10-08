#!/usr/bin/env python3
"""Prepare the LLM-vs-LLM tough spots (source dir from the "LLM player with net tool" thread) for the answer page.
Usage: seed_llm_games.py <src dir with batch.json + batch-priority.json> <out dir> <first_order> [--all]
Priority (reviewer likely-mistake) positions come first; with --all the other flagged positions follow.
Cards written to <out>/docs/<id>.json carry no reviewer notes and no game result (hindsight would bias the answer);
those, plus the seeds, go to <out>/review-private.jsonl."""
import json, os, sys

src, out, first = sys.argv[1], sys.argv[2], int(sys.argv[3])
allp = '--all' in sys.argv
full = json.load(open(f'{src}/batch.json'))
prio_ids = [x['id'] for x in json.load(open(f'{src}/batch-priority.json'))]
by = {x['id']: x for x in full}
order = prio_ids + ([x['id'] for x in full if x['id'] not in set(prio_ids)] if allp else [])
priv = {}
for fn in ('batch-private.jsonl', 'batch-priority-private.jsonl'):
    p = f'{src}/{fn}'
    if os.path.exists(p):
        for l in open(p):
            r = json.loads(l); priv[r['id']] = r
os.makedirs(f'{out}/docs', exist_ok=True)
cards, rev = [], []
for i, pid in enumerate(order):
    c = dict(by[pid])
    rev.append({**priv.get(pid, {'id': pid}), 'flags': c.pop('flags'), 'reasons': c.pop('reasons'), 'game_result_for_llm': c.pop('game_result_for_llm'),
                'priority': pid in set(prio_ids), 'order': first + i})
    c.update(order=first + i, batch='LLM games')
    cards.append(c)
    json.dump(c, open(f'{out}/docs/{pid}.json', 'w'))
json.dump(cards, open(f'{out}/batch.json', 'w'))
open(f'{out}/review-private.jsonl', 'w').write(''.join(json.dumps(r) + '\n' for r in rev))
print(len(cards), 'cards, orders', first, '..', first + len(cards) - 1)
