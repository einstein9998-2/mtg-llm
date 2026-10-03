# Milestone log

## M0/M1 (2026-10-01)

Scope: throughput/clone spike plus core skeleton on a hand-built test pool (no DSL, no triggers yet).

Results (details in the thread report):
- `cargo test --workspace`: all green (6 core, 17 engine, 2 non-interference, 1 golden-replay tests; canary test with `--features canary`).
- Invariant fuzz: 30,000 random games, 19.5M decisions, 92M engine steps, invariants checked at every decision, zero violations (release). 1,500 games in a debug-assertions build also clean.
- Non-interference test: 300 seeds x 2 observers x 2 variants; 4 deliberate leaks (canaries) are all caught.
- Golden replays: 12 stored games replay bit-identically (`goldens/`).
- Benchmark (`cargo run --release -p mtg-fuzz --bin bench`, random play, test pool, one core): ~2,600 games/s, ~1.8M decisions/s, clone of a mid-game Game ~2 us.

Not done (by design, later milestones): triggers/APNAP, replacement effects, RON card DSL, differential harness, loop shortcuts (optional).

## M2 (2026-10-01): card DSL, spec runner, full Legacy pool

Scope: IR v2 plus RON card DSL, triggers/replacements/layers (stage 0), casting in full, the spec scenario runner, and every card of the eight decks in `decks/` (144 non-basic cards, plus faces, tokens, emblems).

Results:
- Spec runner (`python3 tools/spec2json.py <spec dir> /tmp/spec.json`, then `specrun /tmp/spec.json`): **803 of 803 visible scenarios pass**, 0 unsupported. The sealed holdout (338 scenarios) is not run here.
- Own scenarios (`own-scenarios/`, run the same way): 9 regression scenarios for what the spec has none for yet (Brazen Borrower Adventure, modal land faces, Ajani both faces, Tamiyo flip, Witch Enchanter). These are implementer-written smoke tests, not independent spec.
- Legacy-pool fuzz (`legacyfuzz decks N check_every seed deep_every deep_sample`): all eight decks, random legal play, invariants every decision, replay and fork checks. 3,000 games with deep checks plus 600 debug-assertion games clean; every main-deck card is cast or entered in an 8,000-game run. A short version runs in `cargo test` (`crates/mtg-fuzz/tests/legacy_fuzz.rs`). Speed is about 150 games/s on one core with checks every decision (invariants cost most of that).
- `cargo test --workspace` green, including the non-interference test.

Bugs the real-pool fuzz found: library to battlefield transitions and library to library (bottom of library) were flagged by invariant I3 though legal; Command-zone objects (emblems, dungeons) were not counted by I1; a stale battlefield length in the planeswalker SBA (index out of range); Lazotep Quarry activatable with no creature card of the chosen mana value; a u32 overflow in the priority-option sort key.

Known deviations and simplifications (all noted in the code):
- Tamiyo, Seasoned Scholar's +2 is applied directly in combat code, not as a stackable trigger.
- Raph & Mikey's PileBack shuffle uses the rng and ignores the scripted-shuffle queue; the new attacking creature copies the source's attack target.
- The Ring's level 3 sacrifice is a delayed trigger at end of combat.
- Faces: a def swap on entering the battlefield and a restore on leaving; Adventure uses cast way 252 and `exile_plays`; modal land play is `Opt::PlayLandBack(card, pay_life)`.
- No sideboarding in games yet; fuzz plays main decks only.

Not done (M3 and later): layers stage 1 (timestamps and dependencies beyond the pool's needs), forks and determinizer for search, core freeze review, loop shortcuts.

## M3 (2026-10-01): forks, determinizer, deps-scan, freeze candidate

- `Game::fork(seat, seed, &dyn BeliefModel)` (mtg-view `fork.rs`, mtg-core `fork.rs`): hidden identities re-sampled from the decklist minus what the observer can identify, unknown library positions shuffled, RNG reseeded, opponent's raw events and sideboard dropped. `UniformConsistentModel` is the baseline belief model. Only valid at the observer's own decision.
- Tests (`crates/mtg-fuzz/tests/fork.rs`): observer's view unchanged by forking, fork playable with invariants, forks of indistinguishable states identical (structural non-interference), seeds matter, inconsistent decklists refused, forks at about every decision (25k) sound. Found and fixed: a hidden Show and Tell pick surviving determinization (an instant could enter the battlefield). New invariant I15 (no instant or sorcery on the battlefield).
- `depsscan` (mtg-cards bin, run in `cargo test`): lists same-layer effect pairs whose footprints overlap; two flagged, both recorded as independent in `deps-reviewed.txt`.
- `legacybench`: clone 2.4 us, fork 10 us, observe 2.8 us on real-pool mid-game states. Random play 500 games/s without checks.
- `ENGINE_CORE_VERSION` 2, goldens regenerated (identical hashes), `rfcs/0000-template.md`, and `CORE-FREEZE-REVIEW.md` with gate table and deviations. The core is a freeze candidate, not frozen: it needs two independent reviewers and Brady's sign-off.

## M3d (2026-10-01): fixes from the rules freeze review and the sealed holdout

- Holdout engine bugs fixed: warp cards are castable from exile after the turn they were exiled (`AllowCastMoved`), Voice of Victory has Mobilize 2 (`TapAttackMoved`, sacrificed at the next end step), a named land is tapped before floating mana is spent. Delve over-exile was a runner gap (the runner now fails on unused delve cards).
- Rules review, blocking: (1) `TriggeredDef.event_cond` marks conditions that belong to the trigger event so they are not re-checked at resolution (Tamiyo, Inquisitive Student); (2) `batch.rs` batches simultaneous zone changes: events are held until the batch ends, leaves-the-battlefield abilities look back at every object that left together (Ajani and his Cat token under Massacre), enters triggers see everything that entered together (Guide of Souls and Ocelot Pride under Hide on the Ceiling), and enters replacements come only from permanents that were already there (Containment Priest returning together with another creature); (3) "can't enter" (Grafdigger's Cage) wins over "exile instead" (614.17c); (4) first and double strikers are recorded when the first-strike step begins; (5) warp recast.
- Review asks on deviations: Tamiyo, Seasoned Scholar +2 is a delayed trigger on the stack (`Effect::WatchAttacks`, `State.attack_watch`) instead of combat code; Raph & Mikey's and Mobilize's new attackers ask which player or planeswalker they attack (`DeclareAttacker` decision during resolution; skipped when only the player is possible); the Sand Warrior token has the types Sand and Warrior; lower-priority deviations are listed in `CORE-FREEZE-REVIEW.md` section 3.
- Also: Doomsday skips the pile choice when every candidate must go in the pile, scripted `bottom_order` random events, runner `choose_attack_target` decision and mana abilities counting for `legal` activate patterns.
- Gates after these changes: see `CORE-FREEZE-REVIEW.md` section 2. Goldens were regenerated twice (card database hash changed because the IR gained fields, and one record's trajectory changed because of batching): this is a core change under doc 01 section 14.2 and is the reason the freeze should wait for the reviewers to re-read `trigger.rs`, `replace.rs`, `sba.rs` and the new `batch.rs`.

## M3e (2026-10-01): second rules re-review items

- Stronghold Gambit batches both placements (reading chosen: one simultaneous event, 614.12). Animate Dead is detached when its return fails (Grafdigger's Cage), so state-based actions put it into the graveyard (704.5m); the deviation note was corrected. Stale `TapAttackMoved` doc comment fixed. Visible spec 806 of 806, own 21 of 21, `cargo test` green. The spec thread reports the sealed holdout at 338 of 338 on M3d.

## M3f (2026-10-01): fixes from the independent hidden-information and determinism reviews

Two read-only reviewers who did not write the code ran probes against the M3e build (`/mnt/project-files/rust-engine-review/`). Their findings, all fixed unless listed under "accepted" below:

- Forks leaked the hidden state: dealing sampled identities to cards by slot let an agent decode the opponent's hand (51 of 51 states) and the whole library order (3,549 of 3,549 positions). Identities are now dealt by position within hand and library, known-identity library cards are re-placed at random positions, the opponent's secret picks (Show and Tell, Stronghold Gambit) are always resampled, and the opponent's private knowledge is reset. Regression tests: `crates/mtg-fuzz/tests/hidden_info.rs`.
- Agents could reach the raw engine (`Game::pending`, `observe` for either seat, state hashes). They are now behind the `diff-harness` feature; an agent holds a `SeatView` bound to one seat (`agent_api.rs` compile-tests this). `Game::fork` requires the observer's own pending decision.
- Decision ids were one global counter, so one seat's id depended on the other's hidden scry choice (6 of 30 opponent scries); ids and view ids are per seat now. Cards offered from hidden zones get ids in definition order, never library order.
- Thassa's Oracle's random bottom order no longer leaves the owner a "known" bottom run, Dig effects reveal to their controller only, Doomsday skips a forced pile choice, the miracle reveal is announced (forced by 702.94a).
- Observation gaps closed: exile, designations (Ring, dungeon, blessing), attack targets, blockers only for live objects, `subject_def` on options that name a hidden-zone card; the encoder grew to 11 blocks and 46 scalars.
- Determinism: `hash_rules` and `hash_full` destructure `State` exhaustively (a new field fails to compile until it is hashed or marked ignored), `free` order, timestamps, `moved`, `cfg` and `scripted` are hashed, records carry their full configuration and strict checkpoints, hashing refuses to build on big-endian targets, slot allocation asserts, RNG stream pinned by a test. `ENGINE_CORE_VERSION` 3, `HASH_SCHEMA` 2.
- New gates: the non-interference harness runs on the real eight-deck pool (6,000 pairs, 0 failures; every legitimate divergence is an explicit, counted stop reason), fork decoders as tests, real-pool goldens replayed against a pinned snapshot of the card text (`goldens-real/`), a poke test that equal hashes imply equal futures.
- Accepted or deferred, for Brady's call: (a) an opponent's conditional decision (an Aether Vial or Show and Tell choice that exists only if its hand holds an eligible card) can differ between two worlds the observer cannot tell apart; the observer never acts during it and the per-seat ids close the id channel, so it is documented rather than changed; (b) the miracle reveal is forced by the rules and a miracle card drawn by the opponent is announced (Triumph of Saint Katherine is sideboard only, so latent); (c) `GameRecord` holds hidden state and is a harness type, documented rather than split; (d) `next_decision` is part of the hash, so a state that returns to an earlier position does not hash equal (no loop detection from hashes); (e) u8 and u16 counters are not overflow-checked (advisory).
- Goldens and the card-database hash changed again (core change). The freeze needs a fresh read of `fork.rs`, `batch.rs`, `trigger.rs`, `replace.rs`, `sba.rs`, the `read.rs` hashing and the `mtg-view` API by the reviewers.

## M3g (2026-10-01): fixes from the spec thread's re-read of M3f

- Miracle regression: M3f's affordability check in `Effect::MiracleCast` also evaluated the miracle alternative cost's condition (Triumph of Saint Katherine uses an always-false condition to keep the card from being cast normally this way), so the miracle was never offered. `way_feasible_inner(.., check_cond)` skips the condition for the miracle only.
- Trigger order (603.3b): `obj_sig` hashed only that an Equipment was attached, not to what, so two Cori-Steel Cutters on different creatures had their flurry triggers ordered by the engine. It now hashes the host.
- Found by the first MCTS self-play run: a random rollout in a Boros mirror grew an Ocelot Pride token loop until the 16-bit object arena was exhausted (an assertion panic). Token creation now declares the game a draw past 60,000 live objects (`RUNAWAY_OBJECTS`); tested by `arena_exhaustion_is_a_draw_not_a_panic`; drivers also survive a panic per game.
- New own scenarios for both (fail before, pass after); visible 806 of 806; limits list gains "forks ignore known library segments".
- Phase 3 hooks (new `mtg-agent` crate, no core change beyond the two fixes above): determinized MCTS with a pluggable `Evaluator` (random rollouts, or a small policy/value net evaluated on the CPU), batched environment server with a Python client, MCTS self-play data dump, torch trainer and expert-iteration loop; see `PHASE3-HANDOFF.md`.

## M3h: first post-freeze change (RFC 0001, differential finding E3)
Lazotep Quarry's "tap, sacrifice a creature: any color" now counts in the castability planner (fallback after plain sources; canonical victim = token first, then cheapest). Core files touched: `legal.rs` only; no state, hash or version change. Gates: workspace suite green (goldens unchanged), spec visible 806/806, own 25/25, 1000-game legacyfuzz 0 violations. Tag `core-frozen-m3h`.

## M3i: RFC 0002 (lockstep findings E6, E7)
Fixed: last-known-information filters now test subtypes, supertypes, keywords, mana value, power and toughness (Ajani, Nacatl Pariah flipped when any permanent left the battlefield), and the payment planner tries 1..n Lazotep Quarries in sacrifice mode. `ENGINE_CORE_VERSION` 4, `HASH_SCHEMA` 3 (Lki is inside hashed trigger captures); all 36 goldens re-recorded with identical action sequences. E4 (Cavern) and E5 (Animate Dead leaves trigger) recorded as accepted-for-now limits in the RFC. Tag `core-frozen-m4`.

## M3j: RFC 0003 (Ajani Avenger runaway loop, found by the first self-play run)
Effect-loop index `u8` wrapped on lists over 255 objects (Cat tokens from Ocelot Pride copying), so Ajani Avenger's +2 never finished and the process died allocating events; combat attacker/blocker indices widened the same way. Added a 4M-instruction resolution cap (draw) and made Quarry auto-payment avoid the spell's own target. `ENGINE_CORE_VERSION` 5, `HASH_SCHEMA` 4, goldens re-recorded (action sequences unchanged). Tag `core-frozen-m5`. Regression: 300-Cat scenario (fails on the old code), Quarry own-target scenario. Stored game records and self-play dumps from before this must be regenerated.
