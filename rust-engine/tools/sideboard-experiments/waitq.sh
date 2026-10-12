#!/bin/bash
# waitq.sh <queue file> <timeout_s>: returns when every cell of the queue is done
cd /home/claude/sbexp; S=$(date +%s)
while true; do
  r=$(./pending.sh $1 | awk '{print $3}')
  [ "$r" = 0 ] && { echo "queue $1 done"; exit 0; }
  [ $(( $(date +%s)-S )) -gt ${2:-3300} ] && { echo "timeout, $r left"; exit 0; }
  sleep 120
done
