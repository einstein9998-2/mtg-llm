#!/usr/bin/env python3
"""Validate scenario files against schema/scenario.schema.json plus semantic checks.

Usage: validate.py [--oracle reference/oracle.json] [--refs reference] PATH [PATH ...]
PATH is a .yaml file or a directory (searched recursively for *.yaml).
Exit code 1 if any error is found. Warnings do not fail the run.
"""
import sys, os, re, json, argparse, glob
import yaml
import jsonschema

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)

STEP_KEYS = {"p0", "p1", "advance", "resolve_top", "random", "bind", "check", "expect_illegal"}
STEP_META = {"label", "note"}
PHASES = {"untap","upkeep","draw","main1","begin_combat","declare_attackers","declare_blockers",
          "first_strike_damage","combat_damage","end_combat","main2","end","cleanup"}
ACTION_T = {"pass","play_land","cast","activate","activate_from_hand","declare_attackers","declare_blockers"}
DECISIONS = {"choose_cards","order_cards","yes_no","choose_target","choose_dungeon","choose_room","order_triggers",
             "legend_rule","choose_color","choose_mode","choose_number","surveil","scry","search",
             "simultaneous_secret_choice","assign_combat_damage","declare_attackers","declare_blockers",
             "choose_player","choose_name","order_replacements","choose_x","choose_alt_cost","choose_optional_cost","pay_mana",
             "mulligan","bottom_cards","choose_option"}
ALIAS_RE = re.compile(r"^@([A-Za-z0-9_]+)$")
SHORT_RE = re.compile(r"^(.*?)\s+@([A-Za-z0-9_]+)$")

class Ctx:
    def __init__(self, oracle, ref_text):
        self.cards = set(oracle["cards"].keys())
        self.tokens = set(oracle["tokens"].keys())
        self.dungeons = set(oracle["dungeons"].keys())
        self.oracle = oracle
        self.ref_text = ref_text

def load_refs(refdir):
    txt = ""
    for f in glob.glob(os.path.join(refdir, "cr-excerpts*.md")):
        txt += open(f, encoding="utf8").read() + "\n"
    return txt

def norm(s):
    return re.sub(r"\s+", " ", s.replace("’", "'").replace("“", '"').replace("”", '"').replace("—", "-").replace("—", "-")).strip().lower()

def entry_name_alias(e):
    """Return (name, alias, dict) for a setup zone entry."""
    if isinstance(e, str):
        m = SHORT_RE.match(e)
        if m:
            return m.group(1).strip(), m.group(2), {}
        return e.strip(), None, {}
    if isinstance(e, dict):
        name = e.get("card") or e.get("token") or e.get("dungeon")
        return name, e.get("as"), e
    return None, None, {}

def collect_strings(obj, out):
    if isinstance(obj, str):
        out.append(obj)
    elif isinstance(obj, list):
        for x in obj: collect_strings(x, out)
    elif isinstance(obj, dict):
        for k, v in obj.items():
            out.append(("KEY", k))
            collect_strings(v, out)

def validate_doc(doc, ctx, where, errs, warns):
    def E(msg): errs.append(f"{where}: {msg}")
    def W(msg): warns.append(f"{where}: {msg}")
    schema = json.load(open(os.path.join(ROOT, "schema", "scenario.schema.json")))
    for e in jsonschema.Draft202012Validator(schema).iter_errors(doc):
        E("schema: " + "/".join(map(str, e.absolute_path)) + ": " + e.message[:200])
    if not isinstance(doc, dict):
        return
    cards_decl = set(doc.get("cards", []) or [])
    for c in cards_decl:
        if c not in ctx.cards and c not in ctx.tokens:
            E(f"cards: unknown card name {c!r} (not in reference/oracle.json)")
    # sources
    for i, s in enumerate(doc.get("source", []) or []):
        k = s.get("kind")
        if k == "oracle" and s.get("verified"):
            cn = s.get("card")
            if cn not in ctx.cards:
                E(f"source[{i}]: verified oracle source needs a known card, got {cn!r}")
            else:
                if norm(s["text"]).rstrip(".") not in norm(ctx.oracle["cards"][cn]["oracle_text"]) :
                    E(f"source[{i}]: verified oracle text not found in oracle.json for {cn}")
        if k == "cr":
            if not s.get("ref"):
                E(f"source[{i}]: cr source needs ref")
            elif s.get("verified"):
                ref = re.escape(s["ref"].strip())
                if not re.search(r"(^|\n)\s*(##\s*)?" + ref + r"[ \.]", ctx.ref_text):
                    E(f"source[{i}]: cr {s['ref']} marked verified but not present in reference/cr-excerpts*.md")
                elif s.get("text"):
                    # quote check: every fragment (split on ellipses) must appear in the excerpts
                    nref = norm(ctx.ref_text)
                    for frag in re.split(r"\.\.\.|\u2026|\[\.\.\.\]", s["text"]):
                        f = norm(frag).strip(" .")
                        if len(f) > 8 and f not in nref:
                            E(f"source[{i}]: cr {s['ref']} quoted text not found in cr-excerpts*.md: {f[:70]!r}")
                            break
        if k == "ruling" and not s.get("card"):
            E(f"source[{i}]: ruling needs card")
    if doc.get("derived_from") == "ruling" and not any(s.get("kind") == "ruling" for s in doc.get("source", [])):
        E("derived_from=ruling but no ruling source")
    if doc.get("derived_from") == "cr" and not any(s.get("kind") == "cr" for s in doc.get("source", [])):
        E("derived_from=cr but no cr source")
    if doc.get("derived_from") == "oracle" and not any(s.get("kind") == "oracle" for s in doc.get("source", [])):
        E("derived_from=oracle but no oracle source")

    setup = doc.get("setup") or {}
    aliases = {}      # alias -> name
    used_names = set()
    if setup.get("phase") not in PHASES:
        E(f"setup.phase invalid: {setup.get('phase')!r}")
    if setup.get("active") not in ("p0", "p1"):
        E("setup.active must be p0 or p1")
    if not isinstance(setup.get("turn"), int):
        E("setup.turn must be an int")
    for seat in ("p0", "p1"):
        s = setup.get(seat)
        if s is None:
            E(f"setup.{seat} missing"); continue
        for zone in ("hand", "battlefield", "graveyard", "exile", "library", "command"):
            for e in s.get(zone, []) or []:
                name, alias, d = entry_name_alias(e)
                if name is None:
                    E(f"setup.{seat}.{zone}: unreadable entry {e!r}"); continue
                if zone == "command" and isinstance(e, dict) and "dungeon" in e:
                    if name not in ctx.dungeons: E(f"setup.{seat}.command: unknown dungeon {name!r}")
                    continue
                if isinstance(e, dict) and "token" in e:
                    if name not in ctx.tokens: E(f"setup.{seat}.{zone}: unknown token {name!r}")
                elif name not in ctx.cards:
                    E(f"setup.{seat}.{zone}: unknown card {name!r}")
                used_names.add(name)
                if alias:
                    if alias in aliases: E(f"duplicate alias @{alias}")
                    aliases[alias] = name
    for n in used_names:
        if n not in cards_decl and n not in ctx.tokens:
            E(f"card {n!r} used in setup but missing from `cards`")

    # script
    script = doc.get("script") or []
    labels = set()
    all_refs = []
    def check_refs(obj, where2, defined):
        out = []
        collect_strings(obj, out)
        for x in out:
            if isinstance(x, tuple):
                continue
            m = ALIAS_RE.match(x.strip())
            if m and m.group(1) not in defined:
                E(f"{where2}: alias @{m.group(1)} used before definition")
            for h in re.findall(r"@([A-Za-z0-9_]+)", x):
                pass
    defined = set(aliases)
    for i, st in enumerate(script):
        w = f"script[{i}]"
        if not isinstance(st, dict):
            E(f"{w}: step must be a mapping"); continue
        keys = set(st) - STEP_META
        if len(keys & STEP_KEYS) != 1 or len(keys - STEP_KEYS) > 0:
            E(f"{w}: step must have exactly one of {sorted(STEP_KEYS)} (+label/note), got {sorted(keys)}")
            continue
        if "label" in st:
            if st["label"] in labels: E(f"{w}: duplicate label {st['label']}")
            labels.add(st["label"])
        k = next(iter(keys & STEP_KEYS))
        v = st[k]
        if k in ("p0", "p1"):
            if not isinstance(v, dict):
                E(f"{w}: action/decision must be a mapping"); continue
            if "t" in v:
                if v["t"] not in ACTION_T: E(f"{w}: unknown action t={v['t']!r}")
                if v["t"] == "cast" and "card" not in v: E(f"{w}: cast needs card")
                if v["t"] == "cast":
                    # alt cost exclusivity is a scenario about illegality
                    ac = v.get("alt_cost")
                    if isinstance(ac, dict) and len(ac) > 1 and not ({"exile_blue_card","pay_life"} >= set(ac)):
                        E(f"{w}: alt_cost names more than one alternative cost (CR 118.9a); use expect_illegal")
            elif "decision" in v:
                if v["decision"] not in DECISIONS: E(f"{w}: unknown decision kind {v['decision']!r}")
            else:
                E(f"{w}: needs `t` (action) or `decision`")
            check_refs(v, w, defined)
        elif k == "advance":
            if not isinstance(v, dict) or v.get("to") not in PHASES or v.get("of") not in ("p0", "p1"):
                E(f"{w}: advance needs to: <step> and of: p0|p1")
        elif k == "bind":
            if not isinstance(v, dict) or "as" not in v or "find" not in v:
                E(f"{w}: bind needs as and find")
            else:
                if v["as"] in defined: E(f"{w}: alias @{v['as']} already defined")
                defined.add(v["as"])
        elif k == "check":
            check_refs(v, w, defined)
            if isinstance(v, dict):
                _l = []
                check_expect_keys(v, w, _l)
                for m in _l: E(m)
        elif k == "expect_illegal":
            check_refs(v, w, defined)
        elif k == "random":
            pass
    # names used in script card fields etc
    strs = []
    collect_strings([setup, script, doc.get("expect")], strs)
    for x in strs:
        if isinstance(x, tuple):
            continue
        xs = x.strip()
        if xs in ctx.cards and xs not in cards_decl:
            E(f"card name {xs!r} appears in the scenario but is missing from `cards`")
    # expect
    exp = doc.get("expect") or []
    for i, e in enumerate(exp):
        at = e.get("at")
        if at != "after_script" and at not in labels:
            E(f"expect[{i}]: at={at!r} is neither after_script nor a step label")
        check_refs(e, f"expect[{i}]", defined)
        check_expect_keys(e, f"expect[{i}]", errs_local := [])
        for m in errs_local: E(m)
        if len(e) < 2:
            E(f"expect[{i}]: no assertions")
    return

GLOBAL_KEYS = {"at","game","turn","phase","active","priority","stack","pending","legal","events_contain","observer","obs","p0","p1","note"}
SEAT_KEYS = {"life","energy","hand","hand_count","library_count","library_top","library_bottom","library_bottom_multiset",
             "graveyard","graveyard_ordered","exile","battlefield","battlefield_contains","mana_pool","lands_played",
             "spells_cast_this_turn","command","completed_dungeons","designations","known_to_me","poison","cards_drawn_this_turn"}
DESC_KEYS = {"attacking","card","token","controller","tapped","counters","pt","damage","types","keywords","keywords_exclude","attached_to","sick",
             "alias","face_up","as","phased","count","colors","subtypes","supertypes","power","toughness","note","flags"}
STACK_KEYS = {"kind","card","controller","targets","modes","x","source","text_contains","copy","note"}

def check_expect_keys(e, w, errs):
    def bad(msg): errs.append(f"{w}: {msg}")
    for k in e:
        if k not in GLOBAL_KEYS: bad(f"unknown expectation key {k!r}")
    for seat in ("p0","p1"):
        s = e.get(seat)
        if isinstance(s, dict):
            for k, v in s.items():
                if k not in SEAT_KEYS: bad(f"unknown {seat} expectation key {k!r}")
                if k in ("battlefield","battlefield_contains","exile") and isinstance(v, list):
                    for d in v:
                        if isinstance(d, dict):
                            for kk in d:
                                if kk not in DESC_KEYS: bad(f"unknown descriptor key {kk!r} in {seat}.{k}")
    st = e.get("stack")
    if isinstance(st, list):
        for d in st:
            if isinstance(d, dict):
                for kk in d:
                    if kk not in STACK_KEYS: bad(f"unknown stack entry key {kk!r}")

def iter_files(paths):
    for p in paths:
        if os.path.isdir(p):
            for dp, dn, fn in os.walk(p):
                for f in sorted(fn):
                    if f.endswith((".yaml", ".yml")):
                        yield os.path.join(dp, f)
        else:
            yield p

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--oracle", default=os.path.join(ROOT, "reference", "oracle.json"))
    ap.add_argument("--refs", default=os.path.join(ROOT, "reference"))
    ap.add_argument("paths", nargs="+")
    a = ap.parse_args()
    oracle = json.load(open(a.oracle, encoding="utf8"))
    ctx = Ctx(oracle, load_refs(a.refs))
    errs, warns, ids, n = [], [], {}, 0
    for f in iter_files(a.paths):
        try:
            docs = list(yaml.safe_load_all(open(f, encoding="utf8")))
        except Exception as ex:
            errs.append(f"{f}: YAML error: {ex}"); continue
        for j, d in enumerate(docs):
            if d is None: continue
            n += 1
            where = f"{os.path.relpath(f)}#{j}" + (f"({d.get('id')})" if isinstance(d, dict) else "")
            validate_doc(d, ctx, where, errs, warns)
            if isinstance(d, dict) and "id" in d:
                if d["id"] in ids: errs.append(f"{where}: duplicate id also in {ids[d['id']]}")
                ids[d["id"]] = where
    for w in warns: print("WARN ", w)
    for e in errs: print("ERROR", e)
    print(f"{n} scenario(s), {len(errs)} error(s), {len(warns)} warning(s)")
    sys.exit(1 if errs else 0)

if __name__ == "__main__":
    main()
