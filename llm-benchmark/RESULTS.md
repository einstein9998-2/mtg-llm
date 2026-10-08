# LLM player vs search: Alurentell vs UR Cutter on the Rust engine (2026-10-05/06)

Engine: core-frozen-m5 (v5), copied from `/mnt/project-files/rust-engine/` at the start of the run and **pinned**: no Orim's Chant (175 card defs), Alurentell list identical to `decks/superseded-v3/alurentell.txt`. Everything ran in the cloud sandbox, nothing on Brady's PC. The matchup-1 net (`nets/m1-alurentell-vs-ur-cutter-v5-net3.bin`, gen 3, v5) is valid on exactly this engine and list; it is not valid on the newer engine/list.

## Setup
- **LLM side: Alurentell.** One fresh Claude per game, no search or tools beyond the game commands and card-text lookup. It plays from text prompts rendered from its `SeatView` only (hand, both boards, graveyards, stack, opponent hand size, events since its last prompt) and picks option indices from the same legal-action menu a search agent gets. The engine skips windows where the only action is Pass.
- **Opponent: UR Cutter played by determinized MCTS, 32 iterations**, in two versions: (A) rollout evaluator, (B) the matchup-1 net as evaluator (the "net + search" player from the brief).
- 20 games per opponent, LLM seat alternating (10 on the play, 10 on the draw), seeds 101-120 (A) and 201-220 (B).
- **Baselines** with the same driver, 200 games each, seat alternating: search playing Alurentell instead of the LLM, rollout evaluator or net evaluator, both 32 iterations.
- Hidden information: the driver holds the real game but writes only the SeatView rendering for the player; its own log is written after the game ends. Players were told to read nothing else; I did not audit their tool calls beyond that.

## Results (Alurentell win rate, 95% Wilson interval)
| Alurentell player | vs rollout-search Cutter (A) | vs net-search Cutter (B) |
|---|---|---|
| **Claude (LLM)** | **19-1, 95% (76-99%)** over 20 games | **18-2, 90% (70-97%)** over 20 games |
| Rollout search, 32 it | 124-76, 62% (55-68%) over 200 | 68-132, 34% (28-41%) over 200 |
| Net + search, 32 it | not run | 112-88, 56% (49-63%) over 200 |

Reading it: the net-search Cutter is a clearly harder opponent than rollout-search Cutter (rollout Alurentell drops from 62% to 34%), and the LLM barely drops (95% to 90%; the two LLM results are not distinguishable at n=20). Against the same net-search opponent the LLM beats net-search-as-Alurentell, 18 of 20 vs 56%, roughly p=0.001 if the true rate were 56%. Losses: g115 (one-land, no-blue keep, never cast a coloured spell), n203 (no Show and Tell found, died to Cori-Steel Cutter Monks), n213 (completed Tomb of Annihilation into an opponent with open Aluren targets; its two free Dragon's Rage Channelers got delirium and flew over). Several wins came after mulligans or keeps with 1-2 lands against a stuck opponent.

## Cost per game
| (40 games) | LLM vs A | LLM vs B | search (any) |
|---|---|---|---|
| Prompts the LLM answered, mean (median, range) | 114 (89; 18-375) | 142 (110; 35-391) | none |
| Prompt text per game | 178k chars | 245k chars | |
| Subagent tokens per game, as reported by the harness | 157k mean | 176k mean (156k median; 90k-387k) | 0 |
| Wall clock per game | 12 min | 12 min | about 5 s on one CPU core |

- Total for the 40 LLM games: about 6.7M subagent tokens, 5,100 prompts. That is the quota cost of this benchmark. I can't convert it to dollars or to a share of Brady's quota: the harness figure does not look like cumulative context reads (the 362-tool-call game reports the fewest tokens), so treat it as a lower bound.
- Cost per win: the LLM spends about 175k subagent tokens per game for a 90-95% win rate; net search spends no tokens and gets 56% for the same seat. In win rate per dollar, search wins by a wide margin whenever its cost is only CPU; the LLM is the higher-ceiling, much slower and non-free option.
- Long games are the Acererak/Lost Mine loop (about 4 prompts per cast, 4 casts per drain point): 375-391 prompts in the worst games where the loop alone drained 20 life, versus 35-60 prompts for games won by Show and Tell into Atraxa. Players noted afterwards that attacking or completing Tomb earlier would have saved 60-200 prompts.

## Caveats
1. The opponents are still weak players. Even the net-search Cutter Dazes its own spells, Wastelands its own lands and casts Unholy Heat on its own creatures; a strong Cutter would be a harder test. Alurentell is also the stronger side in this matchup (net vs net: 56% for Alurentell).
2. 20 games per cell: the intervals are wide (70-97% for the net result).
3. The LLM players had the Alurentell playbook and later a few menu tips added to `harness/BRIEF.md` (duplicate Cast entries, "Do it?" meaning, scry/surveil buttons, from wave 2 on and again before the net run); games 101-104 (and 201-204) had fewer tips.
4. The LLM runs and the baselines share seeds but not trajectories, so only aggregates compare.
5. Prompts are not hash-checked for leaks like the Forge serializer was; the `SeatView` type is the barrier, with its own non-interference tests in the engine.

## Issues the players hit
Engine, rules or card-text:
- **Possible rules bug: Daze vs Veil of Summer.** Two players reported Daze still asking for payment (or countering) with Veil of Summer's "can't be countered" active (n211, n219). Not verified by me; worth a scenario test.
- Daze offers no payment prompt when there is no untapped mana; the spell just gets countered (reported twice).
- Wasteland shows "targeting (gone)" after resolution (cosmetic).
- Dungeon completion marker updates only after the last room's ability resolves; "completed dungeons" repeats per lap (cosmetic).
- After completing Tomb of Annihilation, a second Acererak cast is still offered and would hit the legend rule.
- One unverified report (g110): after Ponder put a Petal on top, Stock Up's five cards did not include it (probably a misread).

Prompt/menu clarity (most cost players mana, life or prompts):
- Duplicate unlabelled `Cast X` entries when two ways to cast exist (hard cast vs Aluren/Omniscience free cast). The later entry was the free one every time, but many players picked the hard cast once, costing mana and Ancient Tomb damage. The labels should say which is free.
- `Do it?` on Tomb room payments does not say Yes means pay (discard/sacrifice); same label as Ponder's shuffle. Several players declined and took the life loss for fear the engine would pick their sacrifice (it does let you choose).
- Scry/surveil: `Choose X` bottoms or bins the card and `Done` keeps it, which reads backwards.
- Auto-payment taps Ancient Tomb (2 damage) and Lotus Petal when other mana would do, with no way to choose; City of Traitors cannot be tapped before a land drop.
- Legend rule prompt shows two identical Atraxa options with no ids (many players cast a second Atraxa under Omniscience and lost one copy; the playbook should say "only if no Atraxa is out").
- Attack declarations are one prompt per creature, which is expensive with a Goblin army.

## Conclusion for the self-play question
The LLM plays Alurentell competently from the plain-text menu and beats both search opponents, including the net-guided one. It is not a substitute for self-play data: it costs about 175k tokens and 12 minutes per game and produces no training signal on its own. It is useful as an evaluator, and as a teacher for the combo decks (the game logs and loop lines are reusable).

## Files
- `games.tsv` (LLM vs rollout Cutter, games 101-120) and `games-vs-net.tsv` (games 201-220): per-game numbers. `game-logs/`: opponent choices and result per game, written after each game.
- `baseline-rollout-vs-rollout-32it.tsv`, `baseline-rollout-vs-net-cutter-32it.tsv`, `baseline-net-vs-net-32it.tsv` (200 games each).
- `nets/`: matchup-1 net and its README.
- `harness/`: `llmgame.rs` (add `[[bin]] name = "llmgame"` to `crates/mtg-agent/Cargo.toml`; flags `--opp-net`, `--me-net`, `--auto-llm`), `lg.sh` player interface (reads `opp_net.cfg` for the net run), `BRIEF.md` player instructions, `baseline.sh`.
