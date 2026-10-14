#!/usr/bin/env python3
"""Aggregate endingspot output. Usage: analyze.py spots-*.jsonl"""
import sys, json, math, collections
ARMS=('cast','wait','never','free','late1','late2')
rows=[]
for f in sys.argv[1:]:
    for l in open(f):
        l=l.strip()
        if l: rows.append(json.loads(l))
feas=[r for r in rows if r.get('feasible')]
def myturn(r):
    return (r['turn']+1)//2 if r['first']==0 else r['turn']//2
def p(r,a):
    x=r['arms'][a]; return x['wins']/x['n'] if x['n'] else None
def feats(r):
    h=r['hand']; bf=r['bf']
    r['myturn']=myturn(r)
    r['ncre']=len(r['ocreatures'])
    r['opow']=sum(c['p'] for c in r['ocreatures'])
    r['snt']='Show and Tell' in h
    r['aluren']=('Aluren' in h) or ('Aluren' in bf)
    r['acer']='Acererak the Archlich' in h
    r['atraxa']='Atraxa, Grand Unifier' in h
    r['combo']=r['snt'] or (r['aluren'] and r['acer'])
    r['clock']= r['life']/max(r['opow'],1)
    r['pc']={a:p(r,a) for a in ARMS}
    cs=[c for c in r['cast_opts']]
    r['snt_now']='Cast Show and Tell' in cs
    r['snt_after']='Cast Show and Tell' in r['after_opts']
    r['aluren_now']='Cast Aluren' in cs
    r['aluren_after']='Cast Aluren' in r['after_opts']
    r['atraxa_now']='Cast Atraxa, Grand Unifier' in cs
    r['atraxa_after']='Cast Atraxa, Grand Unifier' in r['after_opts']
for r in feas: feats(r)
def stat(rs,a,b):
    """mean of p_a - p_b over spots, cluster (game) SE"""
    by=collections.defaultdict(list)
    for r in rs:
        by[(r['matchup'],r['game'])].append(r['pc'][a]-r['pc'][b])
    n=sum(len(v) for v in by.values())
    if n==0: return None
    mean=sum(sum(v) for v in by.values())/n
    # cluster-robust variance of ratio estimator
    resid=[sum(x-mean for x in v) for v in by.values()]
    var=sum(e*e for e in resid)/(n*n)*len(by)/max(len(by)-1,1)
    return n,len(by),mean*100,math.sqrt(var)*100
def table(title,rs,key,order=None,arms=None):
    print(f"\n### {title}")
    print("| bucket | spots (games) | P(win) cast | late1 | late2 | wait | never | free | cast−never | cast−late1 | cast−late2 |")
    print("|---|---|---|---|---|---|---|---|---|---|---|")
    groups=collections.defaultdict(list)
    for r in rs: groups[key(r)].append(r)
    for k in (order or sorted(groups)):
        g=groups.get(k,[])
        if not g: continue
        m={a:sum(r['pc'][a] for r in g)/len(g)*100 for a in ARMS}
        s1=stat(g,'cast','never'); s2=stat(g,'cast','late1'); s3=stat(g,'cast','late2')
        print(f"| {k} | {len(g)} ({s1[1]}) | {m['cast']:.1f} | {m['late1']:.1f} | {m['late2']:.1f} | {m['wait']:.1f} | {m['never']:.1f} | {m['free']:.1f} | {s1[2]:+.1f} ±{1.96*s1[3]:.1f} | {s2[2]:+.1f} ±{1.96*s2[3]:.1f} | {s3[2]:+.1f} ±{1.96*s3[3]:.1f} |")
print(f"{len(rows)} spots found, {len(feas)} with a feasible exile")
table("All feasible spots",feas,lambda r:'all')
def bucket(x,cuts,labels):
    for c,l in zip(cuts,labels):
        if x<=c: return l
    return labels[-1]
table("By number of opposing creatures",feas,lambda r:'1' if r['ncre']==1 else '2' if r['ncre']==2 else '3+')
table("By my life",feas,lambda r:bucket(r['life'],[5,10,15],['<=5','6-10','11-15','16+']),['<=5','6-10','11-15','16+'])
table("By my turn number",feas,lambda r:bucket(r['myturn'],[3,6],['T1-3','T4-6','T7+']),['T1-3','T4-6','T7+'])
table("Combo piece in hand (Show and Tell, or Aluren+Acererak)",feas,lambda r:'combo in hand' if r['combo'] else 'no combo piece')
table("Opposing creatures x life",feas,lambda r:(('1' if r['ncre']==1 else '2+'),bucket(r['life'],[7],['life<=7','life 8+'])))
table("Creatures x turn",feas,lambda r:(('1' if r['ncre']==1 else '2+'),bucket(r['myturn'],[4],['T<=4','T5+'])))
table("Clock: my life / their total power",feas,lambda r:bucket(r['clock'],[2,4,8],['<=2','2-4','4-8','8+']),['<=2','2-4','4-8','8+'])
mc=[r for r in feas]
print("\nMCTS bot cast rate at these spots:",sum(r['mcts_cast'] for r in mc)/len(mc))

table("Show and Tell castable now, and still castable after Ending (mana for both)",feas,lambda r:'S&T both' if (r['snt_now'] and r['snt_after']) else 'S&T only without Ending' if r['snt_now'] else 'no S&T castable now')
table("Same, life <=7 only",[r for r in feas if r['life']<=7],lambda r:'S&T both' if (r['snt_now'] and r['snt_after']) else 'S&T only without Ending' if r['snt_now'] else 'no S&T castable now')
