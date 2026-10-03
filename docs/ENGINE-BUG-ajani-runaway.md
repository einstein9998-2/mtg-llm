# Engine bug: runaway resolution loop in "Ajani, Nacatl Avenger" (frozen core, engine v4)

Found while generating full-pool self-play data. One game aborted the whole data process with
`memory allocation of 30064771072 bytes failed` (a Rust allocation failure aborts, so the per-game `catch_unwind` guard in `mctsdata` cannot catch it).

## What happens
`mtg_core::resolve::run`, stage `ResolveStage::Ops`, loops `while f.pc < code.len()` over a script. For the ability of **Ajani, Nacatl Avenger**
the script is at `pc 3` with `Flow::Jump(1)` (a backward jump), and the loop condition never changes, so it runs forever, emitting
`Event::CountersChanged` through `Cx::add_counters` -> `Cx::emit` each pass until `Vec<Event>` (`state.events`) grows to tens of GiB.
Call stack at the allocation (release build with debug info): `Vec<Event>::push` <- `Cx::emit` <- `Cx::add_counters` (ops.rs:579) <- `resolve::leaf` (resolve.rs:577)
<- `resolve::run` (resolve.rs:140) <- `engine::run_top` <- `Game::advance` <- `sim::playout` (a random rollout inside MCTS).
Confirmed by a temporary step counter in the Ops loop (panic after 3,000,000 steps, reported `card "Ajani, Nacatl Avenger" pc 3 op Jump(1)`); that diagnostic was removed, the core is unchanged.

## Reproduction (deterministic)
Boros Aggro mirror, pool run seed 100, game index 1449 of the 8-deck schedule (`pick_decks(8, 1449)` = decks 1 vs 1):

    mctsdata decks 1 32 out.jsonl --first 1449 --threads 1 --seed 100 --rollout 2500

(`--first`/`--skip`/`--log-games` are new options in `crates/mtg-agent/src/bin/mctsdata.rs`.) It aborts in about 40 s. It happens in a random playout, so it reachable from any
game state that lets the random policy repeat the Ajani loop; it is not specific to this game.

## Impact
- Rollout-guided search over decks that include Ajani can hit it (Boros Aggro; other decks with the card are untested). Observed once in about 1,500 pool games.
- It is a hang-or-crash for any consumer that loops the resolver, not only training: the same loop would hang a real game if an agent reached the state.

## Workaround in the training loop (no core change)
`selfplay_loop.py` now runs `mctsdata` with `--log-games`; on an abort it replays each in-flight game alone to find the runaway one(s) and reruns the generation with `--skip`.
A tiny step cap in the Ops loop (panic past some millions of steps, caught by the existing `catch_unwind`) would also contain it, but that is a core change and needs a decision under the freeze rules.

## Suggested fix (core owner)
Look at Ajani's script around pc 1..3 (the loop's exit condition, probably a count or target-set check that is not updated inside the loop) and add an engine-level step cap for resolution scripts.
