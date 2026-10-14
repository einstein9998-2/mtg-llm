#!/bin/bash
# mkx.sh <id> <matchup ur|ub|uw> <exp archon|plains|arch61> <first A|B>
#   archon: game 1 (unboarded), Alurentell main has Archon of Cruelty instead of one Ponder
#   plains: game 2 (boarded with the base plan), Alurentell base 75 has Plains instead of Island
cd /home/claude/work/llm2
ID=$1; M=$2; EXP=$3; FIRST=$4
case $M in ur) OPPD=ur-cutter; ON="ur-cutter (UR Cutter, Legacy)";; ub) OPPD=dimir-tempo; ON="dimir-tempo (UB Dimir Tempo, Legacy)";; uw) OPPD=uwx-control; ON="uwx-control (UW Phelia control, Legacy)";; esac
case $EXP in archon) G=1; BA=/home/claude/work/llm2/decks-base-arch; BR=/home/claude/work/llm2/BRIEF-g1-archon.md;;
  arch61) G=1; BA=/home/claude/work/llm2/decks-base-61; BR=/home/claude/work/llm2/BRIEF-g1-arch61.md; export MAIN_N=61;; plains) G=2; BA=/home/claude/work/llm2/decks-base-plains; BR=/home/claude/work/llm2/BRIEF-post-plains.md;; esac
W=/home/claude/work/llm2; BO=$W/decks-base
export OPP=$OPPD PLANS_A=/home/claude/sbexp/plans/$M/base PLANS_B=/home/claude/sbexp/oppplans BASE_A=$BA
export LLM_BIN=$W/bin/llmgame-live
python3 match.py prep $ID $G $FIRST > /tmp/mkx-$ID.txt || { cat /tmp/mkx-$ID.txt; exit 1; }
export LLM_EXTRA="--pub0 $BA/alurentell.txt --pub1 $BO/$OPPD.txt --no-consult"
eval "$(head -1 /tmp/mkx-$ID.txt)"
if [ $FIRST = A ]; then FA="You play FIRST (on the play)."; FB="The opponent plays FIRST; you are on the draw."; else FA="The opponent plays FIRST; you are on the draw."; FB="You play FIRST (on the play)."; fi
if [ $EXP = arch61 ]; then KIND="Game ONE (unboarded, main decks)."; NOTEA="Your deck is the usual 60 plus a 61st card, one Archon of Cruelty (61 cards in your game deck): this is the thing being tested, so read its card text with get_card and decide for yourself how and when to use it (it is not castable for its mana cost in practice; Show and Tell and Omniscience can put it in). Note: Show and Tell lets each player put a land onto the battlefield too, so land choices in its menu are correct, not a bug."; OPPBASE="unknown to you at game one"; elif [ $G = 1 ]; then KIND="Game ONE (unboarded, main decks)."; NOTEA="Your deck has one Archon of Cruelty in the main deck in place of one Ponder: this is the thing being tested, so read its card text with get_card and decide for yourself how and when to use it (it is not castable for its mana cost in practice; Show and Tell and Omniscience can put it in)."; OPPBASE="unknown to you at game one"; else KIND="Post-sideboard game (both sides have boarded with their usual plan)."; NOTEA="Your base 75 has one Plains in place of the Island in the usual list: this is the thing being tested (more white access for Orim's Chant and Prismatic Ending). The boarded-in cards are the plan; use them as you judge best."; OPPBASE="$BA/alurentell.txt (it has sideboarded; you do not know how)"; fi
cat > tasks/${ID}g${G}a.txt <<T
You are a player in a Magic: The Gathering game. Do not call any mcp__hearthbot__ tools. First read $BR completely and follow it exactly (it explains how to play with lg.sh, the rules of the exercise and the report you must write). Where the brief names a base-75 location, the paths named in this message override it.
YOUR DECK: Alurentell. OPPONENT'S DECK: $ON. YOUR TAG: ${ID}g${G}a.
$KIND $FA Your 60-card game deck (main, blank line, sideboard): $W/matches/$ID/g${G}/a-decklist.txt. Your base 75: $BA/alurentell.txt. The opponent's base 75: $BO/$OPPD.txt.
$NOTEA
Start with: $W/lg.sh show ${ID}g${G}a . Play to the end, then write $W/reports/${ID}g${G}a.md as the brief says and give the same text as your final answer.
T
cat > tasks/${ID}g${G}b.txt <<T
You are a player in a Magic: The Gathering game. Do not call any mcp__hearthbot__ tools. First read $BR completely and follow it exactly (it explains how to play with lg.sh, the rules of the exercise and the report you must write). Where the brief names a base-75 location, the paths named in this message override it.
YOUR DECK: $ON. OPPONENT'S DECK: Alurentell. YOUR TAG: ${ID}g${G}b.
$KIND $FB Your 60-card game deck (main, blank line, sideboard): $W/matches/$ID/g${G}/b-decklist.txt. Your base 75: $BO/$OPPD.txt. The opponent's base 75 is $OPPBASE.
You have no playbook: use your own Legacy knowledge. Start with: $W/lg.sh show ${ID}g${G}b . Play to the end, then write $W/reports/${ID}g${G}b.md as the brief says (write 'n/a' on the Alurentell-only line) and give the same text as your final answer.
T
echo "ready ${ID}: $M $EXP first=$FIRST game=$G"
