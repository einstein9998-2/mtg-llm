#!/usr/bin/env python3
"""Turn Brady's saved answers into labeled positions and a notes digest.

usage: import_answers.py answers_dir batch-private.jsonl out_dir
  answers_dir: documents saved with ArtifactData (action list/query on `answers`, out_dir=...), one JSON per position id
Writes out_dir/labels.jsonl (state + option features + Brady's pick as the policy target, plus the bot's choice) and
out_dir/notes.md (every note, grouped: rule candidates first). Positions answered "Either is fine" get a
two-hot target when both picks are given; "Can't tell from this view" answers are kept but weight 0.
"""
import json, os, sys, glob

adir, priv, out = sys.argv[1:4]
os.makedirs(out, exist_ok=True)
ans = {}
for f in glob.glob(os.path.join(adir, '**', '*.json'), recursive=True):
    d = json.load(open(f))
    d = d.get('data', d)
    ans[os.path.basename(f)[:-5]] = d
n = agree = 0
rules, other, moot = [], [], []
with open(os.path.join(out, 'labels.jsonl'), 'w') as lab:
    for l in open(priv):
        p = json.loads(l)
        a = ans.get(p['id'])
        if a and a.get('pick') is None and a.get('note'):
            moot.append(f"- {p['id']} ({p['decision_kind']}, turn {p['turn']}): no pick; Brady declined to answer: {a['note']}")
            continue
        if not a or a.get('pick') is None:
            continue
        fl = a.get('flags', {})
        weight = 0.0 if fl.get('cant_tell') else (0.5 if fl.get('either') else 1.0)
        n += 1
        agree += a['pick'] == p['bot_choice']
        lab.write(json.dumps({'id': p['id'], 'n_defs': p['train']['n_defs'], 'state_sparse': p['train']['state_sparse'],
                              'option_feats': p['train']['option_feats'], 'brady_pick': a['pick'], 'bot_choice': p['bot_choice'],
                              'deep_best': p['deep_best'], 'weight': weight, 'flags': fl, 'replay': p['replay'],
                              'game_seed': p['game_seed'], 'bot_seat': p['bot_seat'], 'first': p['first']}) + '\n')
        row = f"- {p['id']} ({p['decision_kind']}, turn {p['turn']}): Brady picked '{p['options'][a['pick']]['label']}', bot '{p['options'][p['bot_choice']]['label']}'. {a.get('note','')}"
        (rules if fl.get('rule') else other).append(row)
with open(os.path.join(out, 'notes.md'), 'w') as f:
    f.write('## Rule candidates\n' + '\n'.join(rules) + '\n\n## Other answers\n' + '\n'.join(other) + '\n\n## Declined (the position is wrong because of an earlier bot mistake)\n' + '\n'.join(moot) + '\n')
print(f'{n} labeled positions; Brady agrees with the bot on {agree}')
