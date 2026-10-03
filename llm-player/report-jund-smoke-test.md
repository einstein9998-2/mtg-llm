# No-tools LLM player: Death and Taxes vs Forge AI (Jund)

> **Status (2026-10-01): provisional.** Brady objected that the Jund list is not a real Legacy deck, and the deck set is being revised. These five games are a pipeline smoke test against the old list and do not count toward any win rate. Counted games wait for the new first matchup in `/mnt/project-files/decks/`.

Date: 2026-10-01. Player: Claude (this thread), subscription only, no API, no tools beyond the serializer prompt. Opponent: Forge's built-in AI on Jund. Engine: Forge + the Phase 1 serializer (External policy over named pipes), `default` stop preset from game 2 on (game 1 used `minimal`).

## Result

| Game | Seed | Seat | Stops | Result | Turns | Prompts answered | Prompt tokens (chars/3.6) |
|---|---|---|---|---|---|---|---|
| g01 | 101 | draw | minimal | loss | 29 | 54 | ~11.1k |
| g02 | 102 | draw | default | win | 16 | 67 | ~9.8k |
| g03 | 103 | play | default | loss | 18 | 45 | ~6.2k |
| g04 | 104 | draw | default | win | 22 | 114 | ~22.3k |
| g05 | 105 | play | default | win | 9 | 67 | ~9.9k |
| **Total** | | | | **3 W / 2 L** | | **347** | **~59k** |

- Win rate 3/5 = 60% (Wilson 95% interval about 23% to 88%). That is not distinguishable from the Forge-AI-vs-Forge-AI baseline for this matchup (D&T 49 wins in 120 games = 41%). Five games cannot say whether the LLM plays better or worse than the built-in AI.
- Luck matters here: g05 was won against a Jund hand full of red cards with no red source; g02 and g03 turned on Liliana/Bolt draws.
- Roughly 12k prompt tokens per game at the default stops (about 70 prompts), consistent with the serializer's earlier estimate (~10.8k tokens, 57 prompts). This counts only the text the engine sent, not my reasoning output and not the tool-call overhead of re-reading the conversation on every pick, which was the real subscription cost. One game was 50 to 115 sequential tool calls.

## Blunders (judged after each game against engine ground truth)

Method: after a game ended I ran `review.py`, which lists each decision next to the opponent's real hand, my real library top, and the Forge AI's own priority pick (recorded only in `decisions.jsonl`, never in the prompt). I judged each decision by hindsight with full information. There was no engine fork or search, so "blunder" means "I can name a clearly better line that the known information at that moment supported", not "worse by simulated equity".

| Game | Costly blunders | Slips / reasoning errors (no cost) | Notes |
|---|---|---|---|
| g01 | 0 clear, 1 arguable | 0 | Long grindy loss. The arguable line is in the g01 notes (`runs/g01/player.jsonl`). |
| g02 | 0 | 1 | I expected Sword of Fire and Ice's 2 damage to kill Orcish Bowmasters before its draw trigger. The trigger resolves first. Mother of Runes covered it. |
| g03 | **1** | 1 | Decisive error: I held Armageddon as "discard fodder" instead of casting it while ahead on board. Also a chained-pick slip that skipped a land drop (recovered in MAIN2). |
| g04 | 0 | 0 | |
| g05 | 0 | 0 | |

- Costly blunders: 1 in 5 games (0.2 per game), 1 in 347 prompts (0.3%).
- Counting slips and reasoning errors as well: 3 in 347 prompts (0.9%), with 1 arguable line extra.
- Where the Forge AI hint disagreed with me (about a third of priority prompts), nearly all were noise: the AI passes at MAIN1 and casts in MAIN2, or wants to pump Jitte on its own turn. Disagreement is not a usable blunder detector. The real review work was reading each non-trivial decision.

## What the harness could and could not do

- Worked: the serializer prompts were sufficient to play full games including Vial, SFM, Jitte, Revoker naming, combat, Karakas, Wasteland lines. No hidden information was visible to me during a game (opponent hand, library order only appear in the log).
- Limitation 1, mana payment is delegated to the Forge AI. It sometimes taps Plains for generic costs, which blocked a second WW spell in a turn (g03/g04). I sequenced spells to reduce the risk. A real fix needs a payment choice in the policy interface.
- Limitation 2, `minimal` stops give no instant-speed windows (no Mother of Runes in combat, no end-of-turn Vial). `default` is the minimum for D&T; it costs about 20% more prompts.
- Limitation 3, the opponent plays only the game 1 main-deck 60 (no sideboard, no game 2 and 3 plans).
- Limitation 4, one `pick` per call. Chaining two picks in one call once skipped a decision (g03), and once matched by luck (g05, a Vial search sequence). Do not chain.

## Recommendations

1. Treat these results as a smoke test of the loop and the prompts, not as a strength measurement. 40+ games per matchup would be needed to compare with the Forge AI.
2. An LLM-in-the-loop at this rate is cost-bounded by the subscription, not by the engine: about 70 tool calls per game. It suits generating labelled reviewed games and spot-checking a neural player, not self-play volume.
3. Add a payment-choice decision and an opponent-visible "last N log lines" check to the serializer, since both caused my only avoidable mana losses.
4. Keep `review.py` plus the shadow-hint patch: they make blunder auditing fast and leak nothing into prompts.

## Files

- `play.py` broker (`start`, `next`, `pick`), `review.py` post-game review, `shadow-hint.patch` (diff over `serializer/src`; the original serializer is unmodified).
- `runs/g01 ... g05`: `decisions.jsonl` (state, menu, my pick, Forge AI hint and ground truth; do not read during a game), `player.jsonl` (my pick and one-line reasoning for every decision), `games.jsonl`, `review.txt` for each game.
- Nothing was pushed, merged or opened as a PR.
