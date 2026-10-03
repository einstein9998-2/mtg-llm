"""Parity: the torch model's export, evaluated by the Rust forward pass, must equal the torch forward.
Usage: python3 test_train.py <netcheck binary>   (needs torch)"""
import json
import os
import random
import subprocess
import sys
import tempfile

import torch

import mtg_net
from train import PVNet, N_KIND, N_DEC, N_ZONE

netcheck = sys.argv[1]
torch.manual_seed(2)
rng = random.Random(4)
S, H, E, ND = 80, 12, 7, 41
m = PVNet(S, ND, H, E)
with torch.no_grad():
    for p in m.parameters():
        p.add_(torch.randn_like(p) * 0.3)  # nonzero biases and w_val so every term matters
    m.scale.copy_(torch.tensor([rng.choice([1.0, 0.5, 0.1]) for _ in range(S)]))
m.eval()
with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "net.bin")
    m.export(path)
    worst = 0.0
    for trial in range(25):
        state = [(i, float(rng.randint(1, 3))) for i in rng.sample(range(S), rng.randint(1, 10))]
        opts = [(rng.randrange(N_KIND), rng.randrange(N_DEC), rng.randrange(ND + 3), rng.randrange(N_ZONE), rng.choice([0, 0, 2, 9])) for _ in range(rng.randint(2, 8))]
        idx = torch.tensor([i for i, _ in state]); off = torch.tensor([0]); wts = torch.tensor([v for _, v in state])
        T = lambda j: torch.tensor([[o[j] for o in opts]])
        with torch.no_grad():
            v, logits = m(idx, off, wts, T(0), T(1), T(2), T(3), T(4), torch.ones(1, len(opts), dtype=torch.bool))
        p_t = torch.softmax(logits, -1)[0].tolist()
        sample = os.path.join(d, "s.json")
        json.dump({"state": state, "options": opts}, open(sample, "w"))
        r = json.loads(subprocess.check_output([netcheck, path, sample]))
        err = max(abs(r["value"] - v.item()), max(abs(a - b) for a, b in zip(r["priors"], p_t)))
        worst = max(worst, err)
        assert err < 2e-4, (trial, err)
    print(f"ok: torch model and Rust forward agree (worst abs diff {worst:.2e})")
