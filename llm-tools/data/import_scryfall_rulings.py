#!/usr/bin/env python3
"""Replace the curated rulings with the official ones, from Scryfall's bulk files (download them where api.scryfall.com is reachable;
this sandbox cannot reach it, so this script has only been tested on a small fixture, not on the real bulk files).
usage: import_scryfall_rulings.py oracle-cards.json rulings.json [--out FILE]   (writes data/rulings.json by default, keeping the curated entries for cards with no official rulings)"""
import json, os, sys
HERE = os.path.dirname(os.path.abspath(__file__))
oracle, rulings = json.load(open(sys.argv[1])), json.load(open(sys.argv[2]))
pool = json.load(open(os.path.join(HERE, "cards.json")))["cards"]
by_oracle = {}
for c in oracle:
    names = [c["name"]] + [f["name"] for f in c.get("card_faces", [])] + c["name"].split(" // ")
    for n in names:
        if n in pool: by_oracle.setdefault(c["oracle_id"], n)
out = {}
for r in sorted(rulings, key=lambda r: r["published_at"]):
    n = by_oracle.get(r["oracle_id"])
    if n and r.get("source") == "wotc": out.setdefault(n, []).append(f"{r['published_at']}: {r['comment']}")
cur = json.load(open(os.path.join(HERE, "rulings.json")))
for n, rs in cur["rulings"].items(): out.setdefault(n, rs)
OUT = sys.argv[sys.argv.index("--out") + 1] if "--out" in sys.argv else os.path.join(HERE, "rulings.json")
json.dump({"note": "official Gatherer rulings via Scryfall (curated notes kept where none exist)", "rulings": out}, open(OUT, "w"), indent=1, ensure_ascii=False)
print(len(out), "cards with rulings")
