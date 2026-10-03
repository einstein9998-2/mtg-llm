package silo;

import forge.game.Game;
import forge.game.player.Player;

import java.util.Set;

/** Everything one seat needs for one game: observer knowledge, serializer, policy, counters, log. */
public final class Harness {
    public final Policy policy; public final Stats stats; public final Jsonl log; public final StopConfig stops;
    public final Set<String> poolNames; public final boolean logPrompts; public final int gameNo; public final long seed;
    public Knowledge k; public Render r; public EventLog events; public StateSerializer ser;
    public int decisionSeq;
    public boolean leakTest;
    /** Extra check hook for tests: called with every rendered prompt (e.g. non-interference test). */
    public java.util.function.Consumer<Decision> onPrompt;

    public Harness(Policy policy, Stats stats, Jsonl log, StopConfig stops, Set<String> poolNames, boolean logPrompts, int gameNo, long seed) {
        this.policy = policy; this.stats = stats; this.log = log; this.stops = stops; this.poolNames = poolNames;
        this.logPrompts = logPrompts; this.gameNo = gameNo; this.seed = seed;
    }

    public void init(Game game, Player me) {
        k = new Knowledge(me); r = new Render(k); events = new EventLog(game, k, r); ser = new StateSerializer(game, me, k, r, events);
    }
}
