#!/bin/bash
# mkcx.sh <arm cur|brady> <n 1..12> <first A|B>   -> game id cx<arm><n>; opponent by n: 1-2 UR, 3-4 Dimir, 5-6 UW, 7-8 Boros, 9-10 Storm, 11-12 Blue Tron. Same seed for a given n in every arm.
cd /home/claude/work/llm2
ARM=$1; N=$2; FIRST=$3; ID=cx$ARM$N
case $(( (N+1)/2 )) in 1) OPPD=ur-cutter; ON="ur-cutter (UR Cutter, Legacy)";; 2) OPPD=dimir-tempo; ON="dimir-tempo (UB Dimir Tempo, Legacy)";; 3) OPPD=uwx-control; ON="uwx-control (UW Phelia control, Legacy)";; 4) OPPD=boros-aggro; ON="boros-aggro (Boros Aggro / Energy, Legacy)";; 5) OPPD=storm; ON="storm (Beseech Storm, Legacy)";; 6) OPPD=blue-tron; ON="blue-tron (Mono Blue Tron with Force of Will and Force of Negation, Legacy)";; esac
W=/home/claude/work/llm2; D=$W/matches/$ID/g1; mkdir -p $D/engine-decks
grep -v '^#' $W/arms/$ARM/alurentell.txt | sed '1{/^$/d}' > $D/engine-decks/alurentell.txt
grep -v '^#' $W/decks-base/$OPPD.txt | sed '1{/^$/d}' > $D/engine-decks/$OPPD.txt
cp $D/engine-decks/alurentell.txt $D/a-decklist.txt; cp $D/engine-decks/$OPPD.txt $D/b-decklist.txt
SEED=$((9300+N)); SEAT=0; [ $FIRST = A ] && F=0 || F=1
export LLM_BIN=$W/bin/llmgame-live LLM_DECKS=$D/engine-decks LLM_DECK=alurentell LLM_OPP=$OPPD
export LLM_EXTRA="--pub0 $W/arms/$ARM/alurentell.txt --pub1 $W/decks-base/$OPPD.txt --no-consult"
$W/lg.sh launch2 ${ID}g1 $SEED $SEAT $F
if [ $FIRST = A ]; then FA="You play FIRST (on the play)."; FB="The opponent plays FIRST; you are on the draw."; else FA="The opponent plays FIRST; you are on the draw."; FB="You play FIRST (on the play)."; fi
BR=$W/BRIEF-chantveil.md
cat > tasks/${ID}g1a.txt <<T
You are a player in a Magic: The Gathering game. Do not call any mcp__hearthbot__ tools. First read $BR completely and follow it exactly (it explains how to play with lg.sh, the rules of the exercise and the report you must write).
YOUR DECK: Alurentell. OPPONENT'S DECK: $ON. YOUR TAG: ${ID}g1a.
Game ONE (unboarded, main decks). $FA Your 60-card game deck (main, blank line, sideboard): $D/a-decklist.txt. Your base 75: $D/a-decklist.txt. The opponent's main deck: $W/decks-base/$OPPD.txt (first 60 lines).
Start with: $W/lg.sh show ${ID}g1a . Play to the end, then write $W/reports/${ID}g1a.md as the brief says and give the same text as your final answer.
T
cat > tasks/${ID}g1b.txt <<T
You are a player in a Magic: The Gathering game. Do not call any mcp__hearthbot__ tools. First read $BR completely and follow it exactly (it explains how to play with lg.sh, the rules of the exercise and the report you must write).
YOUR DECK: $ON. OPPONENT'S DECK: Alurentell. YOUR TAG: ${ID}g1b.
Game ONE (unboarded, main decks). $FB Your 60-card game deck (main, blank line, sideboard): $D/b-decklist.txt. Your base 75: $W/decks-base/$OPPD.txt. The opponent's deck is unknown to you at game one.
You have no playbook: use your own Legacy knowledge. Start with: $W/lg.sh show ${ID}g1b . Play to the end, then write $W/reports/${ID}g1b.md as the brief says (PROTECT: n/a) and give the same text as your final answer.
T
echo "ready $ID: $ON first=$FIRST seed=$SEED"
