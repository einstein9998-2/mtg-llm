#!/usr/bin/env python3
"""Converts the spec thread's YAML scenarios to one JSON array (read-only on the spec folder).

usage: spec2json.py SPEC_DIR OUT.json
"""
import glob, json, os, sys
import yaml

spec, out = sys.argv[1], sys.argv[2]
docs = []
for f in sorted(glob.glob(os.path.join(spec, "scenarios", "**", "*.yaml"), recursive=True)):
    rel = os.path.relpath(f, spec)
    for d in yaml.safe_load_all(open(f)):
        if not d:
            continue
        d["_file"] = rel
        docs.append(d)
json.dump(docs, open(out, "w"), default=str)
print(f"{len(docs)} scenarios -> {out}")
