# Core freeze review (M3), prepared 2026-10-01

Status: freeze CANDIDATE, not frozen. Doc 01 section 14.2 needs two independent reviewers (one a rules reviewer) and Brady's sign-off before the core is frozen; the implementer cannot approve this. After sign-off, changes to the frozen parts go through `rfcs/0000-template.md`.

Candidate: workspace commit named in `MILESTONES.md` (M3f), `ENGINE_CORE_VERSION = 3`, `HASH_SCHEMA = 2` (with the M3g fixes).

## 1. What would be frozen (doc 01 section 14.1)

`mtg-core`: `State` layout and object identity (`state.rs`), turn/priority machine (`turn.rs`, `priority.rs`, `engine.rs`, `stabilize.rs`), cast and resolve (`cast.rs`, `cost.rs`, `resolve.rs`), SBA (`sba.rs`), events and trigger/replacement pipelines (`event.rs`, `trigger.rs`, `replace.rs`), layers (`derive.rs`), combat (`combat.rs`), RNG and determinism (`rng.rs`, hashing in `read.rs`), the IR and effect VM (`ir.rs`, `compile.rs`; 90 effect variants, 30 expressions, 17 conditions, 18 ability kinds), the enumerator (`legal.rs`), and the `mtg-view` API types (`Game`, `Observation`, `Decision`, `Fork`, `BeliefModel`, records).

Not frozen: card data (`legacy.cards.ron`), `mtg-spec`, `mtg-debug`, `mtg-fuzz`, tools. Size today: mtg-core 12.2k lines, mtg-view 0.9k, 174 card definitions.

Reviewer note on the IR: many of the 90 effect variants are card-shaped (for example `GambitReveal`, `ExileReturnTransformed`, `PileBack`, `DigCreatureAttacking`). That is deliberate for a closed pool, but it means "adding a case to an existing closed enum" (core-adjacent, doc 01 section 14.1) will be common when the pool changes. Decide whether card-shaped leaves count as core or card data.

## 2. Gate status

| Gate (doc 01 section 14.2) | State |
|---|---|
| Unit and engine tests | `cargo test --workspace --release` green on the M3f build (including the 56 s test-pool non-interference test, the new hidden-information tests, agent API compile test, real-pool goldens); `cargo test` in debug for mtg-core, mtg-fuzz and mtg-debug run too |
| Rulings suite, visible | 806 of 806 pass (`specrun`) |
| Rulings suite, holdout | Run by the spec thread before the M3d fixes: 321 of 338 (4 engine bugs, all fixed since, plus runner gaps). Needs a rerun on the current build |
| Own regression scenarios | 23 pass (`own-scenarios/`), implementer-written, not independent |
| Invariant fuzz | 3,000 real-pool games on the M3f build (invariants every decision, deep replay and fork checks every 200th, 2.08M decisions, 0 violations, 0 truncated, every main-deck card cast or entered). Earlier: 4,000 real-pool games on the M3d build (invariants every decision, replay on every 200th, 0 violations, every main-deck card cast or entered); earlier 20,000 games before M3d. Before that: 20,000 real-pool games (8 decks, invariants every decision, replay and fork checks on a sample) plus 600 debug-assertion games: clean. Every main-deck card cast or entered |
| Non-interference | Test pool: 300 seeds x 2 observers x 2 variants, 4 canary leaks all caught. Real eight-deck pool (new, `hidden_info.rs`): 6,000 pairs (1,500 games x 2 observers x own library permuted or not), about 400k lockstep steps, 0 failures. Every legitimate divergence is an explicit stop reason that is counted (observer shown different cards, own shuffle makes its draws differ, an opponent decision that exists only in one world, random bottom order); stops rather than failures remain a coverage limit |
| Hidden-information probes | The independent reviewer's decoders are tests: opponent hand recovered by forks (was 51 of 51 states, now chance level), library order (was 3,549 of 3,549 positions, now chance level), indistinguishable twins fork identically (was 0 of 119), Show and Tell fork independent of the true pick, per-seat decision ids independent of the opponent's scry, Oracle bottom knowledge, opponent private knowledge not carried into forks |
| Fork soundness | 25,410 forks at (nearly) every decision of 400 games: observer's view unchanged, invariants hold in the fork, playable. 787 at Show and Tell choices (this found a real leak of a hidden pick, now resampled) |
| Golden replays | 20 test-pool records and, new, 16 real-pool records (every deck at least twice, checkpoints every 25 actions) replayed against a pinned snapshot of the card text (`goldens-real/`), so card edits do not invalidate them and core changes do. 20 test-pool records replay bit-identically. Regenerated in M3d: the database hash changed with the IR and the state hash changed with the new `attack_watch` field and batching (core change, needs reviewer attention) |
| Hash coverage | `hash_rules` and `hash_full` destructure `State` exhaustively; a test pokes free-list order, timestamps, `moved`, config, step counter and the derived flag and requires either a different hash or an identical future (determinism reviewer's `hole` probe: 769 of 772 timestamp pokes changed the future while the hash stayed equal before the fix) |
| deps-scan | 20 layered effects in the pool, 2 flagged pairs (Kaito and Overlord of the Balemurk vs Magus of the Moon, layer 4), both reviewed independent in `crates/mtg-cards/cards/deps-reviewed.txt`. A second reader should confirm |
| Differential vs Forge | NOT DONE (belongs to M4 deck gates, doc 04) |
| Benchmark | Real pool, random play, release, one core: about 500 games/s without checks (about 350k decisions/s), 150 games/s with invariants every decision. Clone 2.4 us, fork (clone plus determinize) 10 us, observe 2.8 us (mid-game states) |

## 3. Known deviations and limits (what a reviewer should challenge)

Fixed after the rules review (2026-10-01, spec thread's `freeze-review-rules-2026-10-01.md`): trigger conditions that belong to the event versus intervening-if (`TriggeredDef.event_cond`); simultaneous zone changes are batched (`batch.rs`: leaves-the-battlefield abilities look back at every object that left together, enters triggers see everything that entered together, enters replacements come only from permanents already there); "can't enter" (Grafdigger's Cage) beats Containment Priest (614.17c); first and double strikers are recorded when the first-strike step begins; warped cards can be cast from exile later; Tamiyo, Seasoned Scholar +2 is a delayed trigger (`WatchAttacks`) that goes on the stack; Raph & Mikey's and Mobilize's new attackers ask their controller which player or planeswalker they attack (508.4); Raph & Mikey's bottom order honours scripted `bottom_order`; the Sand Warrior token has the types Sand and Warrior.

1. Batches are used for state-based creature, Aura and planeswalker deaths, "destroy", "exile", "sacrifice" and "move" effects over several objects, ExileUntil, and Show and Tell's simultaneous entry. Stronghold Gambit's two placements are one batch (the text does not say "simultaneously"; we read it as one event, 614.12, so a Containment Priest and a Spider-Woman revealed together do not affect each other). Ajani, Nacatl Avenger's -4 sacrifices, empty-hand discards and ExileGraveyard still go one object at a time (no observable pool case). The Aura and planeswalker batches take their look-back snapshot at move time. Each SBA group is its own batch (704.3 says all state-based actions are one event); the legend rule is a separate pass after them.
2. 616.1 (the affected object's controller chooses the order of competing replacement effects) is not modelled; enter replacements are applied in a fixed order, "can't enter" first. No pool combination is order dependent.
3. The Ring's level 3 sacrifice is a delayed trigger at end of combat.
4. Forks are valid only at the observer's own decision (`ForkError::NotObserversDecision`): options of another seat's pending decision can depend on its hidden cards.
5. Determinizer limits: no top-K constraint (Triumph of Saint Katherine's pile order is not modelled, known-identity cards are re-placed at random unknown positions of that library); the opponent's sideboard is dropped; secret picks (Show and Tell, Stronghold Gambit) are always resampled; opponent decklist is assumed fully known (`UniformConsistentModel`).
6. No sideboarding or match flow in the core; loop shortcuts not built (doc 02 section 9.6, shelved).
7. Layers stage 1 only (timestamps, no dependency detection). Only layers 4, 6, 7a, 7b, 7c exist in the IR, so unsupported layers are rejected by construction.
8. Counters (+1/+1, -1/-1, keyword counters) are applied in `derive.rs` as built-in layer 7c and 6 effects, not as pool effects, so `deps-scan` does not see them. Reviewer should confirm none of the pool's layer-4 or layer-6 effects depends on a counter.
9. Lower-priority deviations from the rules review, all without a case in the pool: within one layer every resolved effect applies before every static regardless of timestamp (613.7; Guide of Souls' Angel on Kaito is wiped, no card reads Angel); Aura legality is checked only as "attached object exists" (704.5m; the only Aura is Animate Dead, whose "enchant creature put onto the battlefield with this Aura" restriction is modelled by detaching it when the return fails, which is the Grafdigger's Cage case, so the pool case works); Phyrexian mana is announced after targets and delve (601.2b puts it first; only the order of the agent's decisions differs); `derive.rs` re-checks the `lost` flag in every layer (breaks 613.6 lock-in only for a nonbasic land whose static spans several layers; none in the pool).
10. Open rules questions in `rust-engine-spec/OPEN-QUESTIONS.md` (about 20) are decided by whatever the visible scenarios force; unasserted ones are unverified.

### Spec-thread re-read of M3f (2026-10-01)

Two findings fixed before the freeze (`MILESTONES.md` M3g): the miracle cast was never offered because the new affordability check also evaluated Triumph of Saint Katherine's alternative-cost condition (`way_feasible_inner` with `check_cond` false for the miracle), and Equipment on different creatures were treated as one choice for trigger ordering (`obj_sig` now includes what the permanent is attached to). Own scenarios for both fail on the pre-fix code and pass now.

### Review round M3f (hidden information and determinism, two read-only reviewers)

Reports in `/mnt/project-files/rust-engine-review/`. Fixed: fork position dealing and resampled secret picks (finding 1), agent access to the raw engine (`SeatView`, `diff-harness` feature), per-seat decision and view ids (finding 4), Oracle and Dig knowledge, observation gaps, hash coverage, strict records, endianness guard. Accepted or deferred, each needing a decision at sign-off: (a) an opponent's conditional decision (Aether Vial, Show and Tell) can differ between worlds the observer cannot tell apart, but the observer never acts in it and ids are per seat; (b) the miracle reveal is announced (forced by the rules; latent, the card is sideboard only); (c) `GameRecord` carries hidden state and is harness-only by convention; (d) `next_decision` is hashed, so no loop detection by hash; (e) u8 and u16 counters are not overflow checked; (f) "reveal it" tutors under-reveal; (g) a game with more than 60,000 live objects (a token-copy loop such as Ocelot Pride under random play, found by the first MCTS run) is declared a draw instead of exhausting the 16-bit object arena; (h) forks ignore known library segments: after Thassa's Oracle, Dig or Raph & Mikey the observer knows where some cards are, but the fork's determinizer places them at random, and Triumph's pile-back is not modelled (agent search worlds only, not real games). Core files changed since the last review: `fork.rs`, `batch.rs`, `trigger.rs`, `replace.rs`, `sba.rs`, hashing in `read.rs`, `engine.rs` (`ask`), `record.rs`, the whole `mtg-view` API. They need a fresh read.

## 4. Suggested review split

- Rules reviewer: `trigger.rs`, `replace.rs`, `sba.rs`, `derive.rs`, `combat.rs`, `cast.rs` against the CR (effective 2025-11-14), starting from OPEN-QUESTIONS.md.
- Hidden-information reviewer: `read.rs` (`rerandomize_hidden` is test-only), `fork.rs`, `mtg-view/src/project.rs`, `fork.rs` in `mtg-view`, `legal.rs` canonical keys.
- Determinism reviewer: `rng.rs`, hashing, record replay, option ordering in `priority.rs`.

## 5. After the freeze

M4 (two non-combo decks through the deck gate) and later milestones need differential testing against Forge (doc 04), the sealed holdout run, and a Python/batching layer (M6). None of that changes the frozen core unless a gate fails, in which case the fix goes through an RFC.

## Freeze declaration (2026-10-01)

Brady signed off ("sure, go ahead and freeze"). Core frozen at tag `core-frozen-m3g`, ENGINE_CORE_VERSION 3, HASH_SCHEMA 2.
Final gates: workspace release suite green, specrun visible 806/806 and own 23/23, 1000-game legacyfuzz with 0 violations and every deck card exercised, 6000-pair real-pool noninterference with 0 failures.
After the freeze, changes to `mtg-core`, `mtg-cards` rules data or the `mtg-view` observation/encoding go through `rfcs/0000-template.md` (state the bug or need, the change, the effect on hashes/version, the tests that fail before and pass after). Phase 3 work lives in `mtg-agent` and does not touch the core.

Post-freeze limit (RFC 0001, spec review 2026-10-01): automatic payment may sacrifice the spell's own target to Lazotep Quarry (the spell then fizzles); explicit Quarry activation avoids it. To be fixed in the next RFC.

Post-freeze (RFC 0003, 2026-10-02): a resolution script that runs more than 4,000,000 instructions ends the game as a draw (a runaway guard; the cause found was an effect-loop index that wrapped at 256, now widened). Quarry auto-payment no longer sacrifices the spell's own target unless nothing else can be sacrificed (supersedes the limit noted after RFC 0001). `ENGINE_CORE_VERSION` 5, `HASH_SCHEMA` 4.

Deviation list additions (spec thread review of RFC 0003, 2026-10-02): (1) the 4,000,000-instruction resolution cap ends the game as a draw (the rules have no such limit; a safety valve, no pool scenario reaches it); (2) `AttackersDeclared` and `BlockersDeclared` event counts saturate at 255 (no pool card cares about more than 255 attackers). Review state on `core-frozen-m5`: spec thread rules review done (visible 806/806, sealed 352/352), differential rerun done (no new divergences); the hidden-information and determinism reviewers have not re-read the changes since M3f (RFCs 0001 to 0003: legal.rs, cost.rs, eval.rs, event.rs, frame.rs, combat.rs, resolve.rs, lib.rs hash schema), which touch no view, fork or noninterference code but do change the hash schema.

## Sign-off on core-frozen-m5 (2026-10-03)
Brady confirmed `core-frozen-m5` (ENGINE_CORE_VERSION 5, HASH_SCHEMA 4) as the frozen engine after the spec, differential and hidden-information/determinism reviews were clean. Changes from here go through `rfcs/`; the four small follow-ups at the end of RFC 0003 are queued for the next one.

Post-freeze (RFC 0004, 2026-10-03): `RUNAWAY_OBJECTS` lowered from 60,000 to 1,000 live objects (draw) after the self-play run met 2,063 Ocelot Pride tokens; the card text was checked and the engine is correct (four Prides with the city's blessing make 30 tokens per end step, older tokens are not recopied), so this is a performance/degenerate-play guard, not a rules fix. Review follow-ups of RFC 0003 applied (stale-cache refresh and wider avoid list in the Quarry victim choice, view-id ordering documented, Show and Tell fork test stratified). `ENGINE_CORE_VERSION` 6, `HASH_SCHEMA` 4.

## Re-read of core-frozen-m6 (RFC 0004): hidden information and determinism (independent reviewer, 2026-10-03)

**Verdict: no finding. `core-frozen-m6` can be signed off from the hidden-information and determinism side.** All four items of the m5 review are addressed as intended.

Diff read: the mirror has no git history, so the reviewer diffed the m6 mirror against the scratch copy of m5 made for the previous review. Core changes are exactly: `cost.rs` (`avoid` gains `ctx.picks` and `source`), `legal.rs` (`refresh()` at the top of `sac_candidates`; a comment on `obj_key`), `state.rs` (`RUNAWAY_OBJECTS` 60,000 to 1,000), `lib.rs` (`ENGINE_CORE_VERSION` 6), and a comment in `resolve.rs`. The only test change is the Show and Tell fork test. The 36 goldens differ only in the `engine` header line (5 to 6); no checkpoint hash changed.

Per change:
- **`avoid` with picks and source** (`cost.rs`): both are public battlefield objects; the victim order is unchanged except that these are tried last, so the choice still depends only on public state and `obj_key`. No RNG, no hidden read.
- **`refresh()` in `sac_candidates`** (`legal.rs`): closes the stale-cache read. The derived cache is not hashed, and a refresh at that point is idempotent, so hash-equal states now behave alike whatever the cache state was.
- **Object cap 1,000** (`state.rs`, checked in `create_token_with`): `live_objects()` is `objs.len() - free.len()`, a count of every object including cards in both hands and libraries. Those counts are public and the same in any two worlds the observer cannot tell apart (a fork keeps them), so the point where the game is declared a draw cannot depend on hidden state. It is a public result, determined by hashed state (`objs`, `free`). Headroom: in 6,000 random-play games involving Boros Aggro the arena never exceeded 201 slots (the deck with Ocelot Pride; other decks were not measured). The draw is a rules deviation, already recorded; it is also a scoring consequence for search (a draw scores 0.5), which the engine owner should keep in mind for degenerate token lines.
- **Show and Tell test**: now compares per opponent hand size (public), weighted by the smaller group, bound 0.15. On the run, per-stratum differences are 0.00 to 0.02 in every hand size 1 to 7, against 0.67 for the leak it guards against, so the bound is loose but the test would still fail on a regression. No leak.
- **`obj_key` comment**: accurate (checked against `commit_move`, which assigns fresh view ids to both seats at battlefield entry). The invariant test suggested in the m5 review (view-id order agrees for both seats on the battlefield) was not added; optional.

Evidence (release build of the m6 mirror, pinned rustc 1.97.0, scratch copy):
- `hidden_info` suite with `NI_GAMES=1500`: 10 of 10 pass. Real-pool non-interference: 6,000 pairs, 408,202 steps, 0 failures, step count identical to m5. Arena test: 185 draws in 400 games (as m5).
- `fork` 6, `agent_api` 2, `goldens` and `goldens_real` (36 records) pass.
- Quarry-focused non-interference (`m5_probe`): 1,200 pairs, 169,082 steps, 0 failures, 585 Quarry taps, 508 with a sacrifice; identical to m5.
- m5 versus m6 digests (`det_probe`; hash, both view hashes, decider, id and option count at every decision): identical on 300 default-matchup games (`daa6d5fa21b41e00`, 217,727 decisions) and on 2,500 games with Boros Aggro in a fixed matchup (Boros mirror, Boros against two other decks, both seat orders; 2.4 million decisions): the cap and the wider `avoid` change nothing in these games. A debug build (overflow checks and debug assertions) of m6 gives the same digests for the default set and for the Boros mirror (`b7f222c7b9563816`, 570,092 decisions), with no panic.

Limits: the token-explosion behavior itself (the game that hit the cap on m5) is covered only by the engine owner's own scenarios (`own-ocelot-token-explosion-ends-as-a-draw`); random play never reached 1,000 objects in these runs, so the cap's behavior in a search-driven game was not exercised here. The derived-state refresh is still quadratic in permanent count, as RFC 0004 says.
