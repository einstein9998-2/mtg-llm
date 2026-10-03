"""Protocol test: random play through the batched environment, checking the invariants a trainer
relies on. Usage: python3 test_env.py <envserver> <decks dir> [steps]"""
import random
import sys
import time

from mtg_env import MtgEnv, dense

server, decks = sys.argv[1], sys.argv[2]
steps = int(sys.argv[3]) if len(sys.argv) > 3 else 400
env = MtgEnv(server, decks)
meta = env.meta()
assert len(meta["decks"]) == 8 and meta["state_len"] == meta["n_blocks"] * meta["n_defs"] + meta["n_scalars"]
obs, done = env.start(n_games=32, seed=5)
assert len(obs) + len(done) >= 32 - 0
rng = random.Random(1)
finished, decisions = 0, 0
t0 = time.time()
for _ in range(steps):
    waiting = {o["g"] for o in obs}
    assert len(waiting) == len(obs), "one observation per game"
    for o in obs:
        assert o["n"] == len(o["options"]) >= 2
        assert o["seat"] in (0, 1)
        assert all(0 <= i < meta["state_len"] for i, _ in o["state"])
    obs2, done = env.step([(o["g"], rng.randrange(o["n"])) for o in obs])
    decisions += len(obs)
    for d in done:
        assert d["result"] in (-1, 0, 1)
        finished += 1
    obs = obs2
# invalid answers are refused without killing the server
for bad in ([(0, 10**6)], [(10**6, 0)]):
    try:
        env.step(bad)
        raise SystemExit("bad action accepted")
    except RuntimeError:
        pass
assert len(dense(obs[0]["state"], meta["state_len"])) == meta["state_len"]
dt = time.time() - t0
print(f"ok: {decisions} decisions, {finished} games finished, {decisions/dt:.0f} decisions/s through the pipe")
env.close()
