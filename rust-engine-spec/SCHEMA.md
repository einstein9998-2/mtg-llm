# Scenario format v1 (draft)

Status: v1-draft, written by the spec-writer thread. Extends `engine-design/04-differential-testing-spec.md` section 4.2 and uses the canonical action language (CAL) of section 3.2 there and the decision kinds of `engine-design/02-action-api-and-hidden-info.md` section 2.2. The implementer's scenario runner (doc 04 phase D0) consumes this. If the runner needs a format change, ask the spec-writer thread; do not edit scenario files or this file.

One scenario is one YAML document. A file may hold many documents separated by `---`. The validator is `tools/validate.py` (JSON Schema in `schema/scenario.schema.json` plus semantic checks: aliases, card names against `reference/oracle.json`, source rules).

## 1. Document skeleton

```yaml
id: daze-return-island-and-pay          # unique, kebab-case, stable forever
title: Daze countered Brainstorm unless the caster pays {1}
area: card                              # core | card | matchup | hidden-info
cards: [Daze, Brainstorm, Island]       # every distinct card name that appears anywhere in the scenario (validator checks)
tags: [alternative-cost, counter-unless-pay, mana-payment]   # rules-feature tags, free-form kebab-case, used for coverage and triage signatures
derived_from: ruling                    # ruling | cr | oracle   (never forge_observed in this directory; doc 04 section 4.3)
source:                                 # at least one entry
  - {kind: oracle, card: Daze, text: "Counter target spell unless its controller pays {1}.", verified: true}
  - {kind: cr, ref: "118.9a", text: "...", verified: true}
  - {kind: ruling, card: Aluren, date: "2004-10-04", text: "...", verified: false, note: "recalled, not retrieved"}
setup: {...}                            # section 2
random: any                             # section 5 (optional, default any)
script: [...]                           # section 3
expect: [...]                           # section 4
notes: "free text for reviewers (why this scenario exists, what a wrong engine would do)"
```

### Source rules (anti self-confirmation, doc 04 section 4.3)

- `verified: true` is allowed only when the quoted `text` is in `reference/oracle.json` (oracle), in `reference/cr-excerpts*.md` (cr), or was retrieved from a ruling source and the source is named in `note`. Everything else is `verified: false`.
- A scenario whose only sources are `verified: false` still counts, but it is listed in `COVERAGE.md` under "needs verification" and the reviewer must verify or downgrade it before the deck gate.
- Expectations must follow from the cited text. Do not copy expectations from any engine's output.

## 2. Setup

```yaml
setup:
  turn: 3                       # turn number (1 = first turn of the game); the first player is p0 unless `first_player` is given
  active: p0
  phase: main1                  # untap upkeep draw main1 begin_combat declare_attackers declare_blockers first_strike_damage combat_damage end_combat main2 end cleanup
  priority: p0                  # optional, default active
  p0:
    life: 20                    # default 20
    energy: 0                   # optional
    lands_played: 0             # optional, lands played this turn
    hand:       [Brainstorm, "Island @isl_hand"]
    battlefield:
      - Volcanic Island
      - {card: Island, as: isl1, tapped: true}
      - {card: Murktide Regent, as: murk, counters: {"+1/+1": 2}, sick: false, damage: 0}
      - {card: Cori-Steel Cutter, as: cutter, attached_to: "@murk"}
    graveyard:  [Lightning Bolt]     # bottom to top
    exile:      [{card: Force of Will, face_up: true}]
    library:    [Island, Ponder, Island]   # TOP FIRST
    library_pad: 20             # append N plain Islands at the bottom (so draws never fail unless the scenario means them to)
    command:                    # dungeons and emblems, rarely set up directly
      - {dungeon: Lost Mine of Phandelver, room: Goblin Lair}
    completed_dungeons: []      # names of dungeons this player has completed
    mana_pool: {}               # e.g. {R: 1}
  p1: {...}
  stack: []                     # rarely set directly; prefer building the stack with script actions
```

Entry shorthand: `"Name"`, `"Name @alias"`, or a mapping `{card, as, tapped, sick, counters, damage, attached_to, face_up, token, controller, flags}`. Defaults for battlefield permanents: untapped, not summoning sick (controlled continuously since before this turn), no counters, no damage. `sick: true` marks a permanent that came under its controller's control this turn. Tokens in setup: `{token: Goblin Token, as: gob}`.

Aliases (`as`, or `@alias` in the shorthand string) name one physical object. Scripts refer to it as `"@alias"`. Without an alias, the doc 04 handle `pN:Name#k` is available, where k counts copies of that name in that seat's setup in this fixed zone order: battlefield, hand, graveyard, exile, library, command.

The scenario's cards are exactly those listed in the setup (plus `library_pad`). There is no sideboard unless `sideboard: [...]` is given.

## 3. Script

A list of steps run in order. **Scenario mode runs with auto-resolution off**: every decision the engine raises is presented, including a priority decision whose only option is Pass and a choice with exactly one candidate (doc 02 section 5.2's skip policy is a runner option, not a rule). So scripts are explicit; `resolve_top` and `advance` are sugar for runs of explicit passes. A step is a mapping with exactly one of these keys:

| Key | Meaning |
|---|---|
| `p0:` / `p1:` | the named seat takes the given action or answers the given decision (section 3.1, 3.2) |
| `advance:` | both players pass priority in turn (taking no actions) until the engine gives priority in the named step; see 3.3 |
| `resolve_top: true` | sugar for two explicit passes: the player holding priority passes, then the other player passes. Fails if the stack is empty or the first passer does not hold priority. Any decision the resolution raises (a surveil, a pay-or-not prompt, a target for a trigger) is then answered by the following steps. Expands to the explicit pass actions before any oracle sees it. |
| `random:` | supply scripted randomness for the next random event (section 5) |
| `bind:` | name an object created during play (a token, a copy) so later steps can refer to it: `bind: {as: gob, find: {name: "Goblin Token", controller: p0}}`. Fails unless exactly one object matches. |
| `check:` | assertions evaluated now (same vocabulary as `expect`, section 4); a failure fails the scenario at that step |

Any step may carry `label: some-name` (so `expect` can refer to it) and `note: "..."`.

### 3.1 Priority-level actions (CAL macro actions, doc 04 section 3.2)

```yaml
- p0: {t: pass}
- p0: {t: play_land, card: "@isl1"}
- p0: {t: cast, card: "@bolt", targets: [{player: p1}], pay: auto}
- p0: {t: cast, card: "@fow", alt_cost: {exile_blue_card: "@brainstorm", pay_life: 1}, targets: ["@bolt"]}     # a spell on the stack is named by its card alias
- p1: {t: cast, card: "@daze", alt_cost: {return_island: "@isl2"}, targets: ["@brainstorm"]}
- p0: {t: cast, card: "@murk", delve: ["@gy1", "@gy2"], pay: auto}
- p0: {t: cast, card: "@dismember", phyrexian: [life, mana], targets: ["@gob"]}      # one entry per {B/P} symbol: life | mana
- p0: {t: cast, card: "@charm", modes: [2], targets: ["@a", {player: p1}]}          # modes are 1-based, in printed order
- p0: {t: cast, card: "@consign", replicate: 2, targets: [{trigger_of: "@delta", nth: 0}]}
- p0: {t: cast, card: "@meltdown", x: 1}
- p0: {t: activate, source: "@delta", ability: 0, targets: []}                        # ability index = position in the card's Oracle text, 0-based, counting only activated/mana abilities
- p0: {t: activate, source: "@petal", ability: 0, choose_color: R}                   # mana abilities are activatable on their own; the mana goes to the pool
- p0: {t: activate_from_hand, source: "@boseiju", ability: 1, targets: ["@land"]}    # channel and other hand abilities (Boseiju: {T}: Add {G} is 0, channel is 1)
```

`pay: auto` (default) means the engine's payment; equivalent payments must give the same canonical state (doc 04 3.2). `pay: {tap: ["@isl1", "@isl2"]}` names the sources to tap, for scenarios where the choice matters.

Targets: `"@alias"` (any object, including a spell on the stack by its card alias), `{player: p0|p1}`, `{trigger_of: "@alias", nth: n}` (the n-th ability currently on the stack whose source is that object, counted from the top, 0-based), `{ability_of: "@alias", nth: n}` (same, but for activated abilities; the validator accepts either, the runner treats them alike).

Free-cast permission use (Aluren, Omniscience) is a cast with `alt_cost: {free: aluren}` or `{free: omniscience}`. `alt_cost` is exclusive (CR 118.9a): a step naming two is a scenario about illegality and belongs in an `expect_illegal` step (3.4).

### 3.2 Decisions (sub-prompts of an action or of a resolving effect)

```yaml
- p0: {decision: choose_cards, purpose: put_back, answer: ["@a", "@b"]}       # Brainstorm: which two cards go back (unordered selection)
- p0: {decision: order_cards, purpose: put_back, answer: ["@a", "@b"]}        # then their order; the answer lists cards TOP FIRST
- p1: {decision: yes_no, purpose: pay_for_daze, answer: true}
- p0: {decision: choose_target, slot: 0, answer: "@x"}
- p0: {decision: choose_dungeon, answer: Lost Mine of Phandelver}
- p0: {decision: choose_room, answer: Goblin Lair}
- p0: {decision: order_triggers, answer: ["@src1", "@src2"]}                  # first listed is put on the stack first (resolves last)
- p0: {decision: legend_rule, keep: "@acererak_2"}
- p0: {decision: choose_color, answer: U}
- p0: {decision: choose_mode, answer: [1]}
- p0: {decision: choose_number, answer: 2}
- p0: {decision: surveil, to_graveyard: ["@c1"], on_top_order: ["@c2"]}        # also used for scry (to_bottom) via {decision: scry, to_bottom: [...], on_top_order: [...]}
- p0: {decision: search, found: "@island"}      # or found: null (fail to find)
- p0: {decision: simultaneous_secret_choice, answer: "@acererak"}               # Show and Tell; answer null means "put nothing"
- p0: {decision: assign_combat_damage, assignment: {"@blocker1": 2, "@blocker2": 1, player: 0}}
- p0: {decision: declare_attackers, attacks: [{creature: "@c", target: {player: p1}}]}     # also available as macro action {t: declare_attackers, ...}
- p1: {decision: declare_blockers, blocks: [{blocker: "@b", attacker: "@c"}]}
```

Every decision step may add `expect_options:` (the mask the engine must offer, doc 02 section 2.2), evaluated before the answer is applied:

```yaml
expect_options: {include: ["@a"], exclude: ["@b"], count: 3}      # include / exclude / exact (a full list) / count; entries are aliases, names, or canonical action patterns
```

`purpose` is a free label from doc 02's `DecisionKind` context and is checked only loosely (the runner may ignore it); the decision `kind` (first key) is checked strictly. If the engine raises a decision whose kind differs from the step, the scenario fails. If the engine raises no decision where the script expects one, it fails. If the engine auto-resolves a trivial decision (doc 02 section 5.2), the script omits the step; the scenario states in `notes` which decisions are trivial by the rules (exactly one legal option).

### 3.3 `advance`

```yaml
- advance: {to: main1, of: p0}     # stop when p0, as the active player, first has priority in main1
- advance: {to: end, of: p1}
- advance: {to: declare_blockers, of: p1}
```

Both players pass whenever they receive priority, until the engine gives the active player priority in the named step of the named player's turn (stack must be empty at each pass or the step fails). Turn-based actions and triggers on the way are processed by the engine; if a decision other than priority is raised on the way (a trigger target, a discard to hand size, a surveil), the step fails, so the script must either add that step explicitly before the `advance` or use explicit `pass` actions instead.

### 3.4 Illegality probes

```yaml
- expect_illegal: {p0: {t: cast, card: "@bolt", targets: ["@hexproof_thing"]}}
```

Asserts that the action is not in the legal set at that decision (doc 02 section 2.2: illegal moves are impossible by construction) and that no state changes. For macro-level actions the runner checks that no legal macro action matches. This is how scenarios say "X is not offered".

## 4. Expectations

```yaml
expect:
  - at: after_script            # or a step label
    game: {over: false}
    turn: 3
    phase: main1
    active: p0
    priority: p0
    stack: []                   # bottom to top; entries described in 4.2
    pending: {actor: p0, kind: priority}     # who must act next, and the decision kind; kind in the doc 02 DecisionKind spelling snake-cased
    p0:
      life: 19
      energy: 0
      hand: [Brainstorm, Island]             # multiset of names, exact
      hand_count: 2                          # when the identities are not the point
      library_count: 24
      library_top: [Island, Ponder]          # top first, a prefix of the library
      library_bottom: [Island, Island]       # bottom-most last, a suffix
      library_bottom_multiset: [Island, Island]   # when the order was random
      graveyard: [Lightning Bolt]            # multiset exact; use graveyard_ordered (bottom to top) when order matters
      exile: [Force of Will]                 # multiset exact; {card, face_up} entries allowed
      battlefield: [...]                     # exact multiset of permanent descriptors (4.1); battlefield_contains for subsets
      mana_pool: {}
      lands_played: 1
      spells_cast_this_turn: 2
      command: [{dungeon: Lost Mine of Phandelver, room: Cave Entrance}]
      completed_dungeons: []
      designations: []
      known_to_me: {...}                     # hidden-info scenarios only (section 6)
    p1: {...}
    legal: {p0: {include: [{t: pass}, {t: cast, card: "@bolt"}], exclude: [...]}}     # offered macro actions at the pending decision (doc 04 section 5.2)
    events_contain: [ {kind: zone_change, card: "@bolt", from: stack, to: graveyard} ]   # optional, T2
```

Unmentioned fields are not checked, except that `battlefield`, `hand`, `graveyard`, `exile`, and `stack` are exact when present.

### 4.1 Permanent descriptors

`{card: Murktide Regent, controller: p0, tapped: false, counters: {"+1/+1": 3}, pt: [6, 6], damage: 0, types: [Creature], keywords: [flying], attached_to: "@x", sick: false, token: false, alias: murk}`. Only listed fields are checked. `pt` is the derived power/toughness after all effects. `keywords` lists keywords the permanent must have (a subset check: extra keywords such as delve do not fail it); use `keywords_exclude` to assert absence. Tokens are written `{token: Goblin Token, ...}`.

### 4.2 Stack entries

`{kind: spell, card: "@bolt", controller: p0, targets: [...], modes: [...], x: null}` or `{kind: triggered, source: "@acererak", text_contains: "return"}` or `{kind: activated, source: "@delta"}`, listed bottom to top.

## 5. Randomness

`random: any` (default) means the scenario is written so every outcome gives the same asserted result; assertions on shuffled libraries use `library_count` and multisets only. For scenarios that need a fixed outcome, supply a queue of scripted outcomes consumed in order:

```yaml
random:
  - {kind: shuffle, player: p0, result: [Island, Ponder, Island]}     # library order after the shuffle, top first (names; interchangeable copies need no handles)
  - {kind: bottom_order, player: p0, result: [Island, Lightning Bolt]}
```

If the engine consumes a random event the queue does not cover, or the queue has entries left at the end, the scenario fails (this catches engines that shuffle too often or too rarely).

## 6. Hidden-information scenarios (`area: hidden-info`)

Add `observer: p0` and assert what that seat can see (doc 02, doc 04 section 7.2):

```yaml
expect:
  - at: after_script
    observer: p0
    obs:
      opp: {hand_count: 6, revealed_hand: [], library_count: 30}
      me:  {library_known_top: ["Ponder"]}
      option_labels_exclude_names: [Force of Will]     # card names that must appear nowhere in the observation, option labels, or event text
      events_text_exclude: [Force of Will]
```

A paired-world form uses two setups that differ only in what the observer is not entitled to know and asserts identical observer streams:

```yaml
world_b:                       # replaces parts of setup; observer-visible state must stay identical
  p1: {hand: [Island, Island]}
assert_streams_identical: {observer: p0}
```

## 7. Holdout

Authors never mark scenarios as holdout and never see the assignment. `tools/split_holdout.py` takes a secret seed and moves about 30% of each card's scenarios (and of each core area's scenarios) into the sealed archive; see `holdout/README.md`.

## 8. Stability and versioning

`schema: 1` is implied. A breaking change bumps the schema number and `tools/validate.py` supports both for a transition period. Scenario ids never change; a rewritten scenario keeps its id and records `supersedes: <old-hash>` in a `history` list.

## Addenda from review (2026-10-01)

- `advance` skips only steps where the engine would raise no decision. If a creature can attack, `declare_attackers` is a decision and `advance` stops there; with no legal attacker, no decision is raised and `advance` passes through combat. Scenarios that need the declaration use an explicit step.
- `declare_attackers` / `declare_blockers` are single decisions carrying the whole declaration (design doc 02 describes one decision per creature; the scenario runner adapts).
- Equipment: `attached_to` is set on the Equipment and names the creature.
- `game: {over: true, winner: pN}` or `game: {over: true, draw: true}` may be asserted; `game: {over: false}` as before.
- Decisions on which the rules give the player no choice are not raised: `assign_combat_damage` is raised only when the attacker has more than one blocker or has trample with a blocker (CR 510.1c); a single blocker without trample takes all the damage automatically. "One-candidate choices are presented" applies to choices the rules give the player (surveil, Brainstorm put-back, targets).
- `assignment: {player: N}` means the defending player (two-player game) for trample damage.
- `expect_illegal` may carry a decision answer (for example an illegal `declare_blockers` or `assign_combat_damage`).
- Acererak's attack trigger ("unless that player sacrifices a creature of their choice") is canonically two steps, both by the defending player: `yes_no` (sacrifice?) then `choose_cards` (which creature). The rules fix who decides and the outcomes, not this shape; the runner adapter owns the exact shape.
- `check` is allowed before any step (including while a decision is pending); `expect_options` may carry `required: [...]` for forced attackers.
- `setup.stack` entries may start a scenario with an ability already on the stack: `{kind: triggered, source: <alias or card>, controller: pN, targets: [...], text_contains: "<fragment of the ability text>"}`, listed bottom to top. Used for dungeon room abilities so venture mechanics need not be replayed.
- Dungeon cards in the command zone are referenced by the doc 04 handle `"pN:<Dungeon>#k"` (the validator does not register `as:` on them).
- No decision is raised for a declaration with no legal creature (attackers or blockers): `advance` passes through. A trivial `order_replacements` step is likewise not raised when every order gives the same final state (CR 616.1); where the order matters the scenario must supply the step.
- `attached_to: null` asserts unattached. `game: {over: true, result: draw}` is accepted as a synonym of `draw: true`.
- Replicate copy retargeting (CR 707.10c, "may choose new targets"): canonically `yes_no` (change targets?) then, on yes, `choose_target` with new legal targets, both by the copy's controller. Same two-step shape as other "may" choices; the runner adapter owns exact shape.
- Observation key `library_known_bottom: [names]` (bottom-most last) lists library cards at the bottom whose identity AND position the observer knows (design doc 02 section 6: FromBottom(n)). A card put on the bottom in a chosen order is known to its owner; cards bottomed in random order are not (empty list).
- Decision shapes used by W5 and accepted: Atraxa's pick is one `choose_cards` (purpose to_hand, one card per card type at most); a "may search" is `yes_no` then `search` (the `search` answer may be `found: null`); Carpet of Flowers is `yes_no` then `choose_color`; Acererak/replicate shapes above. Runner adapters own exact shapes; the rules outcomes are what the scenarios fix.
- Show and Tell: a player with no eligible card has no choice, so no decision is raised for that player; a player with an eligible card gets a `choose_cards`/may-choice (secret from the other player; each player must clearly indicate that they chose, CR 101.4a/b, but not which card).
- Boseiju search: `yes_no` then `search` (not a single `search` with a null option). Scenarios written the other way are corrected at review.
- `activate_from_hand` and `activate` ability indexes count the card's abilities in Oracle text order, mana abilities included (Boseiju: `{T}: Add {G}` is 0, channel is 1).
- `alt_cost: null` in a `legal` pattern means "cast paying the mana cost". `random: []` means no random events may occur. A pattern with no `targets`/`pay` matches any completion.
- `world_b` is deep-merged over `setup` (maps merge key by key, lists replace). Both worlds use the same seed and the same script; `expect` runs in world A; `assert_streams_identical` compares everything the observer sees (observations, option labels, event text) across the whole script.
- Default knowledge at setup: nobody knows any library order unless `library_known_top`/`library_known_bottom` says so. The decklist is not part of the observation.
- Venture (CR 701.49b, 309.5a): the first venture puts the marker on the topmost room (its ability triggers); afterwards `choose_room` is raised only when the current room has multiple arrows. One-exit rooms raise no decision. A room with several sacrifice/discard clauses (Oubliette): discard first (608.2c), then one unordered `choose_cards, purpose: sacrifice` covering the creature, artifact and land.
- `choose_name` is a decision kind (Disruptor Flute): `{decision: choose_name, answer: "<exact card name>"}`.
- Dungeon room text sources are `kind: oracle` with `card: <Dungeon name>` and `verified: false` until the validator learns dungeons; reviewers check them against `reference/oracle.json` dungeons.
- Surgical Extraction: one `search` decision across zones, `{decision: search, found: ["@a", "@b"]}` (any number of same-named cards from graveyard, hand, library; empty list allowed, CR 701.23b).
- Cori-Steel Cutter flurry: after the Monk is created, `yes_no` with `purpose: attach_cutter` (Equipment attach is a "may").
- Scry/bottom with several cards: order of bottomed cards unspecified; assert only the multiset unless the player chose the order (`order_cards, purpose: bottom`).
- `obs.opp` accepts the same keys as `obs.me` (`library_known_top`, `library_known_bottom`, `revealed_hand`, `hand_count`, `library_count`). A search decision is raised for a search even when nothing legal can be found (the answer is `found: null`).
- `symmetric: true` (optional top-level key): the scenario's outcome must not depend on seat order. The runner runs it twice, the second time with p0 and p1 swapped throughout (setup, script, expectations). Use it on scenarios where both seats could be the actor (an any-player card such as Aluren, a mirror setup). Idea from the Forge interaction tests (a Forge bug made Aluren usable only by the first seat).
- Lands with basic land types (Volcanic Island, Tropical Island, Hedge Maze, ...) have one intrinsic mana ability per basic type (CR 305.6). Scenarios write `{t: activate, source: ..., ability: 0, choose_color: U}`: `ability: 0` means "its mana ability" and `choose_color` picks which intrinsic ability is used.
- Permanent descriptors accept `subtypes: [...]` (subset check of the derived subtypes) alongside `types`.
- Open rules question (not asserted by any scenario): if p0 passes, p1 activates a mana ability and then passes, p0 gets priority again (117.4 requires no actions between the passes). Scenarios are written to give the same result under either reading where possible.
- Alternative-cost cast syntax (wave 2): `alt_cost: {flashback: true}`, `{escape: {exile: [aliases]}}` (the whole exile set is one choice; the adapter expands doc 02's sequential ChooseCards), `{evoke: {exile_white_card: "@x"}}`, `{exile_black_card: "@x"}` (Unmask), `{pay_life: 4}` (Snuff Out). A `legal` pattern without `alt_cost` matches any way of casting; `alt_cost: null` matches only paying the mana cost.
- `order_triggers` answers for different triggers controlled by one player: a list of stack descriptors, the first listed goes on the stack first (resolves last): `[{source: "@ph", text_contains: "deals 3 damage"}, ...]`.
- Cavern of Souls chosen type in setup: `flags: {chosen_type: <creature type>}`.
- Decision purposes used in wave 2: `put_onto_battlefield` (Aether Vial "may put": one `choose_cards`, empty answer declines), `cast_from_graveyard` (Bilbo: `choose_cards`, then `choose_target`; payment automatic), `doomsday_pile` then `doomsday_order` (answer top first), `put_on_top` (Thassa's Oracle; not raised when X = 0).
- `symmetric: true`: when seats are swapped, `first_player` swaps too so that the same player's turn is active at the scripted time.
- Standardized spellings (wave 2 reviewers conform scenarios to these): escape is `alt_cost: {escape: {exile: [aliases]}}`; a card-granted free cast (Massacre) is `alt_cost: {free: true}`; Bilbo-style "may cast from graveyard" during resolution is one `choose_cards` with `purpose: cast_from_graveyard` (empty answer declines) then `choose_target` if needed, payment automatic; ninjutsu's returned attacker is `choose_cards, purpose: return_unblocked_attacker` after the activation; planeswalker and stun counters are `counters: {loyalty: N}` and `{stun: N}`; an attacking creature descriptor accepts `attacking: true`; emblems are asserted as `command: [{emblem: "<text fragment>"}]`. Printed loyalty is not in `oracle.json`; scenarios put loyalty in setup and assert only changes.
- More standardized spellings: `order_triggers` for two triggers from one source uses `[{source: "@x", text_contains: "..."}, ...]` (first listed goes on the stack first); evoke is `alt_cost: {evoke: {exile_white_card: "@x"}}`; warp is `alt_cost: {warp: true}`; a copy on the stack is `{kind: spell, card: <Name>, copy: true}` (assertable in `stack`); energy payment is `{decision: choose_number, purpose: pay_energy}` and is raised only when there is a real choice. For triggers with targets: `order_triggers` first, then `choose_target` as each goes on the stack. Delayed-trigger source for warp is not fixed by the CR: do not assert it in `stack`.
- Flashback with a choice: `alt_cost: {flashback: true}`, or `{flashback: {sacrifice: "@creature"}}` (Cabal Therapy). Cost choices that are part of a cast or activation sit on the action like `delve:`: `sacrifice: ["@x"]`, `discard: ["@x"]`. `observer` + `obs` may appear in a `check` step (assert what an observer sees while a decision is pending). Doomsday: `choose_cards, purpose: doomsday_pile` (raised only when more than five candidates exist; with five or fewer all are kept, CR 609.3) then `order_cards, purpose: doomsday_order` (top first). Other loose purposes accepted: `oracle_top`, `tutor`.
- Cost choices on activations use the same direct fields as casts: `sacrifice: ["@x"]` (Goblin Bombardment, Lazotep Quarry sacrificing itself), `discard: [...]`; `x: N` on an activation or cast sets X; `choose_color: C` on a mana ability that makes one mana of any color. Permanent descriptors also accept `colors: [W, U, ...]` (subset check of derived colors). Designations (city's blessing, monarch) have no assertion key yet: assert their effects only.
- Keyword spelling: lowercase Oracle keyword names with spaces ("first strike", "double strike"), as already used. Impending: `alt_cost: {impending: true}`; setup flag for a permanent that was cast for its impending cost: `flags: {impending_paid: true}`. The Ring (emblem, tempt count, Ring-bearer) has no assertion key: assert effects only. "Up to one" target with candidates but zero chosen: answer `targets: []`.
- Targeting an ability on the stack (Stifle, Consign to Memory): `{trigger_of: "@source"}` with optional `text_contains: "..."` to pick between several abilities of one source.
- Permanent descriptor sets: `types` is the EXACT set of card types (Creature, Enchantment, ...), so `types: [Enchantment]` proves a permanent is not a creature; `subtypes`, `supertypes`, `keywords` and `colors` are subset checks.
- `purpose:` on a decision is a free-form label that documents what is being decided; runners match on decision kind and actor, and use the purpose only as a hint. The purposes used in the pool are therefore not a closed vocabulary.
- `bind.find` keys: `name`, `controller`, `token: true|false`, `zone`. By default it searches the battlefield and the stack only (a card with no controller reports its owner, CR 108.4a, so searching exile or the graveyard would bind ambiguously); name the `zone` explicitly to bind elsewhere.
- Identical triggers (same source, same ability text, same target after targets are chosen) raise no `order_triggers` decision (design doc 01 section 7.3): the order cannot change the outcome. Different abilities of one source (Phlage's two triggers, Solitude's two) do raise it. `ability_of` / `trigger_of` without `text_contains` is ambiguous whenever one source has several abilities on the stack (a sacrifice-as-cost ability plus its dies trigger: Nihil Spellbomb, Mishra's Bauble): always add `text_contains` there; `nth` counts from the top of the stack.
- One instruction with several independent choices (Oubliette: "sacrifice a creature, an artifact, and a land"): the rules do not order the choices, so a runner adapter may raise them as one unordered `choose_cards` selection or as one prompt per category in text order; scripts written after the holdout run use the one-at-a-time form with `expect_options` per category. Do not write a scenario that depends on which of the two shapes an engine uses without saying so in its note.
- `legal:` patterns for alternative costs that name a chosen object (Daze `alt_cost: {return_island: "@x"}`) prove only that the alternative way is offered, not which object may be chosen. To prove a specific object is not allowed use a script step `expect_illegal: {p1: {t: cast, card: "@daze", alt_cost: {return_island: "@mtn"}}}`.
- Setup entries for tokens accept the same keys as card entries (`counters`, `damage`, `tapped`, `sick`, `attached_to`); a runner that drops any of them makes scenarios fail for reasons that are not rules bugs.
