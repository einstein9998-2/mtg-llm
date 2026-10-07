#!/usr/bin/env python3
"""Turn flagged decisions from LLM-vs-LLM games into tough-spot cards for Brady's answer page.

usage: llmgame_positions.py <llm2 dir> <out dir> <batch tag> <first order> <game> [<game> ...]

Flag sources (any mix):
  <llm2>/flags/<game>a.tsv, <game>b.tsv      players' own flags: tag \t seq \t note
  <llm2>/reviews/<game>.flags.json           reviewer's flags: [{"seat":"a|b","seq":N,"note":"...","kind":"likely-mistake|close"}]
Reads <llm2>/runs/<game>/truth/transcript-{a,b}.txt (each player's own view, so nothing hidden leaks into a card).
Writes <out>/batch.json (cards for the page), batch-private.jsonl (replay actions, seeds, flag sources), batch.md.
Cards carry source="llm-vs-llm", llm_choice and flag notes; the bot-number fields are null (no bot search was run).
"""
import json, os, re, sys, collections
# --- vendored from tough-spots/select_batch.py (that module runs main() on import) ---
import re, collections
CARD_LIST = re.compile(r'(.+?) #\d+(?:, |$)')
ID = re.compile(r' #\d+')
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
# --- end vendored ---

DECK_NAMES = {'alurentell': 'Alurentell', 'ur-cutter': 'UR Cutter', 'boros-aggro': 'Boros Aggro', 'bw-death-and-taxes': 'BW Death and Taxes',
              'dimir-tempo': 'Dimir Tempo', 'uwx-control': 'UWx Control', 'doomsday': 'Doomsday', 'reanimator': 'Reanimator'}
HDR = re.compile(r'^=== PROMPT (\d+) \| turn (\d+) \| (.*?) \| (YOUR|OPPONENT\'S) turn ===$')


def parse_transcript(path):
    out, cur = {}, None
    for ln in open(path).read().split('\n'):
        m = HDR.match(ln)
        if m:
            cur = {'seq': int(m.group(1)), 'turn': int(m.group(2)), 'step': m.group(3), 'whose': m.group(4), 'lines': [ln], 'actions_before': None, 'chose': None}
            out[cur['seq']] = cur
        elif cur is not None:
            if ln.startswith('>>> ACTIONS-BEFORE'):
                cur['actions_before'] = int(ln.split()[-1])
            elif ln.startswith('>>> CHOSE'):
                m2 = re.match(r'>>> CHOSE \[(\d+)\] (.*)', ln)
                cur['chose'] = (int(m2.group(1)), m2.group(2))
            else:
                cur['lines'].append(ln)
    return out


def split_decision(lines):
    """Return (view_text_for_parse_view, decision_kind, context, options)"""
    body = []
    kind = ctx = None
    opts = []
    in_dec = False
    for i, ln in enumerate(lines):
        if i == 0:
            m = HDR.match(ln)
            body.append(f"Turn {m.group(2)} | {m.group(3)} | {m.group(4)} turn")
            continue
        if ln.startswith('DECISION:'):
            in_dec = True
            kind = ln.split(':', 1)[1].strip()
            continue
        if in_dec:
            m = re.match(r'^  \[(\d+)\] (\S+) (.*)$', ln)
            if m:
                opts.append({'idx': int(m.group(1)), 'kind': m.group(2), 'label': m.group(3)})
            elif ln.strip():
                ctx = (ctx + ' ' if ctx else '') + ln.strip()
        else:
            body.append(ln)
    return '\n'.join(body), kind, ctx or '', opts


def category(kind, opts):
    if kind == 'Mulligan':
        return 'keep'
    if kind == 'Priority':
        return 'cast-or-wait'
    return 'card-pick'


def load_flags(llm2, game):
    flags = collections.defaultdict(list)  # (seat, seq) -> [notes]
    for seat in 'ab':
        p = f'{llm2}/flags/{game}{seat}.tsv'
        if os.path.exists(p):
            for ln in open(p):
                parts = ln.rstrip('\n').split('\t')
                if len(parts) >= 3 and parts[1].isdigit():
                    flags[(seat, int(parts[1]))].append({'by': 'player', 'note': parts[2]})
    p = f'{llm2}/reviews/{game}.flags.json'
    if os.path.exists(p):
        for f in json.load(open(p)):
            flags[(f['seat'], int(f['seq']))].append({'by': 'reviewer', 'note': f.get('note', ''), 'kind': f.get('kind', '')})
    return flags


def main():
    llm2, out, tag, first_order = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4])
    games = sys.argv[5:]
    os.makedirs(out, exist_ok=True)
    cards, priv = [], []
    order = first_order
    meta = json.load(open(f'{llm2}/games.json')) if os.path.exists(f'{llm2}/games.json') else {}
    for g in games:
        gm = meta.get(g, {})
        flags = load_flags(llm2, g)
        tr = {s: parse_transcript(f'{llm2}/runs/{g}/truth/transcript-{s}.txt') for s in 'ab'}
        actions = None
        ap = f'{llm2}/runs/{g}/truth/actions.json'
        if os.path.exists(ap):
            actions = json.load(open(ap))
        for (seat, seq), notes in sorted(flags.items(), key=lambda kv: (kv[0][0], kv[0][1])):
            blk = tr[seat].get(seq)
            if not blk or not blk['chose']:
                continue
            view_text, kind, ctx, opts = split_decision(blk['lines'])
            events = []
            for s in sorted(k for k in tr[seat] if k <= seq):
                for x in tr[seat][s]['lines']:
                    pass
            # game log: every RECENT EVENTS line this player saw up to and including this prompt
            log = []
            for s in sorted(k for k in tr[seat] if k <= seq):
                sect = False
                for x in tr[seat][s]['lines']:
                    if x.startswith('RECENT EVENTS'):
                        sect = True
                    elif x.startswith('DECISION:'):
                        sect = False
                    elif sect and x.startswith('  '):
                        log.append(re.sub(r' #\d+', '', x.strip()))
            v = parse_view(view_text)
            deck = gm.get(seat + '_deck')
            opp = gm.get('b_deck' if seat == 'a' else 'a_deck')
            labels = [clean_label(o['label']) for o in opts]
            rows = [{'idx': o['idx'], 'label': labels[i], 'kind': o['kind'], 'quick': None, 'deep': None, 'net': None, 'win': None} for i, o in enumerate(opts)]
            chosen = blk['chose'][0]
            card = {
                'id': f'{tag}-{g}-{seat}{seq}', 'n': len(cards) + 1, 'category': category(kind, opts), 'turn': blk['turn'], 'view': v,
                'game_log': log, 'decision_kind': kind, 'context': ctx, 'question': ctx or kind, 'options': rows,
                'source': 'llm-vs-llm', 'llm_choice': chosen, 'bot_choice': None, 'deep_best': None, 'net_top': None, 'win_est': None,
                'flags': notes, 'reasons': [n['by'] + ('-' + n['kind'] if n.get('kind') else '') for n in notes],
                'matchup': f"{DECK_NAMES.get(deck, deck)} vs {DECK_NAMES.get(opp, opp)}" if deck else '', 'me_deck': DECK_NAMES.get(deck, deck) if deck else '',
                'game_result_for_llm': gm.get(seat + '_result'), 'on_play': gm.get('first_seat') == gm.get(seat + '_seat'),
                'batch': tag, 'order': order,
            }
            order += 1
            cards.append(card)
            priv.append({'id': card['id'], 'game': g, 'seat_tag': seat, 'seq': seq, 'seed': gm.get('seed'), 'llm_seat': gm.get(seat + '_seat'),
                         'first': gm.get('first_seat'), 'decks': [gm.get('a_deck'), gm.get('b_deck')],
                         'replay': actions[:blk['actions_before']] if (actions is not None and blk['actions_before'] is not None) else None})
    json.dump(cards, open(f'{out}/batch.json', 'w'), indent=1)
    with open(f'{out}/batch-private.jsonl', 'w') as f:
        for p in priv:
            f.write(json.dumps(p) + '\n')
    md = []
    for c in cards:
        v = c['view']
        md.append(f"## {c['id']} ({c['category']}, {c['matchup']})")
        md.append(f"Turn {v.get('turn')} {v.get('step')}, {v.get('whose')} turn. You {v.get('my_life')} life, opponent {v.get('opp_life')} life, opponent hand {v.get('opp_hand')}.")
        md.append(f"Your hand: {', '.join(v.get('hand', [])) or '-'}")
        md.append('Options: ' + '; '.join(f"{chr(65 + o['idx'])}. {o['label']}" for o in c['options']) + f"  (LLM chose {chr(65 + c['llm_choice'])})")
        for n in c['flags']:
            md.append(f"- {n['by']}: {n['note']}")
        md.append('')
    open(f'{out}/batch.md', 'w').write('\n'.join(md))
    print(f'{len(cards)} positions -> {out}')


if __name__ == '__main__':
    main()
