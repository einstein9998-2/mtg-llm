#!/bin/bash
# mk2.sh <game id> <matchup> <arm plan> <first A|B> <game 1|2>: like mk.sh for game 1 (unboarded) or game 2 (boarded), with Alurentell's base 75 from $BASE_A and the Emrakul engine build
cd /home/claude/work/llm2
ID=$1; M=$2; ARM=$3; FIRST=$4; G=$5
case $M in storm) OPPD=storm;; ur) OPPD=ur-cutter;; mirror) OPPD=alurentell-b;; tron) OPPD=colorless-tron;; esac
export OPP=$OPPD
export PLANS_A=/home/claude/sbexp/plans/$M/$ARM
export PLANS_B=/home/claude/sbexp/oppplans
export BASE_A=${BASE_A:-/home/claude/work/llm2/decks-base-emr}
export LLM_BIN=/home/claude/work/llm2/bin/llmgame-emr2
python3 match.py prep $ID $G $FIRST > /tmp/mk-$ID.txt || exit 1
export LLM_EXTRA="--pub0 $BASE_A/alurentell.txt --pub1 /home/claude/work/llm2/decks-base/$OPPD.txt --no-consult"
eval "$(head -1 /tmp/mk-$ID.txt)"
tail -2 /tmp/mk-$ID.txt
