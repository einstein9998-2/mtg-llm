#!/bin/sh
# Position mode: one flagged position per tag (tough-spot test). usage:
#   pos.sh start POSID | pick POSID SEQ IDX | show POSID | consult|odds|opp|sim POSID SEQ ...
# POSID.replay.json / pos.json are written by tough-eval/make_positions.py; the answer is recorded and the run ends.
W=${POS_W:-/home/claude/work/pos}
cmd=$1; id=$2
if [ "$cmd" = start ]; then
  eval $(python3 - "$W/pos.json" "$id" <<'PY'
import json,sys
p=json.load(open(sys.argv[1]))[sys.argv[2]]
print(f"SEED={p['seed']} SEAT={p['seat']} FIRST={p['first']} ME={p['me']} OPP={p['opp']} NET={p['net']}")
PY
)
  export LLM_DECK=$ME LLM_OPP=$OPP LLM_EXTRA="--replay $W/$id.replay.json --oneshot --consult-net $NET"
  exec ${LG:-/home/claude/work/llm/lg.sh} start $id $SEED $SEAT $FIRST
fi
shift 2
exec ${LG:-/home/claude/work/llm/lg.sh} $cmd $id "$@"
