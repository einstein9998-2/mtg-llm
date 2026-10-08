#!/usr/bin/env python3
"""matchprompt.py MID K A|B [--reasons]: print the task message for one player of game K of a match."""
import json, sys
W = '/home/claude/work/llm2'
mid, k, side = sys.argv[1], int(sys.argv[2]), sys.argv[3]
reasons = '--reasons' in sys.argv
st = json.load(open(f'{W}/matches.json'))[mid]
g = st['games'][k - 1]
a = sum(1 for x in st['games'][:k-1] if x['result'] == 'A'); b = sum(1 for x in st['games'][:k-1] if x['result'] == 'B')
opp = st['opp']
tag = f'{g["id"]}{side.lower()}'
deck = 'Alurentell' if side == 'A' else opp
other = opp if side == 'A' else 'Alurentell'
mine_score, their_score = (a, b) if side == 'A' else (b, a)
first = 'You play first.' if g['first'] == side else 'The opponent plays first.'
notes = ('Your strategy notes (read the playbook fully, then the skill files that fit your situation as it comes up; ALWAYS read skills/sideboarding.md before writing your sideboard plan): '
         '/home/claude/work/llm2/restructure-v2/playbook.md and /home/claude/work/llm2/restructure-v2/skills/*.md. Ignore the older alurentell-playbook.md named in BRIEF-vs.md.') if side == 'A' else \
        (f'You have no strategy notes: use your own Legacy knowledge of {opp}. Sideboard suggestions: /home/claude/work/live/engine/sideboard-plans/{opp}.txt.')
r = ('\nDEMO MODE: Brady will read a play-by-play of this game with your reasons. Every `pick` must carry a reason as a 4th argument: `lg.sh pick TAG SEQ INDEX "why"`, one or two sentences naming the options you weighed and why this one (the mulligan, land drops, every spell, every attack, blocks, targets, Force/Daze/Veil decisions, what you put back or discard). Passing priority with nothing to do and forced single options need no reason. For `auto` give the reason as an extra 6th argument after the rules. Be honest about uncertainty; do not invent certainty.\n' if reasons else '')
print(f'''Read /home/claude/work/llm2/BRIEF-vs.md and then /home/claude/work/llm2/BRIEF-match.md (the second changes the first). You are a player in a best-of-three match.
MID: {mid}. Game {k} of the match. Your side: {side} ({deck}). Your opponent: {other}. Your TAG: {tag}.
Score before this game: you {mine_score}, opponent {their_score}. {first}
Your list for this game: /home/claude/work/llm2/match/{mid}/g{k}/decks/{side.lower()}.txt (base 75: /home/claude/work/llm2/decks/{deck.lower() if side=='A' else opp}.txt).
{('Your memo from the previous game: /home/claude/work/llm2/match/'+mid+'/memo'+side+'.md. ') if k>1 else ''}{notes}
Your memo and sideboard files after the game: memo{side}.md and board{side}.txt in /home/claude/work/llm2/match/{mid}/ (see BRIEF-match.md). Use ONLY your own side's files.{r}
Do not call any mcp__hearthbot__ tools. Start with `/home/claude/work/llm2/lg.sh show {tag}`.''')
