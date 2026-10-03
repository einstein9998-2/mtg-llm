"""Expert-iteration loop: generation 0 searches with random rollouts, every later generation searches
with the previous net. Each generation: generate MCTS self-play data, train on everything so far
(warm-started), then measure the new net against random play and against rollout MCTS.

    python3 selfplay_loop.py --bin target/release --decks decks --work runs/a --gens 4 \
        --games 200 --iters 32 --threads 8 --epochs 6

Needs torch for the training step (GPU used if present). All artifacts land in --work.
"""
import argparse
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))


def run(cmd, **kw):
    print("$", " ".join(cmd), flush=True)
    return subprocess.run(cmd, check=True, text=True, **kw)


def arena(args, net, opp):
    out = subprocess.check_output([f"{args.bin}/mctsmatch", args.decks, str(args.arena_games), str(args.iters), "--net", net, "--opp", opp], text=True)
    print(out.strip(), flush=True)
    m = re.search(r": (\d+) wins, (\d+) losses, (\d+) draws", out)
    return tuple(int(x) for x in m.groups())


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--bin", required=True, help="directory with mctsdata and mctsmatch")
    ap.add_argument("--decks", required=True)
    ap.add_argument("--work", required=True)
    ap.add_argument("--gens", type=int, default=3)
    ap.add_argument("--games", type=int, default=100)
    ap.add_argument("--iters", type=int, default=32)
    ap.add_argument("--rollout", type=int, default=2500)
    ap.add_argument("--threads", type=int, default=4)
    ap.add_argument("--epochs", type=int, default=6)
    ap.add_argument("--hidden", type=int, default=128)
    ap.add_argument("--emb", type=int, default=64)
    ap.add_argument("--arena-games", type=int, default=40)
    ap.add_argument("--value-mix", type=float, default=0.0)
    args = ap.parse_args()
    os.makedirs(args.work, exist_ok=True)
    data, net, history = [], None, []
    for gen in range(args.gens):
        d = os.path.join(args.work, f"data{gen}.jsonl")
        cmd = [f"{args.bin}/mctsdata", args.decks, str(args.games), str(args.iters), d, "--threads", str(args.threads), "--seed", str(100 + gen), "--rollout", str(args.rollout)]
        if net:
            cmd += ["--net", net]
        run(cmd)
        data.append(d)
        new = os.path.join(args.work, f"net{gen}.bin")
        tcmd = [sys.executable, os.path.join(HERE, "train.py"), *data, "--out", new, "--epochs", str(args.epochs), "--hidden", str(args.hidden), "--emb", str(args.emb), "--value-mix", str(args.value_mix)]
        if net and os.path.exists(net + ".pt"):
            tcmd += ["--init", net + ".pt"]
        run(tcmd, cwd=HERE)
        net = new
        vs_random = arena(args, net, "random")
        vs_rollout = arena(args, net, "rollout")
        history.append({"gen": gen, "vs_random": vs_random, "vs_rollout": vs_rollout})
        print("history:", history, flush=True)


if __name__ == "__main__":
    main()
