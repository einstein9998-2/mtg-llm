# MTG Engine — Differential Testing and Verification Spec

Status: DRAFT v0.1. Companion to `01-core-design.md`, `02-action-api-and-hidden-info.md`, `03-card-dsl.md`. Section numbers 4.4, 7, 8, 9 are referenced from those docs.

Verification is the real bottleneck of an agent-built engine (brief). This document specifies how we make "the engine is correct" an evidence-backed claim per deck, and how we keep agents from writing tests that merely agree with their own code.

Statements about Forge internals below were checked by reading the Forge source (commit `fd5c996`, 2026-09-30, a shallow clone). I have **not built or run** Forge or XMage, so anything about runtime behavior is still marked **(verify)** and should be confirmed by the Phase 1 Forge work. Where the Phase 1 deliverables (headless Forge runner, legal-action enumerator, state serializer) already provide a component, I say so, because the differential harness should reuse them rather than duplicate them.

---

## 1. Purpose, oracles, and the trust model

### 1.1 Four sources of truth, ranked by how much we trust them

| Rank | Source | Role | Failure mode |
|---|---|---|---|
| 1 | **Comprehensive Rules + official Gatherer/Scryfall rulings** (pinned CR version) | Final authority | Ambiguity; needs human or careful agent interpretation |
| 2 | **Hand-authored scenario expectations** derived from rulings by the spec-writer agent | Executable form of rank 1 | Mis-encoding a ruling |
| 3 | **Forge** (primary differential oracle) | Independent implementation; catches our bugs at scale | Forge's own bugs; lag behind the pinned CR version; simplified card scripts |
| 4 | **XMage** (optional tie-breaker oracle) | Third opinion when Forge and we disagree | Same class of bugs as Forge; extra adapter cost |

**A disagreement with Forge is a finding, not a verdict.** The triage workflow (section 6) decides who is right using ranks 1-2 (and rank 4 when available).

### 1.2 What each technique is good at

| Technique | Finds | Does not find |
|---|---|---|
| Rulings-as-spec scenarios | Known tricky interactions, each card's documented behavior | Interactions nobody wrote a ruling for |
| Lockstep differential (random legal games) | Unknown interactions, ordering bugs, state divergence at scale | Bugs both engines share; bugs where Forge is wrong and we copied the same mistake |
| Invariant fuzzing | Engine-internal inconsistency (conservation, zones, cache coherence, determinism, mask validity, info leaks) | Wrong-but-self-consistent rules |
| Self-play anomaly detection | Bugs only exploitable by strong or weird play | Bugs that never matter to play |
| Human spot checks | Systematic misunderstandings | Scale |

Deck gate (section 8) requires all of them.

---

## 2. Architecture

```text
                         +--------------------+
   Scenario / Generator  |   mtg-diff driver  |   Comparator + Shrinker + Triage queue
   (YAML, seeds, cov.)-->|  (Rust, in-process)|---> mismatch records, coverage, dashboards
                         +----+----------+----+
                              |          |
                 EngineDriver |          | EngineDriver
                              v          v
                    +---------------+  +---------------------------+
                    | OurDriver     |  | ForgeDriver (JSON lines    |
                    | in-process    |  | over stdio/socket to a     |
                    | (diff-harness |  | long-lived JVM)            |
                    |  feature)     |  +---------------------------+
                    +---------------+  | XMageDriver (optional)     |
                                       +---------------------------+
```

### 2.1 The `EngineDriver` trait

```rust
trait EngineDriver {
    fn reset(&mut self, setup: &GameSetup) -> Result<()>;              // decks, opening hands, library order, board state, life, turn/phase
    fn legal_actions(&mut self) -> Result<DecisionSnapshot>;           // who decides, kind, legal macro-actions in canonical form
    fn apply(&mut self, action: &CanonicalAction) -> Result<ApplyOutcome>;
    fn snapshot(&mut self) -> Result<CanonicalState>;                  // full, canonicalized (section 3.3)
    fn events_since_last(&mut self) -> Result<Vec<CanonicalEvent>>;   // optional but strongly preferred
    fn provide_randomness(&mut self, outcomes: &[RandomOutcome]);      // follower mode (section 5.3)
    fn drain_randomness(&mut self) -> Vec<RandomOutcome>;              // leader mode: what the engine chose
}
```

The driver for our engine is trivial and in-process. It uses `mtg-debug` (the `diff-harness` feature), which exposes the full state and is never linked into agent builds (doc 02 section 3).

### 2.2 Forge adapter

**Approach.** Run a long-lived JVM with a thin adapter in front of Forge's headless game engine, controlled by JSON lines over stdio or a local socket. A custom player controller replaces the AI: every decision Forge raises (priority, targets, costs, ordering, optional choices) blocks the Forge game thread until the harness answers.

**What the Forge source confirms (commit fd5c996):**

- **Custom controllers are a supported shape.** `forge.game.player.PlayerController` is an abstract class (about 110 abstract methods) with three existing implementations: `PlayerControllerAi`, `PlayerControllerHuman`, and a test-only `PlayerControllerForTests` in `forge-gui-desktop/src/test/.../gamesimulationtests/util`. The adapter is a fourth implementation that forwards each call over the protocol. It is large but mechanical.
- **Decision API maps to macro actions.** `chooseSpellAbilityToPlay()` returns the abilities to play (one per priority decision); targets, modes, costs, and payment follow as separate calls (`chooseTargetsFor`, `chooseModeForAbility`, `payManaCost`, `chooseCardsForCost`, `chooseOptionalCosts`, ...). `orderSimultaneousSa` and `chooseSingleReplacementEffect` give the trigger and replacement ordering decisions; `declareAttackers`/`declareBlockers`/`assignCombatDamage`/`orderBlockers` cover combat; `mulliganKeepHand`/`tuckCardsViaMulligan` cover mulligans; `vote` covers Council's Judgment; `chooseCardsPile` exists for pile-style effects.
- **Headless mode exists.** `forge-gui-desktop/.../view/SimulateMatch.java` (the `sim` command) runs AI-vs-AI games with an optional RNG seed (`-s`), which is the Phase 1 batch runner's starting point. Instrumenting it with a custom controller is the adapter's entry point.
- **State injection exists.** `forge.game.GameState` loads a text format (`turn=`, `activeplayer=`, `activephase=`, per-player `life`, `landsplayed`, `manapool`, and per-zone card lists for hand, battlefield, graveyard, library, exile, command, sideboard). Per-card modifiers include `Tapped`, `Counters:`, `SummonSick`, `Damage:`, `AttachedTo:`, `ExiledWith:`, `Id:`, `NamedCard:`, `ChosenType:`, `FaceDown`, `Transformed`, `PhasedOut`. The state can **precast** spells and **put spells on the stack** (`precast`, `putonstack` keys, intended for puzzle mode). Library zones are set from the card list as given (`zone.setCards`), and Forge draws from index 0 (`library.get(0)` in `Player.java`), so the first-listed library card is the top. That gives us scripted openings and library order directly. Battlefield cards enter through the normal move path (ETB triggers run) unless a card is marked `NoETBTrigs`.
- **Legal ability enumeration exists in the engine.** `Card.getAllPossibleAbilities(player, removeUnplayable)` returns a card's abilities that the player can currently play. The AI and several effects use it. The Phase 1 enumerator should build on it; the harness reuses that enumerator.
- **Forge has its own game-simulation test harness** (`forge-gui-desktop/src/test/.../gamesimulationtests`, with `comprehensiverules` tests for CR sections 103 and 104 only). It is a pattern for the "known interactions" suite, not coverage we can rely on.
- **Yield/auto-pass is UI-layer.** Forge's advanced yield options live in `InputPassPriority`, `YieldStateSnapshot`, and the human controller, and the engine calls `autoPassCancel()` on the controller at turn boundaries. A custom controller bypasses them. I have not verified that no engine-level auto-decision remains (for example, simultaneous-trigger ordering is delegated to the controller via `orderSimultaneousSa`, which is good).
- **Events exist.** `forge.game.event` has about 60 event classes (zone change, damage, tap, counters, spell cast, spell resolved, shuffle, scry, mulligan, token created, turn phase, and so on), usable for `CanonicalEvent`s.
- **Rules-version divergence is real.** Forge's combat still asks for blocker order (`PlayerController.orderBlockers`, `Combat.orderBlockersForDamageAssignment`), while the CR effective 2025-11-14 removed damage assignment order (CR 510.1c). See kd-0001 in section 9.

**Remaining adapter hard parts.**

1. **Randomness.** Forge shuffles with `Collections.shuffle(list, MyRandom.getRandom())` (`Zone.shuffle`, `Player.java`), and `MyRandom.setRandom(Random)` accepts any `java.util.Random`. See section 5.3 for how a scripted `Random` lets Forge follow our shuffle outcomes. Random discard (Hymn, Gamble) and random-order bottoming likely use the same generator **(verify by grep when building the adapter)**.
2. **Legal-action enumeration quality.** `getAllPossibleAbilities` returns abilities, not complete macro actions with targets and payments; the adapter must expand each into legal target and cost choices. Expect a gap between what Forge's UI offers a human and what its engine accepts; the adapter records which one it uses. **(verify at runtime)**
3. **Decision granularity.** Forge raises its own sequence of prompts for a cast while our core raises its own (doc 02 section 2.2). The harness avoids comparing at sub-decision level by working in **macro actions** (section 3.2); both adapters translate between macros and native sub-decision sequences.
4. **Mana payment.** `payManaCost` and `applyManaToCost` are controller-level, so the adapter implements "auto" by delegating to Forge's AI payment utilities and explicit payment by tapping named sources. **(verify)**
5. **State injection gaps.** Whether `putonstack` can express every stack object we need (Daze with an alternative cost already paid, abilities from triggered sources) is untested. Scenarios Forge cannot express are Forge-excluded and covered by rank-2 expectations only.

### 2.3 XMage adapter (optional, phase D4)

Same trait. Lower priority: its value is as a third opinion on Forge-vs-us disagreements. XMage is a different codebase with a different set of bugs, so agreement between XMage and us against Forge is strong evidence (still checked against the CR). Cost is another adapter and a second JVM fleet, so it is deferred until D1-D3 are working and the mismatch backlog shows real Forge-vs-us ambiguity.

### 2.4 Licensing note

Forge's repository `LICENSE` is GPL-3.0 (checked in the clone) and XMage's `LICENSE` is MIT (checked on its default branch). The harness runs them as separate processes and exchanges data only as JSON. We MUST NOT copy Forge card scripts, card logic, or code into the engine or the card DB. Oracle text and rulings come from MTGJSON/Scryfall (doc 03 section 2), not from Forge's files.

---

## 3. Canonical formats

Everything is compared in a **canonical action language (CAL)** and **canonical state (CS)** that are engine-neutral.

### 3.1 Object identity

- **Card handle:** `seat:Name#ordinal`, where ordinal is the 1-based index among copies of that name in that seat's decklist (`p0:Island#3`). Both engines are given the same decklists in the same order; each adapter maps handle <-> native id. These are test-harness handles, unrelated to engine `CardId`/`ViewId`.
- **Tokens and copies:** have no setup-time handle. They get a **creation key**: `(creating event index, source handle, ordinal within the event)`. Where engines emit simultaneous creations in different orders, objects with identical characteristics are interchangeable (next).
- **Interchangeability:** two objects are interchangeable when they have the same name, controller, zone, and state. The comparator never fails on which of two interchangeable objects was chosen; it compares the multiset of **object signatures** (section 3.3). This resolves "which Island got tapped".

### 3.2 Canonical actions (macro level)

Examples (JSON):

```json
{ "t": "pass" }
{ "t": "play_land", "card": "p0:Wasteland#1" }
{ "t": "cast", "card": "p0:Brainstorm#1", "face": "front", "alt_cost": null, "x": null,
  "modes": [], "targets": [], "pay": "auto" }
{ "t": "cast", "card": "p1:Daze#1", "alt_cost": "return_island:p1:Island#2", "targets": [{"t":"stack_obj","key":"cast:42"}], "pay": "auto" }
{ "t": "activate", "source": "p0:Polluted Delta#1", "ability": 0, "targets": [], "pay": "auto" }
{ "t": "choose", "decision_kind": "discard", "cards": ["p0:Force of Will#1"] }
{ "t": "yes_no", "value": true }
{ "t": "order_triggers", "order": ["trigger:7","trigger:8"] }
{ "t": "declare_attackers", "attacks": [{"creature": "p0:Barrowgoyf#1", "target": {"t":"player","seat":1}}] }
{ "t": "declare_blockers", "blocks": [{"blocker": "p1:Phelia, Exuberant Shepherd#1", "attacker": "p0:Barrowgoyf#1"}] }
```

- `pay: "auto"` is the default and means "engine-chosen payment". Equivalent payments produce identical canonical state after interchangeability normalization. `pay: {explicit: [...]}` is used by targeted scenarios that depend on which mana source is spent (for example, Wasteland-vulnerable lands). Adapters that cannot honor explicit payment mark the scenario Forge-excluded.
- Our engine builds each macro action by walking its sub-decision tree on **forks** of the state and collecting legal complete sequences; this uses the cheap-fork property (doc 01 section 4.3) and the fact that every enumerated option is pre-validated as completable (doc 01 section 6.3). Macro enumeration is a harness feature (`mtg-debug`), not an agent-facing API.
- At each decision, each engine yields a **legal set** in CAL. The sets are compared (section 5.2).

**Shortcuts.** The CAL has no shortcut action. Our engine's loop shortcuts (doc 02 section 9.6) are always **expanded to their stepwise actions** before they reach an adapter, because Forge has no such feature. The comparison therefore sees the same stream whether or not a shortcut was used, and invariant I18 separately checks that the shortcut and its expansion agree.

### 3.3 Canonical state

Fields (T1 = must match, T2 = compared but reported as warning, T3 = ignored):

| Field group | Tier | Notes |
|---|---|---|
| Turn number, active player, phase/step, priority holder | T1 | |
| Life, poison, other player counters | T1 | |
| Hand contents (multiset of names) per seat | T1 | |
| Library: size | T1 | |
| Library: order (when scripted) | T2 until follower mode proves reliable, then T1 | See section 5.3 |
| Graveyard: ordered list of names | T1 for membership, T2 for order | Order rarely matters |
| Exile: contents, face state, "exiled with" relations | T1 | |
| Battlefield: multiset of object signatures | T1 | Signature = (name, controller, tapped, phased, summoning-sick flag, damage, counters, derived power/toughness, derived types/colors, set of ability/keyword names, attachments as signatures-of-neighbors) |
| Attachments and "exiled with"-style links | T1 | Compared via signature-neighborhood hashing, section 3.4 |
| Stack: ordered list of (source signature, kind, controller, targets by signature, modes, X) | T1 | |
| Pending triggers not yet put on stack | T1 | |
| Mana pool contents | T1 | |
| Per-turn counters (lands played, spells cast, storm count) | T1 | |
| Combat: attackers, blockers, damage assignments | T1 | |
| Delayed triggers and continuous effects (as derived effect summaries) | T2 | Compared by their *observable effects* (derived characteristics), not by internal representation |
| Internal ids, timestamps (absolute), flags with no rules meaning, log text | T3 | Relative timestamp ordering is compared only where the CR makes it matter (T2) |

### 3.4 Canonicalization algorithm

Object graphs with attachments and "exiled with" links are canonicalized by **neighborhood refinement** (Weisfeiler-Leman-style hashing): start with each object's local signature, iteratively mix in sorted neighbor hashes for a few rounds, then compare the sorted multiset of final hashes. Mismatch reports show the hash classes that differ and print the objects in them.

### 3.5 Event stream

Where available, adapters emit `CanonicalEvent`s (draw, zone change with cause, damage, life change, trigger put on stack, spell cast, spell countered, counters changed). Comparing event sequences localizes divergence even if states reconverge, and it surfaces ordering bugs that state comparison hides. For Forge, events come from its game event bus where available, else are derived from state diffs **(verify)**. Event comparison is T2 initially.

---

## 4. Rulings-as-spec

### 4.1 Pipeline

```text
CR section + Gatherer/Scryfall ruling text  --(spec-writer agent)-->  scenario YAML + expectation
                                                   |
                                         (spec-reviewer agent: adversarial check, "does the scenario faithfully encode the ruling?")
                                                   |
                                  run on OUR engine (must pass)  AND  on Forge (disagreement -> flagged, never auto-fixed)
```

For each card in the pool, the spec-writer agent produces `tests/spec/cards/<card>.yaml` with one or more scenarios per ruling and per relevant CR case. A ruling that cannot be turned into a scenario (for example, pure UI advice) is recorded as `not_testable` with a reason. Each scenario has a `source` block (ruling date and text, or CR section) so reviewers can check fidelity.

### 4.2 Scenario format

```yaml
id: daze-return-island-pay-one
cards: [Daze, Brainstorm, Island]
source:
  - kind: oracle
    text: "Counter target spell unless its controller pays {1}."
  - kind: cr
    ref: "alternative costs (CR 118.9 and 601.2; exact sections to be confirmed against the pinned CR)"
setup:
  turn: 3
  active: 0
  phase: main1
  p0:
    life: 20
    hand: [Brainstorm]
    battlefield: [{card: Island, count: 2, tapped: [false, false]}]   # one Island pays for Brainstorm, one is left for Daze's tax
    library: [Island, Island, Island, Island, Island, Island]         # explicit order, top first
  p1:
    life: 20
    hand: [Daze]
    battlefield: [{card: Island, count: 1, tapped: [false]}]
script:
  - {actor: 0, action: {t: cast, card: "p0:Brainstorm#1", pay: auto}}
  - {actor: 0, action: {t: pass}}                                    # p0 passes priority with Brainstorm on the stack
  - {actor: 1, action: {t: cast, card: "p1:Daze#1", alt_cost: "return_island:p1:Island#1", targets: [{t: stack_obj, ref: brainstorm}], pay: auto}}
  - {actor: 0, action: {t: pass}}
  - {actor: 1, action: {t: pass}}                                    # Daze resolves
  - {actor: 0, decision: pay_for_counter_unless, answer: {t: yes_no, value: true}}   # p0 pays {1} with the remaining Island
  - {actor: 0, action: {t: pass}}
  - {actor: 1, action: {t: pass}}                                    # Brainstorm resolves: p0 draws 3
  - {actor: 0, decision: put_back_from_hand, answer: {t: choose, cards: [any_two]}}   # interchangeable Islands
expect:
  - at: after_script
    p0: {hand_count: 1, library_count: 5, graveyard: [Brainstorm]}
    p1: {hand: [Island], graveyard: [Daze], battlefield: []}
    stack: []
```

(The scenario is illustrative; the spec-writer verifies exact CR/ruling behavior and decision ordering for each real scenario. In the real scenario set, the library would contain distinguishable cards so the put-back choice and resulting order are checked.)


**Other first-class scenario types:**

- `stifle-fetchland`: Stifle the Delta's ability after costs; expect land sacrificed, life paid, no search. Stifle is a main-deck pool card (UWx Control), so this is a pool requirement, not an illustration. Companion scenarios: Stifle on a Sand Scout or Phelia trigger, Consign to Memory on a triggered ability with replicate copies, Stifle on ward (Koma), and a check that Stifle cannot target a mana ability.
- `aluren-acererak-loop`: Aluren plus Acererak, venturing into Lost Mine of Phandelver or Dungeon of the Mad Mage each time so Acererak keeps returning to hand; check the room triggers, the choose-a-dungeon decision on every fresh venture, Stifle or Consign on the ETB trigger and on a room trigger, and that the loop is bounded by the action budget rather than the state-hash counter. Variants with Disruptor Flute naming Acererak and with Defense Grid on the opponent's turn: the free cast still pays the cost increase (CR 118.9d, 601.2f), so the loop stops, and the line that completes Tomb of Annihilation (Acererak stays, The Atropal 4/4 is created) is checked step by step.
- *(Optional, only if loop shortcuts are built; doc 02 section 9.6.)* `shortcut-aluren-acererak`: Aluren plus Acererak with the loop detector: two stepwise iterations, then `ProposeShortcut` with N; the shortcut stops cleanly when the game ends mid-loop, at N, and when Tomb of Annihilation completes (Acererak no longer bounces, so the next `ActionRef` has no match).
- `shortcut-taxed`: Disruptor Flute naming Acererak and Defense Grid on the opponent's turn: the detector does not flag a loop because mana changes the board signature; a hand-submitted proposal stops at the first iteration whose payment step has no matching option; the free cast still pays the tax (CR 118.9d, 601.2f).
- `shortcut-opponent-window`: the opponent holds Stifle or Consign to Memory (response windows on Acererak's ETB trigger and on room triggers): they get `ShortcutResponse`; `StopAfter(n)` stops after exactly n iterations; with no response available there is no prompt and the proposer's event stream is byte-identical to the prompted case (non-interference).
- `shortcut-new-window`: a window that did not exist at the same step of iteration one (the opponent gains a castable response mid-loop) stops the shortcut and gives them priority.
- `shortcut-hidden-info`: a body containing the proposer's own draw or scry; later steps that need a different answer stop the shortcut at that decision; the per-step knowledge updates arrive in `ShortcutSummary` and the `SeatLog`.
- `venture-dungeons`: all three dungeons, every room and arrow; completing a dungeon (venture from the bottom room, or the state-based action once no room ability is on the stack, CR 704.5t, 309.6, 309.7); Acererak stops bouncing only once Tomb of Annihilation is completed; Tomb's opponent choices (discard or lose life, sacrifice or lose life) as the edge case.
- `ring-tempts`: Samwise's four Ring levels across repeated temptations, including the Ring-bearer designation moving to a new creature.
- `aluren-free-cast`: both players casting small creatures for free at instant speed, Lavinia countering a no-mana cast, Aluren plus Containment Priest or Grafdigger's Cage, and Aluren not combining with another alternative cost.
- `kaito-creature-turn`: Kaito as a creature only on his controller's turn, ninjutsu with an unblocked attacker, stun counters, hexproof only on his controller's turn; checks layers 4, 6, 7b.
- `reanimation-replacements`: Containment Priest, Grafdigger's Cage and Spider-Woman against Reanimate, Animate Dead, Shallow Grave, Show and Tell, Phelia and Hide on the Ceiling returns.
- `secret-simultaneous`: Show and Tell and Stronghold Gambit with hidden choices; the second chooser's observation must carry no information about the first pick.
- `magus-lands`: Magus of the Moon with surveil lands, fetchlands, Wasteland and MDFC lands (layer 4 plus the abilities they lose).
- `replacement-ordering`: two replacement effects on the same event; expect the affected player gets an ordering decision.
- `apnap-triggers`: both players have simultaneous triggers; verify APNAP stacking and per-player ordering decisions.
- `layers`: timestamps and (when in scope) dependencies.
- `hidden-info`: scenarios flagged `observer: p0` that assert what `p0` can and cannot see (feeds section 7).

### 4.3 Expectation derivation

Expectations MUST be derived from the ruling/CR text, **not** from running either engine. A scenario whose expectation was copy-pasted from engine output is rejected in review (the reviewer agent is told to look for this; the scenario file records `derived_from: ruling|cr|forge_observed`, and `forge_observed` scenarios are treated as differential cases, not as spec).

### 4.4 Separation, holdout, and anti-overfitting

- **Different agents write tests and code.** The spec-writer cannot modify engine or card files; the implementer cannot modify `tests/spec/**`, `tests/holdout/**`, or `known-divergences/**` (doc 01 section 14.4). Enforced by repo path permissions and a CI check that fails when one author touches both a card and its spec file.
- **Visibility.** The implementer sees the failing scenario's ID, setup, script, and expectation (needed to fix it). The implementer does not see scenarios in the **holdout set**.
- **Holdout set.** About 30% of scenarios per card (randomly chosen by a script using a seed the implementer does not know) are held out. CI reports only pass/fail counts per card for the holdout set; the first failure of a holdout scenario reveals its title to the spec-writer and a triage agent, not the implementer. After a card is fixed, revealed holdout scenarios are promoted to the visible set and replaced by new ones from the spec-writer.
- **Generalization checks.** Even when every scenario passes, the lockstep differential (section 5) and invariant fuzz (section 7) run on random states the implementer never saw.
- **Metamorphic tests.** For rules where it applies, assert invariances: renaming interchangeable objects, swapping seats, shuffling hand order, and permuting simultaneous-trigger input order must not change canonical outcomes (apart from decisions offered).

---

## 5. Lockstep random differential testing

### 5.1 Setup

For each run:

1. **Scripted opening.** Decks, library order, and opening hands are injected into both engines. No real shuffles occur before the first decision. Mulligans and the first-player coin flip have their own scenario tests rather than lockstep.
2. **Common randomness.** All remaining randomness (shuffles, coin flips, random discard) is handled by the **randomness protocol** (section 5.3).
3. **A policy chooses actions.** A seeded policy picks one canonical action at each decision from the **intersection** of both legal sets. Policies: uniform random; weighted-random (prefers casting spells and attacking over passing, so games develop); matchup-scripted (follow a fixed plan for the first N turns to reach a target interaction); coverage-guided (section 5.4).
4. **After every applied action**, the comparator snapshots both engines and diffs canonical states (T1 fails the run, T2 logs a warning). Events are compared when available.
5. **On first T1 mismatch**, the run stops (to avoid cascading noise) and the mismatch is recorded with the full action history, both snapshots, and the diff.

### 5.2 Action-set differential

Legal sets themselves are compared at every decision:

- `ours \ forge` non-empty: we may be offering an illegal action. High-priority finding.
- `forge \ ours` non-empty: we may be missing a legal action (also bad: it hides states from search and self-play). High-priority finding.
- Differences caused by known Forge limitations (for example, Forge's enumerator not offering actions in a certain window) go through the same triage and known-divergence process.

Because sets are compared before applying anything, a bug in the enumerator is caught even if no random policy would have chosen the discrepant action. This directly serves the brief's "illegal moves must be impossible" requirement.

### 5.3 Randomness protocol (leader/follower)

Neither engine's RNG algorithm will match the other's, so randomness is treated as an **input channel**:

- **Leader**: for every random event (shuffle result, coin flip, random choice) the leader engine produces an outcome, which is serialized as a `RandomOutcome` (for a shuffle, the resulting permutation of library handles).
- **Follower**: the other engine is told the outcome and applies it instead of drawing its own. For a library shuffle there are two ways to make Forge follow. (a) **Scripted `Random`:** Forge's shuffle is `Collections.shuffle(cardList, MyRandom.getRandom())` and `MyRandom.setRandom(Random)` takes any `java.util.Random`, so the adapter installs a `Random` whose `nextInt` sequence is computed (by inverting the shuffle algorithm) to produce exactly the permutation the leader chose; this needs no mid-game library surgery. (b) **Re-stack the library** through state injection. Prefer (a) and fall back to (b). Both need runtime verification.
- Our engine can both lead and follow by construction (RNG output is an explicit input; doc 01 section 12).
- If an adapter supports neither, the run is marked "no shuffle effects": generators avoid scenarios with shuffle (fetch-heavy decks lose coverage; a fallback is to compare library as a multiset after the shuffle, and then force top-of-library re-stack through the setup channel).

### 5.4 Interaction-targeted and coverage-guided generation

Random play from the opening is too slow at reaching rare interactions. Generators:

- **Pair scenarios.** For every pair (A, B) of cards across the two decks of a matchup, generate states in which A and B are in the relevant zones (A on battlefield or stack; B in hand with mana) and let random play proceed. This guarantees pairwise interaction coverage.
- **Stack-depth scenarios.** Several instants in hand on both sides, with open mana, so that responses nest (Force of Will on a Daze on a Brainstorm).
- **Coverage-guided fuzzing.** Our engine records coverage tuples: (card, ability index), (event kind, card pair that interacted), (replacement effects applied), (layer used), (decision kind). The generator keeps a corpus of seeds that reached new tuples and mutates them (change a hand, swap a card, alter a policy seed). This is cheap because instrumentation is on our side; it needs nothing from Forge.
- **Corner scenarios by rule family:** simultaneous triggers (APNAP), replacement ordering, mana pool across steps, split second-style restrictions (if pool), lethal damage and SBA ordering, legend rule, combat with first strike, copy effects, zone-change identity (target leaves and returns).

### 5.5 Expected throughput and cost

Throughput is bounded by Forge (JVM warm, plus canonicalization). The Phase 1 runner measurement of Forge games/sec gives the baseline; lockstep steps/sec will be much lower than raw playout because of per-step snapshots and IPC. Sizing formula for planning:

```text
wall_clock_hours = (gate_steps_required) / (steps_per_sec_per_jvm * n_jvms * 3600)
```

Gate sizes (section 8) are proposals; revise once steps/sec is measured. This is CPU work with no LLM in the loop, so the marginal cost is machine time.

---

## 6. Triage

### 6.1 Mismatch record

`{id, signature, kind, matchup, seed, action_history, canonical_states (ours, forge), diff, events, shrunk_repro?, status, verdict, owner}`.

**Kinds:** `STATE_MISMATCH`, `ACTION_SET_MISMATCH`, `EVENT_MISMATCH`, `CRASH_OURS`, `CRASH_ORACLE`, `TIMEOUT`, `ILLEGAL_ACCEPTED` (our engine accepted an action outside its mask in the probe test).

**Signature** (for deduplication): `(kind, involved card names sorted, first differing field group, rules-feature tags)`. Hundreds of thousands of games can produce thousands of mismatches that reduce to a few dozen root causes; triage works on signatures, not on individual games.

### 6.2 Shrinking

Both engines are deterministic given the randomness protocol, so a mismatch can be **delta-debugged**:

1. Remove actions from the history while the mismatch persists.
2. Remove cards from the setup (hands, boards, libraries) while it persists.
3. Simplify values (life, counters, X).

Our side is cheap (fork/replay), so the bottleneck is Forge replays. A long-lived JVM with a fast `reset(setup)` is a requirement of the Forge adapter.

### 6.3 Verdict taxonomy

| Verdict | Meaning | Who may assign | Next step |
|---|---|---|---|
| `OUR_BUG` | We deviate from the rules | Triage agent, confirmed by rules reviewer when core is affected | Fix by implementer; add scenario authored by spec-writer |
| `FORGE_BUG` | Forge deviates from the rules | Triage agent proposes; human approves | Add `known-divergences` entry with CR citation and our-side regression scenario; optionally report upstream |
| `VERSION_DIVERGENCE` | Forge implements an older/newer rules version | Triage agent proposes; human approves | `known-divergences` entry (section 9) |
| `SPEC_AMBIGUITY` | CR/ruling does not decide; judgment needed | Human | Decision recorded in the spec |
| `HARNESS_BUG` | Canonicalization, adapter, or driver error | Triage agent | Fix harness |
| `FLAKE` | Does not reproduce | Triage agent (after N replays) | Investigate nondeterminism; flakes in our engine are bugs |

### 6.4 Triage agent workflow

For each new signature:

1. Reproduce and shrink.
2. Retrieve relevant CR sections and card rulings for the cards involved (retrieval from pinned CR text and ruling database).
3. Reason about which side is right. If XMage is available, run it as a third opinion.
4. Emit a verdict with citations. `FORGE_BUG`, `VERSION_DIVERGENCE`, and `SPEC_AMBIGUITY` verdicts require **human approval** before becoming accepted divergences, because that is exactly where an agent could rationalize away a real bug in our engine.
5. The triage agent never edits engine, card, or spec files; it files tasks.

LLM cost is per **unique signature**, not per game, so it stays small even with massive game volume.

### 6.5 Gate impact

Any open `OUR_BUG`, `ACTION_SET_MISMATCH`, or unresolved `SPEC_AMBIGUITY` involving a card in the deck blocks the deck gate.

---

## 7. Invariants, non-interference, and self-play detection (engine-only; no oracle needed)

### 7.1 Invariant fuzzing

Checked after every step in `audit` builds, and sampled in release fuzz. Each is a function `fn check(&State) -> Result<(), Violation>` in `mtg-debug`.

| ID | Invariant |
|---|---|
| I1 | Card conservation: every physical card is in exactly one zone; total equals decklist size plus live tokens and spell/ability objects |
| I2 | Zone/object consistency: each zone's index lists agree with each object's recorded zone; no duplicate refs |
| I3 | Legal zone transitions only (as per a transition table; for example, no library -> stack except via cast-from-library effects) |
| I4 | No dangling refs: attachments, targets of stack objects, "exiled with" links point to live or explicitly LKI-marked objects |
| I5 | SBA stability: whenever a player receives priority, running the SBA check finds nothing to do |
| I6 | Stack/priority: priority holder exists whenever a decision is pending; stack objects have valid controllers and legal targets at the time of casting |
| I7 | Mana pool is empty at step/phase boundaries |
| I8 | Counters nonnegative; +1/+1 and -1/-1 counters never coexist after SBA |
| I9 | Life and poison change only via events (log reconciliation: replaying the event log reproduces life totals) |
| I10 | Determinism: replaying the `GameRecord` yields identical state hashes at every checkpoint |
| I11 | Derived-cache coherence: the incrementally maintained derived characteristics equal a full recompute (doc 01 section 10.3) |
| I12 | Fork consistency: `clone -> apply same actions` yields the same hash trajectory as the original |
| I13 | **Mask validity:** every enumerated option executes without error and leads to a state satisfying I1-I12; additionally, a **probe test** applies random indices outside the mask and random stale `DecisionId`s and asserts rejection with no state change |
| I14 | Frame stack empty at decision points where only a priority decision should be pending; empty at game end |
| I15 | No panic, no unbounded loop (action budget and repeated-hash detection, doc 01 section 5.5) |
| I16 | Audit-vs-release parity: audit and release builds produce identical hash trajectories on the same records |
| I17 | Zobrist incremental hash equals `hash_full` |
| I18 | **Shortcut equals stepwise** (optional; applies only if loop shortcuts, doc 02 section 9.6, are built) (audit builds): every executed loop shortcut (doc 02 section 9.6) is also replayed stepwise on a clone, and the state hashes and event streams must match at every stop point |

Corpus: generator seeds, coverage-guided (section 5.4), plus a fixed regression corpus of past failures. Gate: 10^7 steps per deck pair with no violation (proposal).

### 7.2 Non-interference (hidden-information) test

Formalizes doc 02 section 1's guarantee. Procedure:

1. Generate a mid-game state S for observer seat A (random play to a random depth).
2. Build S' by re-randomizing everything A is not entitled to know: opponent hand identities (preserving known/revealed cards and counts), both libraries' unknown portions, the RNG state. Ensure S' is consistent with A's knowledge (as per `Knowledge`).
3. Drive both games with an identical **observer-A action policy** and an **opponent policy that picks only actions legal in both worlds** (for example, pass, play lands present in both, cast cards present in both) so that the two games remain comparable.
4. After each step, assert A's observation streams are byte-identical: `Observation`, `Decision.options` (including order), `ApplyError`s, `ViewId`s, `view_hash`, `ViewEvent` text.
5. Fork test: for several S and S' pairs, assert `fork(A, seed, model)` built from S and from S' yields **identical** forks (doc 02 section 5.3).
6. Targeted hidden-info scenarios from section 4.2 (`hidden-info` type) assert specific entitlements: opponent sees only a hand count after a draw; library order stays hidden after Brainstorm; bounced cards remain known; Show and Tell's second chooser sees nothing of the first choice.
7. **Canary mutation test:** the test suite is itself tested by intentionally injecting known leaks (a `ViewId` derived from `CardId`; an option list in arena order; a hash over `State`; an error message mentioning a card) into a mutant build and asserting the non-interference test **fails**. If it does not, the test is too weak.

CI: runs on every PR; any new card or event kind adds scenarios.

### 7.3 Self-play as bug detector

The engine is the training environment; RL agents are good at finding rules bugs. Detectors run over batches of self-play games:

| Detector | Flag |
|---|---|
| Turn-of-win distribution | Wins on turns far earlier than the deck's known speed (for a non-combo deck, turn 2 wins are suspicious) |
| Action-pattern outliers | Actions repeated in loops; spells cast with no effect; same card cast thousands of times; resource totals (mana, life, cards) outside plausible bounds |
| Card-level win-rate outliers | A single card's presence moves win rate far more than comparable cards (z-score on a per-card win delta) |
| Illegal-looking states | Any I1-I17 violation in a training game (audit sampling), any `TIMEOUT` or truncation |
| Policy-vs-baseline gap | A trained policy's win rate dramatically exceeds heuristic/Forge-AI baselines in a particular matchup |
| Replay divergence | Re-running the `GodRecord` on a fresh engine instance differs |

Flagged games go to a **rules-review queue**: replayed with an annotated event log, reviewed by a rules-reviewer agent with CR/ruling retrieval, then by a human for confirmed or disputed findings. This feeds back into new scenarios (spec-writer) and, if the bug is real, into `OUR_BUG` triage. Detectors run continuously during Phase 3 training, not only at the gate.

---

## 8. Deck gate

A deck is **done** only when the checklist below passes and sign-off is recorded. Numeric thresholds are proposals to be tuned once throughput is measured.

1. **Implementation:** every card in the 75 (including sideboard cards that can enter the game) has `status >= DiffTested` (doc 03 section 10) and no `Draft` cards; custom ops reviewed.
2. **Rulings suite:** 100% of visible and holdout scenarios for the deck's cards pass. Every Gatherer ruling is covered or marked `not_testable` with reason.
3. **Lockstep differential:** for each ordered matchup of the deck against every other gated or candidate deck, at least N_g games (proposal: 10,000) and at least M steps total (proposal: 10^6) with zero unresolved T1 mismatches; zero unresolved action-set mismatches.
4. **Interaction coverage:** for each deck pair, >= 95% of (card in A, card in B) pairs exercised in lockstep at least k times (proposal: 20), via the pair-scenario generator and coverage tracking; the remainder explained (for example, no possible interaction).
5. **Invariant fuzz:** >= 10^7 steps involving the deck with zero violations; non-interference test and canary mutation test green.
6. **Known divergences:** every entry touching the deck's cards has CR citation, our-side regression scenario, and human approval (section 9).
7. **Performance:** benchmark non-regression vs the last gated build (doc 01 section 13).
8. **Human review:** a named human (Brady or delegate) reviews (a) all accepted divergences, (b) a sample of full game replays with annotated logs (proposal: 20 per deck), and (c) a randomly chosen set of 10 card implementations against the Oracle text.
9. **Record:** `decks/<deck>/GATE.md` with commit hashes of engine, card DB, spec, holdout, and harness; test counts; sign-off identities and date.

**Invalidation.** The gate is tied to hashes of: core version, every card file in the deck, every card in common matchup decks that interact (derived from the interaction graph), and the pinned CR version. If any changes, the affected decks re-run the relevant portions automatically; CI shows "gate stale".

**Use in training.** Phase 3 trains and evaluates only on gated decks. A `provisional` mode exists (decks not gated), with a loud flag in every training record, for experimentation. Models trained on provisional decks are not benchmarked as results.

---

## 9. Known-divergence registry

`known-divergences/<id>.toml`, schema:

```toml
id = "kd-0001"
title = "Combat damage assignment order"
status = "proposed"            # proposed | accepted | forge-bug-reported | retired | scope-cut
who_is_right = "engine"        # engine | oracle | both-acceptable
cr_citation = "CR 510.1c (effective 2025-11-14): a creature blocked by two or more creatures divides its combat damage as its controller chooses; there is no damage assignment order"
rulings = ["<ruling id>"]
affected_oracles = ["forge"]
match = { kinds = ["STATE_MISMATCH"], cards = [], feature_tags = ["combat-damage-assignment"] }
suppress = true                # the comparator downgrades matching diffs to a counted note
regression_scenario = "tests/spec/combat/damage-assignment-multiblock.yaml"
approved_by = "<human>"
notes = "Forge (fd5c996) still orders blockers (PlayerController.orderBlockers, Combat.orderBlockersForDamageAssignment). Lockstep policy restricts damage assignments to those legal under both rule sets, so only the action-set difference is suppressed. Awaiting human approval."
```

Rules:

- Suppression is by **pattern match on the diff**, not by blanket disable; the suppressed count is reported so a pattern that starts matching too much is visible.
- Every entry MUST have a regression scenario asserting **our** behavior against the rules text (rank 1-2). A divergence is not "accepted" until our side is checked independently.
- A `status = "scope-cut"` entry records a **documented scope cut**: a legal action or rule our engine deliberately does not implement (doc 05 section 8). The comparator downgrades the resulting action-set mismatch to a counted note, and the entry states the revisit condition.
- Entries are reviewed every release and on every CR version bump; retired when Forge catches up.
- Only humans (or an agent with explicit human delegation for that entry) change `status` to `accepted`. Implementer agents have no write access (doc 01 section 14.4).

---

## 10. Infrastructure and operations

- **Compute:** CPU only. Lockstep: N JVMs (each long-lived, pre-warmed with Forge's card database loaded) plus our in-process engine. Invariant fuzz and self-play detectors: our engine only, highly parallel across games.
- **Storage:** mismatch records, shrunk repros, coverage corpora, `GodRecord`s of flagged games, all content-addressed and hash-stamped with engine version, card DB hash, harness version.
- **Reproducibility:** every record can be replayed with one command (`mtg-diff replay <id>`), producing the same mismatch.
- **Dashboards:** per deck: gate checklist status; per card: status, scenario counts, mismatch counts, coverage; global: open triage by verdict, lockstep steps/sec, flake rate, divergences count and age.
- **LLM usage:** spec-writer (once per card plus on Oracle change), spec-reviewer (once per scenario batch), triage (once per new signature), rules-review (per flagged game). All off the hot path; cost is proportional to cards and signatures, not games.

---

## 11. Phasing

| Phase | Deliverable | Depends on |
|---|---|---|
| D0 | CAL and canonical state formats, our-side `snapshot_canonical`, scenario YAML parser and runner against our engine (no oracle yet) | Core M1-M2 |
| D1 | Forge adapter: `reset`, `legal_actions`, `apply`, `snapshot` for scripted scenarios; run the rulings suite on both; first mismatch triage | Phase 1 Forge headless + enumerator; D0 |
| D2 | Lockstep random generator with randomness protocol; action-set differential; mismatch records; signature dedupe | D1 |
| D3 | Shrinker; triage agent workflow; known-divergence registry; dashboards | D2 |
| D4 | XMage adapter (optional) | D3; evidence it is needed |
| D5 | Coverage-guided and pair-scenario generators; interaction-coverage gate metric | D2 |
| D6 | Invariant fuzz at scale; non-interference test with canary mutations; self-play anomaly detectors | Core M3 for forks; training environment |

D0, invariant fuzz v0, and the non-interference test do not need Forge and can start as soon as the core skeleton exists.

---

## 12. Risks

| Risk | Mitigation |
|---|---|
| Forge cannot be driven at the granularity or with the state control the harness needs | Find out in Phase 1 (adapter feasibility is largely the same work as the Forge enumerator and serializer). Fallback: scenario tests only (smaller initial states), with lockstep random play restricted to what Forge can set up |
| Forge's bugs and rules-version lag drown triage | Signature dedupe; known-divergence registry; XMage tie-breaker; triage budget measured in signatures |
| Canonicalization hides real bugs (too lenient) or invents false ones (too strict) | Harness has its own test suite using **intentionally mutated engines** (inject known bugs; assert detection); T1/T2 tiers reviewed with each false positive/negative |
| Both engines wrong the same way | Rank-2 scenarios from rulings; human spot checks; invariant and self-play detectors |
| Tests overfit to implementation | Role separation, holdout set, metamorphic tests, expectation derivation rules (section 4.3) |
| Random play does not reach interesting states | Pair scenarios, coverage guidance, matchup scripts; measure interaction coverage as a gate metric |
| Agent-authored scenarios encode misread rulings | Reviewer agent plus differential cross-check with Forge (disagreement flagged) plus human review of divergences |
| Flaky infra (JVM crashes, IPC timeouts) masquerades as mismatches | `CRASH_ORACLE` and `TIMEOUT` kinds separate from state mismatches; automatic retry on fresh process; flake tracking |

---

## 13. Open questions

1. Is XMage worth the cost now, or only once Forge-vs-us ambiguity is measured?
2. Who is the human rules reviewer for accepted divergences and gate sign-off (Brady, or a delegate with deep Legacy rules knowledge)?
3. Gate sizing: are 10,000 games and 10^6 steps per ordered matchup, and 10^7 fuzz steps per deck, acceptable once Forge steps/sec is known?
4. Should the post-mortem LLM see `GodRecord` after games (doc 02 section 8)?
5. Which CR version to pin (affects the first known-divergence entries, especially combat damage assignment).
