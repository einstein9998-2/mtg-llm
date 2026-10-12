#!/usr/bin/env python3
"""Best-of-three match driver for the LLM players.

match.py prep   MATCH GAME FIRST   -> writes the boarded decks for game GAME (1..3) and a launch line
        FIRST = 'A' or 'B' (who plays first); A = Alurentell (seat 0), B = the opponent deck (seat 1).
match.py result MATCH GAME A|B     -> records the winner of a game

Boarding follows /mnt/project-files/rust-engine/sideboard-plans (one for one, 60 main, applied to the base 75 of each deck).
"""
import json, os, re, subprocess, sys

W = "/home/claude/work/llm2"
PLANS = "/mnt/project-files/rust-engine/sideboard-plans"
# sideboard experiments: PLANS_A = dir holding alurentell.txt for the arm under test, PLANS_B = dir holding the opponent plan
PLANS_A = os.environ.get("PLANS_A"); PLANS_B = os.environ.get("PLANS_B")
BASE = f"{W}/decks-base"
OPP = os.environ.get("OPP", "ur-cutter")
BASE_A = os.environ.get("BASE_A", BASE)  # directory holding Alurentell's base 75 for the arm under test


def read_deck(path):
    main, side, cur = {}, {}, None
    cur = main
    for ln in open(path):
        ln = ln.rstrip("\n")
        if ln.startswith("#"):
            continue
        if not ln.strip():
            if main:
                cur = side
            continue
        m = re.match(r"(\d+) (.+)", ln)
        cur[m.group(2)] = cur.get(m.group(2), 0) + int(m.group(1))
    return main, side


def plan(deck, vs, side="A"):
    ops, on = [], False
    d = (PLANS_A if side == "A" else PLANS_B) or PLANS
    if not os.path.exists(f"{d}/{deck}.txt"):
        return ops
    for ln in open(f"{d}/{deck}.txt"):
        ln = ln.rstrip("\n")
        if ln.startswith("vs "):
            on = ln[3:].strip() == vs
            continue
        if on and re.match(r"[+-]\d+ ", ln):
            ops.append((int(ln.split()[0]), ln.split(" ", 1)[1]))
    return ops


def board(deck, vs, pside="A"):
    main, side = read_deck(f"{BASE_A if deck == 'alurentell' else BASE}/{deck}.txt")
    for n, card in plan(deck.replace("-b", "") if deck == "alurentell-b" else deck, vs, pside):
        if n < 0:
            assert main.get(card, 0) >= -n, (deck, card)
            main[card] -= -n
            side[card] = side.get(card, 0) + -n
        else:
            assert side.get(card, 0) >= n, (deck, card)
            side[card] -= n
            main[card] = main.get(card, 0) + n
    assert sum(main.values()) == 60 and sum(side.values()) == 15, (deck, sum(main.values()), sum(side.values()))
    return {k: v for k, v in main.items() if v}, {k: v for k, v in side.items() if v}


def write(path, main, side):
    with open(path, "w") as f:
        for k, v in main.items():
            f.write(f"{v} {k}\n")
        f.write("\n")
        for k, v in side.items():
            f.write(f"{v} {k}\n")


def state_path(m):
    return f"{W}/matches/{m}/state.json"


def main():
    cmd, m = sys.argv[1], sys.argv[2]
    os.makedirs(f"{W}/matches/{m}", exist_ok=True)
    st = json.load(open(state_path(m))) if os.path.exists(state_path(m)) else {"games": []}
    if cmd == "prep":
        g, first = int(sys.argv[3]), sys.argv[4]
        d = f"{W}/matches/{m}/g{g}"
        os.makedirs(f"{d}/engine-decks", exist_ok=True)
        if g == 1:
            ma, sa = read_deck(f"{BASE_A}/alurentell.txt")
            mb, sb = read_deck(f"{BASE}/{OPP}.txt")
        else:
            ma, sa = board("alurentell", OPP.replace("alurentell-b", "alurentell"), "A")
            mb, sb = board(OPP, "alurentell", "B")
        write(f"{d}/engine-decks/alurentell.txt", ma, sa)
        write(f"{d}/engine-decks/{OPP}.txt", mb, sb)
        write(f"{d}/a-decklist.txt", ma, sa)
        write(f"{d}/b-decklist.txt", mb, sb)
        st.setdefault("first", {})[str(g)] = first
        json.dump(st, open(state_path(m), "w"))
        seed = 7000 + 10 * int(re.sub(r"\D", "", m) or 0) + g
        print(f"LLM_DECKS={d}/engine-decks LLM_DECK=alurentell LLM_OPP={OPP} {W}/lg.sh launch2 {m}g{g} {seed} 0 {0 if first == 'A' else 1}")
        print(f"swaps A: {plan('alurentell', OPP) if g > 1 else 'none (main deck)'}")
        pass
    elif cmd == "result":
        g, w = int(sys.argv[3]), sys.argv[4]
        st["games"] = st["games"][: g - 1] + [w]
        json.dump(st, open(state_path(m), "w"))
        wa, wb = st["games"].count("A"), st["games"].count("B")
        print(f"games {st['games']}  A {wa} - B {wb}", "MATCH OVER" if max(wa, wb) >= 2 else f"next: game {g+1}, first = {'B' if w == 'A' else 'A'} (loser)")


main()
