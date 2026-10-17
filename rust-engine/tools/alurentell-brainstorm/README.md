# Alurentell brainstorm (2026-10-14)

Brady's list of deck ideas (Chrome Mox, Emrakul vs Archon, more Chant, Veil timing, Ending vs white mana, Propaganda, other white cards, Theorist's Sanctum, dead second Aluren).

- `takes-2026-10-14.md`: one row per idea: engine support, what earlier work already showed, take.
- `problems-from-reports.md`: tally of recurring problems in 253 earlier Alurentell player reports (mana, dead duplicates, hand size, how the losses happened).
- `REPORT-archon.md` + `archon-games/`: Archon of Cruelty for one Ponder, game 1, 12 LLM games (5-7).
- `REPORT-archon61.md` + `archon61-games/`: Archon of Cruelty as a 61st card (Brady's follow-up), game 1, 12 LLM games (6-6).
- `REPORT-plains.md` + `plains-games/`: Plains for the Island, post-board, 12 LLM games (7-5).
- `REPORT-pe-list.md` + `pe-games/`: post-board list with 2 Petal, 3 Chant, 0 Prismatic Ending, 3 Aluren against the three tempo decks, 12 LLM games (8-4).
- `REPORT-eureka.md` + `eureka-g1-games/` + `eureka-post-games/` + `eureka-engine/`: 2 Eureka for 2 Ponder, game 1 (10-2) and post-board (7-5), 24 LLM games; `eureka-engine/` holds the scratch implementation of the card (core is frozen, nothing under `crates` changed).
- `REPORT-eureka-veil-ending.md` + `eureka-veil-g1-games/` + `eureka-ending-post-games/`: Brady's follow-up, 2 Eureka for the 2 main Veil of Summer in game 1 (8-4) and 2 Eureka for the 2 Prismatic Ending in the post-board plan with Veil still at 4 (4-8), 24 LLM games.
- `REPORT-pe2-list.md` + `pe2-games/`: the Petal/Chant list again with a land trimmed instead of a Ponder (UR Island, UW Boseiju + 1 Ponder, UB unchanged), post-board, 12 LLM games (5-7).
- `REPORT-wasteland-lands.md` + `wasteland-games/`: stock no-white list with 3 basics, then a third Tropical Island for an Island, then an eighth fetch for the other Island, how much Wasteland interferes with colors, game 1, 36 LLM games (6-6 / 9-3 / 7-5).
- `REPORT-chant-veil.md` + `chantveil-games/`: Brady's league swap, 2 Orim's Chant in the main deck instead of 2 Veil of Summer (Veil to the sideboard), game 1, 12 LLM games per list against UR, UB and UW (Veil main 8-4, Chant main 7-5).

Reproducing the game runs: the launcher is `archon-games/mkx.sh <id> <ur|ub|uw> <archon|plains> <A|B>` (the 61st-card version in `archon61-games/mkx.sh` also takes `arch61`, and needs `MAIN_N=61` in `match.py`'s 60-card check) (it expects the `llm2` working directory layout used by the earlier waves: `lg.sh`, `match.py`, `decks-base*`, the playbook, and the `llmgame-live` binary from `rust-engine-tron-payfix2/` in the shared project folder). Each game needs two player agents, one per seat, started from the task files it writes. The briefs are `archon-games/BRIEF-g1-archon.md` and `plains-games/BRIEF-post-plains.md`.
