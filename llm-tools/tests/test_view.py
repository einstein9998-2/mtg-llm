import json, os, sys, unittest
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import recorded as R
from mtgtools import view as V

FULL = """T2 your turn, MAIN1 | life you 20 opp 19
YOUR HAND (3): Atraxa, Grand Unifier#4, Island#3, Show and Tell#9
YOUR BF: Dragon's Rage Channeler#7 1/1 (T, sick); lands: Island#1, Island#2 (T)
OPP BF: Germ#40 0/0 (on Batterskull#41, token)
YOUR GY: -
OPP GY: Lightning Bolt#30
OPP HAND 5 (known: Force of Will#31), OPP LIB 50 | YOUR LIB 49
STACK (top first): Show and Tell#9 (you) -> opp; trigger of Dragon's Rage Channeler#7 (you)
D5 Priority: you have priority in MAIN1, stack non-empty
0 Pass
"""
DIFF = """T2 your turn, MAIN1 | life you 20 opp 19
YOUR GY: Brainstorm#12
EXILE: yours Force of Will#2 | opp -
STACK (top first): empty
D6 Priority: you have priority in MAIN1, stack empty
0 Pass
"""

class ViewTests(unittest.TestCase):
    def test_parse_names_with_commas_flags_and_lands(self):
        v = V.fold(V.empty_view(), FULL, key=(0, 5))
        self.assertEqual([c["name"] for c in v["hand"]], ["Atraxa, Grand Unifier", "Island", "Show and Tell"])
        self.assertEqual([(c["name"], c["flags"]) for c in v["ybf"]], [("Dragon's Rage Channeler", "T, sick"), ("Island", ""), ("Island", "T")])
        self.assertEqual(v["obf"][0]["flags"], "on Batterskull#41, token")
        self.assertEqual(v["opp_hand_n"], 5); self.assertEqual([c["name"] for c in v["opp_hand_known"]], ["Force of Will"])
        self.assertEqual((v["opp_lib_n"], v["my_lib_n"]), (50, 49))
        self.assertEqual([(s["name"], s["kind"], s["owner"]) for s in v["stack"]], [("Show and Tell", "spell", "you"), ("Dragon's Rage Channeler", "trigger", "you")])
        self.assertFalse(v["stale"])

    def test_diff_keeps_unchanged_sections_and_replaces_changed_ones(self):
        v = V.fold(V.empty_view(), FULL, key=(0, 5)); V.fold(v, DIFF, key=(0, 6))
        self.assertEqual(len(v["hand"]), 3)                      # not in the diff: kept
        self.assertEqual([c["name"] for c in v["ygy"]], ["Brainstorm"])
        self.assertEqual([c["name"] for c in v["yex"]], ["Force of Will"]); self.assertEqual(v["stack"], [])

    def test_idempotent_per_decision(self):
        v = V.fold(V.empty_view(), FULL, key=(0, 5)); n = v["prompts"]
        V.fold(v, DIFF, key=(0, 5)); self.assertEqual(v["prompts"], n)

    def test_sequence_gap_marks_view_stale_until_next_full_state(self):
        v = V.fold(V.empty_view(), FULL, key=(0, 5), seq=1)
        V.fold(v, DIFF, key=(0, 6), seq=3)
        self.assertTrue(v["stale"]); self.assertIn("missed", v["stale_reason"])
        V.fold(v, FULL, key=(0, 7), seq=4); self.assertFalse(v["stale"])

    def test_mulligan_prompt_sets_hand(self):
        v = V.fold(V.empty_view(), "D4 Mulligan: opening hand (3): Aluren#4, Atraxa, Grand Unifier#1, Ponder#3 | you are on the draw\n0 Keep\n1 Mulligan\n", key=(0, 4))
        self.assertEqual([c["name"] for c in v["hand"]], ["Aluren", "Atraxa, Grand Unifier", "Ponder"])

    def test_message_extra_fields_never_reach_the_view(self):
        msg = {"type": "decision", "game": 0, "id": 5, "prompt": FULL, "seq": 1, "truth": "oppHand=SECRET CARD NAME", "oppHand": ["SECRET CARD NAME"]}
        v = V.fold_message(V.empty_view(), msg)
        self.assertNotIn("SECRET CARD NAME", json.dumps(v))

    @unittest.skipUnless(R.runs(), "recorded runs not available")
    def test_hand_size_matches_state_count_on_every_recorded_prompt(self):
        import re
        bad = n = 0
        for run in R.runs():
            for r, v, _ in R.replay(run):
                m = re.search(r"^YOUR HAND \((\d+)\)", r["prompt"], re.M)
                if m: n += 1; bad += len(v["hand"]) != int(m.group(1))
        self.assertGreater(n, 500); self.assertEqual(bad, 0)

if __name__ == "__main__": unittest.main()
