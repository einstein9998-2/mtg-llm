"""The player's own view: a running fold of the prompts the serializer showed to this seat.

This is the ONLY game input the tools take. A prompt carries full state at turn start and changed-only sections afterwards, so the view
keeps the latest value of every section. Nothing here reads engine state, engine logs or any file other than the view file itself.
"""
import json, os, re, tempfile

SECTIONS = ("hand", "ybf", "obf", "ygy", "ogy", "cnt", "stack")  # a prompt with all of these is a full state
ITEM = re.compile(r"(?:^|, |; )(?:lands: )?(.+?)#(\d+)((?: \d+/\d+)?(?: \([^)]*\))?)")


def parse_cards(text):
    """'Atraxa, Grand Unifier#40 7/7 (sick), Island#3 (T); lands: Forest#9' -> [{'name', 'id', 'flags'}]. '-' and '' -> []."""
    text = text.strip()
    if text in ("", "-"): return []
    out = []
    for m in ITEM.finditer(text):
        fm = re.search(r"\(([^)]*)\)", m.group(3))
        out.append({"name": m.group(1).strip(), "id": int(m.group(2)), "flags": fm.group(1) if fm else ""})
    return out


def parse_stack(text):
    """Only spells (cards on the stack) matter for zone accounting; abilities and triggers are kept as kind != 'spell'."""
    text = text.strip()
    if text == "empty": return []
    out = []
    for part in text.split("; "):
        part = part.split(" -> ")[0].strip()
        kind = "spell"
        for pre, k in (("trigger of ", "trigger"), ("ability of ", "ability")):
            if part.startswith(pre): part, kind = part[len(pre):], k
        m = re.match(r"^(.+?)#(\d+) \((you|opp)\)", part)
        if m: out.append({"name": m.group(1), "id": int(m.group(2)), "owner": m.group(3), "kind": kind})
    return out


def empty_view():
    return {"turn": None, "phase": "", "my_turn": None, "life_you": None, "life_opp": None, "hand": [], "ybf": [], "obf": [], "ygy": [], "ogy": [],
            "yex": [], "oex": [], "opp_hand_n": 7, "opp_hand_known": [], "opp_lib_n": None, "my_lib_n": None, "stack": [],
            "have": [], "last_key": None, "last_seq": None, "stale": True, "stale_reason": "no state seen yet", "prompts": 0}


def load(path):
    try: return json.load(open(path))
    except (FileNotFoundError, json.JSONDecodeError): return empty_view()


def save(path, v):
    d = os.path.dirname(os.path.abspath(path))
    fd, tmp = tempfile.mkstemp(dir=d, prefix=".view-")
    with os.fdopen(fd, "w") as f: json.dump(v, f)
    os.replace(tmp, path)


def fold(v, prompt, key=None, seq=None):
    """Fold one prompt into the view in place. key=(game, decision id) makes the call idempotent; seq (the broker's message counter), if given,
    flags a gap, after which the view is stale until the next full-state prompt."""
    if key is not None:
        if v["last_key"] == list(key): return v
        v["last_key"] = list(key)
    if seq is not None:
        if v["last_seq"] is not None and seq != v["last_seq"] + 1 and not v["stale"]:
            v["stale"], v["stale_reason"] = True, f"missed prompt(s) between seq {v['last_seq']} and {seq}"
        v["last_seq"] = seq
    v["prompts"] += 1
    seen = set()
    ex_line = None
    for line in prompt.split("\n"):
        m = re.match(r"^(?:T(\d+) (your|opp) turn, (\S+)|pre-game) \| life you (-?\d+) opp (-?\d+)", line)
        if m:
            v["turn"], v["my_turn"] = (int(m.group(1)), m.group(2) == "your") if m.group(1) else (0, None)
            v["phase"] = m.group(3) or "pre-game"
            v["life_you"], v["life_opp"] = int(m.group(4)), int(m.group(5))
        elif line.startswith("YOUR HAND ("): v["hand"] = parse_cards(line.split("): ", 1)[1]) if "): " in line else []; seen.add("hand")
        elif line.startswith("YOUR BF: "): v["ybf"] = parse_cards(line[9:]); seen.add("ybf")
        elif line.startswith("OPP BF: "): v["obf"] = parse_cards(line[8:]); seen.add("obf")
        elif line.startswith("YOUR GY: "): v["ygy"] = parse_cards(line[9:]); seen.add("ygy")
        elif line.startswith("OPP GY: "): v["ogy"] = parse_cards(line[8:]); seen.add("ogy")
        elif line.startswith("EXILE: "): ex_line = line[7:]
        elif line.startswith("OPP HAND "):
            m = re.match(r"^OPP HAND (\d+)(?: \(known: (.*?)\))?, OPP LIB (\d+) \| YOUR LIB (\d+)", line)
            if m:
                v["opp_hand_n"], v["opp_hand_known"] = int(m.group(1)), parse_cards(m.group(2) or "")
                v["opp_lib_n"], v["my_lib_n"] = int(m.group(3)), int(m.group(4))
                seen.add("cnt")
        elif line.startswith("STACK (top first): "): v["stack"] = parse_stack(line[19:]); seen.add("stack")
        else:
            m = re.match(r"^D\d+ Mulligan: opening hand \(\d+\): (.*?)(?: \| |$)", line)
            if m: v["hand"] = parse_cards(m.group(1)); seen.add("hand")
    if ex_line is not None:
        m = re.match(r"^(?:-|yours (.*?) \| opp (.*))$", ex_line.strip())
        v["yex"], v["oex"] = (parse_cards(m.group(1)), parse_cards(m.group(2))) if m and m.group(1) is not None else ([], [])
    elif set(SECTIONS) <= seen: v["yex"], v["oex"] = [], []  # a full state omits EXILE when both sides are empty
    v["have"] = sorted(set(v["have"]) | seen)
    if set(SECTIONS) <= seen: v["stale"], v["stale_reason"] = False, ""
    return v


def fold_message(v, msg):
    """msg: the External-policy decision message ({'game','id','prompt', optional 'seq'}), e.g. the broker's cur.json."""
    return fold(v, msg["prompt"], key=(msg.get("game", 0), msg["id"]), seq=msg.get("seq"))


def names(cards): return [c["name"] for c in cards]


def summary(v):
    return (f"T{v['turn']} {'your' if v['my_turn'] else 'opp'} {v['phase']} | life {v['life_you']}-{v['life_opp']} | hand {len(v['hand'])}, "
            f"opp hand {v['opp_hand_n']}, libs you {v['my_lib_n']} opp {v['opp_lib_n']} | " + ("STALE: " + v["stale_reason"] if v["stale"] else "view complete"))
