# RFC 0009: UR Cutter alternate (Class levels, divided damage, Stormchaser's Talent, Pyrokinesis, Price of Progress)

Status: applied to the live engine at Brady's go-ahead (2026-10-08, "start mixing in the stormchaser's talent UR deck"); independent reviewers pending
Author: "Alternate UR Cutter list" thread  Reviewers (two independent, one a rules reviewer): pending

## Problem
Brady's alternate UR Cutter 75 (`decks/ur-cutter-alt.txt`, section 1b of `legacy-deck-set.md`) uses three cards the engine did not define: Stormchaser's Talent (a Class, 4 main), Pyrokinesis (divided damage, 2 sideboard) and Price of Progress (2 sideboard). Everything else in the list was already defined (Boomerang Basics came with RFC 0007). The main UR list `ur-cutter.txt` stays the default UR opponent; the alternate is for LLM vs LLM games and a later comparison.

## Proposed change
`mtg-core` (all enum cases appended; no existing discriminant moves):
1. **Class** (CR 716): `AbilityDef::Class` (marker). A Class enters with a level counter (`CounterKind::Level`, appended after `Lore`), which is its level 1 (`ops.rs`, same place as the Saga's lore counter; `CardDb::has_class`). Level-ups are ordinary sorcery-speed `Activated` abilities with an "activate only if" level condition (`Cmp(Counters(This, Level), Eq, n-1)`, RFC 0007's `cond`), so they use the stack and cannot skip or repeat a level. `EventPat::ClassLevel(n)` ("when this Class becomes level n") mirrors the Saga's `Chapter(n)` and is read in `trigger.rs`; the counter-event prefilter admits `Level`. Level-3's "whenever you cast" ability is a normal `Triggered` ability with an intervening-if level condition.
2. **Divided damage** (Pyrokinesis): each point of damage is one target slot. `TargetSpec.ascending` (a `HashFalseFlag`, hashes like absent when false) makes a slot offer only targets not before the previous slot's target, so a division is offered once and not once per ordering (`cast.rs`). `Effect::DamageSlots { per, first, n }` (`resolve.rs`, `lint.rs`) sums the points per distinct target that is still legal at resolution and deals one damage event per target; a target that became illegal loses its points, and the spell fizzles only if every target is illegal (the existing rule).
3. **Subtypes** Class and Otter appended to `SUBTYPE_NAMES` (116 of 128 used before, 118 after). Like Lesson and Sorcerer in RFC 0007 they also appear among the creature-type choices a Cavern of Souls or Disruptor Flute choice offers (a known wart of the table, not new).
4. **Scenario setup** (`scenario.rs`, test support): a counter listed on a permanent in a scenario now sets the value when the permanent already entered with that counter kind (a Class's level 1, a Saga's lore counter) instead of adding a second entry. No older scenario depended on the old behaviour (visible spec unchanged).
5. **Spec adapter** (`mtg-spec`, test-only): counter kind `level`; a cast action may give `divide: {"@a": 3, "@b": 1}` and the adapter answers the target choices in the engine's ascending order.

Card data: `crates/mtg-cards/cards/ur.cards.ron` (fourth card source in `legacy.rs`): Stormchaser's Talent, Otter Token (1/1 blue and red, prowess), Pyrokinesis (alternative cost: exile a red card from hand), Price of Progress (plain data: twice the nonbasic lands each player controls, damage to each player).

## Documented limits (not modelled)
- Pyrokinesis needs at least one target creature to be cast ("any number", including zero, is not modelled) and always divides all 4 points. A creature that takes k points is the target of k slots, so "becomes the target" triggers (ward-style) fire once per point.
- The Class level is a counter named `level` in the hash and the views; counter removal or proliferate on a Class would change the level (none in the pool).
- Level 3's "creates an Otter" has no cap; Otters from Aluren/Omniscience free casts count as casts (they are).

## Rules basis
Oracle text from Savecraft card search (Scryfall), 2026-10-07. CR 716 (Classes): 716.2 level abilities and level-up as a sorcery-speed activated ability, 716.3 a Class enters at level 1. 601.2d divided damage is announced when the spell is cast. 702.108 prowess. Scenario writers' verified CR excerpts: `cr-excerpts-ur-A.md`, `cr-excerpts-ur-B.md`.

## Impact
- State layout and hashes: no new state field (`counters` already hashed). `TargetSpec` gains one field and `Effect`, `EventPat`, `AbilityDef` and `CounterKind` gain appended cases; the field hashes like absent when false, so the 8 older decks' definitions hash as before. The card database hash and `n_defs` change because cards were added: nets and game records made against the previous live pool need regenerating (as after RFC 0007/0008).
- Determinism and golden replays: test-pool and real-pool goldens replay bit-identically (see results).
- Hidden information: none new.
- `ENGINE_CORE_VERSION` bump: no (goldens bit-identical).

## Test plan and results
Scenarios were written by two separate agents from Oracle text and the CR only (the implementer did not write or see them first). Defects found in them (wrong summoning-sickness expectation, a harness-unevaluable legal pattern, creatures missing from the closed pool, an unresolved prowess trigger) were sent back to their writers.
- 54 UR scenarios (`rust-engine-ur/scenarios/`): group A 25 (Stormchaser's Talent and the Otter), group B 29 (Pyrokinesis 12, Price of Progress 9, Boomerang Basics 8). All pass.
- Visible spec plus the Chant, Tron, Storm and UR packages: 1229 / 1229 (1175 before; the older ones did not move).
- `cargo test --release --workspace --no-fail-fast`: 83 passed, 0 failed, test-pool and real-pool goldens bit-identical (no `ENGINE_CORE_VERSION` bump).
- Fuzz (`legacyfuzz`, invariants and fork/replay checks at every decision, deep checks every 100): 600 games over six decks and 3000 games over the ten decks without the two Storm decks, with `ur-cutter-alt` in the pool: 0 violations, every deck card cast or entered.
- **Unrelated, existing bug found by the fuzz** (also reproduces on the unmodified live engine): with the Storm decks included, `storm v alurentell` seed 103 and `alurentell v storm-tes-alt` seed 6392 stop with `I3 illegal zone transition Hand -> Hand` (a Storm card moving from hand to hand). Not touched here; it needs its own fix.
- Not run: the sealed holdout and Forge differential testing (the three new cards are not in the holdout).
