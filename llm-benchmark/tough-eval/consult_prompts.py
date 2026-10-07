#!/usr/bin/env python3
"""Tough-spot agreement test with the consult tool.

usage: consult_prompts.py <tough-spots dir> <baseline prompts dir (llm_prompts.py output)> <out dir>
For every labeled position (batches 1-3) take the blinded prompt the no-tool LLM got (llm_prompts.py output) and
append what `lg.sh consult` would show for that position: the net's value and priors and the search's visit share and
win estimate per option, from the bot's own view (stored in batch-private.jsonl). No Brady answer and no Brady note.
Writes <out>/<id>.txt and <out>/key.json (copy of the baseline key). Numbers come from the flagger's stored searches
(64-400 iterations), not from a fresh 1000-iteration call, and are on the engine that batch was made on.
"""
import json, os, sys
root, base, out = sys.argv[1], sys.argv[2], sys.argv[3]
os.makedirs(out, exist_ok=True)
key = json.load(open(os.path.join(root, 'llm-eval', 'key.json')))
INTRO = ("\n--- Consult tool: a net + search bot's read of this decision ---\n"
         "The bot was computed from the same view you see (the opponent's hand is guessed from their decklist). It plays this kind of position at a modest level "
         "and you are the stronger player: it is good at tempo, sequencing, mana and counter-magic tradeoffs, and weak on long combo lines and on what the opponent's hidden cards do. "
         "Use it as a second opinion; the decision is yours.\n")
def pct(x): return '-' if x is None else f"{round(x*100):d}"
n = 0
for b in ('batch1', 'batch2', 'batch3'):
    for l in open(os.path.join(root, b, 'batch-private.jsonl')):
        r = json.loads(l)
        if r['id'] not in key: continue
        p = open(os.path.join(base, r['id'] + '.txt')).read()
        assert 'Reply with exactly two lines' in p
        head, tail = p.rsplit('\nReply with exactly two lines', 1)
        rows = sorted(range(len(r['options'])), key=lambda i: -(r['options'][i]['search_share'] or 0))
        t = [INTRO, f"Net alone (no search): you win about {pct((r['net_value'] + 1) / 2)}%.",
             f"Search: you win about {pct(r.get('deep_win_est'))}%.",
             "Per option (net% = the net's prior, visit% = share of search visits, win% = search's win estimate if you take it; a rare option has a noisy estimate):",
             "  option  net%  visit%  win%"]
        for i in rows[:6]:
            o = r['options'][i]
            t.append(f"  {chr(65 + i)}      {pct(o['net_prior']):>4}  {pct(o['search_share']):>6}  {pct(o.get('win_est')):>4}   {o['label']}")
        if len(rows) > 6: t.append(f"  ({len(rows) - 6} other options got almost no visits)")
        open(os.path.join(out, r['id'] + '.txt'), 'w').write(head + '\n' + '\n'.join(t) + '\n\nReply with exactly two lines' + tail)
        n += 1
json.dump(key, open(os.path.join(out, 'key.json'), 'w'), indent=1)
print(n, 'prompts')
