"""Trains the policy/value net on MCTS self-play data (`mctsdata`) and exports the weights file the
Rust search loads (`--net`). Needs torch; uses the GPU when there is one.

    python3 train.py data1.jsonl [data2.jsonl ...] --out net.bin [--epochs 8] [--hidden 128]

The value target is the game result for the acting seat (+1 win, -1 loss, 0 draw), optionally mixed
with the search's root value (--value-mix). The policy target is the MCTS visit distribution.
"""
import argparse
import json
import math
import random
import sys

import torch
import torch.nn as nn
import torch.nn.functional as F

import mtg_net

N_KIND, N_DEC, N_ZONE = 20, 18, 9


class PVNet(nn.Module):
    """Same architecture as crates/mtg-agent/src/net.rs (keep in sync; test_train.py checks parity)."""

    def __init__(self, state_len, n_def, hidden=128, emb=64):
        super().__init__()
        self.state_len, self.hidden, self.emb, self.n_def = state_len, hidden, emb, n_def
        self.register_buffer("scale", torch.ones(state_len))
        self.w1 = nn.EmbeddingBag(state_len, hidden, mode="sum")
        self.b1 = nn.Parameter(torch.zeros(hidden))
        self.l2 = nn.Linear(hidden, hidden)
        self.vh = nn.Linear(hidden, 1)
        self.lp = nn.Linear(hidden, emb)
        self.e_kind = nn.Embedding(N_KIND, emb)
        self.e_dec = nn.Embedding(N_DEC, emb)
        self.e_def = nn.Embedding(n_def, emb)
        self.e_zone = nn.Embedding(N_ZONE, emb)
        self.w_val = nn.Parameter(torch.zeros(emb))
        for e in (self.e_kind, self.e_dec, self.e_def, self.e_zone):
            nn.init.normal_(e.weight, std=0.3)
        nn.init.normal_(self.w1.weight, std=0.1)

    def forward(self, idx, off, wts, kind, dec, df, zone, val, mask):
        """idx/off/wts: flat sparse state (EmbeddingBag input); kind..val, mask: [B, N] option tensors."""
        h1 = F.relu(self.w1(idx, off, per_sample_weights=wts * self.scale[idx]) + self.b1)
        h2 = F.relu(self.l2(h1))
        value = torch.tanh(self.vh(h2)).squeeze(-1)
        q = self.lp(h2)
        kind, dec, zone = kind.clamp(max=N_KIND - 1), dec.clamp(max=N_DEC - 1), zone.clamp(max=N_ZONE - 1)
        o = self.e_kind(kind) + self.e_dec(dec) + self.e_def(df.clamp(max=self.n_def - 1)) + self.e_zone(zone) + self.w_val * torch.log1p(val.float()).unsqueeze(-1)
        logits = (o * q.unsqueeze(1)).sum(-1) / math.sqrt(self.emb)
        return value, logits.masked_fill(~mask, float("-inf"))

    def export(self, path):
        dims = dict(state_len=self.state_len, hidden=self.hidden, emb=self.emb, n_kind=N_KIND, n_dec=N_DEC, n_def=self.n_def, n_zone=N_ZONE)
        c = lambda t: t.detach().cpu().float().flatten().tolist()
        T = lambda lin: lin.weight.detach().t().contiguous()  # torch Linear is [out, in]; the file wants [in][out]
        t = {
            "scale": c(self.scale), "w1": c(self.w1.weight), "b1": c(self.b1), "w2": c(T(self.l2)), "b2": c(self.l2.bias),
            "wv": c(self.vh.weight.squeeze(0)), "bv": c(self.vh.bias), "wp": c(T(self.lp)), "bp": c(self.lp.bias),
            "e_kind": c(self.e_kind.weight), "e_dec": c(self.e_dec.weight), "e_def": c(self.e_def.weight), "e_zone": c(self.e_zone.weight), "w_val": c(self.w_val),
        }
        mtg_net.write_net(path, dims, t)


def load_rows(paths):
    rows = []
    for p in paths:
        with open(p) as f:
            for line in f:
                rows.append(json.loads(line))
    return rows


def batch(rows, value_mix, device):
    idx, off, wts, nnz = [], [], [], 0
    N = max(len(r["options"]) for r in rows)
    B = len(rows)
    kind = torch.zeros(B, N, dtype=torch.long); dec = torch.zeros_like(kind); df = torch.zeros_like(kind); zone = torch.zeros_like(kind)
    val = torch.zeros(B, N); mask = torch.zeros(B, N, dtype=torch.bool); pol = torch.zeros(B, N)
    z = torch.zeros(B)
    for b, r in enumerate(rows):
        off.append(nnz)
        for i, v in r["state"]:
            idx.append(i); wts.append(v)
        nnz += len(r["state"])
        for j, o in enumerate(r["options"]):
            kind[b, j], dec[b, j], df[b, j], zone[b, j], val[b, j] = o
            mask[b, j] = True
        pol[b, :len(r["policy"])] = torch.tensor(r["policy"])
        z[b] = (1 - value_mix) * r["result"] + value_mix * r["root_value"]
    t = lambda x: x.to(device)
    return (t(torch.tensor(idx, dtype=torch.long)), t(torch.tensor(off, dtype=torch.long)), t(torch.tensor(wts)), t(kind), t(dec), t(df), t(zone), t(val), t(mask)), t(pol), t(z)


def evaluate(model, rows, args, device):
    model.eval()
    tot = {"n": 0, "pol": 0.0, "val": 0.0, "top1": 0.0, "base": 0.0}
    with torch.no_grad():
        for i in range(0, len(rows), args.batch):
            chunk = rows[i:i + args.batch]
            x, pol, z = batch(chunk, args.value_mix, device)
            v, logits = model(*x)
            lp = F.log_softmax(logits, -1)
            tot["pol"] += -(pol * lp.masked_fill(pol == 0, 0.0)).sum().item()
            tot["val"] += ((v - z) ** 2).sum().item()
            tot["base"] += (z ** 2).sum().item()  # predicting 0 everywhere
            tot["top1"] += (logits.argmax(-1) == pol.argmax(-1)).float().sum().item()
            tot["n"] += len(chunk)
    n = max(tot["n"], 1)
    return {"policy_ce": tot["pol"] / n, "value_mse": tot["val"] / n, "value_mse_zero_baseline": tot["base"] / n, "top1_agreement": tot["top1"] / n}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("data", nargs="+")
    ap.add_argument("--out", required=True)
    ap.add_argument("--epochs", type=int, default=8)
    ap.add_argument("--batch", type=int, default=256)
    ap.add_argument("--lr", type=float, default=1e-3)
    ap.add_argument("--hidden", type=int, default=128)
    ap.add_argument("--emb", type=int, default=64)
    ap.add_argument("--value-mix", type=float, default=0.0)
    ap.add_argument("--val-frac", type=float, default=0.1)
    ap.add_argument("--init", help="start from a saved .pt checkpoint")
    ap.add_argument("--seed", type=int, default=1)
    args = ap.parse_args()
    random.seed(args.seed); torch.manual_seed(args.seed)
    device = "cuda" if torch.cuda.is_available() else "cpu"
    rows = load_rows(args.data)
    meta = json.load(open(args.data[0] + ".meta.json"))
    n_defs, state_len = meta["n_defs"], meta["state_len"]
    games = sorted({(r["game"]) for r in rows})
    val_games = set(random.Random(args.seed).sample(games, max(1, int(len(games) * args.val_frac)))) if len(games) > 1 else set()
    train = [r for r in rows if r["game"] not in val_games]
    val = [r for r in rows if r["game"] in val_games]
    print(f"{len(rows)} rows ({len(train)} train, {len(val)} validation), device {device}", flush=True)
    model = PVNet(state_len, n_defs + 1, args.hidden, args.emb).to(device)
    if args.init:
        model.load_state_dict(torch.load(args.init, map_location=device))
    opt = torch.optim.Adam(model.parameters(), lr=args.lr)
    for ep in range(args.epochs):
        model.train()
        random.shuffle(train)
        run = 0.0
        for i in range(0, len(train), args.batch):
            x, pol, z = batch(train[i:i + args.batch], args.value_mix, device)
            v, logits = model(*x)
            lp = F.log_softmax(logits, -1).masked_fill(pol == 0, 0.0)
            loss = -(pol * lp).sum(-1).mean() + F.mse_loss(v, z)
            opt.zero_grad(); loss.backward(); opt.step()
            run += loss.item() * len(z)
        msg = f"epoch {ep + 1}: train loss {run / max(len(train), 1):.4f}"
        if val:
            msg += " | validation " + json.dumps({k: round(v, 4) for k, v in evaluate(model, val, args, device).items()})
        print(msg, flush=True)
    torch.save(model.state_dict(), args.out + ".pt")
    model.export(args.out)
    print("wrote", args.out, "and", args.out + ".pt")


if __name__ == "__main__":
    main()
