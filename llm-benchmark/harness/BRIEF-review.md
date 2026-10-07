# You are the post-game reviewer for one LLM-vs-LLM Magic game

Two Claude players (A = Alurentell, B = UR Cutter, or another pairing named in your task) just played a full Legacy game. You may read everything, with hindsight:
- `/home/claude/work/llm2/runs/GAME/truth/transcript-a.txt` and `transcript-b.txt`: every prompt each player saw and the option it chose (A's and B's own views only; read both side by side to reconstruct what each knew).
- `/home/claude/work/llm2/runs/GAME/truth/log.txt`: engine log (summary line at the end).
- `/home/claude/work/llm2/reports/GAMEa.md` and `GAMEb.md`: the players' own after-action reports (they may be wrong or flattering).
- Deck lists `/home/claude/work/llm2/decks/*.txt` and the current Alurentell playbook `/home/claude/work/llm2/alurentell-playbook.md` (Brady's rules; read it first so you do not propose a rule that already exists).
Do not use any other tool or file. Read the transcripts selectively: start from the end, find the turn the game was decided, then walk back.

## What to produce (write to `/home/claude/work/llm2/reviews/GAME.md`, at most 70 lines, plus the flags file)
1. **Result and turning point**: who won, on which turn, and the one or two decisions that decided it (turn, prompt number, what was chosen, what was better and why, with the information the player had then).
2. **First real mistake per side**, if any (a mistake = a worse choice given what that player could see, not bad luck; say so when a loss was just variance).
3. **Candidate Alurentell playbook rules** (0 to 4). Each: rule text in Brady's style (imperative, one or two sentences), the evidence (turn and prompt), confidence (high/medium/low), and whether it duplicates or contradicts an existing playbook rule (quote it). Only propose a rule that would have changed a decision and that is likely to generalize beyond this game. If nothing generalizes, write "none".
4. **Candidate UR Cutter notes** (0 to 2), same format, for a future UR Cutter brief.
5. **Process and engine flags**: wasted tool calls, menu confusions, anything that looks like an engine bug (with the prompt number), land drops skipped, upkeep cantrips.
6. **Tough-spot flags for Brady** (a Legacy expert who will answer them on a page): write `/home/claude/work/llm2/reviews/GAME.flags.json`, a JSON list of 0 to 6 objects `{"seat": "a" or "b", "seq": <prompt number from that player's transcript>, "kind": "likely-mistake" or "close", "note": "one line: what was chosen, what the alternative was, why it matters"}`. Flag decisions where you think the player probably erred (give the better line) and genuinely close, high-impact decisions (keep/mulligan, jam or wait, Brainstorm or Stock Up choices, sequencing around Daze/Force/Wasteland). `seq` must be the number in `=== PROMPT n ===` of that player's own transcript and the decision must be one the player actually made (a `>>> CHOSE` line follows it). Players may have flagged some themselves (`/home/claude/work/llm2/flags/GAMEa.tsv`, `GAMEb.tsv`); do not duplicate those, add what they missed. Skip trivial or forced decisions and positions where the earlier play already made the question moot (for example a skipped land drop; flag that earlier mistake instead).
Be concrete and honest; do not pad. End your final answer with 3 lines: `RESULT: ...`, `TOP MISTAKE: ...`, `TOP RULE: ...`.
