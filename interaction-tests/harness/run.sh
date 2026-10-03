#!/bin/sh
# Usage: sh run.sh [-v] [--only substring] <scenario.scn | directory> ...
FORGE_ROOT=${FORGE_ROOT:-/home/claude/card-forge/forge}
JAR=$FORGE_ROOT/forge-gui-desktop/target/forge-gui-desktop-2.0.16-SNAPSHOT-jar-with-dependencies.jar
OUT=${OUT:-/home/claude/itest-build}
ARGS=""
while [ $# -gt 0 ]; do
  case "$1" in
    -v) ARGS="$ARGS -v";;
    --only) ARGS="$ARGS --only '$2'"; shift;;
    *) ARGS="$ARGS $(readlink -f "$1")";;
  esac; shift
done
cd "$FORGE_ROOT/forge-gui" || exit 1
eval exec java -Djava.awt.headless=true -Xmx2g -XX:+UseParallelGC -cp "$OUT:$JAR" itest.Interact $ARGS
