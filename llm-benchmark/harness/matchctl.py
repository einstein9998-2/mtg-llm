#!/usr/bin/env python3
"""Best-of-three match control for the LLM-vs-LLM harness.
  matchctl.py new MID OPPDECK FIRST_G1     FIRST_G1 = A|B   (A = Alurentell, B = OPPDECK)
  matchctl.py check MID A|B                validate match/MID/boardA.txt (or boardB.txt) against the base 75
  matchctl.py launch MID K                 build the boarded lists for game K and start the engine (game id MIDgK, tags MIDgKa / MIDgKb)
  matchctl.py result MID K                 read the finished game, update the score, print who plays first next
  matchctl.py status [MID]
Swap files are relative to the BASE 75 (never stacked): lines '-N Card' (out of the main) and '+N Card' (in from the sideboard).
Game 1 plays the main decks. Games 2 and 3: the loser plays first."""
import json, os, re, subprocess, sys, shutil, glob
W = '/home/claude/work/llm2'
DECKS = '/home/claude/work/live/engine/decks'
CARDS = '/home/claude/work/live/engine/crates/mtg-cards/cards'
MJ = f'{W}/matches.json'
BASIC = {'Plains', 'Island', 'Swamp', 'Mountain', 'Forest'}

def load():
    return json.load(open(MJ)) if os.path.exists(MJ) else {}
def save(m):
    json.dump(m, open(MJ, 'w'), indent=1)

def parse(path):
    main, side, cur, hdr = {}, {}, None, []
    cur = main
    started = False
    for line in open(path):
        line = line.rstrip('\n')
        if line.startswith('#'):
            hdr.append(line); continue
        if not line.strip():
            if started: cur = side
            continue
        m = re.match(r'(\d+)\s+(.+)$', line.strip())
        if not m: continue
        started = True
        cur[m.group(2)] = cur.get(m.group(2), 0) + int(m.group(1))
    return main, side, hdr

def names():
    s = set()
    for f in glob.glob(f'{CARDS}/*.ron'):
        s |= set(re.findall(r'name:\s*"([^"]+)"', open(f).read()))
    return s

def swaps(path):
    out, inn = {}, {}
    if not os.path.exists(path): return out, inn
    for line in open(path):
        line = line.split('#')[0].strip()
        if not line: continue
        m = re.match(r'([+-])\s*(\d+)\s+(.+)$', line)
        if not m: raise ValueError(f'bad swap line: {line!r} (use "-N Card" or "+N Card")')
        d = out if m.group(1) == '-' else inn
        d[m.group(3).strip()] = d.get(m.group(3).strip(), 0) + int(m.group(2))
    return out, inn

def board(base, swapfile):
    main, side, hdr = parse(base)
    out, inn = swaps(swapfile)
    known = names()
    if sum(out.values()) != sum(inn.values()):
        raise ValueError(f'swaps must be one for one: out {sum(out.values())}, in {sum(inn.values())}')
    for c, n in out.items():
        if main.get(c, 0) < n: raise ValueError(f'cannot take out {n} {c}: main has {main.get(c, 0)}')
    for c, n in inn.items():
        if side.get(c, 0) < n: raise ValueError(f'cannot bring in {n} {c}: sideboard has {side.get(c, 0)}')
        if c not in known: raise ValueError(f'{c} is not defined in the engine')
    m2, s2 = dict(main), dict(side)
    for c, n in out.items():
        m2[c] -= n; s2[c] = s2.get(c, 0) + n
    for c, n in inn.items():
        s2[c] -= n; m2[c] = m2.get(c, 0) + n
    m2 = {c: n for c, n in m2.items() if n}; s2 = {c: n for c, n in s2.items() if n}
    if sum(m2.values()) != 60: raise ValueError(f'main would have {sum(m2.values())} cards')
    if sum(s2.values()) != sum(side.values()): raise ValueError('sideboard size changed')
    for c, n in m2.items():
        if n > 4 and c not in BASIC: raise ValueError(f'{n} copies of {c}')
    return m2, s2, hdr

def write(path, main, side, hdr, note=''):
    with open(path, 'w') as f:
        for h in hdr: f.write(h + '\n')
        if note: f.write(f'# {note}\n')
        for c, n in main.items(): f.write(f'{n} {c}\n')
        f.write('\n')
        for c, n in side.items(): f.write(f'{n} {c}\n')

def summary(base, swapfile):
    out, inn = swaps(swapfile)
    return 'OUT ' + ', '.join(f'{n} {c}' for c, n in out.items()) + ' | IN ' + ', '.join(f'{n} {c}' for c, n in inn.items()) if out or inn else 'no change'

def mnum(mid): return int(re.sub(r'\D', '', mid))

def cmd_new(mid, opp, first):
    m = load()
    assert mid not in m, f'{mid} exists'
    assert first in ('A', 'B')
    os.makedirs(f'{W}/match/{mid}', exist_ok=True)
    m[mid] = {'opp': opp, 'first_g1': first, 'games': []}
    save(m); print(f'{mid}: Alurentell (A) vs {opp} (B), {first} plays first in game 1')

def cmd_check(mid, side):
    m = load()[mid]
    base = f'{DECKS}/' + ('alurentell' if side == 'A' else m['opp']) + '.txt'
    sw = f'{W}/match/{mid}/board{side}.txt'
    try:
        main, sd, _ = board(base, sw)
    except Exception as e:
        print('INVALID:', e); sys.exit(1)
    print('OK', summary(base, sw), f'| main {sum(main.values())} side {sum(sd.values())}')

def first_of(m, mid, k):
    g = m[mid]['games']
    if k == 1: return m[mid]['first_g1']
    prev = g[k - 2]
    r = prev.get('result')
    if r in ('A', 'B'): return 'B' if r == 'A' else 'A'   # loser plays first
    return prev['first']                                   # draw: same player

def cmd_launch(mid, k):
    m = load(); st = m[mid]; k = int(k)
    assert len(st['games']) == k - 1, f'games played so far: {len(st["games"])}'
    gid = f'{mid}g{k}'
    d = f'{W}/match/{mid}/g{k}/decks'
    os.makedirs(d, exist_ok=True)
    baseA, baseB = f'{DECKS}/alurentell.txt', f'{DECKS}/{st["opp"]}.txt'
    for tag, base in (('a', baseA), ('b', baseB)):
        shutil.copy(base, f'{d}/{tag}-main.txt')
        sw = f'{W}/match/{mid}/board{tag.upper()}.txt'
        if k == 1 or not os.path.exists(sw):
            shutil.copy(base, f'{d}/{tag}.txt')
        else:
            main, sd, hdr = board(base, sw)
            write(f'{d}/{tag}.txt', main, sd, hdr, f'boarded for game {k}')
            shutil.copy(sw, f'{W}/match/{mid}/g{k}/board{tag.upper()}.txt')
    first = first_of(m, mid, k)
    a_seat = 0 if first == 'A' else 1
    seed = 5000 + 10 * mnum(mid) + k
    env = dict(os.environ, LLM_DECKS=d, LLM_DECK='a', LLM_OPP='b', LLM_EXTRA='--expect-opp b-main --expect-llm a-main')
    subprocess.run([f'{W}/lg.sh', 'launch2', gid, str(seed), str(a_seat), '0'], env=env, check=True)
    subprocess.run(['python3', f'{W}/mkmeta.py', 'add', gid, str(seed), str(a_seat), '0', 'alurentell', st['opp']], check=True)
    st['games'].append({'k': k, 'id': gid, 'first': first, 'seed': seed, 'result': None})
    save(m)
    print(f'{gid}: {first} plays first. A (Alurentell) tag {gid}a, B ({st["opp"]}) tag {gid}b. Boarded lists: {d}/a.txt, b.txt')

def cmd_result(mid, k):
    m = load(); st = m[mid]; k = int(k)
    g = st['games'][k - 1]
    log = f'{W}/runs/{g["id"]}/truth/log.txt'
    last = open(log).read().strip().split('\n')[-1] if os.path.exists(log) else ''
    if last.startswith('YOU WON'): g['result'] = 'A'
    elif last.startswith('YOU LOST'): g['result'] = 'B'
    elif last.startswith('DRAW'): g['result'] = 'D'
    else:
        print('game not finished'); sys.exit(1)
    save(m)
    subprocess.run(['python3', f'{W}/mkmeta.py', 'results'], check=True)
    a = sum(1 for x in st['games'] if x['result'] == 'A'); b = sum(1 for x in st['games'] if x['result'] == 'B')
    print(f'{mid} game {k}: {"A (Alurentell)" if g["result"]=="A" else "B ("+st["opp"]+")" if g["result"]=="B" else "draw"} won. Score A {a} - B {b}.')
    if a == 2 or b == 2 or len(st['games']) >= 5:
        st['winner'] = 'A' if a > b else 'B' if b > a else 'D'; save(m)
        print('MATCH OVER, winner', st['winner'])
    else:
        print(f'next: game {k+1}, {first_of(m, mid, k+1)} plays first. Players write boardA.txt / boardB.txt first (checked with: matchctl.py check {mid} A|B).')

def cmd_status(mid=None):
    m = load()
    for i, st in m.items():
        if mid and i != mid: continue
        print(i, st['opp'], 'G1 first', st['first_g1'], [(g['id'], g['first'], g['result']) for g in st['games']], st.get('winner', ''))

if __name__ == '__main__':
    c, *a = sys.argv[1:]
    {'new': cmd_new, 'check': cmd_check, 'launch': cmd_launch, 'result': cmd_result, 'status': cmd_status}[c](*a)
