"""get_card: Oracle text (from the Forge scripts at the engine's commit) and rulings for the closed card pool. Static data only."""
import difflib, json, os, re

DATA = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "data")
_cards = _rulings = _index = None


def _load():
    global _cards, _rulings, _index
    if _cards is None:
        _cards = json.load(open(os.path.join(DATA, "cards.json")))["cards"]
        _rulings = json.load(open(os.path.join(DATA, "rulings.json")))
        _index = {}
        for n, c in _cards.items():
            for f in c["faces"]: _index[norm(f["name"])] = n
            _index[norm(n)] = n
    return _cards


def norm(s):
    s = re.sub(r"#\d+$", "", s.strip())                      # a serializer id suffix, 'Island#12'
    return re.sub(r"[^a-z0-9]+", " ", s.lower().replace("'", "").replace("’", "")).strip()


def all_names():
    return sorted(_load())


def find(name):
    """-> (canonical name or None, suggestions)"""
    cards = _load()
    n = _index.get(norm(name))
    if n: return n, []
    key = norm(name)
    pref = [c for c in cards if norm(c).startswith(key)] if key else []
    if len(pref) == 1: return pref[0], []
    near = difflib.get_close_matches(key, list(_index), n=4, cutoff=0.6)
    return None, sorted({_index[x] for x in near} | set(pref))


def card(name):
    cards = _load()
    n, sug = find(name)
    if n is None: return {"error": f"not in the card pool: {name}", "suggestions": sug}
    c = cards[n]
    r = _rulings["rulings"].get(n, [])
    return {"name": n, "faces": c["faces"], "rulings": r, "rulings_note": _rulings["note"]}


def render(c):
    if "error" in c: return c["error"] + (" | did you mean: " + "; ".join(c["suggestions"]) if c["suggestions"] else "")
    lines = []
    for f in c["faces"]:
        head = f["name"] + (" " + f["mana_cost"] if f["mana_cost"] else "") + " | " + f["type_line"]
        if f["pt"]: head += " " + f["pt"]
        if f["loyalty"]: head += " loyalty " + f["loyalty"]
        if f["mana_cost"]: head += f" | mana value {f['mana_value']}"
        lines += [head, f["oracle"]]
    if c["rulings"]: lines += ["Rulings (" + c["rulings_note"] + "):"] + ["- " + r for r in c["rulings"]]
    return "\n".join(lines)


def type_set(name):
    """Card types of the front face, for type: queries."""
    cards = _load()
    n, _ = find(name)
    return set(cards[n]["faces"][0]["types"].split()) if n else set()


def mana_value(name):
    cards = _load()
    n, _ = find(name)
    return cards[n]["faces"][0]["mana_value"] if n else None
