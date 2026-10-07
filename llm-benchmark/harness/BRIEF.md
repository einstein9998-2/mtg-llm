# You are the player in a Magic: The Gathering game (Legacy, 1v1, closed card pool)

You play **Alurentell** (Aluren + Show and Tell + Omniscience combo) against **UR Cutter** (tempo: Dragon's Rage Channeler,
Murktide Regent, Cori-Steel Cutter, Daze, Force of Will, Lightning Bolt, Wasteland, Unholy Heat, Prismari Charm).
Both 75-card lists are in /mnt/project-files/decks/alurentell.txt and /mnt/project-files/decks/ur-cutter.txt (main deck, blank line, sideboard; no sideboarding in this game). Your strategy notes are in
/mnt/project-files/llm-player/alurentell-playbook.md (written for a different engine; the strategy is right, the menu/prompt details are not).
Card text: `cd /mnt/project-files/llm-tools && PYTHONPATH=. python3 -m mtgtools get_card "Card Name" ...` (works for every card in both decks).

## How to play: ONE command per decision
```
/home/claude/work/llm/lg.sh start  TAG SEED SEAT 0     # starts the game, prints prompt 1
/home/claude/work/llm/lg.sh pick   TAG SEQ INDEX       # answers prompt SEQ with option INDEX, prints the next prompt
/home/claude/work/llm/lg.sh show   TAG                 # re-prints the current prompt without answering
```
- SEQ is the number in `=== PROMPT n ===` of the prompt you are answering; INDEX is the `[k]` of the option you choose. A wrong SEQ is rejected, so a stale answer cannot slip through.
- **One pick per command, always.** Indices change after every action. Never chain picks in a loop or guess the next menu. `[0]` is always Pass priority (or the first option of non-priority decisions): never use pick 0 just to look; use `show`.
- The engine skips windows where you could only pass, and on the opponent's turn it only stops for you when something is on the stack or at their end step. Priority passes with an empty stack move the game on; with something on the stack, passing lets it resolve (the opponent gets priority first, then it resolves).
- Fetchlands: activate (costs 1 life), then choose the land. Mana is paid automatically when you cast a spell (Ancient Tomb deals 2 damage each time it is tapped; Lotus Petal is sacrificed). Force of Will appears in the menu when you have a blue card to exile.
- Multi-card choices are asked one card at a time (Ponder: order the three cards top first, then "Do it?" = shuffle yes/no; Stock Up: pick two, then order the rest to the bottom; Atraxa: pick one card per type, then Done; Show and Tell: pick the card to put in). If the option list shows a name once, it may stand for several copies.
- The prompt shows only what you are entitled to see: your hand, both battlefields and graveyards, the stack, opponent's hand size, cards you know, and the events since your last prompt. Library counts are shown. You never see the opponent's hand or either library order beyond what you have revealed.
- The game ends with `=== GAME OVER === YOU WON/YOU LOST`. Play to the end, make your best decisions, do not concede.

## Rules of the exercise (important)
- Use ONLY the three `lg.sh` commands for the game and `get_card` for card text, plus the files named above. **Do not read, list or search anything else**: not the other directories under /home/claude/work, not /home/claude/work/llm/runs/*/truth, not engine source, logs or processes. The point is to measure what you can do from the prompts alone; looking at anything else would make the result meaningless.
- Do not write notes to files. Think as much as you need before each pick, but keep tool use to `lg.sh`.
- At the end report in this exact format and nothing long: `RESULT: WON|LOST|DRAW`, turn count, number of prompts, then up to 5 short bullets: key plays, mistakes you think you made (be honest), anything about the prompts or menus that was confusing or looked like an engine bug.

## Menu tip from earlier games
When Aluren is out and you also have mana for a hard cast, a creature can appear twice as `Cast X` (the same label). Earlier players found the first entry is the hard cast (taps mana, Ancient Tomb damage) and the later one is the free Aluren cast. Check your mana pool and life after casting to confirm which you got.
- Tomb of Annihilation rooms (Veils of Fear, Sandfall Cell) ask "Do it?": Yes means pay the cost to avoid the life loss (discard a card / sacrifice a permanent), No means take the life loss. A Lotus Petal only pays for Daze once it is on the battlefield.
- Scry/surveil prompts: `Choose <card>` sends that card to the bottom (scry) or graveyard (surveil); `Done` keeps it on top. Atraxa's reveal keeps listing types you already picked, so re-read the menu before every pick.
