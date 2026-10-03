package silo;

import java.util.*;

/** Counters for one run. Everything is per decision kind or per delegated Forge-AI method. */
public final class Stats {
    public final Map<String, Integer> asked = new TreeMap<>();       // prompts shown to the policy, by kind
    public final Map<String, Integer> forced = new TreeMap<>();      // single legal action: resolved without prompting
    public final Map<String, Integer> autoStop = new TreeMap<>();    // priority passed by stop rules (reason -> count)
    public final Map<String, Integer> delegated = new TreeMap<>();   // handed to Forge's own AI because no masked decision exists yet
    public final Map<String, Long> chars = new TreeMap<>();          // prompt characters by kind
    public final Map<String, Long> charsFull = new TreeMap<>();      // what the same prompts would cost if every one carried the full state (stateless calls)
    public long menuNanos, menuCalls, promptNanos, promptCalls;
    public final Map<String, Integer> problems = new TreeMap<>();    // enumerator or engine anomalies (see Run for meaning)
    public final List<String> problemSamples = new ArrayList<>();
    public int games, aiMenuMatches, aiMenuMisses, leakChecks, leakFailures;                   // Mirror policy: Forge AI's priority pick found in / missing from our menu
    private static final Set<String> HOUSEKEEPING = Set.of("isAI", "getAi", "pilotsNonAggroDeck", "setupAutoProfile", "acceptsDrawOffer",
            "awaitNextInput", "cancelAwaitNextInput", "autoPassCancel", "resetAtEndOfTurn", "notifyOfValue", "cheatShuffle",
            "complainCardsCantPlayWell", "revealAnte", "revealAISkipCards", "revealUnsupported", "reveal", "tempShowCards", "endTempShowCards");

    public void inc(Map<String, Integer> m, String k) { m.merge(k, 1, Integer::sum); }
    public void delegated(String method) { if (!HOUSEKEEPING.contains(method)) inc(delegated, method); }
    public void problem(String kind, String detail) { inc(problems, kind); if (problemSamples.size() < 40) problemSamples.add(kind + ": " + detail); }
    public void addChars(String kind, int n) { chars.merge(kind, (long) n, Long::sum); }
    public int total(Map<String, Integer> m) { int t = 0; for (int v : m.values()) t += v; return t; }
}
