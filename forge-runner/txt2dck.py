#!/usr/bin/env python3
"""Convert the deck-set thread's N-Card-Name .txt lists to Forge .dck (main + sideboard)."""
import sys, re
src, dst, name = sys.argv[1:4]
main, side, cur = [], [], None
cur = main
for line in open(src):
    line = line.rstrip("\n")
    if line.startswith("#"): continue
    if not line.strip():
        if main: cur = side
        continue
    cur.append(line.strip())
with open(dst, "w") as f:
    f.write(f"[metadata]\nName={name}\n[Main]\n" + "\n".join(main) + "\n[Sideboard]\n" + "\n".join(side) + "\n")
print(dst, "main", sum(int(re.match(r"\d+", l).group()) for l in main), "side", sum(int(re.match(r"\d+", l).group()) for l in side))
