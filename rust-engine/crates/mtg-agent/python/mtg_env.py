"""Client for the batched MTG environment server (`envserver`). Standard library only.

    env = MtgEnv("target/release/envserver", "decks")
    meta = env.meta()                       # vocabulary: defs, n_defs, state_len, scalar names
    obs, done = env.start(n_games=64, seed=1)
    while training:
        actions = [(o["g"], choose(o)) for o in obs]      # one option index per waiting game
        obs, done = env.step(actions)

Each observation `o` is one decision of one game for the seat that must act:
    o["g"]        game slot
    o["seat"]     0 or 1 (both seats are driven by the client; this is self-play)
    o["n"]        number of options
    o["state"]    sparse state encoding, [[index, value], ...], length meta["state_len"] when dense
                  (n_blocks blocks of n_defs card counts, then n_scalars scalars)
    o["options"]  one row per option: [kind, decision_kind, subject_def_plus_1, subject_zone, value]
Each `done` entry is a finished game: {"g", "result" (+1 seat 0 won, -1 seat 1 won, 0 draw),
"decks", "first", "decisions", "truncated"}. With autoreset (the default) the slot immediately
holds a new game; credit a finished episode to its seats by keeping a per-(game, seat) trajectory.
Observations contain only what the acting seat may see; there is no way to read the other seat.
"""
import json
import subprocess


class MtgEnv:
    def __init__(self, server_path, decks_dir):
        self.p = subprocess.Popen([server_path, decks_dir], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)

    def _call(self, msg):
        self.p.stdin.write(json.dumps(msg) + "\n")
        self.p.stdin.flush()
        line = self.p.stdout.readline()
        if not line:
            raise RuntimeError("envserver exited")
        r = json.loads(line)
        if not r.get("ok"):
            raise RuntimeError(r.get("error", "server error"))
        return r

    def meta(self):
        return self._call({"cmd": "meta"})

    def start(self, n_games=16, seed=1, pairs=None, autoreset=True, max_decisions=4000):
        msg = {"cmd": "start", "n_games": n_games, "seed": seed, "autoreset": autoreset, "max_decisions": max_decisions}
        if pairs:
            msg["pairs"] = pairs
        r = self._call(msg)
        return r["obs"], r["done"]

    def step(self, actions):
        r = self._call({"cmd": "step", "actions": [[g, a] for g, a in actions]})
        return r["obs"], r["done"]

    def close(self):
        try:
            self.p.stdin.write('{"cmd":"quit"}\n')
            self.p.stdin.flush()
        except Exception:
            pass
        self.p.wait(timeout=5)


def dense(state, length):
    """Sparse [[i, v], ...] to a dense list (use numpy.zeros in a real trainer)."""
    x = [0.0] * length
    for i, v in state:
        x[i] = v
    return x
