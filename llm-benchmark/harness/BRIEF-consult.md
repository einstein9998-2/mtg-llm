# You are the player in a Magic: The Gathering game (Legacy, 1v1, closed card pool)

You play **Alurentell** (Aluren + Show and Tell + Omniscience combo) against **UR Cutter** (tempo: Dragon's Rage Channeler,
Murktide Regent, Cori-Steel Cutter, Daze, Force of Will, Lightning Bolt, Wasteland, Unholy Heat, Prismari Charm).
Both 75-card lists are in /home/claude/work/llm/decks/alurentell.txt and /home/claude/work/llm/decks/ur-cutter.txt (main deck, blank line, sideboard; no sideboarding in this game). Your strategy notes are in
/home/claude/work/llm/alurentell-playbook.md (written for a different engine; the strategy is right, the menu/prompt details are not).
Card text: `cd /mnt/project-files/llm-tools && PYTHONPATH=. python3 -m mtgtools get_card "Card Name" ...` (works for every card in both decks).

## How to play: ONE command per decision
```
/home/claude/work/llm/lg.sh start  TAG SEED SEAT 0     # starts the game, prints prompt 1
/home/claude/work/llm/lg.sh pick   TAG SEQ INDEX       # answers prompt SEQ with option INDEX, prints the next prompt
/home/claude/work/llm/lg.sh show   TAG                 # re-prints the current prompt without answering
/home/claude/work/llm/lg.sh consult TAG SEQ [ITERS]    # NEW: ask the net + search bot for its read of the current prompt (does NOT answer it)
/home/claude/work/llm/lg.sh odds    TAG SEQ N ["Card A;Card B;lands"]   # NEW: chance of seeing those cards in the top N of YOUR library
/home/claude/work/llm/lg.sh opp     TAG SEQ ["Force of Will;Daze"]     # NEW: chance the OPPONENT holds those cards right now
/home/claude/work/llm/lg.sh sim     TAG SEQ 'SCRIPT'                   # NEW: Monte Carlo what-if over your unknown cards (see below)
```
- SEQ is the number in `=== PROMPT n ===` of the prompt you are answering; INDEX is the `[k]` of the option you choose. A wrong SEQ is rejected, so a stale answer cannot slip through.
- **One pick per command, always.** Indices change after every action. Never chain picks in a loop or guess the next menu. `[0]` is always Pass priority (or the first option of non-priority decisions): never use pick 0 just to look; use `show`.
- The engine skips windows where you could only pass, and on the opponent's turn it only stops for you when something is on the stack or at their end step. Priority passes with an empty stack move the game on; with something on the stack, passing lets it resolve (the opponent gets priority first, then it resolves).
- Fetchlands: activate (costs 1 life), then choose the land. Mana is paid automatically when you cast a spell (Ancient Tomb deals 2 damage each time it is tapped; Lotus Petal is sacrificed). Force of Will appears in the menu when you have a blue card to exile.
- Multi-card choices are asked one card at a time (Ponder: order the three cards top first, then "Do it?" = shuffle yes/no; Stock Up: pick two, then order the rest to the bottom; Atraxa: pick one card per type, then Done; Show and Tell: pick the card to put in). If the option list shows a name once, it may stand for several copies.
- The prompt shows only what you are entitled to see: your hand, both battlefields and graveyards, the stack, opponent's hand size, cards you know, and the events since your last prompt. Library counts are shown. You never see the opponent's hand or either library order beyond what you have revealed.
- The game ends with `=== GAME OVER === YOU WON/YOU LOST`. Play to the end, make your best decisions, do not concede.

## Tools (all computed from your own view only: they know nothing you cannot see)
**Net + search read** (`consult`): the trained net's win estimate for you, and a determinized search over your legal options against a simulated opponent: net prior, share of search visits and estimated win chance for the top options. It is a bot at roughly 55-60% in this matchup, good at tempo, sequencing, mana and counter-magic tradeoffs, weak on long combo lines and on the opponent's hidden cards. A second opinion, not an answer: on close positions it has been wrong more often than you.

**Odds** (`odds`, `opp`): exact hypergeometric arithmetic. `lg.sh odds TAG SEQ N "Show and Tell;Omniscience;lands"` = for each term the copies left in your library, the chance at least one is among the top N cards, the expected number, and "any of the above". Terms are card names (punctuation and case ignored), `lands`, `nonlands`, `creatures`, separated by `;`. Known top cards (put-backs, scry) count as drawn first. `lg.sh opp TAG SEQ "Force of Will;Daze"` = the chance the opponent holds one of them (uniform over their unseen cards, so a floor for "could they have it": it does not know they kept up mana).

**Sim** (`sim`): a Monte Carlo "what if" over your unknown library (20,000 shuffles consistent with what you know). You write a short script of steps and one or more goals; it prints the chance of each goal.
```
lg.sh sim TAG SEQ 'brainstorm putback lands "Lotus Petal"; turn; goal "Aluren" and lands_playable >= 3'
```
- Steps, separated by `;`: `draw N` | `turn [N]` (draw N and gain N land drops: "by my next turn") | `brainstorm putback "Card" lands ...` (draw 3, then put back two cards from your hand, tried in the order listed; `lands`/`nonlands` allowed) | `ponder want "Card" ... [shuffle]` (look at 3: take a wanted card if there, else shuffle if `shuffle`, then draw) | `look N take K want "Card" ... rest bottom|top|gy` (Stock Up is `look 5 take 2 want ... rest bottom`) | `shuffle` | `drops N` (override land drops now) | `samples N` | `opp` (ask about the OPPONENT's hand instead; their unseen cards sampled from their decklist minus what you can see).
- Goals: `goal <expression>` (several `goal` statements share the same samples). Expressions: `"Card" >= 1` (a bare `"Card"` means at least 1), `any("A","B") >= 2`, sums `"Lotus Petal" + lands_playable >= 3`, `and`, `or`, `not`, parentheses. Counts are over your hand plus battlefield after the steps. Specials: `lands`, `lands_bf`, `lands_playable` (lands in play plus lands in hand you can still play, one per land drop), `nonlands`, `creatures`, `hand_size`.
- To compare two plays, run the same goal with two scripts (same seed, so the difference is the plan, not noise).
- Not modeled: colors of mana (name the lands in the goal, e.g. `any("Forest","Tropical Island","Misty Rainforest") >= 1`), Daze/Force/Wasteland, fetchland thinning, anything the opponent does.
- Good uses: keep or mulligan ("chance I have 3 lands and a payoff by my third turn"), which cards to put back after Brainstorm, whether Stock Up or Ponder finds the missing piece, whether to hold a spell this turn and draw into more mana, and "could they have Force of Will or Daze right now".

Use them where a decision turns on a number, roughly 5-15 calls a game; not on decisions with an obvious answer. Disagree with the bot when your combo or the playbook says so; the numbers from `odds` and `sim` are facts about your library, the `consult` read is an opinion.

## Rules of the exercise (important)
- Use ONLY the `lg.sh` commands (start, pick, show, consult, odds, opp, sim) for the game and `get_card` for card text, plus the files named above. **Do not read, list or search anything else**: not the other directories under /home/claude/work, not /home/claude/work/llm/runs/*/truth, not engine source, logs or processes. The point is to measure what you can do from the prompts alone; looking at anything else would make the result meaningless.
- Do not write notes to files. Think as much as you need before each pick, but keep tool use to `lg.sh`.
- At the end report in this exact format and nothing long: `RESULT: WON|LOST|DRAW`, turn count, number of prompts, number of consult/odds/sim calls, then up to 5 short bullets: key plays, mistakes you think you made (be honest), anything about the prompts or menus that was confusing or looked like an engine bug, and one bullet on the consult tool and the odds tools: did they change a decision, and were they right or wrong when you disagreed.

## Menu tip from earlier games
When Aluren is out and you also have mana for a hard cast, a creature can appear twice as `Cast X` (the same label). Earlier players found the first entry is the hard cast (taps mana, Ancient Tomb damage) and the later one is the free Aluren cast. Check your mana pool and life after casting to confirm which you got.
- Tomb of Annihilation rooms (Veils of Fear, Sandfall Cell) ask "Do it?": Yes means pay the cost to avoid the life loss (discard a card / sacrifice a permanent), No means take the life loss. A Lotus Petal only pays for Daze once it is on the battlefield.
- Scry/surveil prompts: `Choose <card>` sends that card to the bottom (scry) or graveyard (surveil); `Done` keeps it on top. Atraxa's reveal keeps listing types you already picked, so re-read the menu before every pick.
