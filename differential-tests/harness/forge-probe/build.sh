#!/bin/sh
# Compile the Forge-side probe (package itest, next to the interaction harness classes) against the Forge jar.
set -e
FORGE_ROOT=${FORGE_ROOT:-/home/claude/card-forge/forge}
JAR=$FORGE_ROOT/forge-gui-desktop/target/forge-gui-desktop-2.0.16-SNAPSHOT-jar-with-dependencies.jar
HERE=$(cd "$(dirname "$0")" && pwd)
OUT=${OUT:-/home/claude/itest-build}
mkdir -p "$OUT"
javac -nowarn -Xlint:-unchecked -d "$OUT" -cp "$OUT:$JAR" "$HERE"/src/*.java
echo "built into $OUT"
