# Forge game-state forking for `simulate`: feasibility (MEASURED)

Forge commit fd5c996 (2026-09-30), built from source. Everything below was read from that source or measured with the harness in `harness/` (raw output in `data/`). This replaces the earlier preliminary version, which was written without source access. Where a claim is an inference rather than something I ran, it says so.

Test conditions: 4 vCPU shared box, two JVMs running at once (`-Xmx2g`, ParallelGC), warm JIT, UR Delver vs Jund (the runner thread's placeholder lists, from memory), 12 games, 285 probes at main phases of turn 2 and later with an empty stack, plus 119 probes with a spell or ability on the stack. This is two non-combo decks and one engine build, so treat the numbers as an order of magnitude, not a benchmark.

## 1. Answer

**Forking is feasible, cheap enough for an LLM `simulate` tool, and leak-safe if hidden zones are never copied. It is not good enough yet for stack-interaction lines or Storm counts, and it is far too slow for search or RL.**

| Question | Result |
|---|---|
| Can a Forge game be forked? | Yes. `forge-ai`'s `GameCopier` rebuilds an independent `Game` from a live one. The AI uses it for lookahead, and Forge's own tests check the copy against the original. |
| Cost per fork | **4.1 ms median** (p90 9 ms, one outlier at 73 ms), flat from 0 to 19 permanents. **2.8 MB** of heap per copy. With hidden cards pruned (below): **1.8 ms**. |
| Throughput | About 490 forks/s across 4 threads on this box, no failures. About 250 forks/s per core. |
| Fork fidelity (stack empty) | AI evaluator score identical in 285/285 probes. Full state digest identical except per-turn "spells cast this turn" counters (57/285 probes). |
| Does the copy leak hidden info? | **Yes by default**: opponent hand and library, with identity and order, were present in 285/285 copies. Fixable (section 3). |
| `simulate` cost | About 30 ms per simulated action including target iteration; about 17 ms on the pruned and sampled pipeline. A 3-5 call budget is roughly 0.1-0.15 s per decision. |
| Deterministic replay | Works: same seed gave byte-identical game logs in 3 runs (2 games, 787 log lines each). But a replay fork re-runs the whole game so far, which is seconds. Test oracle only. |

**Recommendation:** use `GameCopier`-based forking for the Phase 1 `simulate` tool, on the non-combo matchups first, with a patched copier that never constructs hidden cards (section 3). Before relying on it for Force of Will, Daze, Stifle or Storm lines, fix the gaps in section 4. Forge's fork speed (hundreds per second) does not support Phase 3 search or RL, which supports the Phase 2 engine argument, but it is not an argument that forking Forge is infeasible for Phase 1.

## 2. How forking works in Forge (read from source)

- `forge-ai/.../simulation/GameCopier.java` builds a new `Game` and copies players, phase, and every card in Battlefield, Hand, Graveyard, Library, Exile, Stack and Command. Cards are rebuilt from their `PaperCard` through `CardFactory.getCard`, which the source itself calls "very expensive" and "the vast majority of GameCopier execution time". Card ids are deliberately preserved so forked games diverge the same way as the mainline.
- `GameSimulator` wraps it: copy, apply one spell ability, resolve the stack, score the result. `SpellAbilityPicker.evaluateSa` also iterates over target and mode choices.
- `GameSnapshot` (behind the `EXPERIMENTAL_RESTORE_SNAPSHOT` flag, used for undo) is a second copy path. I did not evaluate it.
- **Randomness:** all game-side randomness goes through the static `MyRandom` (a `SecureRandom` unless seeded with `setRandom`). `SpellAbilityPicker.evaluateSa` swaps that global during simulation, and its own comment says simulations can't run in parallel because of it. So forks can be built in parallel (4 threads, 0 failures), but `evaluateSa`-style simulation is effectively single-threaded per JVM. Use several JVMs or make `MyRandom` per-game.
- **Not copied**, per `TODO`s in the copier: `thisTurnCast` (the list behind the Storm count), `countersAddedThisTurn`, `creatureAttackedThisTurn`, `ExiledWith`, and others. The stack is not copied at all unless `GameSimulator.COPY_STACK` is set (default false).

## 3. Hidden information

**Default copy leaks.** The copier copies the opponent's hand and library cards exactly (285 of 285 probes). There is a dead-code hook, `PRUNE_HIDDEN_INFO = false`, that replaces cards the acting player can't see with an empty placeholder artifact instead of rebuilding them. Making it settable (a one-line change, `harness/prune-flag.patch`) and passing the acting player to `makeCopy(null, me)` gave:

- 17,266 of 17,266 opponent hidden cards and 15,961 of 15,961 of the acting player's own library cards became placeholders; the acting player's own hand (1,403 cards) stayed real.
- Copy time fell from 4.1 to 1.8 ms, because hidden cards are not rebuilt.
- Filling the placeholders with a sampled world (opponent hand and library from decklist minus known public cards, own library likewise) took 3.6 ms and left 0 placeholders. Simulating on that world worked (1,242 simulated actions).
- My sampler's output depended only on public information: two real worlds with the same information set but different hidden hands and library orders gave identical samples with the same seed (285/285). This tests my sampler, not Forge.

**Card ids are an identity side channel.** Forge assigns card ids contiguously per card name per player, for example all four of one player's Bloodstained Mires are ids 73-76 and both Daze copies are adjacent. The copier preserves ids, so a placeholder still carries its id. Anything that shows Forge ids of hidden cards to a player or LLM (the state serializer included) leaks which hidden cards share a name. The sampler must assign fresh ids (mine did, via `nextCardId`), and the serializer must renumber hidden cards per decision.

**Not handled or not tested:**
- Pruning uses Forge's zone-based `canBeShownTo`, with a source `TODO` saying revealed-card memory is not checked. A card the opponent revealed (Thoughtseize, a Force of Will pitch, Brainstorm put-backs) would be treated as hidden in the fork, so the fork can contradict known information. A knowledge tracker built from the game event stream is still needed. Not tested here.
- Sampling is uniform over decklist minus known cards. It ignores inference from play history (what the opponent kept, didn't cast).
- A bug in my first version of the sampler is itself a lesson: transformed and double-faced cards change name in play (Insectile Aberration is Delver of Secrets), so known-card accounting must use the underlying paper card name.

## 4. Where fidelity falls short

1. **The stack is not copied by default.** With the default setting, 0 of 119 forks made while something was on the stack contained the stack items; the spell cards sit in the stack zone with no stack entry (78 cases). With `COPY_STACK = true`, 71 of 119 matched. The 48 misses are:
   - Activated and triggered abilities: fetchlands, Wasteland, Grim Lavamancer, Delver's upkeep trigger, Dragon's Rage Channeler and others are dropped, because `copyStack` only handles spells.
   - **Force of Will cast with its alternative cost** was dropped in all 3 probes where it was on the stack.
   - **Order**: one two-item stack of two Chain Lightnings came out in reversed order (n=1, not investigated further).
   - Resolving the copied stack threw an NPE on the split card Fire // Ice in all 3 probes where it was on the stack.
   For Force of Will, Daze, Stifle and Spell Pierce lines this matters directly.
2. **The opponent never responds in the simulator.** `GameSimulator.resolveStack` resolves the stack with an AI controller and gives no priority-passing or response decisions. Modeling counterspell and Daze responses needs a priority loop built on top, not just a copy.
3. **Per-turn counters** (spells cast this turn, which drives Storm) are not copied: all 57 digest differences were these counters. Fix is small (copy `thisTurnCast` and the per-player counters) but required before any Storm deck.
4. **`simulate` failures in my probe: 117 of 946** simulated actions failed, all on cards that need a target or a spell on the stack (Daze, Spell Pierce, Force of Will, Counterspell, Abrupt Decay, Maelstrom Pulse), which my probe enumerated with an empty stack. I believe these are enumeration artifacts rather than copy bugs, but I did not triage each one. A real `simulate` tool should enumerate actions through the engine's own legal-action path.
5. **Not covered by my probes:** combat phases, delayed triggers, "until end of turn" effects, replacement effects, and anything in a combo deck. My digest does not cover them, and the score check does not either.

## 5. Options compared

| Option | Verdict |
|---|---|
| A. Deep clone of the live object graph | Reject. Forge has no such API, and it would copy the real hidden zones. |
| B. `GameCopier` with pruning and sampling | **Recommended.** 2-4 ms to fork plus about 4 ms to sample, leak-safe by construction once placeholders are used and ids are renumbered. Needs the fixes in section 4. |
| C. Seed plus action-log replay | Deterministic in my test, but costs the whole game so far (the runner measured about 2-3 s per game). Use as a test oracle to check fork fidelity, not in production. |
| D. OS-level process fork | Reject. Copies the real opponent hand and is heavy. |

## 6. Consequences for the plan

- **Phase 1 `simulate`:** viable on the Delver/Jund-style non-combo matchups with pruned-and-sampled forks. Per decision a 3-5 call budget is roughly 0.1-0.15 s of engine time, small next to LLM latency. Do not describe stack-interaction or Storm simulation as supported until section 4 is fixed.
- **Engineering estimate for the fixes** (my judgment, not measured): per-turn counters and `MyRandom` per game are small; copying abilities/triggers/alt-cost spells and stack order is a few days of careful work in Forge's stack code; an opponent-response model is a new component.
- **Phase 2 argument:** fork speed of a few hundred per second per box caps `simulate`-style tools and rules out MCTS-scale search or RL on Forge. That, and the amount of patching needed for faithful stack interaction, is the real argument for a custom engine, not an inability to fork.

## 7. Open items

- Run the fidelity digest at stack-nonempty states and in combat, and on a combo deck (Storm or Doomsday) once the archetype thread picks decks.
- Triage the 117 `simulate` failures individually.
- Test known-information handling (revealed cards, Brainstorm put-backs) against the event stream.
- Compare `GameCopier` with the `GameSnapshot` path.
- Measure fork cost under the final deck lists; cost scales with cards in play and in hidden zones.

## Files

- `harness/ForkProbe.java`, `harness/aggregate.py`, `harness/README.md`, `harness/prune-flag.patch`: how to reproduce. The Forge checkout itself was not modified.
- `data/`: raw probe output (`c_*` baseline, `p_*` prune pipeline, `st_*` stack probes).
