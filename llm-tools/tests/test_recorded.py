"""Replay the 14+ recorded Alurentell vs UR Cutter games through the tools (prompt text only) and grade them against the engine's ground truth."""
import os, sys, unittest
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import recorded as R
from mtgtools import calc, decks

@unittest.skipUnless(R.runs(), "recorded runs not available")
class RecordedGames(unittest.TestCase):
    def test_own_library_accounting_matches_the_state_count(self):
        mine = decks.load("alurentell")["main"]; n = bad = 0; bad_kinds = set()
        for run in R.runs():
            for r, v, _ in R.replay(run):
                if v["stale"] or v["my_lib_n"] is None: continue
                n += 1
                if sum(calc.my_library(v, mine)[0].values()) != v["my_lib_n"]: bad += 1; bad_kinds.add(r["kind"])
        self.assertGreater(n, 1500)
        # the only misses are mid-cost prompts: the Force of Will being cast is in neither hand nor stack yet (1 card)
        self.assertLessEqual(bad / n, 0.005, f"{bad}/{n} {bad_kinds}"); self.assertTrue(bad_kinds <= {"ChooseCards"})

    def test_real_opponent_hand_is_always_inside_the_sampler_support(self):
        opp = decks.load("ur-cutter")["main"]; n = 0
        for run in R.runs():
            for r, v, truth in R.replay(run):
                if v["stale"] or v["opp_lib_n"] is None: continue
                pool, known, w = calc.opp_pool(v, opp); n += 1
                self.assertEqual(sum(truth.values()), v["opp_hand_n"], (run, r["d"]))
                self.assertTrue(all(pool.get(c, 0) + known.get(c, 0) >= k for c, k in truth.items()), (run, r["d"], dict(truth)))
                self.assertFalse([x for x in w if "wrong deck" in x], w)
        self.assertGreater(n, 1500)

    def test_no_warnings_on_ordinary_priority_prompts(self):
        mine = decks.load("alurentell")["main"]; opp = decks.load("ur-cutter")["main"]; n = 0
        for run in R.runs():
            for r, v, _ in R.replay(run):
                if r["kind"] != "Priority" or v["stale"] or v["my_lib_n"] is None: continue
                n += 1; self.assertEqual(calc.my_library(v, mine)[1], [], (run, r["d"])); self.assertEqual(calc.opp_pool(v, opp)[2], [], (run, r["d"]))
        self.assertGreater(n, 800)

if __name__ == "__main__": unittest.main()
