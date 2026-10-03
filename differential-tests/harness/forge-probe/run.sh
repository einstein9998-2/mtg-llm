#!/bin/sh
# Usage: sh run.sh <positions.jsonl> <forge-out.jsonl>
FORGE_ROOT=${FORGE_ROOT:-/home/claude/card-forge/forge}
JAR=$FORGE_ROOT/forge-gui-desktop/target/forge-gui-desktop-2.0.16-SNAPSHOT-jar-with-dependencies.jar
OUT=${OUT:-/home/claude/itest-build}
IN=$(readlink -f "$1"); O=$(readlink -f "$2" 2>/dev/null || echo "$(cd "$(dirname "$2")" && pwd)/$(basename "$2")")
cd "$FORGE_ROOT/forge-gui" || exit 1
exec java -Djava.awt.headless=true -Xmx${XMX:-3g} -XX:+UseParallelGC -cp "$OUT:$JAR" itest.Probe "$IN" "$O"
