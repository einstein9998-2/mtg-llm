# LvL second night (g44 to g112): results (2026-10-07)

Same method as the first night (`RESULTS-lvl.md`): two Claude players per game on the live engine, Brady's main decks only, one opus reviewer per game that reads the truth log and drafts candidate rules and tough-spot flags. New this night: the macro (`lg.sh auto`) was reworked to Brady's Lost Mine technique, the player and reviewer briefs were updated, and games ran in a rolling pool of 4 until 21:00Z. Nothing new is in the playbook: every candidate rule needs Brady's sign-off.

## Short answer
- **69 finished games, Alurentell won 43 (62%)**: 24 of 35 against Boros (69%), 19 of 34 against UR Cutter (56%). Night one was 25 of 40 (63%). The overall rate did not move; UR got slightly worse.
- **Seat caveat**: this night the seat was tied to the opponent deck. Alurentell was always on the play against UR and always on the draw against Boros. Play/draw and opponent deck cannot be separated in these numbers.
- **Every win was the Lost Mine drain** (all by game turn 11, 29 of 43 by turn 7, as early as turn 2). Brady's technique (leave the room triggers on the stack except Dark Pool) worked in every game checked: no library loss, no legend-rule deaths. Night one's loop problems are gone.
- **Cost**: players 15.10M subagent tokens (219k per game), reviews 9.49M (138k per game), so **a reviewed game costs about 356k**. Synthesis 0.34M and the playbook restructure draft 0.12M on top. About 25.1M for the night.

## Why Alurentell lost (26 games, one primary cause each)
1. Wasteland and mana denial, made worse by Alurentell's own land handling: 9 (g53, g54, g57, g59, g66, g69, g71, g76, g83).
2. Keeps with no route to a payoff: 6 (g70, g79, g94, g107, g110, g112).
3. Variance against well-timed free counters or clocks: 5.
4. Combo-turn resources thrown away (discarded spare Atraxa, put back the second Acererak): 3.
5. Force of Will with no blue pitch card: 2.
6. Engine: 1 (g65, the Ancient Tomb auto-payer killed Alurentell at 2 life during a winning Show and Tell).

## Macro rework check
The reworked macro ran in the first reviewed game (g47) and every win after it with no library loss and no legend problems; budget was raised to 200 picks. One opponent player ran the macro past its stop condition once (g60b). The loop itself is no longer a source of losses.

## Harness and engine issues
Full list with game ids in `lvl/SYNTHESIS2.md` section 4. The ones that matter:
- **Ancient Tomb auto-pay** prefers Tomb and over-taps, costing life (g65, g72, g79, g81, g105, g111). It decided g65. Fixing the auto-payer is the single most useful engine change.
- **No priority window after blockers** when the player has no castable spell (g105, g111).
- **The loser sees nothing after the game ends**, so about 15 reports named the wrong kill turn or cause. A final event digest would fix it.
- Lands offer no mana abilities in the menu, so City of Traitors mana cannot float across a land drop.
- Ascend and city's blessing timing (g94). Duplicate "Cast X" menu labels (under Omniscience the later one is the free cast). Boseiju channel fetches nonbasic lands for the opponent (real card: basic only).

## Candidate rules and open questions
`lvl/SYNTHESIS2.md`: 36 candidate rules grouped under the 14 principles of the restructure draft, 15 conflicts with Brady's existing lines (each with evidence on both sides), deduplicated opponent notes, and a section on how often player reports misstate facts.

## Playbook restructure draft
Brady's idea: organise decision-making as a short playbook of principles plus skill files, and treat specific rules (for example "Brainstorm, fetch, Ponder") as cases of a principle ("see as many new cards as possible"). Draft in `lvl/restructure/`: `playbook.md` (60 lines, 14 principles), 11 skill files, and `MAPPING.md` (every old line mapped to its new home, 11 open conflicts). No rule was added or changed in meaning.

## Tough spots
`/mnt/project-files/tough-spots/llmgames/s9/` (tag s9, orders 335 up): `batch-priority.json` has the 79 reviewer likely-mistake positions, `batch.json` all 493. Not seeded on the answer page.

## Caveats
Small samples (+-12 points), same model on both sides, reviewers have hindsight and the player reports often misstated facts. g113 was launched after the 21:00Z cut-off (the sandbox clock lagged) and stopped without a result.
