"""Run play.py (the broker) and fold the decision text it prints into the player's view. Reads only play.py's stdout, which is exactly what the
player is shown; it never opens the engine's logs. A new game (decision id restarting, or 'start') resets the view."""
import os, re, subprocess, sys

from . import view as V

PLAY = os.environ.get("PLAY_PY", "/mnt/project-files/llm-player/play.py")
HEAD = re.compile(r"^\[decision seq=(\d+) id=(\d+) kind=(\w+)")


def main(argv):
    path = os.environ.get("MTG_VIEW", "view.json")
    r = subprocess.run([sys.executable, PLAY, *argv], capture_output=True, text=True)
    out = r.stdout
    sys.stdout.write(out); sys.stderr.write(r.stderr)
    if argv and argv[0] == "start" and os.path.exists(path): os.remove(path)
    m = HEAD.match(out.split("\n", 1)[0])
    if m:
        v = V.load(path)
        seq, did = int(m.group(1)), int(m.group(2))
        if v["last_key"] is not None and did < v["last_key"][1]: v = V.empty_view()      # a new game began: ids restart
        V.fold(v, out.split("\n", 1)[1], key=(0, did), seq=seq)
        V.save(path, v)
        print(f"[view: {V.summary(v)}]")
    return r.returncode


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
