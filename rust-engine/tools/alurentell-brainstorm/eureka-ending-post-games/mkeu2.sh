#!/bin/bash
# mkeu.sh <id> <matchup ur|ub|uw> <g1|post> <first A|B>
#   g1:   game 1, main deck with 2 Eureka replacing the 2 main Veil of Summer (4 Ponder stay) (decks-base-eur2)
#   post: post-board game 2, the usual plan with 2 Eureka boarded in instead of the 2 Prismatic Ending (plans/<m>/eur3 on decks-base-eur3); Veil is still 4 (2 main + 2 boarded in)
cd /home/claude/work/llm2
ID=$1; M=$2; KIND=$3; FIRST=$4
case $M in ur) OPPD=ur-cutter; ON="ur-cutter (UR Cutter, Legacy)";; ub) OPPD=dimir-tempo; ON="dimir-tempo (UB Dimir Tempo, Legacy)";; uw) OPPD=uwx-control; ON="uwx-control (UW Phelia control, Legacy)";; esac
W=/home/claude/work/llm2; BO=$W/decks-base
case $KIND in g1) G=1; BR=$W/BRIEF-g1-eur.md; BA=$W/decks-base-eur2; PL=base;; post) G=2; BR=$W/BRIEF-post-eur.md; BA=$W/decks-base-eur3; PL=eur3;; esac
export OPP=$OPPD PLANS_A=/home/claude/sbexp/plans/$M/$PL PLANS_B=/home/claude/sbexp/oppplans BASE_A=$BA
export LLM_BIN=$W/bin/llmgame-eureka MAIN_N=60
python3 match.py prep $ID $G $FIRST > /tmp/mkeu-$ID.txt || { cat /tmp/mkeu-$ID.txt; exit 1; }
export LLM_EXTRA="--pub0 $BA/alurentell.txt --pub1 $BO/$OPPD.txt --no-consult"
eval "$(head -1 /tmp/mkeu-$ID.txt)"
if [ $FIRST = A ]; then FA="You play FIRST (on the play)."; FB="The opponent plays FIRST; you are on the draw."; else FA="The opponent plays FIRST; you are on the draw."; FB="You play FIRST (on the play)."; fi
if [ $G = 1 ]; then KINDT="Game ONE (unboarded, main decks)."; OPPBASE="unknown to you at game one"; else KINDT="Post-sideboard game (both sides have boarded with their usual plan)."; OPPBASE="$BA/alurentell.txt (it has sideboarded; you do not know how)"; fi
if [ $G = 1 ]; then REPL="your deck has 2 Eureka in place of the 2 Veil of Summer in the main (you have 4 Ponder; your sideboard still has 2 Veil)."; else REPL="your sideboard plan boards in 2 Eureka (in place of the 2 Prismatic Ending the usual plan brings in; you still have 4 Veil of Summer and 4 Ponder), so you have 2 Eureka in the 60."; fi
EUR="Eureka is not in the get_card pool; its text is: Eureka {2}{G}{G}, Sorcery: Starting with you, each player may put a permanent card from their hand onto the battlefield. Repeat this process until no one puts a card onto the battlefield. In the menu it asks the caster for one card at a time (Done to decline), then the other player, round after round; every card enters at once and triggers wait until Eureka has finished resolving. Both players get the chance, so the opponent also puts in permanents (lands, creatures) each round."
cat > tasks/${ID}g${G}a.txt <<T
You are a player in a Magic: The Gathering game. Do not call any mcp__hearthbot__ tools. First read $BR completely and follow it exactly (it explains how to play with lg.sh, the rules of the exercise and the report you must write). Where the brief names a base-75 location, the paths named in this message override it.
YOUR DECK: Alurentell. OPPONENT'S DECK: $ON. YOUR TAG: ${ID}g${G}a.
$KINDT $FA Your 60-card game deck (main, blank line, sideboard): $W/matches/$ID/g${G}/a-decklist.txt. Your base 75: $BA/alurentell.txt. The opponent's base 75: $BO/$OPPD.txt.
This is the thing being tested: $REPL $EUR Decide for yourself how and when to use it (it is a sorcery; you can hard cast it with GG, or cheat it with Aluren/Omniscience free casts where legal). Note: Show and Tell also lets each player put a land onto the battlefield, so land choices in its menu are correct, not a bug.
Start with: $W/lg.sh show ${ID}g${G}a . Play to the end, then write $W/reports/${ID}g${G}a.md as the brief says and give the same text as your final answer.
T
cat > tasks/${ID}g${G}b.txt <<T
You are a player in a Magic: The Gathering game. Do not call any mcp__hearthbot__ tools. First read $BR completely and follow it exactly (it explains how to play with lg.sh, the rules of the exercise and the report you must write). Where the brief names a base-75 location, the paths named in this message override it.
YOUR DECK: $ON. OPPONENT'S DECK: Alurentell. YOUR TAG: ${ID}g${G}b.
$KINDT $FB Your 60-card game deck (main, blank line, sideboard): $W/matches/$ID/g${G}/b-decklist.txt. Your base 75: $BO/$OPPD.txt. The opponent's base 75 is $OPPBASE.
Alurentell may have Eureka in its deck. $EUR
You have no playbook: use your own Legacy knowledge. Start with: $W/lg.sh show ${ID}g${G}b . Play to the end, then write $W/reports/${ID}g${G}b.md as the brief says (write 'n/a' on the Eureka line if it was not cast) and give the same text as your final answer.
T
echo "ready ${ID}: $M $KIND first=$FIRST"
