"""Command line entry points. Each tool takes only the player's own view (see view.py), static card data, decklists and the notes directory."""
import argparse, json, os, re, sys
from collections import Counter

from . import calc, cards, decks, notes, view as V

DEFAULT_VIEW = os.environ.get("MTG_VIEW", "view.json")


def _view(a, fold_required=False):
    v = V.load(a.view)
    if getattr(a, "cur", None):
        V.fold_message(v, json.load(open(a.cur)))
        V.save(a.view, v)
    return v


def _common(p, deck=None):
    p.add_argument("--view", default=DEFAULT_VIEW, help="view file (default $MTG_VIEW or ./view.json)")
    p.add_argument("--cur", help="also fold this pending decision message (the broker's cur.json) into the view first")
    p.add_argument("--json", action="store_true", help="machine-readable output")
    if deck == "my": p.add_argument("--my-deck", default=os.environ.get("MTG_MY_DECK"), help="your decklist: name in the decks dir or a path ($MTG_MY_DECK)")
    if deck == "opp": p.add_argument("--opp-deck", default=os.environ.get("MTG_OPP_DECK"), help="the opponent's decklist ($MTG_OPP_DECK)")


def _out(a, obj, text):
    print(json.dumps(obj, indent=1) if a.json else text)


def cmd_view_update(a):
    v = V.load(a.view)
    if a.cur: V.fold_message(v, json.load(open(a.cur)))
    else:
        text = sys.stdin.read() if a.text in (None, "-") else open(a.text).read()
        V.fold(v, text, seq=a.seq)
    V.save(a.view, v)
    _out(a, {"summary": V.summary(v)}, V.summary(v))


def cmd_get_card(a):
    res = [cards.card(n) for n in a.names]
    _out(a, res, "\n\n".join(cards.render(c) for c in res))
    return 1 if any("error" in c for c in res) else 0


def groups_from_terms(terms, pool_names):
    """card name -> itself; type:Land -> all pool cards with that type; mv<=3 / mv>=2 / mv=0 -> by mana value (nonland cards)."""
    groups = {}
    for t in terms:
        m = re.match(r"^type:(\w+)$", t, re.I)
        if m:
            ty = m.group(1).capitalize()
            groups[t] = {n for n in pool_names if ty in cards.type_set(n)}
            continue
        m = re.match(r"^mv(<=|>=|=|<|>)(\d+)$", t)
        if m:
            op, k = m.group(1), int(m.group(2))
            f = {"<=": lambda x: x <= k, ">=": lambda x: x >= k, "=": lambda x: x == k, "<": lambda x: x < k, ">": lambda x: x > k}[op]
            groups[t] = {n for n in pool_names if "Land" not in cards.type_set(n) and f(cards.mana_value(n))}
            continue
        n, sug = cards.find(t)
        if n is None: raise SystemExit(f"unknown card: {t}" + (" (did you mean: " + "; ".join(sug) + ")" if sug else ""))
        groups[n] = {n}
    return groups


def cmd_library_odds(a):
    if not a.my_deck: raise SystemExit("need --my-deck (or $MTG_MY_DECK)")
    v = _view(a)
    deck = decks.load(a.my_deck)["main"]
    rem, warnings = calc.my_library(v, deck, a.top or [], a.bottom or [])
    draws = [int(x) for x in a.draws.split(",")]
    res = {"library_size": v["my_lib_n"] if v["my_lib_n"] is not None else sum(rem.values()), "known_top": a.top or [], "known_bottom": a.bottom or [],
           "warnings": warnings, "view": V.summary(v), "remaining": dict(sorted(rem.items())), "queries": {}}
    lines = [f"Your library: {res['library_size']} cards ({V.summary(v)})"] + ["WARNING: " + w for w in warnings]
    if a.list:
        lines.append("Remaining (deck minus seen): " + ", ".join(f"{n} x{k}" for n, k in sorted(rem.items())))
    if a.terms:
        groups = groups_from_terms(a.terms, set(rem) | set(deck))
        for d in draws:
            r = calc.odds(rem, groups, d, top=a.top or [], bottom=a.bottom or [])
            res["queries"][str(d)] = r
            lines.append(f"-- seeing the top {d} cards" + (f" ({len(a.top)} known on top)" if a.top else ""))
            for label in groups:
                x = r[label]
                lines.append(f"{label}: {x['in_library']} in library | P(at least one) {x['p_at_least_one']:.1%} | expected copies {x['expected_copies']:.2f}")
            if "__all__" in r: lines.append(f"all of them: {r['__all__']:.1%} | any of them: {r['__any__']:.1%}")
    _out(a, res, "\n".join(lines))


def cmd_sample_opp_hands(a):
    if not a.opp_deck: raise SystemExit("need --opp-deck (or $MTG_OPP_DECK)")
    v = _view(a)
    d = decks.load(a.opp_deck)
    deck = d["main"] + d["side"] if a.pool == "75" else d["main"]
    hands, marg, info, warnings = calc.sample_hands(v, deck, a.n, a.seed)
    res = {"opp_deck": d["name"], "pool": a.pool, **info, "warnings": warnings, "view": V.summary(v), "hands": hands, "marginals": marg}
    lines = [f"Opponent ({d['name']}): hand {info['hand_size']}" + (f", revealed {', '.join(f'{n} x{k}' if k > 1 else n for n, k in info['known'].items())}" if info["known"] else "")
             + f", {info['unknown_pool_size']} unknown cards in hand+library. Uniform over what is consistent with what you have seen; it does not model their play."]
    lines += ["WARNING: " + w for w in warnings]
    lines += [f"Hand {i + 1}: " + ", ".join(h) for i, h in enumerate(hands)]
    top = sorted(marg.items(), key=lambda kv: (-kv[1]["p_in_hand"], kv[0]))[:a.top]
    lines.append("P(card in their hand): " + ", ".join(f"{n} {x['p_in_hand']:.0%}" for n, x in top))
    _out(a, res, "\n".join(lines))


def cmd_notes_read(a):
    t = notes.read(a.matchup)
    if not t:
        _out(a, {"matchup": notes.slug(a.matchup), "text": ""}, f"(no notes yet for {notes.slug(a.matchup)})")
        return
    cut = len(t) > a.max_chars and not a.full
    shown = t[:a.max_chars] if cut else t
    _out(a, {"matchup": notes.slug(a.matchup), "chars": len(t), "truncated": cut, "text": shown},
         (f"[notes are {len(t)} chars; showing the first {a.max_chars}. Use --full, or consolidate with notes_write --mode replace]\n" if cut else "") + shown.rstrip())


def cmd_notes_write(a):
    text = sys.stdin.read() if a.text == ["-"] else " ".join(a.text)
    if not text.strip(): raise SystemExit("empty note")
    n = notes.write(a.matchup, text, a.mode, a.source, a.game)
    _out(a, {"matchup": notes.slug(a.matchup), "bytes": n}, f"saved ({n} bytes) to {notes.slug(a.matchup)}")


def main(argv=None):
    ap = argparse.ArgumentParser(prog="mtgtools")
    sub = ap.add_subparsers(dest="cmd", required=True)

    p = sub.add_parser("view_update", help="fold the prompt you were just shown into your view")
    p.add_argument("--view", default=DEFAULT_VIEW); p.add_argument("--cur"); p.add_argument("--text", help="file with the prompt text, or - for stdin (default)")
    p.add_argument("--seq", type=int, help="the broker's message number, to detect missed prompts"); p.add_argument("--json", action="store_true")
    p.set_defaults(f=cmd_view_update)

    p = sub.add_parser("get_card", help="Oracle text and rulings for cards in the pool")
    p.add_argument("names", nargs="+"); p.add_argument("--json", action="store_true"); p.set_defaults(f=cmd_get_card)

    p = sub.add_parser("library_odds", help="hypergeometric odds over your remaining library")
    _common(p, "my")
    p.add_argument("terms", nargs="*", help="card names, type:Land, type:Creature, mv<=3 ...")
    p.add_argument("--draws", default="1", help="how many cards you will see from the top (draws, or a look-at-top-N effect); comma list allowed")
    p.add_argument("--top", action="append", help="a card YOU know is on top of your library (repeat, in order)")
    p.add_argument("--bottom", action="append", help="a card YOU know is on the bottom (repeat)")
    p.add_argument("--list", action="store_true", help="also print the remaining library composition")
    p.set_defaults(f=cmd_library_odds)

    p = sub.add_parser("sample_opp_hands", help="plausible opponent hands from their decklist minus the cards you have seen")
    _common(p, "opp")
    p.add_argument("--n", type=int, default=5); p.add_argument("--seed", type=int); p.add_argument("--pool", choices=["main", "75"], default="main")
    p.add_argument("--top", type=int, default=12, help="how many cards in the probability summary")
    p.set_defaults(f=cmd_sample_opp_hands)

    p = sub.add_parser("notes_read", help="read the playbook for a matchup")
    p.add_argument("matchup"); p.add_argument("--max-chars", type=int, default=8000); p.add_argument("--full", action="store_true"); p.add_argument("--json", action="store_true")
    p.set_defaults(f=cmd_notes_read)

    p = sub.add_parser("notes_write", help="append to (default) or replace the playbook for a matchup")
    p.add_argument("matchup"); p.add_argument("text", nargs="+", help="the note, or - to read it from stdin")
    p.add_argument("--mode", choices=["append", "replace"], default="append"); p.add_argument("--source", default="in-game"); p.add_argument("--game")
    p.add_argument("--json", action="store_true"); p.set_defaults(f=cmd_notes_write)

    a = ap.parse_args(argv)
    if getattr(a, "n", 1) > 50: raise SystemExit("--n is capped at 50")
    return a.f(a) or 0
