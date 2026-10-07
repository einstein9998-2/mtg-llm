# Tough-spot flagging for the Alurentell bot

Built 2026-10-06 (thread "tough situations", Brady's ask). Agent-side only: built against `mtg-view` without `diff-harness`, no change to the frozen core.

- `src/main.rs` (`tough-spots` binary): plays Alurentell (net-guided determinized MCTS, 64 its) vs UR Cutter (same net, 32 its) and, at each real bot decision, records net priors/value, the search result, and, when the position is contested (top two options within 5% win chance, or net favourite is not the search's choice, or a close mulligan), two deeper searches (400 its each). Flagged positions are written to `positions.jsonl` with the rendered view (the bot's own Observation only), option table, state encoding and replay actions.
  Build: `cd tough-spots && CARGO_TARGET_DIR=/home/claude/target-tough cargo build --release` (path-deps on `../rust-engine`).
  Run: `tough-spots <decks dir> --net <net.bin> --out DIR --games 100 --threads 4 --seed N` (about 3 s per game on 4 threads).
- `select_batch.py`: picks a varied batch (keeps, cast-or-wait, cast-which, card picks, ...), prefers turns 1-8, one per game, and writes `batch.json` (for the page), `batch.md`, `batch-private.jsonl`.
- `answer-page.html`: the answer page (Artifact with the `db` capability: collection `positions` = cards, `answers` = Brady's picks). New batches are added by writing `positions` docs; no republish needed.
- `import_answers.py`: Brady's answers -> `labels.jsonl` (labeled positions) + `notes.md` (rule candidates).
- `batch1/`: the first ten positions.

How answers are used: (1) agreement rate between Brady and the bot on flagged positions is the first metric (held-out set); (2) once there are a few hundred labels they are mixed into policy training as extra targets (weight 1, or 0.5 for "either is fine", 0 for "can't tell"); (3) notes ticked "Make this a rule" are drafted into `llm-player/alurentell-playbook.md` for Brady's sign-off.

Known limits: opponent is the net's Cutter (passive; it often sits on 7 cards), games run long; value estimates are noisy to a few points; the net was trained on the old Alurentell list (no Savannah/Orim's Chant yet).
