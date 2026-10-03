#!/bin/sh
# Usage: bash run.sh <silo.Run args>   e.g. bash run.sh --a dnt.dck --b jund75.dck --games 10 --policy mirror --out /home/claude/silo-out/mirror
FORGE_ROOT=${FORGE_ROOT:-/home/claude/card-forge/forge}
JAR=$FORGE_ROOT/forge-gui-desktop/target/forge-gui-desktop-2.0.16-SNAPSHOT-jar-with-dependencies.jar
OUT=${OUT:-/home/claude/silo-build}
HERE=$(cd "$(dirname "$0")" && pwd)
cd "$FORGE_ROOT/forge-gui" || exit 1   # Forge finds ./res relative to cwd
exec java $JAVA_EXTRA -Djava.awt.headless=true -Xmx2g -XX:+UseParallelGC -cp "$OUT:$JAR" silo.Run --decks /mnt/project-files/forge-runner/decks "$@"
