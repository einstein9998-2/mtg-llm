#!/usr/bin/env python3
"""What happened to a card once it was drawn (traced cells, sbtrace features).
Usage: cardfate.py <mode> <cfg> <batch> "<Card>" [matchup ...]
Buckets, for every game where the card was drawn by Alurentell:
  cast                  - cast at least once
  no white mana         - never cast; at no own main phase after the draw was a white source (Tundra, Savannah, Lotus Petal, Carpet) on the battlefield
                          or a Tundra/Savannah in hand (so the card could not be paid for)
  late                  - never cast; drawn on the game's last own turn or the one before
  held with mana        - never cast; a white source and at least 1 mana were available on some main phase, and the game went on (the net chose not to / no target)
Win rates are Alurentell's in that bucket."""
import sys, glob, os, json, subprocess, collections
MODE,CFG,BT,CARD=sys.argv[1:5]; MS=sys.argv[5:] or ['ur','ub','uw','boros','mirror','storm','tron','reanimator']
SB='/home/claude/target-live/release/sbtrace'; SBE='/home/claude/target-emr/release/sbtrace'
W_BF={'Tundra','Savannah','Lotus Petal','Carpet of Flowers'}; W_HAND={'Tundra','Savannah'}
names={'ur':'UR Cutter','ub':'UB (Dimir)','uw':'UW Phelia','mirror':'Mirror','boros':'Boros','storm':'Storm','tron':'Tron','reanimator':'Reanimator'}
def feats(jf):
    out=jf[:-6]+'.features.jsonl'
    if not os.path.exists(out) or os.path.getmtime(out)<os.path.getmtime(jf):
        with open(out,'w') as o: subprocess.run([SBE if '/emr' in jf else SB,jf],stdout=o,check=True)
    return [json.loads(l) for l in open(out)]
def bucket(g):
    if CARD in g['cast']: return 'cast'
    d=min(g['drawn'][CARD]); last=g['a_turns']
    snaps=[s for s in g['snaps'] if s['t']>=d]
    if d>=last-1: return 'late'
    ok=any(any(k in s['bf'] for k in W_BF) or any(k in s['hand'] for k in W_HAND) for s in snaps)
    return 'held with mana' if ok else 'no white mana'
for m in MS:
    G=[]
    for jf in glob.glob(f'results/{MODE}/{m}/{CFG}-*.b{BT}.jsonl'): G+=feats(jf)
    if not G: continue
    dr=[g for g in G if CARD in g['drawn']]
    if not dr: continue
    B=collections.defaultdict(list)
    for g in dr: B[bucket(g)].append(g)
    nd=[g for g in G if CARD not in g['drawn']]
    w=lambda L: f"{len(L)} games, {100*sum(g['winner']==0 for g in L)/len(L):.0f}% win" if L else '-'
    print(f"\n### {CARD} vs {names[m]} ({CFG}, {len(G)} games; drawn in {len(dr)})")
    for b in ['cast','held with mana','no white mana','late']: print(f"- {b}: {w(B[b])}")
    print(f"- never drawn: {w(nd)}")
