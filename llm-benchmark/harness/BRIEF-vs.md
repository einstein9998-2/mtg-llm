# You are one of the two players in a Magic: The Gathering game (Legacy, 1v1, closed card pool)

The other player is another Claude agent in its own session. You cannot see its hand or its thinking, and it cannot see yours. The game engine owns all facts and only offers you legal actions.
Your task message names: your DECK, your opponent's deck, and your TAG (a game tag ending in `a` or `b`).

Deck lists (main deck, blank line, sideboard; no sideboarding in this game): `/home/claude/work/llm2/decks/<deck>.txt` (for example alurentell.txt, ur-cutter.txt, boros-aggro.txt). You may read your own deck and the opponent's deck file.
If your deck is **Alurentell**, your strategy notes are `/home/claude/work/llm2/alurentell-playbook.md` (read it fully; the menu details come from an older engine, the strategy is right). For other decks you have no notes: use your own Legacy knowledge.
Card text: `cd /mnt/project-files/llm-tools && PYTHONPATH=. python3 -m mtgtools get_card "Card Name" ...`.

## How to play: ONE command per decision
```
/home/claude/work/llm2/lg.sh show   TAG                 # prints your current prompt (waits while the other player is deciding)
/home/claude/work/llm2/lg.sh pick   TAG SEQ INDEX       # answers prompt SEQ with option INDEX, then waits and prints your next prompt
/home/claude/work/llm2/lg.sh odds   TAG SEQ N ["Card A;Card B;lands"]   # chance of seeing those cards in the top N of YOUR library
/home/claude/work/llm2/lg.sh opp    TAG SEQ ["Force of Will;Daze"]     # chance the OPPONENT holds those cards right now
/home/claude/work/llm2/lg.sh sim    TAG SEQ 'SCRIPT'                   # Monte Carlo what-if over your unknown cards (below)
/home/claude/work/llm2/lg.sh auto   TAG SEQ N LIFEMIN "rule;rule;..."   # macro for loops (below): answers this prompt and the next ones by label rules
/home/claude/work/llm2/lg.sh flag   TAG SEQ "why"                      # mark this decision as a tough spot (see below); does not answer it
```
- Start with `show`. The game is already running; the first prompt may take a moment if the other player moves first.
- SEQ is the number in `=== PROMPT n ===`; INDEX is the `[k]` of the option you choose. A wrong SEQ is rejected. The numbers jump between your prompts (the other player's decisions use numbers too): that is normal.
- **One pick per command, always.** Indices change after every action. Never chain picks or guess the next menu. `[0]` is usually Pass priority (or the first option): never `pick 0` just to look; use `show`.
- `pick` and `show` can take several minutes while the other player thinks. If the output says TIMEOUT, run `show` again. Never answer from memory.
- The engine skips windows where you could only pass, and on the opponent's turn it only stops for you when something is on the stack or at their end step. Passing with an empty stack moves the game on; with something on the stack, passing lets it resolve.
- Fetchlands: activate (costs 1 life), then choose the land. Mana is paid automatically when you cast a spell. Multi-card choices are asked one card at a time. If an option list shows a name once, it may stand for several copies.
- The prompt shows only what you are entitled to see: your hand, both battlefields and graveyards, the stack, opponent's hand size, cards you know, events since your last prompt, library counts.
- The game ends with `=== GAME OVER === YOU WON/YOU LOST`. Play to the end; do not concede.

## Tools (computed from your own view only)
**Odds** (`odds`, `opp`): exact hypergeometric arithmetic. `lg.sh odds TAG SEQ N "Show and Tell;Omniscience;lands"` = per term, the copies left in your library, the chance at least one is among the top N, the expected number, and "any of the above". Terms are card names (punctuation/case ignored), `lands`, `nonlands`, `creatures`, separated by `;`. Known top cards (put-backs, scry) count as drawn first. `lg.sh opp TAG SEQ "Force of Will;Daze"` = the chance the opponent holds one of them (uniform over their unseen cards; a floor for "could they have it").
**Sim** (`sim`): a Monte Carlo "what if" over your unknown library (20,000 shuffles consistent with what you know). Script of steps and goals, separated by `;`:
`lg.sh sim TAG SEQ 'brainstorm putback lands "Lotus Petal"; turn; goal "Aluren" and lands_playable >= 3'`
- Steps: `draw N` | `turn [N]` (draw N and gain N land drops) | `brainstorm putback "Card" lands ...` | `ponder want "Card" ... [shuffle]` | `look N take K want "Card" ... rest bottom|top|gy` | `shuffle` | `drops N` | `samples N` | `opp` (ask about the opponent's hand instead).
- Goals: `goal <expression>`: `"Card" >= 1`, `any("A","B") >= 2`, sums like `"Lotus Petal" + lands_playable >= 3`, `and`, `or`, `not`, parentheses. Specials: `lands`, `lands_bf`, `lands_playable`, `nonlands`, `creatures`, `hand_size`. Same seed in two scripts makes plays comparable.
- Not modeled: mana colours, Daze/Force/Wasteland, anything the opponent does.
Use them where a decision turns on a number (keep or mulligan, Brainstorm put-backs, "could they have Force or Daze"), roughly 5 to 15 calls a game, not on obvious decisions.

## Loop macro (`auto`): do not click a combo loop 600 times
When a free loop is running (Aluren or Omniscience with Acererak, or Aluren with a creature that gives value), use `auto` instead of one `pick` per cast.
`lg.sh auto TAG SEQ N LIFEMIN "rule;rule;..."` answers the current prompt SEQ and then up to N-1 more of your prompts with the FIRST rule that matches an option label (a rule is plain text found inside the label). Prefixes: `last:` picks the last matching option (the free Aluren cast is the later duplicate of "Cast X"), `stack:` makes the rule apply only while something is on the stack, `=` needs the whole label to match. It stops by itself (and shows you the prompt with `AUTO MACRO STOPPED ... why`) when no rule matches, N is used up, or your life is at or below LIFEMIN. The first rule list must match something on the current prompt or the command is refused.
Typical Lost Mine loop with Aluren in play, opponent tapped out: `lg.sh auto TAG SEQ 40 6 "last:Cast Acererak the Archlich;stack:Pass priority;Lost Mine;Cave Entrance;Goblin Lair;Dark Pool;Temple;Done"` (about 10 laps' worth of picks; look at the result and the opponent's life, then repeat). To finish with Tomb of Annihilation instead, put `Tomb of Annihilation` before `Lost Mine`.
Care: the macro passes priority while the opponent has a spell on the stack (the `stack:Pass priority` rule), so it cannot respond for you with Force of Will or Veil. Start it when the opponent is tapped out or cannot interact, or when you accept that risk, and keep N modest (20 to 60) so you re-check. Only the options named in your rules are ever picked.

## Flag your tough spots (for Brady, a Legacy expert who will review them)
Whenever you answer a prompt and were genuinely unsure (two or more options looked close, you guessed, or you suspect it matters a lot), run `lg.sh flag TAG SEQ "one line: the options you weighed and why it was close"` BEFORE the pick (flag first, then pick). Aim for 3 to 8 flags per game, only real ones: keep/mulligan, sequencing against Daze/Force/Wasteland, whether to jam or wait, Brainstorm/Stock Up choices. It costs almost nothing and does not touch the game.

## Rules of the exercise (important)
- Use ONLY `lg.sh` (including `flag`) and `get_card`, plus the deck files and playbook named above. **Do not read, list or search anything else**: not other directories under /home/claude/work, not any `runs/` directory (it holds the other player's view and the truth log), not engine source, logs or processes. Looking at anything else voids the game.
- Do not write notes to files, except your final report (below). Think as much as you need before each pick.
- At the end write your report to `/home/claude/work/llm2/reports/TAG.md` (TAG is your tag) and also give it as your final answer, in this exact format and nothing long: `RESULT: WON|LOST`, turn count, number of prompts, number of odds/opp/sim calls, number of flags, then up to 6 short bullets: your plan, key plays, mistakes you think you made (be honest, name the turn), what the opponent did that surprised you, anything confusing or that looked like an engine bug, and whether the tools changed a decision.

## Menu tips
- When Aluren is out and you also have mana for a hard cast, a creature can appear twice as `Cast X` (same label). The first entry is the hard cast (taps mana), the later one is the free Aluren cast. Check mana and life afterwards.
- Tomb of Annihilation rooms ask "Do it?": Yes means pay the cost to avoid the life loss, No means take it. Scry/surveil: `Choose <card>` sends it to the bottom/graveyard; `Done` keeps it on top. Atraxa's reveal keeps listing types you already picked: re-read the menu each pick.
