#!/usr/bin/env python3
"""Per-game land stats for Alurentell (seat A) from the harness transcript (post-game view of A's prompts).
usage: landstats.py TRANSCRIPTS_DIR REPORTS_DIR  (e.g. landstats.py transcripts reports)  -> tsv on stdout"""
import re, sys, os, glob, gzip
LANDS = {"Ancient Tomb","Boseiju, Who Endures","City of Traitors","Flooded Strand","Forest","Hedge Maze","Island",
         "Misty Rainforest","Polluted Delta","Prismatic Vista","Tropical Island"}
FETCH = {"Flooded Strand","Misty Rainforest","Polluted Delta","Prismatic Vista"}
BLUE = {"Island","Tropical Island","Hedge Maze"}          # untapped-for-blue after it is on the battlefield (City excluded: pays life)
GREEN = {"Forest","Tropical Island","Hedge Maze","Boseiju, Who Endures"}
def parse(path):
    txt = gzip.open(path, "rt").read().split("\n")
    ev = []; inev = False
    for ln in txt:
        if ln.startswith("RECENT EVENTS:"): inev = True; continue
        if ln.startswith("DECISION:") or ln.startswith("=== PROMPT"): inev = False
        if inev: ev.append(ln.strip())
    # wasteland targets from stack display (dedupe by ids)
    wl = {}
    for ln in txt:
        m = re.search(r"Wasteland #(\d+) \(ability\) controlled by opponent targeting (.+?) #(\d+)", ln)
        if m: wl[(m.group(1), m.group(3))] = m.group(2)
    return ev, wl
def analyze(tag, rundir, repdir):
    p = f"{rundir}/{tag}-a.txt.gz"
    if not os.path.exists(p): return None
    ev, wl = parse(p)
    myturn = 0; opturn = 0; side = None
    played = {}; bf = {}   # bf: turn -> list of my lands at end of my turn
    cur = []              # my lands on the battlefield now
    first_blue = first_gg = None
    opp_wl_played = 0; lost_wl = []
    drops = {}
    seen = set()
    for l in ev:
        m = re.match(r"-- your turn (\d+)", l)
        if m: myturn = int(m.group(1)); side = "me"; drops.setdefault(myturn, 0); continue
        m = re.match(r"-- opponent's turn (\d+)", l)
        if m: opturn = int(m.group(1)); side = "op"; continue
        m = re.match(r"you played (.+)$", l)
        if m and m.group(1) in LANDS: drops[myturn] = drops.get(myturn, 0) + 1
        if l == "opponent played Wasteland": opp_wl_played += 1
        m = re.match(r"(.+?) \(you's\) moved (\w+) -> (\w+)$", l)
        if m and m.group(1) in LANDS:
            n, a, b = m.groups()
            if b == "Battlefield": cur.append(n)
            elif a == "Battlefield" and n in cur: cur.remove(n)
            # record at every change; per-turn snapshot below
        if m and side == "me" or (m and side == "op"):
            pass
        # snapshot: blue/green availability
        bl = sum(1 for x in cur if x in BLUE); gr = sum(1 for x in cur if x in GREEN)
        if first_blue is None and bl >= 1 and side == "me": first_blue = myturn
        if first_gg is None and gr >= 2 and side == "me": first_gg = myturn
        if side == "me": bf[myturn] = list(cur)
    wl = {}
    for i, l in enumerate(ev):
        if l == "Wasteland (opponent's) moved Battlefield -> Graveyard":
            tgt = "?"
            for k in ev[i+1:i+4]:
                m2 = re.match(r"(.+?) \(you's\) moved Battlefield -> Graveyard$", k)
                if m2 and m2.group(1) in LANDS: tgt = m2.group(1); break
            wl[i] = tgt
    nwl = len(wl)
    res = "?"
    rp = f"{repdir}/{tag}a.md"
    if os.path.exists(rp):
        mm = re.search(r"RESULT:\s*(WON|LOST)", open(rp).read())
        if mm: res = mm.group(1)
    turns = max(drops) if drops else 0
    nodrop = [t for t in range(1, turns + 1) if drops.get(t, 0) == 0]
    return dict(tag=tag, result=res, turns=turns, wl_hits=nwl, wl_targets="|".join(sorted(wl.values())),
                opp_wl_played=opp_wl_played, nodrop_turns=len(nodrop), first_blue=first_blue, first_gg=first_gg,
                lands_t3=len(bf.get(3, [])) if 3 in bf else "", lands_t4=len(bf.get(4, [])) if 4 in bf else "")
if __name__ == "__main__":
    rundir, repdir = sys.argv[1], sys.argv[2]
    cols = ["tag","result","turns","wl_hits","wl_targets","opp_wl_played","nodrop_turns","first_blue","first_gg","lands_t3","lands_t4"]
    print("\t".join(cols))
    tags = sorted({os.path.basename(p)[:-4] for p in glob.glob(f"{repdir}/*g1a.md")})
    for t in tags:
        r = analyze(t, rundir, repdir)
        if r: print("\t".join(str(r[c]) for c in cols))
