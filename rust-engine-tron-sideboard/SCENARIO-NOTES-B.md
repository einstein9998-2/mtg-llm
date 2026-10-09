# Group B scenarios (Argentum Masticore and Mycosynth Lattice): notes for the implementer

34 scenarios, one file each, `scenarios/tron2-B-<topic>.yaml` (ids `tron2-b-...`). Written from the Oracle text in SCENARIO-BRIEF.md (re-checked with Savecraft `card_search`, exact name, 2026-10-09) and the Comprehensive Rules (2025-11-14, Savecraft `rules_search`: rules 702.16, 702.7, 105.2, 105.4, 603.12 retrieved fresh; the others copied from the existing `rust-engine-spec/reference/cr-excerpts*.md`, which came from the same module). No engine source, patch or `*.cards.ron` was read and no `mcp__hearthbot__*` tool was called. Run result against `/home/claude/target-tron/release/specrun` (2026-10-09): 34 pass, 0 fail, 0 unsupported. I confirmed the checks bite by mutating four expectations in a scratch copy (all four failed).

## Delivery and validation

- `oracle-additions-B.json`: `cards` Argentum Masticore and Mycosynth Lattice (every other card used is already in the reference oracle or the Tron/Storm/UR/Boros additions).
- `cr-excerpts-tron2-B.md`: every CR rule a `verified: true` source cites.
- Validation: scratch `oracle.json` (reference file plus all additions plus group B's) and a refs dir holding only it and `cr-excerpts-tron2-B.md`:
  `python3 /mnt/project-files/rust-engine-spec/tools/validate.py --oracle <scratch>/refs2/oracle.json --refs <scratch>/refs2 scenarios/tron2-B-*.yaml` -> `34 scenario(s), 0 error(s), 0 warning(s)`.
- No `ruling` sources and no `verified: false` entries: every expectation follows from Oracle text plus CR.

## Scenario list (file prefix tron2-B-)

### Argentum Masticore (16)

| file | asserts |
|---|---|
| masticore-upkeep-empty-hand-sacrificed | hand empty at upkeep (before the draw): sacrificed with no decision, no reflexive trigger |
| masticore-upkeep-decline-discard-sacrificed-hand-kept | hand non-empty: `yes_no` false = sacrificed, hand untouched, nothing destroyed |
| masticore-discard-mv-one-bounds-the-target | discard Bolt (MV 1): offered targets are the opponent's MV<=1 nonland permanents only (Bauble 0, Key 1); not Null Rod (2), a land, or p0's own permanents; target chosen as the trigger goes on the stack |
| masticore-discard-a-land-targets-only-mana-value-zero | discard Island (MV 0): token and Lotus Petal only; a token destroyed ceases to exist |
| masticore-discard-three-mana-card-bounds-at-three | discard Dismember (MV 3, Phyrexian hybrid): Trinisphere (3) and Null Rod (2) yes, Karn (4) and land no ("less than or equal") |
| masticore-discard-x-spell-counts-x-as-zero | discard Meltdown {X}{R}: X is 0 in hand (202.3e), MV 1, Null Rod (2) not offered |
| masticore-discard-no-legal-target-trigger-removed-masticore-stays | no legal target (only a MV 2 permanent and a land on the other side; p0's own cheap permanents excluded): no target decision, trigger removed, discard done, Masticore stays |
| masticore-reflexive-trigger-resolves-after-masticore-exiled | Swords to Plowshares (mono-white, legal despite the protection) exiles the Masticore in response to the reflexive trigger; the trigger still destroys its target (113.7a); p0 gains 5 |
| masticore-destroys-nothing-indestructible-the-one-ring | indestructible target survives; targeting it is legal; Masticore stays |
| masticore-triggers-only-on-its-controllers-upkeep | Masticore under p1: nothing at p0's upkeep; at p1's upkeep p1 decides, "an opponent" is p0 |
| masticore-stifle-the-upkeep-trigger-no-sacrifice-no-discard | Stifle on the upkeep trigger: Masticore stays, no discard prompt |
| masticore-spell-countered-by-force-of-will-force-of-negation-cannot-target | Force of Negation cannot target the creature spell; Force of Will pitching Brainstorm counters it |
| masticore-protection-multicolored-prismari-charm-cannot-target-bolt-can | Prismari Charm (both bounce and damage modes) cannot target; mono-red Bolt can (3 damage) |
| masticore-multicolored-creature-cannot-block-it-mono-colored-can-first-strike | Lavinia (WU) cannot block the attacking Masticore (`expect_illegal` on the declaration); Containment Priest can, dies to first strike, deals nothing |
| masticore-first-strike-kills-five-five-attacker-takes-no-damage | a 5/5 attacker blocked by the Masticore dies in the first-strike step and deals no damage |
| masticore-damage-from-multicolored-creature-is-prevented | Gaddock Teeg (6/6 with counters) attacks into the blocking Masticore: first-strike 5 on Teeg, Teeg's 6 damage prevented (702.16e), Masticore undamaged |

### Mycosynth Lattice (18)

| file | asserts |
|---|---|
| lattice-all-permanents-are-artifacts-in-addition-to-their-types | exact `types` for both players' land, creature, token, planeswalker, artifact |
| lattice-voltaic-key-untaps-a-tapped-island | a land is a legal target of "untap target artifact" |
| lattice-karn-static-stops-opponents-lands-tapping-for-mana | Karn's "activated abilities of artifacts your opponents control can't be activated" now covers the opponent's lands' mana abilities |
| lattice-karn-plus-one-targets-a-land-which-dies-as-zero-zero | Karn +1 targets a land (noncreature artifact); it becomes a 0/0 and dies (704.5f) |
| lattice-null-rod-stops-every-permanents-activated-abilities-lands-included | Null Rod + Lattice: own lands' mana abilities and Wasteland's sacrifice ability cannot be activated |
| lattice-hydroblast-cannot-counter-a-red-spell-it-is-colorless | spell on the stack is colorless: Hydroblast's counter mode does nothing |
| lattice-pyroblast-cannot-destroy-a-blue-permanent-it-is-colorless | permanent is colorless: Pyroblast's destroy mode does nothing to Thassa's Oracle |
| lattice-consign-to-memory-counters-a-red-spell-as-colorless | Consign to Memory may target (and counter) a red spell |
| lattice-force-of-will-cannot-exile-a-blue-card-from-hand | the free alternative cost is unavailable (hand cards are colorless) |
| lattice-force-of-will-hard-cast-with-colorless-mana | {3}{U}{U} paid with only colorless mana (Ancient Tomb, City of Traitors, Wasteland); Lattice works for the opponent's mana |
| lattice-colored-mana-still-cannot-pay-a-colorless-symbol | Warping Wail {1}{C} not castable with Islands, nor with {U} floating |
| lattice-colorless-mana-pays-ww-for-either-player | `symmetric: true`; Grim Monolith pays {W}{W} for White Orchid Phantom, Lattice under the other seat |
| lattice-floating-colorless-mana-pays-colored-symbols | mana already in the pool ({C}{C}{C}) pays {W}{W}; {C} left over |
| lattice-leaving-restores-colors-types-and-mana-restrictions | Abrade destroys the Lattice; lands lose Artifact, the Channeler is red again, the Phantom can no longer be cast with colorless mana |
| lattice-snuff-out-black-creature-is-nonblack-cast-with-islands | Snuff Out hard-cast with four Islands (any color) destroys a "nonblack" Street Wraith |
| lattice-mox-opal-metalcraft-counts-lands-as-artifacts | Lattice + Mox Opal = 2 artifacts, no metalcraft; playing a land makes 3 |
| lattice-multicolored-prismari-charm-can-target-argentum-masticore | under Lattice the Charm spell is colorless: it can target the Masticore (bounce resolves) |
| lattice-multicolored-creature-damage-to-argentum-masticore-not-prevented | Teeg is colorless: its 6 damage to the Masticore is not prevented and the Masticore dies (mirror of the no-Lattice Teeg scenario) |

## Spellings I used (the adapter owns exact shapes)

- Upkeep walk: setup `turn: 2, active: p1, phase: end`, then `advance: {to: upkeep, of: p0}` stops with the Masticore trigger on the stack; `resolve_top: true`; then for a non-empty hand `yes_no` (purpose `discard_to_keep_masticore`, answer true = discard), `choose_cards` (purpose `discard`), then `choose_target` slot 0 for the reflexive trigger (raised even with a single candidate; not raised with none; the target is on the stack before priority, `stack: [{kind: triggered, source: "@mast", targets: [...]}]`), then `resolve_top` for the reflexive trigger. With an empty hand no decision is raised. The implementation passed this exact sequence.
- `expect_options` with `include`/`exclude`/`count` on the `choose_target` decisions carries the MV and "opponent's nonland permanent" assertions.
- Illegal declarations (`expect_illegal` of `declare_blockers`) and illegal casts, as in the existing pool.
- Alternative cost for Force of Will: `alt_cost: {exile_blue_card: "@bs", pay_life: 1}` as in the existing Tron scenarios. Modes of modal spells: 1-based `modes: [n]`.
- Stifle target `{trigger_of: "@mast"}` (a single ability of that source is on the stack).
- Seat swap: only the White Orchid Phantom scenario is `symmetric: true`.

## Rulings and rules decisions worth knowing (no unverified items)

- "Sacrifice unless you discard" with an empty hand: no choice exists, so the sacrifice happens without a decision (118.12a: the player "may" discard; nothing to discard). The brief asks for this.
- Mana value of the discarded card is read from the card in the graveyard/hand: X is 0 (202.3e); a land is 0; there is no "tokens are not cards" angle (the card is the discarded card).
- The reflexive trigger is a separate triggered ability that triggers on the discard and resolves independently of the Masticore (113.7a); the target legality is checked on being put on the stack and on resolution.
- "Up to one target" triggers (White Orchid Phantom's ETB) go on the stack with no target even when no candidate exists; my first draft assumed they were removed (603.3d only removes a trigger that REQUIRES a target it cannot have). The engine was right and the scenarios were corrected.
- Protection from multicolored is tested for targeting, damage and blocking only, as the brief asks (not enchanting/equipping).

## Not covered (by the brief's exclusions or by design)

- Orim's Chant, Red Elemental Blast and Force of Negation's "blue card" pitch (only Force of Will is tested; Force of Negation behaves identically).
- Masticore in response to a Karn static (a triggered ability is not activated, so Karn is irrelevant; one scenario mentions it in a note).
- Protection from multicolored on a Masticore that has been turned into a different color by an effect (no such effect in the pool besides Lattice).
- Lattice affecting opposing "cast" costs of artifact-matters cards beyond Mox Opal, Voltaic Key, Karn and Null Rod.
- Colorless-source damage to the Masticore (an artifact or colorless source is fine) is implied by the Lattice Teeg scenario, not tested separately.

## Schema gaps

- No assertion key for "colorless": `colors: [R]` is a subset check, so a creature's loss of color under Lattice can only be shown indirectly (the target-legality scenarios) and its regained color is shown after the Lattice leaves. A `colors_exact` key would let the Lattice scenarios assert `colors: []`.
