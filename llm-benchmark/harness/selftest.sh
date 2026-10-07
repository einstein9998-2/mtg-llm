#!/bin/sh
# Checks the odds/opp/sim tools against exact hypergeometric values at the start of game seed 201 (Alurentell vs UR Cutter, pinned engine).
# usage: selftest.sh   (needs lg.sh's environment: LLM_W, LLM_BIN, LLM_DECKS, opp_net.cfg as for a normal game)
LG=${LG:-$(dirname $0)/lg.sh}
$LG start selftest 201 0 0 >/dev/null || exit 1
fail=0
chk() { # name, got, want, tol
  ok=$(python3 -c "print(1 if abs($2-$3)<=$4 else 0)")
  [ "$ok" = 1 ] && echo "ok   $1: $2 (exact $3)" || { echo "FAIL $1: $2 (exact $3)"; fail=1; }
}
pct() { sed -n 's/.*: \([0-9.]*\)% .*/\1/p' | head -1; }
chk "odds top3 Show and Tell"  "$($LG odds selftest 1 3 "Show and Tell" | sed -n 's/.*top 3: \([0-9]*\)%.*/\1/p')" 21 1
chk "sim draw 3 Show and Tell" "$($LG sim selftest 1 'draw 3; goal "Show and Tell"' | pct)" 21.35 1.2
chk "sim brainstorm = draw 3"  "$($LG sim selftest 1 'brainstorm putback lands lands; goal "Show and Tell"' | pct)" 21.35 1.2
chk "opp holds Force of Will"  "$($LG sim selftest 1 'opp; goal "Force of Will"' | pct)" 39.95 1.5
chk "opp Force or Daze"        "$($LG sim selftest 1 'opp; goal "Force of Will" or "Daze"' | pct)" 65.36 1.5
chk "opp odds tool"            "$($LG opp selftest 1 "Force of Will" | sed -n 's/.*hold at least one: \([0-9]*\)%.*/\1/p')" 40 1
for p in $(ps aux | grep llmgame | grep selftest | grep -v grep | awk '{print $2}'); do kill $p; done
exit $fail
