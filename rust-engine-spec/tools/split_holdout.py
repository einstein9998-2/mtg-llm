#!/usr/bin/env python3
"""Split a full scenario pool into a visible set and a sealed holdout set.

Roles (doc 04 section 4.4): the spec-writer runs this. The implementer must not know the seed or the key.

  split_holdout.py --pool POOL_DIR --visible-out DIR --sealed-out FILE --manifest FILE
                   --seed-file FILE --key-file FILE [--fraction 0.30]

* Grouping: one group per file (a card slug, or a core topic file). Within a group of n scenarios,
  k = round(fraction * n) are held out when n >= 3 (so at least 2 stay visible), else 0.
* Selection is a pure function of (seed, scenario id): the scenarios with the smallest HMAC-SHA256(seed, id) per group.
  Re-running with the same seed and pool reproduces the split.
* The sealed file is a tar of the held-out documents, one YAML file per source file, encrypted with AES-256-CBC (PBKDF2)
  using the key file's contents as the passphrase.
* Replacing revealed scenarios (doc 04 section 4.4): `--revealed FILE` forces ids visible (their slot is not refilled, so the rest of the split is unchanged); `--fresh FILE` adds new ids to the holdout without changing any group's split.
* The public manifest has counts per group and a sha256 of each held-out document's text, never ids or content.
Document text is preserved byte for byte (split on lines that are exactly '---').
"""
import argparse, os, sys, re, hmac, hashlib, json, tarfile, io, subprocess, tempfile, datetime
import yaml

def split_docs(text):
    chunks, cur = [], []
    for line in text.splitlines(keepends=True):
        if re.fullmatch(r"---\s*", line):
            chunks.append("".join(cur)); cur = []
        else:
            cur.append(line)
    chunks.append("".join(cur))
    return [c for c in chunks if c.strip()]

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--pool", required=True)
    ap.add_argument("--visible-out", required=True)
    ap.add_argument("--sealed-out", required=True)
    ap.add_argument("--manifest", required=True)
    ap.add_argument("--seed-file", required=True)
    ap.add_argument("--key-file", required=True)
    ap.add_argument("--fraction", type=float, default=0.30)
    ap.add_argument("--revealed", help="file of scenario ids (one per line) that were revealed to the implementer: forced visible, their held-out slot is not refilled")
    ap.add_argument("--fresh", help="file of scenario ids (one per line) written after the first split to replace revealed ones: forced held out, not counted in their group's size")
    a = ap.parse_args()
    read_ids = lambda f: {l.strip() for l in open(f) if l.strip() and not l.startswith("#")} if f else set()
    revealed, fresh = read_ids(a.revealed), read_ids(a.fresh)
    seed = open(a.seed_file).read().strip().encode()
    groups = {}
    for dp, dn, fn in os.walk(a.pool):
        for f in sorted(fn):
            if f.endswith(".yaml"):
                p = os.path.join(dp, f)
                rel = os.path.relpath(p, a.pool)
                docs = split_docs(open(p, encoding="utf8").read())
                items = []
                for d in docs:
                    meta = yaml.safe_load(d)
                    items.append((meta["id"], d))
                groups[rel] = items
    held_total = 0; vis_total = 0
    manifest = {"tool": "split_holdout.py v1", "date": datetime.date.today().isoformat(), "fraction": a.fraction, "groups": {}}
    held_by_file = {}
    for rel, items in sorted(groups.items()):
        fresh_items = [(i, d) for i, d in items if i in fresh]
        items = [(i, d) for i, d in items if i not in fresh]
        n = len(items)
        k = int(a.fraction * n + 0.5) if n >= 3 else 0
        k = min(k, n - 2) if n >= 2 else 0
        ranked = sorted(items, key=lambda it: hmac.new(seed, it[0].encode(), hashlib.sha256).hexdigest())
        held_ids = {i for i, _ in ranked[:k]} - revealed
        vis = [(i, d) for i, d in items if i not in held_ids]
        held = [(i, d) for i, d in items if i in held_ids] + fresh_items
        n += len(fresh_items)
        out = os.path.join(a.visible_out, rel)
        os.makedirs(os.path.dirname(out), exist_ok=True)
        if vis:
            with open(out, "w", encoding="utf8") as fh:
                fh.write("---\n".join(d if d.endswith("\n") else d + "\n" for _, d in vis))
        if held:
            held_by_file[rel] = held
        manifest["groups"][rel] = {"total": n, "visible": len(vis), "holdout": len(held),
                                  "holdout_doc_sha256": sorted(hashlib.sha256(d.encode()).hexdigest() for _, d in held)}
        held_total += len(held); vis_total += len(vis)
    manifest["totals"] = {"scenarios": held_total + vis_total, "visible": vis_total, "holdout": held_total}
    buf = io.BytesIO()
    with tarfile.open(fileobj=buf, mode="w") as tf:
        for rel, held in held_by_file.items():
            data = "---\n".join(d if d.endswith("\n") else d + "\n" for _, d in held).encode()
            ti = tarfile.TarInfo(rel); ti.size = len(data); ti.mtime = 0
            tf.addfile(ti, io.BytesIO(data))
    with tempfile.NamedTemporaryFile(delete=False) as t:
        t.write(buf.getvalue()); tmp = t.name
    subprocess.check_call(["openssl", "enc", "-aes-256-cbc", "-pbkdf2", "-salt", "-in", tmp, "-out", a.sealed_out, "-pass", "file:" + a.key_file])
    os.unlink(tmp)
    manifest["sealed_archive_sha256"] = hashlib.sha256(open(a.sealed_out, "rb").read()).hexdigest()
    json.dump(manifest, open(a.manifest, "w"), indent=1)
    print(f"{vis_total} visible, {held_total} held out, groups={len(groups)}")

if __name__ == "__main__":
    main()
