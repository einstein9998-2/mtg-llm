package silo;

import java.util.Random;

/** Chooses among a Decision's options. Return null to delegate this decision to Forge's own AI (counted as delegated). */
public interface Policy {
    /** @return indices into d.opts (length within [d.min, d.max]) or null to delegate. */
    int[] choose(Decision d);
    default String name() { return getClass().getSimpleName(); }
    /** True if the controller should also compute what Forge's AI would play at priority and expose it as Decision.aiHint. */
    default boolean wantsAiHint() { return false; }

    /** Always the first option (Pass for priority). Smoke-test baseline. */
    final class First implements Policy {
        public int[] choose(Decision d) {
            int n = Math.max(d.min, 1);
            int[] r = new int[Math.min(n, d.opts.size())];
            for (int i = 0; i < r.length; i++) r[i] = i;
            return r;
        }
    }

    /** Uniform random over legal options, with extra weight on passing so games progress. Exercises every code path. */
    final class RandomPolicy implements Policy {
        final Random rng; final double passBias;
        public RandomPolicy(long seed, double passBias) { rng = new Random(seed); this.passBias = passBias; }
        public int[] choose(Decision d) {
            int n = d.opts.size();
            if (d.kind.equals("Priority") && rng.nextDouble() < passBias) return new int[]{0};
            int k = d.min + (d.max > d.min ? rng.nextInt(Math.min(d.max, n) - d.min + 1) : 0);
            k = Math.min(k, n);
            int[] idx = new int[n];
            for (int i = 0; i < n; i++) idx[i] = i;
            for (int i = n - 1; i > 0; i--) { int j = rng.nextInt(i + 1); int t = idx[i]; idx[i] = idx[j]; idx[j] = t; }
            int[] r = new int[k];
            System.arraycopy(idx, 0, r, 0, k);
            java.util.Arrays.sort(r);
            return r;
        }
    }

    /** Delegates every decision to Forge's AI (the prompt is still built and measured). Use to get realistic trajectories. */
    final class Mirror implements Policy {
        public int[] choose(Decision d) { return null; }
    }

    /** Follows Forge's AI at priority (through the masked menu), random on every sub-decision. Gives long, varied games that
     *  still exercise targets, modes, attacks, blocks and choices through our own action path. */
    final class Hybrid implements Policy {
        final RandomPolicy rnd;
        public Hybrid(long seed) { rnd = new RandomPolicy(seed, 0.0); }
        public boolean wantsAiHint() { return true; }
        public int[] choose(Decision d) {
            if (d.kind.equals("Priority")) return new int[]{Math.max(d.aiHint, 0)};
            return rnd.choose(d);
        }
    }

    /**
     * Hands every decision to an external process (an LLM harness) over two named pipes, one JSON object per line.
     *   engine -> driver  {"type":"decision","game":0,"id":12,"kind":"Priority","min":1,"max":1,"n":4,"prompt":"..."}
     *   driver -> engine  {"pick":[2]}          (indices into the numbered options in the prompt)
     * The driver sees only what the prompt shows. Illegal replies are rejected by the controller (recorded, first legal option used).
     */
    final class External implements Policy, java.io.Closeable {
        private final java.io.PrintWriter out; private final java.io.BufferedReader in; public int game;
        public External(String toDriverPipe, String fromDriverPipe) throws java.io.IOException {
            out = new java.io.PrintWriter(new java.io.FileWriter(toDriverPipe), true);   // blocks until the driver opens it for reading
            in = new java.io.BufferedReader(new java.io.FileReader(fromDriverPipe));      // blocks until the driver opens it for writing
        }
        public int[] choose(Decision d) {
            out.println("{\"type\":\"decision\",\"game\":" + game + ",\"id\":" + d.id + ",\"kind\":" + Jsonl.esc(d.kind) + ",\"min\":" + d.min + ",\"max\":" + d.max
                    + ",\"n\":" + d.opts.size() + ",\"prompt\":" + Jsonl.esc(d.prompt) + "}");
            try {
                String line = in.readLine();
                if (line == null) throw new IllegalStateException("driver closed the pipe");
                java.util.regex.Matcher m = java.util.regex.Pattern.compile("\"pick\"\\s*:\\s*\\[([^\\]]*)\\]").matcher(line);
                if (!m.find()) return new int[]{-1}; // malformed: rejected as an illegal pick by the controller
                String body = m.group(1).trim();
                if (body.isEmpty()) return new int[0];
                String[] parts = body.split(",");
                int[] r = new int[parts.length];
                for (int i = 0; i < r.length; i++) r[i] = Integer.parseInt(parts[i].trim());
                return r;
            } catch (java.io.IOException e) { throw new IllegalStateException(e); }
        }
        public void close() { out.println("{\"type\":\"bye\"}"); out.close(); }
    }
}
