#!/usr/bin/env python3
"""Generates plans/<matchup>/<config>/ for the leave-one-group-out sideboard experiment."""
import os, shutil, collections
REPO='/home/claude/mtg-llm/rust-engine'
def parse(path):
    plans={};cur=None
    for l in open(path):
        l=l.split('#')[0].strip()
        if not l: continue
        if l.startswith('vs '): cur=l[3:].strip(); plans[cur]=collections.Counter(); continue
        n,name=l[1:].strip().split(' ',1); plans[cur][name.strip()]+= (1 if l[0]=='+' else -1)*int(n)
    return plans
base_all=parse(f'{REPO}/sideboard-plans/alurentell.txt')
base_all.update(parse(f'{REPO}/sideboard-pending/storm/alurentell-storm-plan.txt'))
base_all.update(parse(f'{REPO}/sideboard-pending/tron/alurentell-tron-plan.txt'))
G=lambda **k:k
M={
 'ur':   dict(opp='ur-cutter', vs='ur-cutter', net='ur',
   outs=[('Lotus Petal',4),('Omniscience',2),('Force of Will',2),('Boseiju, Who Endures',1),('Aluren',1)],
   groups={'Carpet':{'Carpet of Flowers':4},'Chant':{"Orim's Chant":2},'Ending':{'Prismatic Ending':2},'Veil':{'Veil of Summer':2}}),
 'ub':   dict(opp='dimir-tempo', vs='dimir-tempo', net='ub',
   outs=[('Lotus Petal',4),('Omniscience',2),('Force of Will',4)],
   groups={'Carpet':{'Carpet of Flowers':4},'Chant':{"Orim's Chant":2},'Ending':{'Prismatic Ending':2},'Veil':{'Veil of Summer':2}}),
 'uw':   dict(opp='uwx-control', vs='uwx-control', net='uwphelia',
   outs=[('Lotus Petal',4),('Omniscience',2),('Force of Will',2),('Aluren',2)],
   groups={'Carpet':{'Carpet of Flowers':4},'Chant':{"Orim's Chant":2},'Ending':{'Prismatic Ending':2},'Veil':{'Veil of Summer':2}}),
 'boros':dict(opp='boros-aggro', vs='boros-aggro', net='boros',
   outs=[('Veil of Summer',2),('Stock Up',1),('Aluren',1)],
   groups={'Chant':{"Orim's Chant":2},'Ending':{'Prismatic Ending':2}}),
 'mirror':dict(opp='alurentell', vs='alurentell', net='mirror',
   outs=[('Omniscience',2),('Show and Tell',4),('Lotus Petal',4),('City of Traitors',1)],
   groups={'Carpet':{'Carpet of Flowers':4},'Chant':{"Orim's Chant":2},'Veil':{'Veil of Summer':2},'ConsignFoN':{'Consign to Memory':2,'Force of Negation':1}}),
 'reanimator':dict(opp='reanimator', vs='reanimator', net='reanimator',
   outs=[('Omniscience',2),('Stock Up',1),('Show and Tell',3),('Island',1)],
   groups={'Veil':{'Veil of Summer':2},'Macabre':{'Faerie Macabre':2},'FoN':{'Force of Negation':1},'Chant':{"Orim's Chant":2}}),
 'storm':dict(opp='storm', vs='storm', net='storm',
   outs=[('Veil of Summer',2),('Island',1),('Stock Up',2),('Atraxa, Grand Unifier',1),('Acererak the Archlich',1)],
   groups={'Consign':{'Consign to Memory':2},'Chant':{"Orim's Chant":2},'Ending':{'Prismatic Ending':2},'FoN':{'Force of Negation':1}}),
 'tron': dict(opp='colorless-tron', vs='colorless-tron', net='tron',
   outs=[('Veil of Summer',2),('Island',1),('Omniscience',2)],
   groups={'Consign':{'Consign to Memory':2},'FoN':{'Force of Negation':1},'Ending':{'Prismatic Ending':2}}),
}
def flat(pairs): return [c for c,n in pairs for _ in range(n)]
def emit(path, vs, outs, ins):
    os.makedirs(os.path.dirname(path),exist_ok=True)
    with open(path,'w') as f:
        if outs or ins:
            f.write(f'vs {vs}\n')
            for c,n in collections.Counter(outs).items(): f.write(f'-{n} {c}\n')
            for c,n in collections.Counter(ins).items(): f.write(f'+{n} {c}\n')
def configs(m):
    d=M[m]; outs=flat(d['outs'])
    ins=[c for g in d['groups'].values() for c,n in g.items() for _ in range(n)]
    # the base plan must equal Brady's plan file
    b=base_all[d['vs']]
    got=collections.Counter(ins); got.subtract(collections.Counter(outs))
    assert +collections.Counter({c:n for c,n in b.items() if n>0})==+collections.Counter(ins), (m,'ins')
    assert collections.Counter({c:-n for c,n in b.items() if n<0})==collections.Counter(outs), (m,'outs')
    cf={'none':([],[]),'base':(outs,ins)}
    for gname,g in d['groups'].items():
        k=sum(g.values()); gi=[c for c,n in g.items() for _ in range(n)]
        ins2=list(ins)
        for c in gi: ins2.remove(c)
        cf['no'+gname]=(outs[:len(outs)-k],ins2)
    return cf, {'no'+g:flat(d['outs'])[len(outs)-sum(v.values()):] for g,v in d['groups'].items()}
if __name__=='__main__':
    shutil.rmtree('plans',ignore_errors=True)
    for m,d in M.items():
        cf,restored=configs(m)
        for c,(o,i) in cf.items():
            p=f'plans/{m}/{c}'
            emit(f'{p}/alurentell.txt', d['vs'], o, i)
            if d['opp']!='alurentell':
                src=f'oppplans/{d["opp"]}.txt'
                if os.path.exists(src): shutil.copy(src,f'{p}/{d["opp"]}.txt')
        print(m,{c:len(o) for c,(o,i) in cf.items()}, 'restored:',{k:dict(collections.Counter(v)) for k,v in restored.items()})
