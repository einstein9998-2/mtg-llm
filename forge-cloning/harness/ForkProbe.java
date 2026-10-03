package probe;

import com.google.common.eventbus.Subscribe;
import forge.ai.ComputerUtilAbility;
import forge.ai.simulation.GameCopier;
import forge.ai.simulation.GameSimulator;
import forge.ai.simulation.GameStateEvaluator;
import forge.ai.simulation.SimulationController;
import forge.deck.Deck;
import forge.deck.io.DeckSerializer;
import forge.game.*;
import forge.game.card.Card;
import forge.game.card.CardFactory;
import forge.game.event.GameEventTurnPhase;
import forge.game.event.GameEventSpellAbilityCast;
import forge.game.phase.PhaseType;
import forge.game.player.Player;
import forge.game.player.RegisteredPlayer;
import forge.game.spellability.SpellAbility;
import forge.game.zone.Zone;
import forge.game.zone.ZoneType;
import forge.gui.GuiBase;
import forge.GuiDesktop;
import forge.item.PaperCard;
import forge.localinstance.properties.ForgePreferences.FPref;
import forge.model.FModel;
import forge.player.GamePlayerUtil;
import forge.util.MyRandom;

import java.io.File;
import java.lang.reflect.Field;
import java.util.*;
import java.util.concurrent.*;

/**
 * Probes Forge's GameCopier / GameSimulator at real mid-game decision points:
 * copy latency, state fidelity, hidden-info exposure, determinization, simulate() latency, parallel copy, heap per copy.
 * Output: one JSON object per line on stdout, prefixed "PROBE ".
 */
public class ForkProbe {
    static String deckDir, d1, d2;
    static long seed;
    static int game, reps = 5, maxSims = 4;
    static boolean prune = false, debugOnce = true, debugNames = true, idDump = true;
    static final Set<Integer> memDone = new HashSet<>(), parDone = new HashSet<>();

    public static void main(String[] a) throws Exception {
        deckDir = a[0]; d1 = a[1]; d2 = a[2]; seed = Long.parseLong(a[3]);
        int nGames = Integer.parseInt(a[4]);
        prune = a.length > 5 && a[5].equals("prune");
        if (prune) setPrune(true);
        GuiBase.setInterface(new GuiDesktop());
        FModel.initialize(null, p -> { p.setPref(FPref.LOAD_CARD_SCRIPTS_LAZILY, false); p.setPref(FPref.UI_LANGUAGE, "en-US"); return null; });
        MyRandom.setRandom(new Random(seed));
        for (game = 1; game <= nGames; game++) {
            GameRules rules = new GameRules(GameType.Constructed);
            rules.setAppliedVariants(EnumSet.of(GameType.Constructed));
            List<RegisteredPlayer> pp = new ArrayList<>();
            String[] names = {d1, d2};
            for (int i = 0; i < 2; i++) {
                Deck d = DeckSerializer.fromFile(new File(deckDir, names[i]));
                RegisteredPlayer rp = new RegisteredPlayer(d);
                rp.setPlayer(GamePlayerUtil.createAiPlayer("Ai" + (i + 1), i, ""));
                pp.add(rp);
            }
            Match mc = new Match(rules, pp, "Probe");
            Game g = mc.createGame();
            g.setNoGUIUser();
            g.subscribeToEvents(new Listener(g));
            long t0 = System.nanoTime();
            try { mc.startGame(g); } catch (Throwable t) { emit("{\"kind\":\"game_error\",\"game\":" + game + ",\"err\":" + q(t.toString()) + "}"); }
            emit("{\"kind\":\"game_end\",\"game\":" + game + ",\"turns\":" + g.getPhaseHandler().getTurn() + ",\"ms\":" + (System.nanoTime() - t0) / 1000000 + "}");
        }
        System.exit(0);
    }

    static void setPrune(boolean v) throws Exception {
        Field f = GameCopier.class.getDeclaredField("PRUNE_HIDDEN_INFO");
        f.setAccessible(true);
        f.set(null, v); // only works if the flag has been made non-final in the local patch
    }

    static void emit(String s) { System.out.println("PROBE " + s); System.out.flush(); }
    static String q(String s) { return "\"" + s.replace("\\", "\\\\").replace("\"", "'").replace("\n", " ") + "\""; }

    static class Listener {
        final Game g; int probes = 0;
        Listener(Game g) { this.g = g; }

        int stackProbes = 0;
        @Subscribe
        public void onCast(GameEventSpellAbilityCast e) {
            if (stackProbes >= 15 || g.getStack().isEmpty() || g.getPhaseHandler().getTurn() < 2) return;
            stackProbes++;
            try { stackProbe(g); } catch (Throwable t) {
                emit("{\"kind\":\"stack_probe_error\",\"game\":" + game + ",\"err\":" + q(t.toString()) + "}");
            }
        }

        @Subscribe
        public void onPhase(GameEventTurnPhase e) {
            PhaseType ph = e.phase();
            if (ph != PhaseType.MAIN1 && ph != PhaseType.MAIN2) return;
            int turn = g.getPhaseHandler().getTurn();
            if (turn < 2 || !g.getStack().isEmpty() || g.isGameOver()) return;
            Player me = g.getPhaseHandler().getPlayerTurn();
            try { probe(g, me, ph, turn); } catch (Throwable t) {
                emit("{\"kind\":\"probe_error\",\"game\":" + game + ",\"turn\":" + turn + ",\"err\":" + q(t.toString()) + "}");
            }
        }
    }

    static void probe(Game g, Player me, PhaseType ph, int turn) throws Exception {
        Player opp = me.getWeakestOpponent();
        int boardSize = g.getCardsIn(ZoneType.Battlefield).size();
        int gyExile = g.getCardsIn(ZoneType.Graveyard).size() + g.getCardsIn(ZoneType.Exile).size();

        if (idDump) { idDump = false; Map<String, TreeSet<Integer>> m = new TreeMap<>();
            for (Card c : opp.getCardsIn(ZoneType.Library)) m.computeIfAbsent(c.getName(), k -> new TreeSet<>()).add(c.getId());
            for (Card c : opp.getCardsIn(ZoneType.Hand)) m.computeIfAbsent(c.getName(), k -> new TreeSet<>()).add(c.getId());
            emit("{\"kind\":\"id_dump\",\"opp_card_ids_by_name\":" + q(m.toString()) + "}"); }

        // 1. copy latency (first copy of each probe discarded as warm-up)
        long[] ns = new long[reps];
        Game copy = null;
        GameCopier copier = null;
        for (int i = 0; i <= reps; i++) {
            long t = System.nanoTime();
            copier = new GameCopier(g);
            Game c = copier.makeCopy();
            long dt = System.nanoTime() - t;
            if (i > 0) ns[i - 1] = dt;
            copy = c;
        }
        Arrays.sort(ns);
        double medMs = ns[reps / 2] / 1e6;

        // 2. fidelity: digest and AI evaluator score
        List<String> dOrig = digest(g, true), dCopy = digest(copy, true);
        List<String> diffs = diff(dOrig, dCopy);
        Player meCopy = copier.find(me);
        GameStateEvaluator ev = new GameStateEvaluator();
        int sOrig = ev.getScoreForGameState(g, me).value, sCopy = ev.getScoreForGameState(copy, meCopy).value;

        // 3. exposure: does the copy contain the real opponent hidden cards? (ids+names)
        Player oppCopy = copier.find(opp);
        String hidOrig = hiddenSig(opp), hidCopy = hiddenSig(oppCopy);
        boolean hiddenIdentical = hidOrig.equals(hidCopy);

        // 4. determinize opponent hidden zones in the copy; check invariants
        Deck oppDeck = opp.getRegisteredPlayer().getDeck();
        long t = System.nanoTime();
        Map<String, Object> det = determinize(copy, oppCopy, oppDeck, new Random(seed * 31 + turn));
        double detMs = (System.nanoTime() - t) / 1e6;
        List<String> dDet = digest(copy, true);
        // public-info digest must be unchanged by determinization
        List<String> pubBefore = publicDigest(dCopy, opp), pubAfter = publicDigest(dDet, opp);
        boolean publicUnchanged = pubBefore.equals(pubAfter);

        // 5. info-set invariance: two worlds with same info set but different hidden contents must determinize identically
        boolean invariant = infoSetInvariant(g, me, opp, oppDeck);

        // 6. simulate() latency over the legal spell abilities
        List<SpellAbility> sas = new ArrayList<>();
        try {
            for (SpellAbility sa : ComputerUtilAbility.getSpellAbilities(ComputerUtilAbility.getAvailableCards(g, me), me)) {
                if (sa.canPlay() && forge.ai.ComputerUtilMana.canPayManaCost(sa, me, 0, false)) sas.add(sa);
            }
        } catch (Throwable t2) { /* enumerate failure is reported below */ }
        int simOk = 0, simFail = 0; double simMs = 0; List<String> simErr = new ArrayList<>();
        for (int i = 0; i < Math.min(maxSims, sas.size()); i++) {
            SpellAbility sa = sas.get(i);
            long ts = System.nanoTime();
            try {
                // same path the Forge AI uses: evaluateSa iterates target/mode choices through an interceptor
                GameStateEvaluator.Score sc = new forge.ai.simulation.SpellAbilityPicker(me)
                        .evaluateSa(new SimulationController(new GameStateEvaluator.Score(0), 0), null, sas, i);
                if (sc.value == Integer.MIN_VALUE) { simFail++; simErr.add(sa.getHostCard() + " -> MIN_VALUE"); } else simOk++;
            } catch (Throwable t3) { simFail++; simErr.add(sa.getHostCard() + " -> " + t3); }
            simMs += (System.nanoTime() - ts) / 1e6;
        }

        StringBuilder sb = new StringBuilder("{\"kind\":\"probe\",\"game\":" + game + ",\"turn\":" + turn + ",\"phase\":\"" + ph + "\"");
        sb.append(",\"board\":").append(boardSize).append(",\"gyExile\":").append(gyExile);
        sb.append(",\"copy_ms_median\":").append(String.format("%.2f", medMs));
        sb.append(",\"copy_ms_min\":").append(String.format("%.2f", ns[0] / 1e6));
        sb.append(",\"digest_diffs\":").append(diffs.size()).append(",\"digest_diff_sample\":").append(q(String.join(" | ", diffs.subList(0, Math.min(3, diffs.size())))));
        sb.append(",\"score_orig\":").append(sOrig).append(",\"score_copy\":").append(sCopy);
        sb.append(",\"copy_exposes_opp_hidden\":").append(hiddenIdentical);
        sb.append(",\"determinize_ms\":").append(String.format("%.2f", detMs)).append(",\"determinize\":").append(q(det.toString()));
        sb.append(",\"public_unchanged_by_determinize\":").append(publicUnchanged);
        sb.append(",\"infoset_invariant\":").append(invariant);
        sb.append(",\"legal_sas\":").append(sas.size()).append(",\"sim_ok\":").append(simOk).append(",\"sim_fail\":").append(simFail);
        sb.append(",\"sim_ms_avg\":").append(String.format("%.1f", (simOk + simFail) == 0 ? 0 : simMs / (simOk + simFail)));
        sb.append(",\"sim_err\":").append(q(String.join(" ; ", simErr))).append("}");
        emit(sb.toString());

        if (prune) pruneProbe(g, me, opp, ph, turn);

        // 7. heap per copy and parallel copy, once per game at turn >= 4
        if (turn >= 4 && memDone.add(game)) heapAndParallel(g);
    }

    static String stackDesc(Game g) {
        StringBuilder b = new StringBuilder();
        for (forge.game.spellability.SpellAbilityStackInstance si : g.getStack()) b.append(si.getSpellAbility().getHostCard().getName()).append('{').append(si.getSpellAbility().getActivatingPlayer().getId()).append('}');
        return b.toString();
    }

    /** Fork while something is on the stack: is the stack copied, and does resolving it in the fork behave? */
    static void stackProbe(Game g) {
        int n = g.getStack().size();
        String orig = stackDesc(g);
        Player caster = g.getStack().peekAbility().getActivatingPlayer();
        boolean old = GameSimulator.COPY_STACK;
        StringBuilder out = new StringBuilder("{\"kind\":\"stack_probe\",\"game\":" + game + ",\"turn\":" + g.getPhaseHandler().getTurn() + ",\"orig_stack\":" + q(orig));
        try {
            for (boolean cs : new boolean[]{false, true}) {
                GameSimulator.COPY_STACK = cs;
                GameCopier gc = new GameCopier(g);
                Game c = gc.makeCopy();
                out.append(",\"copy_stack_").append(cs).append("\":").append(q(stackDesc(c)));
                out.append(",\"stack_zone_cards_").append(cs).append("\":").append(c.getStackZone().size());
                if (cs) {
                    List<String> before = digest(c, true);
                    String err = "";
                    try { GameSimulator.resolveStack(c, gc.find(caster.getWeakestOpponent())); } catch (Throwable t) { err = t.toString(); }
                    List<String> after = digest(c, true);
                    out.append(",\"resolve_err\":").append(q(err)).append(",\"stack_after_resolve\":").append(c.getStack().size())
                       .append(",\"changed_lines\":").append(diff(before, after).size());
                }
            }
        } finally { GameSimulator.COPY_STACK = old; }
        emit(out.append(",\"orig_size\":").append(n).append("}").toString());
    }

    static int named(Player p, ZoneType z, String name, boolean eq) {
        int n = 0; for (Card c : p.getCardsIn(z)) if ((c.getName().equals(name) || c.getName().isEmpty()) == eq) n++; return n;
    }

    /** Pipeline for by-construction hiding: copy with PRUNE_HIDDEN_INFO (hidden cards never rebuilt), fill placeholders with a sampled world, simulate on that world. */
    static void pruneProbe(Game g, Player me, Player opp, PhaseType ph, int turn) throws Exception {
        long[] ns = new long[reps]; Game w = null; GameCopier gc = null;
        for (int i = 0; i <= reps; i++) {
            long t = System.nanoTime(); gc = new GameCopier(g); w = gc.makeCopy(null, me);
            if (i > 0) ns[i - 1] = System.nanoTime() - t;
        }
        Arrays.sort(ns);
        if (debugOnce) { debugOnce = false;
            Field f = GameCopier.class.getDeclaredField("PRUNE_HIDDEN_INFO"); f.setAccessible(true);
            StringBuilder dbg = new StringBuilder("flag=" + f.get(null) + " ");
            for (Card c : opp.getCardsIn(ZoneType.Hand)) dbg.append(c.getName()).append(":zone=").append(c.getView().getZone()).append(":shown=").append(c.getView().canBeShownTo(me.getView())).append(' ');
            emit("{\"kind\":\"debug\",\"msg\":" + q(dbg.toString()) + "}"); }
        Player meW = gc.find(me), oppW = gc.find(opp);
        if (debugNames) { debugNames = false; StringBuilder b = new StringBuilder();
            for (Card c : oppW.getCardsIn(ZoneType.Hand)) b.append("[").append(c.getName()).append("|id=").append(c.getId()).append("] ");
            emit("{\"kind\":\"debug\",\"msg\":" + q("copy opp hand: " + b) + "}"); }
        int oppHidden = oppW.getZone(ZoneType.Hand).size() + oppW.getZone(ZoneType.Library).size();
        int oppPlaceholders = named(oppW, ZoneType.Hand, "hidden", true) + named(oppW, ZoneType.Library, "hidden", true);
        int myLibPlaceholders = named(meW, ZoneType.Library, "hidden", true), myLib = meW.getZone(ZoneType.Library).size();
        int myHandReal = named(meW, ZoneType.Hand, "hidden", false), myHand = meW.getZone(ZoneType.Hand).size();
        long t = System.nanoTime();
        determinize(w, oppW, opp.getRegisteredPlayer().getDeck(), new Random(seed + turn), true);
        determinize(w, meW, me.getRegisteredPlayer().getDeck(), new Random(seed + 7 * turn), false);
        double fillMs = (System.nanoTime() - t) / 1e6;
        int stillHidden = named(oppW, ZoneType.Hand, "hidden", true) + named(oppW, ZoneType.Library, "hidden", true) + named(meW, ZoneType.Library, "hidden", true);
        // simulate on the sampled world
        List<SpellAbility> sas = new ArrayList<>();
        try { for (SpellAbility sa : ComputerUtilAbility.getSpellAbilities(ComputerUtilAbility.getAvailableCards(w, meW), meW))
            if (sa.canPlay() && forge.ai.ComputerUtilMana.canPayManaCost(sa, meW, 0, false)) sas.add(sa); } catch (Throwable t2) { }
        int ok = 0, fail = 0; List<String> errs = new ArrayList<>(); double ms = 0;
        for (int i = 0; i < Math.min(maxSims, sas.size()); i++) {
            long ts = System.nanoTime();
            try {
                GameStateEvaluator.Score sc = new forge.ai.simulation.SpellAbilityPicker(meW)
                        .evaluateSa(new SimulationController(new GameStateEvaluator.Score(0), 0), null, sas, i);
                if (sc.value == Integer.MIN_VALUE) { fail++; errs.add(sas.get(i).getHostCard() + " -> MIN_VALUE"); } else ok++;
            } catch (Throwable t3) { fail++; errs.add(sas.get(i).getHostCard() + " -> " + t3); }
            ms += (System.nanoTime() - ts) / 1e6;
        }
        emit("{\"kind\":\"prune_probe\",\"game\":" + game + ",\"turn\":" + turn + ",\"phase\":\"" + ph + "\",\"prune_copy_ms_median\":" + String.format("%.2f", ns[reps / 2] / 1e6)
            + ",\"opp_hidden\":" + oppHidden + ",\"opp_placeholders\":" + oppPlaceholders + ",\"my_lib\":" + myLib + ",\"my_lib_placeholders\":" + myLibPlaceholders
            + ",\"my_hand\":" + myHand + ",\"my_hand_real\":" + myHandReal + ",\"fill_ms\":" + String.format("%.2f", fillMs) + ",\"still_hidden_after_fill\":" + stillHidden
            + ",\"sims\":" + (ok + fail) + ",\"sim_fail\":" + fail + ",\"sim_ms_avg\":" + String.format("%.1f", (ok + fail) == 0 ? 0 : ms / (ok + fail)) + ",\"sim_err\":" + q(String.join(" ; ", errs)) + "}");
    }

    static void heapAndParallel(Game g) throws Exception {
        System.gc(); Thread.sleep(200); System.gc();
        Runtime rt = Runtime.getRuntime();
        long before = rt.totalMemory() - rt.freeMemory();
        List<Game> keep = new ArrayList<>();
        for (int i = 0; i < 20; i++) keep.add(new GameCopier(g).makeCopy());
        System.gc(); Thread.sleep(200); System.gc();
        long after = rt.totalMemory() - rt.freeMemory();
        double mbPer = (after - before) / 20.0 / 1048576.0;
        keep.clear();
        // parallel: 4 threads x 10 copies of the same paused game
        int threads = 4, per = 10;
        ExecutorService ex = Executors.newFixedThreadPool(threads);
        List<Future<long[]>> fs = new ArrayList<>();
        long t0 = System.nanoTime();
        for (int i = 0; i < threads; i++) fs.add(ex.submit(() -> {
            long fails = 0;
            for (int k = 0; k < per; k++) { try { new GameCopier(g).makeCopy(); } catch (Throwable t) { fails++; } }
            return new long[]{fails};
        }));
        long fails = 0; for (Future<long[]> f : fs) fails += f.get()[0];
        double wall = (System.nanoTime() - t0) / 1e6;
        ex.shutdown();
        emit("{\"kind\":\"heap_parallel\",\"game\":" + game + ",\"heap_mb_per_copy\":" + String.format("%.1f", mbPer)
                + ",\"parallel_threads\":" + threads + ",\"parallel_copies\":" + threads * per + ",\"parallel_wall_ms\":" + String.format("%.0f", wall)
                + ",\"parallel_copies_per_sec\":" + String.format("%.1f", threads * per / (wall / 1000.0)) + ",\"parallel_failures\":" + fails + "}");
    }

    static String hiddenSig(Player p) {
        StringBuilder sb = new StringBuilder();
        for (Card c : p.getCardsIn(ZoneType.Hand)) sb.append(c.getId()).append(':').append(c.getName()).append(',');
        sb.append('|');
        for (Card c : p.getCardsIn(ZoneType.Library)) sb.append(c.getId()).append(':').append(c.getName()).append(',');
        return sb.toString();
    }

    static List<String> digest(Game g, boolean includeHands) {
        List<String> out = new ArrayList<>();
        out.add("G turn=" + g.getPhaseHandler().getTurn() + " phase=" + g.getPhaseHandler().getPhase() + " active=" + g.getPhaseHandler().getPlayerTurn().getId() + " stack=" + g.getStack().size() + " stackCastThisTurn=" + g.getStack().getSpellsCastThisTurn().size());
        for (Player p : g.getPlayers()) {
            out.add("P" + p.getId() + " life=" + p.getLife() + " lands=" + p.getLandsPlayedThisTurn() + " hand=" + p.getZone(ZoneType.Hand).size() + " lib=" + p.getZone(ZoneType.Library).size() + " mana=" + p.getManaPool().totalMana() + " spellsThisTurn=" + p.getSpellsCastThisTurn());
            for (ZoneType z : new ZoneType[]{ZoneType.Battlefield, ZoneType.Graveyard, ZoneType.Exile, ZoneType.Hand, ZoneType.Command}) {
                if (z == ZoneType.Hand && !includeHands) continue;
                for (Card c : p.getCardsIn(z)) {
                    StringBuilder s = new StringBuilder("P" + p.getId() + " " + z + " " + c.getId() + ":" + c.getName());
                    if (z == ZoneType.Battlefield) {
                        s.append(" T=").append(c.isTapped()).append(" dmg=").append(c.getDamage()).append(" sick=").append(c.hasSickness());
                        if (c.isCreature()) s.append(" pt=").append(c.getNetPower()).append('/').append(c.getNetToughness());
                        s.append(" ctrl=").append(c.getController().getId());
                        TreeMap<String, Integer> cn = new TreeMap<>();
                        for (com.google.common.collect.Multiset.Entry<?> en : c.getCounters().entrySet()) cn.put(en.getElement().toString(), en.getCount());
                        s.append(" counters=").append(cn);
                        if (c.isAttachedToEntity()) s.append(" att=").append(c.getEntityAttachedTo());
                    }
                    out.add(s.toString());
                }
            }
        }
        Collections.sort(out);
        return out;
    }

    static List<String> diff(List<String> a, List<String> b) {
        List<String> d = new ArrayList<>();
        Set<String> sb = new HashSet<>(b), sa = new HashSet<>(a);
        for (String s : a) if (!sb.contains(s)) d.add("-" + s);
        for (String s : b) if (!sa.contains(s)) d.add("+" + s);
        return d;
    }

    static List<String> publicDigest(List<String> dg, Player opp) {
        List<String> r = new ArrayList<>();
        for (String s : dg) {
            if (s.startsWith("P" + opp.getId() + " Hand ")) continue;
            if (s.startsWith("P" + opp.getId() + " ")) s = s.replaceAll(" hand=\\d+ lib=\\d+", ""); // sizes are public and checked elsewhere
            r.add(s);
        }
        return r;
    }

    /** Replace the opponent's hand+library in the (already copied) game with a random world consistent with deck minus known cards. */
    static Map<String, Object> determinize(Game copy, Player oppCopy, Deck oppDeck, Random rnd) { return determinize(copy, oppCopy, oppDeck, rnd, true); }
    static Map<String, Object> determinize(Game copy, Player oppCopy, Deck oppDeck, Random rnd, boolean hideHand) {
        Map<String, Object> info = new LinkedHashMap<>();
        Zone hand = oppCopy.getZone(ZoneType.Hand), lib = oppCopy.getZone(ZoneType.Library);
        int h = hideHand ? hand.size() : 0, l = lib.size();
        Map<String, Integer> pool = new TreeMap<>();
        Map<String, PaperCard> pc = new HashMap<>();
        for (Map.Entry<PaperCard, Integer> e : oppDeck.getMain()) { pool.merge(e.getKey().getName(), e.getValue(), Integer::sum); pc.put(e.getKey().getName(), e.getKey()); }
        // known = anything of the opponent's that is in a public zone (owned by opp)
        int knownCnt = 0;
        for (ZoneType z : new ZoneType[]{ZoneType.Battlefield, ZoneType.Graveyard, ZoneType.Exile, ZoneType.Stack, ZoneType.Command}) {
            for (Card c : copy.getCardsIn(z)) {
                if (!c.getOwner().equals(oppCopy) || c.isToken() || c.isImmutable()) continue;
                pool.merge(c.getPaperCard() != null ? c.getPaperCard().getName() : c.getName(), -1, Integer::sum); knownCnt++;
            }
        }
        if (!hideHand) for (Card c : hand.getCards()) pool.merge(c.getPaperCard() != null ? c.getPaperCard().getName() : c.getName(), -1, Integer::sum);
        List<String> unknown = new ArrayList<>();
        for (Map.Entry<String, Integer> e : pool.entrySet()) for (int i = 0; i < Math.max(0, e.getValue()); i++) unknown.add(e.getKey());
        info.put("hand", h); info.put("lib", l); info.put("pool", unknown.size()); info.put("known_public", knownCnt);
        info.put("size_consistent", unknown.size() == h + l);
        { List<String> neg = new ArrayList<>(); for (Map.Entry<String, Integer> e : pool.entrySet()) if (e.getValue() < 0) neg.add(e.getKey() + "=" + e.getValue()); if (!neg.isEmpty()) info.put("over_subtracted", neg); }
        if (hideHand) for (Card c : new ArrayList<Card>(hand.getCards())) hand.remove(c);
        for (Card c : new ArrayList<Card>(lib.getCards())) lib.remove(c);
        Collections.shuffle(unknown, rnd);
        int n = Math.min(unknown.size(), h + l);
        for (int i = 0; i < n; i++) {
            Card nc = CardFactory.getCard(pc.get(unknown.get(i)), oppCopy, copy.nextCardId(), copy);
            (i < h ? hand : lib).add(nc);
        }
        info.put("hand_after", hand.size()); info.put("lib_after", lib.size());
        return info;
    }

    /** Take a real game, make world B = copy with the opponent's hand and library re-dealt (same info set, different hidden contents);
     *  determinize copies of both with the same RNG; digests must be identical. */
    static boolean infoSetInvariant(Game g, Player me, Player opp, Deck oppDeck) {
        GameCopier c1 = new GameCopier(g); Game a = c1.makeCopy(); Player oppA = c1.find(opp);
        GameCopier c2 = new GameCopier(g); Game b = c2.makeCopy(); Player oppB = c2.find(opp);
        // re-deal B's hidden zones: swap contents of hand and the first |hand| library cards where possible, then reverse the library
        Zone hb = oppB.getZone(ZoneType.Hand), lb = oppB.getZone(ZoneType.Library);
        List<Card> all = new ArrayList<>(); all.addAll(hb.getCards()); all.addAll(lb.getCards());
        int h = hb.size();
        for (Card c : all) { if (hb.contains(c)) hb.remove(c); else lb.remove(c); }
        Collections.reverse(all);
        for (int i = 0; i < all.size(); i++) (i < h ? hb : lb).add(all.get(i));
        determinize(a, oppA, oppDeck, new Random(99));
        determinize(b, oppB, oppDeck, new Random(99));
        return digest(a, true).equals(digest(b, true));
    }
}
