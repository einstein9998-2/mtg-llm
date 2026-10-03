# MTG Engine — Core Design (Phase 2)

Status: DRAFT v0.1 for review. Docs only; no engine code exists yet.
Part of: `engine-design/` (see `README.md`). Companion docs: `02-action-api-and-hidden-info.md`, `03-card-dsl.md`, `04-differential-testing-spec.md`, `05-pool-mechanics-inventory.md` (pool-specific scope).

Conventions used in all docs:

- "MUST / SHOULD / MAY" are used in the RFC sense. MUST items are the review checklist for the frozen core.
- The engine language is Rust (confirmed by Brady).
- Code blocks are illustrative sketches to pin down shape and invariants, not normative source.
- Numeric performance figures are **targets to be validated by the M0 spike** (section 15), not measurements.
- Rules references cite the Comprehensive Rules (CR) by topic, with section numbers where I am confident of them. The CR version MUST be pinned at implementation start (section 16); section lettering shifts between versions, so the pinned version's text wins over anything cited here.

---

## 1. Goals, non-goals, requirements

### 1.1 Why a custom engine

From the project brief: Forge is the Phase 1 engine. Phase 2 exists because Forge's throughput, state cloning, and hidden-information story are expected to limit (a) the `simulate` tool for the LLM player and (b) self-play volume for a policy/value net with search. The custom engine buys:

1. Speed: target 100x or more over Forge's measured games/sec on the same matchup (the Phase 1 runner provides the baseline).
2. Cheap state cloning (microseconds), which makes `simulate`, determinized search, and differential-test shrinking cheap.
3. Hidden-information guarantees by construction (doc 02).
4. A clean masked action API (doc 02).
5. Determinism: a game is a pure function of (card DB version, decks, seed, action list).

### 1.2 Non-goals

- Generalizing to unseen cards. The engine supports a closed pool (roughly 200-300 cards). Every rules feature outside the pool's needs is deliberately unimplemented and MUST fail loudly (a compile-time or load-time error naming the missing feature), never silently approximate.
- A UI, networking, or Arena/MTGO compatibility.
- Multiplayer, Commander, Planechase, Conspiracy, Un-sets, subgames, ante, day/night, sagas, battles, and other mechanics no deck in the pool uses. These are added only if a chosen deck needs them. (Dungeons are **in** scope: Acererak in the Alurentell deck needs Tomb of Annihilation, see doc 05 section 2.3.)
- Sideboarding and match flow inside the rules core. Those live in a thin match layer (section 14.4) because they are low-frequency decisions that the brief wants handled separately.

### 1.3 Requirements that shape the design

| # | Requirement | Consequence |
|---|---|---|
| R1 | Illegal moves are impossible | The engine only ever offers legal actions; applying anything else is rejected. Legality is decided in one place (the enumerator) and re-checked on apply. |
| R2 | Hidden info enforced by construction | Agents never touch `State`. See doc 02. |
| R3 | State clone is cheap | `State` is plain data: indices, no pointers, no trait objects, no `Rc`. |
| R4 | Determinism | No hash-order dependence, no floats in rules, no wall-clock, seeded RNG that lives inside `State`. |
| R5 | Resumable | The engine pauses at every decision and resumes later, so the whole in-progress resolution must live in `State` as data (section 6.2), not on the host call stack. |
| R6 | Verifiable | Every state transition emits typed events; every card has provenance; invariants are machine-checkable (doc 04). |
| R7 | Core can be frozen | The rules core is small, generic, and card-agnostic. Cards never require core changes except through reviewed RFCs (section 14). |
| R8 | Agent-buildable | Exhaustive `match` on enums, strong types for ids and zones, no clever global state. Many agents will touch this; the compiler should catch drift. |

---

## 2. Language choice

**Rust (confirmed by Brady).** Reasons, in order of weight:

1. Exhaustive `match` on `Event`, `Effect`, `Frame`, `DecisionKind` turns "I added a new event and forgot a handler" into a compile error. This is the main defense against core drift when many agents contribute.
2. Crate privacy gives us a real hidden-information boundary (doc 02, section 3): the agent-facing crate cannot name `State` fields.
3. `#[derive(Clone)]` on pure-data structs makes R3 the path of least resistance. The borrow checker pushes toward index-based arenas, which is what we want anyway.
4. `#![forbid(unsafe_code)]` is realistic for the core crate. Perf-sensitive containers come from reviewed dependencies (`smallvec`, `arrayvec`).
5. Good Python bindings (PyO3 + maturin) for the Phase 3 training stack, and straightforward `rayon`-style parallelism across games.

Compile times matter for agent iteration loops once card code grows; keeping cards as data (doc 03) means card additions do not recompile the core.

---

## 3. Architecture overview

### 3.1 Crates (Rust workspace)

| Crate | Contents | Visible to agents? |
|---|---|---|
| `mtg-core` | `State`, zones, turn/priority machine, stack, SBA, events/triggers, replacement pipeline, layers, combat, effect VM, enumerator, RNG | No (internal) |
| `mtg-cards` | Card DB: parsed DSL data compiled to IR/bytecode, plus registered hand-written card functions | No |
| `mtg-view` | `Observation`, `Decision`, `Action`, `Determinizer`, `Fork` handle. The only API agents link against | **Yes** |
| `mtg-py` | PyO3 bindings over `mtg-view` (batched stepping, vectorized observations) | Yes |
| `mtg-match` | Match layer: Bo1/Bo3, mulligans-to-start-of-game wiring, sideboarding hooks, game records | Internal |
| `mtg-debug` | Full-state dumps, canonical snapshot for differential testing, invariant checks. Feature-gated `diff-harness`; MUST NOT be linked by agent builds | No |
| `mtg-diff` | Differential testing harness (doc 04) | No |
| `mtg-fuzz` | Invariant fuzzing and self-play anomaly detectors | No |

### 3.2 The engine as a pure transition function

```text
advance(state: &mut State) -> Status        // runs rules machinery until a decision is needed or the game ends
apply(state: &mut State, d: DecisionId, a: ActionIdx) -> Result<(), ApplyError>
```

- All rules machinery runs inside `advance`. It never blocks, never calls out to an agent, and never reads anything but `State` and the immutable `CardDb`.
- `Status` is `NeedDecision(player)` or `GameOver(result)`.
- Everything the machinery was in the middle of is stored in `state.frames` (section 6.2). There is no hidden host stack.

### 3.3 Immutable vs mutable data

- `CardDb` (immutable, `Arc`-shared across all games and forks): card definitions, compiled ability bytecode, keyword tables, trigger index, version hashes.
- `State` (mutable, `Clone`): everything that changes during a game.
- Per-process singletons: none. No `static mut`, no thread-locals in rules code.

---

## 4. State layout

Sketch, not normative:

```rust
pub struct State {
    // ---- identity / objects ----
    objs: Vec<Obj>,              // slot arena; slots 0..N_CARDS are the physical cards (fixed for the game),
                                 // later slots are tokens, copies, and ability objects on the stack
    free: Vec<u32>,              // free list for non-card slots

    // ---- zones ----
    players: [PlayerState; 2],   // hand, library, graveyard, exile, sideboard, life, mana pool, counters (energy), designations (city's blessing, Ring), dungeon progress, per-turn flags and history
    command: CommandZone,        // dungeon cards with venture markers, emblems (The Ring, Kaito's emblem); small, public
    battlefield: Vec<ObjRef>,    // ordered by timestamp
    stack: Vec<ObjRef>,          // bottom..top

    // ---- turn state ----
    turn: TurnState,             // turn number, active player, phase/step, priority holder, passes, extra turns/combats queue
    combat: Option<Combat>,

    // ---- rules machinery ----
    frames: Vec<Frame>,          // continuation stack (section 6.2)
    pending: Option<PendingDecision>,
    pending_triggers: SmallVec<[PendingTrigger; 8]>,
    delayed: Vec<DelayedTrigger>,
    conts: Vec<ContinuousEffect>,        // duration-bearing effects created by resolved spells/abilities
    repls: Vec<ReplacementEffect>,       // dynamically created replacement/prevention effects (static ones come from permanents)
    derived: DerivedCache,               // layer results; dirty-flagged (section 10)
    hist: TurnHistory,                   // per-turn counters and lists (storm count, spells cast, cards drawn, lands played, ...)

    // ---- information & reproducibility ----
    knowledge: Knowledge,        // who knows which card identities (doc 02, section 6)
    rng: Pcg64,                  // shuffles and random choices only; seeded; part of state so forks fork it
    ts_counter: u32,             // timestamp source
    event_seq: u32,
    zobrist: u64,                // incremental state hash for tests, replay checks, and search
}
```

### 4.1 Objects, identity, and rule 400.7

Rule 400.7: an object that changes zones becomes a new object with no memory of its previous existence. The engine models this directly:

- `CardId(u16)`: the physical card. Fixed for the game. Assigned in **decklist order**, never in library order (leak prevention, doc 02 section 4).
- `ObjRef { slot: u16, gen: u16 }`: reference to an object. A slot's `gen` increments on every zone change. A stale `ObjRef` (gen mismatch) means "that object no longer exists as such". Targets, attachments, "exiled with" links, and delayed-trigger references all use `ObjRef`, so "target became illegal because it left and returned" falls out for free (rule 608.2b, 400.7).
- Last-known information (LKI): when an object leaves a zone, the engine snapshots the characteristics needed by triggers and by "if it was X" checks into the event that caused the move (section 7.3). Events carry LKI by value.
- Tokens, spell copies, and ability objects live in non-card slots and cease to exist (slot freed, gen bumped) when they would leave their legal zone (tokens: rule 704.5 token state-based action; copies of spells: same family).
- `Obj` holds: `card: Option<CardId>`, `def: CardDefId` (or token def), `owner`, `controller`, `zone: ZoneKind`, `face: FaceState` (face up/down, which face for split/adventure), `tapped`, `phased`, `damage`, `counters: SmallVec<[(CounterKind, u16); 4]>`, `attached_to: Option<ObjRef>`, `timestamp`, `entered_turn` (for summoning sickness), `copy_of: Option<CopyBase>`, `x_value`, `chosen: SmallVec<[Chosen; 2]>` (modes, targets, colors, names), `cast_info` (alt cost used, kicked, etc., for spells on the stack and permanents that care how they were cast).

### 4.2 Zones

| Zone | Representation | Order matters? | Visibility (doc 02) |
|---|---|---|---|
| Library | `Vec<ObjRef>`, top = last | Yes | Hidden from both players; own order unknown unless known via effects |
| Hand | `Vec<ObjRef>` kept in canonical order; treated as a multiset by everything that matters | No | Owner sees; opponent sees only count |
| Battlefield | one shared `Vec<ObjRef>` ordered by timestamp | By timestamp | Public |
| Graveyard | `Vec<ObjRef>`, top = last | Yes (some cards care; also players may inspect order in practice) | Public |
| Stack | `Vec<ObjRef>`, top = last | Yes | Public (including targets/modes, except where rules hide them) |
| Exile | `Vec<ObjRef>` with per-object `face_up` and links | Rarely | Face-up public; face-down visible only to those allowed |
| Sideboard | `Vec<CardId>` | No | Owner only; used only by the match layer and any "wish" effects the pool contains |
| Command | A small `CommandZone` holding **dungeon cards** (one per player at most, with a venture marker on a room) and **emblems** (The Ring, Kaito's emblem). Commander and Planechase are not modeled. Dungeon cards start outside the game and enter only by venturing (CR 309.2); emblems and dungeons are not permanents, but their abilities can trigger from the command zone (CR 309.4c). Per-player records of which **named** dungeons are completed (Acererak checks Tomb of Annihilation specifically) live in `PlayerState` | n/a | Public |

### 4.3 Clone cost

The design goal is that `State::clone()` is a handful of `Vec` memcpys. Rules to protect that:

- No `Box<dyn ...>`, no closures, no `Rc/Arc` inside `State` except `Arc<CardDb>` held by the owning `Game` wrapper, not by `State` itself.
- Per-object variable data uses `SmallVec` with inline capacity chosen from the pool's actual statistics (measure in M0).
- Arena objects: about 100-200 slots at steady state; budget about 10-30 KB per `State`. If measurement shows clone cost dominating search, consider copy-on-write chunks per zone; do not pre-optimize.
- `Frame`, `PendingTrigger`, `ContinuousEffect`, etc. are plain enums and structs; code is referenced by `(CardDefId, AbilityIdx, pc)` into `CardDb`, never by pointer.

### 4.4 Hashing

`State` carries an incrementally maintained Zobrist-style hash over rules-relevant state (not over `knowledge` or `rng`, which have separate hashes). Used for: replay verification, loop detection (section 5.5), differential-test snapshots, transposition tables in search, and fork-consistency invariants. Hash inputs MUST be defined by an explicit function `hash_full(&State)` that is also used to verify the incremental hash in audit builds.

---

## 5. Turn structure and priority

### 5.1 Phases and steps

Beginning (untap, upkeep, draw), precombat main, combat (beginning, declare attackers, declare blockers, first-strike damage if any, combat damage, end of combat), postcombat main, ending (end step, cleanup). Rules points the machine MUST encode:

- Untap step: no priority; phasing and untapping happen as turn-based actions. Summoning sickness is "controlled continuously since the start of this turn", compared against `entered_turn`/control-change timestamp.
- Draw step: the first player skips the draw on their first turn (two-player rule).
- Mana empties at the end of each step and phase (rule 500.5). This matters for ritual-based combo lines.
- Cleanup: discard to hand size, then damage wears off and "until end of turn" effects end simultaneously. If SBA or triggers occur during cleanup, players receive priority and then another cleanup step happens. The machine MUST implement the re-entry loop; it is rare but real (e.g., discard triggers).
- Extra turns and extra combat phases: a queue in `TurnState`.
- Turn-based actions do not use the stack. Triggers that arise from them (e.g., upkeep triggers, draw-step triggers) go on the stack at the next priority.

### 5.2 Priority loop

```text
loop {
  run_sba_and_triggers()            // section 8 and 7: repeat until stable
  give priority to priority_player  // emits Decision::Priority
  on Pass:   if all players passed in succession:
                 if stack empty -> end step/phase
                 else          -> resolve top of stack; priority goes to active player
             else priority passes to next player
  on action: reset the pass counter; execute it; loop
}
```

- Special actions (play a land; others only if the pool needs them) do not use the stack and do not pass priority. After taking one, the same player is checked for SBA/triggers and retains priority.
- Mana abilities do not use the stack (rule 605) and are resolved immediately as part of whatever is being paid. They are not separate "actions" at the priority level unless a player explicitly activates one to produce mana for a later purpose (offered only when it matters; section 6.4).
- All players pass in succession with an empty stack ends the step/phase; with a nonempty stack, the top object resolves.

### 5.3 Auto-passing

The engine itself never skips a decision that an agent could meaningfully take. The match/runner layer MAY apply a **skippable-decision policy**: if a player's only legal action is Pass, the decision is auto-answered. That is a runner policy over the enumerated action set, not an engine shortcut, and it is logged. (This matches the brief's "auto-resolution of forced/trivial decisions" without baking it into rules code.) Doc 02 section 5 defines `Decision::trivial()` precisely.

### 5.4 Who gets priority and when it matters

Because Legacy hinges on responses (Brainstorm in response to a Daze, Stifle, Force of Will), the machine MUST give both players priority at every step where the rules do, including during the opponent's upkeep, draw, and end step, and with the stack non-empty. No "smart stop" heuristics inside the engine.

### 5.5 Loop and runaway protection

- Mandatory infinite loops are rare; optional loops (combo lines that loop for value) are the agent's problem and are bounded by a configurable **action budget per game** and a **same-state-hash repeat counter**. When exceeded, the game ends as a draw (rule 104.4b, mandatory loop) or is aborted as `Truncated` with a logged reason; the runner chooses. The pool contains few true loops; revisit per deck.
- **Loop shortcuts are an optional, shelved design, not in the initial build scope** (Brady, 2026-10-01: "maybe that's not necessary"). The Aluren + Acererak loop is played stepwise for now, bounded by the budget and repeat counter above. If it proves too costly, the design in doc 02 section 9.6 can be built later, since the core change is small. The design follows the CR's shortcut rules (CR 731: a loop repeated a stated number of times, no conditional actions, the other player may accept or shorten it, nobody is forced to break a loop) and is specified in doc 02 section 9.6. The rules core only supplies two things: (a) a `Shortcut` run mode that replays a recorded body of choices through the normal engine paths and **stops at the first deviation**, so a shortcut can never produce a state that stepwise play could not, and (b) an audit-build check that the shortcut result equals the stepwise result (doc 04 invariant I18). The action budget and same-state repeat counter above stay the primary mechanism now and the backstop later for mandatory loops and runaway agents; a shortcut that runs N iterations counts as one agent action but each iteration's engine steps count toward the budget.

---

## 6. The stack, casting, resolution

### 6.1 Stack objects

Stack entries are `ObjRef`s of one of: a card spell, a copy of a spell, an activated ability, a triggered ability. Abilities on the stack are objects with: source (as `ObjRef` plus LKI snapshot), controller, the ability's code reference, chosen targets/modes/X, and any "captured" values from trigger events (e.g., "that much damage"). Abilities exist as objects precisely so that Stifle-style effects (counter target activated or triggered ability) and copy effects have something to target.

### 6.2 Resumable resolution: the continuation stack

The single most important structural decision. Spell resolution, casting, combat damage assignment, and replacement ordering all pause for decisions, possibly several deep (a spell resolves, asks for a choice, which triggers a replacement effect that needs an order choice). Host-language coroutines and recursion are not clonable. Instead:

```rust
enum Frame {
    CastSpell     { obj: ObjRef, step: CastStep, ctx: CastCtx },
    PayCosts      { for_obj: ObjRef, remaining: SmallVec<[CostItem; 4]>, ctx: PayCtx },
    ResolveObject { obj: ObjRef, vm: VmState },          // running effect bytecode, paused at a decision
    ApplyEvent    { proposed: ProposedEvent, stage: ReplStage },
    PutTriggers   { queue: SmallVec<[PendingTrigger; 8]>, apnap_stage: ApnapStage },
    Combat(CombatStep),
    Cleanup(CleanupStage),
    // ... one variant per multi-step procedure; all plain data
}
```

- `advance` pops/pushes frames in a loop. A procedure that needs a decision stores its progress in its frame, sets `pending`, and returns.
- `apply(decision, action)` feeds the answer to the top frame.
- Effect code is compiled to a small bytecode (doc 03, section 5) with `VmState { code: (CardDefId, AbilityIdx), pc, locals }`. This makes "resume after the player picks a target/card/number" a matter of restoring `pc`.

Alternative considered: tree-walking interpreter with explicit continuations. Rejected because bytecode with a program counter gives the simplest possible resume story and is trivially clonable. If DSL authoring ergonomics suffer, the DSL can stay tree-shaped at the surface and compile down.

### 6.3 Casting a spell (CR 601.2)

Implemented as the ordered sequence in the CR, each step a distinct frame stage so decisions fall at the right moments:

1. Move the card to the stack (zone change; becomes a new object). Rule 601.2a. If casting fails later, state reverts (section 6.5).
2. Choose modes (including "choose one or more" and escalate-style patterns if present), X, whether to pay optional costs (kicker, etc.), and which alternative cost (Force of Will, Daze, Misdirection-style costs, flashback, evoke, delve payment decisions later).
3. Choose targets (and divisions, e.g., "divided as you choose").
4. Determine the total cost: base or alternative, plus additional costs, plus cost increases, minus reductions (Thalia-style taxes and Trinisphere-style minimums, with delve and convoke as "payment helpers"), in the CR-specified order. Phyrexian mana and hybrid mana are resolved here.
5. Activate mana abilities (including during cost payment, e.g., Lion's Eye Diamond's discard-hand-as-cost mana ability), then pay costs.
6. Cast/"becomes cast" triggers (storm, cascade, Young Pyromancer, Monastery Mentor) are placed after the spell is fully cast, then the caster gets priority.

Every stage that has more than one legal continuation is a `Decision`. Stages with exactly one legal continuation are executed inline (but still logged as `Decision::trivial` records so the dataset is faithful).

**Legality is guaranteed by pre-validation, not backtracking.** The enumerator only offers an option (a mode, a target, an alternative cost) if a complete legal cast is still achievable after choosing it. The payability check is a small solver (section 6.4). This is what makes "illegal moves are impossible" true without any rollback machinery for normal play.

### 6.4 Mana and costs

- Mana pool: per-player, typed by color/colorless plus restriction tags (spell-only, specific-type) if the pool needs them. Empties per section 5.1.
- **Payability solver**: given a cost (possibly with hybrid, Phyrexian, X, and snow/colorless-specific symbols) and the set of available mana sources (untapped permanents with mana abilities, floating mana, free-cast helpers like delve fodder), decide payable/not, and enumerate **equivalence classes** of payments. Two payments are equivalent if they tap permanents that are interchangeable under all observable rules (same card definition, same state, same attachments). This collapses the huge permutation space (which Island to tap) into a handful of meaningful options.
- Payment decision granularity: the default is **one canonical auto-payment per cast** chosen by a deterministic, documented preference order that preserves flexibility (e.g., spend colorless-producing lands first when the cost allows, preserve lands with relevant extra abilities such as fetchable duals for later Wasteland-vulnerability concerns). The agent API MAY request `PaymentMode::Manual` (a flag on the engine config) to expose the equivalence-class choices as a sub-decision. Rationale: the brief wants small masked prompts; paying 5 mana with 12 lands should not be 5 LLM decisions, but combo decks (Storm, ANT) do depend on which mana is spent when, so the manual mode must exist and be tested.
- Other cost types are primitives (doc 03 section 3): tap/untap, sacrifice (filter), discard (filter; hand choice), pay life, exile from zone (Force of Will's blue card, delve), return permanent to hand (Daze), remove counters, reveal, and compound costs. Each primitive implements `can_pay`, `enumerate_choices`, and `pay`.
- Mana abilities with side effects (Lion's Eye Diamond, Mox-style, fetch-like "sacrifice: add mana", Treasure) are ordinary mana abilities whose costs may include the primitives above. LED's quirk (activatable while casting, discarding the hand as part of the cost) is why mana-ability activation during 601.2g payment is a first-class stage rather than a pre-step.

### 6.5 Reversion

Normal play never reverts because options are pre-validated. A debug/audit safety net is still required: when entering `CastSpell`, `audit` builds snapshot the state; if a cast ever fails mid-sequence, the audit build panics with the snapshot attached (that is an engine bug). Release builds treat the same case as `ApplyError::EngineBug` and abort the game record, flagging it for triage. We do **not** implement the CR's "revert to before casting" semantic for player-initiated cancellation because the API has no cancel action. This is a deliberate scope cut.

### 6.6 Resolution (CR 608)

- On resolution, re-check targets: if all targets are illegal, the spell/ability does not resolve (fizzles); if some are illegal, it resolves and ignores illegal ones (608.2b). Legality is rechecked with current characteristics, including protection, hexproof, and "can't be the target".
- The effect bytecode runs; every state change goes through `ProposedEvent` (section 9) so replacement and prevention effects always get their chance.
- Instants and sorceries go to the graveyard as the last step; spells that become permanents enter the battlefield (an `EnterBattlefield` `ProposedEvent`). Copies of spells cease to exist.
- Countering: a countered spell goes to the graveyard (or wherever a replacement sends it), and "unless its controller pays" (Daze, Flusterstorm) is an effect with an embedded **decision for the spell's controller** during resolution of the counter, offered only if the payment is actually possible (so no illegal "yes" is available).

---

## 7. Events, triggers, APNAP

### 7.1 Event model

Two phases for every change to the game:

1. `ProposedEvent`: what is about to happen (draw a card, move object X from zone A to B, deal N damage from S to R, put counters, gain life, create a token, ...). Passes through the replacement/prevention pipeline (section 9). May be modified, replaced, or prevented.
2. `Event`: what actually happened, after replacements. Appended to the per-turn event log, delivered to the trigger matcher, and emitted to observers (filtered per player, doc 02 section 7). Carries LKI snapshots for departing objects.

`Event` and `ProposedEvent` are closed enums. Adding a variant is a core change (section 14), and the compiler enforces that all handlers (trigger matcher, observer redaction, logging, hashing) are updated.

### 7.2 Trigger detection

- At pool-compile time, the card DB builds a **trigger index**: for each `EventKind`, the list of `(CardDefId, AbilityIdx, zone-of-interest)` triggers that could match. Zones of interest include battlefield (most), graveyard (e.g., "when this is put into a graveyard"), hand (cycling and discard triggers), library and stack where relevant.
- When an event is emitted, only the indexed candidates currently in a relevant zone are examined. No full scan of all objects. Cost is proportional to the number of relevant objects, not the size of the game.
- Triggers from objects that left the game state at the same time (leaves-the-battlefield and dies triggers) use the **LKI snapshot** carried by the event: "look back in time" per rule 603.10.
- Intervening-if clauses (rule 603.4) are checked when the trigger event occurs and again on resolution.
- Triggered mana abilities and replacement-like "as enters" choices are handled in their own paths and are not stack triggers.

### 7.3 Putting triggers on the stack (APNAP)

- Whenever a player would receive priority, after SBA stabilize, any pending triggers are put on the stack: **active player's first, then non-active player's** (APNAP, CR 603.3b). Each player orders their own triggers; the last one put on the stack resolves first, so in a two-player game the non-active player's triggers from the same event resolve before the active player's. CR 603.3b is a **two-part process**: first each player in APNAP order puts on the stack the triggers whose trigger condition is *not* "another ability triggering"; second, remaining triggers (those that trigger off other abilities triggering) go on the same way. SBA are then re-checked and the loop repeats until nothing new happens.
- Ordering decision: `Decision::OrderTriggers` is offered to a player only when they control two or more pending triggers that are not interchangeable. Interchangeable means identical source definition and identical captured parameters; those are placed in canonical order without asking.
- Targets and modes for a triggered ability are chosen at the moment it is put on the stack (603.3d), by its controller, subject to the same pre-validation as casting. If a trigger has no legal target, it is removed.
- A trigger that is created while SBA or another trigger loop is still running is held until the next stabilization point; the machine loops until no SBA applies and no triggers are pending.
- Delayed triggers (Shallow Grave's end-step exile, Phelia's return, mobilize Warriors' sacrifice, "at the beginning of the next end step") are registered into `state.delayed` with an event pattern and a reference to the object they care about; they fire into the same pending queue.
- Reflexive triggers ("when you do") are modeled as an effect primitive that registers a delayed trigger firing immediately after the parent resolves.

### 7.4 Simultaneity and ordering inside a resolution

Events produced by a single resolution step that the CR treats as simultaneous (e.g., an effect destroying several creatures; combat damage dealt) are emitted as a batch. SBA and triggers evaluate after the whole batch. Within a batch, per-object events are ordered by timestamp and then by canonical id for determinism only, with no rules meaning.

---

## 8. State-based actions

Run in a loop whenever a player would receive priority, and at a few other defined points (e.g., during cleanup), until a pass finds nothing to do. All applicable SBA in a pass are performed simultaneously (rule 704.3).

SBA implemented (names below; lettering matches CR 704.5a-z as of the CR effective 2025-11-14, e.g. life 704.5a, empty-library draw 704.5b, legend rule 704.5j, planeswalker loyalty 704.5i, Aura 704.5m, Equipment 704.5n, counter annihilation 704.5q). Tier A = needed by a pool deck; Tier B = only if a deck needs it; Tier X = not needed, loader rejects.

| Tier | SBA |
|---|---|
| A | Player with 0 or less life loses |
| A | Player who attempted to draw from an empty library loses |
| X | Player with 10 or more poison counters loses (no poison sources in the pool; loader rejects poison) |
| A | Token in a non-battlefield zone ceases to exist; spell copy not on the stack and card copy not on the battlefield cease to exist |
| A | Creature with toughness 0 or less goes to graveyard |
| A | Creature with lethal damage marked is destroyed (indestructible and regeneration-like replacements apply via the pipeline) |
| A | Creature dealt damage by a deathtouch source is destroyed |
| A | Legend rule: controller chooses one legendary permanent with the same name to keep; the rest go to the graveyard (a decision, with duplicates of identical-state permanents collapsed) |
| A | Aura attached illegally or not attached goes to graveyard; Equipment attached illegally becomes unattached |
| A | +1/+1 and -1/-1 counters on the same permanent annihilate in pairs |
| A | Planeswalker with 0 loyalty goes to graveyard (the pool has Kaito, Jace Wielder, and the back faces of Ajani and Tamiyo) |
| A | Dungeon whose venture marker is on its bottommost room, and which is not the source of a room ability that is still on the stack, is removed from the game and counts as completed (CR 704.5t, 309.6, 309.7; Acererak) |
| B | World rule, saga, battle, role, and other rarely-relevant SBAs: unimplemented unless a deck needs them |

Notes:

- SBA never use the stack and cannot be responded to.
- The "player loses" and "game ends" machinery handles simultaneous losses (draw) per rule 104.4a.
- The SBA pass reads **derived** characteristics (layers, section 10), so it MUST ensure the derived cache is fresh before checking toughness, types, and legend names.

---

## 9. Replacement and prevention effects

### 9.1 Pipeline

Every game-state change that is subject to replacement is expressed as a `ProposedEvent` and processed by:

```text
fn process(proposed) -> Outcome:
    loop:
        applicable = replacement effects whose condition matches `proposed`
                     and that have not yet applied to this event (rule 614.5)
        if applicable is empty: break
        chosen = if |applicable| == 1: that effect
                 else: ask the affected player/controller to choose (rule 616.1), see ordering below
        proposed = chosen.apply(proposed)           // may substitute a different event, or prevent (Outcome::Prevented)
    commit(proposed) -> Event(s)
```

Ordering when multiple replacement effects apply (CR 616.1, checked against the CR effective 2025-11-14): the affected object's controller (or owner, or the affected player) chooses; if several players must choose, they choose in APNAP order. The choice is constrained step by step: if any self-replacement effect applies one must be chosen (616.1a); else if any effect changes under whose control the object enters (616.1b); else any copy-as-enters effect (616.1c); else any enters-with-back-face-up effect (616.1d); else any applicable effect (616.1e). After each application the process repeats with only the effects still applicable (616.1f), and an effect that applies inside another event's replacement can't be chosen until the outer one has been (616.1g). The chooser sees a `Decision::OrderReplacements` only if their options are not interchangeable.

### 9.2 Sources of replacement effects

- Static abilities of permanents (Rest in Peace, Leyline of the Void, Containment Priest, Blood Moon is **not** a replacement effect but a layer effect, section 10).
- "As this enters" and "enters with" effects: shocklands (pay 2 life or enter tapped), enters-with-counters, enters tapped, copy-as-enters (Clone-like cards).
- Continuous effects created by resolved spells (Yawgmoth's Will-style "if a card would go to your graveyard, exile it instead").
- Dredge (draw replaced by a return from graveyard; an optional replacement, so agent decision).
- Damage prevention/redirection (if the pool has them). Prevention effects are implemented as replacement effects returning `Prevented` or a reduced amount.
- Protection is **not** modeled as a replacement effect: it is a static quality enforced at four points (damage, enchant/equip legality, blocking, targeting) via a single `protection_blocks(source, object)` function used by all four.

### 9.3 Scope cuts

- No "layered" interactions between replacement effects beyond what the CR ordering rules require; the pipeline handles chains by iterating.
- Regeneration, if the pool has it, is a replacement shield effect on destroy events.
- Replacement effects that modify what zone an object enters *and* what counters it has *and* which face it is (Dryad Arbor-style, copy effects) are ordered per CR and tested explicitly in the rulings suite.

---

## 10. Continuous effects, layers, timestamps

### 10.1 Representation

Continuous effects have three origins:

1. **Static abilities** of permanents (and cards in other zones where relevant, e.g., cost reducers on the stack). Defined in card data. Their timestamp is the timestamp of the source object's arrival on the battlefield (or the ability's gain time).
2. **Resolved spell/ability effects** with durations (until end of turn, until your next turn, as long as X). Stored in `state.conts` with a timestamp, a duration, and either a **locked-in affected set** (rule 611.2c: most effects from resolved spells affect only what was affected at resolution) or a dynamic set (static abilities).
3. **Characteristic-defining abilities** (Nethergoyf, Barrowgoyf, devotion-style): static abilities evaluated in layer 7a with no dependence on source state beyond the stated formula.

### 10.2 Layers (rule 613)

Applied in this order, each layer in timestamp order unless a dependency overrides:

| Layer | What it does | Pool relevance |
|---|---|---|
| 1 | Copy effects (and face-down effects) | Not needed as a continuous effect (doc 05). Copiable values are still needed to create token copies (Ocelot Pride, Lazotep Quarry) |
| 2 | Control-changing effects | **Not needed** (Reanimate-style cards enter under your control; that is not a layer effect); Tier X |
| 3 | Text-changing | Not needed; Tier X |
| 4 | Type/subtype/supertype changing | **Needed**: Kaito is a creature on your turn, Overlord of the Balemurk is not a creature while it has time counters (impending), Magus of the Moon (Reanimator sideboard) makes nonbasic lands Mountains |
| 5 | Color changing | Not needed; Tier X |
| 6 | Ability adding/removing | **Needed**: Magus of the Moon strips nonbasic lands' abilities (CR 305.7), Kaito gains hexproof on your turn, Guide of Souls' flying counter, Veil of Summer's hexproof from two colors |
| 7a | Characteristic-defining P/T | **Needed**: Barrowgoyf, Nethergoyf |
| 7b | Set P/T | **Needed**: Kaito's 3/4 while a creature; token base values (Warriors, Cats, Zombies) |
| 7c | Modify P/T (including +1/+1 and -1/-1 counters) | **Needed**: counters (Moonshadow's -1/-1, Guide of Souls' +1/+1), Cori-Steel Cutter, Kaito's emblem |
| 7d | Switch P/T | Not needed; Tier X |

The table keeps all layers so the machinery is shaped correctly (cheap to keep the ordering skeleton), but only the rows marked Needed are implemented for the current pool: layers 4, 6, 7a, 7b and 7c. The loader rejects effects in layers 2, 3, 5, 7d and layer-1 continuous effects (section 16).

### 10.3 Evaluation and caching

- `derive(state) -> Derived` computes per-object characteristics: types, subtypes, supertypes, colors, controller, ability list (as a handle into card data plus granted abilities), power, toughness. It runs on demand when the **dirty flag** is set.
- Dirty triggers: any zone change involving the battlefield or a zone with static abilities of interest, any counter change, any attachment change, any new or expired continuous effect, any control change, any phasing, or any characteristic-relevant choice (e.g., chosen color for a "choose a color" permanent).
- Incremental recompute is an optimization, to be considered only if M0/M1 profiling shows derive dominating. The correctness contract is: **incremental result == full recompute** (checked in audit builds; doc 04 invariant I11).
- Timestamps: a monotonic `u32` allocated by the engine on permanent entry, effect creation, ability gain, and control change. Effects that enter simultaneously are ordered by APNAP then by the controller's chosen order; this is a `Decision::OrderSimultaneousEntries` and is trivially skipped if there is no interaction.

### 10.4 Dependencies (rule 613.8) — scope cut with a safety net

Full dependency ordering is the one deliberately staged feature:

- **Stage 1 (default):** layers and timestamps only, no dependency detection.
- **Stage 2 (not needed by the current 8-deck pool per my manual read in doc 05, even though layers 4 and 6 are now in use: Kaito, Overlord and Magus touch disjoint objects; `deps-scan` must confirm, with Magus of the Moon vs any land-ability granter as the pair to check; enable only if the pool changes):** dependency detection by **trial application**: for two effects in the same layer, A depends on B if applying B first would change A's existence, what it applies to, or what it does. Implemented by applying each effect to a scratch copy of derived state, which is cheap because derive operates over small data. Opalescence/Humility-style interplay and Blood Moon vs Urborg-style type-change fights are the targets (Magus of the Moon is a Blood Moon effect, so a second type-changer on lands would trigger this).
- **Safety net, always on:** a static analysis tool `deps-scan` runs over the pool's card DB and lists card pairs (within the closed pool) whose effects share a layer **and** touch overlapping attributes. Any pair flagged and not covered by Stage 2 behavior fails the deck gate until it is either (a) handled by Stage 2, (b) proven independent in a recorded review, or (c) cut from the pool. This matches the brief's rule 3 (skip dependency ordering if no Humility/Opalescence-style interactions are present) without trusting a human to remember.

### 10.5 Cost-modification and restriction effects

Effects that change costs (Thalia, Trinisphere-style minimums), restrict actions (Rule of Law, Ethersworn Canonist, Null Rod/Collector Ouphe-style activation limits, Leyline of Sanctity-style hexproof-for-player), or impose "can't" statements are static abilities in a separate **rules-modifier registry**. They are consulted by the enumerator and by cost determination. "Can't" beats "can" (rule 101.2). These are not layered; they are evaluated at the point of use against current derived state.

---

## 11. Combat

- Declare attackers: attackers are chosen per-creature as a sequence of small masked decisions (attack this creature at {player, planeswalker, no attack}) with canonicalization that collapses identical creatures. Attack requirements and restrictions ("must attack", "can't attack unless", "can't attack alone") are static abilities consulted by the mask; the sequence MUST end in a legal declaration (pre-validated, so no revert).
- Attack triggers and tap-on-attack are handled via the normal event path.
- Declare blockers: per potential blocker, a decision among legal attackers (and "no block"). Evasion and restrictions (flying, menace, protection, "can't be blocked by", "can block additional creatures") are enforced by a single `can_block(attacker, blocker)` function plus a final legality check for menace-style requirements on the full assignment.
- Damage assignment: **version-sensitive, and verified.** Older CR versions made the attacker order its blockers and assign damage in that order. The CR effective 2025-11-14 has no damage assignment order: a creature blocked by two or more creatures assigns its combat damage divided as its controller chooses among them (CR 510.1c); trample still requires lethal damage to all blockers before excess goes to the player. The Forge source (commit fd5c996, 2026-09-30) still has the old ordering step (`orderBlockers` on its player controller, `Combat.orderBlockersForDamageAssignment`), so **Forge diverges from the CR on multi-block combat**; this is known-divergence entry kd-0001 (doc 04, section 9). Assignment is a `Decision::AssignCombatDamage` only when the choice is non-trivial (multiple blockers, or trample with multiple recipients); otherwise it is automatic and deterministic.
- First strike and double strike: two damage steps; creatures with the relevant keywords deal in the first step; double strikers deal in both.
- Trample, deathtouch, lifelink, infect-like, wither-like: damage is a `ProposedEvent` carrying source properties (lifelink applies at damage event application, deathtouch marks a flag for SBA).
- Combat-specific continuous effects (e.g., "attacks each combat", "can't block this turn") are ordinary continuous effects with duration.
- Removed-from-combat rules: leaving the battlefield removes a creature from combat; phasing out removes too. Tested in the rulings suite.

---

## 12. Randomness and determinism

- One `Pcg64`-class generator (or similar well-specified, portable, version-pinned algorithm) per game in `State`. It drives only shuffles and the few random choices the rules require (random discard, random target selection if any).
- RNG algorithm and shuffle algorithm are fixed and documented (e.g., Fisher-Yates over the library `Vec` using a specified `gen_range` implementation). Changing either is a **core change** and invalidates golden replays (section 14.3).
- **No** use of: `std::collections::HashMap/HashSet` iteration in rules code (lint-enforced; use `BTreeMap`, `IndexMap`, or sorted `Vec`), floating point, thread-local state, system time, pointer addresses, or parallelism inside a game.
- A game record `GameRecord { engine_version, card_db_hash, decks, seed, actions: Vec<(DecisionId, ActionIdx)> }` replays bit-identically. This is the unit of storage for the training dataset and for bug repros.
- Search/simulation forks reseed their RNG (doc 02 section 5) so rollouts do not share the real game's future shuffles.

---

## 13. Performance design

Targets for the M0 spike to validate (all are initial hypotheses):

| Metric | Initial target | Notes |
|---|---|---|
| Random-legal-play game, midrange vs midrange, one core | >= 100x Forge games/sec on the same matchup (Forge number comes from Phase 1 deliverable 1) | Brief's 100-1000x claim becomes a measured gate |
| `State::clone` | few microseconds | Pure memcpy plus a few small allocations |
| Legal-action enumeration at a priority decision | single-digit microseconds typical | Incremental caches allowed |
| Memory per live game | tens of KB | Enables thousands of concurrent games per core pool |

Design levers (already in the layout):

- Index-based arenas, `SmallVec`, no per-event heap allocation in the steady state. Event and frame vectors are reused scratch buffers inside `State` (not part of the hash or the clone-equality contract).
- Trigger index (section 7.2) and rules-modifier registry (section 10.5) avoid scanning the whole board.
- Derived-characteristics cache with a dirty flag (section 10.3).
- Enumerator results are cached per `(state hash, decision)` only inside search wrappers, not in the core.
- Self-play parallelism is across games (one thread per game, many games), never inside a game.
- Benchmarks (criterion) are part of CI from M1 with regression thresholds. A change that slows the headline benchmark by more than an agreed percentage needs an explicit sign-off (section 14.3).

---

## 14. Change control, freeze policy, and agent roles

### 14.1 What "core" means

Frozen after the M3 review (section 16):

- `mtg-core`: `State` layout, zones and object identity, turn/priority machine, stack and cast/resolve procedure, SBA, event enums and pipelines (trigger, replacement), layer evaluation, combat machinery, RNG and determinism contract, effect VM instruction set and semantics, enumerator contract, `mtg-view` API types.

Not frozen (extension points with a lighter review):

- Card data (DSL files) and hand-written card functions.
- New DSL primitives or VM ops (adding a case to an existing closed enum is a **core-adjacent** change: needs review by someone other than the author, doc 04 differential run, and a rulings scenario authored by the spec agent).
- Observers, encoders, search, Python bindings.

### 14.2 Change process for core

1. An RFC file `rfcs/NNNN-title.md`: problem, proposed change, CR citations, impact on state layout and hash, impact on determinism (golden replays), impact on hidden-info, and test plan.
2. Review by at least two independent reviewers (any mix of human and reviewer-agents, none of whom authored the change; one MUST be a "rules reviewer" role reading against the CR).
3. All gates green: unit tests, rulings suite (including holdout), differential suite on the matchups affected, invariant fuzz, noninterference test (doc 04 section 7), benchmark non-regression.
4. If the change alters golden replay hashes, the PR MUST include a regenerated-hash report explaining every changed replay (expected vs surprising).
5. `ENGINE_CORE_VERSION` bumps. Stored game records carry it.

### 14.3 Golden replays

A small library of recorded games (`goldens/`) with expected state hashes at checkpoints. Any change that alters them without an RFC fails CI. This is the cheapest guard against silent drift from many agents.

### 14.4 Role separation (agent-built engine)

| Role | Writes | May NOT write |
|---|---|---|
| Spec-writer agent | Rulings-derived scenarios, expected outcomes, known-divergence entries, RFC test plans | Engine code, card implementations |
| Implementer agent | Engine code, card data, hand-written card functions | Anything under `tests/spec/**`, `tests/holdout/**`, `known-divergences/**` |
| Reviewer agent (rules) | Review comments, RFC verdicts | Merges its own review targets |
| Triage agent | Differential mismatch classifications (doc 04) | Fixes to code it classifies |
| Human (Brady or delegate) | Final say on accepted divergences, deck-gate sign-off, core RFC approval | n/a |

Enforcement is mechanical, not honor-system: CODEOWNERS-style path permissions in the repo, a CI check that rejects PRs where the author wrote both a card and its spec scenarios, and a **holdout set** (doc 04 section 4.4) the implementer cannot read.

---

## 15. Milestones

| M | Deliverable | Exit criteria |
|---|---|---|
| M0 | Throughput/clone spike: vanilla creatures, basic lands, burn, combat-lite; random play | Measured games/sec vs Forge; clone cost; decision on layout tweaks. No core freeze. |
| M1 | Core skeleton: turn/priority, stack, SBA, events, action API, determinism, golden replays, invariant fuzz v0 | Fuzz 10^7 steps clean; hash stable under replay |
| M2 | Casting in full, costs, triggers, APNAP, replacement pipeline, effect VM, DSL v0, first non-combo cards | Rulings suite for implemented cards green; differential vs Forge on scripted scenarios |
| M3 | Combat, layers (stage 1), Tier A SBA, forks/determinizer, `mtg-view` | **Core review and freeze** |
| M4 | Two non-combo decks fully implemented and gated | Deck gate (doc 04 section 8) passed for both |
| M5 | Combo decks (Alurentell/Aluren, Doomsday, Reanimator; Show and Tell and Stronghold Gambit shapes) and the dungeon, Ring and Kaito cards: hand-written cards, dependency stage 2 if the scan demands it | Deck gates |
| M6 | Python bindings, batched stepping, training handoff | Phase 3 can consume it |

The brief says the core can be developed in parallel with Phase 1; M0 and M1 are the work that can safely start early, since M0 is exactly what tells us whether Forge's limits justify the rest.

---

## 16. Scope management: the mechanics inventory

Before choosing the final 6-8 decks, a script scans candidate decklists and emits a **mechanics inventory**: for each rules feature, how many cards use it. Features are tiered:

| Tier | Meaning | Examples |
|---|---|---|
| Core | Always implemented | Priority, stack, SBA, triggers, combat basics, mana, replacement pipeline, layers 4-7c |
| Pool-gated | Implemented only if a chosen deck uses it | Dependency stage 2, copy effects (layer 1), control change (layer 2), planeswalkers, delve, storm, cascade, flashback, adventure/split cards, Phyrexian mana, convoke, suspend, madness, prowess/cycling-style keywords, Lion's Eye Diamond payment timing, wishes |
| Out | Never, unless the user adds a deck that needs it | Day/night, sagas, battles, Commander rules |

The inventory is regenerated whenever the pool changes. It sets the work list and doubles as the scope guard: any rule feature not in the inventory is unimplemented, and the card loader refuses cards that use it.

The concrete inventory for the current 8-deck pool is `05-pool-mechanics-inventory.md`.

**CR version pin:** at M0, pin one CR version (date) in `RULES_VERSION.md` (the latest text available while drafting these docs was effective 2025-11-14; a newer one may exist), vendor its text under `docs/cr/` (check redistribution terms with Wizards' fan content policy before committing; otherwise keep a fetch script and a hash), and record rule-section references against it. Every card carries the Oracle-text snapshot hash it was implemented against; an Oracle update flags the card for re-review.

---

## 17. Match layer (outside the frozen core)

- Starts a game from two 75-card lists (60 main + 15 sideboard, or whatever the pool uses), performs the pre-game: determine who plays first, London mulligan (draw 7; each mulligan draws 7 and the player puts N on the bottom, a decision with both hidden-info implications), opening-hand effects.
- Mulligan decisions are real decisions via the same `Decision` API but are tagged `Low-frequency` so Phase 3 can route them to heavier methods, per the brief.
- Sideboarding between games of a match is a separate API that never enters the rules core; the engine just receives two final 60-card decks plus the sideboards (for wish effects, if any).

---

## 18. Risks specific to this design

| Risk | Mitigation |
|---|---|
| Plausible-but-wrong rules (agent-built) | Role separation, rulings-as-spec, holdout set, differential testing, self-play anomaly detection (doc 04) |
| Core drift | Freeze after M3, RFC process, golden replays, exhaustive matches |
| Resumable-frame design proves too heavy | M0/M1 prototypes it immediately; fallback is a deterministic-replay design (replay the action list from the last checkpoint on fork) at some performance cost |
| Clone cost not as low as hoped | Measured in M0; COW chunks per zone are the escape hatch |
| Forge is wrong where we are right | Three-oracle triage (doc 04), accepted-divergences registry with CR citations |
| RL exploits an engine bug | Deck gate before training; anomaly detectors during training; periodic re-gating |
| Combo decks stress rarely-used rules (LED timing, Doomsday piles, simultaneous hidden choice in Show and Tell and Stronghold Gambit, Aluren's any-player free casting, mana-ability timing, storm copies) | Each is a named hard-card entry in doc 03 section 8 with its own scenario set |
| License contamination | Do not copy Forge card scripts or code. Forge's `LICENSE` is GPL-3.0 (checked in the repository), so copying its card scripts or code would encumber the engine; using it as an external oracle process is fine. XMage's `LICENSE` is MIT (checked on its default branch). |

---

## 19. Open questions

1. Deck set: the final eight decks are in `/mnt/project-files/decks/` (UR Cutter, Alurentell, Boros Aggro, BW Death and Taxes, Dimir Tempo, UWx Control, Doomsday, Reanimator). If it changes, regenerate `05-pool-mechanics-inventory.md`.
   1a. Dungeons: all three are implemented (Alurentell's Acererak loop depends on choosing Lost Mine or Mad Mage instead of Tomb of Annihilation, and finishing Tomb is the alternative when tax effects stop the loop); edge-case rulings go to the spec-writer.
2. Compute sizing. Brady's resources are one local GPU and a subscription budget (no per-token API budget). That makes engine throughput and CPU-side search the main levers, and it favors a modest-size policy/value net (sized to fit one GPU), batched inference, and keeping the LLM out of inner loops; LLM use (spec-writer, triage, orchestrator) must fit subscription usage limits, so doc 04 section 10 keeps LLM calls per card or per mismatch signature, never per game.
3. Whether Brady wants a human rules reviewer in the loop at deck-gate time (doc 04 section 8 assumes yes for sign-off and for accepted divergences).
4. Whether sideboard-dependent cards ("wishes") are in the pool (affects hidden-zone modeling of the sideboard).
5. Python training stack constraints (for `mtg-py` batching API shape).
