#!/bin/bash
# Plays every variant in sideboard-variants/ (or $VARIANTS) as best-of-three, traces and replays the
# games, then summarises. Usage: run_variants.sh <net.bin|none> <matches> <iterations> <out dir> [threads]
# Deck A is Alurentell in seat 0, deck B is UR Cutter; only the two plan files of the variant are used.
set -e
cd "$(dirname "$0")/.."
NET=$1; N=${2:-1000}; IT=${3:-32}; OUT=${4:-/tmp/sbvariants}; TH=${5:-3}
BIN=${CARGO_TARGET_DIR:-/home/claude/target-mtg}/release
for d in ${VARIANTS:-sideboard-variants}/*/; do
  v=$(basename "$d"); mkdir -p "$OUT/$v"
  netarg=(); [ "$NET" != none ] && netarg=(--net "$NET")
  $BIN/bo3 decks "$d" alurentell ur-cutter "$N" "$IT" "${netarg[@]}" --threads "$TH" --seed 21 --trace "$OUT/$v/games.jsonl" | tee -a "$OUT/summary.log"
  $BIN/sbtrace "$OUT/$v/games.jsonl" > "$OUT/$v/features.jsonl"
done
python3 tools/sbvariants.py "$OUT" > "$OUT/REPORT.md"
