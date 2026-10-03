package itest;

import forge.GuiDesktop;
import forge.LobbyPlayer;
import forge.ai.*;
import forge.deck.Deck;
import forge.deck.DeckSection;
import forge.game.*;
import forge.game.card.*;
import forge.game.cost.*;
import forge.game.zone.Zone;
import forge.game.ability.AbilityUtils;
import forge.game.ability.ApiType;
import forge.game.ability.effects.CharmEffect;
import forge.card.CardStateName;
import forge.game.combat.Combat;
import forge.game.combat.CombatUtil;
import forge.game.phase.PhaseType;
import forge.game.player.*;
import forge.game.spellability.*;
import forge.game.zone.ZoneType;
import forge.gui.GuiBase;
import forge.item.PaperCard;
import forge.model.FModel;
import forge.player.GamePlayerUtil;
import forge.util.MyRandom;
import forge.util.collect.FCollectionView;

import java.io.*;
import java.nio.file.*;
import java.util.*;
import java.util.function.Consumer;
import java.util.regex.*;
import java.util.stream.*;

/**
 * Scripted-state rules tests for Forge. One JVM runs many scenarios; each gets a fresh Game built from a Forge
 * GameState text, two scripted seats (subclasses of the stock AI controller, so every unscripted decision falls back to the
 * Forge AI and is logged as "(default)"), and a list of expectations checked after the script completes.
 * Scenario format: see ../README.md.
 */
public final class Interact {
    static boolean VERBOSE = false;

    // ------------------------------------------------------------------ scenario model
    static final class Scenario {
        String name, title = "", ref = "", expectFail = "", file = ""; boolean swapSeats;
        List<String> state = new ArrayList<>();
        Map<Integer, List<String>> script = new HashMap<>();   // 0 = p1, 1 = p2
        List<String> expect = new ArrayList<>();
        List<String> notes = new ArrayList<>();
    }

    static final class Result {
        Scenario sc; String status = "PASS"; List<String> failures = new ArrayList<>(); List<String> trace = new ArrayList<>();
        String error; String finalDump = ""; String gameLog = "";
    }

    static final class ScenarioDone extends RuntimeException { ScenarioDone() { super("scenario done", null, false, false); } }

    // ------------------------------------------------------------------ parsing
    static List<Scenario> parse(Path file) throws IOException {
        List<Scenario> out = new ArrayList<>();
        Scenario cur = null; String section = null;
        for (String raw : Files.readAllLines(file)) {
            String line = raw.stripTrailing();
            if (line.startsWith("=== ")) { cur = new Scenario(); cur.name = line.substring(4).trim(); cur.file = file.getFileName().toString(); out.add(cur); section = null; continue; }
            if (cur == null || line.isBlank() || line.trim().startsWith("#")) continue;
            if (!Character.isWhitespace(line.charAt(0))) {
                int i = line.indexOf(':'); String key = line.substring(0, i).trim(), val = line.substring(i + 1).trim();
                switch (key) {
                    case "title" -> cur.title = val;
                    case "ref" -> cur.ref = val;
                    case "swap-seats" -> cur.swapSeats = val.equalsIgnoreCase("yes");
                    case "xfail" -> cur.expectFail = val;   // known/expected Forge deviation: scenario is expected to fail
                    case "note" -> cur.notes.add(val);
                    case "state", "script", "expect" -> section = key;
                    default -> throw new IllegalArgumentException(file + ": unknown key " + key + " in " + cur.name);
                }
                continue;
            }
            String t = line.trim();
            switch (section) {
                case "state" -> cur.state.add(t);
                case "expect" -> cur.expect.add(t);
                case "script" -> {
                    int i = t.indexOf(':'); String who = t.substring(0, i).trim(); String rest = t.substring(i + 1).trim();
                    int idx = switch (who) { case "p1" -> 0; case "p2" -> 1; default -> throw new IllegalArgumentException("script line needs p1:/p2: " + t); };
                    cur.script.computeIfAbsent(idx, k -> new ArrayList<>()).add(rest);
                }
                default -> throw new IllegalArgumentException(file + ": stray line in " + cur.name + ": " + t);
            }
        }
        return out;
    }

    // ------------------------------------------------------------------ run state shared by both seats
    static final class Run {
        Game game; final Scenario sc; final Result res;
        final List<Deque<String>> queues = List.of(new ArrayDeque<>(), new ArrayDeque<>());
        final int[] priorityCalls = new int[2];
        int startTurn; boolean finished;
        Run(Scenario s, Result r) { sc = s; res = r;
            for (int i = 0; i < 2; i++) queues.get(i).addAll(s.script.getOrDefault(i, List.of())); }
        void log(String s) { res.trace.add(s); if (VERBOSE) System.out.println("    " + s); }
        final Player[] players = new Player[2];
        Player pl(int i) { return players[i]; }
        int idx(Player p) { return p == players[0] ? 0 : p == players[1] ? 1 : -1; }
        String who(Player p) { return "p" + (idx(p) + 1); }
    }

    // ------------------------------------------------------------------ the scripted controller
    static final class Seat extends PlayerControllerAi {
        final Run run; final int me;
        Seat(Game g, Player p, LobbyPlayer lp, Run run, int me) { super(g, p, lp); this.run = run; this.me = me; }
        Deque<String> q() { return run.queues.get(me); }

        // ---- priority
        @Override public List<SpellAbility> chooseSpellAbilityToPlay() {
            Game g = run.game;
            if (++run.priorityCalls[me] > 400) throw new IllegalStateException("script did not finish (400 priority windows)");
            if (g.getPhaseHandler().getTurn() > run.startTurn + 6) throw new IllegalStateException("script did not finish (6 turns passed)");
            while (true) {
                String head = q().peekFirst();
                if (head == null) {
                    if (g.getStack().isEmpty() && run.queues.get(0).isEmpty() && run.queues.get(1).isEmpty() && noWaiters()) { run.finished = true; throw new ScenarioDone(); }
                    return null;
                }
                String[] w = head.split("\\s+", 2); String cmd = w[0], arg = w.length > 1 ? w[1] : "";
                switch (cmd) {
                    case "pass": q().pollFirst(); run.log(run.who(player) + " pass"); return null;
                    case "at": {
                        if (!phaseMatches(arg)) return null;
                        q().pollFirst(); continue;
                    }
                    case "wait-empty": { if (!g.getStack().isEmpty()) return null; q().pollFirst(); continue; }
                    case "when-stack": {
                        SpellAbility top = g.getStack().isEmpty() ? null : g.getStack().peekAbility();
                        if (top == null || !top.getHostCard().getName().contains(arg.trim())) return null;
                        q().pollFirst(); continue;
                    }
                    case "check": q().pollFirst(); String bad = Expect.eval(run, arg); if (bad != null) run.res.failures.add("inline check [" + arg + "]: " + bad); continue;
                    case "pick": case "attack": case "block": return null; // consumed by the matching decision hook; if none ever asks, the script never finishes
                    case "cast": case "play": case "activate": case "try": {
                        q().pollFirst();
                        boolean tryOnly = cmd.equals("try"); String spec = tryOnly ? arg.replaceFirst("^(cast|play|activate)\\s+", "") : arg;
                        SpellAbility sa = findSa(spec, tryOnly);
                        if (sa == null) { run.log(run.who(player) + " could not " + head); if (tryOnly) { continue; } return null; }
                        pending = new Pending(sa, spec);
                        run.log(run.who(player) + " " + head + "  => " + sa.getHostCard().getName() + " [" + sa + "]");
                        return List.of(sa);
                    }
                    default: throw new IllegalArgumentException("unknown script command: " + head);
                }
            }
        }
        boolean noWaiters() { return true; }

        boolean phaseMatches(String arg) {
            String a = arg.trim(); Integer owner = null;
            if (a.startsWith("p1:") || a.startsWith("p2:")) { owner = a.charAt(1) - '1'; a = a.substring(3); }
            PhaseType want = PhaseType.valueOf(a.toUpperCase());
            if (run.game.getPhaseHandler().getPhase() != want) return false;
            return owner == null || run.idx(run.game.getPhaseHandler().getPlayerTurn()) == owner;
        }

        record Pending(SpellAbility sa, String spec) {}
        Pending pending;

        /** spec: Card Name [~filter] [-> t1, t2] [x=N] */
        SpellAbility findSa(String spec, boolean quiet) {
            String filter = null; String targets = null; Integer x = null;
            Matcher mx = Pattern.compile("\\s+x=(\\d+)").matcher(spec); if (mx.find()) { x = Integer.parseInt(mx.group(1)); spec = mx.replaceAll(""); }
            int ta = spec.indexOf("->"); if (ta >= 0) { targets = spec.substring(ta + 2).trim(); spec = spec.substring(0, ta).trim(); }
            int tf = spec.indexOf('~'); if (tf >= 0) { filter = spec.substring(tf + 1).trim(); spec = spec.substring(0, tf).trim(); }
            allowManaAb = filter != null;
            String cardSpec = spec.trim(); String ctlFilter = null;
            int at = cardSpec.lastIndexOf('@'); if (at >= 0) { ctlFilter = cardSpec.substring(at + 1); cardSpec = cardSpec.substring(0, at); }
            List<SpellAbility> cands = new ArrayList<>();
            for (Card c : run.game.getCardsIncludePhasingIn(ZoneType.Hand)) consider(c, cardSpec, ctlFilter, cands);
            for (ZoneType z : List.of(ZoneType.Battlefield, ZoneType.Graveyard, ZoneType.Exile, ZoneType.Command, ZoneType.Library))
                for (Card c : run.game.getCardsIn(z)) consider(c, cardSpec, ctlFilter, cands);
            List<SpellAbility> pick = new ArrayList<>();
            for (SpellAbility sa : cands) {
                if (filter != null) {
                    if (filter.equals("alt")) { if (!isAlt(sa)) continue; }
                    else if (filter.equals("hard")) { if (isAlt(sa)) continue; }
                    else if (filter.equals("free")) { if (!isAlt(sa) || !String.valueOf(sa.getPayCosts()).trim().isEmpty()) continue; }
                    else if (filter.matches("#\\d+")) { if (cands.indexOf(sa) != Integer.parseInt(filter.substring(1))) continue; }
                    else if (filter.startsWith("cost:")) { if (!String.valueOf(sa.getPayCosts()).toLowerCase().contains(filter.substring(5).toLowerCase())) continue; }
                    else if (!(sa.toString() + " " + sa.getDescription() + " " + sa.getStackDescription()).toLowerCase().contains(filter.toLowerCase())) continue;
                }
                pick.add(sa);
            }
            if (filter == null) { // default: prefer the plain version (no alternative cost)
                List<SpellAbility> plain = pick.stream().filter(s -> !isAlt(s)).collect(Collectors.toList());
                if (!plain.isEmpty()) pick = plain;
            }
            if (pick.isEmpty()) {
                run.log("  no playable ability for '" + spec + "' (filter " + filter + "); candidates: " + IntStream.range(0, cands.size()).mapToObj(i -> "#" + i + " " + (isAlt(cands.get(i)) ? "ALT " : "") + cands.get(i).getPayCosts() + " :: " + cands.get(i)).collect(Collectors.joining(" | ")));
                return null;
            }
            SpellAbility sa = pick.get(0);
            run.log("  abilities of '" + cardSpec + "': " + IntStream.range(0, cands.size()).mapToObj(i -> "#" + i + " " + (isAlt(cands.get(i)) ? "ALT " : "") + cands.get(i).getPayCosts() + " :: " + cands.get(i)).collect(Collectors.joining(" | ")) + "  -> chose #" + cands.indexOf(sa));
            sa.setActivatingPlayer(player);
            if (x != null) sa.setXManaCostPaid(x);
            sa.setSVar("itest_targets", targets == null ? "" : targets);
            return sa;
        }

        /** Alternative-cost versions are the extra SpellAbility copies that Card.getAllPossibleAbilities adds (Force of Will, Daze, Aluren/Omniscience free casts, ...). */
        final Set<SpellAbility> altSet = Collections.newSetFromMap(new IdentityHashMap<>());
        boolean isAlt(SpellAbility sa) { return altSet.contains(sa); }

        boolean allowManaAb;
        void consider(Card c, String name, String ctl, List<SpellAbility> out) {
            if (!c.getName().equals(name)) return;
            if (ctl != null) { int i = ctl.equals("p1") ? 0 : 1; if (!(c.getController() == run.pl(i) || (c.getController() == null && c.getOwner() == run.pl(i)))) return; }
            Set<SpellAbility> base = Collections.newSetFromMap(new IdentityHashMap<>());
            base.addAll(c.getSpellAbilities());
            for (CardStateName sn : List.of(CardStateName.Backside, CardStateName.Secondary)) if (c.hasState(sn)) base.addAll(c.getState(sn).getSpellAbilities());
            for (SpellAbility sa : c.getAllPossibleAbilities(player, true)) {
                if (sa.isManaAbility() && !c.isLand() && !allowManaAb) continue; // nonland mana abilities (LED, Petal) only when the spec names one with ~filter
                if (!base.contains(sa) && sa.isSpell()) altSet.add(sa);
                out.add(sa);
            }
        }

        @Override public boolean playChosenSpellAbility(SpellAbility sa) {
            Pending p = pending; pending = null;
            String tspec = sa.getSVar("itest_targets");
            if (sa.isLandAbility()) { if (sa.canPlay()) { sa.resolve(); } return true; }
            if (p == null) return super.playChosenSpellAbility(sa);
            final String[] errHolder = new String[1];
            Consumer<SpellAbility> chooser = root -> {
                if (tspec == null || tspec.isEmpty()) return;
                // "a; b | c": groups separated by '|' go to successive targeting parts of the ability chain (charm modes, sub-abilities)
                SpellAbility cur = root;
                for (String group : tspec.split("\\|")) {
                    while (cur != null && !cur.usesTargeting()) cur = cur.getSubAbility();
                    if (cur == null) { errHolder[0] = "no targeting part for group '" + group.trim() + "'"; return; }
                    for (String t : group.split(";")) {
                        GameObject o = resolveObject(t.trim()); if (o == null) { errHolder[0] = "target not found: " + t; return; }
                        if (!cur.canTarget(o)) { errHolder[0] = "illegal target " + o + " for " + cur; return; }
                        cur.getTargets().add(o);
                    }
                    cur = cur.getSubAbility();
                }
            };
            boolean ok = playScripted(sa, chooser);
            if (errHolder[0] != null) run.log("  TARGET ERROR: " + errHolder[0]);
            run.log("  " + (ok ? "cast/activated ok" : "play FAILED (cost or target)") + " " + sa.getHostCard().getName());
            if (!ok) run.log("CASTFAIL " + sa.getHostCard().getName());
            return true;
        }

        /**
         * Same sequence as ComputerUtil.handlePlayingSpellAbility (move to stack, choose modes, choose targets, pay) but costs go through
         * ScriptCost so the script can pick pitch cards etc., and a failed attempt is rolled back to the origin zone (the upstream method leaves the
         * card stuck on the stack zone for spells cast from hand).
         */
        boolean playScripted(SpellAbility sa, Consumer<SpellAbility> chooseTargets) {
            final Game game = run.game; final Card source = sa.getHostCard();
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
            if (ok && !sa.checkRestrictions(player)) { run.log("  checkRestrictions refused " + sa.getHostCard().getName() + " (CantBeCast and similar statics, CR 601.2)"); ok = false; } // PlaySpellAbility does this; the AI shortcut path does not
            if (sa.getApi() == ApiType.Charm && !CharmEffect.makeChoices(sa)) ok = false;
            if (ok) { chooseTargets.accept(sa); if (!sa.isTargetNumberValid()) ok = false; }
            if (ok) {
                game.getStack().freezeStack(sa);
                if (new CostPayment(sa.getPayCosts(), sa).payComputerCosts(new ScriptCost(player, sa, false, this))) { game.getStack().addAndUnfreeze(sa); return true; }
                game.getStack().unfreezeStack();
            }
            // roll back
            sa.setSkip(true);
            if (orig != null && sa.getHostCard().isInZone(ZoneType.Stack)) game.getAction().moveTo(orig, sa.getHostCard(), null, null);
            return false;
        }

        GameObject resolveObject(String t) {
            if (t.equals("p1")) return run.pl(0);
            if (t.equals("p2")) return run.pl(1);
            if (t.startsWith("stack:")) { // an item on the stack (spell, activated or triggered ability) by source-card name, topmost first
                String n = t.substring(6).trim();
                for (var si : run.game.getStack()) if (si.getSourceCard().getName().equals(n)) return si.getSpellAbility();
                return null;
            }
            Card c = findCard(run, t, null);
            if (c != null && c.isInZone(ZoneType.Stack)) { // targeting a spell means targeting its SpellAbility
                for (var si : run.game.getStack()) if (si.getSourceCard() == c) return si.getSpellAbility();
            }
            return c;
        }

        // ---- combat
        @Override public void declareAttackers(Player attacker, Combat combat) {
            String head = q().peekFirst();
            if (head == null || !head.startsWith("attack")) return;
            q().pollFirst();
            String spec = head.substring(6).trim(); String tgt = "p" + (3 - (me + 1));
            int a = spec.indexOf("->"); if (a >= 0) { tgt = spec.substring(a + 2).trim(); spec = spec.substring(0, a).trim(); }
            GameEntity defender = tgt.startsWith("p") ? run.pl(tgt.equals("p1") ? 0 : 1) : (GameEntity) findCard(run, tgt, null);
            for (String n : spec.split(";")) {
                if (n.isBlank()) continue;
                Card c = findCard(run, n.trim(), ZoneType.Battlefield);
                if (c == null || !CombatUtil.canAttack(c, defender)) { run.log("  attack declaration illegal/missing: " + n); continue; }
                combat.addAttacker(c, defender);
            }
            run.log(run.who(player) + " attacks with " + spec + " -> " + tgt);
        }

        @Override public void declareBlockers(Player defender, Combat combat) {
            String head = q().peekFirst();
            if (head == null || !head.startsWith("block")) return;
            q().pollFirst();
            for (String pair : head.substring(5).split(";")) {
                String[] ab = pair.split(" blocks ");
                if (ab.length != 2) continue;
                Card blocker = findCard(run, ab[0].trim(), ZoneType.Battlefield), atk = findCard(run, ab[1].trim(), ZoneType.Battlefield);
                if (blocker == null || atk == null || !CombatUtil.canBlock(atk, blocker)) { run.log("  block illegal/missing: " + pair); continue; }
                combat.addBlocker(atk, blocker);
            }
            run.log(run.who(player) + " blocks: " + head.substring(5));
        }

        // ---- generic choice hooks
        /** Pops the head of the queue if it is a pick entry; returns its argument or null. */
        String takePick() {
            String head = q().peekFirst();
            if (head == null || !head.startsWith("pick")) return null;
            q().pollFirst(); return head.substring(4).trim();
        }

        static String label(Object o) { return o instanceof Card c ? c.getName() : o instanceof Player p ? p.getName() : String.valueOf(o); }

        <T> List<T> pickFrom(String kind, Collection<T> options, int min, int max, String why) {
            List<T> opts = new ArrayList<>(options);
            String pk = takePick();
            if (pk == null) { run.log(run.who(player) + " DEFAULT " + kind + " [" + why + "] options=" + opts.stream().map(Seat::label).toList()); return null; }
            List<T> chosen = new ArrayList<>();
            if (pk.startsWith("!")) { // negative assertion: none of these may be offered; the engine's choice then falls back to the AI default
                for (String name : splitTop(pk.substring(1))) for (T o : opts) if (match(o, name.trim())) run.res.failures.add("script: '" + name.trim() + "' was offered (" + why + ") but must not be: " + opts.stream().map(Seat::label).toList());
                run.log(run.who(player) + " pick-absent " + pk.substring(1) + " ok [" + why + "] (offered " + opts.stream().map(Seat::label).toList() + ")");
                return null;
            }
            if (pk.equals("none")) { run.log(run.who(player) + " pick none [" + why + "] (offered " + opts.stream().map(Seat::label).toList() + ")"); return chosen; }
            if (pk.equals("all")) { chosen.addAll(opts); return chosen; }
            List<T> pool = new ArrayList<>(opts);
            for (String name : splitTop(pk)) {
                name = name.trim(); T found = null;
                for (T o : pool) { if (match(o, name)) { found = o; break; } }
                if (found == null) { run.log(run.who(player) + " PICK MISMATCH '" + name + "' not among " + opts.stream().map(Seat::label).toList() + " [" + why + "]"); run.res.failures.add("script: pick '" + name + "' not offered (" + why + "): " + opts.stream().map(Seat::label).toList()); return null; }
                chosen.add(found); pool.remove(found);
            }
            run.log(run.who(player) + " pick " + pk + " [" + why + "] (offered " + opts.stream().map(Seat::label).toList() + ")");
            return chosen;
        }
        static boolean match(Object o, String name) {
            if (o instanceof Player p) return name.equals(p.getName()) || name.equals("p1") && p.getName().startsWith("P1") || name.equals("p2") && p.getName().startsWith("P2");
            if (o instanceof Card c) return c.getName().equals(name);
            if (o instanceof SpellAbility sa) return (sa.toString() + " " + sa.getDescription() + " " + sa.getHostCard().getName()).toLowerCase().contains(name.toLowerCase());
            return String.valueOf(o).toLowerCase().contains(name.toLowerCase());
        }
        static List<String> splitTop(String s) { return Arrays.stream(s.split(";")).map(String::trim).collect(Collectors.toList()); }

        @Override public CardCollectionView chooseCardsForEffect(CardCollectionView src, SpellAbility sa, String title, int min, int max, boolean opt, Map<String, Object> params) {
            List<Card> r = pickFrom("cards", src, min, max, title); if (r == null) return super.chooseCardsForEffect(src, sa, title, min, max, opt, params);
            return new CardCollection(r);
        }
        @Override public <T extends GameEntity> T chooseSingleEntityForEffect(FCollectionView<T> optionList, DelayedReveal dr, SpellAbility sa, String title, boolean opt, Player tp, Map<String, Object> params) {
            if (dr != null) reveal(dr);
            List<T> r = pickFrom("entity", optionList, 0, 1, title + " / " + sa.getHostCard().getName());
            if (r == null) return super.chooseSingleEntityForEffect(optionList, null, sa, title, opt, tp, params);
            return r.isEmpty() ? null : r.get(0);
        }
        @Override public <T extends GameEntity> List<T> chooseEntitiesForEffect(FCollectionView<T> optionList, int min, int max, DelayedReveal dr, SpellAbility sa, String title, Player tp, Map<String, Object> params) {
            if (dr != null) reveal(dr);
            List<T> r = pickFrom("entities", optionList, min, max, title + " / " + sa.getHostCard().getName());
            if (r == null) return super.chooseEntitiesForEffect(optionList, min, max, null, sa, title, tp, params);
            return r;
        }
        @Override public SpellAbility chooseSingleSpellForEffect(List<SpellAbility> spells, SpellAbility sa, String title, Map<String, Object> params) {
            List<SpellAbility> r = pickFrom("spell", spells, 1, 1, title); if (r == null) return super.chooseSingleSpellForEffect(spells, sa, title, params);
            return r.isEmpty() ? null : r.get(0);
        }
        @Override public List<AbilitySub> chooseModeForAbility(SpellAbility sa, List<AbilitySub> possible, int min, int num, boolean allowRepeat) {
            List<AbilitySub> r = pickFrom("mode", possible, min, num, sa.getHostCard().getName()); if (r == null) return super.chooseModeForAbility(sa, possible, min, num, allowRepeat);
            return r;
        }
        Boolean yn(String kind, String why) {
            String pk = takePick(); if (pk == null) { return null; }
            run.log(run.who(player) + " pick " + pk + " [" + kind + ": " + why + "]");
            return pk.equalsIgnoreCase("yes") || pk.equalsIgnoreCase("true");
        }
        @Override public boolean confirmAction(SpellAbility sa, PlayerActionConfirmMode mode, String message, List<String> options, Card cardToShow, Map<String, Object> params) {
            String h = q().peekFirst(); Boolean b = (h != null && (h.equals("pick yes") || h.equals("pick no"))) ? yn("confirm", message) : null;
            if (b != null) return b;
            boolean d = super.confirmAction(sa, mode, message, options, cardToShow, params); run.log(run.who(player) + " DEFAULT confirm [" + message + "] -> " + d); return d;
        }
        @Override public boolean confirmTrigger(forge.game.trigger.WrappedAbility w) {
            String h = q().peekFirst(); Boolean b = (h != null && (h.equals("pick yes") || h.equals("pick no"))) ? yn("trigger", w.toString()) : null;
            if (b != null) return b;
            boolean d = super.confirmTrigger(w); run.log(run.who(player) + " DEFAULT trigger-confirm [" + w + "] -> " + d); return d;
        }
        @Override public boolean confirmReplacementEffect(forge.game.replacement.ReplacementEffect re, SpellAbility sa, GameEntity affected, String question) {
            String h = q().peekFirst(); Boolean b = (h != null && (h.equals("pick yes") || h.equals("pick no"))) ? yn("replacement", question) : null;
            if (b != null) return b;
            boolean d = super.confirmReplacementEffect(re, sa, affected, question); run.log(run.who(player) + " DEFAULT replacement-confirm [" + question + "] -> " + d); return d;
        }
        @Override public boolean chooseBinary(SpellAbility sa, String question, BinaryChoiceType kind, Boolean def) {
            String h = q().peekFirst(); Boolean b = (h != null && (h.equals("pick yes") || h.equals("pick no"))) ? yn("binary", question) : null;
            if (b != null) return b;
            boolean d = super.chooseBinary(sa, question, kind, def); run.log(run.who(player) + " DEFAULT binary [" + question + "] -> " + d); return d;
        }
        @Override public boolean chooseBinary(SpellAbility sa, String question, BinaryChoiceType kind, Map<String, Object> params) {
            String h = q().peekFirst(); Boolean b = (h != null && (h.equals("pick yes") || h.equals("pick no"))) ? yn("binary", question) : null;
            if (b != null) return b;
            boolean d = super.chooseBinary(sa, question, kind, params); run.log(run.who(player) + " DEFAULT binary [" + question + "] -> " + d); return d;
        }
        @Override public int chooseNumber(SpellAbility sa, String title, int min, int max) {
            String pk = takePick(); if (pk != null) { run.log(run.who(player) + " pick " + pk + " [number " + title + "]"); return Integer.parseInt(pk); }
            int d = super.chooseNumber(sa, title, min, max); run.log(run.who(player) + " DEFAULT number [" + title + "] -> " + d); return d;
        }
        @Override public int chooseNumber(SpellAbility sa, String title, int min, int max, Map<String, Object> params) {
            String pk = takePick(); if (pk != null) { run.log(run.who(player) + " pick " + pk + " [number " + title + "]"); return Integer.parseInt(pk); }
            int d = super.chooseNumber(sa, title, min, max, params); run.log(run.who(player) + " DEFAULT number [" + title + "] -> " + d); return d;
        }
        /** Triggers of an AI-seat controller go through prepareSingleSa -> AiController.doTrigger, never chooseTargetsFor, so script the targets here. */
        @Override public void orderAndPlaySimultaneousSa(List<SpellAbility> sas) {
            for (SpellAbility sa : orderSimultaneousSa(sas)) {
                String h = q().peekFirst();
                SpellAbility inner = sa instanceof forge.game.trigger.WrappedAbility w ? w.getWrappedAbility() : sa;
                SpellAbility tgt = inner; while (tgt != null && !tgt.usesTargeting()) tgt = tgt.getSubAbility();
                if (sa.isTrigger() && !sa.isCopied() && tgt != null && h != null && h.startsWith("pick !")) { // negative assertion: these objects must not be legal targets; the AI then picks as usual
                    String pk = takePick().substring(1);
                    for (String t : pk.split(";")) { GameObject o = resolveObject(t.trim()); if (o != null && tgt.canTarget(o)) run.res.failures.add("script: '" + t.trim() + "' is a legal target of trigger " + sa.getHostCard().getName() + " but must not be"); }
                    run.log(run.who(player) + " pick-untargetable " + pk + " checked for " + sa.getHostCard().getName() + " trigger");
                    super.orderAndPlaySimultaneousSa(List.of(sa));
                    continue;
                }
                if (sa.isTrigger() && !sa.isCopied() && tgt != null && h != null && h.startsWith("pick")) {
                    String pk = takePick(); List<GameObject> objs = new ArrayList<>();
                    boolean bad = false;
                    if (!pk.equals("none")) for (String t : pk.split(";")) { GameObject o = resolveObject(t.trim()); if (o == null || !tgt.canTarget(o)) { run.res.failures.add("script: cannot target '" + t + "' with trigger " + sa.getHostCard().getName() + " " + tgt); run.log("TARGET MISMATCH " + t); bad = true; break; } objs.add(o); }
                    if (!bad) { tgt.resetTargets(); for (GameObject o : objs) tgt.getTargets().add(o); run.log(run.who(player) + " targets " + pk + " for " + sa.getHostCard().getName() + " trigger"); }
                    if (!bad && (objs.isEmpty() ? tgt.getMinTargets() == 0 : true)) ComputerUtil.playStack(sa, player, run.game);
                    continue;
                }
                super.orderAndPlaySimultaneousSa(List.of(sa));
            }
        }

        /** Free-form "you may cast it" effects (Bilbo, Amped Raptor, ...) are decided by the AI's canPlayFromEffectAI otherwise; "pick play" / "pick skip" overrides it. */
        @Override public boolean playSaFromPlayEffect(SpellAbility tgtSA) {
            String h = q().peekFirst();
            if (h != null && (h.equals("pick play") || h.equals("pick skip"))) {
                q().pollFirst(); boolean play = h.equals("pick play");
                run.log(run.who(player) + " " + h + " [cast from effect: " + tgtSA.getHostCard().getName() + "]");
                return play && ComputerUtil.playStack(tgtSA, player, run.game);
            }
            boolean d = super.playSaFromPlayEffect(tgtSA);
            run.log(run.who(player) + " DEFAULT cast-from-effect " + tgtSA.getHostCard().getName() + " -> " + d);
            return d;
        }

        /** "pick A;B;C" gives the final library order top-first; Forge moves list elements to position 0 one by one, so the list is bottom-first. */
        @Override public CardCollectionView orderMoveToZoneList(CardCollectionView cards, ZoneType dest, SpellAbility source) {
            String h = q().peekFirst();
            if (h != null && h.startsWith("pick") && cards.size() > 1 && dest == ZoneType.Library) { // only library orders are scripted; graveyard/exile orders are meaningless noise
                List<Card> r = pickFrom("order->" + dest, cards, cards.size(), cards.size(), source.getHostCard().getName());
                if (r != null) { List<Card> all = new ArrayList<>(r); for (Card c : cards) if (!all.contains(c)) all.add(c); if (dest == ZoneType.Library || dest == ZoneType.Graveyard) Collections.reverse(all); return new CardCollection(all); }
            }
            CardCollectionView d = super.orderMoveToZoneList(cards, dest, source);
            run.log(run.who(player) + " DEFAULT order->" + dest + " " + cards.stream().map(Card::getName).toList() + " -> " + d.stream().map(Card::getName).toList());
            return d;
        }

        @Override public boolean chooseTargetsFor(SpellAbility ability) {
            String h = q().peekFirst();
            if (h != null && h.startsWith("pick")) {
                // targets for a triggered ability (spells/abilities we cast are targeted inside playChosenSpellAbility)
                String pk = takePick(); List<GameObject> objs = new ArrayList<>();
                if (!pk.equals("none")) for (String t : pk.split(";")) { GameObject o = resolveObject(t.trim()); if (o == null || !ability.canTarget(o)) { run.res.failures.add("script: cannot target '" + t + "' with " + ability); run.log("TARGET MISMATCH " + t); return false; } objs.add(o); }
                ability.resetTargets(); for (GameObject o : objs) ability.getTargets().add(o);
                run.log(run.who(player) + " targets " + pk + " for " + ability.getHostCard().getName() + " trigger");
                return !objs.isEmpty();
            }
            boolean d = super.chooseTargetsFor(ability);
            run.log(run.who(player) + " DEFAULT targets for " + ability.getHostCard().getName() + ": " + ability.getTargets() + " -> " + d);
            return d;
        }
        @Override public String chooseCardName(SpellAbility sa, java.util.function.Predicate<forge.card.ICardFace> cpp, String valid, String message) {
            String pk = takePick(); if (pk != null) { run.log(run.who(player) + " pick " + pk + " [name]"); return pk; }
            String d = super.chooseCardName(sa, cpp, valid, message); run.log(run.who(player) + " DEFAULT name [" + message + "] -> " + d); return d;
        }
        @Override public byte chooseColor(String message, SpellAbility sa, forge.card.ColorSet colors) {
            String pk = takePick(); if (pk != null) { byte b = forge.card.MagicColor.fromName(pk); run.log(run.who(player) + " pick " + pk + " [color]"); return b; }
            return super.chooseColor(message, sa, colors);
        }
        @Override public boolean mulliganKeepHand(Player firstPlayer, int cardsToReturn) { return true; }
        @Override public CardCollection chooseCardsToDiscardFrom(Player p, SpellAbility sa, CardCollection valid, int min, int max, CardCollectionView visible) {
            List<Card> r = pickFrom("discard", valid, min, max, sa.getHostCard().getName()); if (r == null) return super.chooseCardsToDiscardFrom(p, sa, valid, min, max, visible);
            return new CardCollection(r);
        }

        @Override public Card chooseSingleCardForZoneChange(ZoneType destination, List<ZoneType> origin, SpellAbility sa, CardCollection fetchList, DelayedReveal dr, String selectPrompt, boolean isOptional, Player decider) {
            if (dr != null) reveal(dr);
            String h = q().peekFirst();
            if (h != null && h.startsWith("pick")) {
                List<Card> r = pickFrom("zonechange " + origin + "->" + destination, fetchList, 0, 1, sa.getHostCard().getName() + ": " + selectPrompt);
                if (r != null) return r.isEmpty() ? null : r.get(0);
            }
            Card d = brainsPick(destination, origin, sa, fetchList, decider);
            run.log(run.who(player) + " DEFAULT zonechange " + origin + "->" + destination + " [" + sa.getHostCard().getName() + ": " + selectPrompt + "] options=" + fetchList.stream().map(Card::getName).toList() + " -> " + (d == null ? "none" : d.getName()));
            return d;
        }
        Card brainsPick(ZoneType destination, List<ZoneType> origin, SpellAbility sa, CardCollection fetchList, Player decider) { return getAi().chooseCardToHiddenOriginChangeZone(destination, origin, sa, fetchList, player, decider); }
        @Override public List<Card> chooseCardsForZoneChange(ZoneType destination, List<ZoneType> origin, SpellAbility sa, CardCollection fetchList, int min, int max, DelayedReveal dr, String selectPrompt, Player decider) {
            if (dr != null) reveal(dr);
            List<Card> r = pickFrom("zonechange-multi " + origin + "->" + destination, fetchList, min, max, sa.getHostCard().getName() + ": " + selectPrompt);
            if (r != null) return r;
            return super.chooseCardsForZoneChange(destination, origin, sa, fetchList, min, max, dr, selectPrompt, decider);
        }

        // ---- more hooks: costs, delve, scry/surveil, dungeon choice, "unless pays"
        @Override public CostDecisionMakerBase getCostDecisionMaker(Player pl, SpellAbility ability, boolean effect, String prompt) { return new ScriptCost(pl, ability, effect, this); }

        @Override public boolean payCostToPreventEffect(Cost cost, SpellAbility sa, boolean alreadyPaid, FCollectionView<Player> allPayers) {
            String h = q().peekFirst();
            if (h != null && (h.equals("pick yes") || h.equals("pick no"))) {
                boolean yes = takePick().equals("yes");
                run.log(run.who(player) + " pick " + (yes ? "yes" : "no") + " [pay to prevent: " + cost + " for " + sa.getHostCard().getName() + "]");
                if (!yes) return false;
                if (!ComputerUtilCost.canPayCost(cost, sa, player, true)) { run.log("  (cannot pay)"); return false; }
                return new CostPayment(cost, sa).payComputerCosts(new AiCostDecision(player, sa, true));
            }
            boolean d = super.payCostToPreventEffect(cost, sa, alreadyPaid, allPayers); run.log(run.who(player) + " DEFAULT pay-to-prevent [" + cost + "] -> " + d); return d;
        }
        @Override public CardCollectionView chooseCardsToDelve(int genericAmount, CardCollection grave) {
            List<Card> r = pickFrom("delve", grave, 0, genericAmount, "delve " + genericAmount); if (r == null) return super.chooseCardsToDelve(genericAmount, grave);
            return new CardCollection(r);
        }
        @Override public org.apache.commons.lang3.tuple.ImmutablePair<CardCollection, CardCollection> arrangeForScry(CardCollection topN) {
            List<Card> bottom = pickFrom("scry-to-bottom", topN, 0, topN.size(), "scry"); if (bottom == null) return super.arrangeForScry(topN);
            CardCollection top = new CardCollection(topN); top.removeAll(bottom);
            return org.apache.commons.lang3.tuple.ImmutablePair.of(top, new CardCollection(bottom));
        }
        @Override public org.apache.commons.lang3.tuple.ImmutablePair<CardCollection, CardCollection> arrangeForSurveil(CardCollection topN) {
            List<Card> gy = pickFrom("surveil-to-graveyard", topN, 0, topN.size(), "surveil"); if (gy == null) return super.arrangeForSurveil(topN);
            CardCollection top = new CardCollection(topN); top.removeAll(gy);
            return org.apache.commons.lang3.tuple.ImmutablePair.of(top, new CardCollection(gy));
        }
        @Override public forge.card.ICardFace chooseSingleCardFace(SpellAbility sa, List<forge.card.ICardFace> faces, String message) {
            List<forge.card.ICardFace> r = pickFrom("card-face", faces, 1, 1, message); if (r == null) return super.chooseSingleCardFace(sa, faces, message);
            return r.isEmpty() ? null : r.get(0);
        }
    }

    /** Cost decisions the script can steer (pitch card for Force of Will, Daze's returned Island, discard/sacrifice costs); all others stay with the AI. */
    static final class ScriptCost extends AiCostDecision {
        final Seat seat;
        ScriptCost(Player p, SpellAbility sa, boolean effect, Seat seat) { super(p, sa, effect); this.seat = seat; }
        PaymentDecision scripted(String kind, CardCollectionView valid, int n) {
            String h = seat.q().peekFirst();
            if (h == null || !h.startsWith("pick")) return null;
            List<Card> r = seat.pickFrom(kind, valid, n, n, ability.getHostCard().getName());
            if (r == null || r.size() != n) return null;
            return PaymentDecision.card(new CardCollection(r));
        }
        @Override public PaymentDecision visit(CostExile cost) {
            if (cost.getType().contains("+withTypesGE") && !cost.payCostFromSource()) { // escape: any number of cards, with a collective card-type requirement
                String t = cost.getType().replaceAll("\\+withTypesGE\\d+", "");
                CardCollection valid = CardLists.getValidCards(player.getCardsIn(cost.getFrom()), t, player, source, ability);
                valid.remove(source);
                String h = seat.q().peekFirst();
                if (h != null && h.startsWith("pick")) {
                    List<Card> r = seat.pickFrom("exile-cost", valid, 0, valid.size(), ability.getHostCard().getName());
                    if (r != null) { ability.setXManaCostPaid(r.size()); return PaymentDecision.card(new CardCollection(r)); }
                }
            }
            if (!cost.payCostFromSource() && !cost.getType().equals("All") && !cost.getType().contains("FromTopGrave") && cost.zoneRestriction != 0 && !cost.getFrom().contains(ZoneType.Library)) {
                CardCollection valid = CardLists.getValidCards(player.getCardsIn(cost.getFrom()), cost.getType(), player, source, ability);
                PaymentDecision d = scripted("exile-cost", valid, Math.min(cost.getAbilityAmount(ability), valid.size())); if (d != null) return d;
            }
            return super.visit(cost);
        }
        @Override public PaymentDecision visit(CostDiscard cost) {
            if (!cost.payCostFromSource() && !cost.getType().equals("Hand") && !cost.getType().equals("LastDrawn") && !cost.getType().equals("Random") && !cost.getType().contains("With")) {
                CardCollection valid = CardLists.getValidCards(player.getCardsIn(ZoneType.Hand), cost.getType().split(";"), player, source, ability);
                PaymentDecision d = scripted("discard-cost", valid, Math.min(cost.getAbilityAmount(ability), valid.size())); if (d != null) return d;
            }
            return super.visit(cost);
        }
        @Override public PaymentDecision visit(CostReturn cost) {
            if (!cost.payCostFromSource()) {
                CardCollection valid = CardLists.getValidCards(player.getCardsIn(ZoneType.Battlefield), cost.getType(), player, source, ability);
                PaymentDecision d = scripted("return-cost", valid, Math.min(cost.getAbilityAmount(ability), valid.size())); if (d != null) return d;
            }
            return super.visit(cost);
        }
        @Override public PaymentDecision visit(CostSacrifice cost) {
            if (!cost.payCostFromSource() && !cost.getType().equals("OriginalHost") && !cost.getAmount().equals("All")) {
                CardCollection valid = CardLists.getValidCards(player.getCardsIn(ZoneType.Battlefield), cost.getType(), player, source, ability);
                PaymentDecision d = scripted("sacrifice-cost", valid, Math.min(cost.getAbilityAmount(ability), valid.size())); if (d != null) return d;
            }
            return super.visit(cost);
        }
    }

    // ------------------------------------------------------------------ lobby
    static final class Lobby extends LobbyPlayerAi implements IGameEntitiesFactory {
        final Run run; final int me;
        Lobby(String name, Run run, int me) { super(name, null); this.run = run; this.me = me; }
        @Override public PlayerController createMindSlaveController(Player master, Player slave) { return new Seat(slave.getGame(), slave, this, run, me); }
        @Override public Player createIngamePlayer(Game game, int id) {
            Player p = new Player(getName(), game, id);
            run.players[me] = p;
            p.setFirstController(new Seat(game, p, this, run, me));
            return p;
        }
        @Override public void hear(LobbyPlayer player, String message) { }
    }

    // ------------------------------------------------------------------ helpers
    static Card findCard(Run run, String spec, ZoneType zone) {
        String name = spec; String ctl = null;
        int at = spec.lastIndexOf('@'); if (at >= 0) { name = spec.substring(0, at); ctl = spec.substring(at + 1); }
        List<Card> cands = new ArrayList<>();
        if (zone == null || zone == ZoneType.Stack) for (var si : run.game.getStack()) { Card c = si.getSourceCard(); if (c.getName().equals(name)) cands.add(c); }
        for (ZoneType z : zone != null ? List.of(zone) : List.of(ZoneType.Battlefield, ZoneType.Graveyard, ZoneType.Exile, ZoneType.Hand, ZoneType.Library, ZoneType.Command))
            for (Card c : run.game.getCardsIn(z)) if (c.getName().equals(name)) cands.add(c);
        for (Card c : cands) {
            if (ctl == null) return c;
            int i = ctl.equals("p1") ? 0 : 1;
            if (c.getController() == run.pl(i)) return c;
        }
        return null;
    }

    static PaperCard card(String name) { return FModel.getMagicDb().getCommonCards().getUniqueByName(name); }

    static Deck filler(String name) {
        Deck d = new Deck(name); d.getOrCreate(DeckSection.Main).add(card("Island"), 60); return d;
    }

    // ------------------------------------------------------------------ run one scenario
    static Result runScenario(Scenario sc, long seed) {
        Result res = new Result(); res.sc = sc;
        MyRandom.setRandom(new Random(seed));
        Run run = new Run(sc, res);
        try {
            Lobby l1 = new Lobby("P1", run, 0), l2 = new Lobby("P2", run, 1);
            String profile = ((LobbyPlayerAi) GamePlayerUtil.createAiPlayer("profile-source", 0, "")).getAiProfile();
            l1.setAiProfile(profile); l2.setAiProfile(profile);
            RegisteredPlayer r1 = new RegisteredPlayer(filler("p1")), r2 = new RegisteredPlayer(filler("p2"));
            r1.setPlayer(l1); r2.setPlayer(l2);
            List<RegisteredPlayer> order = sc.swapSeats ? new ArrayList<>(List.of(r2, r1)) : new ArrayList<>(List.of(r1, r2));
            GameRules rules = new GameRules(GameType.Constructed); rules.setAppliedVariants(EnumSet.of(GameType.Constructed));
            Match mc = new Match(rules, order, "itest");
            Game game = mc.createGame(); game.setNoGUIUser();
            run.game = game;
            GameState st = new GameState();
            List<String> lines = new ArrayList<>();
            boolean hasSick = false;
            String k1 = sc.swapSeats ? "ai" : "human", k2 = sc.swapSeats ? "human" : "ai";
            for (String s : sc.state) { lines.add(s.replaceAll("^human", "@H").replaceAll("^ai", "@A").replaceAll("^p1", k1).replaceAll("^p2", k2).replace("activeplayer=p1", "activeplayer=" + k1).replace("activeplayer=p2", "activeplayer=" + k2).replaceAll("^@H", k1).replaceAll("^@A", k2)); if (s.startsWith("removesummoningsickness")) hasSick = true; }
            if (!hasSick) lines.add("removesummoningsickness=true");
            // defaults so a scenario only states what matters: 20 life, ten Islands as library (so nobody decks), turn 3, Main 1 of p1
            for (String side : List.of("human", "ai")) {
                if (lines.stream().noneMatch(x -> x.startsWith(side + "life="))) lines.add(side + "life=20");
                if (lines.stream().noneMatch(x -> x.startsWith(side + "library="))) lines.add(side + "library=" + String.join(";", Collections.nCopies(10, "Island")));
            }
            if (lines.stream().noneMatch(x -> x.startsWith("turn="))) lines.add("turn=3");
            if (lines.stream().noneMatch(x -> x.startsWith("activeplayer="))) lines.add("activeplayer=" + k1);
            if (lines.stream().noneMatch(x -> x.startsWith("activephase="))) lines.add("activephase=MAIN1");
            st.parse(lines);
            try {
                mc.startGame(game, () -> { st.applyToGame(game); run.startTurn = game.getPhaseHandler().getTurn(); });
            } catch (ScenarioDone d) { /* normal */ }
            catch (RuntimeException e) { Throwable t = e; while (t.getCause() != null && !(t instanceof ScenarioDone)) t = t.getCause(); if (!(t instanceof ScenarioDone)) throw e; }
            if (!run.finished && res.error == null && !game.isGameOver()) { res.error = "script did not finish"; }
            res.finalDump = dump(run);
            { List<GameLogEntry> le = new ArrayList<>(game.getGameLog().getLogEntries(null)); Collections.reverse(le);
              res.gameLog = le.stream().map(Object::toString).collect(Collectors.joining("\n")); }
            for (String e : sc.expect) { String bad = Expect.eval(run, e); if (bad != null) res.failures.add(e + "   <-- " + bad); }
            if (res.failures.isEmpty() && res.error == null) res.status = "PASS"; else res.status = res.error != null ? "ERROR" : "FAIL";
        } catch (Throwable t) {
            Throwable c = t; while (c.getCause() != null && c.getCause() != c) c = c.getCause();
            res.status = "ERROR"; res.error = c.getClass().getSimpleName() + ": " + c.getMessage();
            if (VERBOSE) t.printStackTrace(System.out);
            StringWriter sw = new StringWriter(); c.printStackTrace(new PrintWriter(sw)); res.trace.add(sw.toString().lines().limit(8).collect(Collectors.joining("\n")));
        }
        return res;
    }

    /** GameState.applyToGame and several engine paths require a thread named Game-*; run each scenario in one. */
    static Result inGameThread(Scenario sc, long seed) throws InterruptedException {
        final Result[] out = new Result[1];
        Thread t = new Thread(() -> out[0] = runScenario(sc, seed), "Game-itest");
        t.setDaemon(true); t.start(); t.join(90_000);
        if (out[0] == null) { Result r = new Result(); r.sc = sc; r.status = "ERROR"; r.error = "timeout (90 s)"; return r; }
        return out[0];
    }

    static String dump(Run run) {
        StringBuilder sb = new StringBuilder();
        Game g = run.game;
        sb.append("turn ").append(g.getPhaseHandler().getTurn()).append(" phase ").append(g.getPhaseHandler().getPhase()).append(" active ").append(g.getPhaseHandler().getPlayerTurn()).append("\n");
        for (int i = 0; i < 2; i++) {
            Player p = run.pl(i);
            sb.append("  p").append(i + 1).append(" life ").append(p.getLife()).append(" poison ").append(p.getPoisonCounters()).append(p.hasLost() ? " LOST" : "").append("\n");
            for (ZoneType z : List.of(ZoneType.Hand, ZoneType.Battlefield, ZoneType.Graveyard, ZoneType.Exile, ZoneType.Command))
                sb.append("    ").append(z).append(": ").append(p.getCardsIn(z).stream().map(Interact::cardStr).collect(Collectors.joining(", "))).append("\n");
            sb.append("    Library(").append(p.getCardsIn(ZoneType.Library).size()).append(") top: ").append(p.getCardsIn(ZoneType.Library).stream().limit(5).map(Card::getName).collect(Collectors.joining(", "))).append("\n");
        }
        sb.append("  Stack-zone cards: ").append(g.getCardsIn(ZoneType.Stack).stream().map(Card::getName).collect(Collectors.joining(", "))).append("\n  stack: "); for (var si : g.getStack()) sb.append(si.getSourceCard().getName()).append(" | "); sb.append("\n");
        return sb.toString();
    }
    static String cardStr(Card c) {
        StringBuilder sb = new StringBuilder(c.getName());
        if (c.isInZone(ZoneType.Battlefield)) { if (c.isTapped()) sb.append("(T)"); if (c.isCreature()) sb.append(" ").append(c.getNetPower()).append("/").append(c.getNetToughness()); }
        if (!c.getCounters().isEmpty()) sb.append(c.getCounters().toString());
        if (c.isToken()) sb.append("[token]");
        if (c.getOwner() != c.getController() && c.getController() != null) sb.append("[ctl ").append(c.getController().getName()).append("]");
        return sb.toString();
    }

    // ------------------------------------------------------------------ main
    public static void main(String[] args) throws Exception {
        System.setProperty("java.awt.headless", "true");
        List<String> files = new ArrayList<>(); String only = null; String out = null;
        for (int i = 0; i < args.length; i++) {
            switch (args[i]) {
                case "-v" -> VERBOSE = true;
                case "--only" -> only = args[++i];
                case "--out" -> out = args[++i];
                default -> files.add(args[i]);
            }
        }
        GuiBase.setInterface(new GuiDesktop());
        FModel.initialize(null, null);
        List<Scenario> all = new ArrayList<>();
        for (String f : files) { Path p = Paths.get(f); if (Files.isDirectory(p)) { try (var s = Files.list(p)) { for (Path q : s.sorted().toList()) if (q.toString().endsWith(".scn")) all.addAll(parse(q)); } } else all.addAll(parse(p)); }
        int pass = 0, fail = 0, err = 0, xfail = 0, xpass = 0;
        StringBuilder report = new StringBuilder();
        for (Scenario sc : all) {
            if (only != null && !sc.name.contains(only)) continue;
            if (VERBOSE) System.out.println("== " + sc.name);
            Result r = inGameThread(sc, 12345);
            boolean expectedFail = !sc.expectFail.isEmpty();
            String tag = r.status;
            if (expectedFail && r.status.equals("FAIL")) { tag = "XFAIL"; xfail++; }
            else if (expectedFail && r.status.equals("PASS")) { tag = "XPASS"; xpass++; }
            else if (r.status.equals("PASS")) pass++; else if (r.status.equals("FAIL")) fail++; else err++;
            System.out.printf("%-6s %s/%s  %s%n", tag, sc.file, sc.name, sc.title);
            if (!tag.equals("PASS")) {
                for (String f : r.failures) System.out.println("         - " + f);
                if (r.error != null) System.out.println("         ! " + r.error);
            }
            report.append(tag).append("\t").append(sc.file).append("\t").append(sc.name).append("\n");
            if (VERBOSE || !r.status.equals("PASS")) {
                System.out.println(r.gameLog.replaceAll("(?m)^", "           G "));
                if (!VERBOSE) { for (String t : r.trace) System.out.println("           | " + t.replace("\n", "\n           | ")); System.out.println(r.finalDump.replaceAll("(?m)^", "           > ")); }
                else System.out.println(r.finalDump.replaceAll("(?m)^", "      > "));
            }
        }
        System.out.printf("%nTOTAL pass=%d fail=%d error=%d xfail=%d xpass=%d%n", pass, fail, err, xfail, xpass);
        System.exit(0);
    }
}
