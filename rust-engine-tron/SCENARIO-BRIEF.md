# Brief for the Colorless Tron scenario writers (2026-10-06)

You write executable spec scenarios (rulings-as-spec) for Colorless Tron cards, for the Rust engine's RFC 0007. You are NOT the implementer. **Do not read anything under `/mnt/project-files/rust-engine/crates/`, any `*.patch`, `tron.cards.ron`, or `tron-engine-gap.md`.** Write only from the Oracle text below and the Comprehensive Rules (effective 2025-11-14). Expectations never come from the engine or from Forge.

## Read first
- `/mnt/project-files/rust-engine-spec/README.md` and `SCHEMA.md` (read the addenda at the end fully).
- The Orim's Chant package is the model of how to deliver: `/mnt/project-files/rust-engine-orims-chant/scenarios/*.yaml`, its `SCENARIO-NOTES.md` and `reference/cr-excerpts-orims-chant.md`. Copy that style: one file per scenario, `derived_from: oracle`, CR quotes `verified: true` only if retrieved verbatim (Savecraft `rules_search`) and placed in your cr-excerpts file; Oracle quotes only from the text in this brief.
- Savecraft MCP (load with ToolSearch if deferred): `rules_search` for CR text, `card_search` for any card text you need that is not in the pool or in this brief. Never use recalled Oracle text; a ruling you only remember is `verified: false`.
- Existing pool cards you may use in setups are in `/mnt/project-files/rust-engine-spec/reference/oracle.json` (the eight decks plus support cards). Cards in this brief are NOT in that file: supply a scratch copy of `oracle.json` with their entries (the Chant package did the same) and ship the additions as `oracle-additions-<group>.json`.

## Delivery (use your group letter in every name; the shared folder is written by three agents at once)
- Scenarios: `/mnt/project-files/rust-engine-tron/scenarios/tron-<group>-<topic>.yaml`
- Notes for the implementer: `/mnt/project-files/rust-engine-tron/SCENARIO-NOTES-<group>.md` (what each scenario covers, rulings you were unsure about, schema gaps, anything you could not express)
- `oracle-additions-<group>.json`, `cr-excerpts-tron-<group>.md` in the same folder.
- Validate with `python3 /mnt/project-files/rust-engine-spec/tools/validate.py --oracle <scratch oracle.json> --refs <scratch refs dir> <your scenarios dir>` (see the Chant notes for the exact command). 0 errors required.
- Do not edit `rust-engine-spec/` or `rust-engine/`. Do not call `mcp__hearthbot__*` tools; the thread owner posts for you. Keep scratch files in your own temp directory and delete what you create in the shared folder only if it is a throwaway.

## Cover, for each card
Aim for the rules that decide real games (about 5 to 10 scenarios per card, fewer for plain cards): the card's normal effect, its edge cases and negative cases (what it must NOT do), interaction with the cards in the Tron and Alurentell decks (Alurentell: Aluren, Omniscience, Force of Will, Show and Tell, Lotus Petal, Acererak, Atraxa, Brainstorm, Ponder, Stock Up, Carpet of Flowers, Prismatic Ending, Veil of Summer, Orim's Chant), and cost, timing, stack and priority details. Each scenario must assert something an implementation could get wrong.

## Engine simplifications: do NOT write scenarios for these (they are documented divergences)
- Urza's Workshop: the engine models its two mana abilities as one ability; test the outcome (mana produced) only.
- Kozilek's Command: the engine only lets you target yourself with its two "target player" modes. Do not target the opponent with those modes.
- Mishra's Research Desk: no scenario for "until the end of your next turn" expiry, and none for unearth.
- Karn's -2, Ugin's -11, Urza's Saga, Summon: Bahamut, Argentum Masticore, Mycosynth Lattice, Eldrazi Confluence, Extinguisher Battleship's station: not in scope.
- Loyalty: Tezzeret starts at 4, Ugin at 7 (confirmed). Karn's starting loyalty (5) is unconfirmed: avoid scenarios whose result depends on it, or mark them `verified: false` and say so in the notes.

## Oracle text (Scryfall data via Savecraft, 2026-10-06)
**Trinisphere** {3} Artifact: As long as this artifact is untapped, each spell that would cost less than three mana to cast costs three mana to cast. (Additional mana in the cost may be paid with any color of mana or colorless mana. For example, a spell that would cost {1}{B} to cast costs {2}{B} to cast instead.)
**Grim Monolith** {2} Artifact: This artifact doesn't untap during your untap step. / {T}: Add {C}{C}{C}. / {4}: Untap this artifact.
**The One Ring** {4} Legendary Artifact: Indestructible / When The One Ring enters, if you cast it, you gain protection from everything until your next turn. / At the beginning of your upkeep, you lose 1 life for each burden counter on The One Ring. / {T}: Put a burden counter on The One Ring, then draw a card for each burden counter on The One Ring.
**Voltaic Key** {1} Artifact: {1}, {T}: Untap target artifact.
**Manifold Key** {1} Artifact: {1}, {T}: Untap another target artifact. / {3}, {T}: Target creature can't be blocked this turn.
**Karn, the Great Creator** {4} Legendary Planeswalker — Karn: Activated abilities of artifacts your opponents control can't be activated. / +1: Until your next turn, up to one target noncreature artifact becomes an artifact creature with power and toughness each equal to its mana value. / −2: (out of scope)
**Portable Hole** {W} Artifact: When this artifact enters, exile target nonland permanent an opponent controls with mana value 2 or less until this artifact leaves the battlefield.
**Mishra's Research Desk** {1} Artifact: {1}, {T}, Sacrifice this artifact: Exile the top two cards of your library. Choose one of them. Until the end of your next turn, you may play that card. / Unearth {1}{R} (out of scope)
**Torpor Orb** {2} Artifact: Creatures entering don't cause abilities to trigger.
**Ensnaring Bridge** {3} Artifact: Creatures with power greater than the number of cards in your hand can't attack.
**Pithing Needle** {1} Artifact: As this artifact enters, choose a card name. / Activated abilities of sources with the chosen name can't be activated unless they're mana abilities.
**Tormod's Crypt** {0} Artifact: {T}, Sacrifice this artifact: Exile target player's graveyard.
**Liquimetal Coating** {2} Artifact: {T}: Target permanent becomes an artifact in addition to its other types until end of turn.
**Expedition Map** {1} Artifact: {2}, {T}, Sacrifice this artifact: Search your library for a land card, reveal it, put it into your hand, then shuffle.
**Extinguisher Battleship** {8} Artifact — Spacecraft (10/10 when it is a creature; station is out of scope): When this Spacecraft enters, destroy target noncreature permanent. Then this Spacecraft deals 4 damage to each creature.
**Tezzeret, Cruel Captain** {3} Legendary Planeswalker — Tezzeret, loyalty 4: Whenever an artifact you control enters, put a loyalty counter on Tezzeret. / 0: Untap target artifact or creature. If it's an artifact creature, put a +1/+1 counter on it. / −3: Search your library for an artifact card with mana value 1 or less, reveal it, put it into your hand, then shuffle. / −7: You get an emblem with "At the beginning of combat on your turn, put three +1/+1 counters on target artifact you control. If it's not a creature, it becomes a 0/0 Robot artifact creature."
**Ugin, Eye of the Storms** {7} Legendary Planeswalker — Ugin, loyalty 7: When you cast this spell, exile up to one target permanent that's one or more colors. / Whenever you cast a colorless spell, exile up to one target permanent that's one or more colors. / +2: You gain 3 life and draw a card. / 0: Add {C}{C}{C}. / −11: (out of scope)
**Kozilek's Command** {X}{C}{C} Kindred Instant — Eldrazi: Choose two — • Target player creates X 0/1 colorless Eldrazi Spawn creature tokens with "Sacrifice this token: Add {C}." • Target player scries X, then draws a card. • Exile target creature with mana value X or less. • Exile up to X target cards from graveyards.
**Warping Wail** {1}{C} Instant: ({C} represents colorless mana.) Choose one — • Exile target creature with power or toughness 1 or less. • Counter target sorcery spell. • Create a 1/1 colorless Eldrazi Scion creature token. It has "Sacrifice this token: Add {C}."
**Urza's Tower** Land — Urza's Tower: {T}: Add {C}. If you control an Urza's Mine and an Urza's Power-Plant, add {C}{C}{C} instead.
**Urza's Workshop** Land — Urza's: {T}: Add {C}. / Metalcraft — {T}: Add {C} for each Urza's land you control. Activate only if you control three or more artifacts.
**Planar Nexus** Land: This land is every nonbasic land type. (Nonbasic land types include Cave, Desert, Gate, Lair, Locus, Mine, Power-Plant, Sphere, Tower, and Urza's.) / {T}: Add {C}. / {1}, {T}: Add one mana of any color.
**Ancient Tomb** Land: {T}: Add {C}{C}. This land deals 2 damage to you.
(Tokens) Eldrazi Spawn: 0/1 colorless Eldrazi Spawn creature, "Sacrifice this token: Add {C}." Eldrazi Scion: 1/1 colorless Eldrazi Scion creature, same ability.

## Groups
- **A**: Trinisphere, Grim Monolith, The One Ring, Voltaic Key, Manifold Key, Karn (static and +1). Trinisphere is the card that matters most against Alurentell: cover it against Force of Will (pitch and hard cast), Daze-style alternative costs, Aluren and Omniscience free casts, zero-cost and X spells, and Show and Tell (a put onto the battlefield is not a cast).
- **B**: Portable Hole, Mishra's Research Desk, Torpor Orb, Ensnaring Bridge, Pithing Needle, Tormod's Crypt, Liquimetal Coating, Expedition Map, Extinguisher Battleship (enters trigger only). Torpor Orb against Acererak, Atraxa and Cloak and Dagger; Bridge against Atraxa and hand-size changes mid-combat.
- **C**: Urza's Tower, Urza's Workshop, Planar Nexus (types, Tron mana, Workshop with and without three artifacts, the `{1},{T}` any-color ability including paying a colored spell with it), Kozilek's Command (each mode, X = 0, X > 0, two modes together), Warping Wail, Eldrazi tokens, Tezzeret (all four abilities, the loyalty trigger), Ugin (cast trigger, colorless-spell trigger, +2, 0, loyalty-0 or colored-target edge cases).
