#!/usr/bin/env python3
"""Headless Forge AI-vs-AI batch runner: measures games/sec.

Launches W parallel Forge `sim` JVMs, each playing N games of deck1 vs deck2
with its own RNG seed, parses the per-game result lines, and writes a JSON
summary plus a per-game CSV. Forge's `sim` mode is single-threaded per JVM, so
parallelism = number of JVMs.

Note: Forge's sim plays deck1 on the play for game 1 and lets the loser choose
afterwards (normal match rules are NOT used with -n; each game is a fresh game
in one Match object), so "seat" and "player on the play" are not controlled here.
"""
import argparse, concurrent.futures as cf, csv, json, os, re, subprocess, sys, time

FORGE_ROOT = os.environ.get("FORGE_ROOT", "/home/claude/card-forge/forge")
JAR = os.path.join(FORGE_ROOT, "forge-gui-desktop/target/forge-gui-desktop-2.0.16-SNAPSHOT-jar-with-dependencies.jar")
CWD = os.path.join(FORGE_ROOT, "forge-gui")  # Forge locates ./res relative to cwd
HERE = os.path.dirname(os.path.abspath(__file__))

RES = re.compile(r"Game Result: Game (\d+) ended in (\d+) ms\. (.+?) has won!")
DRAW = re.compile(r"Game Result: Game (\d+) ended in a Draw! Took (\d+) ms")
TURN = re.compile(r"Game Outcome: Turn (\d+)")

def worker(wid, a):
    seed = a.seed + wid
    cmd = ["java", "-Djava.awt.headless=true", f"-Xmx{a.heap}", "-XX:+UseParallelGC",
           f"-XX:ActiveProcessorCount={a.cpus_per_worker}",
           "-jar", JAR, "sim", "-d", a.deck1, a.deck2, "-D", a.decks_dir,
           "-n", str(a.games), "-q", "-c", str(a.timeout), "-s", str(seed)]
    t0 = time.time()
    p = subprocess.run(cmd, cwd=CWD, capture_output=True, text=True)
    wall = time.time() - t0
    games, last_turn = [], None
    for line in p.stdout.splitlines():
        m = TURN.search(line)
        if m: last_turn = int(m.group(1))
        m = RES.search(line)
        if m:
            games.append(dict(worker=wid, seed=seed, game=int(m.group(1)), ms=int(m.group(2)),
                              winner=m.group(3), turns=last_turn))
            continue
        m = DRAW.search(line)
        if m:
            games.append(dict(worker=wid, seed=seed, game=int(m.group(1)), ms=int(m.group(2)),
                              winner="DRAW", turns=last_turn))
    errs = [l for l in (p.stdout + p.stderr).splitlines()
            if re.search(r"Exception|Error|Stopping slow|did not have|Did not have", l) and "JAVA_TOOL" not in l]
    return wid, wall, games, errs, p.returncode

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--deck1", default="delver.dck")
    ap.add_argument("--deck2", default="jund.dck")
    ap.add_argument("--decks-dir", default=os.path.join(HERE, "decks"))
    ap.add_argument("--workers", type=int, default=1)
    ap.add_argument("--games", type=int, default=20, help="games per worker")
    ap.add_argument("--seed", type=int, default=1000)
    ap.add_argument("--heap", default="1g")
    ap.add_argument("--cpus-per-worker", type=int, default=1)
    ap.add_argument("--timeout", type=int, default=120, help="per-game sim timeout (s), counted as draw")
    ap.add_argument("--tag", default=None)
    a = ap.parse_args()
    tag = a.tag or f"{os.path.splitext(a.deck1)[0]}_vs_{os.path.splitext(a.deck2)[0]}_w{a.workers}_n{a.games}"

    t0 = time.time()
    with cf.ThreadPoolExecutor(a.workers) as ex:
        results = list(ex.map(lambda w: worker(w, a), range(a.workers)))
    wall = time.time() - t0

    games = [g for _, _, gs, _, _ in results for g in gs]
    errs = sorted({e for *_, es, _ in results for e in es})
    n = len(games)
    wins = {}
    for g in games: wins[g["winner"]] = wins.get(g["winner"], 0) + 1
    game_ms = sorted(g["ms"] for g in games)
    sum_ms = sum(game_ms)
    pct = lambda q: game_ms[min(n - 1, int(q * n))] if n else None
    # steady-state: drop first 3 games per worker (JIT warm-up)
    warm = [g["ms"] for g in games if g["game"] > 3]
    summary = dict(
        tag=tag, deck1=a.deck1, deck2=a.deck2, workers=a.workers, games_per_worker=a.games,
        games_completed=n, wins=wins, return_codes=[r[4] for r in results],
        wall_sec_total=round(wall, 1),
        games_per_sec_wall_incl_jvm_startup=round(n / wall, 3) if wall else None,
        games_per_sec_per_worker_in_game_time=round(n / (sum_ms / 1000 / a.workers), 3) if sum_ms else None,
        aggregate_games_per_sec_steady_state=round(a.workers * len(warm) / (sum(warm) / 1000), 3) if warm else None,
        ms_per_game=dict(mean=round(sum_ms / n) if n else None, p50=pct(.5), p90=pct(.9), max=game_ms[-1] if n else None),
        mean_turns=round(sum(g["turns"] or 0 for g in games) / n, 2) if n else None,
        worker_wall_sec=[round(r[1], 1) for r in results],
        warnings_sample=errs[:10],
    )
    out = os.path.join(HERE, "results")
    os.makedirs(out, exist_ok=True)
    with open(os.path.join(out, f"{tag}.json"), "w") as f: json.dump(summary, f, indent=2)
    with open(os.path.join(out, f"{tag}.csv"), "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=["worker", "seed", "game", "ms", "winner", "turns"])
        w.writeheader(); w.writerows(games)
    print(json.dumps(summary, indent=2))

if __name__ == "__main__":
    main()
