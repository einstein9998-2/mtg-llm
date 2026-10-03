# LLM player tools that need no engine forking

Plain command line tools the LLM player calls instead of doing arithmetic and card lookups in its head. Python 3 stdlib only. Nothing is pushed anywhere.

| Tool | What it does |
|---|---|
| `get_card NAME...` | Oracle text and rulings for the 149 names in the eight decks (main, sideboard, basics). Forgiving about case and `#id` suffixes, suggests names on a miss. |
| `library_odds [TERM...] --draws 1,3,5` | Hypergeometric odds over your remaining library: P(at least one), expected copies, joint "all" and "any". Terms: a card name, `type:Land`, `mv<=3`. `--list` prints the remaining library. `--top`/`--bottom` take cards you yourself know are there (Brainstorm put-backs, scry, mulligan bottoms). |
| `sample_opp_hands --n 5 [--seed S]` | Plausible opponent hands: their decklist minus every opponent card you have seen, drawn uniformly at their real hand size, plus any revealed hand cards. Also prints P(card is in their hand). |
| `notes_read MATCHUP` / `notes_write MATCHUP TEXT` | Per-matchup playbook, one markdown file per matchup. `notes_write` appends a dated entry; `--mode replace` rewrites the whole file (use it to consolidate, the limit is 32 KB; `notes_read` shows the first 8000 characters unless `--full`). |
| `view_update` | Folds the prompt you were just shown into your view (see below). |

`simulate` is left out on purpose (Forge cloning gaps are in `forge-cloning/report.md`).

## Running

```
cd /mnt/project-files/llm-tools
sh bin/get_card "Force of Will" "Atraxa"
export MTG_VIEW=/path/to/run/view.json MTG_MY_DECK=alurentell MTG_OPP_DECK=ur-cutter
sh bin/view_update --cur "$BROKER/cur.json"                  # after each prompt, or pass --cur to the tool itself
sh bin/library_odds --draws 3,5 "Atraxa, Grand Unifier" "Show and Tell" type:Land
sh bin/sample_opp_hands --n 5
sh bin/notes_read alurentell-vs-ur-cutter
echo "Veil of Summer first when they hold Force of Will" | sh bin/notes_write alurentell-vs-ur-cutter - --game a07
```
(The folder is mounted noexec here, so use `sh bin/X`, or `PYTHONPATH=. python3 -m mtgtools X`.) Every command also takes `--json`. Decks are names in `/mnt/project-files/decks/` or a path; `--pool 75` makes the opponent pool main plus sideboard for later games.

## The view: the only game input

The tools never read the engine. They read a **view file**, a running fold of the prompts the serializer showed this seat. A prompt carries full state at turn start and changed-only sections afterwards, so the tools keep the latest value of every section (hand, both battlefields, graveyards, exile, counts, stack, life). `view_update --cur FILE` folds the `{game, id, prompt, seq}` decision message (the broker's `cur.json`) idempotently; `--text -` folds raw prompt text from stdin. Any other field in a message (for example a `truth` field) is ignored and never stored.

**Integration needed from the player loop:** every prompt the player sees must be folded, or the diffs lose sections. Pass `--cur $BROKER/cur.json` on each tool call, or call `view_update` after each `next`. If the broker's `seq` skips a message the view is marked **stale** and every tool prints a warning until the next full-state prompt (turn start). I did not edit `llm-player/`. `sh bin/tplay start|next|pick ...` wraps `play.py` and folds every decision it prints into the view (it reads only play.py's stdout, the player's own prompt), so the player can use `tplay` in place of `play.py` and never has to think about `view_update`. It resets the view on `start`, and a skipped `seq` marks the view stale. Tested against a stub broker, not against a live game.

## What the numbers mean

- **Own library** = your decklist minus own cards in hand, battlefield, graveyard, exile and own spells on the stack (tokens excluded). Cards you put back or bottomed are still in the library, as they should be. The view's `YOUR LIB n` is cross-checked against the subtraction; a mismatch prints a warning.
- **Opponent hands** assume you know their 60 (closed pool). Uniform over consistent hands means it does **not** model their play: an AI that kept a no-land seven or held a counterspell is not weighted. It is for determinizing `simulate` later and for "could they have Force of Will" odds.
- **Library odds** count cards *seen from the top*: draws, or a look-at-top-N effect (Stock Up 5, Ponder 3, Atraxa 10).
- **Rulings are curated**, from the Comprehensive Rules, not Gatherer (api.scryfall.com and gatherer are blocked here, only GitHub, Maven and PyPI are reachable). 21 cards have entries. `data/import_scryfall_rulings.py oracle-cards.json rulings.json` swaps in the official ones when someone downloads the two Scryfall bulk files; it is only tested on a tiny fixture. Oracle text is exact: it comes from the Forge card scripts at the engine's commit (`data/build_cards.py`, `fd5c996`).
- `notes/alurentell-vs-ur-cutter.md` is a copy (2026-10-01) of `llm-player/alurentell-playbook.md`; the copy is the one the tool reads and writes.

## Evidence (all in `tests/`, run `python3 -m unittest discover -s tests -p "test_*.py"`, 30 tests)

**Hidden information** (`test_isolation.py`). Each tool runs under a Python audit hook that records every file opened, network connection and subprocess. Allowed: tool code, static data, the two decklists, the view file and the notes directory. A decoy `decisions.jsonl` and `engine.out` containing a fake "true opponent hand" sit next to the view. Results: no tool opens anything else, none touches the network; a deliberately leaky variant that reads the decoy **is** flagged (negative control); outputs are byte-identical when the decoy's hidden world is replaced by a different one; and the tool source never mentions engine logs, run folders, or truth fields. Inputs are structurally limited to what the serializer already redacts, and the serializer's own non-interference test covers that side.

**Replay of the recorded games** (`test_recorded.py`, all `llm-player/runs/aNN`, 19 games and 2,061 decision states when I ran it; only the `prompt` text goes to the tools, the engine's `truth` field is read by the test as an answer key):
- Own-library accounting equals the state's `YOUR LIB` count on 2,058 of 2,061 states. The three misses (a06, a10, a15) are the Force of Will pitch-cost prompt, where the card being cast is in neither hand nor stack; the tool prints its count warning there.
- The opponent's real hand, size and contents, is inside the sampler's support in 100% of states (the decklist minus seen cards always contains it).
- No warnings on any ordinary priority prompt.
- Calibration of "P(card in their hand)" is in `results/calibration.md`: close at 0.15 predicted vs 0.17 actual and 0.35 vs 0.33, with deviations of 6 to 11 points at 0.25 (0.19 actual) and 0.41 (0.52 actual) where the Forge AI plays or keeps cards. Log loss 15.8k against 18.5k for a naive same-probability baseline.

**Math** (`test_calc.py`): hypergeometric results equal brute-force enumeration (including overlapping groups and known top/bottom cards) and Monte Carlo; sampled hands respect pool counts, hand size and revealed cards; same seed gives the same hands.

## Known limits

- Stale view if a prompt is never folded (flagged only when `seq` is available).
- Stolen or copied cards, cards fetched from the sideboard and cards removed from the game face-down are not modeled; they show up as a count warning.
- No knowledge of your library order beyond `--top`/`--bottom` that you pass yourself. The serializer does not currently expose Brainstorm/Scry results to the view.
- Opponent deck must be given; there is no inference from seen cards.
