# Hidden-information review of the Rust engine freeze candidate

Reviewer: independent hidden-information reviewer (did not write the code). Date: 2026-10-01. Candidate: `/home/claude/rust-engine`, `ENGINE_CORE_VERSION = 2`, M3e. Doc 01 section 14.2 gate. Read-only review: nothing under `/home/claude/rust-engine` or `rust-engine-spec` was edited; all probes ran on a copy (`/tmp/review-hidden/ws`, own target dir). Probe sources and a prototype fix are in `hidden-info-probes/` next to this file (README.txt there explains how to rerun).

## Verdict: DO NOT FREEZE from the hidden-information side

Not "approve with conditions": the doc 02 guarantee ("two worlds that differ only in hidden state give the observer identical forks", section 5.3) is false for `Game::fork`, and the false property is exactly the one the gate table cites as passing. A fork built with an agent-chosen seed lets the agent recover the opponent's true hand and the true order of both libraries. All existing gates are green (I reran `mtg-fuzz --test fork` and `mtg-debug` noninterference/canary: pass) because none of them can see this class of leak.

The view/fork API types are part of what would be frozen, so findings 1 to 4 and 6 must be fixed before the freeze, not through an RFC afterwards. Conditions to flip to "approve": findings 1, 2, 3 fixed with regression tests that fail on the current tree; finding 4 fixed or explicitly accepted by Brady; finding 5 gate extended; finding 6 decided (Observation fields are RFC-level after the freeze).

Summary table

| # | Severity | What |
|---|---|---|
| 1 | Critical | Fork keeps the true object/slot structure: opponent hand and both library orders are recoverable from forks |
| 2 | High | Show and Tell: fork reveals exactly whether the opponent's secret pick was "nothing" |
| 3 | High | No seat-bound API: raw `Game::pending()` (object slots), any-seat `observe`, `state_hash`, raw events, `Policy::choose(&Game)` |
| 4 | Medium | `DecisionId` and pending-seat timing leak hidden choices and hidden-card existence (scry, Aether Vial, Show and Tell) |
| 5 | Medium | The non-interference gate cannot catch 1 to 4 (test pool only, permutes defs not slots, stops at opponent choices); on the real pool it reports divergences |
| 6 | Medium | Observation/option completeness gaps in the frozen API (no exile, no attack target, library-card options carry no identity to the encoder) |
| 7 | Low | Fork leftovers: opponent-private knowledge bits, fork allowed with no pending decision, Gambit pick not resampled (latent) |
| 8 | Low | `revealed_hand` order and view ids follow the opponent's true hand order |
| 9 | Low | Thassa's Oracle leaves stale `pos_known_to`: owner is shown the true "random" bottom order |
| 10 | Low | Miracle reveal is forced; offering the miracle cast does not check affordability (engine panic, latent) |
| 11 | Low | Under-reveal: revealed cards the revealer or the opponent is never told about (fork unsound, not a leak) |
| 12 | Low | `Game::apply` resets the event mark before validating; `GameRecord` is an unsealed god record |

---

## Findings

### 1. Critical: `determinize` preserves true slot structure; forks reveal the opponent's hand and both library orders

Where: `crates/mtg-core/src/fork.rs:75-104` (`determinize`), with `state.rs` (~line 420, `CardId(slot)` and `def: d` assigned in decklist order) and `ids.rs` (CardId "equal to the arena slot").

What happens: sampled identities are dealt to unknown objects in ascending slot order (`fork.rs:83`, `unknown[p].iter().zip(asg.defs[p])`), and unknown library positions are filled by `rng.shuffle` of the objects in their TRUE order (`fork.rs:98-104`). So the fork is `f(true structure, seed)` where "true structure" is which slot is in the hand, which slot is at which library position, and slot index equals decklist index equals true identity. The agent chooses the seed and the belief model, so it can replay both (`Pcg64` and `UniformConsistentModel` are public) and invert them. Doc 02 section 5.3 requires fork(S) == fork(S') for observer-indistinguishable S, S'; here they differ for every pair.

Concrete failing scenario (all probes in `review_hidden.rs`, original code, real 8-deck pool, random mid-game states at the observer's own decision):
- `poc_fork_decodes_opponent_hand`: using only the decklists in the order given to `Game::new`, the observer's own `HiddenRequest` (derivable from its observation), the model it supplied, and `fork.observe(opp).me.hand` over 48 seeds, the opponent's exact hand was recovered in 51 of 51 states (197 of 197 cards; random-guess baseline 46).
- `poc_fork_decodes_library_order`: same inputs plus reading the fork's library (an agent gets that by simulating draws): 3549 of 3549 opponent library positions recovered, including all of the top five (345 of 345), in 69 states. The same works for the observer's own library, which doc 02 section 1 also calls hidden.
- `poc_forks_differ_for_library_order_twins`: take a state and shuffle only the unknown order of the opponent's library (a legitimately possible other world, observer view byte-identical): fork state hashes differ in 119 of 119 pairs, and the sampled library def order differs in 119 of 119.

Why the project's tests miss it: `rerandomize_hidden` (`read.rs:237`) permutes defs among slots and keeps every slot in place; `forks_of_indistinguishable_states_are_identical` (`crates/mtg-fuzz/tests/fork.rs`) uses it, and `determinize` overwrites defs by slot anyway, so the test passes by construction. No test permutes which slot is in the hand or where it sits in the library.

Suggested fix:
1. Deal sampled identities by position within zone, never by slot: unknown hand objects in hand order, then unknown library objects in position order. Known-identity library objects whose position the observer does not know must be re-placed at RANDOM free positions (rng), not shuffled together with the true order. Nothing in the result may be a function of which slot was where.
2. Also reset opponent-only knowledge bits that the observer cannot know (finding 7).
3. Regression tests: (a) a twin generator that permutes slots among hand and library positions and library order (`review_shuffle_unknown_order` in `hidden-info-probes/read-probe-twin.patch` is a start), requiring identical def-by-position and identical observations in the fork; (b) keep my two decoders as tests that must score at chance.
4. Prototype: `hidden-info-probes/fork-position-dealing-prototype.patch` (about 40 lines). With it: library decode 0 of 3549, hand decode 12 of 197 (baseline 46), sampled library def order identical in 119 of 119 twins, and the six tests in `fork.rs` still pass. It is a prototype, not a reviewed fix; full `state_hash` equality between twins still differs because slot numbers differ, so the test must compare def-by-position and observations, or the fork must renumber slots canonically.
5. This only closes the observation-level channel. The slot numbers themselves stay readable through `Game::pending()` (finding 3), so 1 and 3 must land together.

### 2. High: Show and Tell fork reveals whether the opponent's secret pick was "nothing"

Where: `crates/mtg-core/src/fork.rs:118-150` (`resample_secret_picks`), condition at `fork.rs:125`: `f.aux == 1 && f.sc[0] != 0 && self.turn.active != observer`. Related: `resolve.rs:1377-1394` (`EachPutFromHand`, empty candidate list skips the decision).

What happens: the first chooser's pick is resampled only when it is non-zero. If the opponent picked nothing (declined, or had no eligible card), `sc[0] == 0` and the fork keeps "nothing" with certainty. When the pick was something, the resample gives "nothing" with probability 1/(candidates+1). So the fork distribution depends on the secret choice, which doc 02 section 1 names as a protected secret and section 9.1 requires the second chooser not to learn.

Concrete failing scenario (`poc_show_and_tell_fork_depends_on_true_pick`, 700 games, Alurentell vs each deck, observer is the second chooser at the Show and Tell decision, 608 such states, 24 forks each, observer answers "Done" in truth and in each fork): P(fork has the opponent put something onto the battlefield | truth: opponent put nothing) = 0 of 5256; P(same | truth: opponent put something) = 6294 of 9336 (67%). One fork where the opponent enters a permanent proves the truth was not "nothing".

Also latent: `GambitReveal` (`resolve.rs:942`, Stronghold Gambit, Reanimator sideboard) stores the first chooser's pick as a raw slot in `sc[stage]` (`resolve.rs:1764`) and `resample_secret_picks` ignores it, so a fork keeps the true picked hand slot. Sideboard only today (no sideboarding in the core), so latent. Triumph of Saint Katherine is also sideboard only.

Suggested fix: when the first chooser is the opponent and its stage is done (`aux == 1`), always overwrite `sc[0]` with a fresh sample (uniform over eligible cards in the sampled hand plus "nothing", or a model-supplied distribution), regardless of whether the true value was 0; do the same for Gambit before sideboarding exists. Add the 0-versus-67% measurement as a test (it should be equal within noise).

### 3. High: no seat-bound API; raw engine data is public on `Game`

Where: `crates/mtg-view/src/game.rs:52` `pub fn pending(&self) -> Option<&Pending>` (raw `Pending`: `Opt::Cast(ObjRef,..)`, `Opt::PlayLand(ObjRef)`, any seat); `game.rs:99` `observe(&self, seat)` for either seat; `game.rs:104` `decision_for`; `game.rs:119` `take_events` (raw `Event`, including the opponent's `Drew { def }`); `game.rs:115` `set_keep_events`; `game.rs:129` `state_hash` (hash over hidden state and RNG); `sim.rs:15` `Policy::choose(&mut self, g: &Game, ..)`; `fork.rs:51` `fork` takes any seat; `lib.rs`/`Cargo.toml` of `mtg-view`.

What happens: doc 02 section 2.1 and 3 say agents get a `SeatHandle` bound to one seat and "State is not nameable from this crate's public interface". There is no `SeatHandle`. Anything holding `&Game` (every `Policy`, `FlatMc`) can call `observe(opponent)`, read raw object slots, hash hidden state, or drain raw events. Slot number equals decklist index (finding 1), so `pending()` alone identifies hidden cards.

Concrete failing scenario (`poc_raw_pending_exposes_slots`): fork for the agent's own seat, advance the fork to the opponent's decision, read `pending().options`; the slots in `Opt::PlayLand`/`Opt::Cast` mapped through the decklist index identified the true hand card in 17 of 17 cases (the fork keeps true slots, finding 1; on the real game at an opponent decision it is the same). Without the fork, `g.observe(Seat(1))` at any time shows seat 1's hand.

No enforcement of the boundary exists: no `cargo metadata` CI check (I searched), and `mtg-view` re-exports raw types from `mtg-core` through its public signatures. Also: `mtg-fuzz` enables `diff-harness`; with resolver 2 a `cargo test --workspace` unifies that feature onto `mtg-core`/`mtg-view` for every member built in the same invocation, so any future agent crate in this workspace would get `raw_state()` silently.

Suggested fix: introduce `SeatView<'a>`/`SeatHandle` with `observe()`, `decision()`, `apply()` (own decisions only), `fork(seed, model)` (no seat argument), and change `Policy::choose` to take it. Make `Game::{pending, observe(any seat), decision_for, take_events, set_keep_events, state_hash}` `pub(crate)` or `diff-harness` only (`playout` and `FlatMc` need internal versions). Add the CI check from doc 02 section 3 item 1 and a test that builds `mtg-view` without `diff-harness` and asserts the raw symbols are absent; keep agent crates outside this workspace or build them with `-p`.

### 4. Medium: `DecisionId` and pending-seat timing leak hidden choices and hidden-card existence

Where: `crates/mtg-core/src/engine.rs:90-94` (`ask` bumps one global `next_decision` for both seats); `crates/mtg-view/src/project.rs:224` (`Decision { id: p.id, .. }` exposed); conditional skips in `resolve.rs:1299` (`PutChosen`, Aether Vial), `resolve.rs:1390` (`EachPutFromHand`, Show and Tell), and per-choice asks in `libfx.rs` scry/surveil (`fx_scry`, `feed` `ScryBottom`).

What happens: doc 02 section 4.1 says internal counters must not rise with hidden opponent events. `Decision.id` is exactly such a counter. Two kinds of leak:
- The number of decisions the opponent makes depends on a hidden choice: scry asks once per card bottomed (`poc_decision_id_leaks_opponent_scry_choice`: in 6 of 30 opponent scry decisions the observer's next decision id differs, 364 vs 365, 345 vs 346, 379 vs 380, between "keep all" and "bottom one", with every other observed field identical). Doc 02 section 7 says the opponent sees only "Scry N".
- A decision exists only if the opponent holds a qualifying hidden card: Aether Vial's "may put a creature with that mana value" is not asked when the hand has none (`resolve.rs:1299`); Show and Tell's stage is skipped when the first chooser has no eligible card (`resolve.rs:1390`). The observer's next id gains a gap, and while the opponent's decision is pending the observer's stack still holds the ability (Vial) or its hand still holds the card it put in (Show and Tell). Found by the project's own noninterference harness on the real pool: seeds 7, 56, 71, 120, 135 (Aether Vial), 144 (observer casts Show and Tell: in one world its Island is still in hand because the opponent has a pending choice, in the other it has already entered). Whether the opponent holds a creature of the right mana value is exactly what Vial players bluff on.

Suggested fix: (a) expose per-seat decision ordinals to agents (keep the global counter internal for records, or make `Decision.id` a per-seat counter and have `apply` check it); (b) for decisions that exist only conditionally on hidden cards, always ask, with a single `Done`/`No` option when nothing qualifies, so the pending seat and the decision count do not depend on the hand (the trivial-decision machinery already handles single-option decisions). Extend the harness to compare `(pending seat, id)` every step (it does) after fixing finding 5 so these stop being masked.

### 5. Medium: the non-interference gate cannot catch findings 1 to 4

Where: `crates/mtg-debug/tests/noninterference.rs:7-16` (test pool decks only), `crates/mtg-debug/src/noninterference.rs` (stops at any opponent `ChooseCards`, stops when events differ after name normalisation, `own_library` mode stops at the first library count change), `read.rs:237` (`rerandomize_hidden` permutes defs among slots).

What happens: the gate table calls non-interference passed for 300 seeds x 2 observers on the test pool, with Show and Tell, scry, Doomsday, Vial, fetch searches and Oracle absent from it. The harness never varies the opponent's hidden CHOICES (it picks opponent labels common to both worlds), never changes slot structure, and stops exactly where hidden choices begin.

I ran the harness unchanged on the real eight decks (`ni_on_real_pool`, 150 seeds x 2 observers x both modes, 596 world pairs that differ in hidden state): 143 divergences. After triage (a hand reveal such as Thoughtseize, a public exile from the library, or an own-library shuffle or look legitimately differ; patch `noninterference-harness-triage.patch` stops on those), 17 remain: seeds 7, 56, 71, 120, 135 (Aether Vial), 144 (Show and Tell), and 11 `own_library` cases where the observer's own scry/search options name its true top cards, which is legitimate. So the harness does fire on real-pool cards, nobody ran it there.

Suggested fix: run it on the legacy pool as part of the gate; make the "legitimately different" cases explicit (own-library look, hand reveal, public reveal from a hidden zone) instead of stopping; vary opponent choices (take the choice in world B that corresponds to world A's choice by hidden-card identity, not by label); add the slot-permuting twin from finding 1; add a canary for each of the four bugs above (decode, S&T empty pick, id gap) so the gate is shown able to fail.

### 6. Medium: Observation and option completeness gaps in the frozen API (not leaks)

These do not leak, but `Observation` is part of what is frozen and adding a field later needs an RFC.
- No exile zone in `Observation` (doc 02 section 2.3 lists `me.exile` and the opponent's face-up exile). `project.rs:227-305` never reads `State::exile()`. Warp recasts (`ZoneKind::Exile` options), Doomsday's exiled library, Adventure and "exile_plays" cards are therefore castable by option index but invisible, and `encode_options` gives them `subject_def = 0` (`encode.rs:152-196`, `find` finds no vid).
- Missing designations: dungeon room, completed dungeons, emblems, Ring level and bearer, city's blessing (accessors exist in `read.rs:46-70` but are not projected).
- Combat: attackers are only a boolean (`ViewPermanent.attacking`); which player or planeswalker each attacks is not shown to the defender, and `Opt::Attack(AttackTarget::Walker(_))` is labelled "Attack planeswalker" with no subject (`project.rs:136`), so two planeswalkers are indistinguishable options (the attack-target decision new in M3d is therefore not usable with two walkers).
- Cards in the owner's library that the owner has looked at (scry, search candidates, Doomsday candidates, known top) carry `ViewId` 0 (`know()` never allocates a vid, `libfx.rs:77`), so every such option has `subject: None` and the encoder sees `subject_def = 0` for scry, surveil, search and Doomsday choices: the policy network cannot tell those options apart (`encode.rs:187`).
- Not leaks, but doc 02 section 2.3 promises `library_knowledge` unseen multiset and `revealed_hand` for look effects that currently emit no event (scry, search results with "reveal it").

Suggested fix: add exile, designations, per-attacker target and a library-card vid (or an option-level `subject_def`) before the freeze.

### 7. Low: fork leftovers

- `determinize` leaves opponent-only knowledge bits attached to objects (`known_to`/`pos_known_to` for the opponent seat on its own library and hand, `fork.rs` does not touch `self.knowledge`) and the opponent's hand order and view-id counter. They encode hidden choices (how many cards the opponent bottomed or kept, draw order). Counts survive; exact runs are partly scrambled because only the OBSERVER's known positions are held fixed (`poc_fork_keeps_opponent_private_knowledge`: exact structure survived in 1 of 50, count is preserved by construction). Reset or resample them.
- `mtg-view/src/fork.rs:52` only refuses when a decision is pending for the other seat, so a fork taken between `apply` and `advance` (no pending decision) or at game over is allowed, though doc/`ForkError` say forks are valid only at the observer's own decision. Require `pending.seat == seat`.
- Gambit pick: see finding 2 (latent).

### 8. Low: `revealed_hand` order and vids follow the opponent's true hand order

Where: `read.rs:94` (`known_hand` iterates `hand` in list order, unsorted), `project.rs:299` (`revealed_hand`), `resolve.rs:718` and `resolve.rs:1187` (`reveal_to` allocates vids in hand order). The opponent's hand vec is in draw order, so after Thoughtseize/Duress/Cloak and Dagger the observer sees which revealed cards were drawn earlier (opening hand versus later draws), which a player holding a hand would not show. Own hand is correctly sorted `(def, vid)` (`project.rs:233`). Fix: sort `revealed_hand` by `(def, vid)` and allocate vids at reveal time in `(def)` order, not hand order.

### 9. Low: Thassa's Oracle leaves stale position knowledge

Where: `libfx.rs:476-485` (`OracleTop`): the rest go to the bottom via `random_bottom_order` then `lib_to_bottom`, which does not clear `pos_known_to`. `reveal_pick_finish` (Atraxa, `libfx.rs:292-299`) does clear it. If those cards had position knowledge from an earlier Brainstorm, scry or Ponder, `me.library_known_bottom` now shows the true random order to the owner. `poc_oracle_random_bottom_knowledge`: 27 of 286 Oracle resolutions (Doomsday deck, random play) left the owner a "known" bottom run that was empty before. Own-library order is protected information (doc 02 section 1). Fix: clear `pos_known_to` for `rest` after the random placement.

### 10. Low: Miracle reveal is forced; miracle cast is offered without an affordability check

- The miracle trigger is a hand-zone trigger (`trigger.rs` `objs_in(ZoneKind::Hand)`, ability `MiracleCast`, `resolve.rs:980`) that goes on the public stack automatically and calls `reveal_to(opponent)`. Rules 702.94a make revealing optional. In a trace with Triumph of Saint Katherine in the main deck, the trigger shows on the stack and the decider is then asked `CastMiracle`. The owner cannot keep the card secret. Latent today because Triumph is sideboard only (`decks/uwx-control.txt`, sideboard). The rules reviewer should confirm; the hidden-info reading is that the engine gives away a card the owner may choose to keep hidden.
- Incidental, outside this review's scope: with 8 Triumph of Saint Katherine copies in the main deck of `uwx-control`, seed 1 of `poc_miracle_forced_reveal` panics at `cost.rs:361` ("pre-validated payment must succeed (engine bug)") because the `CastMiracle` choice (`Opt::Card`, `resolve.rs:~1818`) is offered with no check that `{1}{W}` can be paid. Latent (sideboard card), but it is a crash in a freeze candidate.

### 11. Low: under-reveal (fork unsound, no leak)

Revealed or looked-at cards the engine does not credit to the seat that saw them: `DigCreatureAttacking` (`resolve.rs:~893`) reveals to the opponent only, not to its own controller; "reveal it" tutors (Recruiter of the Guard, `Search` dest Hand) tell the opponent nothing; `DrawRevealCast` and `DiscardNamed` set `known_to` without a vid; scry, surveil look and search emit no view event ("Scry N" per doc 02 section 7). Consequence: the observer's fork re-samples cards it actually saw. Known limitation 5 (no top-K constraint) has the same family: Brainstorm put-back also loses the "two cards put on top" fact. Not a leak; hurts search quality.

### 12. Low: bookkeeping

- `game.rs:58-61`: `Game::apply` sets `ev_mark` before `engine::apply` validates the id and index, so a rejected apply (stale id, bad index) discards the seat's unseen event backlog. `deep_checks` probes this only on clones.
- `GameRecord` (`record.rs:12`) contains seed, both decklists in slot order, both sideboards and `hash_full` checkpoints: a complete god record with no type distinction, no `god_record()` gate (doc 02 section 2.1, 8). Nothing leaks today because `Game` never produces one mid-game; make the type `GodRecord` and keep construction behind the game-over check.

---

## What I checked and found correct

- Observation projection (`project.rs:227-305`) is built field by field. Opponent hand shows only `hand_count`; library only counts and the observer's known runs; battlefield, stack, graveyards public; no `ObjRef`, slot, `CardId`, RNG, seed or full hash reaches `Observation`. `view_hash` hashes the observation only (the canary that hashes state is off in normal builds).
- `redact` (`project.rs:33-84`) is an exhaustive match; opponent draws, discards to hidden zones and hidden-zone moves redact to counts; `ZoneChange` names only when the observer has a view id on either side. `DrewFromEmpty`, shuffles, mulligan counts are public by rules.
- `ViewId` allocation (`ops.rs:253-258`, `reveal_to`, `cast.rs:73`, `trigger.rs:316`): per-seat counters advance only for objects visible to that seat, so the observer's counter does not move on hidden opponent moves. `CardId` follows decklist order, not library order (and see finding 1 for why that is not enough once forks exist).
- Canonical option keys (`legal.rs:464-472`, `target_key`): built from `(def, observer vid)` and the other seat's knowledge of the viewer's own card; neither iterates arena order or hash maps. Collapsing (`card_options` `resolve.rs:1683`, hand collapse in `priority.rs`) keys on def and public state; the chosen representative is the copy the other seat already knows, so an unknown copy's existence does not change the option. Name choices (`stabilize.rs:138`) list the whole pool, not the opponent's deck. Search offers a single option per card name sorted by definition id, never library order (candidate vids are 0, which is why finding 6 matters but also why order cannot leak).
- Priority always asks (`turn.rs:36`); casting decisions that are skipped when single-option (`cast.rs:247`, delve, phyrexian) depend on affordability, which is public. Attack and block option lists, the attack-target decision and Raph and Mikey / Mobilize prompts depend only on public state (the planeswalkers the defender controls); the decision is skipped when only the player can be attacked.
- Knowledge updates: draw (`ops.rs:commit_draw`), moves (`commit_move` sets `known_to` for seats that can see the destination, clears `pos_known_to`), shuffle clears position knowledge (`ops.rs:455`), scry, Ponder, Brainstorm put-back, Mishra's Bauble (`look_top` gives the looker `known_to` + `pos_known_to`), Personal Tutor reveals and puts on top for both seats (`libfx.rs:~490`, the card does reveal), Atraxa reveals both then random bottom clears position, Thoughtseize/Duress family `reveal_to` the chooser, Doomsday: pile order stays owner-only, exiled cards are public events, the opponent can deduce the pile multiset from the decklist and the fork's unseen multiset is exactly that. `Triumph PileBack` shuffles the top seven and clears position; the lack of a top-K constraint is a documented limitation.
- Show and Tell and Gambit simultaneous entry run in one batch (`batch.rs`, `resolve.rs:1397-1411`, `1000`); `begin_batch`/`end_batch` hold `ZoneChange` events and `departed`/`lki`/`pre_bf` state only for public objects (a batch never spans a decision). Trigger matching (`trigger.rs:34-130`) iterates hand-zone objects of both seats but only Miracle exists there and its effect is public (finding 10 is about choice, not leakage). `interchangeable` trigger collapse uses public data and static IR text.
- Encoder (`encode.rs`): `encode_state` and `encode_options` take only an `Observation`; no opponent hand, opponent library or hidden counters; fork and game give identical encodings (the existing test and my reading agree). `OptionFeat.value` carries only numbers and name ids. Its limits are findings 6, not leaks.
- Canary mechanism is compile-time off without the `canary` feature; `rerandomize_hidden` and `library()` are `diff-harness` only; `State` Debug is redacted without that feature.
- `ApplyError` has a closed set of variants and `describe_error` is static text (checked by the harness probes as well).
- I ran: `cargo test --release -p mtg-fuzz --test fork` (6 pass) and `-p mtg-debug --test noninterference --test canary` (2 pass, 36 s): they pass on the current tree, which is the point of finding 5.

## What I could not check

- Rules correctness (rules reviewer's scope), differential parity with Forge, the sealed holdout, golden-hash stability and the invariant fuzz counts in `CORE-FREEZE-REVIEW.md`; I did not rerun the 4,000-game fuzz or the specrun.
- Stronghold Gambit and Triumph forks and the Gambit resample gap were found by reading; they are sideboard cards, so no random-play probe reaches them without a sideboarding build (the miracle probe needed a modified deck).
- Finding 8 (hand-order vids) and the forced-reveal part of finding 10 are by code reading plus one trace; I did not build a probe that decodes draw order.
- Timing and wall-clock channels (out of scope per doc 02 section 1), and any LLM-facing tool layer (`simulate`, `sample_opp_hands`) that does not exist yet; findings 1 to 4 will apply to it unchanged.
- The prototype fix in `hidden-info-probes/` closes the observation-level fork leak only; I did not review it for statistical correctness of the belief distribution (known cards re-placed uniformly among free positions is the intent) nor for options or hash side effects beyond the six `fork.rs` tests.
- I did not measure whether the real `FlatMc` is statistically biased by finding 1 (it should be unbiased for a non-adversarial agent that never inverts the seed; the leak matters to any agent that picks seeds deliberately, such as an LLM tool).

## Reproduction

Probe source: `hidden-info-probes/review_hidden.rs`; steps in `hidden-info-probes/README.txt`. Numbers above are from the original code (`ni_on_real_pool` and the decode probes before the prototype patch). Build dir used: `CARGO_TARGET_DIR=/home/claude/target-review-hidden`.
