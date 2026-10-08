#!/usr/bin/env python3
"""playbyplay.py GAMEID [outfile]: merge both transcripts + the players' pick reasons into one readable play-by-play
(for Brady, who sees both hands). Needs runs/GAME/truth/transcript-{a,b}.txt and optional runs/GAMEa|b/reasons.tsv."""
import re, sys, os
W = '/home/claude/work/llm2'
gid = sys.argv[1]
out = sys.argv[2] if len(sys.argv) > 2 else f'{W}/match/{gid}-playbyplay.md'

def blocks(path):
    txt = open(path).read()
    parts = re.split(r'(?m)^(?==== PROMPT )', txt)
    res = []
    for p in parts:
        m = re.match(r'=== PROMPT (\d+) \| turn (\d+) \| ([^|]+) \| ([^=]+?) ===', p)
        if not m: continue
        b = {'seq': int(m.group(1)), 'turn': m.group(2), 'step': m.group(3).strip(), 'whose': m.group(4).strip()}
        cur = None; fields = {}
        opts = []; chose = None; desc = []; indec = False; before = None
        for line in p.split('\n')[1:]:
            if line.startswith('>>> CHOSE'): chose = line[10:].strip(); continue
            if line.startswith('>>> ACTIONS-BEFORE'): before = line.split()[-1]; continue
            if line.startswith('DECISION:'): indec = True; fields['DECISION'] = line[9:].strip(); continue
            if indec:
                mo = re.match(r'\s+\[(\d+)\] (.*)$', line)
                if mo: opts.append((int(mo.group(1)), mo.group(2)))
                elif line.strip(): desc.append(line.strip())
                continue
            mk = re.match(r'([A-Z][A-Z ]+?)(?: \([^)]*\))?:(.*)$', line)
            if mk and not line.startswith(' '):
                cur = mk.group(1); fields[cur] = mk.group(2).strip()
            elif cur and line.startswith(' '):
                fields[cur] = (fields[cur] + '; ' if fields[cur] else '') + line.strip()
        b.update(fields=fields, opts=opts, chose=chose, desc=' '.join(desc), before=before)
        res.append(b)
    return res

def reasons(tag):
    d = {}
    p = f'{W}/runs/{tag}/reasons.tsv'
    if os.path.exists(p):
        for line in open(p):
            f = line.rstrip('\n').split('\t')
            if len(f) == 3: d[(int(f[0]), f[1])] = f[2]
    return d

flags = {}
for t in ('a', 'b'):
    fp = f'{W}/flags/{gid}{t}.tsv'
    if os.path.exists(fp):
        for line in open(fp):
            f = line.rstrip('\n').split('\t')
            if len(f) >= 2 and f[0].isdigit(): flags[(t, int(f[0]))] = f[1] if len(f) == 2 else f[-1]

ent = []
for t in ('a', 'b'):
    tp = f'{W}/runs/{gid}/truth/transcript-{t}.txt'
    if not os.path.exists(tp): continue
    rs = reasons(f'{gid}{t}')
    for b in blocks(tp):
        idx = re.match(r'\[(\d+)\]', b['chose'] or '')
        b['side'] = t.upper()
        b['reason'] = rs.get((b['seq'], idx.group(1))) if idx else None
        if b['reason'] is None: b['reason'] = rs.get((b['seq'], 'auto'))
        b['flag'] = flags.get((t, b['seq']))
        ent.append(b)
ent.sort(key=lambda b: b['seq'])

L = [f'# Play-by-play {gid}', '', 'A = Alurentell, B = opponent. Each entry is one decision the player made, with the state it saw, the options, the pick and its stated reason. Prompts with only "Pass priority" and no reason are shown in one line. Macro (auto) stretches show only where the player stopped.', '']
lastturn = None
for b in ent:
    f = b['fields']
    mine = 'YOUR' in b['whose'].upper()
    active = b['side'] if mine else ('B' if b['side'] == 'A' else 'A')
    key = (b['turn'], active)
    if key != lastturn:
        L.append(f'\n## Turn {b["turn"]} ({active}\'s turn)'); lastturn = key
    names = [o[1] for o in b['opts']]
    trivial = len(b['opts']) <= 1 or (b['chose'] and b['chose'].endswith('Pass priority') and not b['reason'] and b['desc'].startswith('You have priority') and not f.get('STACK'))
    ch = b['chose'] or '(no pick recorded)'
    if trivial and not b['reason']:
        L.append(f'- [{b["side"]}{b["seq"]}] {b["step"]}: {ch}'); continue
    L.append(f'\n**[{b["side"]}{b["seq"]}] {b["step"]}, {b["fields"].get("DECISION","")}**')
    ym = re.search(r'life (\d+)', f.get('YOU', '')); om = re.search(r'life (\d+) \| hand (\d+)', f.get('OPPONENT', ''))
    L.append(f'- Life: me {ym.group(1) if ym else "?"}, opponent {om.group(1) if om else "?"} (hand {om.group(2) if om else "?"})' + (f' | Exile: {f["EXILE"]}' if f.get('EXILE') and f['EXILE'] != '-' else ''))
    L.append(f'- Hand: {f.get("YOUR HAND","-")}')
    L.append(f'- Mine: {f.get("YOUR BATTLEFIELD","-")} | Theirs: {f.get("OPPONENT BATTLEFIELD","-")}')
    if f.get('YOUR GRAVEYARD', '-') != '-' or f.get('OPPONENT GRAVEYARD', '-') != '-':
        L.append(f'- Graveyards: mine {f.get("YOUR GRAVEYARD","-")} | theirs {f.get("OPPONENT GRAVEYARD","-")}')
    if f.get('STACK'): L.append(f'- Stack: {f["STACK"]}')
    if f.get('RECENT EVENTS'): L.append(f'- Since last: {f["RECENT EVENTS"][:400]}')
    L.append(f'- Options: ' + ' / '.join(f'[{i}] {n}' for i, n in b['opts'][:14]))
    L.append(f'- **Chose:** {ch}')
    if b['reason']: L.append(f'- **Why:** {b["reason"]}')
    if b['flag']: L.append(f'- *Player flagged this as a tough spot:* {b["flag"]}')
open(out, 'w').write('\n'.join(L) + '\n')
print(out, len(ent), 'decisions')
