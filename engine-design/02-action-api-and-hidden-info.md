# MTG Engine — Action API and Hidden-Information Guarantees

Status: DRAFT v0.1. Companion to `01-core-design.md`. Section numbers here are referenced from doc 01 (sections 3, 4, 5, 6, 7) and doc 04.

---

## 1. Principles and threat model

**Who is an "agent"?** Anything that chooses actions or advises a chooser: the LLM player, the neural policy/value net, the search procedure, every tool the LLM can call (`simulate`, `sample_opp_hands`, `library_odds`, `get_card`, notes), the opponent model inside a forked simulation, and every logging/export pipeline that feeds a training dataset to those.

**What counts as a leak?** Any influence of the opponent's hidden state, or of the true order of the agent's own library, on anything an agent can read, beyond what a human sitting at the table would know. Hidden state means: opponent hand identities, both libraries' order, face-down cards the agent may not see, the RNG state and seed, sideboards, and any secret choices (for example, a card chosen secretly during Show and Tell).

**Guarantee we want (non-interference):** Fix an observer seat. Take two complete game states that agree on everything the observer is entitled to know (public zones, the observer's own hand and knowledge, public history) but differ in hidden state. Apply the same observer actions, and the same opponent actions where legal in both. Then the observer's entire stream of observations, decision lists, and errors is **identical** in the two worlds. Doc 04 section 7 turns this into an automated test; this document describes how the design makes it true.

**Out of scope (residual risks, stated plainly):**

- Timing and side channels from an adversarial agent measuring wall-clock latency of engine calls. Not mitigated in-process; if RL agents ever show suspicious exploitation, move the engine behind a process boundary.
- An agent with arbitrary code execution inside the engine process. The boundary below protects against API misuse and accidental leaks, not against hostile native code.

---

## 2. The agent-facing API (`mtg-view`)

Everything an agent can do goes through these types. `State` is not nameable from this crate's public interface.

### 2.1 Handle and stepping

```rust
pub struct Game { /* private: State, Arc<CardDb> */ }

impl Game {
    pub fn new(db: Arc<CardDb>, decks: [DeckList; 2], seed: u64, cfg: GameConfig) -> Game;
    pub fn advance(&mut self) -> Status;                    // runs the engine to the next decision or game end
    pub fn decision(&self) -> Option<&Decision>;            // the pending decision, if any
    pub fn observe(&self, seat: Seat) -> Observation;       // the only way to read the game
    pub fn apply(&mut self, id: DecisionId, idx: ActionIdx) -> Result<(), ApplyError>;
    pub fn result(&self) -> Option<GameResult>;             // only when the game is over
    pub fn fork(&self, seat: Seat, seed: u64, model: &dyn BeliefModel) -> Fork;  // determinized (section 5.3)
    pub fn god_record(&self) -> Option<GodRecord>;          // returns Some only when the game is over (section 8)
}
```

`observe(seat)` takes a seat so a single process can host both players (self-play) and the match layer can build per-seat training streams. An agent object is handed a `SeatHandle` that is bound to one seat and cannot call `observe` for the other seat. The orchestrator owns the `Game`; agents never do.

### 2.2 Decision and action

```rust
pub struct Decision {
    pub id: DecisionId,            // monotonic per game; stale ids are rejected
    pub seat: Seat,
    pub kind: DecisionKind,
    pub options: Vec<ActionDesc>,  // the legal action mask, densely indexed
    pub context: DecisionContext,  // human/LLM-readable framing: what is being decided and why
    pub trivial: bool,             // see section 5.2
}

pub struct ActionDesc {
    pub idx: ActionIdx,            // u16, position in `options`
    pub kind: ActionKind,          // Pass, PlayLand, CastSpell, ActivateAbility, ChooseTarget, ChooseMode, ...
    pub subject: Option<ViewId>,   // the card/permanent/player acted on, in the observer's id space
    pub detail: ActionDetail,      // structured: alt cost used, target list, X value, ...
    pub label: String,             // short canonical text for LLM prompts (card names only; Oracle text lives in the cached glossary)
}
```

`DecisionKind` (each is a small masked prompt, matching the brief's multi-step decomposition):

| Kind | Used for |
|---|---|
| `Priority` | Pass; play land; cast spell (per castable face and per alternative cost); activate ability; activate mana ability for later use |
| `Mulligan` / `BottomCards` | London mulligan keep-or-mull, then bottoming |
| `ChooseMode`, `ChooseX`, `ChooseOptionalCost`, `ChooseAltCost` | Casting and activating (601.2b) |
| `ChooseTarget { slot }` | One decision per target slot, with only valid candidates in the mask |
| `PayMana` | Only in `PaymentMode::Manual` (doc 01 section 6.4): choose among equivalence classes |
| `ChooseCards { from, count, purpose }` | Discard, sacrifice, cast-from-hand choices, Brainstorm put-back, search results, Doomsday pile selection |
| `OrderCards { purpose }` | Brainstorm/Ponder/Top ordering; Doomsday pile order (sequential picks, bottom to top) |
| `YesNo` | "May" effects, Daze/Flusterstorm "pay {1}?", optional replacements (dredge) |
| `ChoosePlayer`, `ChooseNumber`, `ChooseColor`, `ChooseName` | Self-explanatory |
| `DeclareAttacker { creature }` / `DeclareBlocker { creature }` | Per-creature sequential decisions with canonicalized masks |
| `AssignCombatDamage` | Only when non-trivial |
| `OrderTriggers`, `OrderReplacements`, `OrderEntries`, `LegendRule` | Ordering/selection choices from the rules machinery |
| `SimultaneousSecretChoice` | Show and Tell and Stronghold Gambit simultaneous hidden choices (section 9.1) |
| `ProposeShortcut` (an option inside `Priority`), `ShortcutResponse` | **Optional, not in initial scope.** Loop shortcuts (section 9.6): the proposer endorses "repeat the last iteration N times"; the other player accepts or shortens |
| `ChooseRoom`, `ChooseDungeon` | Venture into a dungeon: pick the next room along an arrow, or pick which dungeon on every fresh venture (all three: Lost Mine of Phandelver, Dungeon of the Mad Mage, Tomb of Annihilation; Acererak's player avoids Tomb on purpose). Opponents also get decisions inside Tomb's room abilities (discard or lose life, sacrifice or lose life) as ordinary `YesNo`/`ChooseCards` prompts |
| `ChooseCards` with a constraint | Escape costs such as Nethergoyf's "four or more card types among the exiled cards": picks are sequential and `done` appears only once the constraint is satisfiable and met |

An action is only ever an index into the current decision's `options`. There is no free-form action input. "Illegal moves are impossible" is therefore true at the API surface: an out-of-range index or stale `DecisionId` is rejected, and every in-range index has already been validated as completable (doc 01 section 6.3).

### 2.3 Observation

```rust
pub struct Observation {
    pub seat: Seat,
    pub turn: u16, pub phase: Phase, pub active: Seat, pub priority_holder: Seat,
    pub me:  SelfView,        // life, energy, mana pool, designations (city's blessing, Ring level and Ring-bearer), dungeon and venture-marker state, hand (identities), graveyard, exile, library_count, library_knowledge, per-turn flags
    pub opp: OppView,         // life, energy, public designations and dungeon state, hand_count, revealed_hand (identities the observer legitimately knows), graveyard, face-up exile, library_count
    pub battlefield: Vec<ViewPermanent>,   // both sides, public
    pub stack: Vec<ViewStackItem>,         // public, with targets and modes
    pub combat: Option<ViewCombat>,
    pub events: Vec<ViewEvent>,            // events since this seat last acted, already redacted (section 7)
    pub decision: Option<DecisionView>,    // present when `seat` is the decider
    pub view_hash: u64,                    // hash over THIS struct only, never over State
}
```

- `SelfView.library_knowledge`: identities and positions of the observer's own library cards the observer genuinely knows (for example, the top two after Brainstorm), plus the **unseen multiset** (decklist minus cards known elsewhere). The unseen multiset is derivable from the decklist and public information, so it does not depend on the library's true order.
- `OppView`: the opponent's decklist is treated as known to the observer by default (the brief's closed-pool assumption, and the premise of `sample_opp_hands`). `GameConfig::opp_decklist_visibility` can be `Full` (default), `Archetype` (card pool known, copy counts unknown), or `None` for experiments. Bo3 sideboarding makes `Full` inexact after game 1; Phase 3 belief models handle that.
- Everything is expressed in the observer's `ViewId` space (section 4.2), never engine ids.

---

## 3. Enforcement by construction

Defense in depth; each layer catches a different class of mistake.

1. **Crate boundary.** Only `mtg-view`, `mtg-match`, `mtg-debug`, `mtg-diff`, `mtg-fuzz` may depend on `mtg-core`. A CI check parses `cargo metadata` and fails if any other crate (agents, encoders, Python bindings, tools) has `mtg-core` in its dependency closure other than through `mtg-view`. `State` has no public fields and `mtg-view` does not re-export it.
2. **Projection, not filtering.** `Observation` is built by constructing new structs from allowed fields. There is no "serialize state, then blank out secrets" path, so a forgotten field cannot leak because it was never copied. Adding a field to `Observation` is an RFC-level change (doc 01 section 14) with a mandatory "who is entitled to see this?" line.
3. **No serialization of internals.** `State` and `Knowledge` do not implement `Serialize`/`Debug`-with-contents in builds without the `diff-harness` feature. `Debug` on `State` prints a redacted placeholder.
4. **Debug tooling is a separate crate and a feature flag.** `mtg-debug` (full-state dumps, canonical snapshots for differential tests) is only buildable with `diff-harness`, and release/agent builds assert at CI time that the symbol set does not include it.
5. **Tools are views over the same API.** `simulate`, `sample_opp_hands`, and `library_odds` (section 6) are implemented against `Observation` + `Knowledge` only, so they cannot see more than the agent does.
6. **Opponent models inside forks play on sampled worlds**, never the true state (section 5.3).
7. **God-view data is sealed until game end.** Replay records contain the seed and thus the full hidden state (section 8); they are returned only after the game ends and are handled as a distinct data class.
8. **Non-interference test** (doc 04 section 7) runs continuously in CI and fuzzing.

---

## 4. Identity and ordering leaks (the subtle ones)

Hidden-info bugs in game engines are usually not "the opponent's hand was printed"; they are identifiers and orderings that correlate with hidden state. Rules we adopt:

### 4.1 Internal ids never reach agents

- `CardId` is assigned in **decklist order**, not library order. If `CardId`s followed the shuffled library, the id of the card you just drew would reveal your remaining library's order.
- `ObjRef.slot/gen` and `event_seq` counters are internal. A rising counter would reveal how many hidden zone changes the opponent performed.

### 4.2 `ViewId`: per-observer opaque ids

- Each observer has a private counter. An object receives a `ViewId` the first time it becomes visible **to that observer** (own hand cards when drawn or seen; any public-zone object when it appears). Ids are stable while the object stays in a zone where it remains the same object; a zone change allocates a new `ViewId` (mirroring rule 400.7), except where the observer legitimately tracks identity across the change (for example, a known card bounced to hand: the observer's `ViewCard` retains a `physical_ref` that is stable across zone changes *only for cards whose identity the observer knows*).
- Opponent hidden-zone objects have **no** `ViewId` at all; the observer sees counts.
- Within a batch (for instance, drawing three cards), ids are assigned in the order of the events the observer would perceive.

### 4.3 Canonical ordering

- The observer's own hand is presented sorted by `(CardDefId, ViewId)`.
- Option lists in `Decision.options` are sorted by a **canonical key** built only from information visible to the observer (`ActionKind`, `CardDefId`, `ViewId`, target view ids). The enumerator must not iterate hash maps or arena order that correlates with hidden state.
- Opponent graveyard and battlefield are shown in their true order (public). Tied orderings use `ViewId`.

### 4.4 Errors and logs

- `ApplyError` has a closed set of variants (`StaleDecision`, `BadIndex`, `GameOver`) and carries no free-form text derived from state.
- `ViewEvent` text is generated from `ViewEvent` fields only (never from internal event structs).
- `view_hash` is over the observation, not over `State`. A full-state hash is available only in `mtg-debug`.

### 4.5 RNG isolation

- The seed and RNG state live in `State` and are never in an observation. `GodRecord` contains the seed (section 8).
- Forks get fresh RNG seeds supplied by the caller (section 5.3), so a fork's future shuffles are unrelated to the real game's future.

---

## 5. Decision flow, trivial decisions, forks

### 5.1 Stepping model

`advance` runs rules machinery until a `Decision` is pending or the game ends. The orchestrator asks the decider's agent for an index, calls `apply`, then `advance` again. Batched self-play runs many `Game`s per worker thread and calls agents with batches of observations (section 10).

### 5.2 Trivial decisions

`Decision.trivial` is `true` iff `options.len() == 1`. The runner's **auto-resolution policy** may answer trivial decisions without calling an agent. It may also apply a **stricter, configurable skip policy** for repeated "Pass-only" priority decisions (for example, "skip priority when the only option is Pass", the default). It is **not** allowed to skip a priority decision where a non-pass option exists, because the point of Legacy is that holding up Force of Will is a real choice.

Whatever the policy, every skipped decision is still written to the log with `auto: true`, so the dataset is faithful and a post-mortem can see when the agent was never consulted.

Because the opponent cannot observe whether a decision was auto-answered or deliberated, skipping leaks nothing.

### 5.3 Forking and determinization (what `simulate` runs on)

```rust
pub struct Fork { /* private: State with hidden slots sampled */ }
pub trait BeliefModel { fn sample(&self, view: &ViewState, knowledge: &KnowledgeView, rng: &mut dyn RngCore) -> HiddenAssignment; }
```

Procedure, enforcing "never leak the real hidden state":

1. **Project** the true state to a `ViewState`: the same `State` type, but every slot the observer is not entitled to see is filled with the placeholder `CardId::UNKNOWN`. The opponent's hand, both libraries (except known cards), face-down cards the observer cannot see, and secret choices are placeholders. This is the only place the true state is read, and it is read **only by projection code** that copies allowed fields.
2. **Sample** a `HiddenAssignment` from the `BeliefModel`. Constraints:
   - Opponent hand: known revealed cards fixed; remaining slots drawn from (opponent decklist minus all cards the observer has seen elsewhere), subject to counts.
   - Both libraries: known-position cards fixed; the rest a random permutation of the unseen multiset.
   - Own library: same as above (the observer does not know its own order either).
   The baseline `UniformConsistentModel` samples uniformly over states consistent with the observer's knowledge. Opponent-behavior-aware beliefs (inference from observed plays) are a Phase 3 `BeliefModel`; the trait exists now so nothing else changes.
3. **Fill** the placeholders with the sample and **reseed** the RNG from the caller's seed. The result is a complete, runnable `State`.
4. The fork never contains the true opponent hand or library order, because step 1 never copied them.
5. The `simulate` tool applies the agent's action list to the fork. Opponent decisions inside the fork are produced by an explicit `OppPolicy` (`Pass`, `Random`, `Heuristic`, `Net(ref)`), which acts on the fork's sampled world, never the real one.

Property used by tests: `fork(seat_A, ...)` built from the true state S equals `fork(seat_A, ...)` built from any other true state S' that agrees on everything seat A is entitled to know (with the same seed and model). This is the structural form of non-interference; doc 04 section 7 tests it.

For search (ISMCTS), many forks per decision are cheap because `State` is a flat clone (doc 01 section 4.3). A sampled world can also be reused across a whole rollout.

---

## 6. Knowledge tracking

Determinization needs to know what each player legitimately knows. The engine maintains, per physical card:

```rust
struct CardKnow {
    known_to: u8,           // bitmask of seats that know this card's identity
    pos_known_to: u8,       // bitmask of seats that know its library position
    lib_pos: LibPos,        // valid only while pos_known_to != 0: FromTop(n) or FromBottom(n)
}
```

Update rules (all driven by events, not by engine internals):

| Situation | Update |
|---|---|
| Card becomes public (battlefield, stack, graveyard, face-up exile, revealed) | `known_to = both` |
| Card moves from a public zone to a hidden zone (bounce, return to library) | `known_to` stays both (players saw which card it is); position knowledge only if put on top face-up or the effect says so |
| Draw | Owner gains `known_to`; position knowledge ends |
| Scry/look at top N | Looker gains `known_to` and `pos_known_to` for those cards |
| Brainstorm put-back, Ponder reorder | Owner keeps `known_to` and `pos_known_to` for the placed cards, positions set to the new order |
| Card put on top or bottom of a library with a publicly known identity (Personal Tutor puts a revealed card on top; Mishra's Bauble looks at a target player's top card) | `known_to = both` if the card was public or revealed to both, else only the seats that saw it; `pos_known_to` set to the same seats with `FromTop(0)` or `FromBottom(0)` |
| Cards put on the bottom in **random order** (Atraxa, Raph & Mikey, Thassa's Oracle if its pinned text uses random order) |
| Cards put on the bottom **in a chosen order** (Flow State, Stock Up) | The owner keeps `known_to` and `pos_known_to` with `FromBottom(n)` set to the chosen order; the opponent learns only that cards went to the bottom |
| Face-down pile shuffled and put back on top (Triumph of Saint Katherine's Praesidium Protectiva) | Nobody knows the order; the owner's `known_to` stays for the cards they saw, and a **top-K membership** constraint ("these K cards are the top K of the library, in unknown order") is recorded so the determinizer respects it until a draw, shuffle, or removal breaks it | Identity knowledge as revealed; `pos_known_to` cleared (the order is unknown to everyone) |
| Shuffle | All `pos_known_to` cleared; `known_to` kept (you still know the card is in the library) |
| Tutor/search | Searcher sees the library multiset for the duration; after the effect, `pos_known_to` cleared by the shuffle; if the chosen card is revealed, `known_to = both` |
| Thoughtseize/Duress-style reveal | The caster gains `known_to` for every card in the target's hand at that moment |
| Card enters hand from public zone (reanimate fail, bounce) | Keeps `known_to = both` |
| Hand card played, discarded, or otherwise moves to a public zone | Becomes public |

`Knowledge` is part of `State` (clonable) but is not projected as such; the projection exposes only the observer's `KnowledgeView` (their own `known_to` bits). The opponent's knowledge of the observer's cards is not observable by the observer, with one exception: the observer may know **what the opponent knows about the observer's hand** (for bluffing models) because that is derivable from public events. Phase 3 can request `OppKnowledgeView` if useful; v1 does not expose it.

---

## 7. Event redaction (per-seat `ViewEvent` stream)

Every `Event` (doc 01 section 7.1) is mapped to one `ViewEvent` per seat, or to nothing:

| Event | Owner sees | Opponent sees |
|---|---|---|
| Draw | Card identity | "Opponent drew a card" (count change) |
| Discard | Card (it goes to a public zone) | Same (public) |
| Cast spell, play land, activate ability | Full | Full (public) |
| Target choices and modes | Full | Full (public once on the stack) |
| Search library (tutor) | Candidate set and chosen card | "Opponent searched library" and chosen card only if revealed or put onto a public zone |
| Shuffle | "Library shuffled" | Same |
| Scry / look at top | Cards seen | "Scry N" / "looked at top N" (count only) |
| Reveal | Revealed cards to all named recipients | Same per rules |
| Mulligan, bottoming | Own cards | Count only |
| Brainstorm return | Own cards | "Put 2 cards from hand on top of library" (count only) |
| Coin flip, dice | Public result | Public result |
| Hidden secret choice (Show and Tell) | Own choice | Not shown until revealed simultaneously (section 9.1) |

Event redaction is a pure function `redact(event, seat) -> Option<ViewEvent>` in `mtg-view`, with an exhaustive match on `Event` so adding an event kind forces someone to decide who may see it.

---

## 8. Data classes for logs and datasets

| Class | Contents | May be given to |
|---|---|---|
| `SeatLog` | Per decision: the seat's `Observation`, offered options, chosen action, tool calls and results, LLM reasoning, outcome | That seat's agent; fine-tuning and distillation datasets; post-mortem generators |
| `GodRecord` | Seed, decks, full action list, true outcome. Replays the whole game including hidden zones | Only after the game ends; engineers, differential harness, anomaly detectors, post-mortem generators if the project allows it (below) |
| Public summary | Final outcome, turn count, public events | Anyone |

Fine-tuning a player on a `SeatLog` never exposes the opponent's hand it could not have seen. Training on `GodRecord`s (for example, a value function with privileged information, asymmetric critics) is possible, but those are separate pipelines labeled privileged; any model trained that way MUST NOT be deployed as a player without a distillation step to seat-view inputs.

**Decision for Brady (recommended default: yes):** let the post-mortem LLM see the `GodRecord` after the game ends. Post-mortem findings go into the playbook, which is consulted in future games. The playbook can legitimately say "this deck often holds Force of Will with a blue card", but it MUST NOT contain per-game card identities as if they were known in a future game; the post-mortem prompt should state this, and playbook writes are reviewed by sampling.

---

## 9. Hard hidden-information cases

### 9.1 Simultaneous secret choices (Show and Tell, Stronghold Gambit)

Show and Tell has each player choose a card to put onto the battlefield, and Stronghold Gambit (Reanimator sideboard) has each player choose a card in hand and then reveal it, with the owners of the lowest-mana-value revealed creature cards putting them onto the battlefield. Per the Gatherer rulings (to be confirmed verbatim by the spec-writer agent), the choices are made without the other player seeing the first player's pick. Both cards share the same decision shape; only the follow-up differs. The engine models this as `DecisionKind::SimultaneousSecretChoice`:

- Both players' decisions are issued as a pair. The second decider's `Observation` contains no information about the first decider's pick. The pair resolves only when both have answered.
- In batch/self-play mode, the two answers are collected before either is applied.
- `simulate` handles it like any other pair: the opponent's pick comes from `OppPolicy` against the sampled world.

### 9.2 Own library knowledge (Brainstorm, Ponder, Flow State, Stock Up)

Positions are tracked per section 6. Any effect that reorders the library must update `Knowledge` explicitly; the effect VM op `ReorderLibraryTop(cards, order)` does this itself so card data cannot forget.

### 9.3 Doomsday

The caster's pile is chosen from library+graveyard (five cards, ordered), everything else exiled. The opponent sees: library count, graveyard count, and the exiled cards (exiled face up, so their identities are public). The pile order is known only to the caster. Knowledge rules from section 6 cover this; the card is a hand-written card (doc 03 section 8), since the choice is unusual.

### 9.4 Searching

The searcher sees the unseen multiset of the library as a **sorted** list (by card name), never in true order. "Fail to find" is an allowed choice where the rules allow it.

### 9.5 Bluffing-relevant public information

Hand size, graveyard contents, untapped lands, and mana-pool state are public and exposed. Whether the opponent *could* have a counterspell is the agent's inference problem.

### 9.6 Loop shortcuts (Aluren + Acererak and similar) — OPTIONAL, not in the initial build scope

**Status (2026-10-01):** Brady walked back the idea ("maybe that's not necessary"). This section is a design kept on the shelf, **not** part of the M0-M5 scope: the initial build plays the Aluren + Acererak loop stepwise, bounded by the action budget (doc 01 section 5.5). Nothing else in the design depends on it, and the `ProposeShortcut` and `ShortcutResponse` decision kinds and invariant I18 are reserved names, not required work. Revisit if stepwise play of the loop proves too costly in practice.

**Why.** Aluren plus Acererak the Archlich (and any Aluren chain) repeats the same short sequence many times. Played one decision at a time it is dozens of decisions per iteration, so a game that is effectively over takes hundreds of agent calls, which is a real cost for an LLM player on a subscription budget. The CR already has a mechanism for this: a player with priority proposes a sequence of game choices for all players, which may be a loop repeated a stated number of times; it can't include conditional actions; it must end at a point where a player has priority; each other player in turn order may accept it or shorten it by naming a place where they would act differently (CR 731.2a-c). Loops of only mandatory actions are a draw (CR 731.4, 104.4b), and no player is forced to take an action that would end a loop (CR 731.5, 731.6).

**Design principle: a shortcut is executed, never asserted.** The proposer does not tell the engine the result. The engine replays a recorded body through exactly the same code paths as stepwise play and **stops at the first deviation**, handing control back as if the players had played it by hand. Stopping early is always safe, so the engine never has to prove the loop is valid in advance. (Invariant I18, doc 04: a shortcut and the stepwise replay of the same actions give identical state hashes.)

```rust
pub struct ShortcutProposal {
    pub body: Vec<ActionRef>,     // one iteration: the recorded last iteration, as canonical option references, not indices
    pub iterations: u32,          // N, 1..=max_iterations (match config, default 10_000)
}
// ActionRef = (ActionKind, subject in the proposer's ViewId space or card name, canonical detail), matched against the
// engine's options at each step, so indices that shift between iterations do not matter.
```

**How a proposal comes about.**
1. A **loop detector** watches the proposer's own action log. When the last two periods of the same canonical `ActionRef` sequence repeat, and the **board signature** is unchanged across periods (battlefield, hand and zone multisets, mana pool, stack shape; life totals, counters, venture markers, turn counters and graveyard size are excluded because loops are *about* changing those), the Priority decision gains an option `ProposeShortcut`.
2. Choosing it asks `ChooseNumber` for N (bounded by `max_iterations`). The body is the recorded last iteration. The agent never writes a script, so a small model or an LLM cannot produce an invalid body. (Phase 3 policies and the match layer may submit an explicit `ShortcutProposal`; it goes through the same replay.)
3. Two iterations are played stepwise first; the shortcut removes everything after that.

**Opponent response (CR 731.2b).**
- The engine runs iteration one on an **internal scratch clone of the true state** and records every point in it at which the opponent would hold a non-pass option (their *response windows*). This run never reaches any agent.
- If there are no windows the shortcut is applied with no prompt. If there are, the opponent gets `DecisionKind::ShortcutResponse` with options `Accept`, `StopAtFirstWindow` (act in iteration one, which means no shortcut), and `StopAfter(n)` (via `ChooseNumber`, 1..N).
- To the proposer both paths look identical: the events are `ShortcutProposed`, then `ShortcutAccepted(N)` or `ShortcutShortened(n)`. The proposer only learns that the opponent wants to act, exactly as at a table. (The prompt-or-no-prompt timing must not be observable; auto-answers are issued in the same step.)
- An accepted shortcut binds the opponent to passing at the windows they saw. The engine adds a safety stop: any window that was **not** present at the same step of iteration one (for example, the opponent's life or hand size changed what they can now cast) stops the shortcut at that point and gives them priority.

**Stop conditions (all are normal, none is an error).** The next `ActionRef` has no matching option; any decision arises that is not in the body (for either seat); a new opponent window appears; the game ends (a win mid-loop is simply the game ending, so "loop until the opponent is dead" is a large N); N is reached; a mandatory-only loop is detected (draw per CR 104.4b via the same-state counter). The result is `ShortcutStopped { iterations_done, reason }`.

**Taxed loops (the tax-piece cases).** A cost increase applies to a free cast (CR 118.9d, 601.2f). With Disruptor Flute naming Acererak (+{3}), Defense Grid off-turn, or any other tax (Sphere of Resistance, Damping Sphere and Trinisphere are outside the pool but behave the same), each iteration consumes mana, so (1) the board signature differs between periods and the detector does not flag a loop, and (2) if a shortcut is proposed anyway, it stops at the first iteration whose payment step has no matching option. Which line is right then (finish Tomb of Annihilation so Acererak stays) is the agent's ordinary decision (doc 05 section 2.3), and Tomb's opponent decisions (discard or lose life, sacrifice or lose life) are decisions not in the body, so they stop the shortcut and are played stepwise.

**Hidden information.**
- Validity never depends on the opponent's hidden state except through the opponent's own answer to `ShortcutResponse`. Everything the engine learns from the scratch run stays inside the engine.
- Events the proposer would legitimately see in stepwise play (their own draws, scry and surveil looks) are delivered as usual, but a body that needs a *different* answer at a hidden-information decision stops at that decision, so nothing is learned in advance. A body whose scry or surveil answer is a fixed ref ("put on the bottom") is fine; one whose answer would depend on the card seen is not part of a loop.
- Agent-facing events are coalesced: `ShortcutSummary { iterations_done, net public deltas per seat, own knowledge updates }` replaces N times the per-step events. The full per-step stream goes to the `SeatLog` and `GodRecord` (section 8).
- The non-interference test (section 3 and doc 04) includes shortcut proposals: mutating the opponent's hidden state must not change the proposer's observations except through the opponent's decision events.

**Records.** `GameRecord` stores one `Shortcut { body, iterations_requested, iterations_done, stop_reason }` action. Replay expands it deterministically. The Forge adapter and the differential harness always receive the **stepwise expansion** (Forge has no shortcut feature), so the oracle comparison is unaffected (doc 04 section 5).


---

## 10. Batched and Python API (`mtg-py`)

Needed for Phase 3 and for efficient LLM batching:

- `BatchEnv::new(n_games, db, matchups, seeds)`; `step(actions: &[ActionIdx]) -> StepBatch` advances every game to its next decision; `StepBatch` holds, per game: seat to act, `Observation` handle, option count, done flag, reward. Observation encoding into fixed tensors lives in a separate `mtg-encode` crate that consumes `Observation` only. It cannot read `State`.
- The same API serves LLM players: render `Observation` into the compact prompt (doc 03's glossary, card names only), take an index back.
- Auto-resolution (section 5.2) happens inside `BatchEnv` so Python only sees real decisions.

---

## 11. Review checklist for the frozen API

- [ ] No `Observation` field derives from hidden state or from internal counters.
- [ ] Every `Event` variant has a `redact` rule written by someone other than the variant's author.
- [ ] `ViewId` allocation is order-of-perception only.
- [ ] Option ordering uses only observer-visible keys.
- [ ] `ApplyError` carries no state-derived text.
- [ ] Fork construction reads `State` only through projection.
- [ ] Non-interference test (doc 04 section 7) is green for all pool cards currently implemented.
- [ ] `mtg-debug` is not in any agent build's dependency closure.
