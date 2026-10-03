#!/bin/sh
# Compile against the Forge jar built per /mnt/project-files/forge-runner/README.md. Output: build/classes
set -e
FORGE_ROOT=${FORGE_ROOT:-/home/claude/card-forge/forge}
JAR=$FORGE_ROOT/forge-gui-desktop/target/forge-gui-desktop-2.0.16-SNAPSHOT-jar-with-dependencies.jar
HERE=$(cd "$(dirname "$0")" && pwd)
OUT=${OUT:-/home/claude/silo-build}   # build output kept off the shared project folder
mkdir -p "$OUT"
python3 "$HERE/tools/gen_traced.py" "$FORGE_ROOT/forge-ai/src/main/java/forge/ai/PlayerControllerAi.java" "$HERE/src/silo/TracedAi.java"
javac -nowarn -Xlint:-unchecked -d "$OUT" -cp "$JAR" "$HERE"/src/silo/*.java
echo "built into $OUT"
