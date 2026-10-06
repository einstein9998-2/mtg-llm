#!/usr/bin/env python3
"""Pick a small, varied batch of flagged positions and turn them into readable cards.

usage: select_batch.py positions.jsonl out_dir [--n 10] [--exclude ids.txt]
Writes out_dir/batch.json (for the answer page), out_dir/batch.md (plain text) and
out_dir/batch-private.jsonl (full records incl. training fields, for mapping answers back).
Reads only what the bot saw (rendered Observation text and its own numbers).
"""
import json, re, sys, collections, os

CARD_LIST = re.compile(r'(.+?) #\d+(?:, |$)')
ID = re.compile(r' #\d+')


def cards(s):
    s = s.strip()
    if s in ('-', ''):
        return []
    return [m.strip() for m in CARD_LIST.findall(s)]


def counted(items):
    c = collections.OrderedDict()
    for i in items:
        c[i] = c.get(i, 0) + 1
    return [f'{n}x {k}' if n > 1 else k for k, n in c.items()]


def parse_view(text):
    v = {'my_bf': [], 'opp_bf': [], 'stack': [], 'events': []}
    lines = text.split('\n')
    sect = None
    for ln in lines:
        if ln.startswith('Turn '):
            a = ln.split(' | ')
            v['turn'] = a[0].replace('Turn ', '')
            v['step'] = a[1]
            v['whose'] = 'your' if 'YOUR' in a[2] else "opponent's"
            sect = None
        elif ln.startswith('YOU:'):
            m = re.search(r'life (\d+) \| library (\d+)', ln)
            v['my_life'], v['my_lib'] = int(m.group(1)), int(m.group(2))
            m = re.search(r'lands played this turn (\d+)', ln)
            v['lands_played'] = int(m.group(1))
            m = re.search(r'mana pool \[(.*?)\]', ln)
            pool = [int(x) for x in m.group(1).split(',')] if m else []
            v['pool'] = pool if any(pool) else []
            sect = None
        elif ln.startswith('OPPONENT:'):
            m = re.search(r'life (\d+) \| hand (\d+) cards \| library (\d+)', ln)
            v['opp_life'], v['opp_hand'], v['opp_lib'] = int(m.group(1)), int(m.group(2)), int(m.group(3))
            sect = None
        elif ln.startswith('  your designations:'):
            v['my_desig'] = ln.split(':', 1)[1].strip()
        elif ln.startswith('  opponent designations:'):
            v['opp_desig'] = ln.split(':', 1)[1].strip()
        elif ln.startswith('  opponent cards you know in hand:'):
            v['opp_known_hand'] = counted(cards(ln.split(':', 1)[1]))
        elif ln.startswith('YOUR HAND:'):
            v['hand'] = counted(cards(ln.split(':', 1)[1]))
            sect = None
        elif ln.startswith('YOUR LIBRARY TOP'):
            v['lib_top'] = cards(ln.split(':', 1)[1])
        elif ln.startswith('YOUR LIBRARY BOTTOM'):
            v['lib_bottom'] = cards(ln.split(':', 1)[1])
        elif ln.startswith('YOUR BATTLEFIELD:'):
            sect = 'my_bf'
        elif ln.startswith('OPPONENT BATTLEFIELD:'):
            sect = 'opp_bf'
        elif ln.startswith('YOUR GRAVEYARD:'):
            v['my_gy'] = counted(cards(ln.split(':', 1)[1]))
            sect = None
        elif ln.startswith('OPPONENT GRAVEYARD:'):
            v['opp_gy'] = counted(cards(ln.split(':', 1)[1]))
            sect = None
        elif ln.startswith('EXILE:'):
            a = ln[len('EXILE:'):].split('| opponent')
            v['my_exile'] = counted(cards(a[0].replace('yours:', '')))
            v['opp_exile'] = counted(cards(a[1].split(':', 1)[1])) if len(a) > 1 else []
            sect = None
        elif ln.startswith('STACK'):
            sect = 'stack'
        elif ln.startswith('RECENT EVENTS'):
            sect = 'events'
        elif ln.startswith('DECISION:'):
            sect = None
        elif ln.startswith('  ') and sect:
            t = ln.strip()
            if sect in ('my_bf', 'opp_bf'):
                t = ID.sub('', t).replace(' (owned by other)', '')
                v[sect].append(t)
            elif sect == 'stack':
                v['stack'].append(ID.sub('', t))
            else:
                v['events'].append(t)
    for k in ('my_bf', 'opp_bf', 'stack'):
        v[k] = counted(v[k])
    return v


def clean_label(l):
    l = re.sub(r' \[v\d+\]', '', l)
    l = re.sub(r' ability \d+$', '', l)
    return l


def option_rows(p):
    rows = []
    labels = [clean_label(o['label']) for o in p['options']]
    dup = collections.Counter(labels)
    for i, o in enumerate(p['options']):
        lab = labels[i]
        if dup[lab] > 1:
            m = re.search(r'\[v(\d+)\]', o['label'])
            lab += f' (copy #{sum(1 for j in range(i) if labels[j] == lab) + 1})'
        rows.append({
            'idx': o['idx'], 'label': lab, 'kind': o['kind'],
            'quick': round(o['quick_share'] * 100), 'deep': round(o['search_share'] * 100),
            'net': round(o['net_prior'] * 100),
            'win': None if o['win_est'] is None else round(o['win_est'] * 100),
        })
    return rows


QUESTIONS = {
    'DiscardToHandSize': 'Cleanup: discard down to hand size. Which card goes?',
    'PutBack': 'Brainstorm: put a card back on top of your library ({r} left to choose).',
    'OrderTop': 'Order the cards for the top of your library ({r} left to place).',
    'OrderBottom': 'Order the cards for the bottom of your library ({r} left to place).',
    'RevealPick': 'Atraxa reveal: you take one card per type, one pick at a time. This pick is one card from the revealed cards (a card of a type already taken is not offered; Done stops).',
    'LookTake': 'Look at the cards: pick one to take ({r} left to take).',
    'ScryBottom': 'Scry: choose a card to put on the bottom (Done keeps the rest on top).',
    'SurveilGraveyard': 'Surveil: choose a card to put into the graveyard (Done keeps the rest).',
    'Search': 'Search your library: which card?',
    'PayCost': 'Pick a card to pay a cost with.',
    'CastFree': 'Pick a card to cast for free.',
    'PutEach': 'Pick a card to put onto the battlefield.',
    'DiscardEffect': 'Pick a card to discard.',
    'SacrificeEffect': 'Pick a permanent to sacrifice.',
}


def question(p):
    k = p['decision_kind']
    if k.startswith('Priority'):
        return 'You have priority. What do you do?'
    if k.startswith('Mulligan'):
        return 'Keep this hand or mulligan?'
    if k.startswith('ChooseCards'):
        pur = re.search(r'purpose: (\w+)', k).group(1)
        r = re.search(r'remaining: (\d+)', k).group(1)
        return QUESTIONS.get(pur, p['context']).replace('{r}', r)
    if k.startswith('ChooseDungeon'):
        return 'Choose a dungeon to venture into.'
    if k.startswith('ChooseRoom'):
        return 'Choose the next dungeon room.'
    if k.startswith('May'):
        return p['context'] + ' (Yes / No)'
    if k.startswith('DeclareAttacker'):
        return p['context']
    if k.startswith('DeclareBlocker'):
        return p['context'] or 'Block?'
    return p['context']


def category(p):
    k = p['decision_kind']
    top = [o['kind'] for o in sorted(p['options'], key=lambda o: -o['search_share'])[:2]]
    if k.startswith('Mulligan'):
        return 'keep'
    if k.startswith('ChooseCards'):
        return 'cards:' + re.search(r'purpose: (\w+)', k).group(1)
    if k.startswith('Priority'):
        if 'CastSpell' in top and 'Pass' in top:
            return 'cast-or-wait'
        if 'CastSpell' in top:
            return 'cast-which'
        if 'PlayLand' in top or 'ActivateAbility' in top:
            return 'land-or-fetch'
        return 'priority-other'
    return 'other:' + k.split('{')[0].strip()


def main():
    path, out = sys.argv[1], sys.argv[2]
    n = int(sys.argv[sys.argv.index('--n') + 1]) if '--n' in sys.argv else 10
    excl = set()
    if '--exclude' in sys.argv:
        excl = set(open(sys.argv[sys.argv.index('--exclude') + 1]).read().split())
    os.makedirs(out, exist_ok=True)
    P = [json.loads(l) for l in open(path)]
    P = [p for p in P if 0.25 <= p['deep_win_est'] <= 0.75 and p['id'] not in excl]
    # drop positions where the best two options are not really distinct choices
    keep = []
    for p in P:
        top = sorted(p['options'], key=lambda o: -o['search_share'])[:2]
        if top[0]['label'] == top[1]['label'] and top[0]['kind'] == top[1]['kind'] and 'v' not in top[0]['label']:
            continue
        keep.append(p)
    P = keep
    # Prefer early and mid-game decisions (a human can judge those), and skip loop-bloated hands.
    def hand_size(p):
        ln = [l for l in p['view'].split('\n') if l.startswith('YOUR HAND:')]
        return ln[0].count(' #') if ln else 0
    P = [p for p in P if hand_size(p) <= 9 or p['decision_kind'].startswith('ChooseCards')]
    for p in P:
        p['score'] *= 1.8 if p['turn'] <= 8 else (1.0 if p['turn'] <= 14 else 0.5)
    by = collections.defaultdict(list)
    for p in P:
        by[category(p)].append(p)
    for c in by:
        by[c].sort(key=lambda p: -p['score'])
    print({c: len(v) for c, v in sorted(by.items(), key=lambda kv: -len(kv[1]))})
    nk = int(sys.argv[sys.argv.index('--keeps') + 1]) if '--keeps' in sys.argv else 2
    quotas = [('keep', nk), ('cast-or-wait', 2), ('cast-which', 2), ('cards:RevealPick', 1), ('cards:PutBack', 1), ('cards:DiscardToHandSize', 1), ('cards:LookTake', 1), ('land-or-fetch', 1)]
    chosen, games = [], set()

    def take(c, k):
        for p in by.get(c, []):
            if k == 0:
                break
            if p['game'] in games:
                continue
            chosen.append(p)
            games.add(p['game'])
            k -= 1

    for c, k in quotas:
        take(c, k)
    rest = sorted((p for c in by for p in by[c]), key=lambda p: -p['score'])
    for p in rest:
        if len(chosen) >= n:
            break
        if p['game'] in games or p in chosen:
            continue
        chosen.append(p)
        games.add(p['game'])
    chosen = chosen[:n]
    cards_out, md = [], []
    with open(os.path.join(out, 'batch-private.jsonl'), 'w') as f:
        for p in chosen:
            f.write(json.dumps(p) + '\n')
    for i, p in enumerate(chosen, 1):
        v = parse_view(p['view'])
        rows = option_rows(p)
        card = {
            'id': p['id'], 'n': i, 'category': category(p), 'turn': p['turn'], 'view': v,
            'game_log': [re.sub(r' #\d+', '', x) for x in p['game_log']],
            'decision_kind': p['decision_kind'], 'context': p['context'], 'question': question(p),
            'options': rows, 'bot_choice': p['bot_choice'], 'deep_best': p['deep_best'], 'net_top': p['net_top'],
            'win_est': round(p['deep_win_est'] * 100), 'reasons': p['reasons'],
            'game_result_for_bot': p['game_result_for_bot'], 'on_play': p['first'] == p['bot_seat'],
        }
        cards_out.append(card)
        md.append(f"## Position {i}  ({card['category']}, {p['id']})")
        md.append(f"Turn {v['turn']} {v['step']}, {v['whose']} turn. You {v['my_life']} life, opponent {v['opp_life']} life, opponent hand {v['opp_hand']} cards, libraries {v['my_lib']}/{v['opp_lib']}.")
        md.append(f"Your hand: {', '.join(v.get('hand', [])) or '-'}")
        md.append(f"Your board: {', '.join(v['my_bf']) or '-'}")
        md.append(f"Opponent board: {', '.join(v['opp_bf']) or '-'}")
        md.append(f"Your graveyard: {', '.join(v.get('my_gy', [])) or '-'} | Opponent graveyard: {', '.join(v.get('opp_gy', [])) or '-'}")
        if v['stack']:
            md.append(f"Stack: {', '.join(v['stack'])}")
        md.append(f"Decision: {p['decision_kind']} {p['context']}")
        for j, r in enumerate(rows):
            tag = ' <- bot played' if j == p['bot_choice'] else ''
            md.append(f"  {chr(65 + j)}. {r['label']}  [net {r['net']}% | search {r['deep']}% | est. win {r['win']}%]{tag}")
        md.append('')
    json.dump(cards_out, open(os.path.join(out, 'batch.json'), 'w'), indent=1)
    open(os.path.join(out, 'batch.md'), 'w').write('\n'.join(md))
    print('selected', [(c['n'], c['category'], c['id']) for c in cards_out])


main()
