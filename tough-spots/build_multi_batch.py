#!/usr/bin/env python3
"""Build one batch from several matchup runs (each run dir = `tough-spots --me A --opp B --out <A>-vs-<B>`).

usage: build_multi_batch.py runs_dir out_dir tag batch_no first_order [--per 2]
Per matchup it calls select_batch.py on that run's positions.jsonl (ids prefixed with tag + matchup), then merges the
cards (renumbered), the private records (with me/opp deck names) and batch.md. Cards carry `matchup` and `me_deck`.
"""
import json, os, subprocess, sys, glob

runs, out, tag, batch_no, first = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4]), int(sys.argv[5])
per = sys.argv[sys.argv.index('--per') + 1] if '--per' in sys.argv else '2'
CATS = ['keep', 'cast-or-wait', 'cast-which', 'land-or-fetch', 'cards:PutBack', 'cards:LookTake', 'cards:RevealPick', 'cards:DiscardToHandSize']
NAMES = {'alurentell': 'Alurentell', 'boros-aggro': 'Boros Aggro', 'bw-death-and-taxes': 'BW Death and Taxes', 'dimir-tempo': 'Dimir Tempo',
         'doomsday': 'Doomsday', 'reanimator': 'Reanimator', 'ur-cutter': 'UR Cutter', 'uwx-control': 'UWx Control', 'storm': 'Storm',
         'colorless-tron': 'Colorless Tron'}
here = os.path.dirname(os.path.abspath(__file__))
os.makedirs(out, exist_ok=True)
cards, priv, md = [], [], []
for k, d in enumerate(sorted(glob.glob(os.path.join(runs, '*-vs-*')))):
    if not os.path.isdir(d):
        continue
    me, opp = os.path.basename(d).split('-vs-')
    abbr = ''.join(w[0] for w in me.split('-'))
    src = os.path.join(d, 'positions.jsonl')
    tmp = os.path.join(out, f'_{me}')
    pre = os.path.join(tmp, 'positions-prefixed.jsonl')
    os.makedirs(tmp, exist_ok=True)
    with open(pre, 'w') as f:
        for l in open(src):
            p = json.loads(l)
            p['id'] = f'{tag}-{abbr}-{p["id"]}'
            f.write(json.dumps(p) + '\n')
    subprocess.run([sys.executable, os.path.join(here, 'select_batch.py'), pre, tmp, '--n', per, '--cats', ','.join(CATS[(int(per) * k + j) % len(CATS)] for j in range(int(per)))], check=True, stdout=subprocess.DEVNULL)
    pc = json.load(open(os.path.join(tmp, 'batch.json')))
    pp = [json.loads(l) for l in open(os.path.join(tmp, 'batch-private.jsonl'))]
    for c, p in zip(pc, pp):
        c['matchup'] = f'{NAMES.get(me, me)} vs {NAMES.get(opp, opp)}'
        c['me_deck'] = NAMES.get(me, me)
        p['me'], p['opp'] = me, opp
        cards.append(c)
        priv.append(p)
for i, (c, p) in enumerate(zip(cards, priv), 1):
    c['n'] = i
    c['batch'] = batch_no
    c['order'] = first + i - 1
with open(os.path.join(out, 'batch-private.jsonl'), 'w') as f:
    for p in priv:
        f.write(json.dumps(p) + '\n')
json.dump(cards, open(os.path.join(out, 'batch.json'), 'w'), indent=1)
for c in cards:
    md.append(f"## Position {c['n']}  ({c['category']}, {c['id']}, {c['matchup']}, you play {c['me_deck']})")
    md.append(f"{c['question']}")
    for j, r in enumerate(c['options']):
        md.append(f"  {chr(65 + j)}. {r['label']}  [net {r['net']}% | search {r['deep']}% | est. win {r['win']}%]" + (' <- bot played' if j == c['bot_choice'] else ''))
    md.append('')
open(os.path.join(out, 'batch.md'), 'w').write('\n'.join(md))
print(len(cards), 'positions:', [(c['n'], c['matchup'], c['category']) for c in cards])
