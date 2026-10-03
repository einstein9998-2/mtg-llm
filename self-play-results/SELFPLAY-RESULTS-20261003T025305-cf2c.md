# Phase 3 self-play + training results (RTX 3060 box, engine v4, tag core-frozen-m4)

Recipe (adopted after matchup 1): expert iteration with determinized MCTS. Gen 0 = rollout-guided search (32 iters, 1000 games);
gens 1-3 = net-guided search (64 iters, 2000 games each) with the previous net; net is the mtg-agent MLP (hidden 512, emb 64),
trained 2 epochs per generation on all data so far, AdamW lr 3e-4, weight decay 0.01, value target = 0.5*result + 0.5*search root value,
best validation epoch kept. Arena: 200 games, 32 search iterations, seats alternate, vs random and vs rollout-guided MCTS (same iterations).

| Matchup | Net | vs random | vs rollout MCTS |
|---|---|---|---|
| 1 UR Cutter vs Alurentell (earlier recipe, 256 hidden, 4 epochs) | gen 3 | 183-17 | 126-74 |
| 1 UR Cutter vs Alurentell (512 hidden, 2 epochs, lr 3e-4, trained on all gen 0-3 data) | v3 net512 | 190-10 | 145-55 (72.5%) |
| 2 Boros Aggro vs BW Death and Taxes (adopted recipe) | gen 3 | 195-5 | 153-47 (76.5%) |
| 3 Dimir Tempo vs UWx Control (adopted recipe) | gen 3 | 199-1 | 174-26 (87.0%) |
| 4 Doomsday vs Reanimator (adopted recipe) | gen 3 | 179-21 | 155-45 (77.5%) |
| Full 8-deck pool (hidden 768, 2000 rollout + 3x4000 net games, arenas of 300) | gen 3 | 216-83-1 (72%) | 169-131 (56.3%) |

Reference: rollout MCTS (no net, 32 iters) vs random on matchup 1 was 50-10 over 60 games.

Throughput: rollout-guided data 0.3-0.9 games/s (matchup 2 slower: 1000 games in 55 min while the PC was shared); net-guided data 3-6 games/s at 64 iterations on 11 threads (Ryzen 5 5600X); training 1-2 min per generation on the GPU.

Full 8-deck pool: finished (runs\pool). The first attempt aborted with a 56 GiB allocation failure, caused by an infinite resolution loop in "Ajani, Nacatl Avenger"
in the engine core (see ENGINE-BUG-ajani-runaway.md; fixed upstream in core-frozen-m5). The rerun used the new crash isolation, which found and skipped game 1449 on its own.
Pool data: 2000 rollout games (41 min), then 3 net-guided generations of 4000 games (13-14 min each, ~5 games/s); 2.0M training rows; held-out value MSE 0.17 (zero baseline 0.57),
top-1 agreement with the search 53%. Total wall time about 4 h 06 min including the two 300-game arenas (rollout-search arena 21 min). The pool net is clearly weaker than the single-matchup nets:
one net must cover 8 decks and every mirror, and each deck pair gets far less data. All pool numbers are on engine v4 (pre-m5) and need regenerating on m5.

Notes: GPU is only used for training; search and net inference are CPU. A 200-game arena has about +-3.5 points of noise.
Files (on Brady's PC): C:\Users\Brady\claude\mtg-llm\rust-engine\runs\m1..m4 and runs\pool (data, nets *.bin, logs). Nothing pushed.
