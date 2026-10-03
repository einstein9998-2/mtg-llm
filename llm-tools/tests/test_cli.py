import json, os, subprocess, sys, tempfile, shutil, unittest
HERE = os.path.dirname(os.path.abspath(__file__)); ROOT = os.path.dirname(HERE)
sys.path.insert(0, ROOT)
from mtgtools import cards, notes

def tool(*args, env=None, stdin=None):
    return subprocess.run([sys.executable, "-m", "mtgtools", *args], capture_output=True, text=True, cwd=ROOT, env=dict(os.environ, **(env or {})), input=stdin)

class CardTests(unittest.TestCase):
    def test_every_deck_card_resolves_and_has_oracle_text(self):
        names = set()
        for f in os.listdir("/mnt/project-files/decks"):
            if f.endswith(".txt"):
                for l in open("/mnt/project-files/decks/" + f):
                    if l[:1].isdigit(): names.add(l.split(" ", 1)[1].strip())
        self.assertEqual(len(names), 149)
        for n in names:
            c = cards.card(n); self.assertNotIn("error", c, n); self.assertTrue(all(f["oracle"] for f in c["faces"]), n)

    def test_names_are_forgiving_and_ids_are_stripped(self):
        for q in ("force of will", "Force of Will#12", "FORCE OF WILL", "ajani, nacatl avenger", "Daze"):
            self.assertNotIn("error", cards.card(q), q)
        self.assertIn("Force of Will", cards.card("force of wil")["suggestions"] if "error" in cards.card("force of wil") else ["Force of Will"])
        self.assertIn("error", cards.card("Black Lotus"))

    def test_oracle_matches_the_engine_script(self):
        self.assertIn("without paying their mana costs", cards.card("Aluren")["faces"][0]["oracle"])
        self.assertEqual(cards.card("Murktide Regent")["faces"][0]["mana_value"], 7)

class NotesTests(unittest.TestCase):
    def setUp(self): self.d = tempfile.mkdtemp(); self.env = {"MTG_NOTES_DIR": self.d}
    def tearDown(self): shutil.rmtree(self.d)
    def test_append_replace_read_and_limits(self):
        self.assertIn("no notes yet", tool("notes_read", "A vs B", env=self.env).stdout)
        tool("notes_write", "A vs B", "first", "note", env=self.env); tool("notes_write", "A vs B", "-", "--game", "g2", env=self.env, stdin="second\nline")
        t = tool("notes_read", "A vs B", env=self.env).stdout
        self.assertTrue(t.index("first note") < t.index("second")); self.assertIn("game g2", t)
        tool("notes_write", "A vs B", "--mode", "replace", "-", env=self.env, stdin="clean playbook"); self.assertEqual(tool("notes_read", "A vs B", env=self.env).stdout.strip(), "clean playbook")
        r = tool("notes_write", "A vs B", "x" * 40000, env=self.env); self.assertNotEqual(r.returncode, 0); self.assertIn("consolidate", r.stderr)
        self.assertEqual(notes.slug("A vs B"), "a-vs-b")

class ToolIntegration(unittest.TestCase):
    def test_view_update_then_odds_and_sampling_end_to_end(self):
        d = tempfile.mkdtemp()
        try:
            view = os.path.join(d, "v.json"); env = {"MTG_NOTES_DIR": d}
            prompt = open(os.path.join(HERE, "test_isolation.py")).read().split('PROMPT = """')[1].split('"""')[0]
            r = tool("view_update", "--view", view, "--text", "-", env=env, stdin=prompt); self.assertIn("view complete", r.stdout)
            r = tool("library_odds", "--view", view, "--my-deck", "alurentell", "--draws", "2", "Atraxa, Grand Unifier", "--json", env=env)
            j = json.loads(r.stdout); self.assertEqual(j["library_size"], 53); self.assertEqual(j["warnings"], [])
            self.assertEqual(j["queries"]["2"]["Atraxa, Grand Unifier"]["in_library"], 3)
            r = tool("sample_opp_hands", "--view", view, "--opp-deck", "ur-cutter", "--n", "3", "--seed", "1", "--json", env=env)
            j = json.loads(r.stdout); self.assertEqual(len(j["hands"]), 3); self.assertTrue(all(len(h) == 5 for h in j["hands"])); self.assertEqual(j["warnings"], [])
            self.assertNotEqual(tool("library_odds", "--view", view, "--my-deck", "alurentell", "Not A Card", env=env).returncode, 0)
        finally: shutil.rmtree(d)


class ImporterFixture(unittest.TestCase):
    def test_import_on_a_tiny_fixture(self):
        d = tempfile.mkdtemp()
        try:
            json.dump([{"oracle_id": "o1", "name": "Daze"}, {"oracle_id": "o2", "name": "Fire // Ice", "card_faces": [{"name": "Fire"}, {"name": "Ice"}]}], open(d + "/o.json", "w"))
            json.dump([{"oracle_id": "o1", "source": "wotc", "published_at": "2020-01-01", "comment": "official daze ruling"},
                       {"oracle_id": "o1", "source": "scryfall", "published_at": "2020-01-02", "comment": "not wotc"}], open(d + "/r.json", "w"))
            r = subprocess.run([sys.executable, os.path.join(ROOT, "data", "import_scryfall_rulings.py"), d + "/o.json", d + "/r.json", "--out", d + "/out.json"], capture_output=True, text=True)
            self.assertEqual(r.returncode, 0, r.stderr); out = json.load(open(d + "/out.json"))["rulings"]
            self.assertEqual(out["Daze"], ["2020-01-01: official daze ruling"]); self.assertIn("Force of Will", out)   # curated kept where no official rulings
        finally: shutil.rmtree(d)



class TplayShim(unittest.TestCase):
    def test_folds_prompts_printed_by_a_stub_broker_and_flags_gaps(self):
        d = tempfile.mkdtemp()
        try:
            prompt = open(os.path.join(HERE, "test_isolation.py")).read().split('PROMPT = """')[1].split('"""')[0]
            stub = os.path.join(d, "play.py")
            open(stub, "w").write("import sys\nseq=int(sys.argv[2])\nprint(f'[decision seq={seq} id={seq+17} kind=Priority pick 1..1 of 1]')\nprint(%r if seq == 1 else 'T5 your turn, MAIN1 | life you 18 opp 17\\nD40 Priority: x\\n0 Pass\\n')\n" % prompt)
            env = dict(os.environ, PLAY_PY=stub, MTG_VIEW=os.path.join(d, "v.json"), PYTHONPATH=ROOT)
            run = lambda seq: subprocess.run([sys.executable, "-m", "mtgtools.tplay", "pick", str(seq)], capture_output=True, text=True, env=env, cwd=d)
            r = run(1); self.assertIn("[view:", r.stdout); self.assertIn("view complete", r.stdout)
            r = run(3); self.assertIn("STALE", r.stdout)
        finally: shutil.rmtree(d)


if __name__ == "__main__": unittest.main()
