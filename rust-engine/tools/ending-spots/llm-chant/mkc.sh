#!/bin/bash
# mkc.sh <id> <matchup ur|ub|uw> <arm c3e1|c4e0|base> <first A|B>  -> preps post-board game 2, launches the engine, writes tasks/<id>g2a.txt and tasks/<id>g2b.txt
cd /home/claude/work/llm2
ID=$1; M=$2; ARM=$3; FIRST=$4; G=2
case $M in ur) OPPD=ur-cutter; ON="ur-cutter (UR Cutter, Legacy)";; ub) OPPD=dimir-tempo; ON="dimir-tempo (UB Dimir Tempo, Legacy)";; uw) OPPD=uwx-control; ON="uwx-control (UW Phelia control, Legacy)";; esac
export OPP=$OPPD PLANS_A=/home/claude/sbexp/plans/$M/$ARM PLANS_B=/home/claude/sbexp/oppplans BASE_A=/home/claude/work/llm2/decks-base
export LLM_BIN=/home/claude/work/llm2/bin/llmgame-live
python3 match.py prep $ID $G $FIRST > /tmp/mkc-$ID.txt || exit 1
D=matches/$ID/g$G/engine-decks
cp decks-base/alurentell.txt $D/alurentell-base.txt; cp decks-base/$OPPD.txt $D/$OPPD-base.txt
export LLM_EXTRA="--expect-opp ${OPPD}-base --expect-llm alurentell-base"
eval "$(head -1 /tmp/mkc-$ID.txt)"
W=/home/claude/work/llm2; BR=$W/BRIEF-post.md; BA=$W/decks-base
if [ $FIRST = A ]; then FA="You play FIRST (on the play)."; FB="The opponent plays FIRST; you are on the draw."; else FA="The opponent plays FIRST; you are on the draw."; FB="You play FIRST (on the play)."; fi
cat > tasks/${ID}g${G}a.txt <<T
You are a player in a Magic: The Gathering game. Do not call any mcp__hearthbot__ tools. First read $BR completely and follow it exactly (it explains how to play with lg.sh, the rules of the exercise and the report you must write).
YOUR DECK: Alurentell. OPPONENT'S DECK: $ON. YOUR TAG: ${ID}g${G}a.
Post-sideboard game. $FA Your 60-card game deck (main, blank line, sideboard): $W/matches/$ID/g${G}/a-decklist.txt. Your base 75: $BA/alurentell.txt. The opponent's base 75: $BA/$OPPD.txt.
Note from Brady: this match tests how many Orim's Chant and Prismatic Ending to bring in against blue tempo decks. Your boarded-in Chant and Ending are the plan, so use them as you judge best; the report asks what you thought of them.
Start with: $W/lg.sh show ${ID}g${G}a . Play to the end, then write $W/reports/${ID}g${G}a.md as the brief says and give the same text as your final answer.
T
cat > tasks/${ID}g${G}b.txt <<T
You are a player in a Magic: The Gathering game. Do not call any mcp__hearthbot__ tools. First read $BR completely and follow it exactly (it explains how to play with lg.sh, the rules of the exercise and the report you must write).
YOUR DECK: $ON. OPPONENT'S DECK: Alurentell. YOUR TAG: ${ID}g${G}b.
Post-sideboard game. $FB Your 60-card game deck (main, blank line, sideboard): $W/matches/$ID/g${G}/b-decklist.txt. Your base 75: $BA/$OPPD.txt. The opponent's base 75 is $BA/alurentell.txt (it has sideboarded; you do not know how).
You have no playbook: use your own Legacy knowledge. Start with: $W/lg.sh show ${ID}g${G}b . Play to the end, then write $W/reports/${ID}g${G}b.md as the brief says (CHANT_ENDING: n/a) and give the same text as your final answer.
T
echo "ready ${ID}: $M $ARM first=$FIRST"; tail -1 /tmp/mkc-$ID.txt
