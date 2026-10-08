#!/bin/sh
# newgame.sh N MATCHUP   (MATCHUP = ur-cutter | boros-aggro | ...): starts game wNgM engine and records metadata.
# Game id: g$N ; players then use tags g${N}a (Alurentell) and g${N}b (opponent deck).
N=$1; OPP=$2
G=g$N
LLM_OPP=$OPP LLM_DECK=alurentell /home/claude/work/llm2/lg.sh launch2 $G $((3000+N)) $(( (N+1)%2 )) 0
python3 /home/claude/work/llm2/mkmeta.py add $G $((3000+N)) $(( (N+1)%2 )) 0 alurentell $OPP
