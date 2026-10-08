# Boros group B scenarios (Forth Eorlingas!, Mindbreak Trap, Human Knight token): notes for the implementer

24 scenarios, one file each, `scenarios/boros-B-<topic>.yaml` (8 Forth Eorlingas!, 16 Mindbreak Trap). Written from the Oracle text in SCENARIO-BRIEF.md (re-checked with Savecraft `card_search`, exact names, 2026-10-08; the Human Knight token was looked up the same way: `Token Creature - Human Knight`, 2/2, red, "Trample, haste", set TLTC) and the Comprehensive Rules (2025-11-14, Savecraft `rules_search`: 107.3, 115.4/115.5, 118.9, 404.1, 406.1, 601.2, 608.2/608.2h, 701.6, 701.13, 702.40 retrieved this session; the other cited rules are copied from the existing `cr-excerpts*.md` files, which came from the same module). No engine source, patch, `boros.cards.ron` or `/home/claude/work/` was read; no `mcp__hearthbot__*` tool was called.

## Delivery and validation

- `oracle-additions-B.json`: `cards` "Forth Eorlingas!" and "Mindbreak Trap", `tokens` "Human Knight Token" (name "Human Knight", 2/2, color R, text "Trample, haste"). Scenarios write the token as `{token: Human Knight Token}`.
- `cr-excerpts-boros-B.md`: every CR rule a `verified: true` source cites (33 rules).
- Command (scratch `oracle.json` = the pool plus all packages' additions plus mine; refs dir holding that `oracle.json` and `cr-excerpts-boros-B.md`):
  `python3 /mnt/project-files/rust-engine-spec/tools/validate.py --oracle <scratch>/refs/oracle.json --refs <scratch>/refs /mnt/project-files/rust-engine-boros/scenarios/boros-B-*.yaml` -> `24 scenario(s), 0 error(s), 0 warning(s)`.
- The Monarch clause is not modelled and never asserted; no scenario involves becoming the monarch, the end-step draw or losing the monarchy.
- Exactly one `verified: false` entry: the recalled storm ruling in `boros-B-trap-in-response-to-storm-trigger-exiles-original-copies-still-come` (see Rulings). Every other source is Oracle text in `oracle-additions-B.json` / `reference/oracle.json` or CR text in `cr-excerpts-boros-B.md`.

## Scenario list

| file (boros-B-...) | asserts |
|---|---|
| forth-x0-creates-no-tokens | X = 0 is legal, costs only {R}{W} (third land stays untapped), resolves, no token |
| forth-x1-creates-one-knight-token-with-stats | X = 1 costs three mana; one token: Creature, Human Knight, red, 2/2, trample, haste, untapped, controlled by caster |
| forth-x3-creates-three-knight-tokens-for-five-mana | X = 3 costs five (Mountain, Plains and three of the four Islands tapped); three separate tokens with the stats |
| forth-x-is-paid-with-mana-three-needs-five | four lands: X = 3 is illegal (state untouched), X = 2 legal and taps all four, two tokens |
| forth-x1-knight-token-attacks-the-turn-it-is-created | haste: the token created this turn is offered as an attacker, deals 2 (20 to 18) |
| forth-x1-knight-token-tramples-over-a-chump-blocker | trample: blocked by a 1/1, 1 to the blocker and 1 to the player; 0 to the blocker and 2 to the player is illegal; Knight survives with 1 damage |
| forth-force-of-will-counters-it-no-tokens | countered with X = 2: no tokens, mana not refunded, card in graveyard |
| forth-exiled-by-mindbreak-trap-no-tokens | Trap hard-cast on it: no tokens, card in p0's exile, not in the graveyard |
| trap-free-three-spells-exiles-all-three | `symmetric`; p1 casts three instants (all on the stack), p0 pays {0} (Islands untapped) and exiles all three; nothing in graveyards, no draw, no mana, no damage |
| trap-hard-cast-exiles-one-spell | one opposing spell: `alt_cost: {three_spells: true}` illegal, normal cost {2}{U}{U} taps four Islands, one target exiled |
| trap-two-spells-not-enough-for-free-hard-cast-exiles-two | boundary: exactly two opposing spells is not free; hard cast exiles both |
| trap-caster-casting-three-spells-does-not-make-it-free | only the caster cast three (the opponent none): not free; the Trap exiles the caster's own Brainstorm |
| trap-free-exiles-two-of-three-third-spell-resolves | free Trap with two of three targets; the third (Bolt) resolves normally |
| trap-exiled-spells-never-reach-the-graveyard-surgical-extraction | after the Trap, p1's graveyard is empty; Surgical Extraction cannot target any of the three exiled cards (expect_illegal) |
| trap-exiled-sorcery-cannot-be-flashed-back | Faithless Looting exiled from the stack cannot be cast with flashback although the flashback cost is payable |
| trap-exiles-spells-that-cannot-be-countered-veil-of-summer | p1 resolved Veil of Summer (counts as a cast spell); the Trap exiles p1's now-uncounterable Brainstorm and Bolt |
| trap-exiles-uncounterable-bolt-where-force-of-will-fails | Force of Will cannot counter the Veil-protected Bolt (Bolt stays on the stack); the Trap then exiles it |
| trap-hard-cast-exiles-own-spell-and-opponents-spell | one Trap targets the caster's own Bolt and the opponent's Brainstorm |
| trap-cannot-target-itself-but-can-target-another-trap | targeting itself (alone or with the Bolt) is illegal; the opponent's Trap legally exiles the first Trap, so the Bolt resolves |
| trap-exiles-storm-original-and-both-copies | storm 2: the Trap exiles Empty the Warrens and both copies (copies cease to exist), no Goblins; the copy count stays at three casts |
| trap-in-response-to-storm-trigger-exiles-original-copies-still-come | Trap in response to the storm trigger (free, third cast): original exiled, the trigger still makes two copies, four Goblins |
| trap-storm-copies-do-not-count-toward-three-spells | Petal + Warrens + a storm copy is two casts, not three: not free; hard cast exiles original and copy |
| trap-two-traps-earlier-exile-leaves-one-target-illegal | Trap B exiles the Bolt first; Trap A resolves with one illegal target and still exiles Brainstorm |
| trap-countered-by-force-of-will-exiles-nothing | the Trap is countered; the targeted Bolt resolves (p0 20 to 17) |

## Spellings I used (the adapter owns exact shapes)

- Free cost: `alt_cost: {three_spells: true}` (the brief's key). Probes that it is NOT offered are `expect_illegal` casts carrying that `alt_cost`.
- Multi-target Trap: `targets: ["@a", "@b", "@c"]` in slot order; the targets are distinct spells.
- Targeting a storm copy: `{copy_of: "@etw", nth: n}` (the form used once in storm-A; the n-th copy of that spell counted from the top, 0-based; the original card alias names the original spell). This is the only place the copies need a name.
- X spells: `x: N` on the cast; the stack entry carries `x: N`. `expect_illegal` with a too-large X for "not enough mana".
- Tokens made during play are bound with `bind: {as: knight, find: {name: "Human Knight Token", controller: p0}}` (works only for X = 1).
- Combat: `advance: {to: begin_combat, of: p0}`, `p0 pass`, `p1 pass`, `declare_attackers` decision (the pattern of the Cori-Steel Cutter scenario). With no possible blocker the declare_blockers decision is not raised, but the step still gives priority (two more pass pairs); with a blocker: `declare_blockers`, then `p0 pass`, `p1 pass`, then `assign_combat_damage` (raised because the attacker has trample and a blocker).
- Surgical Extraction probes use `phyrexian: [life]`.
- `symmetric: true` on the main free-Trap scenario only.

## Rulings I was unsure about

1. **Storm copies after the original leaves the stack** (`trap-in-response-to-storm-trigger-...`). The storm trigger is its own object (702.40a) and resolves independently (113.7a, 608.2h last-known information), so exiling the original in response still lets the two copies be created. This matches the well-known official storm ruling, which I could not retrieve (`kind: ruling`, `verified: false`, date "unknown"). The expectation (four Goblins) depends on it. If the implementer's team decides the ruling is wrong, only that file changes.
2. **Force of Will targeting an uncounterable spell** (`trap-exiles-uncounterable-bolt-where-force-of-will-fails`). I assert the cast is legal and the Bolt stays on the stack (101.2: "can't" beats "can"; it is a legal target, it simply is not countered). I am confident of the rule but an engine may refuse the target; the note says that would also be a deviation.
3. **Surgical Extraction and flashback probes** assert illegality of targeting/casting an exiled card. The Trap resolves, p1 (active) gets priority first and passes, then p0 holds priority on an empty stack; if the engine hands priority differently the pass count before the probes would change (not the rules content).
4. **Murktide Regent delve check not scripted**: it is a sorcery-speed creature spell, so it cannot be cast on the opponent's turn, which is when the Trap's condition is met. The graveyard-zone argument is covered by Surgical Extraction and flashback.
5. **"Exile any number of target spells" with zero targets** is legal by 601.2c ("any number" includes zero) per the brief's engine note; I wrote no scenario for it.
6. **A resolved spell still counts for "cast three or more"** (Veil, Petal, Bauble scenarios): the count is of spells cast this turn, whatever happened to them afterwards. This follows from the Oracle wording ("cast"), not from a CR quote.

## Not covered (by the brief or by design)

- Monarch, "any number" above four targets, Samut/Static Prison/Sunbaked Canyon (group A).
- Becoming-the-target triggers, Cavern of Souls "can't be countered" (Veil of Summer is used for uncounterable spells), Daze on Forth Eorlingas! (mana payment of X is covered by the four-land scenario and Force of Will).
- Forth Eorlingas! on the opponent's turn or with a non-empty stack (sorcery timing is core).
- Attacking with X = 3 tokens: see the gap below.

## Schema gaps

- **No handle for one of several identical created tokens.** `bind` fails when more than one object matches, and doc 04 handles only count copies listed in the setup. A scenario cannot declare "attack with these three Knights" or block with one of them. The X = 3 scenario therefore asserts the three tokens' state only, and the combat scenarios use X = 1. Suggested fix: `bind` with `nth`, or a `declare_attackers` form `all: true`.
- The `copy_of` target spelling is not in SCHEMA.md (it comes from the storm-A notes).
- No assertion key for "spells cast by the opponent this turn" other than `spells_cast_this_turn` per seat (which I used).
