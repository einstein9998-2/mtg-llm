#!/bin/bash
# mkgame2.sh <id> <matchup: storm|ur|tron> <plan> <first A|B> <game 1|2>  -> tasks/<id>g<G>a.txt (Alurentell with Emrakul) and tasks/<id>g<G>b.txt (opponent)
cd /home/claude/work/llm2
ID=$1; M=$2; ARM=$3; FIRST=$4; G=$5
./mk2.sh $ID $M $ARM $FIRST $G > /tmp/mkgame2-$ID.out || { cat /tmp/mkgame2-$ID.out; exit 1; }
case $M in storm) ON="storm (Beseech Storm, Legacy)"; BD=storm;; ur) ON="ur-cutter (UR Cutter, Legacy)"; BD=ur-cutter;; tron) ON="colorless-tron (Colorless Tron, Legacy)"; BD=colorless-tron;; esac
if [ $FIRST = A ]; then FA="You play FIRST (on the play)."; FB="The opponent plays FIRST; you are on the draw."; else FA="The opponent plays FIRST; you are on the draw."; FB="You play FIRST (on the play)."; fi
W=/home/claude/work/llm2; BA=$W/decks-base-emr
if [ $G = 1 ]; then BR=$W/BRIEF-g1.md; KIND="Game ONE (unboarded, main decks)."; else BR=$W/BRIEF-post-emr.md; KIND="Post-sideboard game."; fi
NOTE="Your deck has one Emrakul, the Aeons Torn, in the main deck in place of a Ponder: this is the thing being tested, so read its card text with get_card and decide for yourself how and when to use it. Brady's note: it does get cast for free when Omniscience is in play."
cat > tasks/${ID}g${G}a.txt <<T
You are a player in a Magic: The Gathering game. Do not call any mcp__hearthbot__ tools. First read $BR completely and follow it exactly (it explains how to play with lg.sh, the rules of the exercise and the report you must write).
YOUR DECK: Alurentell. OPPONENT'S DECK: $ON. YOUR TAG: ${ID}g${G}a.
$KIND $FA Your 60-card game deck (main, blank line, sideboard): $W/matches/$ID/g${G}/a-decklist.txt. Your base 75: $BA/alurentell.txt. The opponent's base 75: $W/decks-base/$BD.txt.
$NOTE
Start with: $W/lg.sh show ${ID}g${G}a . Play to the end, then write $W/reports/${ID}g${G}a.md as the brief says and give the same text as your final answer.
T
cat > tasks/${ID}g${G}b.txt <<T
You are a player in a Magic: The Gathering game. Do not call any mcp__hearthbot__ tools. First read $BR completely and follow it exactly (it explains how to play with lg.sh, the rules of the exercise and the report you must write).
YOUR DECK: $ON. OPPONENT'S DECK: Alurentell. YOUR TAG: ${ID}g${G}b.
$KIND $FB Your 60-card game deck (main, blank line, sideboard): $W/matches/$ID/g${G}/b-decklist.txt. Your base 75: $W/decks-base/$BD.txt. The opponent's base 75 is unknown to you at game one$( [ $G = 2 ] && echo "; for this game it is $BA/alurentell.txt" ).
You have no playbook: use your own Legacy knowledge. Start with: $W/lg.sh show ${ID}g${G}b . Play to the end, then write $W/reports/${ID}g${G}b.md as the brief says (EMRAKUL: n/a) and give the same text as your final answer.
T
echo "ready ${ID}: $M $ARM first=$FIRST game=$G"
