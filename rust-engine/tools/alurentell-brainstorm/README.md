# Alurentell brainstorm (2026-10-14)

Brady's list of deck ideas (Chrome Mox, Emrakul vs Archon, more Chant, Veil timing, Ending vs white mana, Propaganda, other white cards, Theorist's Sanctum, dead second Aluren).

- `takes-2026-10-14.md`: one row per idea: engine support, what earlier work already showed, take.
- `problems-from-reports.md`: tally of recurring problems in 253 earlier Alurentell player reports (mana, dead duplicates, hand size, how the losses happened).
- `REPORT-archon.md` + `archon-games/`: Archon of Cruelty for one Ponder, game 1, 12 LLM games (5-7).
- `REPORT-archon61.md` + `archon61-games/`: Archon of Cruelty as a 61st card (Brady's follow-up), game 1, 12 LLM games (6-6).
- `REPORT-plains.md` + `plains-games/`: Plains for the Island, post-board, 12 LLM games (7-5).
- `REPORT-pe-list.md` + `pe-games/`: post-board list with 2 Petal, 3 Chant, 0 Prismatic Ending, 3 Aluren against the three tempo decks, 12 LLM games (8-4).
- `REPORT-eureka.md` + `eureka-g1-games/` + `eureka-post-games/` + `eureka-engine/`: 2 Eureka for 2 Ponder, game 1 (10-2) and post-board (7-5), 24 LLM games; `eureka-engine/` holds the scratch implementation of the card (core is frozen, nothing under `crates` changed).

Reproducing the game runs: the launcher is `archon-games/mkx.sh <id> <ur|ub|uw> <archon|plains> <A|B>` (the 61st-card version in `archon61-games/mkx.sh` also takes `arch61`, and needs `MAIN_N=61` in `match.py`'s 60-card check) (it expects the `llm2` working directory layout used by the earlier waves: `lg.sh`, `match.py`, `decks-base*`, the playbook, and the `llmgame-live` binary from `rust-engine-tron-payfix2/` in the shared project folder). Each game needs two player agents, one per seat, started from the task files it writes. The briefs are `archon-games/BRIEF-g1-archon.md` and `plains-games/BRIEF-post-plains.md`.
