#!/usr/bin/env python3
"""Per-card 'drawn and cast / drawn but never cast / not drawn' win rates from traced cells (sbtrace features).
Usage: tracestats.py <mode> [cfg=base] [batch=2]"""
import sys, glob, os, subprocess, json, math, collections
MODE=sys.argv[1]; CFG=sys.argv[2] if len(sys.argv)>2 else 'base'; BT=sys.argv[3] if len(sys.argv)>3 else '2'
SB='/home/claude/target-emr/release/sbtrace'; SBE='/home/claude/target-emr/release/sbtrace'
CARDS=["Orim's Chant","Prismatic Ending","Consign to Memory","Force of Negation","Veil of Summer","Carpet of Flowers","Faerie Macabre","Force of Will","Lotus Petal","Omniscience","Aluren","Show and Tell"]
names={'ur':'UR Cutter','ub':'UB (Dimir)','uw':'UW Phelia','mirror':'Mirror','boros':'Boros','storm':'Storm','tron':'Tron','reanimator':'Reanimator'}
def ci(w,n):
    if n==0: return '-'
    p=w/n; return f'{100*p:.0f}% ({w}/{n})'
for m in ['ur','ub','uw','boros','mirror','reanimator','storm','tron']:
    fs=glob.glob(f'results/{MODE}/{m}/{CFG}-*.b{BT}.jsonl')
    if not fs: continue
    games=[]
    for f in fs:
        feat=f[:-6]+'.features.jsonl'
        if not os.path.exists(feat) or os.path.getmtime(feat)<os.path.getmtime(f):
            with open(feat,'w') as o: subprocess.run([SBE if '/emr' in f else SB,f],stdout=o,stderr=subprocess.DEVNULL)
        games+=[json.loads(l) for l in open(feat)]
    n=len(games); w=sum(g['winner']==0 for g in games)
    print(f'\n### {names[m]} ({CFG}, {n} games, Alurentell won {100*w/n:.1f}%)\n')
    print('| Card | drawn | cast when drawn | wins: drawn+cast | wins: drawn, never cast | wins: not drawn |')
    print('|---|---|---|---|---|---|')
    for c in CARDS:
        dr=[g for g in games if c in g['drawn']]
        if not dr and not any(c in g['cast'] for g in games): continue
        ca=[g for g in dr if c in g['cast']]; nc=[g for g in dr if c not in g['cast']]
        nd=[g for g in games if c not in g['drawn']]
        f=lambda L: ci(sum(g['winner']==0 for g in L),len(L))
        print(f'| {c} | {100*len(dr)/n:.0f}% | {len(ca)}/{len(dr)} | {f(ca)} | {f(nc)} | {f(nd)} |')
