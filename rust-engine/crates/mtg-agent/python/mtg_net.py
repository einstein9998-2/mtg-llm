"""Weights file for the policy/value network (format MTGNET01, see crates/mtg-agent/src/net.rs) and a
slow pure-Python reference forward pass used to check the Rust one. Standard library only."""
import math
import struct

MAGIC = b"MTGNET01"
TENSORS = ["scale", "w1", "b1", "w2", "b2", "wv", "bv", "wp", "bp", "e_kind", "e_dec", "e_def", "e_zone", "w_val"]
DIM_ORDER = ["state_len", "hidden", "emb", "n_kind", "n_dec", "n_def", "n_zone"]


def sizes(d):
    S, H, E = d["state_len"], d["hidden"], d["emb"]
    return {
        "scale": S, "w1": S * H, "b1": H, "w2": H * H, "b2": H, "wv": H, "bv": 1, "wp": H * E, "bp": E,
        "e_kind": d["n_kind"] * E, "e_dec": d["n_dec"] * E, "e_def": d["n_def"] * E, "e_zone": d["n_zone"] * E, "w_val": E,
    }


def write_net(path, dims, tensors):
    """`tensors[name]` is a flat sequence of floats (row-major as in net.rs)."""
    sz = sizes(dims)
    out = bytearray(MAGIC)
    out += struct.pack("<7I", *[dims[k] for k in DIM_ORDER])
    for name in TENSORS:
        t = list(tensors[name])
        if len(t) != sz[name]:
            raise ValueError(f"{name}: {len(t)} values, expected {sz[name]}")
        out += struct.pack(f"<{len(t)}f", *t)
    with open(path, "wb") as f:
        f.write(out)


def read_net(path):
    b = open(path, "rb").read()
    if b[:8] != MAGIC:
        raise ValueError("not an MTGNET01 file")
    dims = dict(zip(DIM_ORDER, struct.unpack("<7I", b[8:36])))
    off, tensors = 36, {}
    sz = sizes(dims)
    for name in TENSORS:
        n = sz[name]
        tensors[name] = list(struct.unpack(f"<{n}f", b[off:off + 4 * n]))
        off += 4 * n
    if off != len(b):
        raise ValueError("trailing bytes")
    return dims, tensors


def forward_py(dims, t, state, options):
    """state: [(i, v)], options: [(kind, dec, def, zone, value)] -> (value, priors)."""
    H, E = dims["hidden"], dims["emb"]
    h1 = list(t["b1"])
    for i, v in state:
        s = t["scale"][i] * v
        row = t["w1"][i * H:(i + 1) * H]
        for j in range(H):
            h1[j] += s * row[j]
    h1 = [max(0.0, x) for x in h1]
    h2 = list(t["b2"])
    for k in range(H):
        if h1[k] != 0.0:
            row = t["w2"][k * H:(k + 1) * H]
            for j in range(H):
                h2[j] += h1[k] * row[j]
    h2 = [max(0.0, x) for x in h2]
    value = math.tanh(t["bv"][0] + sum(a * b for a, b in zip(h2, t["wv"])))
    q = list(t["bp"])
    for k in range(H):
        if h2[k] != 0.0:
            row = t["wp"][k * E:(k + 1) * E]
            for j in range(E):
                q[j] += h2[k] * row[j]
    def row_of(name, i, n):
        i = min(i, n - 1)
        return t[name][i * E:(i + 1) * E]
    logits = []
    for kind, dec, df, zone, val in options:
        a = row_of("e_kind", kind, dims["n_kind"]); b = row_of("e_dec", dec, dims["n_dec"])
        c = row_of("e_def", df, dims["n_def"]); d = row_of("e_zone", zone, dims["n_zone"])
        lv = math.log(1.0 + val)
        logits.append(sum(q[j] * (a[j] + b[j] + c[j] + d[j] + t["w_val"][j] * lv) for j in range(E)) / math.sqrt(E))
    m = max(logits)
    ex = [math.exp(l - m) for l in logits]
    s = sum(ex)
    return value, [x / s for x in ex]
