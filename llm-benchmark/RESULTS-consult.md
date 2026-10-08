# LLM player with calculator tools: results (2026-10-07)

Thread "LLM player with net tool". Everything here ran on the pinned engine (175 card definitions, old Alurentell list without Orim's Chant) and the m1 net, because that is the only combination the earlier 18-2 baseline can be compared with. The live engine (Storm, Tron, Orim's Chant) was only smoke-tested through a remapped net (`netremap_names.rs`); its matchup nets are being retrained tonight on Brady's PC.

## Short answer
- The tools work and match exact math (`harness/selftest.sh`), but **they have not made the LLM play better yet**. On Brady's labeled tough spots the tool-using player agrees with him on 11 of 19 library-heavy positions, against 12 of 19 for the same player with no tools.
- The consult tool (net and short search read) did not help either: 19/36 vs 21/36 over all 36 positions. The LLM leans on the bot's top pick, and on these positions the bot agrees with Brady only 12 times in 36.
- Full games with the tool were only piloted (4 of 4 won against net-search UR Cutter at 32 iterations). The 20-game A/B was stopped: UR is at a ceiling (the LLM is 18-2 there), so it cannot show a gain.

## Tools (all from the acting player's own view, no hidden information)
| Command | What it answers |
|---|---|
| `consult` | Net policy plus a short determinized search over the legal options: net %, search visit %, search win %. |
| `odds` | Hypergeometric odds over your own library, known top cards drawn first (e.g. P(at least one Aluren in the top N)). |
| `opp` | Odds over the opponent's possible hands, sampled from their decklist minus everything you have seen (e.g. P(Force of Will or Daze in hand)). |
| `sim` | Monte Carlo what-ifs with a small script language: draw, turn, brainstorm (put back chosen cards), ponder, look, shuffle, drops, samples, opp; goals with and/or/not, any(...), sums and specials (lands, lands_bf, lands_playable, nonlands, creatures, hand_size). Same seed gives comparable numbers between scripts. |

`selftest.sh` checks `odds`, `opp` and `sim` against closed-form values and passes. `sim` answers the layered questions Brady asked for, for example "if I Brainstorm now and put back X and Y, what is P(Aluren and three mana by my next turn)?".

## Tough-spot test with all tools
Position mode (`pos.sh`, `BRIEF-pos.md`) replays a recorded position to the decision, gives a fresh Claude agent the tools and takes one answer. Subset: the 19 Alurentell positions where the library or hidden cards matter (keep or mulligan, fetchland choice, Brainstorm), labeled by Brady in batches 1 and 2.

| Setup | Agrees with Brady (of 19) |
|---|---|
| No tools, blinded text only | 12 |
| Static consult numbers appended to the prompt | 10 |
| Deep search bot (800 iterations) | 6 |
| **Tools: consult, odds, opp, sim** | **11** |

Tool use per position: consult 15, odds 11, sim 5 (the player chose when to call). About 60k tokens per position, mostly the brief and playbook it reads. Eight misses (I did not audit each answer's reasoning, so I cannot say yet whether a miss came from ignoring the numbers or from the numbers not deciding the spot).

What this says: with n = 19 the difference between 11 and 12 is noise, so the honest reading is "no measurable gain", not "tools hurt". The positions were also selected because the bot gets them wrong, which may bias toward cases where numbers do not decide the answer (the odds tools do not model things like Daze and Force tempo). The mulligan and keep spots are where `odds` and `sim` should matter most, and there the player did call them (s5-g40, s5-g19, s4-g44) and matched Brady 3 of 3, but that is three positions.

## Full-game pilot (consult tool, UR Cutter net-search 32 iterations, pinned engine)
| Game | Result | Prompts | Prompt chars | Consults |
|---|---|---|---|---|
| c201 | Won | 44 | 48k | 2 |
| c202 | Won | 78 | 109k | 2 |
| c203 | Won | 118 | 177k | 2 |
| c204 | Won | 132 | 198k | 2 |

For comparison the no-tool baseline was 18-2 at about 175k tokens per game. The pilot shows the harness works end to end and the player uses the tool sparingly; it is not evidence of an improvement. Subagent token counts per game were not captured for these four; prompt characters are the proxy.

## Opponent strength reference (net-search Alurentell, m1 net, 100+ games per row unless noted)
| Matchup (Alurentell side vs UR Cutter side) | Alurentell wins |
|---|---|
| Net 1000 iterations vs net 32 iterations | 68 of 102 (67%) |
| Net 32 vs net 1000 | 22 of 40 (55%) |
| Net 1000 vs net 1000 | 18 of 32 (56%) |
| Live engine, remapped net, 32 vs 32 | 28 of 60 (47%) |

A much stronger opponent (1000 iterations) is not much harder than the 32-iteration one once the Alurentell side also searches deeply, so more search is a weak lever for opponent strength; better nets are the real one. The live-engine row is lower partly because the net is remapped and has zero weights for new cards.

## Caveats
- Pinned engine and old list. Results do not transfer automatically to the live engine with Orim's Chant.
- n is small everywhere (19 positions, 4 games). Only large effects would show.
- Subagent model for these tests may differ from the one used in the 18-2 baseline (the baseline run did not record it).
- A per-game blunder count (the metric Brady suggested) is not built yet.
- Selection bias in the tough-spot subset, as above.

## Where to go next
1. Retest tools on batch 4 once Brady labels it, and on mulligan and keep spots in other Alurentell+X matchups, where hidden-card odds should matter more.
2. Put worked examples (Brady's nearest labeled positions) and per-matchup briefs in the prompt; that is where the 58% over the bot comes from today.
3. Build the post-game review loop and the blunder counter, then run the full-game gauntlet on Boros and Reanimator with the retrained live nets.

See `ELITE-PLAN.md` for the full plan.
