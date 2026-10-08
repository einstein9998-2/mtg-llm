#!/bin/sh
# usage: lg.sh start <tag> <seed> <llm-seat> <first>   |  lg.sh pick <tag> <seq> <idx>  |  lg.sh show <tag>
#        lg.sh consult <tag> <seq> [iters]   (net + search read of the current prompt; does not answer it)
#        lg.sh odds <tag> <seq> <N> ["Card;Card;lands"]   (library odds: chance of seeing a card in the top N)
#        lg.sh opp  <tag> <seq> ["Force of Will;Daze"]    (chance the opponent holds a card)
#        lg.sh launch2 <game> <seed> <seat-of-A> <first>   (two-LLM game: starts the engine only; players use tags <game>a and <game>b)
#        lg.sh auto <tag> <seq> <N> <lifemin> "rule;rule"   (macro: answer by label rules for up to N picks)
#        lg.sh sim  <tag> <seq> "<script>"   (Monte Carlo what-if over your unknown cards, see BRIEF)
W=${LLM_W:-/home/claude/work/llm2}
BIN=${LLM_BIN:-/home/claude/target-live-new/release/llmgame}
DECKS=${LLM_DECKS:-/home/claude/work/live/engine/decks}
MYDECK=${LLM_DECK:-alurentell}; OPPDECK=${LLM_OPP:-ur-cutter}   # override to play another matchup
cmd=$1; tag=$2
P=$W/runs/$tag/prompt   # player-visible
T=$W/runs/$tag/truth    # harness log; do not read during a game
wait_new() { # wait until prompt.txt changes from $1 (md5) 
  i=0
  while [ $i -lt 36000 ]; do
    if [ -f $P/prompt.txt ]; then
      h=$(md5sum $P/prompt.txt | cut -d' ' -f1)
      [ "$h" != "$1" ] && { cat $P/prompt.txt; return; }
    fi
    sleep 0.1; i=$((i+1))
  done
  echo "TIMEOUT waiting for the other player or the engine: run lg.sh show $tag again to keep waiting"
}
case $cmd in
 flag) # flag TAG SEQ "why you were unsure": marks a decision for Brady's tough-spot review; does not answer or change the game
  mkdir -p $W/flags; printf "%s\t%s\t%s\n" "$tag" "$3" "$4" >> $W/flags/$tag.tsv; echo "flagged prompt $3" ;;
 launch2) # engine only; A (seat $4) answers through ${tag}a, B through ${tag}b; the orchestrator never reads either
  G=$tag; seed=$3; seat=$4; first=$5; iters=${6:-32}
  PA=$W/runs/${G}a/prompt; PB=$W/runs/${G}b/prompt; T2=$W/runs/$G/truth
  rm -rf $W/runs/${G}a $W/runs/${G}b $W/runs/$G; mkdir -p $PA $PB $T2
  nohup $BIN $DECKS $MYDECK $OPPDECK $PA $T2 --seed $seed --llm-seat $seat --first $first --iters $iters --opp-pdir $PB $LLM_EXTRA > $T2/stdout.txt 2>&1 &
  echo "launched $G" ;;
 start)
  seed=$3; seat=$4; first=$5; iters=${6:-32}
  extra="$LLM_EXTRA"; [ -f $W/opp_net.cfg ] && extra="$extra --opp-net $(cat $W/opp_net.cfg)"
  rm -rf $W/runs/$tag; mkdir -p $P $T
  nohup $BIN $DECKS $MYDECK $OPPDECK $P $T --seed $seed --llm-seat $seat --first $first --iters $iters $extra > $T/stdout.txt 2>&1 &
  wait_new none ;;
 pick)
  h=$(md5sum $P/prompt.txt | cut -d' ' -f1)
  echo "$3 $4" > $P/ans.tmp && mv $P/ans.tmp $P/ans.txt
  wait_new $h ;;
 auto) # auto TAG SEQ N LIFEMIN "rule;rule;..." : answer this prompt and the next ones by label rules (loops), see BRIEF
  h=$(md5sum $P/prompt.txt | cut -d' ' -f1)
  echo "$3 $4 $5 $6" > $P/auto.tmp && mv $P/auto.tmp $P/auto.req
  wait_new $h ;;
 show) cat $P/prompt.txt ;;
 consult|odds|opp|sim)
  rm -f $P/consult.txt
  case $cmd in consult) echo "$3 $4" ;; odds) echo "$3 odds $4 $5" ;; opp) echo "$3 opp $4" ;; sim) echo "$3 sim $4" ;; esac > $P/consult.tmp && mv $P/consult.tmp $P/consult.req
  i=0
  while [ $i -lt 3000 ]; do
    [ -f $P/consult.txt ] && { cat $P/consult.txt; rm -f $P/consult.txt; exit 0; }
    sleep 0.1; i=$((i+1))
  done
  echo "TIMEOUT waiting for consult" ;;
esac
