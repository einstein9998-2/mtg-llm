package itest;

import com.google.gson.*;
import forge.GuiDesktop;
import forge.LobbyPlayer;
import forge.gui.GuiBase;
import forge.model.FModel;
import forge.ai.*;
import forge.deck.Deck;
import forge.game.*;
import forge.game.card.*;
import forge.game.combat.Combat;
import forge.game.cost.*;
import forge.game.ability.AbilityUtils;
import forge.game.ability.ApiType;
import forge.game.ability.effects.CharmEffect;
import forge.game.zone.Zone;
import forge.game.phase.PhaseType;
import forge.game.player.*;
import forge.game.spellability.*;
import forge.game.zone.ZoneType;
import forge.player.GamePlayerUtil;
import forge.util.IterableUtil;
import forge.util.MyRandom;

import java.io.*;
import java.nio.file.*;
import java.util.*;
import java.util.stream.*;

/**
 * Action-set differential, Forge side. Reads positions exported by the Rust engine (JSONL: Forge GameState lines),
 * injects each into a fresh Forge game, and records Forge's own legal priority menu (the Phase 1 enumeration:
 * getAllPossibleAbilities + canPlay + affordability + target availability) as canonical keys, plus a state summary
 * read back from Forge to verify the injection.
 */
public final class Probe {
    static final class Done extends RuntimeException { Done() { super("done", null, false, false); } }

    // ---- step mode: the position's action is played by the active/priority seat, the opponent passes, and every other decision is made by
    // the stock AI and logged so the Rust follower can repeat it.
    static String ACTION = null;           // canonical key of the action to play, null = menu-only probe
    static boolean ACTED = false, ACT_FAILED = false;
    static boolean COMBAT = false;
    static String RESPONSE = null, ACTOR = null;   // stack response: canonical key of the opponent's answer, and who played the position's action
    static boolean RESPONDED = false;
    // ---- line mode (lockstep): at every priority window both seats choose by a shared deterministic policy (a pass with an empty stack ends the line);
    // each window records the seat, stack size, canonical menu, chosen key, state summary and log offset so the Rust follower can replay the line
    static boolean LINE = false;
    static long LINE_SEED = 0;
    static int LINE_MAX = 12;
    static Set<String> FOCUS = new HashSet<>();
    static final Set<String> SHUFFLERS = Set.of("Flooded Strand", "Misty Rainforest", "Polluted Delta", "Scalding Tarn", "Verdant Catacombs", "Bloodstained Mire", "Marsh Flats", "Arid Mesa", "Windswept Heath", "Prismatic Vista", "Personal Tutor", "Edge of Autumn", "Boseiju, Who Endures", "Brainstorm-never");
    static int CALLS = 0, ABILITY_INDEX = -1;
    static final List<String> LOG = new ArrayList<>();
    static final Set<String> NOISE = Set.of("payManaCost", "reveal", "cheatShuffle", "orderSimultaneousSa", "playSpellAbilityNoStack", "playSaFromPlayEffect", "notifyOfValue", "revealAnte", "declareAttackers", "declareBlockers", "playChosenSpellAbility", "orderBlockers", "orderBlocker", "orderAttackers", "autoPassCancel", "awaitNextInput", "cancelAwaitNextInput", "payCombatCost", "payManaOptionalCost", "chooseManaFromPool", "playMana", "getAbilityToPlay", "sideboard");

    static String fmt(Object o) {
        if (o == null) return "null";
        if (o instanceof Card c) return c.getName();
        if (o instanceof Player p) return p.getName();
        if (o instanceof forge.game.trigger.WrappedAbility w) return fmt(w.getWrappedAbility());
        if (o instanceof SpellAbility sa) {
            StringBuilder sb = new StringBuilder(sa.getHostCard() == null ? "?" : sa.getHostCard().getName());
            if (sa.getTargets() != null && (sa.getTargets().isTargetingAnyCard() || sa.getTargets().isTargetingAnyPlayer() || sa.getTargets().isTargetingAnySpell())) {
                sb.append("{"); boolean f = true;
                for (Card t : sa.getTargets().getTargetCards()) { if (!f) sb.append(";"); f = false; sb.append(Seat.label(t)).append(t.isTapped() ? "|T" : ""); }
                for (Player t : sa.getTargets().getTargetPlayers()) { if (!f) sb.append(";"); f = false; sb.append(fmt(t)); }
                for (SpellAbility t : sa.getTargets().getTargetSpells()) { if (!f) sb.append(";"); f = false; sb.append("stack:").append(fmt(t)); }
                sb.append("}");
            }
            return sb.toString();
        }
        if (o instanceof org.apache.commons.lang3.tuple.Pair<?, ?> pr) return "(" + fmt(pr.getLeft()) + "," + fmt(pr.getRight()) + ")";
        if (o instanceof Iterable<?> it) { StringBuilder sb = new StringBuilder("["); boolean f = true; for (Object x : it) { if (!f) sb.append(";"); f = false; sb.append(fmt(x)); } return sb.append("]").toString(); }
        if (o instanceof Map<?, ?> m) { StringBuilder sb = new StringBuilder("{"); boolean f = true; for (var e : m.entrySet()) { if (!f) sb.append(";"); f = false; sb.append(fmt(e.getKey())).append("=").append(fmt(e.getValue())); } return sb.append("}").toString(); }
        String t = String.valueOf(o);
        return t.length() > 120 ? t.substring(0, 120) : t;
    }
    static void log(Player p, String method, Object[] args, Object ret) {
        if ((ACTION == null && !LINE) || !ACTED || QUIET) return;
        if (NOISE.contains(method)) return;
        StringBuilder sb = new StringBuilder();
        sb.append(p.getName()).append(" ").append(method).append(" ");
        for (int i = 0; i < args.length; i++) { if (i > 0) sb.append(" | "); sb.append(fmt(args[i])); }
        sb.append(" => ").append(fmt(ret));
        LOG.add(sb.toString());
    }

    static final class Rec {
        List<String> actions = new ArrayList<>(); List<String> summary = new ArrayList<>(); List<String> notes = new ArrayList<>();
        int stack; String error; PhaseType phase; String activeName;
        List<JsonObject> windows = new ArrayList<>(); boolean over; List<String> finalSummary; String winner = "";
    }

    static final class Seat extends LogAi {
        final Rec rec; final int me; final Game[] gameRef;
        static int WINDOW = 0;
        Seat(Game g, Player p, LobbyPlayer lp, Rec rec, int me, Game[] gameRef) { super(g, p, lp); this.rec = rec; this.me = me; this.gameRef = gameRef; }

        @Override public List<SpellAbility> chooseSpellAbilityToPlay() {
            Game game = gameRef[0];
            if (LINE) return linePriority();
            if (ACTION != null) return stepPriority();
            if (WINDOW == 1 && game.getPhaseHandler().getPlayerTurn() == player) return null; // the active player passes; the probe reads the opponent's window
            rec.stack = game.getStack().size();
            rec.phase = game.getPhaseHandler().getPhase();
            rec.activeName = player.getName();
            rec.actions = quiet(() -> menu());
            rec.summary = summary(game);
            throw new Done();
        }

        boolean canAfford(SpellAbility sa) {
            return ComputerUtilMana.canPayManaCost(sa, player, 0, false) && CostPayment.canPayAdditionalCosts(sa.getPayCosts(), sa, false, player);
        }
        boolean targetsAvailable(SpellAbility sa) {
            // an X cost chosen before targets (Lazotep Quarry): the candidate count depends on X, so accept any X that works
            boolean hasX = sa.getPayCosts() != null && sa.getPayCosts().hasXInAnyCostPart();
            for (int x = 0; x <= (hasX ? 8 : 0); x++) {
                if (hasX) sa.setXManaCostPaid(x);
                boolean ok = true;
                for (SpellAbility s = sa; s != null; s = s.getSubAbility()) {
                    if (!s.usesTargeting()) continue;
                    int min = s.getMinTargets();
                    if (min > 0 && s.getTargetRestrictions().getNumCandidates(s) < min) { ok = false; break; }
                }
                if (ok && hasX && !ComputerUtilMana.canPayManaCost(sa, player, x, false)) ok = false;
                if (ok) { if (hasX) sa.setXManaCostPaid(0); return true; }
            }
            if (hasX) sa.setXManaCostPaid(0);
            return false;
        }

        Map<String, List<SpellAbility>> menuMap() {
            Game game = gameRef[0];
            CardCollection all = new CardCollection(player.getCardsIn(ZoneType.Hand));
            all.addAll(player.getCardsIn(ZoneType.Graveyard));
            all.addAll(IterableUtil.filter(player.getCardsIn(ZoneType.Command), c -> !c.isImmutable() || c.isEmblem()));
            all.addAll(game.getCardsIn(ZoneType.Exile));
            all.addAll(game.getCardsIn(ZoneType.Battlefield));
            Map<String, List<SpellAbility>> keys = new TreeMap<>();
            Map<String, Map<String, SpellAbility>> acts = new TreeMap<>();
            for (Card c : all) {
                for (SpellAbility sa : c.getAllPossibleAbilities(player, false)) {
                    if (sa.isManaAbility()) continue;
                    sa.setActivatingPlayer(player);
                    if (!sa.canPlay()) continue;
                    if (!canAfford(sa)) continue;
                    if (!targetsAvailable(sa)) continue;
                    String z = zone(c);
                    if (sa.isLandAbility()) keys.computeIfAbsent("play|" + z + "|" + c.getName() + "|", k -> new ArrayList<>()).add(sa);
                    else if (sa.isSpell()) keys.computeIfAbsent("cast|" + z + "|" + c.getName() + "|" + (sa.isBasicSpell() ? "" : "alt"), k -> new ArrayList<>()).add(sa);
                    else acts.computeIfAbsent(z + "|" + c.getName(), k -> new TreeMap<>()).putIfAbsent(sa.toString(), sa);
                }
            }
            for (var e : acts.entrySet()) keys.put("act|" + e.getKey() + "|" + e.getValue().size(), new ArrayList<>(e.getValue().values()));
            return keys;
        }

        List<String> menu() { return new ArrayList<>(menuMap().keySet()); }


        static long mix(long x) { x ^= x >>> 33; x *= 0xff51afd7ed558ccdL; x ^= x >>> 33; x *= 0xc4ceb9fe1a85ec53L; x ^= x >>> 33; return x; }
        static String cardOf(String key) { String[] p = key.split("\\|"); return p.length > 2 ? p[2] : ""; }

        /** The shared policy: deterministic in (line seed, window index, menu); null = pass. */
        String linePolicy(List<String> keys, int stack, int idx) {
            List<String> cand = new ArrayList<>();
            for (String k : keys) {
                if (k.startsWith("playback|") || k.startsWith("cast|Exile|") || k.startsWith("cast|Graveyard|Nethergoyf|")) continue;
                // actions that shuffle a library: the two engines shuffle differently, so lines avoid them (single steps cover them)
                if (SHUFFLERS.contains(cardOf(k)) && (k.startsWith("act|") || k.startsWith("cast|"))) continue;
                if (k.startsWith("act|") && !k.endsWith("|1")) continue;
                // Forge offers a modal double-faced card's land face at instant speed under Aluren (a land is not a spell, so Aluren cannot allow it): keep lines legal
                if (stack > 0 && k.startsWith("play|")) continue;
                cand.add(k);
            }
            List<String> foc = new ArrayList<>();
            for (String k : cand) if (FOCUS.contains(cardOf(k))) foc.add(k);
            long h = mix(LINE_SEED * 1000003L + idx * 7919L + String.join("\u0001", keys).hashCode());
            int passPct = stack > 0 ? 50 : (foc.isEmpty() ? 18 : 5);
            if (cand.isEmpty() || (int) ((h >>> 8) % 100) < passPct) return null;
            List<String> pool = (!foc.isEmpty() && (int) ((h >>> 20) % 100) < 75) ? foc : cand;
            return pool.get((int) ((h >>> 32) % pool.size()));
        }

        List<SpellAbility> linePriority() {
            Game game = gameRef[0];
            if (++CALLS > 600) throw new IllegalStateException("too many priority calls");
            if (!ACTED) {
                if (WINDOW == 1 && game.getPhaseHandler().getPlayerTurn() == player) return null;
                ACTED = true;
            }
            if (ACT_FAILED && !rec.windows.isEmpty()) { rec.windows.get(rec.windows.size() - 1).addProperty("failed", true); throw new Done(); }
            Map<String, List<SpellAbility>> m = quiet(this::menuMap);
            List<String> keys = new ArrayList<>(m.keySet());
            Gson gs = new Gson();
            JsonObject w = new JsonObject();
            w.addProperty("seat", player.getName());
            w.addProperty("stack", game.getStack().size());
            w.addProperty("phase", String.valueOf(game.getPhaseHandler().getPhase()));
            w.add("keys", gs.toJsonTree(keys));
            w.add("summary", gs.toJsonTree(summary(game)));
            { List<String> lo = new ArrayList<>(); for (int i = 0; i < 2; i++) lo.add(String.join(";", names(game.getRegisteredPlayers().get(i).getCardsIn(ZoneType.Library), false))); w.add("lib", gs.toJsonTree(lo)); }
            w.addProperty("log_at", LOG.size());
            int idx = rec.windows.size();
            rec.windows.add(w);
            if (idx >= LINE_MAX) { w.addProperty("chosen", "END"); throw new Done(); }
            String choice = linePolicy(keys, game.getStack().size(), idx);
            if (choice == null) {
                w.addProperty("chosen", "PASS");
                if (game.getStack().isEmpty()) throw new Done();
                return null;
            }
            w.addProperty("chosen", choice);
            List<SpellAbility> l = m.get(choice);
            SpellAbility sa = l.get(0);
            ACTOR = player.getName();
            w.addProperty("played", sa.getHostCard().getName() + " [" + sa + "]");
            return List.of(sa);
        }

        /** Step mode priority: play the position's action once, then pass until the stack is empty again. */
        List<SpellAbility> stepPriority() {
            Game game = gameRef[0];
            if (++CALLS > 300) throw new IllegalStateException("too many priority calls");
            if (COMBAT && ACTED) {
                // a combat run ends at the first empty-stack priority of the second main phase
                if (game.isGameOver() || (game.getPhaseHandler().getPhase() == PhaseType.MAIN2 && game.getStack().isEmpty())) {
                    rec.stack = game.getStack().size();
                    rec.phase = game.getPhaseHandler().getPhase();
                    rec.summary = summary(game);
                    throw new Done();
                }
                return null;
            }
            if (!ACTED) {
                if (WINDOW == 1 && game.getPhaseHandler().getPlayerTurn() == player) return null;
                rec.activeName = player.getName();
                if (COMBAT) { ACTED = true; rec.notes.add("combat run"); return null; }
                Map<String, List<SpellAbility>> m = quiet(this::menuMap);
                String want = ACTION; List<SpellAbility> l = m.get(want);
                if (l == null && want.startsWith("act|")) { String pre = want.substring(0, want.lastIndexOf('|') + 1); for (var e : m.entrySet()) if (e.getKey().startsWith(pre)) l = e.getValue(); }
                if (l == null) { rec.error = "action not offered by Forge: " + want; rec.actions = new ArrayList<>(m.keySet()); throw new Done(); }
                SpellAbility sa = l.get(0);
                if (want.startsWith("act|") && ABILITY_INDEX >= 0 && ABILITY_INDEX < l.size()) sa = l.get(ABILITY_INDEX);
                ACTED = true;
                ACTOR = player.getName();
                rec.notes.add("played " + sa.getHostCard().getName() + " [" + sa + "]");
                return List.of(sa);
            }
            if (game.getStack().isEmpty() || game.isGameOver()) {
                rec.stack = game.getStack().size();
                rec.phase = game.getPhaseHandler().getPhase();
                rec.summary = summary(game);
                throw new Done();
            }
            if (RESPONSE != null && !RESPONDED && !player.getName().equals(ACTOR)) {
                // the opponent's first window with the action on the stack: play the position's response
                RESPONDED = true;
                Map<String, List<SpellAbility>> m = quiet(this::menuMap);
                rec.actions = new ArrayList<>(m.keySet());
                List<SpellAbility> l = m.get(RESPONSE);
                if (l == null && RESPONSE.startsWith("act|")) { String pre = RESPONSE.substring(0, RESPONSE.lastIndexOf('|') + 1); for (var e : m.entrySet()) if (e.getKey().startsWith(pre)) l = e.getValue(); }
                if (l == null) { rec.error = "response not offered by Forge: " + RESPONSE; rec.actions = new ArrayList<>(m.keySet()); throw new Done(); }
                SpellAbility sa = l.get(0);
                rec.notes.add("responded with " + sa.getHostCard().getName() + " [" + sa + "]");
                return List.of(sa);
            }
            return null;
        }

        /** `Name{P1P1=2}`: the creature's name and counters, the identity combat specs use. */
        static String label(Card c) {
            List<String> cs = new ArrayList<>();
            for (var en : c.getCounters().entrySet()) if (en.getCount() > 0) cs.add(en.getElement().toString() + "=" + en.getCount());
            Collections.sort(cs);
            return c.getName() + "{" + String.join(",", cs) + "}";
        }

        /** Combat run: the active player attacks with exactly the creatures the position names (`combat|Name;Name`), at the opponent player. */
        @Override public void declareAttackers(Player attacker, Combat combat) {
            if (!COMBAT) { super.declareAttackers(attacker, combat); return; }
            Map<String, Integer> want = new TreeMap<>();
            for (String n : ACTION.substring("combat|".length()).split(";")) if (!n.isEmpty()) want.merge(n, 1, Integer::sum);
            GameEntity def = null;
            for (GameEntity d : combat.getDefenders()) if (d instanceof Player) def = d;
            List<Card> mine = new ArrayList<>(player.getCreaturesInPlay());
            mine.sort((x, y) -> x.getName().equals(y.getName()) ? Integer.compare(x.getId(), y.getId()) : x.getName().compareTo(y.getName()));
            for (Card c : mine) {
                Integer k = want.get(label(c));
                if (k == null || k <= 0 || def == null) continue;
                if (!forge.game.combat.CombatUtil.canAttack(c, def)) continue;
                combat.addAttacker(c, def);
                want.put(label(c), k - 1);
            }
            rec.notes.add("declared attackers " + fmt(combat.getAttackers()) + (want.values().stream().anyMatch(v -> v > 0) ? " (some requested attackers could not attack: " + want + ")" : ""));
        }

        /** The AI chooses blocks; log them per blocker so the Rust follower can mirror them. */
        @Override public void declareBlockers(Player defender, Combat combat) {
            super.declareBlockers(defender, combat);
            if (!COMBAT || !ACTED) return;
            for (Card a : combat.getAttackers()) for (Card b : combat.getBlockers(a)) LOG.add(player.getName() + " blockAssign " + label(b) + " => " + label(a));
        }

        /** Trigger targets are chosen by the stock AI while the trigger goes on the stack; log them per new stack item, in placement order. */
        @Override public void orderAndPlaySimultaneousSa(List<SpellAbility> sas) {
            if ((ACTION == null && !LINE) || !ACTED) { super.orderAndPlaySimultaneousSa(sas); return; }
            Set<Object> before = new HashSet<>();
            for (var si : gameRef[0].getStack()) before.add(si);
            super.orderAndPlaySimultaneousSa(sas);
            List<String> now = new ArrayList<>();
            for (var si : gameRef[0].getStack()) if (!before.contains(si)) now.add(fmt(si.getSpellAbility()));
            Collections.reverse(now); // the stack iterates top first; the first trigger placed is the deepest
            for (String t : now) LOG.add(player.getName() + " triggerTargets " + t + " => ok");
        }

        @Override public boolean playChosenSpellAbility(SpellAbility sa) {
            if (ACTION == null && !LINE) return super.playChosenSpellAbility(sa);
            if (sa.isLandAbility()) { if (sa.canPlay()) { sa.resolve(); } return true; }
            boolean ok = playCanonical(sa);
            if (!ok) { ACT_FAILED = true; rec.notes.add("play FAILED (cost or target)"); }
            return true;
        }

        /** Canonical order of target candidates, shared with the Rust follower: opponent player, own player, then objects by (name, controller). */
        String tkey(GameObject o) {
            if (o instanceof Player p) return "0|" + (p == player ? "1" : "0");
            if (o instanceof SpellAbility ts) return "1|" + ts.getHostCard().getName() + "|" + ctlIdx(ts.getHostCard());
            Card c = (Card) o; return "1|" + c.getName() + "|" + ctlIdx(c);
        }
        int ctlIdx(Card c) { Player ctl = c.getController() == null ? c.getOwner() : c.getController(); return ctl.getName().equals("P1") ? 0 : 1; }

        void canonTargets(SpellAbility root) {
            Game game = gameRef[0];
            for (SpellAbility cur = root; cur != null; cur = cur.getSubAbility()) {
                if (!cur.usesTargeting()) continue;
                TargetRestrictions tr = cur.getTargetRestrictions();
                List<GameObject> cands = new ArrayList<>();
                // spells on the stack are targeted through their SpellAbility, not through the card in the stack zone
                for (GameObject o : tr.getAllCandidates(cur)) if (!(o instanceof Card c && c.isInZone(ZoneType.Stack))) cands.add(o);
                if (tr.getZone().contains(ZoneType.Stack)) for (var si : game.getStack()) if (cur.canTargetSpellAbility(si.getSpellAbility())) cands.add(si.getSpellAbility());
                cands.sort(Comparator.comparing(this::tkey));
                int max = tr.getMaxTargets(cur.getHostCard(), cur);
                int min = tr.getMinTargets(cur.getHostCard(), cur);
                int n = 0;
                for (GameObject o : cands) {
                    if (n >= max) break;
                    if (!cur.canTarget(o)) continue;
                    cur.getTargets().add(o); n++;
                }
                rec.notes.add("targets[" + cur.getHostCard().getName() + "] min " + min + " max " + max + " -> " + fmt(cur));
            }
        }

        boolean playCanonical(SpellAbility sa) {
            final Game game = gameRef[0]; final Card source = sa.getHostCard();
            final Zone hz = source.isCopiedSpell() ? null : source.getZone();
            final ZoneType orig = hz == null ? null : hz.getZoneType();
            source.setSplitStateToPlayAbility(sa);
            if (sa.isSpell() && !source.isCopiedSpell()) {
                sa = AbilityUtils.addSpliceEffects(sa);
                sa.setHostCard(game.getAction().moveToStack(source, sa));
            }
            if (!sa.isCopied()) { sa.resetPaidHash(); sa.setPaidLife(0); }
            sa = GameActionUtil.addExtraKeywordCost(sa);
            boolean ok = true;
            if (!sa.checkRestrictions(player)) { rec.notes.add("checkRestrictions refused"); ok = false; }
            if (ok && sa.getApi() == ApiType.Charm && !CharmEffect.makeChoices(sa)) ok = false;
            if (ok) { canonTargets(sa); if (!sa.isTargetNumberValid()) ok = false; }
            if (ok) {
                game.getStack().freezeStack(sa);
                int lifeBefore = player.getLife();
                List<Card> untappedBefore = new ArrayList<>();
                for (Card pc : player.getCardsIn(ZoneType.Battlefield)) if (!pc.isTapped()) untappedBefore.add(pc);
                if (new CostPayment(sa.getPayCosts(), sa).payComputerCosts(new CanonCost(player, sa, false, this))) {
                    { List<String> paid = new ArrayList<>(); for (Card pc : untappedBefore) if (!pc.isInZone(ZoneType.Battlefield) || pc.isTapped()) paid.add(pc.getName()); Collections.sort(paid); if (!paid.isEmpty()) LOG.add("PAID " + String.join(";", paid)); }
                    if (sa.getPayCosts() != null && sa.getPayCosts().hasManaCost() && sa.getPayCosts().getCostMana().getMana().hasPhyrexian()) LOG.add("PHY " + (lifeBefore - player.getLife()));
                    game.getStack().addAndUnfreeze(sa);
                    if (sa.getPayCosts() != null && sa.getPayCosts().hasXInAnyCostPart()) LOG.add("X " + sa.getXManaCostPaid());
                    return true;
                }
                game.getStack().unfreezeStack();
            }
            sa.setSkip(true);
            if (orig != null && sa.getHostCard().isInZone(ZoneType.Stack)) game.getAction().moveTo(orig, sa.getHostCard(), null, null);
            return false;
        }

        @Override public List<AbilitySub> chooseModeForAbility(SpellAbility sa, List<AbilitySub> possible, int min, int num, boolean allowRepeat) {
            if (ACTION == null && !LINE) return super.chooseModeForAbility(sa, possible, min, num, allowRepeat);
            return new ArrayList<>(possible.subList(0, Math.min(Math.max(min, 1), possible.size())));
        }
    }

    /** Cost choices (pitch, discard, sacrifice, return) are canonical: smallest card name first. The Rust follower applies the same rule. */
    static final class CanonCost extends AiCostDecision {
        final Seat seat;
        CanonCost(Player p, SpellAbility sa, boolean effect, Seat seat) { super(p, sa, effect); this.seat = seat; }
        PaymentDecision canon(String kind, CardCollectionView valid, int n) {
            List<Card> l = new ArrayList<>(valid); l.sort(Comparator.comparing(Card::getName));
            if (l.size() < n) return null;
            List<Card> r = new ArrayList<>(l.subList(0, n));
            seat.rec.notes.add(kind + " -> " + fmt(r));
            return PaymentDecision.card(new CardCollection(r));
        }
        @Override public PaymentDecision visit(CostExile cost) {
            if (!cost.payCostFromSource() && !cost.getType().equals("All") && !cost.getType().contains("FromTopGrave") && !cost.getType().contains("+withTypesGE") && cost.zoneRestriction != 0 && !cost.getFrom().contains(ZoneType.Library)) {
                CardCollection valid = CardLists.getValidCards(player.getCardsIn(cost.getFrom()), cost.getType(), player, source, ability);
                PaymentDecision d = canon("exile-cost", valid, Math.min(cost.getAbilityAmount(ability), valid.size())); if (d != null) return d;
            }
            return super.visit(cost);
        }
        @Override public PaymentDecision visit(CostDiscard cost) {
            if (!cost.payCostFromSource() && !cost.getType().equals("Hand") && !cost.getType().equals("LastDrawn") && !cost.getType().equals("Random") && !cost.getType().contains("With")) {
                CardCollection valid = CardLists.getValidCards(player.getCardsIn(ZoneType.Hand), cost.getType().split(";"), player, source, ability);
                PaymentDecision d = canon("discard-cost", valid, Math.min(cost.getAbilityAmount(ability), valid.size())); if (d != null) return d;
            }
            return super.visit(cost);
        }
        @Override public PaymentDecision visit(CostReturn cost) {
            if (!cost.payCostFromSource()) {
                CardCollection valid = CardLists.getValidCards(player.getCardsIn(ZoneType.Battlefield), cost.getType(), player, source, ability);
                PaymentDecision d = canon("return-cost", valid, Math.min(cost.getAbilityAmount(ability), valid.size())); if (d != null) return d;
            }
            return super.visit(cost);
        }
        @Override public PaymentDecision visit(CostSacrifice cost) {
            if (!cost.payCostFromSource() && !cost.getType().equals("OriginalHost") && !cost.getAmount().equals("All")) {
                CardCollection valid = CardLists.getValidCards(player.getCardsIn(ZoneType.Battlefield), cost.getType(), player, source, ability);
                PaymentDecision d = canon("sacrifice-cost", valid, Math.min(cost.getAbilityAmount(ability), valid.size())); if (d != null) return d;
            }
            return super.visit(cost);
        }
    }

    static String zone(Card c) { return c.getZone() == null ? "?" : switch (c.getZone().getZoneType()) {
        case Hand -> "Hand"; case Battlefield -> "Battlefield"; case Graveyard -> "Graveyard"; case Exile -> "Exile"; case Command -> "Command"; default -> c.getZone().getZoneType().toString(); }; }

    static boolean QUIET = false;
    static <T> T quiet(java.util.function.Supplier<T> q) {
        Random saved = MyRandom.getRandom();
        MyRandom.setRandom(new Random(0));
        QUIET = true;   // menu enumeration asks the AI controller (delve, payment) questions that are not decisions of the game
        try { return q.get(); } finally { MyRandom.setRandom(saved); QUIET = false; }
    }

    static List<String> summary(Game game) {
        List<String> out = new ArrayList<>();
        Player[] ps = { game.getRegisteredPlayers().get(0), game.getRegisteredPlayers().get(1) };
        for (int i = 0; i < 2; i++) {
            Player p = ps[i];
            List<String> hand = names(p.getCardsIn(ZoneType.Hand)), gy = names(p.getCardsIn(ZoneType.Graveyard)), ex = new ArrayList<>(), bf = new ArrayList<>();
            for (Card c : game.getCardsIn(ZoneType.Exile)) if (c.getOwner() == p) ex.add(c.getName());
            for (Card c : p.getCardsIn(ZoneType.Battlefield)) {
                StringBuilder sb = new StringBuilder(c.getName());
                if (c.isLand()) sb.append("^L");
                if (c.isToken()) sb.append("^T");
                if (c.isTapped()) sb.append("|Tapped");
                if (c.isCreature() && c.isSick()) sb.append("|SummonSick");
                if (c.getDamage() > 0) sb.append("|Damage:").append(c.getDamage());
                if (!c.getCounters().isEmpty()) {
                    List<String> cs = new ArrayList<>();
                    for (var en : c.getCounters().entrySet()) if (en.getCount() > 0) cs.add(en.getElement().toString() + "=" + en.getCount());
                    Collections.sort(cs);
                    if (!cs.isEmpty()) sb.append("|Counters:").append(String.join(",", cs));
                }
                bf.add(sb.toString());
            }
            Collections.sort(ex); Collections.sort(bf);
            out.add((i == 0 ? "human" : "ai") + " life=" + p.getLife() + " hand=" + String.join(";", hand) + " bf=" + String.join(";", bf) + " gy=" + String.join(";", gy) + " ex=" + String.join(";", ex) + " lib=" + p.getCardsIn(ZoneType.Library).size());
        }
        return out;
    }
    static List<String> names(Iterable<Card> cs) { return names(cs, true); }
    static List<String> names(Iterable<Card> cs, boolean sort) { List<String> l = new ArrayList<>(); for (Card c : cs) l.add(c.getName()); if (sort) Collections.sort(l); return l; }

    static final class Lobby extends LobbyPlayerAi implements IGameEntitiesFactory {
        final Rec rec; final int me; final Game[] gameRef; final Player[] players;
        Lobby(String name, Rec rec, int me, Game[] gameRef, Player[] players) { super(name, null); this.rec = rec; this.me = me; this.gameRef = gameRef; this.players = players; }
        @Override public PlayerController createMindSlaveController(Player master, Player slave) { return new Seat(slave.getGame(), slave, this, rec, me, gameRef); }
        @Override public Player createIngamePlayer(Game game, int id) {
            Player p = new Player(getName(), game, id);
            players[me] = p;
            p.setFirstController(new Seat(game, p, this, rec, me, gameRef));
            return p;
        }
        @Override public void hear(LobbyPlayer player, String message) { }
    }

    static Rec probe(List<String> stateLines, long seed) {
        Rec rec = new Rec();
        MyRandom.setRandom(new Random(seed));
        Game[] gameRef = new Game[1]; Player[] players = new Player[2];
        try {
            Lobby l1 = new Lobby("P1", rec, 0, gameRef, players), l2 = new Lobby("P2", rec, 1, gameRef, players);
            String profile = ((LobbyPlayerAi) GamePlayerUtil.createAiPlayer("profile-source", 0, "")).getAiProfile();
            l1.setAiProfile(profile); l2.setAiProfile(profile);
            RegisteredPlayer r1 = new RegisteredPlayer(Interact.filler("p1")), r2 = new RegisteredPlayer(Interact.filler("p2"));
            r1.setPlayer(l1); r2.setPlayer(l2);
            GameRules rules = new GameRules(GameType.Constructed); rules.setAppliedVariants(EnumSet.of(GameType.Constructed));
            Match mc = new Match(rules, new ArrayList<>(List.of(r1, r2)), "probe");
            Game game = mc.createGame(); game.setNoGUIUser(); gameRef[0] = game;
            GameState st = new GameState();
            st.parse(stateLines);
            try { mc.startGame(game, () -> st.applyToGame(game)); }
            catch (Done d) { }
            catch (RuntimeException e) { Throwable t = e; while (t.getCause() != null && !(t instanceof Done)) t = t.getCause(); if (!(t instanceof Done)) throw e; }
            if (LINE) { if (game.isGameOver()) { rec.over = true; rec.finalSummary = summary(game); try { var wl = game.getOutcome().getWinningLobbyPlayer(); rec.winner = wl == null ? "draw" : wl.getName(); } catch (Throwable t) { rec.winner = "?"; } } else if (rec.windows.isEmpty()) rec.error = "no priority window reached"; }
            else if (rec.activeName == null) rec.error = "no priority window reached";
        } catch (Throwable t) {
            Throwable c = t; while (c.getCause() != null && c.getCause() != c) c = c.getCause();
            rec.error = c.getClass().getSimpleName() + ": " + c.getMessage();
        }
        return rec;
    }

    static Rec inGameThread(List<String> lines, long seed) throws InterruptedException {
        final Rec[] out = new Rec[1];
        Thread t = new Thread(() -> out[0] = probe(lines, seed), "Game-probe");
        t.setDaemon(true); t.start(); t.join(60_000);
        if (out[0] == null) { Rec r = new Rec(); r.error = "timeout"; return r; }
        return out[0];
    }

    public static void main(String[] args) throws Exception {
        System.setProperty("java.awt.headless", "true");
        String in = args[0], outp = args[1];
        // same bootstrap as Interact.main
        GuiBase.setInterface(new GuiDesktop());
        FModel.initialize(null, null);
        Gson gson = new Gson();
        try (BufferedReader br = Files.newBufferedReader(Paths.get(in)); PrintWriter pw = new PrintWriter(Files.newBufferedWriter(Paths.get(outp)))) {
            String line; int n = 0;
            while ((line = br.readLine()) != null) {
                if (line.isBlank()) continue;
                JsonObject o = JsonParser.parseString(line).getAsJsonObject();
                List<String> lines = new ArrayList<>();
                for (JsonElement e : o.getAsJsonArray("state")) lines.add(e.getAsString());
                Seat.WINDOW = o.has("window") ? o.get("window").getAsInt() : 0;
                ACTION = o.has("action") && !o.get("action").isJsonNull() ? o.get("action").getAsString() : null;
                COMBAT = ACTION != null && ACTION.startsWith("combat|");
                LINE = o.has("mode") && o.get("mode").getAsString().equals("line");
                LINE_SEED = o.has("line_seed") ? o.get("line_seed").getAsLong() : 0;
                LINE_MAX = o.has("line_max") ? o.get("line_max").getAsInt() : 12;
                FOCUS = new HashSet<>();
                if (o.has("focus")) for (JsonElement fe : o.getAsJsonArray("focus")) FOCUS.add(fe.getAsString());
                ACT_FAILED = false;
                RESPONSE = o.has("response") && !o.get("response").isJsonNull() ? o.get("response").getAsString() : null;
                RESPONDED = false; ACTOR = null;
                ABILITY_INDEX = o.has("ability_index") ? o.get("ability_index").getAsInt() : -1;
                ACTED = false; CALLS = 0; LOG.clear();
                Rec r = inGameThread(lines, o.get("seed").getAsLong());
                JsonObject res = new JsonObject();
                res.addProperty("id", o.get("id").getAsString());
                res.add("forge_actions", gson.toJsonTree(r.actions));
                res.add("forge_summary", gson.toJsonTree(r.summary));
                res.addProperty("stack", r.stack);
                if (ACTION != null || LINE) { res.add("log", gson.toJsonTree(LOG)); res.add("notes", gson.toJsonTree(r.notes)); }
                if (LINE) {
                    JsonArray ws = new JsonArray(); for (JsonObject w : r.windows) ws.add(w);
                    res.add("windows", ws);
                    res.addProperty("over", r.over); res.addProperty("winner", r.winner);
                    if (r.finalSummary != null) res.add("final_summary", gson.toJsonTree(r.finalSummary));
                }
                res.addProperty("phase", String.valueOf(r.phase));
                res.addProperty("priority", r.activeName == null ? "" : r.activeName);
                if (r.error != null) res.addProperty("error", r.error);
                pw.println(res);
                if (++n % 100 == 0) { pw.flush(); System.err.println("probed " + n); }
            }
        }
        System.exit(0);
    }
}
