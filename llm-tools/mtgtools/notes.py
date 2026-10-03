"""Per-matchup playbook notes: plain markdown files, one per matchup, written only by the player (or its post-mortem step)."""
import fcntl, os, re, tempfile, time

NOTES_DIR = os.environ.get("MTG_NOTES_DIR", os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "notes"))
MAX_BYTES = 32768


def slug(s):
    s = re.sub(r"[^a-z0-9]+", "-", s.lower()).strip("-")
    if not s: raise ValueError("empty matchup name")
    return s


def path(matchup): return os.path.join(NOTES_DIR, slug(matchup) + ".md")


def read(matchup):
    try: return open(path(matchup)).read()
    except FileNotFoundError: return ""


def write(matchup, text, mode="append", source="in-game", game=None):
    """append: add a dated entry. replace: the new text becomes the whole playbook (use it to consolidate). Atomic, with a file lock."""
    if mode not in ("append", "replace"): raise ValueError("mode must be append or replace")
    os.makedirs(NOTES_DIR, exist_ok=True)
    p = path(matchup)
    with open(p + ".lock", "w") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        old = read(matchup)
        if mode == "replace": new = text.rstrip() + "\n"
        else:
            hdr = "## " + time.strftime("%Y-%m-%d %H:%MZ", time.gmtime()) + f" [{source}]" + (f" game {game}" if game else "")
            new = (old.rstrip() + "\n\n" if old.strip() else "") + hdr + "\n" + text.rstrip() + "\n"
        if len(new.encode()) > MAX_BYTES:
            raise ValueError(f"notes would be {len(new.encode())} bytes (limit {MAX_BYTES}): consolidate with --mode replace")
        fd, tmp = tempfile.mkstemp(dir=NOTES_DIR, prefix=".note-")
        with os.fdopen(fd, "w") as f: f.write(new)
        os.replace(tmp, p)
    return len(new)
