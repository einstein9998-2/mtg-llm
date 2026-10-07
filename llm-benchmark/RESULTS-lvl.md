# LvL (LLM vs LLM) overnight run: results (2026-10-07)

Thread "LLM player with net tool". Two fresh Claude subagents play one game, each from its own view, on the live engine (222 definitions, Storm, Tron, Orim's Chant) with Brady's current 75s, main deck only. Seeds 3000+N; seats alternate. Every finished game gets a separate reviewer Claude that reads the full log with hindsight and drafts candidate playbook rules and tough-spot flags. Nothing here is in the playbook yet: every rule needs Brady's sign-off.

## Short answer
- **40 finished games** (4 early pilot games w1g1-w1g4 plus g5-g42, minus g41 and g43 which stalled and were stopped). g21 is tainted (see below) and excluded from win rates.
- **Alurentell side wins 25 of 40 (62.5%)**: vs UR Cutter 13 of 21, vs Boros 12 of 19. Small samples, about +-15 points. From g15 on (the first games with the loop macro and the cost cheat-sheet in the brief): UR 7 of 12, Boros 9 of 14.
- **Cost per game**: players alone 270k subagent tokens (361k before g15, 224k from g15 on); a review adds 152k. **A reviewed game costs about 424k on average, about 370k from g15 on.** The coordinator's estimate of 400k held. Whole run: 17.1M subagent tokens (11.1M players, 6.1M reviews, 0.2M synthesis).
- Reviews are worth their cost: they found the mistakes behind most losses, corrected false bug reports by players, and produced 20 candidate rules (17 plus 3 that need Brady's ruling) in `lvl/SYNTHESIS.md`.

## Method and what changed during the night
- One subagent per seat per game, protocol through files (`lg.sh pick/show/odds/opp/sim/flag/auto`). Brief: `harness/BRIEF-vs.md`. Reviewer brief: `harness/BRIEF-review.md`. Players flag decisions they were unsure about; reviewers flag likely mistakes and close calls.
- Added to the brief as problems showed up: cost cheat-sheet (Show and Tell {2}{U}, Aluren {2}{G}{G}, Ancient Tomb = 2), deck-out check for the Lost Mine loop, Daze rules, planeswalker label note, own-tag-only warning, shared-/tmp warning, and the corrected macro rule order (`stack:Pass priority` first).
- **g21 is tainted**: seat B ran `show` with seat A's tag and saw A's hand. Excluded from win rates; its flags are still used.
- The macro (`lg.sh auto`) turned the Acererak/Lost Mine loop from 350-650 prompts and about 4 hours into a few relaunches. It cannot answer a Force of Will or Veil while it passes and has no deck-out guard.

## Results table (Alurentell side)
| Matchup | Won | Lost | Share |
|---|---|---|---|
| vs UR Cutter | 13 | 8 | 62% |
| vs Boros Aggro | 12 | 7 | 63% |
| Total | 25 | 15 | 62.5% |

Per-game results are in `lvl/games.json`, tokens in `lvl/tokens.tsv`. For context, the earlier net-search UR Cutter test had the LLM at 18-2 with a smaller set of tools and the old list; the lower rate here reflects a much stronger opponent (an LLM that plays Force of Will, Wasteland and Daze well and sees only its own view), the live list and engine, and that both sides are the same model.

## What decides games (from 40 reviews)
- **Almost every Alurentell win from g13 on is Aluren or Omniscience plus Acererak and the Lost Mine drain.** Win turns range from turn 1 (g35) to 15+; median about turn 8.
- **Losses**: Wasteland on Ancient Tomb, Tropical Island or Hedge Maze (g19, g20, g27, g33, g34, g40); one-land or Tomb-dependent keeps; Murktide and DRC clocks (UR); the Ocelot Pride, Ajani and second-Ajani legend-rule flip curve (Boros); and cost misreads by the player (g15, g17, g37). g11 is the only loss with the full combo in hand.
- Removal on Acererak's enter trigger is the only thing that broke the loop; each time a second Acererak or a Force beat it.

## Candidate playbook rules
**Update 2026-10-07 13:06Z: Brady reviewed the synthesis. His corrections are in `lvl/SYNTHESIS.md` section 0 and the signed-off rules are now in the playbook.** Main corrections: with Aluren in play, leave the Lost Mine room triggers on the stack except Dark Pool's drain (no draws, no library cost; the four-cast arithmetic below applies only under Omniscience, and the macro must be reworked); Veil timing, one-land keeps and waiting a turn are situational, not rules; Bauble should target yourself with a fetchland; Wasteland before they untap. The list below is the original, uncorrected draft.

Full list with games, confidence and evidence: `lvl/SYNTHESIS.md`. Highest value:
1. **Lost Mine lap is four Acererak casts, drains 1 and draws 1.** Count opponent life against library + 1 before starting and do not cast free cantrips mid-loop. This corrects the playbook line that says three casts per point (13 games, high confidence).
2. Macro order and budget for the loop (11 games, high).
3. Count mana before planning; the cost cheat-sheet (10 games, high).
4. Life is mana: count their instant reach before paying Tomb or fetch life (10 games, medium).
5. Take a spare Acererak from Atraxa on the combo turn; on the combo turn hand size never applies (8 games, high).
6. Daze is live whenever they control any Island-typed land, tapped or not (8 games, high).
7. With Aluren or Omniscience and Acererak available, loop now and run it to the kill (7 games, high).
8. Hold Force of Will for what breaks the loop (removal on the Acererak trigger), not for face burn or Swords on a Show-and-Tell Atraxa (7 games).
9. After a mulligan against Boros, bottom Veil of Summer (a blank), never the only Show and Tell payoff (7 games).

### Needs Brady's ruling (contradict or limit his own lines)
- **Hold Veil for the opponent's first counter** instead of casting it first (contradicts the jam line "land, Veil, then Show and Tell"). 4 games, medium.
- **One-land and no-cantrip keeps that won** (g9, g12, g23, g37) against his mulligan rules; two losses confirm his rule (g19, g34). 8 games, low confidence. No change recommended.
- **Wait a turn when Veil is the only spare** instead of jamming. Two reviews disagree. Low.

### Also in the synthesis
Notes for the UR Cutter and Boros bots (Wasteland targets, cast cantrips before the land drop, keep one blue pitch card per Force, Swords on Acererak, second Ajani as reach, hold Goblin Bombardment).

## Harness and engine issues found
Likely bugs and anything that touches hidden information first; full list in the synthesis section 4.
- **Possible hidden-information leak, unverified (g32)**: after Atraxa bottomed a revealed Acererak and Erode shuffled A's library, B's known-cards list still showed Acererak in A's hand. Known-card memory should clear on a shuffle. Worth a look by the engine thread.
- **Aluren and Show and Tell auto-payment over-taps** (g17, g23, g26, g37) and prefers Ancient Tomb over painless sources, costing 2 life (g32, g38). It could cost a game at low life; the payment should prefer painless sources or be manual.
- **Playbook line "at 4 life or less the engine will not offer a Tomb-only cast" is wrong**: g8 (2 life) and g19 (3 life) were offered them.
- Missing priority windows: none in declare-blockers or combat damage (g7, g12); none for the non-active player with Wasteland in upkeep (g9).
- Dungeon completion counted while Temple abilities are still on the stack (g17), no effect on the loop.
- Menu labelling: duplicate "Cast X" entries for hard and free casts, planeswalker "ability 1/2" with no text (decided g26), "#0" ids, collapsed duplicate names, bare "Do it?" prompts, no mana cost in "Cast X".
- Harness: the sim returns 0.0% at the mulligan prompt (g25, g34, g38), probably because `turn` stops before the first land drop; the loser sees no events after GAME OVER; the macro passes over opponent Forces.
- The deck list no longer has a basic Forest or a black land, so playbook lines 42 and 87 that mention Forest are out of date.

## Tough spots from the games
`/mnt/project-files/tough-spots/llmgames/s8/` (README inside). Existing page card format, batch tag s8, ids `s8-gNN-...`, order numbers from 67.
- `batch-priority.json`: **47 positions a reviewer marked likely-mistake**, orders 67-113. Start here.
- `batch.json`: all 268 flagged positions (155 player-unsure, 65 reviewer-close, 47 likely-mistake, 1 both). Seed one file, not both.
- Cards have `source: "llm-vs-llm"`; all bot fields are null; `llm_choice` is what the LLM played; the page needs "LLM played this" wording. No hidden information.

## Caveats
- n is small (40 games), same model on both sides, no game-1/2/3 sideboarding.
- Reviewers have hindsight and can be wrong; they also disagreed with each other on a few rules, which the synthesis marks rather than resolves.
- g41 and g43 were stopped without results (their players stalled when the session clock jumped), g42 has player reports only and no review.
- Cost numbers are subagent tokens only, not subscription usage.
