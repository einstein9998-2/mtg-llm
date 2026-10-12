#!/usr/bin/env python3
"""Reads results/<matchup>/<config>-<play|draw>.csv (one row per single game, A = Alurentell) and prints tables."""
import csv, glob, os, math, sys, collections
R='results'
def load(path):
    d={}
    for r in csv.DictReader(open(path)):
        d[int(r['match'])]={'A':1.0,'B':0.0}.get(r['winner'],0.5)  # a drawn/panicked game counts half
    return d
cells={}
MODE=sys.argv[1] if len(sys.argv)>1 else 'nn'
for p in sorted(glob.glob(f'{R}/{MODE}/*/*-*.csv')):
    m=os.path.basename(os.path.dirname(p)); b=os.path.basename(p)[:-4]; cs,bt=b.rsplit('.',1); c,s=cs.rsplit('-',1)
    for k,v in load(p).items(): cells.setdefault((m,c,s),{})[(bt,k)]=v
def stat(d):
    n=len(d); w=sum(d.values()); p=w/n if n else float('nan')
    return n,w,p,math.sqrt(p*(1-p)/n) if n else float('nan')
def pct(x): return f'{100*x:5.1f}'
def diff(a,b):
    """paired difference a-b on the games both cells have (same batch and match index = same shuffles)"""
    ks=sorted(set(a)&set(b)); n=len(ks)
    if n<2: return float('nan'),float('nan'),n
    z=[a[k]-b[k] for k in ks]; m=sum(z)/n; v=sum((x-m)**2 for x in z)/(n-1)
    return m,math.sqrt(v/n),n
def unpaired(a,b):
    na,wa,pa,sa=stat(a); nb,wb,pb,sb=stat(b)
    return pa-pb, math.sqrt(sa*sa+sb*sb)
MS=[m for m in ['ur','ub','uw','mirror','boros','storm','tron','reanimator'] if os.path.isdir(f'{R}/{MODE}/{m}')]
names={'ur':'UR Cutter','ub':'UB (Dimir)','uw':'UW Phelia','mirror':'Mirror','boros':'Boros','storm':'Storm','tron':'Tron','reanimator':'Reanimator'}
def have(m,c): return (m,c,'play') in cells and (m,c,'draw') in cells
out=[]
P=out.append
P('## Play vs draw, main decks (game one, nobody boarded)\n')
P('| Matchup | Alurentell on the play | on the draw | play minus draw | games per side |')
P('|---|---|---|---|---|')
for m in MS:
    if not have(m,'g1main'): continue
    a,b=cells[(m,'g1main','play')],cells[(m,'g1main','draw')]
    d,se,_=diff(a,b)
    na=stat(a)[0]; nb=stat(b)[0]
    P(f'| {names[m]} | {pct(stat(a)[2])}% | {pct(stat(b)[2])}% | **{100*d:+.1f}** (±{100*1.96*se:.1f}) | {na} / {nb} |')
P('\n## Play vs draw, boarded (base plan, both seats boarded)\n')
P('| Matchup | on the play | on the draw | play minus draw | games per side |')
P('|---|---|---|---|---|')
for m in MS:
    if not have(m,'base'): continue
    a,b=cells[(m,'base','play')],cells[(m,'base','draw')]
    d,se,_=diff(a,b)
    P(f'| {names[m]} | {pct(stat(a)[2])}% | {pct(stat(b)[2])}% | **{100*d:+.1f}** (±{100*1.96*se:.1f}) | {stat(a)[0]} / {stat(b)[0]} |')
P('\n## Sideboard configurations (win rate of Alurentell, boarded game; "avg" = mean of play and draw)\n')
for m in MS:
    cs=sorted({c for (mm,c,s) in cells if mm==m and c!='g1main' and have(m,c)}, key=lambda c:(c not in('none','base'), c!='none' and c!='base', c))
    if not cs: continue
    P(f'### {names[m]}\n')
    P('| Config | on the play | on the draw | avg | avg minus base (paired, ±95%) | games per side |')
    P('|---|---|---|---|---|---|')
    for c in cs:
        a,b=cells[(m,c,'play')],cells[(m,c,'draw')]
        avg=(stat(a)[2]+stat(b)[2])/2
        if have(m,'base') and c!='base':
            d1,s1,_=diff(a,cells[(m,'base','play')]); d2,s2,_=diff(b,cells[(m,'base','draw')])
            dd=(d1+d2)/2; se=math.sqrt(s1*s1+s2*s2)/2
            dtxt=f'{100*dd:+.1f} (±{100*1.96*se:.1f})'
        else: dtxt='-'
        P(f'| {c} | {pct(stat(a)[2])}% | {pct(stat(b)[2])}% | {pct(avg)}% | {dtxt} | {stat(a)[0]} / {stat(b)[0]} |')
    P('')
print('\n'.join(out))
