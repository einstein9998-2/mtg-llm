package silo;

import java.io.File;
import java.io.PrintWriter;
import java.util.Map;

public final class Summary {
    static String obj(Map<String, ?> m) {
        StringBuilder sb = new StringBuilder("{");
        boolean f = true;
        for (Map.Entry<String, ?> e : m.entrySet()) { if (!f) sb.append(','); f = false; sb.append(Jsonl.esc(e.getKey())).append(':').append(e.getValue()); }
        return sb.append('}').toString();
    }

    /** Estimated tokens: chars / 3.6. No tokenizer is available offline; this is a rough planning number, not a billed count. */
    static long tok(long chars) { return Math.round(chars / 3.6); }

    public static void write(File f, String policy, String stops, String a, String b, int games, long wins, long aiWins, long draws, long onPlay, long onPlayWins, long ms, Stats s) throws Exception {
        long totalChars = 0, nAsked = 0;
        for (long c : s.chars.values()) totalChars += c;
        for (int c : s.asked.values()) nAsked += c;
        try (PrintWriter w = new PrintWriter(f)) {
            w.println("{");
            w.println("\"policy\":" + Jsonl.esc(policy) + ",\"stops\":" + Jsonl.esc(stops) + ",\"seatA_deck\":" + Jsonl.esc(a) + ",\"seatB_deck(ForgeAI)\":" + Jsonl.esc(b) + ",");
            w.println("\"games\":" + games + ",\"seatA_wins\":" + wins + ",\"forgeAI_wins\":" + aiWins + ",\"draws\":" + draws + ",\"seatA_games_on_play\":" + onPlay + ",\"seatA_wins_on_play\":" + onPlayWins + ",\"wall_ms\":" + ms + ",");
            w.println("\"prompts\":" + nAsked + ",\"prompt_chars\":" + totalChars + ",\"prompt_tokens_est\":" + tok(totalChars) + ",\"prompts_per_game\":" + (games == 0 ? 0 : Math.round(10.0 * nAsked / games) / 10.0) + ",");
            w.println("\"asked_by_kind\":" + obj(s.asked) + ",");
            w.println("\"chars_by_kind\":" + obj(s.chars) + ",\"chars_if_every_prompt_carried_full_state\":" + obj(s.charsFull) + ",");
            w.println("\"menu_enumeration_ms_avg\":" + (s.menuCalls == 0 ? 0 : Math.round(100.0 * s.menuNanos / s.menuCalls / 1e6) / 100.0) + ",\"menu_calls\":" + s.menuCalls + ",\"prompt_render_ms_avg\":" + (s.promptCalls == 0 ? 0 : Math.round(100.0 * s.promptNanos / s.promptCalls / 1e6) / 100.0) + ",");
            w.println("\"forced_no_prompt\":" + obj(s.forced) + ",");
            w.println("\"auto_pass_by_rule\":" + obj(s.autoStop) + ",");
            w.println("\"delegated_to_forge_ai\":" + obj(s.delegated) + ",");
            w.println("\"problems\":" + obj(s.problems) + ",");
            w.println("\"leak_checks\":" + s.leakChecks + ",\"leak_failures\":" + s.leakFailures + ",");
            w.println("\"mirror_ai_priority_pick_on_menu\":" + s.aiMenuMatches + ",\"mirror_ai_priority_pick_missing\":" + s.aiMenuMisses + ",");
            w.print("\"problem_samples\":[");
            for (int i = 0; i < s.problemSamples.size(); i++) { if (i > 0) w.print(","); w.print(Jsonl.esc(s.problemSamples.get(i))); }
            w.println("]\n}");
        }
    }

    public static void print(String policy, String stops, int games, long wins, long aiWins, long draws, long ms, Stats s) {
        long totalChars = 0, nAsked = 0;
        for (long c : s.chars.values()) totalChars += c;
        for (int c : s.asked.values()) nAsked += c;
        System.out.printf("%npolicy=%s stops=%s games=%d  seatA %d / forgeAI %d / draws %d  wall %.1fs%n", policy, stops, games, wins, aiWins, draws, ms / 1000.0);
        System.out.printf("prompts %d (%.1f/game), %d chars (~%d tokens est), %.0f chars/prompt avg%n", nAsked, games == 0 ? 0 : (double) nAsked / games, totalChars, tok(totalChars), nAsked == 0 ? 0 : (double) totalChars / nAsked);
        { long fc = 0; for (long v : s.charsFull.values()) fc += v; System.out.printf("if every prompt carried the full state: %d chars (~%d tokens est), %.0f chars/prompt%n", fc, tok(fc), nAsked == 0 ? 0 : (double) fc / nAsked);
          System.out.printf("menu enumeration %.2f ms avg over %d calls; prompt render %.2f ms avg%n", s.menuCalls == 0 ? 0 : s.menuNanos / s.menuCalls / 1e6, s.menuCalls, s.promptCalls == 0 ? 0 : s.promptNanos / s.promptCalls / 1e6); }
        System.out.println("asked " + s.asked); System.out.println("forced " + s.forced); System.out.println("autoStop " + s.autoStop);
        System.out.println("delegated " + s.delegated); System.out.println("problems " + s.problems);
        for (String p : s.problemSamples) System.out.println("  " + p);
        System.out.println("non-interference checks " + s.leakChecks + ", failures " + s.leakFailures);
        System.out.println("mirror priority picks on menu " + s.aiMenuMatches + " / missing " + s.aiMenuMisses);
    }
}
