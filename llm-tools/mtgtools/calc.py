"""Zone accounting from the player's view, hypergeometric odds, and opponent hand sampling. Pure functions of (view, decklists, seed)."""
import random
from collections import Counter
from math import comb

from . import view as V


def _nontoken(cards): return [c["name"] for c in cards if "token" not in c["flags"].split(", ")]


def my_seen(v):
    """Own cards known to be outside the library: hand, battlefield, graveyard, own exile, own spells on the stack."""
    seen = Counter(_nontoken(v["hand"])) + Counter(_nontoken(v["ybf"])) + Counter(_nontoken(v["ygy"])) + Counter(_nontoken(v["yex"]))
    seen += Counter(s["name"] for s in v["stack"] if s["kind"] == "spell" and s["owner"] == "you")
    return seen


def opp_seen(v):
    """Opponent cards known to be outside their library, including revealed cards still in their hand."""
    seen = Counter(_nontoken(v["obf"])) + Counter(_nontoken(v["ogy"])) + Counter(_nontoken(v["oex"])) + Counter(V.names(v["opp_hand_known"]))
    seen += Counter(s["name"] for s in v["stack"] if s["kind"] == "spell" and s["owner"] == "opp")
    return seen


def remaining(deck, seen):
    """deck - seen, never below zero. Also returns the names seen more often than the deck has them (wrong deck, a Wish, a stolen card)."""
    rem, extra = Counter(), Counter()
    for n in sorted(set(deck) | set(seen)):
        d = deck.get(n, 0) - seen.get(n, 0)
        if d >= 0: rem[n] = d
        else: extra[n] = -d
    return +rem, extra


def my_library(v, deck, top=(), bottom=()):
    """Own library composition from the view. top/bottom: cards the player itself knows sit on top (in order) or at the bottom (from Brainstorm
    put-backs, Scry, mulligan bottoms), taken from the player's own memory, not from the engine."""
    rem, extra = remaining(deck, my_seen(v))
    warnings = []
    if v["stale"]: warnings.append("view is stale (" + v["stale_reason"] + "): zone counts may be wrong")
    if extra: warnings.append("cards outside the deck list (or seen more often than listed): " + ", ".join(f"{n} x{k}" for n, k in sorted(extra.items())))
    total = sum(rem.values())
    if v["my_lib_n"] is not None and total != v["my_lib_n"]:
        warnings.append(f"deck minus seen cards = {total} but the state says YOUR LIB {v['my_lib_n']}: counts are approximate")
    t, b = Counter(top), Counter(bottom)
    for n, k in (t + b).items():
        if rem.get(n, 0) < k: warnings.append(f"you listed {n} as known top/bottom x{k} but only {rem.get(n, 0)} can be in the library")
    return rem, warnings


# ---------------------------------------------------------------- odds

def p_none(pop, group_total, draws):
    """P(no card of a group of group_total in `draws` cards from pop)."""
    if draws > pop: draws = pop
    return comb(pop - group_total, draws) / comb(pop, draws) if pop - group_total >= draws else 0.0


def p_hit_all(pop, groups, draws):
    """P(at least one card from every group). groups: list of sets of card names; counts: pop is a Counter. Inclusion-exclusion over groups."""
    n = len(groups); total = sum(pop.values()); res = 0.0
    for mask in range(1 << n):
        union = set()
        for i in range(n):
            if mask >> i & 1: union |= groups[i]
        miss = sum(pop[c] for c in union)
        res += (-1) ** bin(mask).count("1") * p_none(total, miss, draws)
    return res


def p_any(pop, groups, draws):
    union = set().union(*groups) if groups else set()
    return 1 - p_none(sum(pop.values()), sum(pop[c] for c in union), draws)


def p_exactly(pop, group, k, draws):
    N = sum(pop.values()); K = sum(pop[c] for c in group); draws = min(draws, N)
    return comb(K, k) * comb(N - K, draws - k) / comb(N, draws) if 0 <= k <= K and 0 <= draws - k <= N - K else 0.0


def split_known(rem, top, bottom, draws):
    """Known top cards are drawn first; known bottom cards are out of reach. Returns (pool, random_draws, forced) where forced are the
    known top cards among the first `draws`."""
    top = list(top)
    pool = rem - Counter(top) - Counter(bottom)
    forced = top[:draws]
    return +pool, max(0, draws - len(top)), forced


def odds(rem, groups, draws, mode="each", top=(), bottom=()):
    """groups: {label: set(names)}. Returns {label: {...}}, plus 'any'/'all' when asked. P(at least one) and expected copies in the first `draws` cards."""
    pool, rdraws, forced = split_known(rem, top, bottom, draws)
    out = {}
    fset = Counter(forced)
    for label, g in groups.items():
        in_forced = sum(fset[c] for c in g)
        kk = sum(pool[c] for c in g); N = sum(pool.values())
        p = 1.0 if in_forced else 1 - p_none(N, kk, rdraws)
        out[label] = {"in_library": sum(rem[c] for c in g), "p_at_least_one": p, "expected_copies": in_forced + (rdraws * kk / N if N else 0)}
    labels = list(groups)
    gs = [groups[l] for l in labels]
    if len(gs) > 1:
        hit_forced = [any(fset[c] for c in g) for g in gs]
        rest = [g for g, h in zip(gs, hit_forced) if not h]
        out["__all__"] = p_hit_all(pool, rest, rdraws) if rest else 1.0
        out["__any__"] = 1.0 if any(hit_forced) else p_any(pool, gs, rdraws)
    return out


# ---------------------------------------------------------------- opponent hands

def opp_pool(v, opp_deck):
    """Unknown opponent cards (their hand's hidden part plus their library) = deck - everything seen. Returns (pool, known_hand, warnings)."""
    seen = opp_seen(v)
    pool, extra = remaining(opp_deck, seen)
    warnings = []
    if v["stale"]: warnings.append("view is stale (" + v["stale_reason"] + "): the pool may be wrong")
    if extra: warnings.append("opponent cards seen outside the given decklist (wrong deck?): " + ", ".join(f"{n} x{k}" for n, k in sorted(extra.items())))
    known = Counter(V.names(v["opp_hand_known"]))
    pool = pool - Counter()  # copy
    expect = (v["opp_hand_n"] - sum(known.values())) + (v["opp_lib_n"] or 0)
    if v["opp_lib_n"] is not None and sum(pool.values()) != expect:
        warnings.append(f"opponent deck minus seen cards = {sum(pool.values())}, expected {expect} unknown cards (hand {v['opp_hand_n']} + library {v['opp_lib_n']} - known): counts are approximate")
    return pool, known, warnings


def sample_hands(v, opp_deck, n, seed=None):
    """n opponent hands of the real size, drawn uniformly from the unknown pool, plus any revealed cards. Never looks at anything but the view and the decklist."""
    pool, known, warnings = opp_pool(v, opp_deck)
    h = v["opp_hand_n"] - sum(known.values())
    bag = [c for c, k in sorted(pool.items()) for _ in range(k)]
    rng = random.Random(seed)
    hands = []
    for _ in range(n):
        hand = list(known.elements()) + (rng.sample(bag, min(h, len(bag))) if h > 0 else [])
        hands.append(sorted(hand))
    N = len(bag)
    marg = {}
    for c, k in pool.items():
        kn = known.get(c, 0)
        marg[c] = {"copies_unknown": k, "p_in_hand": 1.0 if kn else (1 - p_none(N, k, h) if h > 0 else 0.0),
                   "expected_in_hand": kn + (h * k / N if N else 0)}
    for c, kn in known.items():
        marg.setdefault(c, {"copies_unknown": 0, "p_in_hand": 1.0, "expected_in_hand": float(kn)})
    return hands, marg, {"hand_size": v["opp_hand_n"], "known": dict(known), "unknown_pool_size": N, "unknown_slots": max(h, 0)}, warnings
