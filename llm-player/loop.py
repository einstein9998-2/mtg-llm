#!/usr/bin/env python3
"""Mechanical Aluren + Acererak loop driver. Same choices I make by hand: cast Acererak free, Lost Mine, Goblin Lair then Dark Pool (drain),
scry: keep only Veil of Summer / Show and Tell / Force of Will, bottom the rest. Stops at anything else (a prompt of another kind, an opponent spell
on the stack, a lock piece on their board, hand over 7 at cleanup, game over) and prints that decision.
usage: loop.py [max_prompts=60]   (the first thing it does is read the current decision)"""
import subprocess, sys, re, os
HERE = os.path.dirname(os.path.abspath(__file__))
def run(*a): return subprocess.run([sys.executable, os.path.join(HERE, "play.py"), *a], capture_output=True, text=True).stdout
KEEP = ("Veil of Summer", "Show and Tell", "Force of Will")
LOCK = ("Sphere of Resistance", "Damping Sphere", "Trinisphere", "Disruptor Flute", "Thorn of Amethyst", "Null Rod", "Torpor Orb")
out = run("next"); n = 0; mx = int(sys.argv[1]) if len(sys.argv) > 1 else 60
def opts(o): return [(int(m.group(1)), m.group(2)) for l in o.split("\n") if (m := re.match(r"^(\d+) (.*)$", l))]
while n < mx and "GAME OVER" not in out:
    kind = re.search(r"kind=(\w+)", out.split("\n")[0]); kind = kind.group(1) if kind else ""
    if any(l in out for l in LOCK): break
    o = opts(out); pick = None; why = "loop"
    if kind == "ChooseDungeon":
        pick = next((i for i, t in o if t.startswith("Lost Mine")), None)
    elif kind == "ChooseRoom":
        pick = next((i for i, t in o if "loses 1 life" in t), None)
        if pick is None: pick = next((i for i, t in o if "Goblin" in t), None)
        why = "room"
    elif kind == "Scry":
        m = re.search(r"Scry: (.*?) \(card", out); name = m.group(1) if m else ""
        pick = 0 if name in KEEP else 1; why = f"scry {name}"
    elif kind == "Priority":
        st = re.search(r"STACK \(top first\): (.*)", out)
        foreign = st and "empty" not in st.group(1) and any(("(opp)" in part) for part in st.group(1).split(";"))
        if not foreign:
            pick = next((i for i, t in o if t.startswith("Cast Acererak") and ("Aluren" in t or "Omniscience" in t)), None)
    if pick is None: break
    out = run("pick", str(pick), why); n += 1
print(f"[loop made {n} picks]"); print(out)
