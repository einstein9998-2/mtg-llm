# Forge headless runner

Forge (Card-Forge/forge, HEAD fd5c996, 2026-09-30) built from source and driven through its built-in
`sim` mode (AI vs AI, no GUI). `run_batch.py` launches W parallel JVMs (Forge's sim loop is single-threaded,
so parallelism = JVM count), parses per-game results and writes `results/<tag>.json` + `.csv`.

## Reproduce
```
git clone --depth 1 https://github.com/Card-Forge/forge /home/claude/card-forge/forge   # GIT_LFS_SKIP_SMUDGE=1
cd /home/claude/card-forge/forge && mvn -q -B -DskipTests -Dcheckstyle.skip -pl forge-gui-desktop -am package   # ~3 min
cd /mnt/project-files/forge-runner && python3 run_batch.py --workers 4 --games 30 --tag w4_n30
```
Java 21, Maven 3.9.11. Run from `forge-gui/` (Forge finds `./res` relative to cwd).

## Matchup (placeholder, not the archetype set)
`decks/delver.dck` UR Delver (60, no sideboard) vs `decks/jund.dck` Jund (60, no sideboard).
Chosen only because the brief wanted two fair non-combo decks. Lists are written from memory, not tournament
results; all card names resolve in Forge's card scripts. The archetype thread should replace them.

## Results (sandbox: 4 vCPU, 15 GB, JVM -Xmx1g per worker, ParallelGC, 1 CPU per worker)
| run | JVMs | games | wall | games/sec (wall, incl. JVM startup) | steady-state games/sec* | mean ms/game | p90 ms |
|---|---|---|---|---|---|---|---|
| w1_n30 | 1 | 30  | 66.5 s  | 0.45 | 0.54 | 2007 | 3368 |
| w4_n30 | 4 | 120 | 104.5 s | 1.15 | 1.57 | 2921 | 5570 |

| dnt_vs_jund_w4_n30 (deck-set lists) | 4 | 120 | 124.9 s | 0.96 | 1.38 | 3409 | 5492 |

*steady-state = games 4+ of each worker, aggregate over workers (excludes JIT warm-up). 4 JVMs give ~2.5x one JVM
on 4 cores; per-game time rises ~45% under contention. Mean game length ~8.7 turns. No draws/timeouts, all JVMs exit 0.
Win counts (Forge AI both sides, seeded, per run): w1 Delver 13 / Jund 17; w4 Delver 42 / Jund 78.
These are sanity numbers, not a matchup claim (placeholder lists, default AI profile, play/draw not controlled).

## Takeaways
- Order of magnitude: ~1 to 1.5 full games/sec on a 4-core box, i.e. roughly 4k games/hour on the whole box (~1k per core). Fine for LLM-player
  evaluation (LLM latency dominates), far too slow for RL-scale self-play; supports the Phase 2 engine argument.
- JVM + card DB startup is ~6-10 s per process, so batch many games per JVM.
- Stdout noise: `X Did not have activator set in SpellAbilityRestriction.canPlay()` is logged for many cards (incl. Murktide
  Regent, Daze, FoW) during AI simulation; likely harmless AI-lookahead logging but should be confirmed in the interaction test suite.
- Not measured yet: decisions/sec, per-decision latency of an external player (needs Forge's network/player-controller hook), seat/on-the-play control.

## Update: deck-set thread's lists
`decks/dnt.dck` (Death and Taxes) vs `decks/jund75.dck` (Jund), converted from `/mnt/project-files/decks/*.txt` with
`txt2dck.py` (main 60 + sideboard 15; sim plays game 1 main only). 4 JVMs x 30 games: 0.96 games/sec wall, 1.38 steady state,
mean 10.1 turns, no draws. Wins (Forge AI both sides): D&T 49, Jund 71. All card names resolve (split cards like Wear // Tear are
named that way in the sideboard). Use this as the reference throughput for the first pipeline matchup.
