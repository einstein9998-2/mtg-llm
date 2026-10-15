#!/bin/bash
# mkpe.sh <id> <matchup ur|ub|uw> <first A|B>
#   post-board game 2, Brady's list: 2 Lotus Petal, 3 Orim's Chant, 0 Prismatic Ending, 3 Aluren (plans/<m>/pe2: Boseiju or Island trimmed instead of Ponder)
cd /home/claude/work/llm2
ID=$1; M=$2; FIRST=$3; G=2; ARM=pe2
case $M in ur) OPPD=ur-cutter; ON="ur-cutter (UR Cutter, Legacy)";; ub) OPPD=dimir-tempo; ON="dimir-tempo (UB Dimir Tempo, Legacy)";; uw) OPPD=uwx-control; ON="uwx-control (UW Phelia control, Legacy)";; esac
W=/home/claude/work/llm2; BO=$W/decks-base; BA=$W/decks-base-c3; BR=$W/BRIEF-post-pe.md
export OPP=$OPPD PLANS_A=/home/claude/sbexp/plans/$M/$ARM PLANS_B=/home/claude/sbexp/oppplans BASE_A=$BA
export LLM_BIN=$W/bin/llmgame-live
python3 match.py prep $ID $G $FIRST > /tmp/mkpe-$ID.txt || { cat /tmp/mkpe-$ID.txt; exit 1; }
export LLM_EXTRA="--pub0 $BA/alurentell.txt --pub1 $BO/$OPPD.txt --no-consult"
eval "$(head -1 /tmp/mkpe-$ID.txt)"
if [ $FIRST = A ]; then FA="You play FIRST (on the play)."; FB="The opponent plays FIRST; you are on the draw."; else FA="The opponent plays FIRST; you are on the draw."; FB="You play FIRST (on the play)."; fi
cat > tasks/${ID}g${G}a.txt <<T
You are a player in a Magic: The Gathering game. Do not call any mcp__hearthbot__ tools. First read $BR completely and follow it exactly (it explains how to play with lg.sh, the rules of the exercise and the report you must write). Where the brief names a base-75 location, the paths named in this message override it.
YOUR DECK: Alurentell. OPPONENT'S DECK: $ON. YOUR TAG: ${ID}g${G}a.
Post-sideboard game. $FA Your 60-card game deck (main, blank line, sideboard): $W/matches/$ID/g${G}/a-decklist.txt. Your base 75: $BA/alurentell.txt. The opponent's base 75: $BO/$OPPD.txt.
This is the thing being tested: your boarded list has 2 Lotus Petal, 3 Orim's Chant, 0 Prismatic Ending and 3 Aluren (the usual plan has no Petal, 2 Chant, 2 Ending). Room for the extra cards came from Boseiju and Island (and one Ponder against UW) instead of Ponder, so you keep 4 Ponder against UR and UB and 3 against UW. Use the boarded-in cards as you judge best; the report asks what you thought of the list. Note: Show and Tell lets each player put a land onto the battlefield too, so land choices in its menu are correct, not a bug.
Start with: $W/lg.sh show ${ID}g${G}a . Play to the end, then write $W/reports/${ID}g${G}a.md as the brief says and give the same text as your final answer.
T
cat > tasks/${ID}g${G}b.txt <<T
You are a player in a Magic: The Gathering game. Do not call any mcp__hearthbot__ tools. First read $BR completely and follow it exactly (it explains how to play with lg.sh, the rules of the exercise and the report you must write). Where the brief names a base-75 location, the paths named in this message override it.
YOUR DECK: $ON. OPPONENT'S DECK: Alurentell. YOUR TAG: ${ID}g${G}b.
Post-sideboard game. $FB Your 60-card game deck (main, blank line, sideboard): $W/matches/$ID/g${G}/b-decklist.txt. Your base 75: $BO/$OPPD.txt. The opponent's base 75 is $BA/alurentell.txt (it has sideboarded; you do not know how).
You have no playbook: use your own Legacy knowledge. Start with: $W/lg.sh show ${ID}g${G}b . Play to the end, then write $W/reports/${ID}g${G}b.md as the brief says (write 'n/a' on the Alurentell-only line) and give the same text as your final answer.
T
echo "ready ${ID}: $M $ARM first=$FIRST"
