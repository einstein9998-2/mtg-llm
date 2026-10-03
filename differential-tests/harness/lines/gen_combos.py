#!/usr/bin/env python3
"""Writes lines/combos.scn: hand-built starting states for the multi-step lines (Aluren chains, Show and Tell, Doomsday, Reanimator, counter wars).
Deterministic (fixed seed). States use the interaction-test scenario syntax; there is no script, the policy plays them."""
import random
R = random.Random(20261001)
free = ["Orcish Bowmasters", "Phelia, Exuberant Shepherd", "Recruiter of the Guard", "Acererak the Archlich", "Dragon's Rage Channeler", "Tamiyo, Inquisitive Student",
        "Flickerwisp", "Brazen Borrower", "Lavinia, Azorius Renegade", "Witch Enchanter", "Boggart Trawler", "Barrowgoyf", "Nethergoyf", "Bilbo, Thief in the Night",
        "Gaddock Teeg", "Containment Priest", "Amped Raptor", "Guide of Souls", "Ocelot Pride", "Voice of Victory"]
filler = ["Island", "Brainstorm", "Ponder", "Lotus Petal", "Swamp", "Force of Will", "Thoughtseize", "Dark Ritual", "Daze", "Atraxa, Grand Unifier", "Murktide Regent", "Lightning Bolt",
          "Swords to Plowshares", "Stifle", "Preordain", "Tundra"]
big = ["Atraxa, Grand Unifier", "Griselbrand", "Archon of Cruelty", "Omniscience", "Koma, World-Eater", "Solitude", "Quantum Riddler", "Murktide Regent"]
out = []
def scn(name, title, human, ai, active="human"):
    s = [f"=== {name}", f"title: {title}", "state:"]
    for k, v in human.items(): s.append(f"  human{k}={';'.join(v) if isinstance(v, list) else v}")
    for k, v in ai.items(): s.append(f"  ai{k}={';'.join(v) if isinstance(v, list) else v}")
    s.append(f"  activeplayer={active}")
    out.append("\n".join(s) + "\n")
def lib(n=10, extra=()):
    return list(extra) + [R.choice(filler) for _ in range(n - len(extra))]
def pick(pool, n): return R.sample(pool, n)

# Aluren chains: free creatures (mana value 3 or less) cast at any time by either player
for i in range(26):
    lands = R.choice([["Island", "Island"], ["Tropical Island", "Island", "Forest"], ["Ancient Tomb", "Island"], ["Island"], []])
    h = pick(free, R.choice([2, 3, 4])) + R.sample(["Brainstorm", "Ponder", "Lotus Petal", "Show and Tell", "Stock Up", "Force of Will"], R.choice([0, 1, 2]))
    ai_h = pick(free, R.choice([0, 1, 2])) + R.sample(["Force of Will", "Daze", "Swords to Plowshares"], R.choice([0, 1]))
    scn(f"aluren-chain-{i:02d}", "Aluren on the battlefield; creatures of mana value 3 or less are free for both players",
        {"hand": h, "battlefield": ["Aluren"] + lands, "library": lib(10), "graveyard": pick(filler, R.choice([0, 1, 2]))},
        {"hand": ai_h, "battlefield": R.choice([["Island", "Island"], ["Swamp"], []]), "library": lib(10)})
# Acererak with Aluren: the loop that completes Tomb of Annihilation
for i in range(8):
    scn(f"aluren-acererak-{i:02d}", "Aluren with Acererak the Archlich in hand: each free cast ventures; the dungeon completes",
        {"hand": ["Acererak the Archlich"] * R.choice([1, 2]) + R.sample(["Brainstorm", "Ponder", "Phelia, Exuberant Shepherd", "Orcish Bowmasters"], R.choice([0, 1, 2])),
         "battlefield": ["Aluren", "Island"] + R.choice([[], ["Swamp"], ["Lotus Petal"]]), "library": lib(10)},
        {"hand": R.sample(["Force of Will", "Daze", "Brainstorm"], R.choice([0, 1])), "battlefield": ["Island"], "library": lib(10)})
# Aluren cast from hand (it costs {2}{G})
for i in range(6):
    scn(f"aluren-cast-{i:02d}", "Aluren in hand with enough mana: cast it, then chain creatures",
        {"hand": ["Aluren"] + pick(free, 3), "battlefield": ["Forest", "Island", "Island", "Lotus Petal"], "library": lib(10)},
        {"hand": R.sample(["Force of Will", "Daze", "Swords to Plowshares", "Stifle"], R.choice([0, 1, 2])), "battlefield": ["Island", "Island"], "library": lib(10)})
# Aluren with the second seat as the active player (Forge's known seat-order bug applies, see kd-0002)
for i in range(4):
    scn(f"aluren-seat2-{i:02d}", "Second seat is the active player with Aluren on the battlefield",
        {"hand": R.sample(free, 2), "battlefield": ["Island"], "library": lib(10)},
        {"hand": pick(free, 3), "battlefield": ["Aluren", "Island"], "library": lib(10)}, active="ai")
# Show and Tell
for i in range(18):
    h = ["Show and Tell"] + pick(big + free[:4], R.choice([1, 2, 3])) + R.sample(["Force of Will", "Brainstorm", "Lotus Petal"], R.choice([0, 1]))
    ai_h = pick(big + free, R.choice([0, 1, 2])) + R.sample(["Force of Will", "Daze", "Swords to Plowshares", "Thoughtseize"], R.choice([0, 1]))
    scn(f"show-and-tell-{i:02d}", "Show and Tell with a big permanent in hand: the opponent puts one in as well",
        {"hand": h, "battlefield": R.choice([["Island", "Island", "Island"], ["Ancient Tomb", "Island"], ["City of Traitors", "Island", "Lotus Petal"], ["Island", "Island", "Island", "Island"]]), "library": lib(10), "graveyard": pick(filler, R.choice([0, 2]))},
        {"hand": ai_h, "battlefield": R.choice([["Island", "Island"], ["Plains", "Plains"], []]), "library": lib(10)})
# Doomsday
for i in range(16):
    h = ["Doomsday"] + R.sample(["Dark Ritual", "Lotus Petal", "Brainstorm", "Ponder", "Thassa's Oracle", "Force of Will", "Lion's Eye Diamond", "Personal Tutor", "Street Wraith", "Daze"], R.choice([2, 3, 4]))
    scn(f"doomsday-{i:02d}", "Doomsday with a few cheap spells; the pile is chosen by the opponent engine's AI-less seat (the stock AI) and then played",
        {"hand": h, "battlefield": R.choice([["Underground Sea", "Underground Sea", "Underground Sea"], ["Underground Sea", "Swamp", "Swamp", "Island"], ["Ancient Tomb", "Swamp", "Swamp"]]),
         "library": ["Thassa's Oracle", "Brainstorm", "Ponder", "Dark Ritual", "Lotus Petal", "Force of Will", "Island", "Underground Sea", "Daze", "Thoughtseize"][:R.choice([8, 10])] if i % 2 == 0 else lib(10, ["Thassa's Oracle"]),
         "graveyard": pick(filler, R.choice([0, 1, 3]))},
        {"hand": R.sample(["Force of Will", "Daze", "Thoughtseize", "Swords to Plowshares"], R.choice([0, 1, 2])), "battlefield": ["Island", "Island"], "library": lib(10)})
# Reanimator
for i in range(18):
    h = pick(["Reanimate", "Animate Dead", "Shallow Grave", "Faithless Looting", "Dark Ritual", "Thoughtseize", "Unmask", "Cabal Therapy", "Collective Brutality", "Lotus Petal", "Stronghold Gambit"], R.choice([3, 4])) + R.sample(big, R.choice([0, 1]))
    scn(f"reanimator-{i:02d}", "Reanimation lines with creatures in the graveyard",
        {"hand": h, "battlefield": R.choice([["Swamp", "Swamp", "Swamp"], ["Underground Sea", "Badlands", "Swamp"], ["Swamp", "Swamp", "Lotus Petal", "Marsh Flats"]]),
         "graveyard": pick(["Griselbrand", "Archon of Cruelty", "Atraxa, Grand Unifier", "Koma, World-Eater", "Raph & Mikey, Troublemakers"], R.choice([1, 2])) + pick(filler, R.choice([0, 1])), "library": lib(10)},
        {"hand": R.sample(["Force of Will", "Swords to Plowshares", "Surgical Extraction", "Thoughtseize", "Daze"], R.choice([0, 1, 2])),
         "battlefield": R.choice([["Island", "Island"], ["Plains", "Plains", "Plains"], ["Swamp"]]), "graveyard": pick(["Murktide Regent", "Griselbrand", "Phelia, Exuberant Shepherd"], R.choice([0, 1])), "library": lib(10)})
# counter wars: pitch and free spells both ways
for i in range(14):
    pool = ["Force of Will", "Daze", "Brainstorm", "Ponder", "Lightning Bolt", "Swords to Plowshares", "Stifle", "Preordain", "Thoughtseize", "Force of Negation", "Murktide Regent", "Dragon's Rage Channeler", "Orcish Bowmasters"]
    scn(f"counter-war-{i:02d}", "Cheap spells, pitch counters and Daze on both sides",
        {"hand": pick(pool, R.choice([3, 4, 5])), "battlefield": R.choice([["Island", "Island", "Volcanic Island"], ["Island", "Island", "Island"], ["Volcanic Island", "Island"], ["Underground Sea", "Island", "Island"]]), "library": lib(10), "graveyard": pick(filler, R.choice([0, 2, 4]))},
        {"hand": pick(pool, R.choice([2, 3, 4])), "battlefield": R.choice([["Island", "Island"], ["Tundra", "Island", "Island"], ["Island"]]), "library": lib(10)})
open("/home/claude/difftest-m3g/lines/combos.scn", "w").write("# Generated by gen_combos.py (fixed seed). Hand-built starting states for lockstep lines.\n\n" + "\n".join(out))
print(len(out), "states")
