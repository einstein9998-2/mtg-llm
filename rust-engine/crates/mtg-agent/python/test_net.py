"""Parity: Rust forward pass (netcheck) against the pure-Python reference on a random net.
Usage: python3 test_net.py <netcheck binary>"""
import json
import os
import random
import subprocess
import sys
import tempfile

import mtg_net

netcheck = sys.argv[1]
rng = random.Random(3)
dims = dict(state_len=60, hidden=8, emb=6, n_kind=20, n_dec=18, n_def=30, n_zone=9)
sz = mtg_net.sizes(dims)
t = {k: [rng.uniform(-0.6, 0.6) for _ in range(n)] for k, n in sz.items()}
t["scale"] = [rng.choice([1.0, 0.5, 0.05]) for _ in range(sz["scale"])]
with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "net.bin")
    mtg_net.write_net(path, dims, t)
    d2, t2 = mtg_net.read_net(path)
    assert d2 == dims and all(len(t2[k]) == len(t[k]) for k in t)
    worst = 0.0
    for trial in range(20):
        state = [(rng.randrange(60), rng.randint(1, 4) * 1.0) for _ in range(rng.randint(1, 12))]
        opts = [(rng.randrange(20), rng.randrange(18), rng.randrange(35), rng.randrange(9), rng.choice([0, 0, 1, 5, 20])) for _ in range(rng.randint(2, 9))]
        sample = os.path.join(d, "s.json")
        json.dump({"state": state, "options": opts}, open(sample, "w"))
        r = json.loads(subprocess.check_output([netcheck, path, sample]))
        v, p = mtg_net.forward_py(dims, t2, state, opts)
        err = max(abs(r["value"] - v), max(abs(a - b) for a, b in zip(r["priors"], p)))
        worst = max(worst, err)
        assert err < 1e-4, (trial, err)
    print(f"ok: Rust and Python forward passes agree (worst abs diff {worst:.2e})")
