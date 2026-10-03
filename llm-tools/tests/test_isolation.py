"""The tools see only the player's own view. Evidence: (1) every file a tool opens is audited and must be code, static data, the named decks, the view
file or the notes dir; nothing from the engine's logs, the serializer, the run folders or the decoy 'hidden world' planted next to the view; no network,
no subprocesses. (2) the same audit flags a deliberately leaky variant, so the check can fail. (3) outputs do not change when the hidden world changes."""
import json, os, shutil, subprocess, sys, tempfile, unittest
HERE = os.path.dirname(os.path.abspath(__file__)); ROOT = os.path.dirname(HERE)
sys.path.insert(0, HERE)
from mtgtools import view as V
PROJECT = os.path.realpath("/mnt/project-files")
DECK_FILES = {os.path.realpath(f"{PROJECT}/decks/{n}.txt") for n in ("alurentell", "ur-cutter")}
STDLIB = tuple(os.path.realpath(p) for p in {sys.prefix, sys.base_prefix, sys.exec_prefix, "/usr/lib", "/usr/local/lib"})

PROMPT = """T5 your turn, MAIN1 | life you 18 opp 17
YOUR HAND (4): Atraxa, Grand Unifier#5, Misty Rainforest#7, Omniscience#11, Show and Tell#15
YOUR BF: lands: Ancient Tomb#20, Island#21
OPP BF: Volcanic Island#22, Wasteland#23
YOUR GY: Brainstorm#4
OPP GY: Lightning Bolt#29
OPP HAND 5, OPP LIB 52 | YOUR LIB 53
STACK (top first): empty
D18 Priority: you have priority in MAIN1, stack empty
0 Pass
"""

def violations(events, tmp, view_file, notes_dir):
    bad = []
    for e in events:
        if e[0] != "open":
            bad.append(e); continue
        p = e[1]
        if p.startswith(STDLIB) or p.startswith("/proc") or p.startswith("/dev") or p.startswith("/etc/localtime") or p.startswith("/usr"): continue
        if p.startswith(os.path.join(ROOT, "mtgtools")) or p.startswith(os.path.join(ROOT, "data")): continue
        if p in DECK_FILES or p == os.path.realpath(view_file) or p.startswith(os.path.realpath(notes_dir)): continue
        if p.startswith(os.path.realpath(tmp)) and os.path.dirname(p) == os.path.realpath(tmp) and os.path.basename(p).startswith(".view-"): continue   # atomic view write
        bad.append(e)
    return bad


class Isolation(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp(); self.notes = os.path.join(self.tmp, "notes"); os.makedirs(self.notes)
        self.view = os.path.join(self.tmp, "view.json")
        v = V.fold(V.empty_view(), PROMPT, key=(0, 18)); V.save(self.view, v)
        self.decoy = os.path.join(self.tmp, "decisions.jsonl"); self.write_world("oppHand=Force of Will,Daze,Murktide Regent")
        self.env = dict(os.environ, MTG_NOTES_DIR=self.notes, PYTHONDONTWRITEBYTECODE="1")
    def tearDown(self): shutil.rmtree(self.tmp)
    def write_world(self, truth): open(self.decoy, "w").write(json.dumps({"truth": truth}) + "\n"); open(os.path.join(self.tmp, "engine.out"), "w").write(truth)

    def run_tool(self, args, leaky=False):
        log = os.path.join(self.tmp, "audit.json")
        cmd = [sys.executable, os.path.join(HERE, "audit_run.py"), log] + (["--leaky", self.decoy] if leaky else []) + ["--"] + args
        r = subprocess.run(cmd, capture_output=True, text=True, cwd=self.tmp, env=self.env)
        self.assertEqual(r.returncode, 0, r.stderr)
        return r.stdout, json.load(open(log))

    def commands(self):
        v = ["--view", self.view]
        return [["get_card", "Force of Will", "Atraxa, Grand Unifier"],
                ["library_odds", *v, "--my-deck", "alurentell", "--draws", "3,5", "--list", "Atraxa, Grand Unifier", "type:Land"],
                ["sample_opp_hands", *v, "--opp-deck", "ur-cutter", "--n", "4", "--seed", "1"],
                ["notes_write", "alurentell-vs-ur-cutter", "hold Veil for Stock Up"],
                ["notes_read", "alurentell-vs-ur-cutter"]]

    def test_tools_open_only_view_decks_static_data_and_notes(self):
        for c in self.commands():
            out, events = self.run_tool(c)
            self.assertEqual(violations(events, self.tmp, self.view, self.notes), [], c)
            self.assertTrue(out.strip(), c)

    def test_audit_flags_a_tool_that_peeks_at_the_engine_files(self):
        _, events = self.run_tool(["get_card", "Daze"], leaky=True)
        bad = violations(events, self.tmp, self.view, self.notes)
        self.assertTrue(any(e[0] == "open" and e[1].endswith("decisions.jsonl") for e in bad), "negative control did not trigger")

    def test_output_is_independent_of_the_hidden_world(self):
        a = [self.run_tool(c)[0] for c in self.commands()[:3]]
        self.write_world("oppHand=Lightning Bolt,Lightning Bolt,Lightning Bolt,Brainstorm,Brainstorm")    # a different real opponent hand
        b = [self.run_tool(c)[0] for c in self.commands()[:3]]
        self.assertEqual(a, b)

    def test_tool_source_never_names_engine_logs(self):
        for d in ("mtgtools",):
            for f in os.listdir(os.path.join(ROOT, d)):
                if f.endswith(".py"):
                    src = open(os.path.join(ROOT, d, f)).read().replace("/mnt/project-files/llm-player/play.py", "")   # tplay.py runs the broker, whose stdout is the player's own prompt
                    for bad in ("decisions.jsonl", "engine.out", "llm-player", "serializer/", "forge-", "truth", "oppHand", "myLibTop"):
                        self.assertNotIn(bad, src, f"{f} mentions {bad}")

if __name__ == "__main__": unittest.main()
