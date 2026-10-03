#!/bin/bash
# Usage: run-batch.sh <out dir> <games> <seed0> <sample size> [combat|response]
#   normal mode: posgen (Rust positions + one action each) -> Forge probe step mode -> stepdiff;
#                Forge probe menu mode -> compare.py (action sets, known-divergence suppression)
#   response mode: posgen emits only stack responses (a spell, then the opponent answers with a random legal action) -> step mode -> stepdiff + resp_actions.py (action sets at the response window)
#   combat mode: posgen emits only combat runs (attack subset at main phase 1, Forge AI blocks) -> Forge step mode -> stepdiff
# Needs: TARGET (dir with built posgen/stepdiff), OUT (compiled probe classes, see forge-probe/build.sh), DECKS, KD (known-divergences dir).
set -e
DIR=$1; GAMES=${2:-8000}; SEED=${3:-40000}; N=${4:-15000}; MODE=${5:-normal}
TARGET=${TARGET:-/home/claude/target-diff3/release}; DECKS=${DECKS:-/mnt/project-files/decks}
KD=${KD:-/mnt/project-files/differential-tests/known-divergences}
HERE=$(cd "$(dirname "$0")/.." && pwd)
mkdir -p "$DIR"; cd "$DIR"
if [ "$MODE" = combat ]; then CAP=6; EXTRA=combat; elif [ "$MODE" = response ]; then CAP=6; EXTRA=response; else CAP=8; EXTRA=; fi
$TARGET/posgen $DECKS $GAMES $SEED $CAP $EXTRA > pos-all.jsonl 2> posgen.err
python3 - "$N" <<'PY'
import json, random, sys
n = int(sys.argv[1]); random.seed(1)
rows = [json.loads(l) for l in open('pos-all.jsonl')]
act = [r for r in rows if r.get('action')]
random.shuffle(act); act = act[:n]
with open('pos.jsonl', 'w') as f:
    for r in act: f.write(json.dumps(r) + '\n')
with open('pos-menu.jsonl', 'w') as f:
    for r in act:
        r = dict(r); r.pop('action', None); r.pop('action_idx', None); f.write(json.dumps(r) + '\n')
print(len(rows), 'positions,', len(act), 'sampled with an action')
PY
bash $HERE/forge-probe/run-chunked.sh pos.jsonl forge-step.jsonl 800 3
$TARGET/stepdiff pos.jsonl forge-step.jsonl $DECKS step-diff.jsonl > stepdiff.txt 2>&1
if [ "$MODE" = response ]; then
  python3 $HERE/tools/resp_actions.py pos.jsonl step-diff.jsonl $KD > resp-actions.txt 2>&1
elif [ "$MODE" = normal ]; then
  bash $HERE/forge-probe/run-chunked.sh pos-menu.jsonl forge-menu.jsonl 800 3
  python3 $HERE/tools/compare.py pos-menu.jsonl forge-menu.jsonl menu $KD > compare.txt 2>&1
fi
echo DONE
