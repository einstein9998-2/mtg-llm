#!/usr/bin/env python3
"""Write one prompt file per labeled position so a fresh Claude agent can answer it from the same view Brady got.

usage: llm_prompts.py out_dir
Reads batch{1,2,3}/batch.json + labels.jsonl. No bot numbers, no bot choice, no Brady note are written to the prompt.
Alurentell positions get the playbook as it stood BEFORE Brady's tough-spot review (so his answers cannot leak).
Writes out_dir/<id>.txt and out_dir/key.json (id -> letters of Brady's pick, the bot's pick, option count, weight).
"""
import json, os, sys, re
out = sys.argv[1]
os.makedirs(out, exist_ok=True)
root = os.path.dirname(os.path.abspath(__file__))
pb = open(os.path.join(root, '..', 'llm-player', 'alurentell-playbook-v1-archive.md')).read().split('\n## Rules from Brady')[0]
HEAD = ("You are a strong Legacy Magic: The Gathering player piloting one deck in a 1v1 game. You see only what the player sees: your own hand, "
        "the public board, graveyards, library sizes, and the opponent's hand size (not its contents). The engine lists the legal options. "
        "Pick the best one. Do not use any tools and do not read any other file; answer from this text alone.\n")
key = {}
for b in ('batch1', 'batch2', 'batch3'):
    cards = {c['id']: c for c in json.load(open(os.path.join(root, b, 'batch.json')))}
    for l in open(os.path.join(root, b, 'labels.jsonl')):
        lab = json.loads(l)
        c = cards[lab['id']]
        v = c['view']
        me = c.get('me_deck', 'Alurentell')
        opp = c['matchup'].split(' vs ')[1] if c.get('matchup') else 'UR Cutter'
        t = [HEAD, f"You play {me} against {opp}. You are on the {'play' if c.get('on_play') else 'draw'}."]
        if me == 'Alurentell':
            t.append("\n--- Your deck's strategy notes ---\n" + pb)
        t.append("\n--- The position ---")
        if c['category'] == 'keep':
            t.append("Opening hand (before any turn).")
        else:
            t.append(f"Turn {v['turn']} (global count), {v['step']}, {v['whose']} turn. Lands played this turn: {v.get('lands_played', '?')}. Floating mana (W/U/B/R/G/colorless): {'/'.join(str(x) for x in v['pool']) if any(v.get('pool', [])) else 'none'}.")
        t.append(f"Life: you {v['my_life']}, opponent {v['opp_life']}. Library: you {v['my_lib']}, opponent {v['opp_lib']}. Opponent hand: {v['opp_hand']} cards.")
        t.append(f"Your hand: {', '.join(v.get('hand', [])) or '-'}")
        t.append(f"Your battlefield: {', '.join(v['my_bf']) or '-'}")
        t.append(f"Opponent battlefield: {', '.join(v['opp_bf']) or '-'}")
        t.append(f"Your graveyard: {', '.join(v.get('my_gy', [])) or '-'}")
        t.append(f"Opponent graveyard: {', '.join(v.get('opp_gy', [])) or '-'}")
        for k in v:
            if k not in ('my_bf', 'opp_bf', 'stack', 'events', 'turn', 'step', 'whose', 'my_life', 'my_lib', 'lands_played', 'pool', 'opp_life', 'opp_hand', 'opp_lib', 'hand', 'my_gy', 'opp_gy') and v[k]:
                t.append(f"{k}: {v[k]}")
        if v['stack']:
            t.append(f"Stack (top first): {', '.join(v['stack'])}")
        if c['game_log']:
            t.append("\nWhat happened recently (your view):\n" + '\n'.join(c['game_log'][-40:]))
        t.append(f"\nDecision: {c['question']}")
        for j, o in enumerate(c['options']):
            t.append(f"  {chr(65 + j)}. {o['label']}")
        t.append("\nReply with exactly two lines: first 'ANSWER: <letter>', then 'WHY: <one or two sentences>'.")
        open(os.path.join(out, lab['id'] + '.txt'), 'w').write('\n'.join(t) + '\n')
        key[lab['id']] = {'brady': chr(65 + lab['brady_pick']), 'bot': chr(65 + lab['bot_choice']), 'deep': chr(65 + lab['deep_best']), 'n': len(c['options']), 'weight': lab['weight'], 'batch': b}
json.dump(key, open(os.path.join(out, 'key.json'), 'w'), indent=1)
print(len(key), 'prompts')
