#!/usr/bin/env python3
"""Coverage report over a scenario pool.

  coverage.py --pool DIR [--public-out COVERAGE.md] [--private-out FILE]

The public report has counts only (safe for the implementer: it never names holdout scenario ids).
The private report lists ids that need verification and per-source-kind breakdowns.
"""
import argparse, os, collections, json
import yaml

BASICS = {"Island", "Mountain", "Forest", "Plains", "Swamp"}

def load(pool):
    out = []
    for dp, dn, fn in os.walk(pool):
        for f in sorted(fn):
            if f.endswith(".yaml"):
                p = os.path.join(dp, f)
                for d in yaml.safe_load_all(open(p, encoding="utf8")):
                    if d: out.append((os.path.relpath(p, pool), d))
    return out

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--pool", required=True)
    ap.add_argument("--public-out"); ap.add_argument("--private-out")
    a = ap.parse_args()
    docs = load(a.pool)
    by_area = collections.Counter(d["area"] for _, d in docs)
    by_derived = collections.Counter(d["derived_from"] for _, d in docs)
    by_card = collections.Counter()
    by_tag = collections.Counter()
    unver, partial = [], []
    ver_counts = collections.Counter()
    for rel, d in docs:
        for c in d["cards"]:
            if c not in BASICS: by_card[c] += 1
        for t in d["tags"]: by_tag[t] += 1
        vs = [s["verified"] for s in d["source"]]
        for s in d["source"]:
            ver_counts[(s["kind"], s["verified"])] += 1
        if not any(vs): unver.append(d["id"])
        elif not all(vs): partial.append(d["id"])
    L = []
    L.append("# Coverage report (counts only)\n")
    L.append(f"Total scenarios: {len(docs)}\n")
    L.append("## By area\n"); L.append("| area | scenarios |\n|---|---|")
    for k, v in sorted(by_area.items()): L.append(f"| {k} | {v} |")
    L.append("\n## By derivation basis\n"); L.append("| derived_from | scenarios |\n|---|---|")
    for k, v in sorted(by_derived.items()): L.append(f"| {k} | {v} |")
    L.append("\n## Source verification (source entries)\n"); L.append("| kind | verified | entries |\n|---|---|---|")
    for (k, v), n in sorted(ver_counts.items()): L.append(f"| {k} | {v} | {n} |")
    L.append(f"\nScenarios whose every source is unverified: {len(unver)}. Scenarios with a mix: {len(partial)}.\n")
    L.append("## Scenarios per card (a scenario counts for every non-basic card it lists)\n"); L.append("| card | scenarios |\n|---|---|")
    for k, v in sorted(by_card.items()): L.append(f"| {k} | {v} |")
    L.append("\n## Scenarios per rules tag\n"); L.append("| tag | scenarios |\n|---|---|")
    for k, v in sorted(by_tag.items(), key=lambda kv: (-kv[1], kv[0])): L.append(f"| {k} | {v} |")
    pub = "\n".join(L) + "\n"
    if a.public_out: open(a.public_out, "w").write(pub)
    else: print(pub)
    if a.private_out:
        P = ["# Private coverage detail (do not publish)\n", "## Scenarios with only unverified sources\n"]
        P += [f"- {i}" for i in sorted(unver)]
        P += ["\n## Scenarios with mixed verification\n"] + [f"- {i}" for i in sorted(partial)]
        open(a.private_out, "w").write("\n".join(P) + "\n")

if __name__ == "__main__":
    main()
