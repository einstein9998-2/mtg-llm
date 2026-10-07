# LvL overnight state (maintained by the thread Claude; read this after a context reset)
Goal (Brady, 2026-10-07 02:48Z): keep LvL (LLM vs LLM) games running until ~12:00 UTC (8am Brady), as quota allows; morning summary by then
(results, tokens per reviewed game, draft playbook rules, tough-spot flags). Stop if a usage limit is hit.
Rolling pool: keep 4 games in flight. When a game's two players finish (notifications), (1) record tokens in tokens.tsv, (2) run `python3 mkmeta.py results`,
(3) spawn the reviewer (opus, BRIEF-review.md, GAME=gN) and (4) start the next game with `./newgame.sh N OPP` + two sonnet player agents
(prompt: "Read /home/claude/work/llm2/BRIEF-vs.md completely and follow it exactly. Your DECK is <deck>, your opponent plays <deck>, your TAG is gNa|gNb. Do not call any mcp__hearthbot__ tools. Play the whole game, then write the report as the brief says.").
Schedule: games 1-4 = w1g1..w1g4 (done engine-side by launch earlier, UR Cutter). Games 5-8 = ur-cutter, 9-12 boros-aggro, 13+ alternate boros-aggro / ur-cutter (decide again then).
Files: games.json (metadata), tokens.tsv (tag, subagent_tokens), reports/, reviews/ (GAME.md + GAME.flags.json), flags/ (player flags), runs/GAME/truth/ (transcripts, actions.json).
Tough spots: after reviews, run harness/llmgame_positions.py <llm2> /mnt/project-files/tough-spots/llmgames/<batch> <tag> <first order> <games...> (batch file only; the page-seeding is another thread's job; orders after 66).
Reviewer tokens are logged as tag review-<game>. Harness source: /home/claude/mtg-llm/llm-benchmark/harness (PR #8 branch claude/project-thread-5q023p).
IN FLIGHT (updated 02:55Z): w1g1, w1g3 (wave 1 players running), g5, g6 (players running). Reviews running: w1g2. Reviewed: w1g4. Next game number: 7.
03:02Z in flight: g5,g6,g7,g8 players; reviews: w1g2,w1g3,w1g1 (w1g4 done). Next game number 9 = boros-aggro.
03:12Z in flight: g5,g6,g7 players, g9 (boros) starting; reviews: g8 starting. g8a tokens pending (agent a5c55ef8221795ca3). w1 reviews done. Next game 10 = boros-aggro.
03:14Z in flight: g5,g7 players; g9,g10 (boros) players; reviews: g8,g6. g6b tokens pending (agent abf9a1b6ca049c77c). Next game 11 = boros-aggro.
03:18Z in flight: g7,g9,g10,g11 players; reviews g6,g5. g5b tokens pending (a950d40594001b6a8). Note: g5b player ran ps aux (saw cmd lines only). Next game 12 = boros-aggro.
03:20Z harness change: own Main2 now always stops (was only with a creature) from game 12 on (new binary target-live-new). g1-g11 used the old filter. Reviews done: w1g1-4, g8, g6; running: g5.
03:20Z in flight: g9,g10,g11,g12 players; reviews g5,g7. g7b tokens pending (ad3f5620e10f43ac1). Next game 13.
03:25Z reviews done: w1g1-4,g5,g6,g7(tokens pending a3e16909dfc6b9a54),g8. Idea: Acererak loop shortcut tool (loop needs hundreds of picks; players stop looping).
03:35Z in flight: g9,g10,g12,g13 players; review g11. g11a tokens pending (a2f183e62f7779891). Next 14 = boros-aggro, then alternate.
07:02Z in flight: g10,g12,g13,g14 players; review g9 (g9a, g9b tokens pending: agents aec1a972cf27fcaa9=g9b, ab2f9e6b95b481f1c=g9a). Next 15 = ur-cutter.
07:02Z NOTE wall clock is 07:02Z (games take hours when Acererak loops run: g9 3.8h, 538k tokens for two players). In flight: g10,g12,g13,g14 players; review g9 running. g11 review done.
07:10Z auto macro added (lg.sh auto), BRIEF-vs updated; binary target-live-new (g15+ agents read the new brief). g9 review done. g10 both players done (A won, B lost; tokens g10a 266095, g10b 168542).
07:11Z in flight: g12,g13,g14,g15 players; review g10. Next 16 = boros-aggro.
07:11Z in flight: g13,g14,g15,g16 players; reviews g10,g12. Next 17 = ur-cutter. Harness flags to report: no end-step stop for non-active player when stack empty (g12a), Tomb+Aluren loop long games.
07:12Z in flight: g13,g15,g16,g17 players; reviews g10,g12,g14. g14a tokens pending (a2523217de068e121). Next 18 = boros-aggro.
07:13Z g14a logged (125429); g14 review running; g17 launched
07:17Z g10 review done (reviews/g10.md). Open check: whether A's g10 picks leaked to other game (g11? skip). Review token count pending (a220db2af958cb7ed).
07:18Z g14 review done; g10 review tokens logged 157197. g14 review tokens pending (ad117d5cc08da7321).
07:18Z g14 review tokens logged 165193. In flight: g13,g15,g16,g17 players; review g12 running.
07:18Z g15a WON turn 5: auto macro WORKED on real Lost Mine loop (59 prompts + 368 auto picks). g15a tokens pending (a123fddd45b4b7780); g15b pending.
07:19Z g15 done (A won T5, macro worked). g15a logged; g15b tokens pending. g18 (boros) launched; review g15 spawned. Next 19 = ur-cutter.
07:19Z g15b logged 80267. In flight: g13,g16,g17,g18 players; reviews g12,g15.
07:19Z g12 review done (reviews/g12.md); token count pending (a23b30f1558138d0a). Candidate engine bug: no priority window after blockers declared (g12). 
07:19Z g12 review tokens logged. g17a WON T7 (macro again; rule order: put 'stack:Pass priority' first). g17a tokens + g17b pending.
07:19Z g17 done (A won T7). g17a logged; g17b tokens pending. g19 (ur-cutter) launched; review g17 spawned. Next 20 = boros-aggro.
07:19Z g17b logged.
07:20Z g16b WON (Boros, T6). g16b tokens + g16a hand-back pending; then review g16 and launch g20 (boros).
07:20Z g16 done (A lost T6). g16b logged; g16a tokens pending. g20 (boros) launched; review g16 spawned. Next 21 = ur-cutter.
07:20Z g16a logged. In flight: g13,g18,g19,g20 players; reviews g15,g17,g16. Game g13 players still running (long).
07:24Z g15 review done+logged (142507). Brief updated with cost cheat-sheet (S&T {2}{U}, Aluren {2}{G}{G}, 4-cast lap). TODO at end: g15a flags 1,12 notes have wrong premise (Show and Tell cost) - fix in flags/g15a.tsv before conversion; macro risk: stack:Pass priority passes over opp Force (brief warns).
07:24Z g20 done (A lost T10, Boros won T11); tokens logged. g15a flags corrected. g21 (ur-cutter) launched; review g20 spawned. Next 22 = boros.
07:24Z g17 review done+logged. Engine note for Brady: dungeon completion counted while last-room abilities still on stack (g17 prompt 55); Aluren payment tapped extra source leaving floating mana. g17a flags have a wrong-premise (Aluren cost) - fix before conversion.
07:24Z g18a WON (T6, macro). g18a tokens + g18b pending.
07:25Z g18a logged (113524); g16 review done, tokens pending (a5c905c4ff90c9a0b). g18b pending.
07:27Z g16 review logged. g19b WON (UR beat Alurentell T12). g19a + g19b tokens pending. In flight: g13 (long), g18b, g19a, g21.
07:27Z g19 done (A lost T12). g19b logged; g19a tokens pending. g22 (boros) launched; review g19 spawned. Next 23 = ur-cutter.
07:27Z g19a logged. In flight: g13 (long), g18b, g21, g22 players; reviews g18? (not yet; g18b pending), g19, g20.
07:28Z g20 review done (reviews/g20.md); tokens pending (a390c2dc49a640561). Engine check item: Lazotep Quarry / Voice payment (Quarry colorless tap not offered?).
07:31Z g20 review logged; g19 review done (reviews/g19.md), tokens pending (a8180dfc5daf2c277). Engine check: Tomb payment offered at 3 life for Force hardcast (g19 prompt 82).
07:32Z g19 review logged. g13b LOST (UR; A won g13 via ~440-prompt loop before macro). g13a/g13b tokens pending, g13 needs review, then launch g23 (ur-cutter).
07:32Z g13b logged (225123); g13a WON (hand-back received; tokens pending). Shared /tmp helper collision explained g10a cross-game claim; warning added to brief. Next: review g13, launch g23.
07:32Z g13a logged. g13 review + g23 launched. In flight: g18b, g21, g22, g23 players; reviews g13 (+ g18, g21, g22 when done). Next game 24 = boros-aggro.
07:33Z g21a LOST (UR won g21, T11). g21a tokens + g21b hand-back pending.
07:33Z g21 done: A lost T11, TAINTED (B saw A's opening hand: stray show). tainted.txt written. g21a logged; g21b tokens pending. g24 (boros) launched; g21 review spawned (A side only). Next 25 = ur-cutter.
07:33Z g21b logged. In flight: g18b, g22, g23, g24 players; reviews g13, g21.
07:35Z g18 done (A won T6, Boros lost). g18b tokens pending. g25 (ur-cutter) launched, review g18 spawned. Next 26 = boros.
07:35Z g18b logged (76048, may be interim). In flight: g22,g23,g24,g25 players; reviews g13,g21,g18 running; g19/g20 done.
07:35Z g23a WON T5 (macro). tokens pending; g23b pending.
07:35Z g23 done (A won T5). g23a logged; g23b pending. g26 (boros) launched; review g23 spawned. Next 27 = ur-cutter.
07:35Z g23b logged. In flight players: g22,g24,g25,g26; reviews g13,g18,g21,g23.
07:36Z g13 review done (reviews/g13.md); tokens pending (a755574be04254548). Engine/menu issue: duplicate 'Cast X' hard vs free Aluren labels (stale index risk).
07:38Z g13 review logged; g21 review done (tainted game), tokens pending (acf42fbc5c85440ae). Harness TODO idea: block 'show' of other seat tag (g21b saw A's hand).
07:38Z g21 review logged. Brief: own-tag-only warning.
07:39Z g18 review done (reviews/g18.md); tokens pending (ad05d2b9da2f2c4fc). Macro correct again.
07:39Z g18 review logged; g23 review done, tokens pending (ad3f8ac62acc89fc2). Engine note: Aluren payment taps extra source (g17, g23).
07:39Z g23 review logged. g22a WON T10 (Boros side lost? g22b pending): loop drained 36 life, library hit exactly 0 (decking risk: each Temple lap draws a card) - candidate rule.
07:39Z g22 done (A won T10; Boros lost). g22a logged; g22b tokens pending. g27 (ur-cutter) launched; review g22 spawned. Next 28 = boros.
07:39Z g22b logged. In flight players: g24,g25,g26,g27; reviews g22 (g24-27 when done).
07:41Z g24b LOST (A won g24 T8). g24a/g24b tokens pending.
07:41Z g24 done (A won T8). g24b logged; g24a tokens pending. g28 (boros) launched; review g24 spawned. Next 29 = ur-cutter.
07:41Z g24a logged. In flight players: g25,g26,g27,g28; reviews g22,g24 running; g23 review done+logged.
07:47Z g28a WON T4 (macro). Brief example rule order fixed (stack:Pass first). g28a tokens pending; g28b pending.
07:47Z g27b WON (UR beat A T10) -> g27 done. g28a logged. g29 (ur-cutter) launched; review g27 spawned (g27a tokens pending). Next 30 = boros.
07:47Z g27, g28 done (g28: A won T4, Boros T2-Aluren loss). tokens logged g27a/b g28b g25b. g25a hand-back pending (g25b says its game ended abruptly at 11 life vs 30 -> likely A loop). g30 (boros) launched; reviews g28 spawned; g25 review after g25a. Next 31 = ur-cutter.
07:47Z g25 done (A won T13). g22 review + g25a logged. g31 (ur-cutter) launched; g25 review spawned. Next 32 = boros. Brief: decking check added.
07:48Z g24 review logged. In flight players: g26 (a/b hand-back pending), g29, g30, g31; reviews g25, g28 running; g27 review running.
07:50Z g26b LOST (A won g26 T8 combo). Menu issue: planeswalker abilities 'ability 0/1/2' labels vs indices (g26b picked +2 not 0). g26a/g26b tokens pending.
07:50Z g26 done (A won T8). g26b logged; g26a tokens pending. g32 (boros) launched; review g26 spawned. Next 33 = ur-cutter.
07:50Z g26a, g28 review logged. In flight players: g29,g30,g31,g32; reviews g25,g26,g27 running.
07:53Z g27 review done, tokens pending (a361f2e95d7ffc0db). Brief: Daze Island-typed rule added.
07:54Z g27 review logged; g25 review done, tokens pending (acad719019ba71370). Sim tool oddity: 'Ponder sim at mulligan returned 0%' (g25b) - check if time.
07:56Z g25 review logged; g26 review done, tokens pending (a469bbe2522957b6c). Harness issue list: pw ability labels (g11,g26), Aluren overpays (taps extra source/Tomb), duplicate unlabeled menu entries, Quarry 25-35 entries, loser sees no final events, log summary prints other player's tool counts.
07:59Z g26 review logged. g30b LOST (A won g30). g30a/g30b tokens pending.
07:59Z g30 done (A won T8 via S&T for Aluren). g30b logged; g30a pending. g33 (ur-cutter) launched; review g30 spawned. Next 34 = boros.
07:59Z g30a logged. In flight players: g29,g31,g32,g33; reviews g30 running. Pending: g29 (a/b), g31, g32.
07:59Z g32a WON T6 (macro). g32 b + tokens pending.
07:59Z g32 done (A won T6). g32a logged; g32b pending. g34 (boros) launched; review g32 spawned. Next 35 = ur-cutter.
08:00Z g32b logged. In flight players: g29,g31,g33,g34; reviews g30,g32 running.
08:02Z g31b WON (UR beat A T10 -> g31 done). g31a/g31b tokens pending; launch g35 and review g31 once logged.
08:03Z g31 tokens logged; g35 launched, review g31 running. In flight players: g29,g33,g34,g35. Next 36 = boros.
08:04Z g18b final tokens 77088.
08:05Z g32 review done (reviews/g32.md), tokens pending (a05d06b6f02091e91). ENGINE ITEM: possible hidden-info leak: B's known-cards list showed Acererak in A's hand after Atraxa-revealed card went to library bottom, then Erode shuffle, then A drew it (g32 B prompt 49). Needs engine check (known-card knowledge should be cleared on shuffle).
08:05Z g32 review logged; g30 review done, tokens pending (add0fd8a15812574b). Recurring reviewer rule (g30,g32): with Acererak in hand/known, Show and Tell puts in ALUREN not Atraxa. Harness limit: no mana abilities offered (can't float City mana before land drop).
08:05Z g30 review logged. g35b LOST (A won g35 T1 kill: Tomb+Petal+S&T+Aluren). g35a pending. g36 (boros) launched; review g35 spawned. Next 37 = ur-cutter.
08:05Z g35 tokens logged. In flight players: g29,g33,g34,g36; reviews g31,g35 running. Pending: g29 (a,b).
08:07Z g34b WON (Boros beat A, T11). g34a/g34b tokens pending.
08:07Z g34 done (Boros won, A lost T11). g34b logged; g34a pending. g37 (ur-cutter) launched; review g34 spawned. Next 38 = boros. Tool bug note: sim at mulligan prompt returns 0.0% (g25b, g34a) - harness tool issue.
08:07Z g34a, g35 review logged. In flight players: g29,g33,g36,g37; reviews g31,g34 running.
08:08Z g31 review done (reviews/g31.md), tokens pending (a5dc4b8455bd3fd2a).
08:09Z g31 review logged. g29a WON (A beat UR T9); g29a tokens + g29b pending. Game 29 review after both.
08:09Z g29 done (A won T9). g29a logged; g29b pending. g38 (boros) launched; review g29 spawned. Next 39 = ur-cutter.
08:10Z g29b logged. In flight players: g33,g36,g37,g38; reviews g29,g34 running; g33/g36/g37/g38 pending.
08:11Z g33b WON (UR beat A T10 -> g33 done). g33a/g33b tokens pending.
08:11Z g33 done (UR won T10). g33b logged; g33a tokens pending. g39 (ur-cutter) launched; review g33 spawned. Next 40 = boros.
08:12Z g33a logged. In flight players: g36,g37,g38,g39; reviews g29,g33,g34 running.
08:13Z g34 review done (reviews/g34.md), tokens pending (ac111b0acefed05d1). Sim at mulligan prompt 0.0%: unclear if bug; reviewer suggests make sim print the turn/land-drops simulated and warn at mulligan prompts.
08:14Z g34 review logged. g36a WON T6 (macro). g36b + tokens pending.
08:14Z g36 done (A won T6; Boros lost). g36a logged; g36b tokens pending. g40 (boros) launched; review g36 spawned. Next 41 = ur-cutter.
08:15Z g36b logged. In flight players: g37,g38,g39,g40; reviews g29,g33,g36 running.
08:16Z g29 review done (reviews/g29.md), tokens pending (a10a3cbeb3281f0b8).
08:17Z g29 review logged; g33 review done, tokens pending (a043eb29f51c83fd6).
08:19Z g33 review logged; g36 review done, tokens pending (abfbd3468309df488). In flight players: g37,g38,g39,g40.
08:20Z g36 review logged. g40b WON (Boros beat A T7); g40a/g40b tokens pending.
08:20Z g40 done (Boros won T7). g40b logged; g40a tokens pending. g41 (ur-cutter) launched; review g40 spawned. Next 42 = boros.
08:20Z g40a logged. In flight players: g37,g38,g39,g41; reviews g40 running.
08:21Z g37a WON T7 (macro). g37b + tokens pending.
08:21Z g37a logged. g39b WON (UR beat A T10 -> g39 done). g37b, g39a tokens/hand-backs pending.
08:21Z g39 done (UR won T10). g39b logged; g39a tokens pending. g42 (boros) launched; review g39 spawned. Next 43 = ur-cutter.
08:21Z g37 done (A won T7), g39a/g37b logged. g43 (ur-cutter) launched; review g37 spawned. Next 44 = boros.
12:00Z g39 review done (reviews/g39.md), tokens pending (aac9264cad423611b). In flight players: g38,g41,g42,g43; reviews g37,g40.
12:00Z g39 review logged. g38b LOST (A won g38, Boros lost T14). g38a/g38b tokens pending.
12:01Z CLOCK NOTE: sandbox clock jumped 08:22->12:00 (container paused?). It is now 12:00Z = STOP time. No more new games. g41,g42,g43 players were launched but show no progress files since 08:22; g38 review spawned. g38a/b tokens corrected (g38a 160260, g38b 115233). NEXT: compile morning summary (results, tokens, rules, flags), build tough-spot batch (s8, from order 67), push to PR #8, single reply to Brady.
12:10Z: g41,g43 stalled since 08:20 (agents stopped, no result). g42 lost for A. g40 review logged. Waiting g38 review + SYNTHESIS, then converter + RESULTS-lvl.md + push PR8 + single reply.
