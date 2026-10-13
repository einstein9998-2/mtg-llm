#!/usr/bin/env python3
"""Round 2 (2026-10-12): generates plans/<m>/<cfg>/ and deck dirs, and the queue files q12a..q12e.
Every config is a delta on the post-board 60 of Brady's plan (post(m)), recomputed against the main 60 by gen2.emit.
 A  Petal / Force of Will over slower cards (Stock Up = k, Aluren = u) vs the tempo decks
 B  Orim's Chant / Prismatic Ending counts (0..3)
 C  land trims (18-19 lands), the freed slot filled with a slow card
 E  Emrakul, the Aeons Torn swapped for each nonland card of the unboarded main 60 (a game one: no boarding)"""
import os, shutil, collections, sys
from gen2 import *
TEMPO = ('ur', 'uw', 'ub')
LAND = {'I': 'Island', 'H': 'Hedge Maze', 'S': 'Flooded Strand'}
PRIO = {  # slow cards that take a freed slot, in order of preference (availability is checked)
    'ur': [AL, P, F], 'uw': [AL, P, F], 'ub': [P, F, OM], 'mirror': [SH, P], 'storm': [SU, AT, AC], 'tron': [OM, VE],
}
def cap(c, side):  # most copies of c a 60 can hold: main + sideboard
    return MAIN[c] + side[c]
def fill(m, fin, n, side, prio=None):
    """n cards from the matchup's priority list, within availability. returns list of (card, +k)"""
    out = []
    for c in (prio or PRIO[m]):
        room = cap(c, side) - fin[c] - sum(k for cc, k in out if cc == c)
        k = min(room, n)
        if k > 0: out.append((c, k)); n -= k
        if n == 0: return out
    raise AssertionError((m, 'no room', n))
def final_of(m, deltas, side=SIDE):
    fin = collections.Counter(post(m))
    for c, n in deltas: fin[c] += n
    assert all(v >= 0 for v in fin.values()), (m, deltas)
    for c in fin: assert fin[c] <= cap(c, side), (m, c, fin[c])
    return fin
specs = {}      # m -> name -> (deltas, deckedits or None)
queues = {k: [] for k in 'abcde'}
def add(m, name, deltas, q, n, deck=None):
    specs.setdefault(m, {})[name] = (deltas, deck)
    queues[q].append((m, name, n))
# ---------------- A ----------------
for m in TEMPO:
    for nm, (c, k, d) in {
        'pk1': (P, SU, 1), 'pk2': (P, SU, 2), 'pu1': (P, AL, 1), 'pu2': (P, AL, 2),
        'fk1': (F, SU, 1), 'fk2': (F, SU, 2), 'fu1': (F, AL, 1), 'fu2': (F, AL, 2)}.items():
        add(m, nm, D((c, d), (k, -d)), 'a', 400)
    add(m, 'pfk', D((P, 1), (F, 1), (SU, -2)), 'a', 400)
    add(m, 'pfm', D((P, 1), (F, 1), (SU, -1), (AL, -1)), 'a', 400)
# ---------------- B ----------------
for m in TEMPO:
    for nm, deltas in {
        'ch1p': D((CH, -1), (P, 1)), 'ch0p': D((CH, -2), (P, 2)), 'ch0f': D((CH, -2), (F, 2)),
        'en1p': D((EN, -1), (P, 1)), 'en0p': D((EN, -2), (P, 2)), 'en0f': D((EN, -2), (F, 2))}.items():
        add(m, nm, deltas, 'b', 400)
    add(m, 'ch3k', D((CH, 1), (SU, -1)), 'b', 400, deck={CH: +1, MA: -1})
    add(m, 'en3k', D((EN, 1), (SU, -1)), 'b', 400, deck={EN: +1, MA: -1})
for m in ('storm', 'mirror'):
    side = SIDE
    for nm, n in (('ch1', 1), ('ch0', 2)):
        f = collections.Counter(post(m)); f[CH] -= n
        add(m, nm, D((CH, -n)) + fill(m, f, n, side, [SU, AT, AC] if m == 'storm' else [P, SH]), 'b', 400)
# ---------------- C ----------------
CUTS = {'I': [('I', 1)], 'H': [('H', 1)], 'S': [('S', 1)], 'IH': [('I', 1), ('H', 1)], 'HH': [('H', 2)],
        'HS': [('H', 1), ('S', 1)], 'SS': [('S', 2)]}
for m in ('ur', 'uw', 'ub', 'mirror', 'storm', 'tron'):
    have = collections.Counter(post(m))
    names = ['I', 'H', 'S', 'IH', 'HH'] if have[LAND['I']] else ['H', 'S', 'HH', 'HS', 'SS']
    for nm in names:
        cut = [(LAND[l], n) for l, n in CUTS[nm]]
        f = collections.Counter(have)
        for c, n in cut: f[c] -= n
        add(m, 'lt-' + nm, [(c, -n) for c, n in cut] + fill(m, f, sum(n for _, n in cut), SIDE), 'c', 350)
# ---------------- E ----------------
E_CARDS = {'acer': AC, 'atra': AT, 'brain': 'Brainstorm', 'fow': F, 'ponder': 'Ponder', 'snt': SH, 'stock': SU, 'veil': VE,
           'aluren': AL, 'petal': P, 'omni': OM}
def write_main_deck(tag, main_edits):
    d = f'decks-{tag}'; shutil.rmtree(d, ignore_errors=True); shutil.copytree('decks', d)
    lines = open('decks/alurentell.txt').read().rstrip('\n').split('\n')
    blanks = [i for i, l in enumerate(lines) if not l.strip()]
    lo, hi = blanks[0] + 1, blanks[1]          # the main-deck block
    main = []
    for l in lines[lo:hi]:
        n, name = l.split(' ', 1); n = int(n) + main_edits.get(name, 0)
        if n > 0: main.append(f'{n} {name}')
    for name, dlt in main_edits.items():
        if dlt > 0 and not any(l.endswith(' ' + name) for l in main): main.append(f'{dlt} {name}')
    open(f'{d}/alurentell.txt', 'w').write('\n'.join(lines[:lo] + main + lines[hi:]) + '\n')
    return d
E_MATCH = {'ur': 400, 'uw': 400, 'ub': 400, 'tron': 400, 'storm': 400}
if __name__ == '__main__':
    for m, cf in specs.items():
        d = M[m]
        for name, (deltas, deck) in cf.items():
            side = collections.Counter(SIDE)
            if deck:
                for c, n in deck.items(): side[c] += n
            fin = final_of(m, deltas, side)
            p = f'plans/{m}/{name}'
            emit(f'{p}/alurentell.txt', d['vs'], fin, side)
            src = f'oppplans/{d["opp"]}.txt'
            if d['opp'] != 'alurentell' and os.path.exists(src): shutil.copy(src, f'{p}/{d["opp"]}.txt')
            if deck: write_deck(f'{m}-{name}', deck)
            print(m, name, deltas)
    # E: decks only (no plans are read: the run is unboarded)
    for m in E_MATCH:
        for nm in ['base'] + list(E_CARDS):
            tag = f'{m}-emrg1-{nm}'
            edits = {} if nm == 'base' else {E_CARDS[nm]: -1}
            if nm != 'base': edits[EMR] = +1
            write_main_deck(tag, edits)
            # the base config must hold the same deck as the unboarded main: still needs Emrakul-capable nets (nets-emr)
    # queues
    order = ['a', 'b', 'c']
    for q in 'abc':
        with open(f'q12{q}.txt', 'w') as f:
            for m, name, n in queues[q]:
                for side in ('play', 'draw'): f.write(f'nn {m} {name} {side} {n} 1\n')
    with open('q12e.txt', 'w') as f:
        for m in ('ur', 'uw', 'ub', 'tron', 'storm'):
            for nm in ['base'] + list(E_CARDS):
                for side in ('play', 'draw'): f.write(f'nn {m} emrg1-{nm} {side} 400 1\n')
    for q in 'abce': print(q, sum(1 for _ in open(f'q12{q}.txt')))
