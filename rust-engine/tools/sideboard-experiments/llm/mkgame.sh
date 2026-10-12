#!/bin/bash
# mkgame.sh <id> <matchup> <plan> <first A|B>: preps + launches the engine and writes the two player task files (tasks/<id>g2a.txt, tasks/<id>g2b.txt)
cd /home/claude/work/llm2
ID=$1; M=$2; ARM=$3; FIRST=$4
./mk.sh $ID $M $ARM $FIRST > /tmp/mkgame-$ID.out || { cat /tmp/mkgame-$ID.out; exit 1; }
case $M in storm) OD=storm; ON="storm (Beseech Storm, Legacy)"; BD=storm;; ur) OD=ur-cutter; ON="ur-cutter (UR Cutter, Legacy)"; BD=ur-cutter;; mirror) OD=alurentell-b; ON="Alurentell"; BD=alurentell;; tron) OD=colorless-tron; ON="colorless-tron (Colorless Tron, Legacy)"; BD=colorless-tron;; esac
if [ $FIRST = A ]; then FA="You play FIRST (on the play)."; FB="The opponent plays FIRST; you are on the draw."; else FA="The opponent plays FIRST; you are on the draw."; FB="You play FIRST (on the play)."; fi
W=/home/claude/work/llm2
BA=${BASE_A:-$W/decks-base}
cat > tasks/${ID}g2a.txt <<T
You are a player in a Magic: The Gathering game. Do not call any mcp__hearthbot__ tools. First read $W/BRIEF-wave.md completely and follow it exactly (it explains how to play with lg.sh, the rules of the exercise and the report you must write).
YOUR DECK: Alurentell. OPPONENT'S DECK: $ON. YOUR TAG: ${ID}g2a.
Post-sideboard game. $FA Your 60-card game deck (main, blank line, sideboard): $W/matches/$ID/g2/a-decklist.txt. Your base 75: $BA/alurentell.txt. The opponent's base 75: $W/decks-base/$BD.txt.
Start with: $W/lg.sh show ${ID}g2a . Play to the end, then write $W/reports/${ID}g2a.md as the brief says and give the same text as your final answer.
T
if [ $M = mirror ]; then
cat > tasks/${ID}g2b.txt <<T
You are a player in a Magic: The Gathering game. Do not call any mcp__hearthbot__ tools. First read $W/BRIEF-wave.md completely and follow it exactly (it explains how to play with lg.sh, the rules of the exercise and the report you must write).
YOUR DECK: Alurentell (the mirror). OPPONENT'S DECK: Alurentell. YOUR TAG: ${ID}g2b.
Post-sideboard game. $FB Your 60-card game deck (main, blank line, sideboard): $W/matches/$ID/g2/b-decklist.txt. Your base 75: $W/decks-base/alurentell-b.txt (same list as the opponent's base 75: $W/decks-base/alurentell.txt).
Start with: $W/lg.sh show ${ID}g2b . Play to the end, then write $W/reports/${ID}g2b.md as the brief says and give the same text as your final answer.
T
else
cat > tasks/${ID}g2b.txt <<T
You are a player in a Magic: The Gathering game. Do not call any mcp__hearthbot__ tools. First read $W/BRIEF-wave.md completely and follow it exactly (it explains how to play with lg.sh, the rules of the exercise and the report you must write).
YOUR DECK: $ON. OPPONENT'S DECK: Alurentell. YOUR TAG: ${ID}g2b.
Post-sideboard game. $FB Your 60-card game deck (main, blank line, sideboard): $W/matches/$ID/g2/b-decklist.txt. Your base 75: $W/decks-base/$BD.txt. The opponent's base 75: $W/decks-base/alurentell.txt.
You have no playbook: use your own Legacy knowledge. Start with: $W/lg.sh show ${ID}g2b . Play to the end, then write $W/reports/${ID}g2b.md as the brief says and give the same text as your final answer.
T
fi
echo "ready ${ID}: $M $ARM first=$FIRST"
