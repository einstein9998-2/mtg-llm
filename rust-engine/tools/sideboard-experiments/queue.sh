#!/bin/bash
# queue.sh <file>: lines "MODE matchup config side N batch"; runs them in order, skipping finished cells.
cd "$(dirname "$0")"
while read -r MODE M C S N B T; do
  [ -z "$MODE" ] && continue; [ "${MODE:0:1}" = "#" ] && continue
  echo "$(date +%H:%M:%S) $MODE $M $C $S $N b$B"
  TRACE=${T:+1} MODE=$MODE ./run1.sh $M $C $S $N 4 $B
done < "$1"
echo "$(date +%H:%M:%S) queue $1 done"
