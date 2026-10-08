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
When a free loop is running (Aluren or Omniscience with Acererak), use `auto` instead of one `pick` per cast.
`lg.sh auto TAG SEQ N LIFEMIN "rule;rule;..."` answers the current prompt SEQ and then up to N-1 more of your prompts with the FIRST rule that matches an option label (a rule is plain text found inside the label). Conditions go in front of the text, each ending in `:`, in any order: `last:` picks the last matching option (the free Aluren cast is the later duplicate of "Cast X"); `stack:` only while something is on the stack; `empty:` only while the stack is empty; `mytop:` only while the top stack item is yours; `top[TEXT]:` only while the top stack item's name contains TEXT (spells and triggers both show the card name); `after[TEXT]:` only if the macro's previous pick had a label containing TEXT; `=` needs the whole label to match. It stops by itself (and shows you the prompt with `AUTO MACRO STOPPED ... why`) when no rule matches, N is used up, or your life is at or below LIFEMIN. The first rule list must match something on the current prompt or the command is refused. Any item of the OPPONENT on top of the stack makes the `mytop:` rules stop, so the macro hands the prompt back to you when they respond.

**The correct Lost Mine technique with Aluren in play (Brady):** Aluren lets you cast Acererak at instant speed, so do NOT resolve the room triggers. Leave all of them on the stack, except the Dark Pool drain. Each Acererak still has to enter and bounce (resolve his own trigger) before you cast him again, otherwise two Acererak meet and the legend rule kills one. Because the Temple of Dumathoin draw stays unresolved, you draw nothing and there is no deck-out risk; the whole pile resolves later only if you let it.
Macro for it: `lg.sh auto TAG SEQ 60 6 "mytop:top[Acererak]:Pass priority;after[Dark Pool]:mytop:stack:Pass priority;last:mytop:Cast Acererak the Archlich;last:empty:Cast Acererak the Archlich;Lost Mine;Cave Entrance;Goblin Lair;Dark Pool;Temple;Done"`
Read it as: pass to resolve Acererak's own spell and enter trigger; right after you chose Dark Pool, pass once to resolve that drain (it is on top); otherwise cast Acererak again; and answer dungeon and room prompts Lost Mine, Goblin Lair, Dark Pool. One lap is eight picks and drains 1 (about 8 picks per point of the opponent's life), so a kill from 20 takes about 160 picks: launch chunks of 200 and check the opponent's life each time. Never put `Tomb of Annihilation` in the rules unless you want to complete it (then Acererak stays).
**Under Omniscience without Aluren** the triggers must resolve (Brady: you will eventually draw Aluren). Use the old form: `"stack:Pass priority;last:Cast Acererak the Archlich;Lost Mine;Cave Entrance;Goblin Lair;Dark Pool;Temple;Done"`. A lap is then four casts, one drain and one draw (Temple), so check that the opponent's life is at most your library size + 1 before you start.
Care: the macro cannot answer for you with Force of Will or Veil: it stops when the opponent puts anything on the stack, and you then decide by hand and may restart it. Start it when the opponent is tapped out or cannot interact, or when you accept that risk. Only the options named in your rules are ever picked.

## Flag your tough spots (for Brady, a Legacy expert who will review them)
Whenever you answer a prompt and were genuinely unsure (two or more options looked close, you guessed, or you suspect it matters a lot), run `lg.sh flag TAG SEQ "one line: the options you weighed and why it was close"` BEFORE the pick (flag first, then pick). Aim for 3 to 8 flags per game, only real ones: keep/mulligan, sequencing against Daze/Force/Wasteland, whether to jam or wait, Brainstorm/Stock Up choices. It costs almost nothing and does not touch the game.

## Rules of the exercise (important)
- Use ONLY `lg.sh` (including `flag`) and `get_card`, plus the deck files and playbook named above. **Do not read, list or search anything else**: not other directories under /home/claude/work, not any `runs/` directory (it holds the other player's view and the truth log), not engine source, logs or processes. Looking at anything else voids the game.
- Do not write notes to files, except your final report (below). Think as much as you need before each pick.
- At the end write your report to `/home/claude/work/llm2/reports/TAG.md` (TAG is your tag) and also give it as your final answer, in this exact format and nothing long: `RESULT: WON|LOST`, turn count, number of prompts, number of odds/opp/sim calls, number of flags, then up to 6 short bullets: your plan, key plays, mistakes you think you made (be honest, name the turn), what the opponent did that surprised you, anything confusing or that looked like an engine bug, and whether the tools changed a decision.

## Menu tips
- When Aluren is out and you also have mana for a hard cast, a creature can appear twice as `Cast X` (same label). The first entry is the hard cast (taps mana), the later one is the free Aluren cast. Check mana and life afterwards.
- Tomb of Annihilation rooms ask "Do it?": Yes means pay the cost to avoid the life loss, No means take it. Scry/surveil: `Choose <card>` sends it to the bottom/graveyard; `Done` keeps it on top. Atraxa's reveal keeps listing types you already picked: re-read the menu each pick.

## Costs players have misread (check with `get_card` if unsure)
- Show and Tell is {2}{U} (one blue source plus two other mana, e.g. Ancient Tomb counts as two). Aluren is {2}{G}{G} (two green sources). A lap of the Lost Mine loop is FOUR Acererak casts per Dark Pool drain (Cave Entrance, Goblin Lair, Dark Pool, Temple of Dumathoin).
- Macro budgets: with Aluren about 8 picks per point of life (see the macro section); N of 60 per chunk.

## Shared-machine warning
Other players run on this machine at the same time and share /tmp. Never write helper scripts or output files into /tmp or any shared path (they get overwritten by others and can show another game's prompts). Use only `lg.sh` directly, always with your own TAG and the SEQ you just read.
Use ONLY your own TAG in every lg.sh command. A command with the other player's tag (for example `show` of their game) prints THEIR hidden hand and voids the game. If a command fails, re-read the error and retry with your own tag only.
- Decking check: only the resolve-everything line (Omniscience without Aluren) draws a card per lap; the Aluren technique above does not.
- Daze (their free counter) needs any land with the Island type, tapped or untapped (Volcanic Island, Thundering Falls, Underground Sea, Tropical Island). Only a board with no Island-typed land makes Daze impossible.
- Planeswalker abilities are labelled only "ability 1, 2, 3" in printed order, with no loyalty cost shown and index 0 never used for an ability. Check `get_card` for the printed order and which one is the 0 or minus ability before picking (Ajani, Nacatl Avenger: 1 = +2, 2 = 0).
