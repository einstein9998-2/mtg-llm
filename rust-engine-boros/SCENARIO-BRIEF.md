# Brief for the Boros Energy scenario writers (2026-10-08)

You write executable spec scenarios (rulings-as-spec) for five cards of Brady's Boros Energy list, for the Rust engine's RFC 0012. You are NOT the implementer. **Do not read anything under `/mnt/project-files/rust-engine/crates/`, any `*.patch`, `boros.cards.ron`, `/home/claude/work/`, or `/home/claude/mtg-llm/`.** Write only from the Oracle text below, the Comprehensive Rules and rulings.

## Read first
- `/mnt/project-files/rust-engine-spec/README.md` and `SCHEMA.md` (read the addenda at the end fully).
- The UR package is the model of how to deliver: `/mnt/project-files/rust-engine-ur/scenarios/*.yaml` (pick a few), `SCENARIO-NOTES-A.md`, `oracle-additions-A.json`, `cr-excerpts-ur-A.md`, and the UR brief `/mnt/project-files/rust-engine-ur/SCENARIO-BRIEF.md`. Copy that style: one file per scenario, `derived_from: oracle` or `cr`, CR quotes `verified: true` only if retrieved verbatim from the rules module.
- Savecraft MCP (load with ToolSearch if deferred): `rules_search` for CR text, `card_search` for card text. Never use recalled Oracle text; a ruling you only remember is `verified: false`.
- Existing pool cards you may use in setups are in `/mnt/project-files/rust-engine-spec/reference/oracle.json` plus the additions files of the Tron, Storm and UR packages (`/mnt/project-files/rust-engine-{tron,storm,ur}/oracle-additions-*.json`). The cards in this brief are NOT in that file: supply a scratch copy of `oracle.json` with their entries and ship the additions as `oracle-additions-<group>.json`. Tokens too.

## Delivery (use your group letter in every name; the shared folder is written by two agents at once)
- Scenarios: `/mnt/project-files/rust-engine-boros/scenarios/boros-<group>-<topic>.yaml`
- Notes for the implementer: `/mnt/project-files/rust-engine-boros/SCENARIO-NOTES-<group>.md` (what each scenario covers, rulings you were unsure about, schema gaps, anything you could not express)
- `oracle-additions-<group>.json`, `cr-excerpts-boros-<group>.md` in `/mnt/project-files/rust-engine-boros/`.
- Validate with `python3 /mnt/project-files/rust-engine-spec/tools/validate.py --oracle <scratch oracle.json> --refs <scratch refs dir> <your scenarios>` (see the UR notes for the exact command). 0 errors required.
- Do not edit `rust-engine-spec/` or `rust-engine/`. Do not call `mcp__hearthbot__*` tools; the thread owner posts for you. Keep scratch files in your own temp directory. About 20 scenarios per group is plenty; cover the rulings that matter, not every phrasing.

## Groups
- **Group A:** Static Prison, Samut, Hazoret's Champion, Sunbaked Canyon.
- **Group B:** Forth Eorlingas!, Mindbreak Trap.

## Oracle text (Scryfall data via Savecraft, 2026-10-08, except Samut)
**Static Prison** {W} Enchantment: When this enchantment enters, exile target nonland permanent an opponent controls until this enchantment leaves the battlefield. You get {E}{E} (two energy counters). / At the beginning of your first main phase, sacrifice this enchantment unless you pay {E}.
**Samut, Hazoret's Champion** {1}{R} (cost from one web source, a second source showed CMC 2 and a blank cost; the engine uses {1}{R}) Legendary Creature — Human Warrior Cleric 2/2: Creatures you control have haste. (Not in the Savecraft database; text taken from two card sites. Mark CR/ruling claims about it `verified: false`.)
**Forth Eorlingas!** {X}{R}{W} Sorcery: Create X 2/2 red Human Knight creature tokens with trample and haste. / Whenever one or more creatures you control deal combat damage to one or more players this turn, you become the monarch.
**Sunbaked Canyon** Land: {T}, Pay 1 life: Add {R} or {W}. / {1}, {T}, Sacrifice this land: Draw a card.
**Mindbreak Trap** {2}{U}{U} Instant — Trap: If an opponent cast three or more spells this turn, you may pay {0} rather than pay this spell's mana cost. / Exile any number of target spells.
(Token) **Human Knight** 2/2 red Human Knight creature token with trample and haste.

## Engine simplifications: do NOT write scenarios for these (documented divergences)
- **Monarch is not modelled** (no state for it). Forth Eorlingas! only creates the tokens. Do not write scenarios about becoming the monarch, drawing at the end step, or losing the monarchy. You may still write scenarios for X = 0, X = 1, X = 3 and the tokens' stats and keywords.
- **Mindbreak Trap targets** are up to four spell targets (each slot optional and distinct); "any number" above four is not modelled. Its free alternative cost has the key `three_spells` (`alt_cost: {three_spells: true}`). Zero targets is legal in the engine's model because "any number" includes zero.
- **Static Prison's** "unless you pay {E}" is an optional payment (a yes/no decision when you have energy; the sacrifice is automatic with none).
- **Samut's** "Creatures you control have haste" is a continuous effect; do not test layer-ordering interactions with other haste sources.

## Syntax reminders
- Energy: set `energy: N` on a player in a setup; the energy payment decision is `{decision: choose_number, purpose: pay_energy}` and is raised only when there is a real choice (see SCHEMA addenda).
- Beginning of a step trigger: your first main phase starts when the step begins; see how existing scenarios in the UR/Tron/Storm packages advance to a step.
- An alternative cost is exclusive (CR 118.9a).
