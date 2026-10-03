#!/usr/bin/env python3
"""Build data/cards.json from the Forge card scripts at the engine's pinned commit (fd5c996), so get_card text matches what the engine runs.
Network: raw.githubusercontent.com only. Card list = every name in decks/*.txt (main and sideboard) plus the forge-card-coverage.csv scripts.
usage: build_cards.py [--decks DIR] [--ref fd5c996]"""
import csv, json, os, re, subprocess, sys, urllib.request
from concurrent.futures import ThreadPoolExecutor
HERE = os.path.dirname(os.path.abspath(__file__))
DECKS = "/mnt/project-files/decks"; REF = "fd5c996"
for i, a in enumerate(sys.argv):
    if a == "--decks": DECKS = sys.argv[i + 1]
    if a == "--ref": REF = sys.argv[i + 1]
BASE = f"https://raw.githubusercontent.com/Card-Forge/forge/{REF}/forge-gui/res/cardsfolder/"

def deck_names():
    out = set()
    for f in os.listdir(DECKS):
        if not f.endswith(".txt"): continue
        for line in open(os.path.join(DECKS, f)):
            m = re.match(r"^(\d+) (.+?)\s*$", line)
            if m: out.add(m.group(2))
    return out

def script_path(name):
    s = re.sub(r"[^a-z0-9 ]", "", name.lower()).replace(" ", "_")
    return f"{s[0]}/{s}.txt"

def fetch(path):
    return urllib.request.urlopen(BASE + path, timeout=30).read().decode()

def mana_value(cost):
    v = 0
    for t in cost.split():
        if t.isdigit(): v += int(t)
        elif t in ("X", "Y", "Z"): pass
        else: v += 1
    return v

def cost_str(c): return "".join("{" + t + "}" for t in c.split()) if c and c != "no cost" else ""

def parse_face(lines):
    f = {"name": "", "mana_cost": "", "types": "", "pt": "", "loyalty": "", "oracle": []}
    for l in lines:
        k, _, v = l.partition(":")
        if k == "Name": f["name"] = v
        elif k == "ManaCost": f["mana_cost"] = cost_str(v)
        elif k == "Types": f["types"] = v
        elif k == "PT": f["pt"] = v
        elif k == "Loyalty": f["loyalty"] = v
        elif k == "Oracle": f["oracle"] = v.replace("\\n", "\n")
    f["oracle"] = f["oracle"] if isinstance(f["oracle"], str) else ""
    f["mana_value"] = mana_value(f["mana_cost"].replace("{", " ").replace("}", " ")) if f["mana_cost"] else 0
    return f

def type_line(types):
    t = types.split()
    sup = [x for x in t if x in ("Legendary", "Basic", "Snow", "World")]
    main = [x for x in t if x in ("Artifact", "Creature", "Enchantment", "Land", "Instant", "Sorcery", "Planeswalker", "Battle", "Kindred")]
    sub = [x for x in t if x not in sup and x not in main]
    return " ".join(sup + main) + (" - " + " ".join(sub) if sub else "")

def main():
    names = sorted(deck_names())
    cov = {r["card"]: r["forge_script"] for r in csv.DictReader(open(os.path.join(DECKS, "forge-card-coverage.csv")))}
    def job(n):
        p = cov.get(n) or script_path(n)
        return n, p, fetch(p)
    with ThreadPoolExecutor(8) as ex: res = list(ex.map(job, names))
    cards = {}
    for n, p, text in res:
        parts = re.split(r"\nALTERNATE\n", text.strip() + "\n")
        faces = [parse_face(x.strip().split("\n")) for x in parts]
        f0 = faces[0]
        assert f0["name"] == n or n in ("Fire // Ice",), (n, f0["name"])
        cards[n] = {"name": n, "script": p, "faces": [dict(x, type_line=type_line(x["types"])) for x in faces]}
    json.dump({"forge_ref": REF, "cards": cards}, open(os.path.join(HERE, "cards.json"), "w"), indent=1, ensure_ascii=False)
    print(len(cards), "cards,", sum(len(c["faces"]) > 1 for c in cards.values()), "multi-face")
main()
