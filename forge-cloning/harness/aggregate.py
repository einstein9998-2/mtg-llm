#!/usr/bin/env python3
"""Aggregate ForkProbe PROBE lines from one or more .out files."""
import json, sys, statistics as st, collections, re
rows, other = [], []
for f in sys.argv[1:]:
    for l in open(f):
        if l.startswith("PROBE "):
            d = json.loads(l[6:]); (rows if d["kind"] == "probe" else other).append(d)
def pct(v, p): v = sorted(v); return v[min(len(v)-1, int(p/100*len(v)))]
cm = [r["copy_ms_median"] for r in rows]
print(f"probes: {len(rows)} over {len(set((r['game'],) for r in rows))} game-runs (files={len(sys.argv)-1})")
print(f"copy ms: median {st.median(cm):.1f}  p90 {pct(cm,90):.1f}  max {max(cm):.1f}  min {min(cm):.1f}")
for lo,hi in [(0,6),(6,12),(12,20),(20,99)]:
    s=[r["copy_ms_median"] for r in rows if lo<=r["board"]<hi]
    if s: print(f"  board {lo}-{hi-1} permanents: n={len(s)} median {st.median(s):.1f} ms")
print("copy exposes opp hidden cards (ids+names, library order):", sum(r["copy_exposes_opp_hidden"] for r in rows), "/", len(rows))
print("score(orig)==score(copy):", sum(r["score_orig"]==r["score_copy"] for r in rows), "/", len(rows))
dd = [r for r in rows if r["digest_diffs"]]
print("probes with digest diffs:", len(dd), "/", len(rows))
c = collections.Counter()
for r in dd:
    for x in r["digest_diff_sample"].split(" | "):
        c[re.sub(r"\d+", "#", x)[:90]] += 1
for k,v in c.most_common(6): print("   ", v, k)
print("determinize ms: median", f"{st.median(r['determinize_ms'] for r in rows):.1f}", " size_consistent:", sum("size_consistent=true" in r["determinize"] for r in rows), "/", len(rows))
print("public info unchanged by determinize:", sum(r["public_unchanged_by_determinize"] for r in rows), "/", len(rows))
print("info-set invariance (same-seed determinize identical across different hidden worlds):", sum(r["infoset_invariant"] for r in rows), "/", len(rows))
sims = sum(r["sim_ok"]+r["sim_fail"] for r in rows)
print(f"simulate(): {sims} SA sims, ok {sum(r['sim_ok'] for r in rows)}, fail {sum(r['sim_fail'] for r in rows)}; avg ms per sim (weighted) {sum(r['sim_ms_avg']*(r['sim_ok']+r['sim_fail']) for r in rows)/max(1,sims):.1f}")
ec = collections.Counter()
for r in rows:
    for e in filter(None, r["sim_err"].split(" ; ")):
        ec[re.sub(r"\(\d+\)|\d{5,}", "#", e)[:110]] += 1
for k,v in ec.most_common(8): print("   ", v, k)
hp = [o for o in other if o["kind"]=="heap_parallel"]
for o in hp: print("heap/parallel:", {k:o[k] for k in o if k!="kind"})
print("errors:", [ (o["kind"], o.get("err","")[:100]) for o in other if "error" in o["kind"]][:6])
print("game_end:", [(o["game"],o["turns"],o["ms"]) for o in other if o["kind"]=="game_end"])
