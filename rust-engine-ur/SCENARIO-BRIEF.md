# Brief for the UR-alternate scenario writers (2026-10-08)

You write executable spec scenarios (rulings-as-spec) for four cards of Brady's alternate UR Cutter deck, for the Rust engine's RFC 0009. You are NOT the implementer. **Do not read anything under `/mnt/project-files/rust-engine/crates/`, any `*.patch`, `ur.cards.ron`, or `/home/claude/work/`.** Write only from the Oracle text below, the Comprehensive Rules and rulings.

## Read first
- `/mnt/project-files/rust-engine-spec/README.md` and `SCHEMA.md` (read the addenda at the end fully).
- The Tron package is the model of how to deliver: `/mnt/project-files/rust-engine-tron/scenarios/*.yaml` (pick a few), `SCENARIO-NOTES-C.md`, `oracle-additions-C.json`, `cr-excerpts-tron-C.md`. Copy that style: one file per scenario, `derived_from: oracle` or `cr`, CR quotes `verified: true` only if retrieved verbatim from the rules module.
- Savecraft MCP (load with ToolSearch if deferred): `rules_search` for CR text, `card_search` for card text. Never use recalled Oracle text; a ruling you only remember is `verified: false`.
- Existing pool cards you may use in setups are in `/mnt/project-files/rust-engine-spec/reference/oracle.json` (the eight decks plus support cards; Storm and Tron cards are in `/mnt/project-files/rust-engine-spec/reference/` additions or the Tron/Storm `oracle-additions-*.json`). Cards in this brief are NOT in that file: supply a scratch copy of `oracle.json` with their entries and ship the additions as `oracle-additions-<group>.json`. Tokens too (the Otter).

## Delivery (use your group letter in every name; the shared folder is written by two agents at once)
- Scenarios: `/mnt/project-files/rust-engine-ur/scenarios/ur-<group>-<topic>.yaml`
- Notes for the implementer: `/mnt/project-files/rust-engine-ur/SCENARIO-NOTES-<group>.md` (what each scenario covers, rulings you were unsure about, schema gaps, anything you could not express)
- `oracle-additions-<group>.json`, `cr-excerpts-ur-<group>.md` in the same folder.
- Validate with `python3 /mnt/project-files/rust-engine-spec/tools/validate.py --oracle <scratch oracle.json> --refs <scratch refs dir> <your scenarios dir>` (see the Tron notes for the exact command). 0 errors required.
- Do not edit `rust-engine-spec/` or `rust-engine/`. Do not call `mcp__hearthbot__*` tools; the thread owner posts for you. Keep scratch files in your own temp directory.

## New scenario syntax: divided damage (Pyrokinesis)
A cast of a spell that divides damage names the division, not an ordered target list:
```yaml
- p0: {t: cast, card: "@pyro", alt_cost: {exile_red_card: "@bolt"}, divide: {"@elf": 3, "@bird": 1}}
```
`divide` maps each chosen target to its share (shares are at least 1 and sum to the total). It replaces `targets`. The adapter turns it into the engine's target choices; you do not write `choose_target` decisions for it. Expectations are written on the outcome (damage, graveyard, life).
The alternative cost key for Pyrokinesis is `exile_red_card` (the red card you exile, by alias).

## Level-up syntax (Class)
`{t: activate, source: "@talent", ability: 0}` is "Level 2" (the first activated ability in Oracle order) and `ability: 1` is "Level 3". The level 2 trigger asks for its target as `{decision: choose_target, slot: 0, answer: "@card"}` when it is put on the stack. The Class level can be set in a setup with `counters: {level: 1}` (level 1 is the minimum for a Class on the battlefield; use `level: 2` or `level: 3` to start higher).

## Oracle text (Scryfall data via Savecraft, 2026-10-07)
**Stormchaser's Talent** {U} Enchantment — Class: (Gain the next level as a sorcery to add its ability.) / When this Class enters, create a 1/1 blue and red Otter creature token with prowess. / {3}{U}: Level 2 / When this Class becomes level 2, return target instant or sorcery card from your graveyard to your hand. / {5}{U}: Level 3 / Whenever you cast an instant or sorcery spell, create a 1/1 blue and red Otter creature token with prowess.
**Pyrokinesis** {4}{R}{R} Instant: You may exile a red card from your hand rather than pay this spell's mana cost. / Pyrokinesis deals 4 damage divided as you choose among any number of target creatures.
**Price of Progress** {1}{R} Instant: Price of Progress deals damage to each player equal to twice the number of nonbasic lands that player controls.
**Boomerang Basics** {U} Sorcery — Lesson: Return target nonland permanent to its owner's hand. If you controlled that permanent, draw a card.
(Token) **Otter** 1/1 blue and red Otter creature token, prowess. (Prowess: whenever you cast a noncreature spell, this creature gets +1/+1 until end of turn.)

## Engine simplifications: do NOT write scenarios for these (documented divergences)
- Pyrokinesis targets: the engine requires at least one target creature to cast it ("any number" including zero is not modelled), and the spell always divides all 4 points.
- Class levels are represented by a counter named `level`; do not test counter-removal or proliferate effects on a Class.
- Do not test "becomes the target" triggers on a creature targeted by Pyrokinesis (targets are counted once per point internally).

## Groups
- **A: Stormchaser's Talent and the Otter token.** Class rules (CR 716): enters at level 1; the enters trigger makes an Otter with prowess; level-up is a sorcery-speed activated ability (main phase, empty stack, your turn), one level at a time, never skippable, cannot be done twice for the same level; the level 2 trigger returns an instant or sorcery card (target chosen when the trigger goes on the stack; no target if the graveyard has none; the target can be removed in response); level 3 makes an Otter whenever you cast an instant or sorcery (not a creature or artifact spell; it triggers on cast only, not for copies); levels survive until the Class leaves, and a Class that is bounced (Boomerang Basics on your own Talent: you draw a card) comes back at level 1 with a new Otter; Force of Will / Daze against casting the Talent; Wasteland and Stifle do not affect it; prowess on the Otter from noncreature spells (the Talent is an enchantment spell, so an Otter already out gets +1/+1 when you cast it); an Otter token made in response; Level 3 with a spell cast in response to the level-up. Cover the Alurentell interactions that decide games: Aluren/Omniscience free casts of instants and sorceries still trigger level 3; Veil of Summer irrelevant. About 20 to 25 scenarios.
- **B: Pyrokinesis, Price of Progress, Boomerang Basics.** Pyrokinesis: hard cast for six, alternative cost by exiling a red card (only a red card qualifies; the card exiled is exiled as a cost, and with Force of Will in mind it is not "free" for Daze), one target taking all 4, 2+2, 3+1, 1+1+1+1, 1+1+2, a target that dies or becomes illegal in response (its share is lost, the others still get theirs), all targets illegal (spell fizzles), protection from red, damage on creatures with lethal and non-lethal toughness, indestructible, Aluren-board cases (Cavern Harpy, Wirewood Savage, Dryad Arbor, Imperial Recruiter, Raven Familiar, Auriok Salvagers and similar creatures from the Alurentell list in `/mnt/project-files/decks/alurentell.txt`), can't be cast with no creature on the battlefield. Price of Progress: counts only nonbasic lands each player controls (fetchlands before and after cracking, Wasteland, Ancient Tomb, City of Traitors, Volcanic Island, basic Island and Mountain, Hedge Maze and similar dual/surveil lands, Tron lands are nonbasic), both players are dealt damage at once, 0 nonbasics deals 0, lethal for one or both (both at 0 life is a draw, CR 104.4a), in response to a land being played or sacrificed, and cast with Force of Will / Daze in response. Boomerang Basics: a nonland permanent you control (you draw), one an opponent controls (no draw), cannot target a land, bounces a token (token ceases to exist; you still draw if you controlled it), bounces a Class (level resets), bounces Aluren / Omniscience, target becomes illegal. About 25 to 30 scenarios.
