#!/bin/bash
# Usage: run-lines.sh <out dir> <positions.jsonl>   (positions from linegen)
# Forge plays every line in chunked JVMs (the probe leaks memory), then linediff replays them on the engine and the summaries are written.
set -e
DIR=$1; POS=$2
TARGET=${TARGET:-/tmp/target-diff/release}; DECKS=${DECKS:-/mnt/project-files/decks}
KD=${KD:-/mnt/project-files/differential-tests/known-divergences}
HERE=${HERE:-$(cd "$(dirname "$0")/.." && pwd)}
mkdir -p "$DIR"; cd "$DIR"
cp "$POS" pos.jsonl
bash $HERE/forge-probe/run-chunked.sh pos.jsonl forge.jsonl ${CHUNK:-300} ${PAR:-3}
$TARGET/linediff pos.jsonl forge.jsonl $DECKS out.jsonl > linediff.txt 2>&1
python3 $HERE/tools/linesum.py out.jsonl 60 > summary.txt 2>&1
python3 $HERE/tools/linemenu.py out.jsonl $KD > menu.txt 2>&1
echo DONE
