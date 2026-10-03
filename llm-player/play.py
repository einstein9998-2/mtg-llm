#!/usr/bin/env python3
"""Broker between the Forge serializer (External policy, JSONL over named pipes) and an interactive LLM player.

  play.py start <tag> <seed> [stops] [deckA.dck deckB.dck]   launch engine + broker in the background; run dir = runs/<tag>
  play.py next                      block until the next decision (or game over) and print it
  play.py pick <i[,j..]> [note..]   answer the pending decision, log the note, print the next decision

The player only ever sees the prompt text the serializer sends. Engine-side logs (decisions.jsonl, which also holds the Forge AI's
shadow pick and ground truth for post-hoc review) are never printed by this tool.
"""
import json, os, subprocess, sys, time

HERE = os.path.dirname(os.path.abspath(__file__))
RUNS = os.path.join(HERE, "runs")
STATE = os.path.join("/tmp/claude-0/-home-claude/c7ffc76b-a18f-57e7-ae47-4ad97926fc8f/scratchpad", "broker")  # broker handshake files (not logs)
os.makedirs(STATE, exist_ok=True)

def serve(tag, seed, stops="default", deck_a="dnt.dck", deck_b="jund75.dck"):
    run = os.path.join(RUNS, tag); os.makedirs(run, exist_ok=True)
    p_out, p_in = os.path.join(STATE, "e2d"), os.path.join(STATE, "d2e")
    for p in (p_out, p_in):
        if os.path.exists(p): os.remove(p)
        os.mkfifo(p)
    for f in ("cur.json", "ans.json", "over.json"):
        try: os.remove(os.path.join(STATE, f))
        except FileNotFoundError: pass
    env = dict(os.environ, FORGE_ROOT="/home/claude/card-forge/forge", OUT="/home/claude/silo-build")
    eng = subprocess.Popen(["sh", "/home/claude/ser-new/run.sh", "--a", deck_a, "--b", deck_b, "--games", "1", "--seed", str(seed),
                            "--policy", "external", "--stops", stops, "--log-prompts", "true", "--shadow-hint", "true", "--timeout", "100000",
                            "--pipe-out", p_out, "--pipe-in", p_in, "--out", run],
                           stdout=open(os.path.join(run, "engine.out"), "w"), stderr=subprocess.STDOUT, env=env, cwd=HERE)
    rd = open(p_out, "r"); wr = open(p_in, "w")
    my = open(os.path.join(run, "player.jsonl"), "a")
    seq = 0
    while True:
        line = rd.readline()
        if not line: break
        m = json.loads(line)
        if m.get("type") == "bye": break
        seq += 1
        m["seq"] = seq
        tmp = os.path.join(STATE, "cur.tmp"); json.dump(m, open(tmp, "w")); os.replace(tmp, os.path.join(STATE, "cur.json"))
        ap = os.path.join(STATE, "ans.json")
        while not os.path.exists(ap):
            time.sleep(0.05)
            if eng.poll() is not None: break
        if not os.path.exists(ap): break
        a = json.load(open(ap)); os.remove(ap)
        my.write(json.dumps({"seq": seq, "id": m["id"], "kind": m["kind"], "n": m["n"], "pick": a["pick"], "note": a.get("note", "")}) + "\n"); my.flush()
        wr.write(json.dumps({"pick": a["pick"]}) + "\n"); wr.flush()
    eng.wait(timeout=60)
    res = {"engine_exit": eng.returncode}
    try:
        res["game"] = json.loads(open(os.path.join(run, "games.jsonl")).readline())
    except Exception as e:
        res["err"] = str(e)
    json.dump(res, open(os.path.join(STATE, "over.json"), "w"))

def show(after_seq):
    cur, over = os.path.join(STATE, "cur.json"), os.path.join(STATE, "over.json")
    while True:
        if os.path.exists(over):
            print("GAME OVER", open(over).read()); return
        if os.path.exists(cur):
            try: m = json.load(open(cur))
            except Exception: m = None
            if m and m["seq"] > after_seq:
                print(f"[decision seq={m['seq']} id={m['id']} kind={m['kind']} pick {m['min']}..{m['max']} of {m['n']}]")
                print(m["prompt"]); return
        time.sleep(0.1)

def curseq():
    try: return json.load(open(os.path.join(STATE, "cur.json")))["seq"]
    except Exception: return 0

if __name__ == "__main__":
    cmd = sys.argv[1]
    if cmd == "serve": serve(sys.argv[2], sys.argv[3], *(sys.argv[4:7]))
    elif cmd == "start":
        tag, seed = sys.argv[2], sys.argv[3]; stops = sys.argv[4] if len(sys.argv) > 4 else "default"; decks = sys.argv[5:7]
        for f in ("cur.json", "over.json", "ans.json"):
            try: os.remove(os.path.join(STATE, f))
            except FileNotFoundError: pass
        subprocess.Popen([sys.executable, os.path.abspath(__file__), "serve", tag, seed, stops, *decks], stdout=open(os.path.join(STATE, "serve.log"), "w"),
                         stderr=subprocess.STDOUT, start_new_session=True)
        print("started", tag)
    elif cmd == "next": show(0)
    elif cmd == "pick":
        s = curseq()
        pick = [int(x) for x in sys.argv[2].split(",") if x.strip() != ""]
        try:
            cur = json.load(open(os.path.join(STATE, "cur.json")))
            if not (cur["min"] <= len(pick) <= cur["max"]) or any(i < 0 or i >= cur["n"] for i in pick) or len(set(pick)) != len(pick):
                print(f"REJECTED: need {cur['min']}..{cur['max']} distinct indices in 0..{cur['n']-1}, got {pick}. Nothing was sent."); sys.exit(1)
        except FileNotFoundError:
            pass
        json.dump({"pick": pick, "note": " ".join(sys.argv[3:])}, open(os.path.join(STATE, "ans.tmp"), "w"))
        os.replace(os.path.join(STATE, "ans.tmp"), os.path.join(STATE, "ans.json"))
        show(s)
