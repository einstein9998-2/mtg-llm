# Determinism review of the Rust engine freeze candidate

Reviewer: independent determinism reviewer (did not write this code), 2026-10-01.
Candidate: `/home/claude/rust-engine` at git HEAD `f78f92c`, `ENGINE_CORE_VERSION = 2`.
Scope: `rng.rs`, hashing (`read.rs`), record/replay (`mtg-view/record.rs`), option ordering and collapsing (`priority.rs`, `legal.rs`, `resolve.rs::card_options`, `stabilize.rs`, `trigger.rs`, `combat.rs`), HashMap/sort/float/platform use, clone/fork independence, fork and determinize seeds, trigger ordering, `batch.rs`, slot reuse and generations, runaway loops.

## Verdict: APPROVE WITH CONDITIONS (from the determinism side)

The engine is deterministic in the sense that matters for replay: the same (card DB, decks, seed, config, action list) gave bit-identical state hashes in every run I could construct (across processes, build profiles, thread stack sizes, clone points, fork points). I found no HashMap/HashSet iteration, no unstable sort, no floats or `unsafe` in `mtg-core`, no pointer or time dependence, and no shared mutable state between clones.

What I found is not nondeterminism in the rules machinery. It is that the contract around it is weaker than the freeze documents claim, and several of those gaps are in parts that freeze (hash definition, record format, enumerator option sets, view API). Fixing them after the freeze needs an RFC and a golden regeneration, so they should be fixed first. Conditions, in order:

1. Fix the hash coverage (finding 1) and make it structurally impossible to forget a field.
2. Fix `GameRecord`/`replay` (finding 2) and bump `ENGINE_CORE_VERSION` as part of the freeze commit.
3. Fix the unsound collapse in `card_options` and align it with `obj_sig` (finding 3).
4. Either add real-pool goldens or record the explicit decision that the real pool has no core-drift guard (finding 4).
5. Fix `view_id` on stale refs (finding 5) and put a hard cap on object slots (finding 6).

Findings 7 to 9 are advisory and can follow the freeze as non-core changes, except that the toolchain pin in finding 7 should go in with the freeze commit.

Note on the tree: while I was working, `crates/mtg-core/src/fork.rs` in `/home/claude/rust-engine` was modified in the working tree (uncommitted, marked "REVIEW PROTOTYPE", position-based dealing in `determinize`) by someone other than me. All my results are against the committed HEAD. If that prototype is adopted, the fork probes (section "Checked and correct", fork bullet) must be rerun.

## Findings, ordered by severity

### 1. `hash_rules`/`hash_full` omit state that changes future play (Medium-High)

`crates/mtg-core/src/read.rs:299-333` (`hash_rules`), `crates/mtg-core/src/state.rs:341-389` (fields).

Not hashed although they feed later behaviour:

| Field | Where | Effect |
|---|---|---|
| `moved` (`state.rs:365`) | `resolve.rs:1759-1762` pushes the player's `KeepOne` choice into it across decisions; `resolve.rs:878` reads it | Different choices, same hash |
| `free` order (`state.rs:341`; only `free.len()` is hashed, `read.rs:302`) | `alloc_slot` pops the last entry (`state.rs:611`) | Next token/ability gets a different slot, so a different `ObjRef` in every later hash |
| `ts_counter` (`state.rs:387`) | `next_ts()` | Next timestamp differs |
| `scripted` (`state.rs:359`) | shuffle outcomes (`ops.rs:395`, `:420`) | Scenario-only |
| `batch` (`state.rs:371`) | | Empty at every decision boundary today (all `begin_batch`/`end_batch` pairs are synchronous), but nothing asserts it |
| `cfg` (`state.rs:389`) | hand size, life, explicit mana | Constant after construction; see finding 2 |

Concrete failure (probe `keepone`): Ajani, Nacatl Avenger -4 against an opponent with Disruptor Flute, Cori-Steel Cutter, Containment Priest, Phelia. The opponent's first pick (Cutter or Flute) is stored only in `moved`. Two different first picks give the **same `hash_full`** (`bfc2ff2e54e7de7e`) at the second `KeepOne` decision, whose options are identical, and different final boards (survivors `Cutter` versus `Flute`).

Statistical probe (`hole`, 772 mid-game states in 40 real-deck games, state cloned, one field poked, both copies fed identical actions for up to 400 decisions):

| Field poked | Hash still equal | Diverged later |
|---|---|---|
| free-list order | 356 of 356 | 345 |
| `ts_counter` | 772 of 772 | 769 |
| `moved` | 712 of 712 | 0 (only `KeepOne` reads it across a decision) |

Consequences: replay checkpoints still catch a divergence, but only after it surfaces. Transposition tables, the "same-state-hash repeat counter" of doc 01 section 5.5, fork-consistency invariants and any state-equality use are unsound. Separately, `hash_rules` includes `next_decision` (`read.rs:330`), `Obj.gen` and `Obj.timestamp`, which only ever grow, so `hash_full` can never repeat within a game and cannot serve for loop detection as doc 01 section 4.4 says. It is a history fingerprint, not a state fingerprint.

Fix:
- Destructure `State` exhaustively in `hash_rules` (`let State { objs, free, moved, .. } = self;` listing every field, scratch fields bound to `_`) so a new field is a compile error.
- Hash `free` in order, `ts_counter`, `moved`, `scripted`, `cfg`; `debug_assert!(batch.depth == 0)` in `ask()`.
- Add a test: `hash_full(a) == hash_full(b)` implies identical hashes after the same actions (my `hole` probe is a template).
- If loop detection or transposition is wanted, add a separate `hash_canonical` that drops `next_decision`, generations, absolute timestamps, view ids and slot numbering.

### 2. Records and `replay` can silently verify less than they claim (Medium)

`crates/mtg-view/src/record.rs:12` (`GameRecord`), `:109-141` (`replay`), `crates/mtg-core/src/lib.rs:44`.

(a) The record stores only `first`, not the rest of `GameConfig` (`starting_life`, `hand_size`, `explicit_mana`); `replay` rebuilds `GameConfig { first_player, ..default }` (`record.rs:118`). Probe `tamper`: a game recorded with `starting_life = 25` fails at checkpoint 0 (`CheckpointMismatch`); one recorded with `explicit_mana = true` fails at action 139 (`Apply BadIndex`). Non-default games cannot be stored as training data or bug repros.

(b) Checkpoint handling (`record.rs:123-141`) pairs checkpoints with actions by `n == i` using `peek()`. Anything not in ascending order, or with `n` beyond the action list, is silently ignored; a stuck early entry shadows all later ones; only entries with `n == actions.len()` are checked at the end. Probe results, all returning `Ok`: a bogus checkpoint past the end; the same correct checkpoints in reversed order (no intermediate checkpoint verified); a corrupted mid-game checkpoint when the final one is listed first; a record with no checkpoints at all.

(c) `ENGINE_CORE_VERSION` was set to 2 in `f2aa741` and not bumped by the two later commits that changed the hash definition (`7d4be58`: `attack_watch` added, every golden's checkpoint 0 changed) or behaviour (`03a9181`: batching, goldens g04 and g19 trajectories changed). A version-2 record from before those commits now fails as `CheckpointMismatch`, indistinguishable from corruption. Doc 01 section 14.2 item 5 says the version bumps on any reviewed core change.

Fix:
- Serialize the full `GameConfig` in the record (and in `from_text`/`to_text`).
- `replay` should `Err` if any checkpoint is out of order, beyond the end, or unconsumed, and require at least the final checkpoint.
- Bump to 3 in the freeze commit and add a separate `hash_schema` number to the record so a hash-definition change is distinguishable from a rules change.

### 3. Option collapsing is unsound in `card_options`, and `obj_sig` and trigger interchangeability are incomplete (Medium)

`crates/mtg-core/src/resolve.rs:1683-1700` (`card_options`, the `same` closure), `legal.rs:476-486` (`obj_sig`), `trigger.rs:354-368` (`interchangeable`).

`card_options` is what the player sees for edicts (`SacrificeChosen`: Archon of Cruelty, Acererak), `SacrificeOneEach`, `KeepOneOfEach`, discard-to-hand-size and put-back choices. For permanents it merges candidates that agree on def, tapped, damage, counters, controller, attached flag, ring-bearer, entered-this-turn and `chosen`. It ignores derived characteristics, `links`, `until_links`, `exile_plays` and delayed sources (all keyed by `ObjRef`).

Concrete failures (probe `collapse`, run through the real `card_options`):
- Two Containment Priests on one battlefield, one with a +2/+2 effect (2/2 versus 4/4): the edict offers **1** option.
- Two Skyclave Apparitions linked to a mana value 4 and a mana value 1 exiled card: the edict offers **1** option (the older one, which returns the larger token). bw-death-and-taxes runs two Apparitions; reanimator runs Archon of Cruelty; ur-cutter makes Monk tokens that Cori-Steel Cutter may equip. The sacrificing player cannot choose.

`obj_sig` (used by `pick_candidates`, activation collapse, attacker collapse via `declined_sig`) does hash derived characteristics, so the first case is covered there, but it omits `chosen` (Cavern of Souls: two Caverns naming different creature types collapse in `mana_ability_options`; latent, decks have one Cavern) and the `ObjRef`-keyed links, so the second case still fails for sacrifice-cost picks. `interchangeable` ignores derived characteristics, `chosen` and links for pending triggers.

Smaller: the graveyard collapse in `priority_options` (`priority.rs:76-90`) picks the copy with the lowest view id, which changes which position of the graveyard (public, ordered) a flashback copy leaves from. No pool card reads graveyard order, so this is cosmetic today.

None of this makes the engine nondeterministic; it changes the option sets the enumerator contract exposes, which freeze.

Fix: one shared `interchangeable(a, b)` predicate used by `card_options`, `pick_candidates`, activation, attackers and triggers, built from `obj_sig` plus every `ObjRef`-keyed table and `chosen`. Add a soundness test: in a test build, generate the uncollapsed option list at every fuzz decision and check that options in one class lead to equal canonical hashes (up to slot renaming) after applying and advancing.

### 4. Goldens cover only the hand-built test pool (Medium)

`goldens/g01..g20.rec` (all `db 2beab2691be4b09a`, the test pool), `crates/mtg-fuzz/tests/goldens.rs`, `crates/mtg-fuzz/tests/legacy_fuzz.rs`.

The real pool (174 cards, batching, APNAP, layers, Show and Tell, planeswalkers) has no stored replay. `legacy_fuzz` only does same-process `record_and_replay`, which cannot catch drift. The M3d regenerations show the guard working for the test pool and also show how much of the new core behaviour is only exercised through random real-pool fuzz. Card data is deliberately not frozen, so pinning real-pool goldens to the live RON would break on every card edit.

Fix: check in a pinned snapshot of the RON text plus one record per deck (8 or more), checkpoint at every decision, and run them against the snapshot, not the live data. Core drift then fails the golden; card edits do not. The probes `rec`/`rep` produce exactly this format (8 decks, 400 games, 8.2 MB at every-action checkpoints; use a handful).

### 5. Agent-visible output depends on slot reuse through stale `ObjRef` lookups (Medium-Low)

`crates/mtg-core/src/read.rs:169` (`view_id` ignores `gen`), `crates/mtg-view/src/project.rs:242` (`blocking`), `:202` (OrderTriggers label).

`combat.attackers` keeps attackers that died or left. `observe` computes a blocker's `blocking` field as `view_id(a.obj)` for that stale attacker, which returns the view id of whatever now occupies the slot. Perturbing only the order in which freed slots are reused (probe `lock`, FIFO free list) changed the observation in 25 of 3000 real-deck games with the same actions, for example seed 5200 (decks 0v5) decision 658: Griselbrand shows `blocking: Some(ViewId(162))` (the id of a Griselbrand ability object that took the dead attacker's slot) in one run and `Some(ViewId(0))` in the other; with a rotated free list plus padded slots 18 of 3000. So the observation is not a function of the observer's knowledge alone, and it can hand the agent a wrong object id. The OrderTriggers label has the same defect for dead trigger sources (55 stale lookups in 800 games for sources that left; for token sources the id belongs to another object).

After patching `blocking` to check liveness, 6000 games each under FIFO and under rotated-plus-padded free lists produced zero observation differences, so this is the only slot-order dependence I found in the agent-visible surface.

Fix: `view_id` returns `ViewId::NONE` unless the ref is live (check `objs[slot].gen == gen`); drop dead attackers/blockers from `combat` when they leave play, or filter at projection.

### 6. Unbounded object allocation wraps `u16` silently; no in-core budget (Medium-Low)

`crates/mtg-core/src/state.rs:617` (`let slot = self.objs.len() as u16`), `crates/mtg-core/src/engine.rs:21-32` (`advance` loops without a bound; `steps` is counted but never checked).

Legal play can loop forever: a deterministic "always choose the middle option" policy never finished 32 of 160 real-deck games within 60000 decisions (Aluren plus Acererak the Archlich; probe `pol mid`). In seed 1, decks 0v1 (probe `slots`), the stack grows by about one ability object per iteration and the arena reached 65 558 slots after roughly 370 000 decisions. Past 65 535 the next object gets `ObjRef { slot: len as u16 }`, which aliases a physical card. Probe `alloc`: after 65 544 allocations the new object's `ObjRef.slot` is 7 and writing to it set card slot 7 to `Stack` (it was `Library`). This is silent state corruption, in debug and release alike (`as` does not panic). It is deterministic, but it destroys the rules model rather than failing cleanly. Doc 01 section 5.5 puts budgets in the runner; the core has no cap or hook, and `Truncated` is not an engine outcome.

Fix: `assert!(self.objs.len() < u16::MAX as usize)` (or return a deterministic `Status::Truncated`) in `alloc_slot`; add `max_objects`/`max_decisions` to `GameConfig`; note the guard in the freeze review.

### 7. Hash portability and toolchain dependence; the doc 01 section 12 lint does not exist (Low-Medium)

`crates/mtg-core/src/hash.rs`, `state.rs` (`TurnState.own_turn: [u16;2]`, `State.next_vid: [u32;2]`), `frame.rs` (`ResolveFrame.sc: [u16;3]`).

- `Fx64::write` decodes bytes little-endian (good), but `Hash` for arrays and slices of `u16`/`u32` calls `write` with the raw native-endian bytes (I showed `[1u16,2u16]` arrives as `01 00 02 00`). On a big-endian target the three fields above hash differently, so goldens would not be portable. Scalars are fine (`write_usize` mixes `as u64`).
- State and DB hashes rely on `#[derive(Hash)]` expansion and std's `Hash` impls, which Rust does not promise stable across compiler versions. There is no `rust-toolchain.toml`, no `rust-version`, and no CI config in the repo, so a compiler bump could invalidate every golden with no engine change.
- Doc 01 section 12 says the HashMap/float ban is "lint-enforced". There is no `clippy.toml` and no CI. Today there is no HashMap or float in `mtg-core`; nothing prevents one tomorrow. (`mtg-spec` runner uses HashMap; not frozen, not reviewed in depth, its `values_mut()` use looks order-independent.)

Fix: add `rust-toolchain.toml`; either route integer slices through per-element `write_u16` (wrap in newtypes) or hand-write a `StableHash` trait; add `clippy.toml` with `disallowed-types` for `HashMap`/`HashSet` and `disallowed-methods` for float math in `mtg-core`; add golden values for `below` and `shuffle` to `rng.rs` (only `next_u64` is pinned today; a change to `below` is caught only by the replay goldens).

### 8. Minor and latent (Low)

- `Opt::Choice(i as u8)` for OrderTriggers (`stabilize.rs:130`) truncates past 255 pending triggers.
- Mana option sort key (`priority.rs:148`) overlaps fields once a view id exceeds 65 535; order stays deterministic (stable sort, input order), only the intended key is lost.
- Generation counters are `u16` and wrap (`ops.rs:192`, `state.rs:612`, `:630`). Random play reaches 230 (probe `gens`), so headroom is large, but a stale ref aliases after 65 536 zone changes of one object.
- `turn.number` is `u16` (`turn.rs:45`) and `begin_batch` depth is `u8` (`batch.rs:13`, `:34`); both wrap in release and panic in debug. Not reachable in normal play.
- `derive.rs:107` breaks timestamp ties by slot number. In 800 games (about 556 000 decisions) no two statics shared a timestamp, so it is dead code today; scenario setup with hand-set timestamps is the only way to make it matter.
- `LinkedToken` identifies "the leaves trigger's source" by `gen + 1` arithmetic (`resolve.rs:1476`); if the card moves again before the trigger resolves it will not match. Rules rather than determinism.
- `ability_uses_event` (`trigger.rs:342`) tests `format!("{:?}")` of IR for substrings. Deterministic, but it couples trigger collapse to `Debug` formatting and is slow.

### 9. Noticed, outside determinism (for the rules reviewer)

`stabilize.rs:116-135`: triggers that arise while earlier ones are still being placed (for example `BecameTarget` from `push_ability`) are appended to `pending_triggers` and taken immediately if their controller is the active player, ahead of the non-active player's older pending triggers (CR 603.3b says the older ones go first). Deterministic.

## What I checked and found correct

Method: probe crate and a scratch copy of the workspace (patch in `determinism-probes/`), real decks from `/mnt/project-files/decks` (8 decks, all pairings), random-legal-play agent driven by its own `Pcg64`. All counts below are real-deck games unless stated.

- **Same seed, repeated, across processes.** 2000 games (seeds 5000 to 6999) run in two separate processes and in a 128 KB-stack thread: identical state-hash and observation digests, game for game. Stack sizes of 16, 32 and 64 KB also complete and match (no deep recursion anywhere).
- **Debug versus release.** 1800 games (including overflow checks and `debug_assert!`) match release digests exactly; no debug assertion fired.
- **Record then replay, other process.** 400 games recorded with a checkpoint after every action (8.2 MB), replayed by a separate release process and by the debug build: 400 of 400 clean. The 20 stored goldens replay on the pristine candidate (`cargo test --release -p mtg-fuzz --test goldens`).
- **Clone at every decision then continue.** 40 games, a clone at every decision continued to game end with the same actions: 31 805 clones, 0 divergences. Further 3 x 19 363 clones (every 11th decision of 300 games) with three variants (plain; derived cache forcibly invalidated on the clone; `observe` called on both seats at every step on the clone): 0 divergences. Cloning shares no mutable state: `State` and `Game` contain no `Rc`/`Arc`/`Cell`/`RefCell` (the `Arc<CardDb>` is immutable and `CardDb` has no interior mutability); `canary` is a global atomic that is compiled to constant `false` without the feature.
- **Derived cache.** Never dirty at any of 211 234 decision points; observation equals the observation from a force-refreshed clone.
- **Forks.** 2181 forks (every 19th decision of 60 games), built twice, once from a clone of the game, and once with a different seed: same-seed forks have equal hashes and play out identically to game end; the original's hash is unchanged; different seeds never produced equal hashes; the digest of all fork hashes is identical across two processes. Seeds are separated by distinct XOR constants for model sampling (`mtg-view/fork.rs`), library permutation (`fork.rs:~96`), secret-pick resampling and the reseeded game RNG.
- **Option order independent of slot numbers.** Lock-step runs of the same game under the unmodified engine and under (a) FIFO free-list reuse, (b) rotated free-list reuse plus padded dummy slots that shift every new object's slot number: 6000 games each, every observation (all options, labels, view ids, events) identical at every decision, after the finding 5 patch. Before the patch the only difference class was finding 5.
- **No HashMap/HashSet in `mtg-core`, `mtg-view`, `mtg-dsl`, `mtg-cards`.** Only `BTreeMap`/`BTreeSet`. Every `sort_by_key`/`sort` is the stable sort with a total key (no `sort_unstable`); the keys are `(def, view id)` or include a unique tag. No code iterates the object arena in slot order except `fork.rs` (intentionally). Mana-source order, trigger scan order (zone list, then battlefield timestamp order, then departed list) and `legal_targets` order are all state-derived, not address- or slot-derived.
- **Floats.** Only `mtg-view/encode.rs` (f32 features) and `sim.rs` (f64 score); neither feeds back into rules. `#![forbid(unsafe_code)]` in `mtg-core`.
- **RNG.** PCG-XSL-RR 128/64 with the standard multiplier; step-then-output matches the reference construction; Lemire `below` with `n.wrapping_neg() % n` rejection is correct; `shuffle` is Fisher-Yates from the end; SplitMix64 seeding; `stream_is_stable` golden test exists. RNG is consumed only by library shuffles, bottom-order shuffles and one top-of-library shuffle; draw counts depend only on collection sizes.
- **Trigger ordering.** APNAP loop (`stabilize.rs:116`) places the active player's triggers first and the other player's after; ordering decisions list one representative per interchangeable class in pending order; resolution is deterministic. Batch events are held and matched in emission order, `departed` is appended in departure order, `pre_bf` is the battlefield snapshot; all batches are synchronous so none is live at a decision. Free-list reuse is LIFO with a double generation bump (event ref, then free, then alloc) so a stale event ref never matches a reused slot.
- **Degenerate policies.** Always-first, always-last and alternating policies finished all 160 games each (longest 3736 decisions). The one non-terminating policy is finding 6.
- **Existing fuzz baseline** (legacyfuzz, 200 games, invariants at every decision, deep checks on a sample): 0 violations.

## What I could not check

- Big-endian and 32-bit targets, other OSes, other rustc versions: only x86-64 Linux with one toolchain was available. The endianness claim in finding 7 is by construction (shown on the std `Hash` for `[u16;2]`), not by running a big-endian build.
- Cross-compiler stability of derive(Hash) expansion: not testable here.
- `resolve.rs`, `libfx.rs`, `cost.rs`, `eval.rs`, `restrict.rs`, `dungeon.rs` were read selectively (the parts that touch ordering, batching, free lists, `moved`, search and collapse), not line by line. The dynamic probes cover them only as far as random and degenerate play reach.
- The non-interference suite (38 s), the spec runner and holdout, `cargo test --workspace` as a whole, and hidden-information correctness. I ran only the goldens test on the pristine tree plus my own probes.
- Whether any pool card depends on the replacement-order and graveyard-order choices noted in findings 3 and 9; that belongs to the rules reviewer.
- Behaviour with a concurrent runner (threads over many `Game`s). Each `Game` is independent by construction and I ran processes in parallel, but I did not stress a multi-threaded batch runner.

## Reproducing

`determinism-probes/scratch-changes.patch` is a diff against HEAD `f78f92c` (adds a `mtg-probe` crate, a free-list perturbation switch, test pokes in `read.rs`, and the one-line `blocking` liveness fix). `determinism-probes/mtg-probe-main.rs` is the probe source. In a copy of the workspace: apply the patch, `cargo build --release -p mtg-probe --offline`, then for example `mtg-probe traj 5000 2000 0`, `mtg-probe lock 7000 3000 1`, `mtg-probe clone 70000 40 1 0`, `mtg-probe fork 100 60 19`, `mtg-probe rec 60000 400 DIR` then `mtg-probe rep DIR`, `mtg-probe hole 300 40`, `mtg-probe keepone`, `mtg-probe collapse`, `mtg-probe tamper 11`, `mtg-probe pol 1 160 mid`, `mtg-probe slots 1 0 1 400000`, `mtg-probe alloc`. The free-list perturbation is off by default (`PERTURB_MODE` 0).
