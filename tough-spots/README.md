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

## Rules guard (`--rules`, 2026-10-07)
`src/rules.rs` removes options the search may not pick, from Brady's batch 1 and 2 answers: no cantrip (Brainstorm, Ponder, Stock Up) in your own upkeep; in your own main phase with the stack empty and a land drop available, no passing and no committal spell (Show and Tell, Aluren, Atraxa, Acererak, Omniscience) before the land. Only the flagger bot uses it; the training and self-play bots do not.
`rescore` re-runs the search on labeled positions with and without the guard (`rescore <decks> <net> batch1/labels.jsonl batch2/labels.jsonl`). It must be built against the engine the net was trained on (core-frozen-m5 initial import with the old Alurentell list), otherwise the replay and the net do not match.
Result on 40 games vs UR Cutter with the m1 net (seed 51): guard off 27 wins, guard on 31 wins (noise about 7 games); the guard intervened about 5 times per game. On the 22 labeled positions agreement with Brady went from 8 to 9 (deep search) and stayed at 7 (quick search).

## Several decks (batch 3, 2026-10-07)
`build_multi_batch.py runs_dir out tag batch_no first_order` picks two positions per matchup run (`tough-spots --me A --opp B --out runs/A-vs-B`, pool8 net for both seats, old engine) and writes one batch with `matchup` and `me_deck` on each card; the answer page shows them. Batch 3 = 14 positions over Boros/BW Taxes, Dimir/UWx, Doomsday/Reanimator and UR Cutter (vs Alurentell), orders 27-40. The rules guard only applies when the bot plays Alurentell. `import_answers.py` now writes me/opp into the labels.

## LLM player on the labeled positions (2026-10-07)
`llm_prompts.py` writes a blinded prompt per labeled position (the card as Brady saw it, no bot numbers, no bot choice, no Brady note; Alurentell positions get the playbook as it was BEFORE his tough-spot review). One fresh Sonnet agent per position, one answer each (`llm-eval/out`, `results.tsv`). About 50k tokens per agent, 1.8M in all, most of it fixed agent overhead.
Result on 36 labeled positions (batches 1 to 3): LLM agrees with Brady on 21 (58%); bot quick search 10 (28%), bot deep search 12 (33%). Alurentell only (23): LLM 14, bot 7. Other decks (13): LLM 7, bot 3.
Where the LLM and the bot give the same answer (10), Brady agrees on 7. Where they differ (26), Brady sides with the LLM 14 times, with the bot 3 times, with neither 9.
Caveat: the positions were flagged because the bot found them hard, so the bot's rate is biased low; 36 positions is a small sample.
