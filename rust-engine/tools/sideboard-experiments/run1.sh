#!/bin/bash
# run1.sh <matchup> <config> <play|draw> <games> [threads] [batch]   (env MODE=nn|nr: opponent seat plays the net | rollout search)
# One cell: Alurentell (deck A, net) vs the matchup's opponent, N independent single games on a fixed side.
#   config g1main: both decks unboarded (a game one). Any other config: A uses plans/<m>/<config>, the opponent its own
#   plan from plans/<m>/base, both boarded (a postboard game).
set -e
cd "$(dirname "$0")"
M=$1; C=$2; SIDE=$3; N=$4; TH=${5:-4}; BATCH=${6:-1}; SEED=$((20+BATCH)); MODE=${MODE:-nn}
OPPARG=(); [ "$MODE" = nr ] && OPPARG=(--opp rollout)
BIN=/home/claude/target-live/release; NETDIR=nets
case $2 in emr*) BIN=/home/claude/target-emr/release; NETDIR=nets-emr;; esac
case $M in ur) OPP=ur-cutter;; ub) OPP=dimir-tempo;; uw) OPP=uwx-control;; boros) OPP=boros-aggro;; mirror) OPP=alurentell;;
 reanimator) OPP=reanimator;; storm) OPP=storm;; tron) OPP=colorless-tron;; *) echo bad matchup; exit 1;; esac
case $M in ub) NET=ub;; uw) NET=uwphelia;; *) NET=$M;; esac
FIRST=a; [ "$SIDE" = draw ] && FIRST=b
DECKS=decks; [ -d decks-$M-$C ] && DECKS=decks-$M-$C
OUT=results/$MODE/$M; mkdir -p $OUT; TAG=$C-$SIDE.b$BATCH
TRARG=(--trace $OUT/$TAG.jsonl)
[ -s $OUT/$TAG.log ] && { echo "skip $M $C $SIDE (done)"; exit 0; }
if [ "$C" = g1main ]; then
  ARGS=(plans/$M/base --board none --public plans/$M/base)
else
  ARGS=(plans/$M/$C --board both --board-g1 --public plans/$M/base)
  [ "$M" = mirror ] && ARGS+=(--plans-b plans/$M/base)
  # the opponent's plan comes from the base dir: copy it next to the config's plan file
fi
$BIN/bo3 $DECKS "${ARGS[0]}" alurentell $OPP $N 32 --net $NETDIR/$NET.bin --threads $TH --first $FIRST --games 1 --seed $SEED "${OPPARG[@]}" "${ARGS[@]:1}" --csv $OUT/$TAG.csv "${TRARG[@]}" > $OUT/$TAG.log.tmp
mv $OUT/$TAG.log.tmp $OUT/$TAG.log
head -1 $OUT/$TAG.log
