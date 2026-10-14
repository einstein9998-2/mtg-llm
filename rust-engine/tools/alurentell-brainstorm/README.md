# Alurentell brainstorm (2026-10-14)

Brady's list of deck ideas (Chrome Mox, Emrakul vs Archon, more Chant, Veil timing, Ending vs white mana, Propaganda, other white cards, Theorist's Sanctum, dead second Aluren).

- `takes-2026-10-14.md`: one row per idea: engine support, what earlier work already showed, take.
- `problems-from-reports.md`: tally of recurring problems in 253 earlier Alurentell player reports (mana, dead duplicates, hand size, how the losses happened).
- `REPORT-archon.md` + `archon-games/`: Archon of Cruelty for one Ponder, game 1, 12 LLM games (5-7).
- `REPORT-plains.md` + `plains-games/`: Plains for the Island, post-board, 12 LLM games (7-5).

Reproducing the game runs: the launcher is `archon-games/mkx.sh <id> <ur|ub|uw> <archon|plains> <A|B>` (it expects the `llm2` working directory layout used by the earlier waves: `lg.sh`, `match.py`, `decks-base*`, the playbook, and the `llmgame-live` binary from `rust-engine-tron-payfix2/` in the shared project folder). Each game needs two player agents, one per seat, started from the task files it writes. The briefs are `archon-games/BRIEF-g1-archon.md` and `plains-games/BRIEF-post-plains.md`.
