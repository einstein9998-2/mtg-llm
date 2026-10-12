#!/bin/bash
cd /home/claude/sbexp
for q in "$@"; do n=0; while read -r MODE M C S N B T; do [ -z "$MODE" ] && continue; [ -s results/$MODE/$M/$C-$S.b$B.log ] || n=$((n+1)); done < $q; echo "$q remaining $n"; done
