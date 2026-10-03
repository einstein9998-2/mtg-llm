"""Helpers for tests that replay the recorded Alurentell vs UR Cutter games (llm-player/runs/aNN). Only the `prompt` field (what the player saw) is
fed to the tools. The `truth` field (engine ground truth) is read here as an answer key to grade the tools, and never passed to them."""
import collections, glob, json, os, re, sys
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, ROOT)
from mtgtools import view as V
RUNS = "/mnt/project-files/llm-player/runs"

def runs():
    return [d for d in sorted(glob.glob(RUNS + "/a[0-9][0-9]")) if os.path.exists(d + "/decisions.jsonl")]

def replay(run):
    """yield (row, view after folding that prompt, true opponent hand Counter). The view is a fresh object each step's snapshot is the live one: copy if kept."""
    v = V.empty_view()
    with open(run + "/decisions.jsonl") as f: lines = f.readlines()
    for line in lines:
        try: r = json.loads(line)
        except json.JSONDecodeError: break      # a run still being written
        V.fold(v, r["prompt"], key=(r["g"], r["d"]))
        m = re.search(r"oppHand=(.*?),? ?myLibTop=", r["truth"])
        hand = collections.Counter(x.strip() for x in (m.group(1).split(",") if m else []) if x.strip())
        yield r, v, hand
