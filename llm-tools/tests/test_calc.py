import itertools, os, random, sys, unittest
from collections import Counter
from math import comb
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from mtgtools import calc, view as V


def brute(pop, groups, draws):
    """exact enumeration of all draw sets; returns (P(all groups hit), P(any hit))"""
    cards = [c for c, k in pop.items() for _ in range(k)]
    tot = hit_all = hit_any = 0
    for combo in itertools.combinations(range(len(cards)), draws):
        got = {cards[i] for i in combo}
        tot += 1
        hit_all += all(got & g for g in groups); hit_any += any(got & g for g in groups)
    return hit_all / tot, hit_any / tot


class OddsTests(unittest.TestCase):
    def test_known_value(self):
        pop = Counter({"A": 4, "x": 56})
        self.assertAlmostEqual(calc.odds(pop, {"A": {"A"}}, 7)["A"]["p_at_least_one"], 1 - comb(56, 7) / comb(60, 7), places=12)

    def test_matches_brute_force_including_overlapping_groups(self):
        pop = Counter({"A": 2, "B": 1, "C": 3, "D": 4})
        for draws in (1, 2, 3, 5):
            for groups in ([{"A"}, {"B"}], [{"A", "C"}, {"C", "D"}], [{"A"}, {"B"}, {"D"}]):
                r = calc.odds(pop, {f"g{i}": g for i, g in enumerate(groups)}, draws)
                ba, by = brute(pop, groups, draws)
                self.assertAlmostEqual(r["__all__"], ba, places=12); self.assertAlmostEqual(r["__any__"], by, places=12)

    def test_known_top_cards_are_drawn_first(self):
        pop = Counter({"A": 1, "x": 9}); top = ["A"]            # A is known on top of the rest of the library pool
        rem = Counter({"A": 2, "x": 9})
        r = calc.odds(rem, {"A": {"A"}}, 1, top=["A"]); self.assertEqual(r["A"]["p_at_least_one"], 1.0)
        r2 = calc.odds(rem, {"A": {"A"}}, 3, top=["x", "x"]); self.assertAlmostEqual(r2["A"]["p_at_least_one"], 2 / 9)   # one random draw from {A,x...}
        self.assertEqual(calc.odds(rem, {"A": {"A"}}, 2, bottom=["A", "A"])["A"]["p_at_least_one"], 0.0)

    def test_exactly(self):
        pop = Counter({"A": 3, "x": 7})
        self.assertAlmostEqual(sum(calc.p_exactly(pop, {"A"}, k, 4) for k in range(4)), 1.0)

    def test_monte_carlo_agrees(self):
        pop = Counter({"A": 4, "B": 2, "x": 54}); bag = list(pop.elements()); rng = random.Random(1); n = 40000; h = 0
        for _ in range(n):
            s = set(rng.sample(bag, 9)); h += "A" in s and "B" in s
        self.assertAlmostEqual(calc.odds(pop, {"a": {"A"}, "b": {"B"}}, 9)["__all__"], h / n, delta=0.01)


def view_with(**kw):
    v = V.empty_view(); v.update(stale=False, stale_reason="", opp_hand_n=3, opp_lib_n=None); v.update(kw); return v

def cards(*names): return [{"name": n, "id": i, "flags": ""} for i, n in enumerate(names)]


class AccountingTests(unittest.TestCase):
    deck = Counter({"A": 4, "B": 4, "X": 2})

    def test_library_is_deck_minus_visible_own_cards_and_ignores_tokens(self):
        v = view_with(hand=cards("A", "B"), ybf=cards("A") + [{"name": "Goblin", "id": 99, "flags": "token"}], ygy=cards("X"), my_lib_n=5,
                      stack=[{"name": "B", "id": 1, "owner": "you", "kind": "spell"}, {"name": "A", "id": 2, "owner": "opp", "kind": "spell"}])
        rem, w = calc.my_library(v, self.deck)
        self.assertEqual(dict(rem), {"A": 2, "B": 2, "X": 1}); self.assertEqual(w, [])

    def test_warns_on_mismatch_and_on_stale_view(self):
        rem, w = calc.my_library(view_with(hand=cards("A"), my_lib_n=5, stale=True, stale_reason="gap"), self.deck)
        self.assertEqual(len(w), 2)

    def test_opp_sampling_respects_pool_known_cards_and_size(self):
        opp = Counter({"F": 4, "L": 20, "M": 8})
        v = view_with(obf=cards("L", "L"), ogy=cards("F"), opp_hand_n=7, opp_hand_known=cards("M"), opp_lib_n=22)
        hands, marg, info, w = calc.sample_hands(v, opp, 300, seed=3)
        self.assertEqual(w, []); self.assertEqual(info["unknown_pool_size"], 28)       # 32 cards - 2 on bf - 1 in gy - 1 known in hand
        for h in hands:
            self.assertEqual(len(h), 7); self.assertIn("M", h)
            c = Counter(h); self.assertLessEqual(c["F"], 3); self.assertLessEqual(c["L"], 18); self.assertLessEqual(c["M"], 7)
        self.assertEqual(marg["M"]["p_in_hand"], 1.0)

    def test_sampling_is_seeded_and_marginals_match_empirical(self):
        opp = Counter({"F": 4, "L": 20, "M": 8}); v = view_with(opp_hand_n=7, opp_lib_n=25)
        a = calc.sample_hands(v, opp, 5, seed=11)[0]; self.assertEqual(a, calc.sample_hands(v, opp, 5, seed=11)[0])
        hands, marg, _, _ = calc.sample_hands(v, opp, 20000, seed=2)
        emp = sum("F" in h for h in hands) / len(hands)
        self.assertAlmostEqual(emp, marg["F"]["p_in_hand"], delta=0.012)
        self.assertAlmostEqual(sum(h.count("F") for h in hands) / len(hands), marg["F"]["expected_in_hand"], delta=0.02)

if __name__ == "__main__": unittest.main()
