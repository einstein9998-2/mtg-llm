#!/bin/bash
# mk.sh <game id> <matchup: storm|ur|mirror|tron> <arm: base|none> <first: A|B>   -> preps the boarded decks, launches the engine, prints the player tags
cd /home/claude/work/llm2
ID=$1; M=$2; ARM=$3; FIRST=$4
case $M in storm) OPPD=storm;; ur) OPPD=ur-cutter;; mirror) OPPD=alurentell-b;; tron) OPPD=colorless-tron;; esac
export OPP=$OPPD
export PLANS_A=/home/claude/sbexp/plans/$M/$ARM
export PLANS_B=/home/claude/sbexp/oppplans
[ $M = mirror ] && { mkdir -p arms/mirror-b; sed 's/^vs alurentell$/vs alurentell/' /home/claude/sbexp/plans/mirror/base/alurentell.txt > arms/mirror-b/alurentell.txt; PLANS_B=/home/claude/work/llm2/arms/mirror-b; export PLANS_B; }
python3 match.py prep $ID 2 $FIRST > /tmp/mk-$ID.txt || exit 1
export LLM_EXTRA="--pub0 /home/claude/work/llm2/decks-base/alurentell.txt --pub1 /home/claude/work/llm2/decks-base/$OPPD.txt --no-consult"
eval "$(head -1 /tmp/mk-$ID.txt)"
tail -2 /tmp/mk-$ID.txt
echo "tags: ${ID}g2a (Alurentell, $ARM, $([ $FIRST = A ] && echo on the play || echo on the draw)) ${ID}g2b ($OPPD)"
