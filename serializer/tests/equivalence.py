#!/usr/bin/env python3
"""Mirror-vs-stock equivalence: with policy=mirror every decision is made by Forge's own AI, so the serializer and menu enumerator
must be pure observers. Compare winner and turn count of mirror games 0,2,4,.. (seat A first in the player list, as in `forge sim`)
with `forge sim -s <seed*1000+g>` for the same game. Any mismatch means our instrumentation changed the game.
Usage: python3 tests/equivalence.py [seed=3] [pairs=5]"""
import re, subprocess, sys, os, json
seed = int(sys.argv[1]) if len(sys.argv) > 1 else 3
pairs = int(sys.argv[2]) if len(sys.argv) > 2 else 5
FORGE = os.environ.get("FORGE_ROOT", "/home/claude/card-forge/forge")
JAR = f"{FORGE}/forge-gui-desktop/target/forge-gui-desktop-2.0.16-SNAPSHOT-jar-with-dependencies.jar"
DECKS = "/mnt/project-files/forge-runner/decks"
here = os.path.dirname(os.path.abspath(__file__))
out = "/home/claude/silo-out/equiv"
r = subprocess.run(["bash", f"{here}/../run.sh", "--a", "dnt.dck", "--b", "jund75.dck", "--games", str(2 * pairs - 1), "--seed", str(seed),
                    "--policy", "mirror", "--out", out], capture_output=True, text=True)
mine = {int(m.group(1)): (m.group(3), int(m.group(4))) for m in re.finditer(r"^game (\d+) seed (\d+) winner (\S+) turn (\d+)", r.stdout, re.M)}
bad = 0
for g in range(0, 2 * pairs, 2):
    s = seed * 1000 + g
    p = subprocess.run(["java", "-Djava.awt.headless=true", "-Xmx1g", "-jar", JAR, "sim", "-d", "dnt.dck", "jund75.dck", "-D", DECKS, "-n", "1", "-q", "-c", "120", "-s", str(s)],
                       cwd=f"{FORGE}/forge-gui", capture_output=True, text=True)
    m = re.search(r"Game Outcome: Turn (\d+).*?\n.*?\n.*?Game Result: Game 1 ended in \d+ ms\. Ai\((\d)\)", p.stdout, re.S)
    turn = int(re.search(r"Game Outcome: Turn (\d+)", p.stdout).group(1)) * 2  # sim prints per-player turns, we print game turns
    win = re.search(r"Ai\((\d)\)-\S+(?: \S+)* has won!", p.stdout).group(1)
    sim_w = "Seat-A" if win == "1" else "ForgeAI-B"
    mw, mt = mine[g]
    ok = (mw == sim_w and abs(mt - turn) <= 1)  # sim turn is rounded down to per-player turns
    bad += not ok
    print(f"seed {s}: sim winner={sim_w} turns~{turn}  mirror winner={mw} turns={mt}  {'OK' if ok else 'MISMATCH'}")
print("EQUIVALENT" if bad == 0 else f"{bad} MISMATCHES")
sys.exit(1 if bad else 0)
