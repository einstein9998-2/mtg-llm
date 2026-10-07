# From okay to elite: plan for the LLM player (2026-10-07)

## Two tiers
- **Elite single-game player.** The LLM pilots one game and may consult the net and search as a tool. Used for showcase games, tough spots and checking that a plan or a sideboard change works. About 175k tokens a game.
- **Volume engine.** Net plus search does all self-play, training and sideboard A/B testing (about 5 CPU seconds a game, no tokens). The LLM only grades a sample: tough spots, odd net decisions, drafted playbook rules for Brady's sign-off.
- **Flywheel.** Brady's answers and the LLM's reviews become labels that improve the net; a better net makes the LLM's consult tool more useful.

## What makes the LLM better (no fine-tuning: subscription only, no paid API)
Ranked by expected gain per token:

1. **Calculator tools** (built: `lg.sh consult | odds | opp | sim`). `consult` = net plus short search read; `odds` and `opp` = hypergeometric odds over your own library and the opponent's possible hands; `sim` = Monte Carlo what-ifs ("if I Brainstorm and put back X and Y, P(Aluren plus three mana next turn)"). All from the player's own view only. First test (`RESULTS-consult.md`): no measurable gain on tough spots yet (11/19 with tools vs 12/19 without), so ranks below items 2 to 5 until retested on mulligan and keep spots.
2. **Playbook** (`llm-player/alurentell-playbook.md`, Brady's rules). Keep it short and ordered by how often it matters; every rule that fails to change a decision gets cut, because the player pays tokens to read it.
3. **Per-matchup briefs.** One page per Alurentell+X matchup: their win conditions, what to hold, sideboard plan, Brady's play tips (Brainstorm around Bowmasters, Orim's Chant timing, Veil vs Thoughtseize...). Loaded only for that matchup.
4. **Brady's tough-spot answers as worked examples.** 3 to 5 nearest positions, retrieved by matchup and decision kind and shown with Brady's choice and reason. Cheap, and it carries exactly the knowledge the net lacks.
5. **Post-game self-review of losses** (and a sample of wins). A separate reviewer Claude reads the game log with hindsight, finds the first real mistake, writes it up as a candidate rule. Rules go to Brady for sign-off, then into the playbook, then are tested by replaying the seeds that lost.
6. **Mechanical loops automated** (Acererak/Lost Mine, attack declarations) so prompts and tokens go to real decisions. Known from the benchmark: 375-391 prompts in the worst games came from the loop alone.

## How to know it worked
- UR is at about 90% (ceiling), so it can show harm but not gains. Measure on the hard matchups where net and search sit near 50%: **Boros** and **Reanimator**, plus the mirror.
- Opponent strength matters: use net-search opponents with more iterations once the live nets exist, and the matchup nets from tonight's PC run.
- Always report win rate **and** tokens per game; 20 games gives a +-20 point interval, so only large effects are visible. Use paired seeds where possible and treat ties as ties.
- **Primary metric: agreement with Brady's labeled tough spots** (LLM no-tool baseline 21/36), then blunders per game. Win rate only on hard matchups.
- Count process errors, not just results: wasted mana before the land drop, casting into open Daze/Force, wrong menu entry. Those are fixable with rules and are visible with far fewer games than win rate.

## Order of work
1. Consult tool + 20-game UR test (this thread). Pinned engine so the old 18-2 is a clean control.
2. Same test on Boros and Reanimator with the live engine and the retrained matchup nets (needs the PC run).
3. Review loop on the losses from step 1 and 2; first drafted rules to Brady.
4. Matchup briefs from the existing sideboard plans and tough-spot notes; worked-example retrieval.
5. Re-run the hard-matchup gauntlet with all pieces on; keep only pieces that moved win rate or tokens.
