#!/usr/bin/env python3
"""Round-2 tables: win rate (paired against the base cell of the same batch) plus trace metrics, per matchup.
Usage: round2.py <group: A|B|C|E> [matchup ...]   (play and draw pooled; paired differences are averaged over the two sides)"""
import sys, glob, os, csv, json, math, subprocess, collections
G=sys.argv[1]; MS=sys.argv[2:]
names={'ur':'UR Cutter','ub':'UB (Dimir)','uw':'UW Phelia','mirror':'Mirror','boros':'Boros','storm':'Storm','tron':'Tron','reanimator':'Reanimator'}
GROUPS={'A':lambda c:c[:2] in('pk','pu','fk','fu','pf') and c[2:3].isalnum() and len(c)<=3,
        'B':lambda c:c[:2] in('ch','en') and len(c)<=4,
        'C':lambda c:c.startswith('lt-'),
        'E':lambda c:c.startswith('emrg1-')}
BASE={'A':'basetr','B':'basetr','C':'basetr','E':'emrg1-base'}[G]
sys.argv=[sys.argv[0],'nn']  # metrics.py/analyze helpers are not importable as modules; reimplement the small parts
SBE='/home/claude/target-emr/release/sbtrace'
LAND2={'Ancient Tomb','City of Traitors'}
LANDS=LAND2|{'Boseiju, Who Endures','Flooded Strand','Hedge Maze','Island','Misty Rainforest','Savannah','Tundra','Tropical Island'}
PAYOFF={'Atraxa, Grand Unifier','Omniscience','Emrakul, the Aeons Torn'}
BLUE={'Brainstorm','Ponder','Stock Up','Show and Tell','Atraxa, Grand Unifier','Force of Will','Force of Negation','Consign to Memory'}
PROT=['Veil of Summer',"Orim's Chant",'Consign to Memory','Force of Negation']
def feats(jf):
    out=jf[:-6]+'.features.jsonl'
    if not os.path.exists(out) or os.path.getmtime(out)<os.path.getmtime(jf):
        with open(out,'w') as o: subprocess.run([SBE,jf],stdout=o,stderr=subprocess.DEVNULL)
    return [json.loads(l) for l in open(out)]
def mana(s):
    m=sum((2 if k in LAND2 else 1)*n for k,n in s['bf'].items() if k in LANDS)
    hl=[2 if k in LAND2 else 1 for k in s['hand'] if k in LANDS]
    m+=max(hl) if hl else 0
    return m+s['bf'].get('Lotus Petal',0)+min(s['hand'].get('Lotus Petal',0),2)
def prot(s):
    h=s['hand']; p=sum(h.get(c,0) for c in PROT)
    if h.get('Force of Will',0) and (sum(n for k,n in h.items() if k in BLUE)-1>=1 or mana(s)>=5): p+=1
    return p
def lines(s):
    h=s['hand']; bf=s['bf']; m=mana(s)
    snt=min(h.get('Show and Tell',0),sum(h.get(c,0) for c in PAYOFF)) if m>=3 else 0
    alu=1 if (bf.get('Aluren',0)>0 or (h.get('Aluren',0)>0 and m>=3)) and h.get('Acererak the Archlich',0)>0 else 0
    return snt,alu
def game(g):
    r={'win':1.0 if g['winner']==0 else 0.0,'first':None,'prot':None,'life':None,'m4':None,'life4':None,'m3':None}
    for s in g['snaps']:
        t=s['t']; sn,al=lines(s)
        if t==3: r['m3']=mana(s)
        if t==4: r['m4']=mana(s); r['life4']=s.get('life')
        if r['first'] is None and sn+al>=1: r['first']=t; r['prot']=prot(s); r['life']=s.get('life')
    return r
def load(m,c,side):
    d={}; gs=[]
    for jf in sorted(glob.glob(f'results/nn/{m}/{c}-{side}.b1.jsonl')):
        for g in feats(jf): gs.append(game(g)); d[g['match']]=gs[-1]['win']
    if not d:  # untraced base
        p=f'results/nn/{m}/{c}-{side}.b1.csv'
        if os.path.exists(p):
            for r in csv.DictReader(open(p)): d[int(r['match'])]={'A':1.0,'B':0.0}.get(r['winner'],0.5)
    return d,gs
def mean(L):
    L=[x for x in L if x is not None]; return sum(L)/len(L) if L else float('nan')
def diff(a,b):
    ks=sorted(set(a)&set(b)); n=len(ks)
    z=[a[k]-b[k] for k in ks]; mu=sum(z)/n; v=sum((x-mu)**2 for x in z)/(n-1)
    return mu,math.sqrt(v/n)
for m in (MS or ['ur','uw','ub','mirror','storm','tron']):
    cs=sorted({os.path.basename(p)[:-4].rsplit('.',1)[0].rsplit('-',1)[0] for p in glob.glob(f'results/nn/{m}/*.b1.log')})
    cs=[c for c in cs if GROUPS[G](c) or c==BASE]
    if BASE not in cs: continue
    if not all(os.path.exists(f'results/nn/{m}/{c}-{s}.b1.log') for c in cs for s in('play','draw')): cs=[c for c in cs if all(os.path.exists(f'results/nn/{m}/{c}-{s}.b1.log') for s in('play','draw'))]
    if BASE not in cs: continue
    B={s:load(m,BASE,s) for s in('play','draw')}
    print(f'\n### {names[m]}\n')
    print(f'| Config | games/side | win | vs {BASE} (±95%) | line by T4 | mean first-line turn | life at first line | mana T4 | protection at first line |')
    print('|---|---|---|---|---|---|---|---|---|')
    rows=[]
    for c in sorted(cs,key=lambda c:(c!=BASE,c)):
        L={s:load(m,c,s) for s in('play','draw')}
        n=min(len(L['play'][0]),len(L['draw'][0]))
        win=mean([mean(list(L[s][0].values())) for s in L])
        if c==BASE: dt='-'
        else:
            d1,s1=diff(L['play'][0],B['play'][0]); d2,s2=diff(L['draw'][0],B['draw'][0])
            dt=f'{100*(d1+d2)/2:+.1f} (±{100*1.96*math.sqrt(s1*s1+s2*s2)/2:.1f})'
        gs=L['play'][1]+L['draw'][1]
        by4=f"{100*sum(1 for g in gs if g['first'] is not None and g['first']<=4)/len(gs):.0f}%" if gs else '-'
        f=lambda k: f"{mean([g[k] for g in gs]):.2f}" if gs else '-'
        print(f"| {c} | {n} | {100*win:.1f}% | {dt} | {by4} | {f('first')} | {f('life')} | {f('m4')} | {f('prot')} |")
