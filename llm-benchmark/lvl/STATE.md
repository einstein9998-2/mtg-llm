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
13:25Z: Brady asked LvL until 5pm (21:00Z). Macro reworked (top[]/after[]/mytop/empty conditions; Brady's technique: leave triggers stacked except Dark Pool), brief+review brief updated, playbook refreshed. Launched g44-g47 (g44 boros, g45 ur, g46 boros, g47 ur). Rolling pool of 4 until 21:00Z. Next 48 = boros.
13:27Z player agents: g44a a84283d5240669f1d, g44b af7e66d32bde07409, g45a a7af544f4beb78b6e, g45b a83f1f354408a9f06, g46a ad0cfaefdeef6a5bb, g46b adfe062ef572c9a5f, g47a a41b6b2d65971a505, g47b a8a05999f32c691a0. Procedure per finished game: log tokens (subagent_tokens) in tokens.tsv, python3 mkmeta.py results, newgame.sh N opp + 2 Agent spawns (prompts as above), review agent opus (prompt: BRIEF-review.md, GAME, A=Alurentell, B=opp, reports, flags). Stop launching at 21:00Z (earlier if usage limit).
13:24Z g47 done (A won, macro with new technique). g47a logged; g47b tokens pending. next 48 boros + review g47.
13:24Z g47b logged; g48 (boros) launched; review g47 spawning. Next 49 = ur-cutter.
13:24Z g44 done (A won T6, Omniscience loop, B kept no-white 7). logged. g49 (ur) launched; review g44 spawning. Next 50 = boros. Reviews running: g47 (ad502638e7181859a).
13:29Z rev-g44 logged. g45b WON (UR beat A T10), g45a tokens pending.
13:29Z g45 done (UR won T10, A lost). g45b logged, g45a tokens pending. g50 (boros) launched; review g45 spawning. Next 51 = ur-cutter.
13:29Z g45a logged. In flight players g46,g49,g48,g50; reviews g47 (ad502638e7181859a), g45 (ae6e255f8e704843b).
13:29Z g47 review done (macro verified: no library loss, no legend problems; budget raised to 200 in brief). rev-g47 tokens pending.
13:31Z rev-g47 logged. g48a WON T6 (macro); g48b pending.
13:31Z g48 done (A won T6 macro). g48a logged, g48b pending. g51 (ur) launched; review g48 spawning. Next 52 = boros.
13:31Z g48b logged. In flight: g46,g49,g50,g51 players; reviews g45 (ae6e255f8e704843b), g48 (ad1ea7067dded8fa6).
13:33Z g46 done (A won T10 macro, Karakas bounced Atraxa). g46a logged, g46b pending. g52 (boros) launched; review g46 spawning. ENGINE ITEM: Boseiju channel lets the opponent fetch nonbasic lands (Volcanic Island g45b, Plateau/Savannah g46b); real card says basic only. Next 53 = ur-cutter.
13:33Z g46b logged. In flight: g49,g50,g51,g52 players; reviews g45 (ae6e255f8e704843b), g48 (ad1ea7067dded8fa6), g46 (a6b7c74d74954931b).
13:33Z g50 done (A won T4, one-lander keep + Show and Tell for Aluren). g50a logged, g50b pending. g53 (ur) launched; review g50 spawning. Next 54 = boros.
13:33Z g50b logged. In flight players: g49,g51,g52,g53; reviews g45 ae6e255f8e704843b, g48 ad1ea7067dded8fa6, g46 a6b7c74d74954931b, g50 aee756f383d023d9d.
13:35Z rev-g45 logged; g48 review done, tokens pending. ENGINE ITEMS (unverified): Ocelot Pride end-step with 10 permanents made no copies of Cat/Warrior tokens though City's blessing (g48 B prompt 32). Question for Brady: Omniscience-only loop vs dig for Aluren vs Boros (symmetry).
13:36Z rev-g48 logged. g49a WON T9 (macro), g49b pending.
13:36Z g49 done (A won T9). g49a logged, g49b pending. g54 (boros) launched; review g49 spawning. Next 55 = ur-cutter.
13:36Z g49b logged. In flight players: g51,g52,g53,g54; reviews g46 a6b7c74d74954931b, g50 aee756f383d023d9d, g49 a6b8724be630f76da.
13:37Z rev-g50 logged. g51b LOST (A won g51 T7), g51a tokens pending.
13:38Z g51 done (A won T7). g51b logged, g51a pending. g55 (ur) launched; review g51 spawning. Next 56 = boros.
13:38Z g51a logged. In flight players: g52,g53,g54,g55; reviews g46 a6b7c74d74954931b, g49 a6b8724be630f76da, g51 afec2bfd85610068f.
13:41Z rev-g51 logged; g46 review done (tokens pending). CORRECTION: Boseiju channel offering Plateau/Savannah/Volcanic is CORRECT (card says land with a basic land type), not an engine bug; remove from issue list. Harness gap (g46): window with only mana abilities (Quarry sac) is auto-passed on opponent's main 2.
13:41Z rev-g46 logged; g49 review done (tokens pending). Recurring engine item: auto-payment taps Ancient Tomb first (g32,g38,g49 etc).
13:42Z g52 done (Boros won T8, A lost). logged. g56 (boros) launched; review g52 spawning; rev-g49 logged. Next 57 = ur-cutter.
13:43Z g54b WON (Boros beat A T9). g54a pending. In flight: g53,g54,g55,g56 players; reviews g52 (ada2691cfe6939c16), g49/g46 tokens: rev-g49 logged, rev-g46 logged.
13:43Z g54 done (Boros won T9, A lost). g54b logged, g54a pending. g57 (ur) launched; review g54 spawning. Next 58 = boros.
13:43Z g54a logged. In flight players g53,g55,g56,g57; reviews g52 ada2691cfe6939c16, g54 adbd8cfe3a6f9d61a. rev-g49 logged.
13:45Z g55 done (UR won T12, A lost). g55b logged, g55a pending. g58 (boros) launched; review g55 spawning. Next 59 = ur-cutter.
13:45Z g55a logged. In flight players g53,g56,g57,g58; reviews g52 ada2691cfe6939c16, g54 adbd8cfe3a6f9d61a, g55 a9f6810f51daf6d97.
13:47Z rev-g54 logged; g52 review done, tokens pending.
13:47Z g56 done (Boros won T7, A lost). g56b logged, g56a tokens pending. g59 (ur) launched; review g56 spawning. rev-g52 logged. Next 60 = boros.
13:47Z g56a logged. In flight players g53,g57,g58,g59; reviews g55 a9f6810f51daf6d97, g56 a94c9b4f9c3b710b4.
13:48Z g53 done (UR won T8, A lost). g53b logged, g53a pending. g60 (boros) launched; review g53 spawning. Next 61 = ur-cutter.
13:48Z g53a logged. In flight players g57,g58,g59,g60; reviews g55 a9f6810f51daf6d97, g56 a94c9b4f9c3b710b4, g53 a62a0462bcfbda57d.
13:51Z rev-g55 logged; g56 review done, tokens pending.
13:52Z g57 done (UR won T12, A lost). tokens pending. g61 (ur) launched; review g57 spawning. rev-g56 logged. Next 62 = boros.
13:52Z g57 tokens logged. g61 engine running (agents spawning).
13:52Z In flight players g58,g59,g60,g61; reviews g53 a62a0462bcfbda57d, g57 a1b2fce1f091bf467; token pending: rev-g52 logged? yes; rev-g53/rev-g57 pending; g59/g58/g60 reviews not yet.
13:53Z rev-g53 logged. g58a WON T10 (macro); g58b pending.
13:53Z g58 done (A won T10). g58a logged, g58b pending. g62 (boros) launched; review g58 spawning. Next 63 = ur-cutter.
13:53Z g58b logged. In flight players g59,g60,g61,g62; reviews g56 logged? rev-g56 yes; pending rev-g57 a1b2fce1f091bf467, rev-g58 ad0bc5041ccf1c3d0, g59/g60/g61 not yet.
13:58Z g59 (UR won T14, A lost), g60 (A won T8, B mis-ran macro past end of turn), g61 (A won T5, UR lost) done; tokens logged; rev-g57, rev-g58 logged. g63 ur, g64 boros, g65 ur launched; reviews g59,g60,g61 spawning. Next 66 = boros. In flight: g62 players + g63,g64,g65.
13:59Z agents: g63a a054ad8f496e507a3, g63b a9d4b1db01a40ab77, g64a a9cb4036069975c61, g64b af2c2ca7c0bce126f, g65a a7fae7cbd028a0c94, g65b a1a9589c4ae3d7909; reviews g59 a07745ec72631fc16, g60 a6bad0a59da42bbdb, g61 a4c81542f7dd58761. g62 players adfd7b36d34f3211a / a490076cbe04c1b5d still running. NOTE g60b: macro ran 918 picks past N=15 at end step (check review).
14:01Z g62 done (A won T6, Boros lost to Aluren after Erode got Forced). g62a logged, g62b tokens pending. g66 (boros) launched; review g62 spawning. Next 67 = ur-cutter.
14:01Z g62b logged. In flight players g63,g64,g65,g66 (g66a a726fd9b4d93173ee, g66b a2912aac10a7c6ce7); reviews g59 a07745ec72631fc16, g60 a6bad0a59da42bbdb, g61 a4c81542f7dd58761, g62 a331eeca9d21287ed. Next 67 = ur-cutter.
14:03Z rev-g61 logged; g59 review done (tokens pending), g61 review done.
14:04Z rev-g59 logged; g60 review done (tokens pending; note auto_total counter is game-wide, misled B: harness issue; engine: auto-pay Plains+Parlor over Quarry in g60b).
14:05Z rev-g60 logged. g64a WON T8 (macro); g64b pending.
14:05Z g64 done (A won game turn 8, macro). g64a logged, g64b pending. g67 (ur) launched; review g64 spawning. Next 68 = boros.
14:05Z g64b logged. In flight players g63,g65,g66,g67 (g67a ab3ea492a0c40bc91, g67b af01de6c3b0fc826f); reviews g62 a331eeca9d21287ed, g64 a7ac874784371a346; tokens pending: none besides those. Next 68 = boros.
14:06Z rev-g62 logged. g66b WON (Boros beat A T11), g66a pending.
14:06Z g66 done (Boros won T11, A lost). g66b logged? see tokens. g68 (boros) launched; review g66 spawning. Next 69 = ur-cutter.
14:06Z g63 done (A won game T9, hardcast Aluren). tokens logged. g69 (ur) launched; review g63 spawning. g68 players g68a a998546a368fc54e0 g68b a8811b3edd676f27b; review g66 ab9b3cfcf232aa51c; review g64 a7ac874784371a346. Next 70 = boros.
14:07Z g69a a885cb3f9c5fdb37a, g69b a1f9da1676bef78c8, rev-g63 a391a6df80785d2bb. In flight players g65,g67,g68,g69; reviews g64,g66,g63.
14:08Z g65 done (UR won, A lost; A suspects auto-pay tapped Ancient Tomb at 2 life and died: ENGINE ITEM check in review). g65a logged, g65b tokens pending. g70 (boros) launched; review g65 spawning. Next 71 = ur-cutter.
14:08Z g65b logged. g70a a2904223d06e99b33, g70b a58c48a64e1dd8d0e, rev-g65 ab7c9e4ae97f81507. In flight players g67,g68,g69,g70; reviews g64 a7ac874784371a346, g66 ab9b3cfcf232aa51c, g63 a391a6df80785d2bb, g65. Next 71 = ur-cutter.
14:11Z rev-g64 logged; g66 review done (tokens pending).
14:11Z rev-g66 logged; g63 review done (tokens pending).
14:12Z rev-g63 logged. g67a WON (T7 macro); g67b pending.
14:12Z g67 done (A won game T7, Boseiju open for Veil backup, macro). g67a logged, g67b pending. g71 (ur) launched; review g67 spawning. Next 72 = boros.
14:12Z g67b logged. g71a ae334884b36c72437, g71b a080c1ef5cd029c6e, rev-g67 ab5090d2392d58cde. In flight players g68,g69,g70,g71; reviews g65 ab7c9e4ae97f81507, g67. Next 72 = boros.
14:13Z g68 done (A won game T6, Omniscience-only loop, Aluren never cast). g68a logged, g68b pending. g72 (boros) launched; review g68 spawning. Next 73 = ur-cutter.
14:13Z g68b logged. g72a a83e79f2326111bb8, g72b aa74ad6d76f728296, rev-g68 a6040ef4d85004337. In flight players g69,g70,g71,g72; reviews g65 ab7c9e4ae97f81507, g67 ab5090d2392d58cde, g68. Next 73 = ur-cutter.
14:15Z g65 review done (tokens pending). KEY ENGINE ITEM (g65): auto-pay tapped Ancient Tomb for Show and Tell at 2 life although 3 Tomb-free payments existed; A died to the 2 damage (menu had no payment choice). Same at prompts 62-63 (Stock Up paid Island+Tomb over Island+Maze+Tropical). Recurring (g32,g38,g49). Report to Brady as the top engine fix.
14:15Z rev-g65 logged. g70 A lost (Boros won T11); g70a tokens pending (agent a2904223d06e99b33 notification pending), g70b pending.
14:15Z g70 done (Boros won T11, A lost). g70a logged, g70b pending. g73 (ur) launched; review g70 spawning. Next 74 = boros.
14:15Z g70b logged. g73a a9d67fd760c43c42b, g73b a12fb7077f27350b4, rev-g70 accbd6f6c2f49df73. In flight players g69,g71,g72,g73; reviews g67 ab5090d2392d58cde, g68 a6040ef4d85004337, g70. Next 74 = boros.
14:17Z rev-g67 logged. g69b WON (UR beat A T14); g69a pending.
14:17Z g69 done (UR won; A lost mysteriously at 15 life vs 18 right after a Cave Entrance scry: CHECK for decking/engine bug in review). tokens: g69b 135767 g69a pending notification. g74 (boros) launching; review g69.
14:17Z g69a logged. g74a ae063c6b36819cb7f, g74b aa560cc708ebf3db7, rev-g69 a8e986f302913deb6. In flight players g71,g72,g73,g74; reviews g68 a6040ef4d85004337, g70 accbd6f6c2f49df73, g69. Next 75 = ur-cutter.
14:19Z rev-g68 logged; g70 review done (tokens pending). Omniscience-only loop OK (library 42 vs life 20).
14:19Z rev-g70 logged. g74a WON turn 2 (one-lander Boseiju+2 Petal, S&T for Omniscience); g74b pending.
14:19Z g74 done (A won game turn 2: Boseiju + 2 Petal, S&T Omniscience, Atraxa, Aluren, loop). g74a logged, g74b pending. g75 (ur) launched; review g74 spawning. Next 76 = boros.
14:20Z g74b logged. g75a a51c38e6c51c4104d, g75b a3f90069c7aa150ae, rev-g74 a72086ea0afa44dcb. In flight players g71,g72,g73,g75; reviews g69 a8e986f302913deb6, g74. Next 76 = boros.
17:00Z rev-g74 logged; g69 review done (tokens pending): A lost to combat T14, not decked, no engine bug; harness gap: loser sees nothing for auto-passed turns.
17:01Z CLOCK: sandbox clock jumped ~2.7h during g72 (hung pick); now 17:01Z by date. g72 done (Boros won T7, A lost; SECOND Ancient Tomb auto-pay death at 1 life, g72a prompt Aluren cast). g72a,g72b rev-g69 logged. g76 (boros) launched; review g72 spawning. Next 77 = ur-cutter.
17:01Z g76a a4a98b0b9d2820a06, g76b a61c92b217aaa8250, rev-g72 ac29bd5f315ab9e46. In flight players g71,g73,g75,g76; reviews g72 + g69/g70/g74 done; tokens: all logged except none pending. Next 77 = ur-cutter. IDEA after the run: fix auto-pay to avoid Ancient Tomb when other payment exists (2 deaths g65,g72).
17:03Z g75 done (A won game T5, spare Petal, Force war). g75a logged, g75b pending. g77 (ur) launched; review g75 spawning. Next 78 = boros.
17:03Z g71 done (UR won T14, A lost; A misread menu index after menu reorder: Forced a DRC instead of Veil). g75b,g71a,g71b logged. g78 (boros) launched; review g71 spawning. g77a a0847e89e169e899f g77b afefd05a3ba4e4a63 rev-g75 af1860d717e6e0a89. Next 79 = ur-cutter.
17:03Z g78a aa2355f4231000563, g78b a4edb8096d8005d0d, rev-g71 ab1d4d11ea2f7f4b7. In flight players g73,g76,g77,g78; reviews g72 ac29bd5f315ab9e46, g75 af1860d717e6e0a89, g71. Next 79 = ur-cutter.
17:05Z g73 done (UR won T12, A lost). g73b logged, g73a pending. g79 (ur) launched; review g73 spawning. Next 80 = boros.
17:05Z g73a logged. g79a a307f0ed943504b6e, g79b aa68f5f57eccbf75b, rev-g73 a105aa75e95e1d7f8. In flight players g76,g77,g78,g79; reviews g72 ac29bd5f315ab9e46, g75 af1860d717e6e0a89, g71 ab1d4d11ea2f7f4b7, g73. Next 80 = boros.
17:06Z g72 review done (tokens pending). CONFIRMED second Tomb auto-pay death (g72); players' ps visibility note: run players without shared process visibility.
17:07Z rev-g72 logged; g75 review done (tokens pending). Loop = 1 drain per 8 picks (about 8 picks/life point in lap terms; 134 picks for 17 life).
17:08Z rev-g75 logged; g71 review done (tokens pending). HARNESS UX: menu numbering shifts (Force listed only when spell on stack), caused g71a misread.
17:09Z g76b WON (Boros beat A T9); g76a pending. g80 (boros) launched; review g76 spawning (hold until g76a reported). rev-g71 logged. Next 81 = ur-cutter.
17:09Z g76 done (Boros won T9; A died while ordering Stock Up bottom at 10 life? check). g76a,g76b logged. g80a a5d6b44de54cbc0e9, g80b a41f1ff6104082f6e. Review g76 spawning.
17:09Z rev-g73 logged. rev-g76 a02ea8969fc38454f. In flight players g77,g78,g79,g80; reviews g76 + g77.. not yet. Next 81 = ur-cutter.
17:11Z g77 done (UR won game T10; A died the turn after Aluren cast, A suspects engine: check, likely Murktide attack on B's turn). g77b logged, g77a pending. g81 (ur) launched; review g77 spawning. Next 82 = boros.
17:11Z g77a logged. g81a ab61bba717b43f133, g81b ad1070b1cb69cff6f, rev-g77 a00b8642b11871660, rev-g76 a02ea8969fc38454f. In flight players g78,g79,g80,g81. Next 82 = boros.
17:13Z g78 done (A won game T8, 2nd Show and Tell for Omniscience, loop). g78b logged, g78a pending. g79b WON (UR beat A T12), g79a pending. g82 (boros) launched; reviews g78, g79 to spawn. Next 83 = ur-cutter.
17:13Z g79 done (UR won T12; A lost; Tomb auto-pay cost life again 14->12, 5->3). g78a,g79a,g79b logged. g83 (ur) launched; review g79 spawning. g82a a6b90097f376bf46f g82b a72663d0e63cad3be rev-g78 a4405a30d6e9607cb. Next 84 = boros.
17:13Z g83a a21aac08e8fa678d7, g83b a51b929ca5d53ba28, rev-g79 abcde1e490105db8b. In flight players g80,g81,g82,g83; reviews g76 a02ea8969fc38454f, g77 a00b8642b11871660, g78 a4405a30d6e9607cb, g79. Next 84 = boros.
17:15Z rev-g76 logged; g77 review done (tokens pending). Harness item recurring: the loser transcript ends with no final events; suggested fix = print final event block + life totals at game over to both seats.
17:16Z rev-g77 logged. g80a WON T6 (second Show and Tell for Omniscience); g80b pending.
17:16Z g80 done (A won game T6; Boros: second Show and Tell for Omniscience again). g80a logged, g80b pending. g84 (boros) launched; review g80 spawning. Next 85 = ur-cutter.
17:16Z g80b logged. g84a ac3b265787c2e9c45, g84b a3c08c9d2d09ebeba, rev-g80 abcd7ccb4adf39594. In flight players g81,g82,g83,g84; reviews g78 a4405a30d6e9607cb, g79 abcde1e490105db8b, g80. Next 85 = ur-cutter.
17:18Z rev-g78 logged; g79 review done (tokens pending). Tomb auto-pay CONFIRMED third case g79 (not decisive).
17:19Z rev-g79 logged. g82 done (A won game T6; Show and Tell gave B a free Windswept Heath -> Plateau -> Swords on Acererak trigger; Force answered). g82a/b tokens pending. g85 (ur) launched; review g82 spawning. NOTE: Misty offering Tundra is correct (Island type). Next 86 = boros.
17:19Z g81 done (A won game T9, Boseiju channel on Volcanic, spare mana Aluren), g82 done (A won), g84a WON T4 (S&T Omniscience; g84b pending). tokens logged. g86 (boros), g87 (ur) launched; review g81 spawning; g84 review after g84b. Next 88 = boros.
17:19Z g84 done (A won game T4 turn 2 kill, S&T Omniscience; Boros lost). tokens logged. In flight players g83,g85,g86,g87 (g86a a16ed23cc4dfa599e g86b a7e3b0e35732e9c61 g87a ada0bb24e86f77fd8 g87b ae1a8f63d34f6f636; rev-g81 affd82b7762b1c7c1); review g84 spawning. Next 88 = boros.
17:20Z rev-g84 ab9fc0bd46262625e. Reviews in flight: g80 abcd7ccb4adf39594, g82 ab1922827cd4f976d, g81 affd82b7762b1c7c1, g84.
17:22Z rev-g80 logged; g84 review done (tokens pending).
17:22Z rev-g84 logged; g82 review done (tokens pending).
17:22Z rev-g82 logged. g85a WON turn 3 (A won, UR lost); g85b pending.
17:23Z g85 done (A won game turn 3, Aluren T2 with City+Maze+Petal into one-land UR). g85a logged, g85b pending. g88 (boros) launched; review g85 spawning. Next 89 = ur-cutter.
17:23Z g85b logged. g88a ad5dcc5b4d7250256, g88b a0b4743d22cc12eb6, rev-g85 a0796c8fdbe0fe644. In flight players g83,g86,g87,g88; reviews g81 affd82b7762b1c7c1, g85. Next 89 = ur-cutter.
17:24Z g83 done (UR won game T10; A lost). g83b logged, g83a pending. g89 (ur) launched; review g83 spawning. Next 90 = boros.
17:24Z g83a logged. g89a a07c3355f89bb5cc4, g89b af5ca7232e3d01226, rev-g83 a6303b1a844224dd6. In flight players g86,g87,g88,g89; reviews g81 affd82b7762b1c7c1, g85 a0796c8fdbe0fe644, g83. Next 90 = boros.
17:25Z rev-g85 logged. g87b reported LOST (UR lost to S&T Omniscience loop T5); g87a report/tokens pending. g83 review a6303b1a844224dd6 running.
17:25Z g87 done (A won game T5; Veil then S&T Omniscience, 19 laps). g87b logged, g87a tokens pending. g90 (boros) launched; review g87 spawning. Next 91 = ur-cutter.
17:26Z g87a, rev-g81 logged. g90a a9ec646e88530cd90, g90b a86b452326cdd8091, rev-g87 abd8e2222be80d07a. In flight players g86,g88,g89,g90; reviews g83 a6303b1a844224dd6, g87. Next 91 = ur-cutter. Engine note g81: auto-pay tapped all 4 lands for Aluren (cost 4) left no float? check. Boseiju channel on a dual may be a mistake rule.
17:27Z g86 done (A won game T6; second Omniscience S&T, Swords on first Acererak, second Acererak looped). g86a logged, g86b pending. g91 (ur) launched; review g86 spawning. Next 92 = boros.
17:28Z g88 done (A won T6 Aluren hardcast + loop; B lost). g92 boros launched. Reviews pending g83,g87,g86; g88 review spawning. Next 93 = ur-cutter.
17:28Z g92a a32d191ba07c5b39f, g92b a37b9a695a777a810, rev-g88 aa01b5fa29717c469, g91a a573ad5fd86570c75, g91b aec5e50f8ca6959f9, rev-g86 a2b6d3cd12f381564. In flight players g89,g90,g91,g92; reviews g83,g87,g86,g88. Next 93 = ur-cutter.
17:29Z rev-g83 done (B won T10; top mistake Misty crack into untapped Wasteland). tokens pending.
17:29Z rev-g83 logged (140909). rev-g87 done (A won T5; B should have Forced the Veil), tokens pending.
17:30Z rev-g87 logged (119445). rev-g86 done (A won; B tapped out for Raptor; menu duplicate Cast entry = harness issue), tokens pending.
17:32Z rev-g86 logged (153755). g90a reported (A won T6 Aluren hardcast, 24->0); waiting g90b token/report.
17:32Z g90 done (A won T6, Aluren hardcast, B tapped out for Phlage/Karakas). g93 ur launched; rev-g90 spawning. Next 94 = boros.
17:32Z g90b logged. g93a a543c5e2463c37930, g93b a1973f6cdb4184651, rev-g90 a6cc9533044fc533c, rev-g88 aa01b5fa29717c469. In flight players g89,g91,g92,g93; reviews g88,g90 (g86,g87,g83 done). Next 94 = boros.
17:32Z rev-g88 done (B should have cast Bombardment; hold means hold sacrifices; possible engine item: Erode offered from exile with no legal target). tokens pending.
17:34Z rev-g88 logged (142215). g89a reported (A won T7; mull to 6, Show and Tell Omniscience); waiting g89a tokens and g89b.
17:34Z g89 done (A won T7 vs UR; mull to 6; both Show and Tells, second Omniscience; UR countered S&T #1 with FoW, then Forced FoW). g94 boros launched; rev-g89 spawning. Next 95 = ur-cutter.
17:34Z g89a/b logged. g94a a4f8b65ac266f837d, g94b a5ec711b79a4652d7, rev-g89 ab0a21c953bb74872. In flight players g91,g92,g93,g94; reviews g90 (a6cc9533044fc533c), g89. Next 95 = ur-cutter.
17:35Z g91b reported (LOST; threw Daze/FoW/Charm at S&T under Veil). Waiting for g91a hand-back/tokens.
17:35Z g91 done (A won T7 vs UR: waited a turn for Veil+S&T, Omniscience; UR threw Daze/FoW/Charm into Veil). g95 ur launched; rev-g91 spawning. Next 96 = boros.
17:35Z g91a logged. g95a a1aac64ed8254f4e2, g95b a8cb34d6cd79b973b, rev-g91 aea322961b7e74fc8. In flight players g92,g93,g94,g95; reviews g89 ab0a21c953bb74872, g90 a6cc9533044fc533c, g91. Next 96 = boros.
17:37Z rev-g90 done (no decisive mistake; Petal on turn drawn vs Spider-Woman; Karakas note). tokens pending.
17:39Z rev-g90 logged (119772). rev-g91 done (A won; B should Daze the Veil; never pitch Charm after Veil), tokens pending.
17:39Z rev-g91 logged (152475). rev-g89 done (no decisive mistake; candidate: S&T put-in Aluren vs Omniscience conflicts with Brady's line), tokens pending. Outstanding reviews: g89. Players in flight g92,g93,g94,g95.
17:39Z rev-g89 logged (165225). g92b reported (B lost; Omniscience-only loop ~15 laps). Waiting g92a hand-back.
17:39Z g92 done (A won T6 vs Boros: S&T Omniscience, Atraxa digs, Swords+Erode killed first Acererak, second Acererak looped under Omniscience alone). g96 boros launched; rev-g92 spawning. Next 97 = ur-cutter.
17:40Z g92a logged. g96a a1ed7a7e7a3e07f37, g96b a86e94892d705f654, rev-g92 ae597193b8b3df236. In flight players g93,g94,g95,g96; reviews g92 only (g89,g90,g91 done). Next 97 = ur-cutter.
17:42Z g93b reported: B (UR) WON T12 (Wasteland on Tomb, Daze on Stock Up, FoW on S&T, Murktide, Cutter+Bolt). Waiting g93a hand-back/tokens.
17:42Z g93 done (B UR won T12 vs Alurentell: T2 Stock Up into Daze with zero spare, Tomb Wastelanded, S&T Forced). g97 ur launched; rev-g93 spawning. Next 98 = boros.
17:42Z g93a logged. g97a a2db53f8d72822d4e, g97b a40998745d74ad6c5, rev-g93 a1f0a6e5e90ee2402. In flight players g94,g95,g96,g97; reviews g92 ae597193b8b3df236, g93. Next 98 = boros.
17:44Z rev-g92 done (A won; kill 18 drains; first Acererak with one Force into 4 white sources vs dig first; Karakas note), tokens pending.
17:44Z rev-g92 logged (179453). g96a reported (A won T6 vs Boros, Aluren + FoW backup). Waiting g96a tokens and g96b.
17:44Z g96 done (A won T6 vs Boros; Aluren + Force backup; B tapped out T5). g98 boros launched; rev-g96 spawning. Next 99 = ur-cutter.
17:45Z g96b logged. g98a a921fd5f02449feaa, g98b ac75f6853940d9999, rev-g96 ae530f4d40248ca38. In flight players g94,g95,g97,g98; reviews g93 a1f0a6e5e90ee2402, g96. Next 99 = ur-cutter.
17:45Z g95b reported: B (UR) WON T8 (Charm + free Daze on own Charm for flurry, 14 into 12). Waiting g95a hand-back/tokens.
17:46Z g95 done (B UR won T8 vs Alurentell; A Forced B's Force and the engine exiled Atraxa as the pitch card (A says no menu; verify); A ran flag+pick in one command, stale pick). g99 ur launched; rev-g95 spawning. Next 100 = boros.
17:46Z g95a logged. g99a a89d154630906bb64, g99b a9615c4ee0aeb8c22, rev-g95 aad4309e95d1bb1cc. In flight players g94,g97,g98,g99; reviews g93 a1f0a6e5e90ee2402, g96 ae530f4d40248ca38, g95. Next 100 = boros.
17:46Z g94b reported: B (Boros) WON T9 (Voice of Victory, Ocelot tokens, A stumbled and did little). Waiting g94a hand-back/tokens.
17:47Z g94 done (B Boros won T9: A kept no-payoff hand, Voice/Ocelot/Guide snowball. g94a listed the tasks/ scratch dir once (names only) - note, not tainted). g100 boros launched; rev-g94 spawning. Next 101 = ur-cutter.
17:47Z g94a logged. g100a a22550eda6f1cc9cc, g100b a7c4564255e69efcc, rev-g94 acf185fa6b4037111. In flight players g97,g98,g99,g100; reviews g93 a1f0a6e5e90ee2402, g96 ae530f4d40248ca38, g95 aad4309e95d1bb1cc, g94. Next 101 = ur-cutter.
17:48Z rev-g93 done (B won T12; no decisive mistake; B Wasteland on Tomb decided; engine item: no mana abilities so City mana cannot be held across land drop), tokens pending.
17:48Z rev-g93 logged (166668). rev-g96 done (A won; B tapped out T5 for Scout #2; Sand Warrior token haste check), tokens pending.
17:49Z rev-g96 logged (126271). rev-g95 done (A lost: Brainstorm put-back left Atraxa the only Force pitch; FoW single-candidate pitch is not a bug, harness could show it), tokens pending.
17:51Z rev-g95 logged (120896). g97a reported (A won T11 vs UR: Aluren hardcast while B tapped out on one land, 300 picks). Waiting g97a tokens and g97b.
17:51Z g97 done (A won T11 vs UR: UR Wastelanded Tomb twice, S&T lost to Daze+FoW war, then Aluren hardcast into B tapped out on one land. B reports no main-2 window after attacking: check). g97b tokens pending. g101 ur launched; rev-g97 spawning. Next 102 = boros.
17:51Z g97b, rev-g94 logged. rev-g94 done: engine bug Ocelot/city's blessing (Ascend should apply as 10th permanent arrives; token copies missed); keep leaning mulligan (odds counted dead cards). g101a a4d55effd1eb9837e, g101b a40311afbdba512ef, rev-g97 a3b07b3668dfc0340. In flight players g98,g99,g100,g101; reviews g97 only (g98 etc. later). Next 102 = boros.
17:53Z g100b reported: B (Boros) LOST T6 (A: S&T Atraxa T2-ish, Force on Swords, Aluren+Acererak loop). Waiting g100a hand-back/tokens.
17:53Z g100 done (A won T6 vs Boros: T1 Tomb/Petal/Stock Up, S&T Atraxa, Force on Swords, Aluren + loop 536 picks; B mull to 6 stuck on 2 lands... check B's report says mull 1-lander to 7). g102 boros launched; rev-g100 spawning. Next 103 = ur-cutter.
17:53Z g100a logged. g102a afb5a9b76ff9591d6, g102b a0032bd050f32f268, rev-g100 a35b305b69d1aeaee. In flight players g98,g99,g101,g102; reviews g97 a3b07b3668dfc0340, g100. Next 103 = ur-cutter.
17:54Z g98b reported: B (Boros) WON T9 (Guide/Ajani/Scout, Wasteland Tropical, Erode on Atraxa). Waiting g98a hand-back/tokens.
17:54Z g98 done (B Boros won T9 vs Alurentell: A kept no-creature hand, Tropical Wastelanded, Atraxa at 1 life unusable, Erode on Atraxa). g103 ur launched; rev-g98 spawning. Next 104 = boros.
17:55Z g98a, rev-g97 logged. rev-g97 done (A won; B binned Brainstorm, last Force pitch; no main-2 window was a misread). g103a ad7f26a526a6b1cde, g103b a37f1040bee3ec95d, rev-g98 a8fef52dfdab677d0. In flight players g99,g101,g102,g103; reviews g95 done, g100 a35b305b69d1aeaee, g98 a8fef52dfdab677d0. Next 104 = boros.
17:56Z g102a reported: A WON game turn 4 (A's 2nd turn) vs Boros (T2 S&T Aluren, Acererak loop). Waiting g102a tokens and g102b.
17:56Z g102 done (A won game turn 4 vs Boros: T2 Petal, S&T Aluren, Acererak loop). g104 boros launched; rev-g102 spawning. Next 105 = ur-cutter.
17:56Z g102b logged. g104a a02342578af73e567, g104b a231277e32c662375, rev-g102 a3bd7e3441a6529d1. In flight players g99,g101,g103,g104; reviews g100 a35b305b69d1aeaee, g98 a8fef52dfdab677d0, g102. Next 105 = ur-cutter.
17:58Z rev-g100 done (A won; B passed Guide/Raptor free under Aluren; B mulled to 6 not 7 - players misreport mulligan sizes again), tokens pending.
17:58Z rev-g100 logged (148001). g99a reported: A WON T9 vs UR (Aluren while B tapped out for Murktide; City 5 mana). Waiting g99a tokens and g99b.
17:58Z g99 done (A won T9 vs UR: Aluren after B tapped out for Murktide; B lost with no end log). g99b tokens pending. g105 ur launched; rev-g99 spawning. Next 106 = boros.
17:59Z g99b, rev-g98 logged. g105a adbe63c60afced5b8, g105b a1faa6b522bc31038, rev-g99 a68b93ea947d24ffe. In flight players g101,g103,g104,g105; reviews g102 a3bd7e3441a6529d1, g99 a68b93ea947d24ffe (g97,g98,g100 done). Next 106 = boros.
17:59Z g104a reported: A WON game turn 4 vs Boros (T2 Aluren with 2 Petals, loop 22->0). Waiting g104a tokens and g104b.
17:59Z g104 done (A won game turn 4 vs Boros: T2 Aluren with Maze+Tropical+2 Petals; B Guide/Ajani, no interaction; menu offered B uncastable Cast options while tapped = Aluren free-cast rule). g106 boros launched; rev-g104 spawning. Next 107 = ur-cutter.
17:59Z g104b logged. g106a a4ae1cf8d4a814ab3, g106b a1da34c5be0a41af2, rev-g104 a4e875c161c057170. In flight players g101,g103,g105,g106; reviews g102 a3bd7e3441a6529d1, g99 a68b93ea947d24ffe, g104. Next 107 = ur-cutter.
17:59Z rev-g102 done (A won; B tapped out for Voice with Swords in hand), tokens pending.
18:02Z rev-g102 logged (114187). g101a reported: A WON T7 vs UR (mull; waited a turn; Aluren while B tapped out for Murktide). Waiting g101a tokens and g101b.
18:02Z g101a logged (123872). rev-g104 done (A won; B Ajani over Spider-Woman; A held Petals T1), tokens pending. Waiting g101b.
18:02Z rev-g104 logged (99181). g103a reported: A WON T9 vs UR (Aluren hardcast when B tapped out for Murktide). Waiting g103a tokens, g101b, g103b.
18:02Z g103 done (A won T9 vs UR: Aluren hardcast after B tapped out for Murktide; B delve 5 all-in). g103b tokens pending. g107 ur launched; rev-g103 spawning. Next 108 = boros.
18:03Z g101 done (A won T7 vs UR; B tapped out Murktide, Aluren loop; mulligan; waited one turn). g103b, g101b logged. g107a a061d1b5cb49bb11e, g107b af3e4efdef5ab87d4, rev-g103 ad11a317034101fcb. g108 boros launched; rev-g101 spawning. Next 109 = ur-cutter.
18:04Z g108a af92fa9d41e99102d, g108b a8725adbac379385d, rev-g101 abdba0af7fae4f32f. rev-g99 done (A won; B forced back pitching Daze, last Force dead), tokens pending. In flight players g105,g106,g107,g108; reviews g103 ad11a317034101fcb, g101 abdba0af7fae4f32f. Next 109 = ur-cutter.
18:04Z rev-g99 logged (164049).
18:07Z rev-g103 done (A won; B tapped out for Murktide with Charm in hand; A Stock Up vs S&T T7), tokens pending.
18:07Z rev-g103 logged (122219). g106a reported: A WON T6 vs Boros (Aluren hardcast, B ~25 tokens; Aluren over S&T because of Bombardment). Waiting g106a tokens and g106b.
18:07Z g106 done (A won T6 vs Boros, Boros made ~25 tokens, A won a turn ahead; B's report conflicts (ended at B's end step turn 5?) - reviewer to verify timing). g106b tokens pending. g109 ur launched; rev-g106 spawning. Next 110 = boros.
18:08Z g106b, rev-g101 logged. rev-g101 done (A won; B played Tarn instead of Wasteland on Tropical). g109a a96c9486ff636693b, g109b a0d705e677876fd36, rev-g106 a6ff5d835c2081cd9. In flight players g105,g107,g108,g109; reviews g106 only (g99, g101, g103 done; g105 pending its players). Next 110 = boros.
18:08Z g108a reported: A WON game turn 4 vs Boros (T2 Petals early, Aluren, loop). Waiting g108a tokens and g108b.
18:08Z g108 done (A won game turn 4 vs Boros: Petals early, Aluren; B Raptor discovered Wasteland + Swords, Swords exiled). g108b tokens pending. g110 boros launched; rev-g108 spawning. Next 111 = ur-cutter.
18:08Z g108b logged. g110a a8cdf068158d75ffa, g110b a7f810fba8e971f5f, rev-g108 a6c64c6346e340861. In flight players g105,g107,g109,g110; reviews g106 a6ff5d835c2081cd9, g108. Next 111 = ur-cutter.
18:12Z rev-g108 done (A won; B Raptor over Bombardment; possible engine check: Raptor discover leaving Swords exiled), tokens pending.
18:12Z rev-g108 logged (112158). rev-g106 done (A won T6 own main phase; Ocelot/blessing engine OK here), tokens pending.
18:13Z rev-g106 logged (137642). g109a reported: A WON T9 vs UR (3 Aluren hand; second Aluren after first Forced; B tapped out for Murktide). Waiting g109a tokens and g109b.
18:13Z g109 done (A won T9 vs UR; first Aluren Forced, second Aluren after B tapped out for Murktide; B reports YOU LOST mid-delve with no explanation, same as g103/g99 pattern: A's Aluren loop while B tapped out). g109b tokens pending. g111 ur launched; rev-g109 spawning. Next 112 = boros.
18:13Z g109b logged. g111a a35121a8b77365927, g111b a5b7be7434f3a31d7, rev-g109 ae06cd2d20105cdc9. In flight players g105,g107,g110,g111; reviews g109, g105 pending players, g107 pending players, g110 pending. Next 112 = boros.
18:14Z g110b reported: B (Boros) WON T7 vs Alurentell (Guide/Ocelot/Phlage, Bombardment reach, 19->7->dead). Waiting g110b tokens and g110a.
18:14Z g110 done (B Boros won T7: A kept no-payoff 4 lands + 2 Petals + Ponder; Guide/Ocelot/Phlage/Bombardment). g110a tokens pending. g112 boros launched; rev-g110 spawning. Next 113 = ur-cutter.
18:14Z g110a logged. g112a a43713d270aa030db, g112b a45122fd3667a0e5a, rev-g110 ad7a2b1e306487efc. In flight players g105,g107,g111,g112; reviews g109 ae06cd2d20105cdc9, g110 ad7a2b1e306487efc. Next 113 = ur-cutter.
18:15Z g107b reported: B (UR) WON T12 vs Alurentell (Daze, Wasteland Tomb, FoW on Aluren, Cutter+Bolt). Waiting g107b tokens and g107a.
18:15Z g107 done (B UR won T12 vs Alurentell: bare Stock Up into Daze, Tomb Wastelanded, Aluren with no creature Forced, Murktide Forced; DRC/Bolt/Monk). g107a tokens pending. g113 ur launched; rev-g107 spawning. Next 114 = boros.
18:15Z g107a logged. g113a a1964635e73ed8f8b, g113b a546b609a7050f4c8, rev-g107 a9c30fab54eed8f42. In flight players g105,g111,g112,g113; reviews g109 ae06cd2d20105cdc9, g110 ad7a2b1e306487efc, g107. Next 114 = boros. g105 players still running a long time (started ~17:58Z).
18:17Z rev-g109 done (A won; B 'lost mid-delve' = view gap not bug; B Forced pitching Charm = decisive), tokens pending.
18:17Z rev-g109 logged (124965). rev-g110 done (B won; A's keep mana+one cantrip is a mulligan), tokens pending.
22:00Z Coordinator relay says real time 22:00Z (sandbox clock lags). Past the 21:00Z stop: no more launches. g113 players stopped (aborted, not counted). Remaining: players g105, g111, g112; review g107; then wrap-up. Brady 20:45Z idea: playbook + skills, group rules under principles - do after wrap-up.
22:01Z g105 done (A WON T9 vs UR: Veil answered FoW on S&T, Atraxa, Aluren, loop; A fell to 4 life). g105a/b logged. rev-g105 spawning. In flight players g111 (a35121a8b77365927, a5b7be7434f3a31d7), g112 (a43713d270aa030db, a45122fd3667a0e5a). No more launches.
22:02Z rev-g105 a95f5671c79569ea4; restructure agent (playbook+skills per Brady 20:45Z idea) acada6a87762ef070 writes to restructure/. Second-night stats so far: A 42/67 (62.7%): UR 18/33 (A always on play), Boros 24/34 (A always on draw): seat coupled to opponent deck (design flaw to report). Players 14.66M tokens, reviews 8.99M so far.
22:02Z g112a reported: A LOST T7 vs Boros (kept no-enabler 7: Tundra, Savannah, City, Omniscience, Veil, 2 Atraxa). Waiting g112a tokens, g112b, g111a, g111b.
22:02Z g112 done (B Boros WON T7 vs Alurentell: A kept no-enabler 7, Guide/Raptor-Ajani/Bombardment). g112a logged, g112b tokens pending. rev-g112 spawning.
22:02Z g112b logged. rev-g112 aed34d9d8164e72da. Remaining: g111 players (a35121a8b77365927, a5b7be7434f3a31d7), reviews g105 a95f5671c79569ea4, g112, g111; restructure acada6a87762ef070.
22:05Z rev-g112 done (A lost at the keep), tokens pending. Waiting: g111 players, rev-g105 (a95f5671c79569ea4), restructure (acada6a87762ef070).
22:06Z rev-g112 logged (116777). rev-g105 done (A won; no priority after blockers is a real harness gap; auto-pay taps Tomb first cost A 6 life over 3 casts; Stock Up sorcery wasted a turn), tokens pending.
22:07Z rev-g105 logged (228459). g111b reported: B (UR) LOST (A won g111: Veil + S&T, FoW war, Aluren + Acererak loop). Waiting g111a hand-back and tokens.
22:07Z g111 done (A WON T7 vs UR: 2-lander keep, Veil + S&T Atraxa, Aluren hardcast, Force war, loop). g111a tokens pending. rev-g111 spawning. Last game of the night.
22:08Z logged g111a 125965
22:10Z restructure draft done (60-line playbook, 14 principles, 11 skills, MAPPING). tokens 123608. waiting rev-g111 then SYNTHESIS2
