#!/usr/bin/env python3
"""Post-game review: join my picks (player.jsonl) with the engine log (decisions.jsonl: Forge AI shadow pick at priority, hidden-info ground truth).
Only run AFTER a game is over. Usage: review.py runs/g01"""
import json, re, sys
run = sys.argv[1]
dec = [json.loads(l) for l in open(f"{run}/decisions.jsonl")]
mine = {json.loads(l)["id"]: json.loads(l) for l in open(f"{run}/player.jsonl")}
for d in dec:
    opts = re.findall(r"^(\d+) (.*)$", d["prompt"], re.M)
    lab = {int(i): t for i, t in opts}
    p = d["pick"]
    hint = d["hint"]
    flag = ""
    if d["kind"] == "Priority" and hint is not None and hint >= 0 and p and p[0] != hint:
        flag = "  <<< DIFFERS FROM FORGE AI"
    print(f"d{d['d']} T{d['turn']} {d['kind']} n={d['n']} picked={d['labels']!r} aiHint={lab.get(hint) if d['kind']=='Priority' and hint>=0 else '-'} truth={d.get('truth')}{flag}")
    note = mine.get(d["d"], {}).get("note")
    if note: print("      note:", note)
