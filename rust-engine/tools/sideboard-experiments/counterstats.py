#!/usr/bin/env python3
"""What did Alurentell's counterspells hit? (traced cells; sbtrace counters feature)
Usage: counterstats.py <mode> <cfg> <batch> [matchup ...]   Lists, per counter card of ours: games drawn, games cast, casts that countered something and what."""
import sys, glob, os, json, subprocess, collections
MODE,CFG,BT=sys.argv[1:4]; MS=sys.argv[4:] or ['ur','ub','uw','mirror','storm','tron']
SB='/home/claude/target-emr/release/sbtrace'
CARDS=['Force of Negation','Force of Will','Consign to Memory']
def feats(jf):
    out=jf[:-6]+'.cnt.jsonl'
    if not os.path.exists(out) or os.path.getmtime(out)<os.path.getmtime(jf):
        with open(out,'w') as o: subprocess.run([SB,jf],stdout=o,stderr=subprocess.DEVNULL)
    return [json.loads(l) for l in open(out)]
for m in MS:
    G=[]
    for jf in glob.glob(f'results/{MODE}/{m}/{CFG}-*.b{BT}.jsonl'):
        if '.cnt.' in jf or '.features.' in jf: continue
        G+=feats(jf)
    if not G: continue
    print(f"\n### vs {m} ({CFG}, {len(G)} games)")
    for c in CARDS:
        dr=[g for g in G if c in g['drawn']]
        if not dr: continue
        ca=[g for g in dr if c in g['cast']]
        hits=collections.Counter(); ncast=0; nhit=0
        for g in ca:
            ncast+=len(g['cast'][c])
            for x in g['counters']:
                if x['by']==c and x['victim_seat']==1: hits[x['victim']]+=1; nhit+=1
        w=lambda L: f"{100*sum(g['winner']==0 for g in L)/len(L):.0f}%" if L else '-'
        nd=[g for g in G if c not in g['drawn']]
        print(f"- {c}: drawn {len(dr)}, cast in {len(ca)} ({ncast} casts), countered something {nhit}x; win% drawn {w(dr)} / cast {w(ca)} / never drawn {w(nd)}")
        print(f"    hit: {dict(hits.most_common())}")
    # what the opponent's counters hit of ours
    ours=collections.Counter(); 
    for g in G:
        for x in g['counters']:
            if x['victim_seat']==0: ours[(x['by'] or '(ability)',x['victim'])]+=1
    if ours: print('  opponent countered our:', dict(ours.most_common(8)))
