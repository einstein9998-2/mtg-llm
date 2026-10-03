# No-tools LLM player: Alurentell (me) vs UR Cutter (Forge AI)

Date: 2026-10-01. Player: Claude in this thread, subscription only, no API, no tools beyond the serializer prompt. Opponent: Forge's built-in AI piloting UR Cutter (the Forge AI cannot pilot the Aluren/Show and Tell combo, so I pilot Alurentell). Both lists are the game 1 main 60s from `/mnt/project-files/decks/`. Engine: Forge plus the Phase 1 serializer (External policy), `default` stop preset.

The earlier five games of Death and Taxes vs Jund were a smoke test on a deck set Brady rejected. They do not count; they are kept in [report-jund-smoke-test.md](/mnt/project-files/llm-player/report-jund-smoke-test.md).

## Result: 3 wins, 3 losses (6 games, not enough to say anything about strength)

**Update:** the counted batch with the final serializer build (a07 to a21, 15 games) is in [batch-results.md](/mnt/project-files/llm-player/batch-results.md): 17 wins, 3 losses after a26 (see that file). This file keeps the first six games.

| Game | Seed | Seat | Result | Turns | Prompts | Notes |
|---|---|---|---|---|---|---|
| a01 | 201 | draw | loss | 11 | 30 | Mulligan to 6. Opp curved Bolt, Murktide, DRC; my second Stock Up was Force of Will'd. Ran **before** the serializer fixes below: the first Stock Up picked its cards without me. |
| a02 | 203 | draw | win | 16 | 111 | Mulligan to 6. Veil of Summer then Show and Tell for Atraxa on my second turn; the Forge AI answered with Murktide Regent. Ran with the Stock Up and Ponder fixes only. |
| a03 | 204 | draw | loss | 11 | 39 | Mulligan to 6 (no-land 7). Two Dragon's Rage Channelers with delirium flew over, then Murktide; I never found Atraxa in 15 looks. Ran with all fixes. |
| a04 | 205 | draw | **win** | 18 | 169 | Kept 7. Stock Up was countered by Force of Will + Daze, DRCs and Bolts took me to 7. Omniscience was missing from the menu for a turn (engine issue 9 in the oddities file), then Show and Tell put Omniscience in; free Atraxa blocked and gained life; about 40 free Acererak casts through Aluren (Lost Mine, Mad Mage, then Tomb of Annihilation chosen by the AI) drained the opponent to 2 and Atraxa plus Goblins finished. |
| a05 | 206 | play | loss | 22 | 36 | Kept 7 with two lands, Petal and top-end cards. Two Wastelands took Tropical Island and Ancient Tomb, Murktide came down twice (I countered the first with Force of Will), I stalled on 3 lands, and Daze countered my Aluren when I tapped out. |

| a06 | 207 | play | **win** | 15 | 102 | Mulliganed a one-land seven, kept six. Two Stock Ups, Force of Will on Murktide, then a slip (below) handed the opponent a free Murktide and took me to 3 life. Ponder found Atraxa; Show and Tell put it in (Atraxa gained 7 on each attack), Veil of Summer fizzled a Prismari Charm aimed at Atraxa (and drew a card), then a second Veil made Aluren uncounterable, then looped Acererak with Aluren (Lost Mine three times, Goblin Lair, Dark Pool) for 4 drains to finish from 4 life. First game with dungeon, room and cost-choice prompts. |

An extra game on seed 202 was aborted at turn 3 when I found Stock Up was auto-picking; it is not counted (`runs/aborted-a02-seed202`).

- Win rate 3/6. Wilson 95% interval is roughly 19% to 81%, so this says nothing about the matchup. Four of six games were played on the draw; three mulligans were forced by no-land or one-land hands, so the sample is mostly variance.
- Prompt volume: 30 to 169 prompts per game (about 5k to 19k prompt tokens, chars/3.6). The long game was a02, where I looped Acererak about 20 times through Aluren. The turn-2 Atraxa win cost 111 prompts because of the loop and Atraxa's ten-card reveal, not because the win needed it.
- Wall clock per game was 4 to 9 minutes of engine time. The real cost is tool calls: roughly 60 to 150 sequential calls per game.

## Blunders (judged after each game against the engine's ground truth)

Method as before: `review.py` prints each decision next to the opponent's real hand, my real library top and the Forge AI's pick; I judged by hindsight with full information, no engine search.

| Game | Costly blunders | No-cost slips and process errors |
|---|---|---|
| a01 | **1** (moderate) | none noted |
| a02 | 0 | Chained two picks and declined an Atraxa attack on my turn 14 (cost one turn, still won). |
| a03 | 0 | Brainstorm put-backs (a Show and Tell and Acererak) made my next draw a known dud. Hindsight: Atraxa was not among the top four cards anyway, so no realised cost. |
| a04 | **1** (moderate) | Chained two plain passes in one command (harmless). |
| a06 | **1** (moderate) | Re-read of the menu skipped once (see below); otherwise none noted. |
| a05 | 0 | Keep of a two-land hand with no cantrip is defensible but poor on the play against Wasteland and Daze. Tapping out for Aluren into a known Daze was forced (no fifth mana). |

- a01 blunder: on my third turn I cast Stock Up #2 with Force of Will in the opponent's hand (their hand was Heat, Force of Will, Tarn, Brainstorm) while Veil of Summer was in my own hand. Veil ({G}) would have made Stock Up uncounterable; I spent the green source on Ponder instead and the Stock Up was countered. I had decided earlier that Veil was for protecting my key spell and then did not use it on the key spell. Cost: one card and tempo in a game I lost to Murktide tempo; I judge it moderate, not decisive.
- a04 blunder: with my free Omniscience I countered Cori-Steel Cutter with a free Force of Will. The artifact in their graveyard was their fourth card type, so all three Dragon's Rage Channelers became 3/3 flyers that turn (I had not counted graveyard types). I still won, but the DRCs now blocked my Atraxa and killed me nearly a turn faster than the Cutter would have. Moderate.
- a06 blunder: on turn 9 the menu had shifted by one after a land drop and I picked the old index, so I cast Show and Tell instead of Lotus Petal. I put Acererak in; the Forge AI answered with a free Murktide Regent, which took me from 14 to 3 life over the next turns. Process error: I reused a stale menu number. Since then I re-read the latest menu before every pick. Moderate; I still won.
- Costly blunders: 3 in 6 games (3 in about 490 prompts). No-cost slips: 3.
- The results were decided by mulligans, Daze/Force of Will exchanges and the Forge AI's curve (Murktide on turn 3 in a01 and a03). The one blunder above did not decide a01 by itself.

## Engine and serializer problems (details in [engine-oddities.md](/mnt/project-files/llm-player/engine-oddities.md))

The matchup exposed several decisions the serializer hands to the Forge AI instead of the policy, which matters a lot for a deck built on selection:

1. **Stock Up and Ponder choices were delegated** (which two cards, and the top-three order), so my first Stock Up in a01 was an arbitrary pair. Fixed in a patch.
2. **Scry and surveil** were also delegated; fixed.
3. **Dungeon and room choices** were made by the Forge AI (a04 even entered Tomb of Annihilation, which ends the Acererak loop). **Fixed**: dungeon, room and cost choices (discard, exile, sacrifice, Force of Will pitch) are now prompts. Used live in a06 (about 14 dungeon/room prompts, a pitch prompt). Details in the oddities file (items 5, 10, 11, 13).
4. **Mana payment** is still delegated, and the legal-action menu is built from the Forge AI's payability check. The "Omniscience missing from the menu" problem I reported for a04 was **my miscount**: Omniscience costs ten mana and I had nine. The one real case left is Ancient Tomb at very low life (the check refuses lethal damage, which is correct). A payment prompt would still help source choice (keep Tombs and Petals for later), but it is low priority now.
5. **Atraxa's ten-card reveal showed "a hidden card"** in a02, a04 and a06 (two earlier fixes failed). Cause found by replaying a06 with my recorded picks: Forge uses `chooseCardsForEffect` for it. **Fixed**, verified by replay and played live in a07 to a21 (names shown).

The patch is `choice-visibility.patch` (a diff against `serializer/src/silo/SeatController.java`; the original serializer is untouched, nothing was pushed). It builds and was used in a02 to a06. The non-interference test (shuffle hidden zones, require identical prompts) and the mirror-vs-stock equivalence test are being rerun on the final build (results in the thread). Nothing is pushed or merged.

## Recommendations

1. Merge the patch only after the non-interference test passes on the final build (Brady decides; nothing is merged without asking). Dungeon, room and cost prompts are in; a mana-payment prompt is optional.
2. Play 20+ games per side before comparing against the Forge AI, and put the Forge AI on the Alurentell side as a (very weak) baseline only if its combo logic is added.
3. Keep playing the combo side from the draw and the play: all three games were on the draw (seeds were drawn that way), which is a bias.

## Files

`play.py` (broker, now takes deck arguments: `start <tag> <seed> default alurentell.dck ur-cutter.dck`), `review.py`, `shadow-hint.patch`, `choice-visibility.patch`, `runs/a01 ... a05` (`decisions.jsonl` has the state, menu, my pick, Forge AI hint and ground truth; `player.jsonl` has my one-line reasoning for every prompt; `review.txt` is the post-game audit), `engine-oddities.md`. The deck conversions are `forge-runner/decks/alurentell.dck` and `ur-cutter.dck`.
