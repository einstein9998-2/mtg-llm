#!/usr/bin/env python3
"""Goldfish-style metrics from traced cells (sbtrace features with per-turn snapshots of hand and permanents).
Usage: metrics.py <mode> [matchup ...]   -> markdown tables per matchup, one row per config (play and draw pooled)."""
import sys, glob, os, json, math, subprocess, collections
SB='/home/claude/target-emr/release/sbtrace'; SBE='/home/claude/target-emr/release/sbtrace'
MODE=sys.argv[1]; MS=sys.argv[2:] or ['ur','ub','uw','boros','mirror','reanimator','storm','tron']
LAND2={'Ancient Tomb','City of Traitors'}
LANDS=LAND2|{'Boseiju, Who Endures','Flooded Strand','Hedge Maze','Island','Misty Rainforest','Savannah','Tundra','Tropical Island'}
PAYOFF={'Atraxa, Grand Unifier','Omniscience','Emrakul, the Aeons Torn'}
BLUE={'Brainstorm','Ponder','Stock Up','Show and Tell','Atraxa, Grand Unifier','Force of Will','Force of Negation','Consign to Memory'}
PROT=['Veil of Summer',"Orim's Chant",'Consign to Memory','Force of Negation']
names={'ur':'UR Cutter','ub':'UB (Dimir)','uw':'UW Phelia','mirror':'Mirror','boros':'Boros','storm':'Storm','tron':'Tron','reanimator':'Reanimator'}
def feats(jf):
    out=jf[:-6]+'.features.jsonl'
    if not os.path.exists(out) or os.path.getmtime(out)<os.path.getmtime(jf):
        with open(out,'w') as o: subprocess.run([SBE if '/emr' in jf else SB,jf],stdout=o,stderr=subprocess.DEVNULL)
    return [json.loads(l) for l in open(out)]
def mana(s):
    m=sum((2 if k in LAND2 else 1)*n for k,n in s['bf'].items() if k in LANDS)
    hl=[2 if k in LAND2 else 1 for k in s['hand'] if k in LANDS]
    m+=max(hl) if hl else 0
    m+=s['bf'].get('Lotus Petal',0)+min(s['hand'].get('Lotus Petal',0),2)
    return m
def prot(s):
    h=s['hand']; p=sum(h.get(c,0) for c in PROT)
    f=h.get('Force of Will',0)
    if f and (sum(n for k,n in h.items() if k in BLUE)-1>=1 or mana(s)>=5): p+=1
    return p
def lines(s):
    h=s['hand']; bf=s['bf']; m=mana(s)
    snt=min(h.get('Show and Tell',0),sum(h.get(c,0) for c in PAYOFF)) if m>=3 else 0
    alu=1 if (bf.get('Aluren',0)>0 or (h.get('Aluren',0)>0 and m>=3)) and h.get('Acererak the Archlich',0)>0 else 0
    return snt,alu
def game(g):
    r={}
    r['win']=1 if g['winner']==0 else 0
    first=second=None; pr=None; lf=None; mana3=mana4=None
    for s in g['snaps']:
        t=s['t']; sn,al=lines(s)
        if t==3: mana3=mana(s)
        if t==4: mana4=mana(s)
        if first is None and sn+al>=1: first=t; pr=prot(s); fl=(sn,al); lf=s.get('life')
        if second is None and sn+al>=2: second=t
    r['life']=lf if first is not None else None; r['life4']=next((s.get('life') for s in g['snaps'] if s['t']==4),None)
    r['first']=first; r['second']=second; r['prot']=pr; r['m3']=mana3; r['m4']=mana4
    r['snt_first']=next((s['t'] for s in g['snaps'] if lines(s)[0]>=1),None)
    r['alu_first']=next((s['t'] for s in g['snaps'] if lines(s)[1]>=1),None)
    r['carpet']=next((s['t'] for s in g['snaps'] if s['bf'].get('Carpet of Flowers',0)>0),None)
    return r
def pc(x,n): return f'{100*x/n:.0f}%' if n else '-'
def mean(L): 
    L=[x for x in L if x is not None]; return f'{sum(L)/len(L):.2f}' if L else '-'
for m in MS:
    cfgs=collections.defaultdict(list)
    for jf in sorted(x for x in glob.glob(f'results/{MODE}/{m}/*.jsonl') if not x.endswith('.features.jsonl')):
        cs=os.path.basename(jf)[:-6].rsplit('.',1)[0]; c=cs.rsplit('-',1)[0]
        cfgs[c]+=[game(g) for g in feats(jf)]
    if not cfgs: continue
    print(f'\n### {names[m]}\n')
    print('| Config | games | win | line by T2 | T3 | T4 | T5 | mean first-line turn | two lines by T4 | mana T3 | mana T4 | life at first line | life T4 | protection at first line | Show and Tell line by T4 | Aluren line by T4 |')
    print('|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|')
    for c in sorted(cfgs, key=lambda c:(c!='base',c)):
        G=cfgs[c]; n=len(G)
        by=lambda k,key='first': sum(1 for g in G if g[key] is not None and g[key]<=k)
        print(f"| {c} | {n} | {pc(sum(g['win'] for g in G),n)} | {pc(by(2),n)} | {pc(by(3),n)} | {pc(by(4),n)} | {pc(by(5),n)} | {mean([g['first'] for g in G])} | {pc(by(4,'second'),n)} | {mean([g['m3'] for g in G])} | {mean([g['m4'] for g in G])} | {mean([g['life'] for g in G])} | {mean([g['life4'] for g in G])} | {mean([g['prot'] for g in G])} | {pc(by(4,'snt_first'),n)} | {pc(by(4,'alu_first'),n)} |")
