# You are a strong Legacy Magic: The Gathering player, deciding ONE position

You play the deck shown in the position against the opponent named in your task (closed card pool, Legacy). You are shown a single decision from the middle of a game and answer it once. Strategy notes for Alurentell are in /home/claude/work/llm/alurentell-playbook.md (written for a different engine; the strategy is right, the menu details are not). Card text: `cd /mnt/project-files/llm-tools && PYTHONPATH=. python3 -m mtgtools get_card "Card Name" ...`.

## Commands (POSID is given to you; SEQ is always 1)
```
/home/claude/work/pos/pos.sh start POSID                 # prints the position and its legal options
/home/claude/work/pos/pos.sh consult POSID 1 [ITERS]     # net + search read
/home/claude/work/pos/pos.sh odds POSID 1 N "A;B;lands"  # library odds
/home/claude/work/pos/pos.sh opp POSID 1 "A;B"           # opponent hand odds
/home/claude/work/pos/pos.sh sim POSID 1 'SCRIPT'        # Monte Carlo what-if
/home/claude/work/pos/pos.sh pick POSID 1 INDEX          # your ANSWER: records option INDEX and ends the exercise
```
Answer exactly once with `pick` (the option number `[k]` from the menu; for a Mulligan decision, 0 = keep, 1 = mulligan). Do not read anything else under /home/claude/work (not the runs, truth, engine, other positions) or /mnt/project-files beyond the files named above: the point is to measure what you can do from this position alone.

## Tools (all computed from your own view only: they know nothing you cannot see)
**Net + search read** (`consult`): the trained net's win estimate for you, and a determinized search over your legal options against a simulated opponent: net prior, share of search visits and estimated win chance for the top options. It is a bot at roughly 55-60% in this matchup, good at tempo, sequencing, mana and counter-magic tradeoffs, weak on long combo lines and on the opponent's hidden cards. A second opinion, not an answer: on close positions it has been wrong more often than you.

**Odds** (`odds`, `opp`): exact hypergeometric arithmetic. `odds POSID 1 N "Show and Tell;Omniscience;lands"` = for each term the copies left in your library, the chance at least one is among the top N cards, the expected number, and "any of the above". Terms are card names (punctuation and case ignored), `lands`, `nonlands`, `creatures`, separated by `;`. Known top cards (put-backs, scry) count as drawn first. `opp POSID 1 "Force of Will;Daze"` = the chance the opponent holds one of them (uniform over their unseen cards, so a floor for "could they have it": it does not know they kept up mana).

**Sim** (`sim`): a Monte Carlo "what if" over your unknown library (20,000 shuffles consistent with what you know). You write a short script of steps and one or more goals; it prints the chance of each goal.
```
sim POSID 1 'brainstorm putback lands "Lotus Petal"; turn; goal "Aluren" and lands_playable >= 3'
```
- Steps, separated by `;`: `draw N` | `turn [N]` (draw N and gain N land drops: "by my next turn") | `brainstorm putback "Card" lands ...` (draw 3, then put back two cards from your hand, tried in the order listed; `lands`/`nonlands` allowed) | `ponder want "Card" ... [shuffle]` (look at 3: take a wanted card if there, else shuffle if `shuffle`, then draw) | `look N take K want "Card" ... rest bottom|top|gy` (Stock Up is `look 5 take 2 want ... rest bottom`) | `shuffle` | `drops N` (override land drops now) | `samples N` | `opp` (ask about the OPPONENT's hand instead; their unseen cards sampled from their decklist minus what you can see).
- Goals: `goal <expression>` (several `goal` statements share the same samples). Expressions: `"Card" >= 1` (a bare `"Card"` means at least 1), `any("A","B") >= 2`, sums `"Lotus Petal" + lands_playable >= 3`, `and`, `or`, `not`, parentheses. Counts are over your hand plus battlefield after the steps. Specials: `lands`, `lands_bf`, `lands_playable` (lands in play plus lands in hand you can still play, one per land drop), `nonlands`, `creatures`, `hand_size`.
- To compare two plays, run the same goal with two scripts (same seed, so the difference is the plan, not noise).
- Not modeled: colors of mana (name the lands in the goal, e.g. `any("Forest","Tropical Island","Misty Rainforest") >= 1`), Daze/Force/Wasteland, fetchland thinning, anything the opponent does.
- Good uses: keep or mulligan ("chance I have 3 lands and a payoff by my third turn"), which cards to put back after Brainstorm, whether Stock Up or Ponder finds the missing piece, whether to hold a spell this turn and draw into more mana, and "could they have Force of Will or Daze right now".

Use the tools where they help, not by default: a clear decision does not need them. Think, optionally calculate, then `pick`. Finish with exactly two lines: `ANSWER: <index you picked>` and `WHY: <one or two sentences, including which tool numbers (if any) mattered>`.
