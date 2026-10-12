#!/usr/bin/env python3
"""Generates plans/<matchup>/<name>/ for Brady's specific tests (2026-10-11). Each config is a delta on the post-board 60 of
Brady's plan; the plan file is recomputed against the main 60. Deck variants (3 Consign, 2 Force of Negation) change the sideboard."""
import os, shutil, collections, re
from gen import parse, base_all, M, REPO
def deck(path):
    main=collections.Counter(); side=collections.Counter(); cur=main
    for l in open(path):
        s=l.split('#')[0].strip()
        if not s:
            if main: cur=side
            continue
        n,name=s.split(' ',1); cur[name.strip()]+=int(n)
    return main, side
MAIN,SIDE=deck(f'{REPO}/decks/alurentell.txt')
def post(m, side=SIDE):
    d=M[m]; b=base_all[d['vs']]; p=collections.Counter(MAIN)
    for c,n in b.items(): p[c]+=n
    return +p
def emit(path, vs, final, side, oversize=False):
    outs=collections.Counter(); ins=collections.Counter()
    for c in set(MAIN)|set(final):
        dlt=final[c]-MAIN[c]
        if dlt>0: ins[c]=dlt
        elif dlt<0: outs[c]=-dlt
    assert sum(final.values())==(61 if oversize else 60),(path,sum(final.values()))
    for c,n in ins.items(): assert n<=side[c],(path,'not enough in sideboard',c,n,side[c])
    os.makedirs(os.path.dirname(path),exist_ok=True)
    with open(path,'w') as f:
        f.write(f'vs {vs}\n')
        if oversize: f.write('!oversize\n')
        for c,n in sorted(outs.items()): f.write(f'-{n} {c}\n')
        for c,n in sorted(ins.items()): f.write(f'+{n} {c}\n')
P,PE,F,FN,OM,AL,CA,CH,EN,VE,CO,SU,AT,AC,IS,HM,SH,MA,CI,BO='Lotus Petal','Prismatic Ending','Force of Will','Force of Negation','Omniscience','Aluren','Carpet of Flowers',"Orim's Chant",'Prismatic Ending','Veil of Summer','Consign to Memory','Stock Up','Atraxa, Grand Unifier','Acererak the Archlich','Island','Hedge Maze','Show and Tell','Faerie Macabre','City of Traitors','Boseiju, Who Endures'
def D(*pairs): return list(pairs)
# name -> (deltas on the post-board 60, deck variant or None)
def tempo(force_down):
    d={}
    for tag,src in (('a',(AT,)),('s',(SH,)),('l',(IS,))):
        d['petal1'+tag]=D((P,1),(src[0],-1)); d['petal2'+tag]=D((P,2),(src[0],-2)) if src[0]!=IS else D((P,2),(IS,-1),(HM,-1))
    for tag,src in (('a',AT),('s',SH)):
        d['force3'+tag]=D((F,1),(src,-1)); d['force4'+tag]=D((F,2),(src,-2))
    d['omni1']=D((OM,1),(AT,-1)); d['omni2']=D((OM,2),(AT,-2))
    if force_down: d['force1']=D((F,-1),(FN,1))
    return d
EMR='Emrakul, the Aeons Torn'
def emr(): return {'emr0':D(),'emr1a':D((AT,-1),(EMR,1))}
SPEC={
 'ur': dict(tempo(True), trim1=D((IS,-1),(AL,1)), **emr()),
 'uw': dict(tempo(True), trim1=D((IS,-1),(AL,1)), **emr()),
 'ub': dict({k:v for k,v in tempo(False).items() if not k.startswith('force')},
            force1a=D((F,1),(AT,-1)), force2a=D((F,2),(AT,-2)), force1s=D((F,1),(SH,-1)), force2s=D((F,2),(SH,-2)),
            trim1=D((IS,-1),(F,1)), **emr()),
 'boros': {'aluren4_forPetalless':D(), **emr()},
 'mirror': {
  'trim1':D((IS,-1),(SH,1)),
 },
 'reanimator': {'trim1':D((HM,-1),(SU,1))},
 'storm': {
  'fon0':D((FN,-1),(AC,1)), 'ending1':D((EN,-1),(SU,1)), 'trim1':D((HM,-1),(AC,1)),
  'omni0':D((OM,-2),(VE,2)), 'omni1':D((OM,-1),(VE,1)), **emr(),
 },
 'tron': {
  'consign3':D((CO,1),(EN,-1)), 'consign3s':D((CO,1),(SU,-1)), 'consign3a':D((CO,1),(AT,-1)), 'omni1':D((OM,1),(AT,-1)), 'omni2':D((OM,2),(AT,-2)), 'fon0':D((FN,-1),(IS,1)), 'ending1':D((EN,-1),(OM,1)),
  'trim1':D((HM,-1),(OM,1)), **emr(), 'ending0':D((EN,-2),(OM,1),(IS,1)),
 },
}
del SPEC['boros']['aluren4_forPetalless']
# deck variants: sideboard edits (Macabre is the spare slot)
DECKV={**{(m,c):{EMR:+1,MA:-1} for m in ('ur','uw','ub','tron','storm','boros') for c in ('emr0','emr1a','emr61')}, ('tron','consign3'):{CO:+1,MA:-1}, ('tron','consign3s'):{CO:+1,MA:-1}, ('tron','consign3a'):{CO:+1,MA:-1}}
def write_deck(tag, edits):
    d=f'decks-{tag}'; shutil.rmtree(d,ignore_errors=True); shutil.copytree('decks',d)
    lines=open('decks/alurentell.txt').read().split('\n'); out=[]; in_side=False
    for l in lines:
        s=l.strip()
        if not s: 
            out.append(l); 
            if any(x.startswith(('4 ','2 ','1 ','3 ')) for x in out if x.strip() and not x.startswith('#')): in_side=True
            continue
        if in_side and not s.startswith('#'):
            n,name=s.split(' ',1)
            n=int(n)+edits.get(name,0)
            if n>0: out.append(f'{n} {name}')
            continue
        out.append(l)
    for name,dlt in edits.items():
        if dlt>0 and not any(l.endswith(' '+name) for l in out): out.append(f'{dlt} {name}')
    open(f'{d}/alurentell.txt','w').write('\n'.join(out)+'\n')
    return d
if __name__=='__main__':
    # boros: Brady's earlier variant library, copied as plans
    for m,specs in SPEC.items():
        d=M[m]; base=post(m)
        for name,deltas in specs.items():
            fin=collections.Counter(base)
            for c,n in deltas: fin[c]+=n
            fin=+fin
            side=collections.Counter(SIDE)
            if (m,name) in DECKV:
                for c,n in DECKV[(m,name)].items(): side[c]+=n
            p=f'plans/{m}/{name}'
            emit(f'{p}/alurentell.txt', d['vs'], fin, side)
            src=f'oppplans/{d["opp"]}.txt'
            if d['opp']!='alurentell' and os.path.exists(src): shutil.copy(src,f'{p}/{d["opp"]}.txt')
            print(m,name,{c:n for c,n in deltas})
    for (m,name),edits in DECKV.items(): write_deck(f'{m}-{name}',edits)
    # boros library variants: a b c d e f g (base already generated)
    for v in 'abcdefg':
        p=f'plans/boros/v-{v}'; os.makedirs(p,exist_ok=True)
        shutil.copy(f'{REPO}/sideboard-variants-boros/{v}/alurentell.txt',f'{p}/alurentell.txt')
        shutil.copy('oppplans/boros-aggro.txt',f'{p}/boros-aggro.txt')
    # boros deltas (aluren/force/chant/ending/trim/omni) as additional plans
    base=post('boros')
    BS={'ending1':D((EN,-1),(AL,1)),'chant1':D((CH,-1),(AL,1)),'trim1':D((IS,-1),(AL,1)),'omni1':D((OM,-1),(FN,1)),'omni0':D((OM,-2),(FN,1),(CO,1)),
        'ending0':D((EN,-2),(AL,1),(VE,1)),'chant0':D((CH,-2),(AL,1),(VE,1))}
    for name,deltas in BS.items():
        fin=collections.Counter(base)
        for c,n in deltas: fin[c]+=n
        fin=+fin; p=f'plans/boros/{name}'
        emit(f'{p}/alurentell.txt','boros-aggro',fin,SIDE); shutil.copy('oppplans/boros-aggro.txt',f'{p}/boros-aggro.txt'); print('boros',name,deltas)

    # 61-card variants: one extra main-deck card on top of Brady's plan (the plan's own swaps are unchanged)
    X61={'ur':{'emr61':EMR,'p61':P,'f61':F,'o61':OM,'al61':AL}, 'uw':{'emr61':EMR,'p61':P,'f61':F,'o61':OM,'al61':AL}, 'ub':{'emr61':EMR,'p61':P,'f61':F,'o61':OM},
         'boros':{'emr61':EMR,'al61':AL,'su61':SU,'v61':VE}, 'tron':{'emr61':EMR,'c61':CO,'o61':OM}, 'storm':{'emr61':EMR,'v61':VE,'at61':AT,'su61':SU},
         'mirror':{'sh61':SH,'ci61':CI}}
    for m,xs in X61.items():
        base=post(m); d=M[m]
        for name,card in xs.items():
            fin=collections.Counter(base); fin[card]+=1
            side=collections.Counter(SIDE)
            if (m,name)==('tron','c61'): side[CO]+=1; side[MA]-=1
            if name=='emr61': side[EMR]+=1; side[MA]-=1
            p=f'plans/{m}/{name}'
            emit(f'{p}/alurentell.txt', d['vs'], fin, side, oversize=True)
            src=f'oppplans/{d["opp"]}.txt'
            if d['opp']!='alurentell' and os.path.exists(src): shutil.copy(src,f'{p}/{d["opp"]}.txt')
    write_deck('tron-c61',{CO:+1,MA:-1})
