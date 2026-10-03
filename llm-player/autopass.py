#!/usr/bin/env python3
"""Pass priority repeatedly while nothing useful is on the menu (only Pass and fetch-land activations) and it is not my turn.
Stops at the first decision that is anything else (my turn, a real choice, an option such as casting a spell). Prints that decision.
usage: autopass.py [max_passes=40]"""
import subprocess, sys, re, os
HERE = os.path.dirname(os.path.abspath(__file__))
def run(*a): return subprocess.run([sys.executable, os.path.join(HERE, "play.py"), *a], capture_output=True, text=True).stdout
out = run("next"); n = 0
mx = int(sys.argv[1]) if len(sys.argv) > 1 else 40
ignore = "|".join(a for a in sys.argv[2:])  # extra option prefixes to treat as not useful, e.g. "Cast Force of Will"
while n < mx:
    head = out.split("\n", 2)
    kind = re.search(r"kind=(\w+)", head[0])
    if "GAME OVER" in out or not kind or kind.group(1) != "Priority": break
    lines = out.split("\n")
    opts = [l for l in lines if re.match(r"^\d+ ", l)]
    useful = [o for o in opts if not (ignore and o.split(" ",1)[1].startswith(tuple(sys.argv[2:]))) and not re.match(r"^\d+ (Pass|Cast Lotus Petal|Activate (Misty Rainforest|Prismatic Vista|Polluted Delta|Flooded Strand|Scalding Tarn|Wooded Foothills|Verdant Catacombs|Windswept Heath|Marsh Flats|Arid Mesa|Bloodstained Mire|Misty))", o)]
    st = re.search(r"STACK \(top first\): (.*)", out)
    oppstack = bool(st and "(opp)" in st.group(1))
    if oppstack: useful = [o for o in opts if o.split(' ',1)[1].startswith('Cast Veil') or o in useful]
    if oppstack and 'Wasteland' in st.group(1): break  # respond by cracking the targeted fetch/land
    mine = re.search(r"T\d+ your turn, (MAIN1|DRAW|UPKEEP)", out)
    if useful or mine: break
    out = run("pick", "0", "autopass"); n += 1
print(f"[autopass passed {n} times]"); print(out)
