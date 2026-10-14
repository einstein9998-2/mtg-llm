#!/usr/bin/env python3
"""Spots in LLM-played full game logs: Alurentell's turns where Prismatic Ending is in hand and the opponent has a creature."""
import re, sys, glob, json, os
files = sys.argv[1:]
rows = []
CREAT = re.compile(r"^(.*?) (\d+)/(\d+)( \(T\))?( x(\d+))?$")
def parse_bf(s):
    items = []
    for part in s.split(", "):
        m = CREAT.match(part.strip())
        if m:
            n = int(m.group(6) or 1)
            for _ in range(n):
                items.append((m.group(1), int(m.group(2)), int(m.group(3))))
    return items
for f in files:
    txt = open(f).read().split("\n")
    opp_name = re.search(r"vs (.*?) \(best", txt[0])
    opp_name = opp_name.group(1) if opp_name else "?"
    game = 0; result = {}
    for i, l in enumerate(txt):
        m = re.match(r"## Game (\d+)", l)
        if m: game = int(m.group(1))
        m = re.match(r"- Result: \*\*(.*) wins\*\*", l)
        if m: result[game] = m.group(1)
    game = 0
    boarded = {}
    i = 0
    cur = None
    while i < len(txt):
        l = txt[i]
        m = re.match(r"## Game (\d+)", l)
        if m: game = int(m.group(1))
        m = re.match(r"#### Alurentell's turn (\d+) \(game turn (\d+)\)", l)
        if m:
            if cur: rows.append(cur)
            cur = dict(file=os.path.basename(f), opp=opp_name, game=game, myturn=int(m.group(1)), gturn=int(m.group(2)), ending_in_hand=False, casts=[], ending_cast=False, ending_target=None, x=None, result=result.get(game))
            i += 1; continue
        if re.match(r"#### (?!Alurentell's)", l) or re.match(r"## Game", l) or l.startswith("#### Opening"):
            if cur: rows.append(cur); cur = None
        if cur is not None:
            m = re.match(r"  - Alurentell — life (\d+), library (\d+), hand \((\d+)\): (.*)$", l)
            if m and "hand0" not in cur:
                cur["life"] = int(m.group(1)); cur["hand"] = m.group(4); cur["hand_n"] = int(m.group(3))
                cur["ending_in_hand"] = "Prismatic Ending" in m.group(4)
                cur["hand0"] = True
                bf = txt[i+1].strip()
                cur["my_bf"] = bf.split(" | ")[0].replace("battlefield: ", "")
            m = re.match(r"  - (?!Alurentell)(.*?) — life (-?\d+), library (\d+), hand \((\d+)\): (.*)$", l)
            if m and "olife" not in cur:
                cur["olife"] = int(m.group(2)); cur["ohand_n"] = int(m.group(4))
                bf = txt[i+1].strip().split(" | ")[0].replace("battlefield: ", "")
                cur["ocreatures"] = parse_bf(bf)
            m = re.match(r"- \*\*Alurentell\*\* · .*?: \*\*Cast (.*?)\*\*", l)
            if m:
                cur["casts"].append(m.group(1))
                if m.group(1) == "Prismatic Ending": cur["ending_cast"] = True
            m = re.match(r"- \*\*Alurentell\*\* · .*Choose target 1\. \[stack: Prismatic Ending \(Alurentell\)\]: \*\*Target: (.*?) \[", l)
            if m: cur["ending_target"] = m.group(1)
            m = re.match(r"- \*\*Alurentell\*\* · .*Choose the value of X\. \[stack: Prismatic Ending.*?\*\*X = (\d+)", l)
            if m: cur["x"] = int(m.group(1))
        i += 1
    if cur: rows.append(cur)
spots = [r for r in rows if r.get("ending_in_hand") and r.get("ocreatures")]
json.dump(spots, open("llm_spots.json", "w"), indent=0)
print(len(rows), "alurentell turns;", len(spots), "spots with Ending in hand and an opposing creature")
