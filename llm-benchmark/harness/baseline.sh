#!/bin/sh
# baseline.sh <out.tsv> <first_seed> <n> <iters>   : rollout-MCTS plays Alurentell vs rollout-MCTS UR Cutter
out=$1; s0=$2; n=$3; it=$4; extra="$5"
: > $out
i=0
while [ $i -lt $n ]; do
  seed=$((s0+i)); seat=$((i%2))
  d=/home/claude/work/llm/runs/base_$$/p; t=/home/claude/work/llm/runs/base_$$/t
  mkdir -p $d $t
  /home/claude/target-mtg/release/llmgame /home/claude/work/rust-engine/decks alurentell ur-cutter $d $t --seed $seed --llm-seat $seat --first 0 --iters $it --auto-llm $extra
  printf "%s\t%s\t%s\n" $seed $seat "$(cut -d'|' -f1 $d/result.txt)" >> $out
  i=$((i+1))
done
rm -rf /home/claude/work/llm/runs/base_$$
