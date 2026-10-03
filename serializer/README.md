# Phase 1 state serializer, legal-action enumerator and auto-resolution (on Forge)

Built on `/mnt/project-files/forge-runner/` (Forge fd5c996, Java 21). Decks: `dnt.dck` (Death and Taxes) vs `jund75.dck`.
Nothing here is pushed anywhere. ~1800 lines of Java in `src/silo/`.

## What it is

A custom Forge player controller (`SeatController`, subclass of `PlayerControllerAi`) that sits in one seat and:

1. **Enumerates legal actions** with Forge's own predicates and presents each real decision as a small, numbered, masked prompt.
2. **Auto-resolves** forced and trivial decisions (only-Pass windows, single legal target, our own stack items) without asking anything.
3. **Serializes state** for that seat only: full at turn start, diffs afterward, hidden information absent by construction.
4. Sends the prompt to a **Policy**. Anything the policy does not model falls through to the stock Forge AI and is counted as "delegated".
5. Logs every prompt, state and chosen action (`decisions.jsonl`).

## Build and run

```
bash build.sh                      # generates TracedAi.java, compiles to /home/claude/silo-build
bash run.sh --a dnt.dck --b jund75.dck --games 30 --seed 7 --policy hybrid --stops default --leak-test true
```
Options: `--policy random|first|mirror|hybrid|external|plain`, `--stops all|default|minimal`, `--log-prompts true`, `--pass-bias 0.5`, `--pipe-out P --pipe-in P` (external), `--out DIR`.
Run from any directory; the script handles Forge's `./res` cwd requirement.

Policies: **First** (first option), **Random** (with pass bias), **Mirror** (Forge AI decides everything, used for measurement and equivalence), **Hybrid** (Forge AI hint at priority, random on sub-decisions), **External** (JSONL over named pipes: this is where the LLM player loop plugs in; `tools/driver_example.py` is a reference driver), **plain** (stock Forge AI for both seats, framework control).

## Architecture

| File | Role |
|---|---|
| `SeatController` | Core. Decision hooks, forced auto-resolve, stop rules, priority enumeration, validation |
| `StateSerializer` | Sections: hdr, hand, ybf, obf, ygy, ogy, ex, cnt, stack, combat. Full at turn start, diff after |
| `Knowledge` | Per-observer visibility and ids. Cards get ids in first-seen order; Forge ids are never printed. Hidden cards have no id, only counts. An id is dropped when the card goes to a hidden zone, so a redrawn card is a new object |
| `EventLog` | Redacted event stream from Forge GameEvents, rendered by our own renderer (Forge's descriptions contain raw ids) |
| `Render` | Permanent flags, targets, stack items, labels |
| `Glossary` | Oracle text for every pool card, ~10.5k chars (~2.9k tokens). Cached prefix; prompts carry card names only |
| `Policy`, `Decision` | The interface the LLM harness implements |
| `StopConfig` | Stop-rule presets (all / default / minimal) |
| `LeakCheck` | Non-interference test (below) |
| `TracedAi` | Generated from `PlayerControllerAi` by `tools/gen_traced.py`: wraps 119 methods to count delegated decisions |

### Decision kinds (vs engine-design doc 02)

Priority, Target, ChooseCards (discard, sacrifice, search, entity choice, vote), ChooseOption (modes), OrderTriggers, YesNo (optional triggers, confirmations), Mulligan, BottomCards, Attack, Block (per creature), ChooseName (closed card pool), ChooseOption for protection type, counter type and number. Multi-step choices are separate small masked prompts: e.g. cast spell, then Target, then mode.

### Auto-resolution and stop rules

- Only-Pass priority windows are never prompted (90 per game on average).
- A stack containing only our own items is auto-passed (default preset).
- Empty-stack windows in each step are prompted or skipped by the stop preset. Stop presets give identical game trajectories (the mirror policy decides the same thing); only the number of prompts changes.
- Equivalent actions are deduplicated with a multiplicity, e.g. `Swords to Plowshares (x2)`.
- Mana abilities are not actions; mana is paid automatically.

### Hidden information

Enforced three ways: (1) the serializer reads only through `Knowledge.visible()`; hidden cards cannot be given an id; (2) Forge ids never appear; (3) tested by perturbation (next section).

## Evidence

All numbers: D&T as Seat-A vs Forge-AI Jund. Token counts are estimates (chars / 3.6, no tokenizer offline, about ±25%).

**Equivalence** (`results/equivalence.log`): Mirror policy through our controller gives the same games as stock `forge sim` for the same seed: 5/5 identical winner and turn count, and 10/10 identical to the framework control. Getting there found two real bugs (below).

**Non-interference** (`LeakCheck`): at every prompt, shuffle both libraries and the opponent's hand, swap unrevealed opponent-hand cards with library cards, re-render, and require an identical state text and identical priority menu; then restore. Also asserts no ids are held for hidden cards.
- Final runs: hybrid 30 games, 1757 checks, 0 failures; random with all stops 30 games, 929 checks, 0 failures. Earlier runs on other seeds: 1964 and 1241 checks, 0 failures.
- Negative controls (`results/leak_negative_controls.txt`): deliberately injected leaks of opponent-hand names, library top and raw Forge ids are detected in 49/52, 46/52 and 49/52 prompts, so the test can fail.

**Legality**: across all runs, no failed menu action, no illegal pick, no invalid attack or block declaration (`problems {}`).

**Enumerator completeness**: in mirror runs, the Forge AI's own priority choice was on our menu in 1646/1646, 1429/1429 and 2144/2144 cases. The one miss seen early was a bare mana ability, which is excluded on purpose.

**Prompt size and rate** (mirror, default stops, 30 games):

| | |
|---|---|
| Prompts per game | 56.7 (all stops 144.8, minimal 40.1) |
| Estimated tokens per game | ~10.8k (all stops ~18.1k, minimal ~8.0k) |
| Priority prompt tokens | p50 172, p90 351, max 619 |
| Avg chars per prompt | 685 (~190 tokens), with diffs. Full state every time would be 930 chars (~258 tokens) |
| Priority windows per game | ~231, of which 90 forced and 86 skipped by stop rules |
| Glossary (cached once) | ~2.9k tokens |

So the target of 400-800 tokens per prompt is comfortably met; the real cost lever is how many windows are prompted, not prompt length. Per game that is ~57 calls, so an LLM player on every prompt is still far too many calls for self-play, which fits the plan to keep the LLM out of inner loops.

**Overhead**: menu enumeration ~0.7-1.0 ms per call (~230 per game), about +14% game time (plain 2170 ms vs mirror 2484 ms per game). Render ~0.1 ms.

**External policy**: 3 games over named pipes, 157 decisions, no problems.

Samples: `results/sample-prompts.md`. Tables: `results/prompt_size_tables.txt`. Summaries: `results/summary_*.json`. Mirror summaries predate the id-hygiene fix, which does not affect game flow.

## Bugs found on the way (relevant to Phase 2)

- Forge's mana-affordability check (`ComputerUtilMana.canPayManaCost`) consumes the **global RNG**. Enumerating the menu changed the game. Fixed by running enumeration under a scratch Random (`quiet()`). A second divergence came from AI profile defaults, which Forge only applies to `LobbyPlayerAi` instances.
- `ComputerUtilCost.canPayCost` mixes AI preference into "can pay" (it refuses planeswalker ultimates at random), so we use our own affordability check (mana plus additional costs).
- For the custom engine: enumerators must be pure, engine legality must be separate from AI desirability, options must be sorted canonically so order cannot reveal hidden order, and ids must not survive a trip through a hidden zone.

## Known limitations

- **Delegated to the Forge AI**: mana payment and cost choices (`payManaCost`, `playSpellAbilityNoStack`), combat damage assignment, `chooseCardsPile`, replacement-effect choice, move-to-zone ordering, block and damage ordering, color choice. In mirror/hybrid paths `chooseCounterType` is also delegated.
- Standalone mana abilities are not offered (matters for combo lines that pump mana).
- Cards playable from the top of the library are not offered.
- Own-library knowledge after a Brainstorm-style effect and library odds are not exposed.
- The opponent's hand is tracked as known only via hand reveals or bounces.
- Stop rules skip some instant-speed windows. The default preset skips windows with only our own items on the stack; combo decks would need `all`.
- The card-name pool is limited to the two decklists; the glossary covers only those.
- Token numbers are estimates, and the mirror stats cover only the decisions we model.
- The controller reports `isAI() == true`, and the measurements are D&T vs the Forge AI, a weak opponent, yet our random-ish seats still lose most games (hybrid 3 of 30, random 0 of 30), so these runs measure plumbing, not play strength.

## Next steps

1. LLM player loop on the `External` policy (prompt prefix = glossary, one prompt per decision, JSON option picks). Start with `--stops minimal`.
2. Delegated decisions, starting with mana payment, so hybrid/LLM games are fully policy-controlled.
3. Add the remaining pool decks and combo-specific stop rules.
