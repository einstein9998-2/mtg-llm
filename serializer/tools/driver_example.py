#!/usr/bin/env python3
"""Reference external driver for `--policy external`: reads decision lines, replies with a pick. This one picks randomly; an LLM
harness replaces choose(). Usage: driver_example.py <pipe from engine> <pipe to engine> [seed]"""
import json, random, sys
frm, to = sys.argv[1], sys.argv[2]
rng = random.Random(int(sys.argv[3]) if len(sys.argv) > 3 else 0)
def choose(d):
    n, lo, hi = d["n"], d["min"], d["max"]
    k = rng.randint(lo, min(hi, n))
    return sorted(rng.sample(range(n), k))
rd = open(frm, "r")      # open in the engine's order: engine opens its writer first, so we open the matching reader first
wr = open(to, "w")
count = 0
for line in rd:
    msg = json.loads(line)
    if msg["type"] == "bye": break
    wr.write(json.dumps({"pick": choose(msg)}) + "\n"); wr.flush()
    count += 1
print("driver answered", count, "decisions", file=sys.stderr)
