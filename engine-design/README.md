# Custom MTG Engine: Design Package (Phase 2)

Status: DRAFT v0.2 (re-scoped to the final eight-deck pool: UR Cutter, Alurentell, Boros Aggro, BW Death and Taxes, Dimir Tempo, UWx Control, Doomsday, Reanimator), docs only. No engine code exists. Nothing here has been pushed to any repository.

## Reading order

| Doc | What it covers | Words |
|---|---|---|
| [01-core-design.md](01-core-design.md) | Goals, language, crate layout, state layout, turn/priority, stack and casting, events/triggers/APNAP, state-based actions, replacement and prevention, layers and timestamps, combat, determinism, performance targets, freeze policy and agent roles, milestones, scope management | ~8.3k |
| [02-action-api-and-hidden-info.md](02-action-api-and-hidden-info.md) | The agent-facing action/decision/observation API, the hidden-information threat model and the layered enforcement, identity and ordering leaks, determinized forks for `simulate`, knowledge tracking, event redaction, dataset classes, Show and Tell, Stronghold Gambit and Doomsday cases | ~4.1k |
| [03-card-dsl.md](03-card-dsl.md) | The card DSL (costs, selectors, targets, values, effects, triggers, statics, keywords), worked card examples, bytecode VM, custom ops, hard-card list, glossary for cached prompts, card provenance and status | ~3.4k |
| [04-differential-testing-spec.md](04-differential-testing-spec.md) | Oracle trust model, Forge adapter, canonical action/state formats, rulings-as-spec pipeline with holdout set, lockstep differential with randomness protocol, triage and shrinking, invariants, non-interference test, self-play anomaly detection, deck gate, known-divergence registry, phasing | ~6.8k |
| [05-pool-mechanics-inventory.md](05-pool-mechanics-inventory.md) | Rules features (including dungeons, the Ring, energy, ninjutsu, warp, impending), hidden-information cases, randomness uses, custom-card candidates, and a staged rollout for the final 8-deck, 144-card pool in `/mnt/project-files/decks/` | ~4.4k |

## The main decisions, in one place

1. **Language: Rust** (confirmed by Brady). Main reasons are exhaustive matches that stop agent-built code from drifting and crate privacy that makes the hidden-info boundary real.
2. **State is plain, clonable data** (indices, no pointers, no trait objects). Resolution is a **continuation stack of data frames driving effect bytecode**, so the engine can pause at any decision and be cloned mid-resolution.
3. **The agent API is index-into-a-mask only.** Casting is decomposed into small masked decisions; every offered option is pre-validated as completable, so illegal moves cannot be expressed and no rollback machinery is needed in normal play.
4. **Hidden info is enforced by projection, not filtering**: agents see only `Observation`; internal ids never leave the core; per-observer view ids; forks are built from a projected view plus a sampled hidden assignment, never from the true state. A non-interference test with canary mutations verifies it.
5. **Cards are RON data compiled to bytecode** (target 70-80% pure data); hand-written cards are resumable state machines that can only emit events, never mutate state directly.
6. **Verification is built around role separation**: a spec-writer agent turns rulings into scenarios before implementation, a different agent implements, a holdout set the implementer cannot see guards against overfitting, and mismatches against Forge are *findings* triaged against the CR, not verdicts.
7. **Core freezes after milestone M3**; changes need an RFC, two independent reviewers, and green golden-replay, differential, fuzz, and non-interference gates.
8. **Loop shortcuts are optional and shelved** (doc 02 section 9.6; Brady walked the idea back, so they are not in the initial build): the Aluren + Acererak loop would be shortened by an *executed*, not asserted, replay of the last iteration N times, following CR 731 (the opponent may accept or shorten it). The engine stops at the first deviation, so a shortcut can never produce a state stepwise play could not, and an audit invariant (I18) checks it. Tax effects (Disruptor Flute, Defense Grid) naturally stop it.
9. **A deck is "done" only at the deck gate** (doc 04 section 8): all cards specified and diff-tested, rulings suite 100%, lockstep and fuzz volumes met, divergences approved with CR citations, and a human sign-off.

## What was verified, and what was not

Verified in this revision:
- **Forge source** (commit `fd5c996`, 2026-09-30, read from a local clone; not built or run): custom `PlayerController` is a supported shape, `sim` headless mode with a seed exists, `GameState` can inject scripted openings (library order, battlefield, stack), ability enumeration exists, yield/auto-pass is UI-layer, shuffles go through a replaceable `java.util.Random`. Details in doc 04 section 2.2.
- **Licenses:** Forge is GPL-3.0 and XMage is MIT, from their repository `LICENSE` files.
- **Comprehensive Rules** (effective 2025-11-14, via the Savecraft rules module): damage assignment order is gone (CR 510.1c), APNAP trigger ordering (603.3b), replacement ordering (616.1), SBA list (704.5), layers (613). **Forge still uses the old blocker ordering, so it diverges from the CR** (kd-0001 in doc 04 section 9). A newer CR may exist.
- **Oracle text** of 19 hard cards, cross-checked against Forge's card scripts (read only, nothing copied).

Still unverified:
- Runtime behavior of Forge (I did not build or run it): enumeration quality for macro actions, mana payment control, whether `putonstack` can express every scenario, whether random discard uses the shuffle generator.
- Whether the deck-set thread's Forge card-script coverage (every pool card has a script) means the cards behave correctly. It does not; that is what the differential harness is for.
- All performance figures are targets for the M0 spike, not measurements.
- My manual finding that the pool needs layers 4, 6, 7a, 7b and 7c but not layers 1-3, 5 or dependency ordering (doc 05, doc 01 section 10); `deps-scan` should confirm it mechanically.
- Rules text for energy, ninjutsu, escape, ward, miracle, replicate, stun counters, ascend, converge and flurry (doc 05 lists the expected engine impact; the spec-writer confirms each against the CR). Dungeons (CR 309, 701.49, 704.5t), The Ring (701.54), warp, mobilize and impending were checked against the CR text.

## Questions for Brady

1. **Dungeon edge cases.** All three dungeons are implemented, since Alurentell ventures into Lost Mine or Mad Mage to keep Acererak looping with Aluren, and finishes Tomb of Annihilation when a tax effect stops the loop. Which edge cases around completing or re-choosing dungeons matter to you?
   (Deck set: the final eight, 144 non-basic cards, is the working pool for doc 05; changes mean regenerating that inventory.)
2. **Human rules reviewer:** who signs off deck gates and accepted divergences?
3. **Post-mortems and hidden info:** may the post-mortem LLM see the full record after the game ends? (recommended yes; doc 02 section 8)
4. **Opponent decklist visibility** for agents: full list known (default, matches the brief's `sample_opp_hands`), archetype only, or none.
5. **XMage tie-breaker:** build the adapter now, or only if Forge-vs-engine ambiguity proves costly?
6. **Python training stack constraints** (for the batching API shape). Settled already: Rust, one local GPU, subscription-only budget.
7. **Wishes:** none are in the pool, so sideboard-fetching cards are not a concern unless the pool changes.

## Suggested next steps (not started)

- M0 spike (throughput and clone cost) can begin immediately and needs no further decisions.
- Phase 1 Forge work should confirm the runtime **(verify)** items in doc 04 section 2.2 as it builds the headless runner and enumerator, since the differential adapter reuses them.
- If this goes into a repository: say which one. Per the project's rules I will ask before pushing or opening a PR.
