"""Deck files: `N Card Name` lines, main first, a blank line, then the sideboard (the format in /mnt/project-files/decks/*.txt)."""
import os, re
from collections import Counter

DECK_DIR = os.environ.get("MTG_DECKS", "/mnt/project-files/decks")


def resolve(name_or_path):
    """A path, or a deck name such as 'alurentell' / 'ur-cutter' looked up in DECK_DIR."""
    if os.path.isfile(name_or_path): return name_or_path
    p = os.path.join(DECK_DIR, name_or_path if name_or_path.endswith(".txt") else name_or_path + ".txt")
    if os.path.isfile(p): return p
    raise FileNotFoundError(f"deck not found: {name_or_path} (looked in {DECK_DIR})")


def load(name_or_path):
    """-> {"name": stem, "main": Counter, "side": Counter}"""
    path = resolve(name_or_path)
    main, side, in_side = Counter(), Counter(), False
    for line in open(path):
        line = line.strip()
        if line.startswith("#"): continue
        if not line:
            in_side = in_side or bool(main)
            continue
        m = re.match(r"^(\d+)\s+(.+)$", line)
        if m: (side if in_side else main)[m.group(2)] += int(m.group(1))
    return {"name": os.path.splitext(os.path.basename(path))[0], "main": main, "side": side}
