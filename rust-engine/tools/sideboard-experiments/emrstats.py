#!/usr/bin/env python3
"""Emrakul usage and results from traced emr cells (features from the Emrakul build of sbtrace). Usage: emrstats.py"""
import glob, json, os, subprocess
E='Emrakul, the Aeons Torn'; A='Atraxa, Grand Unifier'; SBE='/home/claude/target-emr/release/sbtrace'
def load(mode,m,c):
    G=[]
    for jf in sorted(x for x in glob.glob(f'results/{mode}/{m}/{c}-*.b1.jsonl') if not x.endswith('.features.jsonl')):
        out=jf[:-6]+'.features.jsonl'
        if not os.path.exists(out) or os.path.getmtime(out)<os.path.getmtime(jf):
            with open(out,'w') as o: subprocess.run([SBE,jf],stdout=o,check=True)
        G+=[json.loads(l) for l in open(out)]
    return G
pc=lambda L: f"{100*sum(g['winner']==0 for g in L)/len(L):.0f}% of {len(L)}" if L else '-'
print('| Matchup | plan | games | Emrakul drawn | put in by Show and Tell | cast (Omniscience) | wins: S&T put Emrakul only | wins: S&T put Atraxa only | wins: Emrakul never drawn |')
print('|---|---|---|---|---|---|---|---|---|')
for m,mode in [('ur','nn'),('ub','nn'),('uw','nn'),('tron','nn'),('storm','nn'),('boros','nr')]:
    for c in ['emr1a','emr61']:
        G=load(mode,m,c)
        if not G: continue
        dr=[g for g in G if E in g['drawn']]; ent=[g for g in G if E in g['entered']]; ca=[g for g in G if E in g['cast']]
        eo=[g for g in G if E in g['entered'] and A not in g['entered']]; ao=[g for g in G if A in g['entered'] and E not in g['entered']]
        print(f"| {m} | {c} | {len(G)} | {len(dr)} | {len(ent)} | {len(ca)} | {pc(eo)} | {pc(ao)} | {pc([g for g in G if E not in g['drawn']])} |")
