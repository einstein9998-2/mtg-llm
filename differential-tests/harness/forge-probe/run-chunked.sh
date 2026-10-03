#!/bin/bash
# Usage: run-chunked.sh <positions.jsonl> <forge-out.jsonl> [chunk lines] [parallel JVMs]
# The probe leaks memory over a few thousand positions, so run fixed-size chunks in fresh JVMs and concatenate the outputs in order.
set -e
IN=$(readlink -f "$1"); DEST=$2; CH=${3:-1500}; P=${4:-3}
HERE=$(cd "$(dirname "$0")" && pwd)
TMP=$(mktemp -d "${DEST}.chunks.XXXX")
split -l "$CH" -d -a 4 "$IN" "$TMP/in-"
ls "$TMP"/in-* | xargs -P "$P" -I{} sh -c "sh $HERE/run.sh {} {}.out > {}.log 2>&1"
cat "$TMP"/in-*.out > "$DEST"
rm -rf "$TMP"
