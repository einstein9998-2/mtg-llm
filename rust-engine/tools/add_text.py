#!/usr/bin/env python3
"""One-off helper: adds `text: "..."` (the Oracle sentence) to Triggered/Activated abilities of
cards in a RON card file, from reference/oracle.json plus the explicit overrides below.
Usage: add_text.py cards.ron oracle.json"""
import json, re, sys

KEYWORD_ONLY = re.compile(r'^(Flash|Flying|Lifelink|Vigilance|Deathtouch|Trample|Haste|First strike|Menace|Reach|Defender|Delve.*|Equip \{.*\}|Escape.*|Evoke.*|Cycling.*|Ward.*|Swampwalk.*|Impending.*|Mobilize.*|Warp.*|Ascend.*|Ninjutsu.*|Miracle.*|Replicate.*|Storm.*|Escalate.*|Flashback.*|Channel.*)\s*$')

OVERRIDES = {
    # Cards whose Oracle line holds several abilities (or a delayed trigger sentence).
    "Mishra's Bauble": ["{T}, Sacrifice this artifact: Look at the top card of target player's library.", "Draw a card at the beginning of the next turn's upkeep."],
    "Phelia, Exuberant Shepherd": ["Whenever Phelia attacks, exile up to one other target nonland permanent.", "At the beginning of the next end step, return that card to the battlefield under its owner's control. If it entered under your control, put a +1/+1 counter on Phelia."],
    "Flickerwisp": ["When this creature enters, exile another target permanent.", "Return that card to the battlefield under its owner's control at the beginning of the next end step."],
    "Solitude": ["When this creature enters, exile up to one other target creature. That creature's controller gains life equal to its power.", "When this creature enters, if it was evoked, sacrifice it."],
    "Cori-Steel Cutter": ["Flurry — Whenever you cast your second spell each turn, create a 1/1 white Monk creature token with prowess. You may attach this Equipment to it.", "Equip {1}{R}"],
    "City of Traitors": ["When you play another land, sacrifice this land.", "{T}: Add {C}{C}."],
    "Karakas": ["{T}: Add {W}.", "{T}: Return target legendary creature to its owner's hand."],
    "Lavinia, Azorius Renegade": ["Whenever an opponent casts a spell, if no mana was spent to cast it, counter that spell."],
    "Treasure Token": ["{T}, Sacrifice this artifact: Add one mana of any color."],
    "Wasteland": ["{T}: Add {C}.", "{T}, Sacrifice this land: Destroy target nonbasic land."],
}

def sentences(oracle):
    out = []
    for line in oracle.split("\n"):
        line = line.strip()
        if not line or line.startswith("(") or KEYWORD_ONLY.match(line):
            continue
        out.append(re.sub(r"\s*\([^)]*\)\s*$", "", line))
    return out

def main():
    ron, orj = sys.argv[1], sys.argv[2]
    oracle = json.load(open(orj))["cards"]
    src = open(ron).read()
    parts = re.split(r'(\n  \(name: )', src)
    out = [parts[0]]
    for i in range(1, len(parts), 2):
        sep, body = parts[i], parts[i + 1]
        name = re.match(r'"([^"]*)"', body).group(1)
        n_abil = len(re.findall(r'\n    (?:Triggered|Activated)\(', body))
        if n_abil:
            if name in OVERRIDES:
                texts = OVERRIDES[name]
            else:
                texts = sentences(oracle.get(name, {}).get("oracle_text", ""))
                texts = [t for t in texts if not re.match(r'^(This land enters tapped|Equipped|Each opponent can|Delirium|Nonbasic)', t)]
            if len(texts) < n_abil:
                print(f"!! {name}: {n_abil} abilities but {len(texts)} texts: {texts}", file=sys.stderr)
                texts = (texts + [""] * n_abil)[:n_abil]
            it = iter(texts)
            def repl(m):
                t = next(it)
                if 'text:' in body[m.end():m.end()+10]:
                    return m.group(0)
                return f'{m.group(0)}text: "{t}", ' if t else m.group(0)
            body = re.sub(r'\n    (?:Triggered|Activated)\(', lambda m: m.group(0) , body)  # no-op, keeps structure
            body = re.sub(r'(\n    (?:Triggered|Activated)\()', repl, body)
        out.append(sep + body)
    open(ron, "w").write("".join(out))

main()
