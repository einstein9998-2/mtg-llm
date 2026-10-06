#!/bin/bash
# Plays every variant in sideboard-variants/ (or $VARIANTS) as best-of-three, traces and replays the
# games, then summarises. Usage: run_variants.sh <net.bin|none> <matches> <iterations> <out dir> [threads]
# Deck A is Alurentell in seat 0, deck B is $OPP (default ur-cutter); only the variant's plan files are used.
# PUBLIC=<dir> is the standard plan set both seats are assumed to know (default $VARIANTS/base); beliefs never read the variant under test.
# OPPARG=rollout makes deck B play plain rollout search while A uses the net (the net arena design).
# PLANS_B=<dir> gives deck B its own plan directory (mirror: variant vs baseline). VARIANTS=<dir> picks the variant set.
set -e
cd "$(dirname "$0")/.."
NET=$1; N=${2:-1000}; IT=${3:-32}; OUT=${4:-/tmp/sbvariants}; TH=${5:-3}
BIN=${CARGO_TARGET_DIR:-/home/claude/target-chant2}/release
for d in ${VARIANTS:-sideboard-variants}/*/; do
  v=$(basename "$d"); mkdir -p "$OUT/$v"
  netarg=(); [ "$NET" != none ] && netarg=(--net "$NET")
  $BIN/bo3 decks "$d" alurentell "${OPP:-ur-cutter}" "$N" "$IT" "${netarg[@]}" --threads "$TH" ${OPPARG:+--opp "$OPPARG"} --public "${PUBLIC:-${VARIANTS:-sideboard-variants}/base}" --seed 21 ${PLANS_B:+--plans-b "$PLANS_B"} --trace "$OUT/$v/games.jsonl" | tee -a "$OUT/summary.log"
  $BIN/sbtrace "$OUT/$v/games.jsonl" > "$OUT/$v/features.jsonl"
done
python3 tools/sbvariants.py "$OUT" > "$OUT/REPORT.md"
