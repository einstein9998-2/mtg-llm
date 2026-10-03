package silo;

import forge.LobbyPlayer;
import forge.ai.ComputerUtil;
import forge.ai.ComputerUtilMana;
import forge.game.cost.CostPayment;
import forge.card.ICardFace;
import forge.game.Game;
import forge.game.GameEntity;
import forge.game.GameObject;
import forge.game.ability.effects.CharmEffect;
import forge.game.card.Card;
import forge.game.card.CardCollection;
import forge.game.card.CardCollectionView;
import forge.game.card.CardUtil;
import forge.game.combat.Combat;
import forge.game.combat.CombatUtil;
import forge.game.phase.PhaseType;
import forge.game.player.DelayedReveal;
import forge.game.player.Player;
import forge.game.player.PlayerActionConfirmMode;
import forge.game.player.PlayerController;
import forge.game.spellability.*;
import forge.game.trigger.WrappedAbility;
import forge.game.zone.Zone;
import forge.game.zone.ZoneType;
import forge.util.IterableUtil;
import forge.util.collect.FCollectionView;

import java.util.*;
import java.util.function.Predicate;

/**
 * Player controller that turns Forge's decision callbacks into masked Decisions for a Policy.
 *
 * Forge's own AI (PlayerControllerAi) is the base class so that every decision type we have not modelled yet still works.
 * Those fall through to TracedAi, which counts them as "delegated". Decisions modelled here are enumerated by us from engine
 * predicates (never from the AI's desirability logic) and only options are ever offered, so an illegal index is rejected.
 */
public final class SeatController extends TracedAi {
    private final Game game;
    private final Set<SpellAbility> aiPicked = Collections.newSetFromMap(new IdentityHashMap<>());
    private boolean mirror() { return h.policy instanceof Policy.Mirror; }
    private Player opp() { for (Player p : game.getPlayers()) if (!p.equals(player)) return p; return null; }

    public SeatController(Game g, Player p, LobbyPlayer lp, Harness h) {
        super(g, p, lp, h);
        this.game = g;
    }

    // ------------------------------------------------------------------ core ask()

    /** Build, log and put a decision to the policy. Returns the picked indices, or null if the policy delegated. */
    private int[] ask(String kind, String header, List<Decision.Opt> opts, int min, int max, boolean withState) {
        Decision d = new Decision();
        d.id = ++h.decisionSeq; d.kind = kind; d.header = header; d.min = min; d.max = max; d.opts.addAll(opts); d.first = withState;
        return ask(d);
    }

    private int[] ask(Decision d) {
        if (d.opts.size() <= d.min && d.min == d.max) { // forced: nothing to choose
            h.stats.inc(h.stats.forced, d.kind);
            int[] all = new int[d.opts.size()];
            for (int i = 0; i < all.length; i++) all[i] = i;
            return all;
        }
        long t0 = System.nanoTime();
        d.prompt = h.ser.prompt(d, d.first);
        h.stats.promptNanos += System.nanoTime() - t0; h.stats.promptCalls++;
        h.stats.inc(h.stats.asked, d.kind);
        h.stats.addChars(d.kind, d.prompt.length());
        h.stats.charsFull.merge(d.kind, (long) (h.ser.peekFull().length() + h.ser.prompt(d, false).length()), Long::sum);
        if (h.onPrompt != null) h.onPrompt.accept(d);
        int[] pick = h.policy.choose(d);
        if (pick != null) {
            String err = validate(d, pick);
            if (err != null) { // an illegal index is an agent bug: reject, record, fall back to the first legal choice
                h.stats.problem("policy-illegal-pick", d.kind + " " + Arrays.toString(pick) + " " + err);
                pick = new int[Math.max(d.min, 0)];
                for (int i = 0; i < pick.length; i++) pick[i] = i;
            }
        } else h.stats.delegated("policy:" + d.kind);
        if (h.log != null) {
            if (h.logPrompts) h.log.write("g", h.gameNo, "d", d.id, "turn", game.getPhaseHandler().getTurn(), "kind", d.kind, "n", d.opts.size(),
                    "pick", pick, "labels", labels(d, pick), "hint", d.aiHint, "chars", d.prompt.length(), "prompt", d.prompt);
            else h.log.write("g", h.gameNo, "d", d.id, "turn", game.getPhaseHandler().getTurn(), "kind", d.kind, "n", d.opts.size(),
                    "pick", pick, "labels", labels(d, pick), "hint", d.aiHint, "chars", d.prompt.length());
        }
        return pick;
    }

    private static String labels(Decision d, int[] pick) {
        if (pick == null) return null;
        StringBuilder sb = new StringBuilder();
        for (int i : pick) { if (sb.length() > 0) sb.append(" ; "); if (i >= 0 && i < d.opts.size()) sb.append(d.opts.get(i).label); }
        return sb.toString();
    }

    private static String validate(Decision d, int[] pick) {
        if (pick.length < d.min || pick.length > d.max) return "count " + pick.length + " not in [" + d.min + "," + d.max + "]";
        Set<Integer> seen = new HashSet<>();
        for (int i : pick) { if (i < 0 || i >= d.opts.size()) return "index out of range " + i; if (!seen.add(i)) return "duplicate " + i; }
        return null;
    }

    private static List<Decision.Opt> yesNo(String yes, String no) {
        List<Decision.Opt> l = new ArrayList<>(); l.add(new Decision.Opt("yes", yes, Boolean.TRUE)); l.add(new Decision.Opt("no", no, Boolean.FALSE)); return l;
    }

    // ------------------------------------------------------------------ priority

    private static String flagsKey(Card c) { return c.isTapped() + "/" + c.isSick() + "/" + c.getDamage() + "/" + c.getCounters() + "/" + c.getEquipping() ; }

    private static String key(SpellAbility sa) {
        Card c = sa.getHostCard();
        Zone z = c.getZone();
        return (z == null ? "?" : z.getZoneType() + "/" + z.getPlayer()) + "|" + c.getName() + "|" + sa.toString() + "|" + (sa.isBasicSpell() ? "" : "alt") + "|" + (c.isInPlay() ? flagsKey(c) : "");
    }

    /**
     * Mana (including cost increases such as Thalia) plus non-mana costs. Deliberately not ComputerUtilCost.canPayCost: that one
     * mixes in AI preferences (it refuses planeswalker ultimates at random) which are not part of legality.
     */
    private boolean canAfford(SpellAbility sa) {
        return ComputerUtilMana.canPayManaCost(sa, player, 0, false) && CostPayment.canPayAdditionalCosts(sa.getPayCosts(), sa, false, player);
    }

    /** Targeting feasibility: every targeting part of the chain needs at least its minimum number of candidates. */
    private boolean targetsAvailable(SpellAbility sa) {
        for (SpellAbility s = sa; s != null; s = s.getSubAbility()) {
            if (!s.usesTargeting()) continue;
            int min = s.getMinTargets();
            if (min > 0 && s.getTargetRestrictions().getNumCandidates(s) < min) return false;
        }
        return true;
    }

    /** Why a spell ability is not on the menu (diagnostics for Mirror comparisons). */
    private String whyNot(SpellAbility sa) {
        sa.setActivatingPlayer(player);
        if (sa.isManaAbility()) return "mana ability";
        if (!sa.canPlay()) return "canPlay=false";
        if (!canAfford(sa)) return "cannot pay";
        if (!targetsAvailable(sa)) return "no target candidates";
        return "on menu but dedupe key differs";
    }

    /**
     * Run engine queries that Forge's own helpers answer with side effects on the global RNG (ComputerUtilMana.canPayManaCost
     * shuffles mana sources) against a scratch Random, so asking "what can I do?" can never change how the game plays out.
     * Verified: a Mirror run (Forge AI decides everything) is identical to a stock `forge sim` run only with this isolation.
     */
    static <T> T quiet(java.util.function.Supplier<T> q) {
        java.util.Random saved = forge.util.MyRandom.getRandom();
        forge.util.MyRandom.setRandom(new java.util.Random(0));
        try { return q.get(); } finally { forge.util.MyRandom.setRandom(saved); }
    }

    private List<SpellAbility> legalPriorityActions() {
        List<SpellAbility> out = new ArrayList<>();
        // Forge's own getAvailableCards also adds every library's top card; we leave libraries out so the menu cannot depend on
        // library order (no deck in the pool plays from the library top; revisit if one does, e.g. Future Sight).
        CardCollection all = new CardCollection(player.getCardsIn(ZoneType.Hand));
        all.addAll(player.getCardsIn(ZoneType.Graveyard));
        all.addAll(IterableUtil.filter(player.getCardsIn(ZoneType.Command), c -> !c.isImmutable() || c.isEmblem()));
        all.addAll(game.getCardsIn(ZoneType.Exile));
        all.addAll(game.getCardsIn(ZoneType.Battlefield));
        for (Card c : all) {
            for (SpellAbility sa : c.getAllPossibleAbilities(player, false)) {
                if (sa.isManaAbility()) continue; // mana is paid automatically as part of costs in v1
                sa.setActivatingPlayer(player);
                if (!sa.canPlay()) continue;
                if (!canAfford(sa)) continue;
                if (!targetsAvailable(sa)) continue;
                out.add(sa);
            }
        }
        return out;
    }

    private String label(SpellAbility sa, int copies) {
        Card c = sa.getHostCard();
        String name = c.getZone() != null && c.getZone().getZoneType() == ZoneType.Battlefield ? h.r.card(c) : (h.k.visible(c) ? h.k.ref(c) : c.getName());
        String s;
        if (sa.isLandAbility()) s = "Play land " + name;
        else if (sa.isSpell()) {
            String cost = sa.getPayCosts() == null ? "" : sa.getPayCosts().getTotalMana().toString();
            s = "Cast " + name + (cost.isEmpty() || cost.equals("{0}") && !c.getManaCost().isZero() ? "" : " " + cost) + (sa.isBasicSpell() ? "" : " [alt: " + trim(sa.toString().replace(c.getName(), "~"), 60) + "]");
        } else s = "Activate " + name + ": " + trim(sa.toString().replace(c.getName(), "~"), 100);
        return copies > 1 ? s + " (x" + copies + ")" : s;
    }

    private static String trim(String s, int n) { s = s.replaceAll("\\s*\\[[^\\]]*\\]\\s*", " ").replaceAll("\\s+", " ").trim(); return s.length() <= n ? s : s.substring(0, n - 1) + "…"; }

    /** Stop rules. Returns null when we should prompt, otherwise the reason priority is auto-passed. */
    private String autoPassReason(List<SpellAbility> menu) {
        if (menu.isEmpty()) return "forced:only-pass";
        StopConfig sc = h.stops;
        if (!game.getStack().isEmpty()) {
            boolean oppItem = false;
            for (SpellAbilityStackInstance si : game.getStack()) if (!si.getActivatingPlayer().equals(player)) oppItem = true;
            if (oppItem || sc.promptOnOwnStack) return null;
            return "stack:only-own-items";
        }
        PhaseType ph = game.getPhaseHandler().getPhase();
        boolean mine = game.getPhaseHandler().getPlayerTurn().equals(player);
        return (mine ? sc.mine : sc.theirs).contains(ph) ? null : "stop-rule:" + (mine ? "my" : "opp") + ":" + ph;
    }

    /** Canonical menu labels of the current priority window, for the non-interference test (no state change, no prompt). */
    List<String> priorityMenuLabels() {
        List<String> out = new ArrayList<>();
        Map<String, List<SpellAbility>> groups = new TreeMap<>();
        for (SpellAbility sa : quiet(this::legalPriorityActions)) groups.computeIfAbsent(key(sa), k -> new ArrayList<>()).add(sa);
        for (List<SpellAbility> g : groups.values()) out.add(label(g.get(0), g.size()));
        Collections.sort(out);
        return out;
    }

    @Override
    public List<SpellAbility> chooseSpellAbilityToPlay() {
        long t0 = System.nanoTime();
        List<SpellAbility> menuAll = quiet(this::legalPriorityActions);
        h.stats.menuNanos += System.nanoTime() - t0; h.stats.menuCalls++;
        // collapse equivalent actions (4 Plains in hand, 2 identical untapped Wastelands): one option with a multiplicity
        Map<String, List<SpellAbility>> groups = new TreeMap<>();
        for (SpellAbility sa : menuAll) groups.computeIfAbsent(key(sa), k -> new ArrayList<>()).add(sa);
        String why = autoPassReason(menuAll);
        boolean hint = mirror() || h.policy.wantsAiHint();
        List<SpellAbility> aiPick = hint && (mirror() || why == null) ? super.chooseSpellAbilityToPlay() : null;
        if (why != null) {
            if (why.startsWith("forced")) h.stats.inc(h.stats.forced, "Priority"); else h.stats.inc(h.stats.autoStop, why);
            if (mirror()) { if (aiPick != null) aiPicked.addAll(aiPick); return aiPick; }
            return null;
        }
        // canonical order: lands, spells, abilities, each by label; labels contain only observer-visible text
        List<Decision.Opt> opts = new ArrayList<>();
        for (List<SpellAbility> g : groups.values()) {
            SpellAbility rep = g.get(0);
            opts.add(new Decision.Opt(rep.isLandAbility() ? "land" : rep.isSpell() ? "spell" : "ability", label(rep, g.size()), rep));
        }
        opts.sort(Comparator.comparing((Decision.Opt o) -> o.type.equals("land") ? 0 : o.type.equals("spell") ? 1 : 2).thenComparing(o -> o.label));
        List<Decision.Opt> withPass = new ArrayList<>();
        withPass.add(new Decision.Opt("pass", "Pass", null));
        withPass.addAll(opts);
        PhaseType ph = game.getPhaseHandler().getPhase();
        Decision d = new Decision();
        d.id = ++h.decisionSeq; d.kind = "Priority"; d.first = true; d.min = 1; d.max = 1;
        d.header = "you have priority in " + ph + (game.getStack().isEmpty() ? ", stack empty" : ", stack non-empty");
        d.opts.addAll(withPass);
        if (hint) {
            int hi = 0;
            if (aiPick != null) {
                String k = key(aiPick.get(0)); hi = -1;
                for (int i = 1; i < d.opts.size(); i++) if (key((SpellAbility) d.opts.get(i).payload).equals(k)) { hi = i; break; }
                if (hi < 0) { h.stats.aiMenuMisses++; h.stats.problem("mirror:ai-pick-not-on-menu", aiPick.get(0).getHostCard().getName() + " / " + trim(aiPick.get(0).toString(), 70) + " -> " + whyNot(aiPick.get(0))); }
                else h.stats.aiMenuMatches++;
            } else h.stats.aiMenuMatches++;
            d.aiHint = hi;
        }
        int[] pick = ask(d);
        if (mirror()) { if (aiPick != null) aiPicked.addAll(aiPick); return aiPick; }
        if (pick == null) { List<SpellAbility> ai = super.chooseSpellAbilityToPlay(); if (ai != null) aiPicked.addAll(ai); return ai; }
        Decision.Opt o = d.opts.get(pick[0]);
        if (o.payload == null) return null;
        return Collections.singletonList((SpellAbility) o.payload);
    }

    @Override
    public boolean playChosenSpellAbility(SpellAbility sa) {
        if (aiPicked.remove(sa)) return super.playChosenSpellAbility(sa);
        sa.setActivatingPlayer(player);
        if (sa.isLandAbility()) {
            if (sa.canPlay()) { sa.resolve(); return true; }
            h.stats.problem("menu-action-failed", "land " + sa.getHostCard().getName());
            return false;
        }
        Card host = sa.getHostCard();
        Zone from = host.getZone();
        ZoneType fromType = from == null ? null : from.getZoneType();
        boolean ok = ComputerUtil.handlePlayingSpellAbility(player, sa, s -> { for (SpellAbility t = s; t != null; t = t.getSubAbility()) if (t.usesTargeting()) chooseTargetsFor(t); });
        if (!ok) { // enumerated as legal but the engine could not complete it: record, and put a spell stuck on the stack back
            h.stats.problem("menu-action-failed", sa.getHostCard().getName() + " / " + trim(sa.toString(), 60));
            Card now = game.getCardState(host);
            if (sa.isSpell() && now != null && now.isInZone(ZoneType.Stack) && !game.getStack().isEmpty() && game.getStack().getSpellMatchingHost(now) == null && fromType != null) {
                game.getAction().moveTo(fromType, now, null, null);
            }
        }
        return ok;
    }

    // ------------------------------------------------------------------ targets, modes, triggers

    private String zoneTag(Card c) {
        Zone z = c.getZone();
        if (z == null) return "";
        String who = c.getController().equals(player) && z.getZoneType() == ZoneType.Battlefield || c.getOwner().equals(player) ? "your" : "opp";
        return " (" + who + " " + (z.getZoneType() == ZoneType.Battlefield ? "bf" : z.getZoneType() == ZoneType.Graveyard ? "gy" : z.getZoneType().name().toLowerCase()) + ")";
    }

    @Override
    public boolean chooseTargetsFor(SpellAbility sa) {
        if (mirror() || sa.isDividedAsYouChoose() || sa.getTargetRestrictions().isRandomTarget() || sa.hasParam("TargetingPlayer")) return super.chooseTargetsFor(sa);
        sa.resetTargets();
        TargetRestrictions tr = sa.getTargetRestrictions();
        int min = sa.getMinTargets(), max = sa.getMaxTargets();
        if (max == 0 && min == 0) return true;
        boolean first = true;
        while (sa.getTargets().size() < max) {
            List<GameObject> cands = new ArrayList<>();
            for (GameEntity e : tr.getAllCandidates(sa, true)) if (!sa.getTargets().contains(e)) cands.add(e);
            if (tr.getZone().contains(ZoneType.Stack)) {
                for (SpellAbilityStackInstance si : game.getStack()) if (sa.canTargetSpellAbility(si.getSpellAbility()) && !sa.getTargets().contains(si.getSpellAbility())) cands.add(si.getSpellAbility());
            } else {
                for (Card c : CardUtil.getValidCardsToTarget(sa)) if (!sa.getTargets().contains(c)) cands.add(c);
            }
            boolean canStop = sa.getTargets().size() >= min;
            if (cands.isEmpty()) break;
            List<GameObject> sortedC = new ArrayList<>(cands);
            sortedC.sort(Comparator.comparing((GameObject o) -> o instanceof Player ? 0 : 1).thenComparing(o -> o instanceof Player p ? (p.equals(player) ? "b" : "a") : labelOf(o)));
            List<Decision.Opt> opts = new ArrayList<>();
            if (canStop) opts.add(new Decision.Opt("done", sa.getTargets().size() == 0 ? "No target" : "Done choosing targets", null));
            // visible cards get ids in canonical order via ref(); candidate order must not depend on hidden state
            for (GameObject o : sortedC) opts.add(new Decision.Opt("target", labelOf(o), o));
            String head = "targets for " + h.r.card(sa.getHostCard()) + (sa.getTargets().size() > 0 ? " (chosen: " + h.r.targets(sa) + ")" : "") + ": " + trim(sa.getTargetRestrictions().getVTSelection().replace("CARDNAME", sa.getHostCard().getName()), 80);
            Decision d = new Decision(); d.id = ++h.decisionSeq; d.kind = "Target"; d.header = head; d.opts.addAll(opts); d.min = 1; d.max = 1; d.first = false;
            int[] pick = ask(d);
            if (pick == null) return super.chooseTargetsFor(sa);
            Decision.Opt o = d.opts.get(pick[0]);
            if (o.payload == null) break;
            sa.getTargets().add((GameObject) o.payload);
            first = false;
        }
        return sa.isTargetNumberValid();
    }

    private String labelOf(GameObject o) {
        if (o instanceof Player p) return h.r.player(p);
        if (o instanceof Card c) return h.r.card(c) + zoneTag(c);
        if (o instanceof SpellAbility s) return "spell " + h.r.card(s.getHostCard());
        return String.valueOf(o);
    }

    @Override
    public List<AbilitySub> chooseModeForAbility(SpellAbility sa, List<AbilitySub> possible, int min, int num, boolean allowRepeat) {
        if (mirror() || allowRepeat) return super.chooseModeForAbility(sa, possible, min, num, allowRepeat);
        List<Decision.Opt> opts = new ArrayList<>();
        for (AbilitySub m : possible) opts.add(new Decision.Opt("mode", trim(m.getDescription().replace(sa.getHostCard().getName(), "~"), 120), m));
        Decision d = new Decision(); d.id = ++h.decisionSeq; d.kind = "Mode"; d.header = "choose mode(s) for " + h.r.card(sa.getHostCard());
        d.opts.addAll(opts); d.min = min; d.max = Math.min(num, opts.size()); d.first = false;
        int[] pick = ask(d);
        if (pick == null) return super.chooseModeForAbility(sa, possible, min, num, allowRepeat);
        List<AbilitySub> out = new ArrayList<>();
        for (int i : pick) out.add((AbilitySub) d.opts.get(i).payload);
        return out;
    }

    @Override
    public void orderAndPlaySimultaneousSa(List<SpellAbility> sas) {
        boolean simple = !mirror();
        for (SpellAbility sa : sas) if (!sa.isTrigger() || sa.isCopied() || sa.hasParam("TargetingPlayer")) simple = false;
        if (!simple) { super.orderAndPlaySimultaneousSa(sas); return; }
        List<SpellAbility> remaining = new ArrayList<>(sas), ordered = new ArrayList<>();
        // the first trigger chosen is put on the stack first and so resolves last (CR 603.3b)
        while (remaining.size() > 1) {
            List<Decision.Opt> opts = new ArrayList<>();
            for (SpellAbility sa : remaining) opts.add(new Decision.Opt("trigger", "trigger of " + h.r.card(sa.getHostCard()) + ": " + trim(sa.toString().replace(sa.getHostCard().getName(), "~"), 90), sa));
            opts.sort(Comparator.comparing(o -> o.label));
            Decision d = new Decision(); d.id = ++h.decisionSeq; d.kind = "OrderTriggers"; d.header = "pick the trigger to put on the stack next (first picked resolves last)";
            d.opts.addAll(opts); d.min = 1; d.max = 1; d.first = true;
            int[] pick = ask(d);
            SpellAbility sa = (SpellAbility) d.opts.get(pick == null ? 0 : pick[0]).payload;
            ordered.add(sa); remaining.remove(sa);
        }
        ordered.addAll(remaining);
        for (SpellAbility sa : ordered) {
            if (sa.getApi() == forge.game.ability.ApiType.Charm && !CharmEffect.makeChoices(sa)) continue;
            SpellAbility target = sa;
            boolean ok = true;
            for (SpellAbility t = target; t != null && ok; t = t.getSubAbility()) if (t.usesTargeting()) ok = chooseTargetsFor(t);
            if (ok) ComputerUtil.playStack(sa, player, game);
        }
    }

    @Override
    public boolean confirmTrigger(WrappedAbility wrapper) {
        if (mirror() || wrapper.isMandatory()) return super.confirmTrigger(wrapper);
        SpellAbility sa = wrapper.getWrappedAbility();
        Decision d = new Decision(); d.id = ++h.decisionSeq; d.kind = "YesNo";
        d.header = "use optional trigger of " + h.r.card(sa.getHostCard()) + ": " + trim(wrapper.toString().replace(sa.getHostCard().getName(), "~"), 100);
        d.opts.addAll(yesNo("Yes", "No")); d.first = true;
        int[] pick = ask(d);
        return pick == null ? super.confirmTrigger(wrapper) : pick[0] == 0;
    }

    @Override
    public boolean playSaFromPlayEffect(SpellAbility tgtSA) {
        boolean optional = !tgtSA.getPayCosts().isMandatory();
        if (mirror() || !(tgtSA instanceof Spell)) return super.playSaFromPlayEffect(tgtSA);
        if (optional) {
            Decision d = new Decision(); d.id = ++h.decisionSeq; d.kind = "YesNo"; d.first = true;
            d.header = "cast " + h.r.card(tgtSA.getHostCard()) + " from the effect (without paying its mana cost where stated)?";
            d.opts.addAll(yesNo("Yes", "No"));
            int[] pick = ask(d);
            if (pick != null && pick[0] == 1) return false;
        }
        tgtSA.setActivatingPlayer(player);
        return ComputerUtil.handlePlayingSpellAbility(player, tgtSA, s -> { for (SpellAbility t = s; t != null; t = t.getSubAbility()) if (t.usesTargeting()) chooseTargetsFor(t); });
    }

    // ------------------------------------------------------------------ mulligan, starting player

    @Override
    public boolean mulliganKeepHand(Player first, int cardsToReturn) {
        Decision d = new Decision(); d.id = ++h.decisionSeq; d.kind = "Mulligan"; d.first = false;
        List<Card> hand = new ArrayList<>(player.getCardsIn(ZoneType.Hand));
        hand.sort(Comparator.comparing(Card::getName));
        StringBuilder sb = new StringBuilder();
        for (Card c : hand) { if (sb.length() > 0) sb.append(", "); sb.append(h.r.card(c)); }
        d.header = "opening hand (" + hand.size() + "): " + sb + " | you are " + (first.equals(player) ? "on the play" : "on the draw") + (cardsToReturn > 0 ? " | you will bottom " + cardsToReturn : "");
        d.opts.addAll(yesNo("Keep", "Mulligan"));
        int[] pick = ask(d);
        return pick == null ? super.mulliganKeepHand(first, cardsToReturn) : pick[0] == 0;
    }

    @Override
    public CardCollectionView tuckCardsViaMulligan(CardCollectionView hand, int cardsToReturn) {
        List<Card> cards = new ArrayList<>(hand); cards.sort(Comparator.comparing(Card::getName));
        Decision d = new Decision(); d.id = ++h.decisionSeq; d.kind = "BottomCards"; d.first = false;
        d.header = "put " + cardsToReturn + " card(s) on the bottom of your library";
        for (Card c : cards) d.opts.add(new Decision.Opt("card", h.r.card(c), c));
        d.min = d.max = cardsToReturn;
        int[] pick = ask(d);
        if (pick == null) return super.tuckCardsViaMulligan(hand, cardsToReturn);
        CardCollection out = new CardCollection();
        for (int i : pick) out.add((Card) d.opts.get(i).payload);
        return out;
    }

    @Override
    public Player chooseStartingPlayer(boolean isFirstGame) {
        Decision d = new Decision(); d.id = ++h.decisionSeq; d.kind = "YesNo"; d.first = false;
        d.header = "you won the die roll: play first or draw first?";
        d.opts.addAll(yesNo("Play first", "Draw first"));
        int[] pick = ask(d);
        if (pick == null) return super.chooseStartingPlayer(isFirstGame);
        return pick[0] == 0 ? player : opp();
    }

    // ------------------------------------------------------------------ combat

    @Override
    public void declareAttackers(Player attacker, Combat combat) {
        if (mirror() || !attacker.equals(player)) { super.declareAttackers(attacker, combat); return; }
        Map<Card, GameEntity> mustAttack = combat.getAttackConstraints().getLegalAttackers().getLeft();
        List<GameEntity> defenders = new ArrayList<>();
        for (GameEntity e : combat.getDefenders()) defenders.add(e);
        List<Card> creatures = new ArrayList<>(player.getCreaturesInPlay());
        creatures.sort(Comparator.comparing(Card::getName));
        boolean firstAsk = true;
        for (Card c : creatures) {
            if (mustAttack.containsKey(c)) { combat.addAttacker(c, mustAttack.get(c)); continue; }
            List<GameEntity> ds = new ArrayList<>();
            for (GameEntity e : defenders) if (CombatUtil.canAttack(c, e)) ds.add(e);
            if (ds.isEmpty()) continue;
            List<Decision.Opt> opts = new ArrayList<>();
            opts.add(new Decision.Opt("noattack", "Do not attack", null));
            for (GameEntity e : ds) opts.add(new Decision.Opt("attack", "Attack " + h.r.entity(e), e));
            Decision d = new Decision(); d.id = ++h.decisionSeq; d.kind = "Attack"; d.first = firstAsk; firstAsk = false;
            d.header = "declare attack for " + h.r.permanent(c);
            d.opts.addAll(opts);
            int[] pick = ask(d);
            if (pick == null) { combat.clearAttackers(); super.declareAttackers(attacker, combat); return; }
            if (pick[0] > 0) combat.addAttacker(c, (GameEntity) d.opts.get(pick[0]).payload);
        }
        if (!CombatUtil.validateAttackers(combat)) {
            h.stats.problem("attack-declaration-invalid", "fell back to Forge AI declaration");
            combat.clearAttackers();
            super.declareAttackers(attacker, combat);
        }
    }

    @Override
    public void declareBlockers(Player defender, Combat combat) {
        if (mirror() || !defender.equals(player)) { super.declareBlockers(defender, combat); return; }
        List<Card> attackers = new ArrayList<>(combat.getAttackers());
        attackers.sort(Comparator.comparing(Card::getName));
        List<Card> blockers = new ArrayList<>();
        for (Card c : player.getCreaturesInPlay()) if (CombatUtil.canBlock(c, combat)) blockers.add(c);
        blockers.sort(Comparator.comparing(Card::getName));
        boolean firstAsk = true;
        for (Card b : blockers) {
            List<Decision.Opt> opts = new ArrayList<>();
            opts.add(new Decision.Opt("noblock", "Do not block", null));
            for (Card a : attackers) if (CombatUtil.canBlock(a, b, combat)) opts.add(new Decision.Opt("block", "Block " + h.r.permanent(a), a));
            if (opts.size() == 1) continue;
            Decision d = new Decision(); d.id = ++h.decisionSeq; d.kind = "Block"; d.first = true; firstAsk = false;
            d.header = "declare block for " + h.r.permanent(b);
            d.opts.addAll(opts);
            int[] pick = ask(d);
            if (pick == null) { clearBlocks(combat, attackers); super.declareBlockers(defender, combat); return; }
            if (pick[0] > 0) combat.addBlocker((Card) d.opts.get(pick[0]).payload, b);
        }
        String err = CombatUtil.validateBlocks(combat, defender);
        if (err != null) {
            h.stats.problem("block-declaration-invalid", err);
            clearBlocks(combat, attackers);
            super.declareBlockers(defender, combat);
        }
    }

    private void clearBlocks(Combat combat, List<Card> attackers) {
        for (Card a : attackers) for (Card b : new ArrayList<>(combat.getBlockers(a))) combat.removeBlockAssignment(a, b);
    }

    // ------------------------------------------------------------------ card and entity choices

    private String optionLabel(Card c, Set<Card> shown) {
        if (shown != null && shown.contains(c)) { h.k.markRevealedInOppHand(c); }
        if (h.k.visible(c)) return h.k.ref(c) + zoneTag(c);
        if (shown != null && shown.contains(c)) return c.getName() + zoneTag(c); // shown for this choice only (library search, reveal)
        return "a hidden card" + zoneTag(c);
    }

    /** Generic pick-N-cards. Hidden cards are indistinguishable; cards from a library are listed by distinct name (order is not exposed). */
    private List<Card> chooseCards(Collection<? extends Card> source, Collection<? extends Card> shown, String header, int min, int max, boolean first) {
        Set<Card> sh = shown == null ? new HashSet<>() : new HashSet<>(shown);
        List<Card> cards = new ArrayList<>(source);
        List<Decision.Opt> opts = new ArrayList<>();
        Set<String> seenNames = new HashSet<>();
        cards.sort(Comparator.comparing((Card c) -> sh.contains(c) || h.k.visible(c) ? c.getName() : "~hidden"));
        for (Card c : cards) {
            boolean lib = c.getZone() != null && c.getZone().is(ZoneType.Library);
            String lab = lib && (sh.contains(c)) ? c.getName() : optionLabel(c, sh);
            if ((lib || lab.startsWith("a hidden card")) && !seenNames.add(lab)) { // dedupe indistinguishable options
                continue;
            }
            opts.add(new Decision.Opt("card", lab, c));
        }
        Decision d = new Decision(); d.id = ++h.decisionSeq; d.kind = "ChooseCards"; d.first = first; d.header = header;
        d.opts.addAll(opts); d.min = Math.min(min, opts.size()); d.max = Math.min(Math.max(max, d.min), opts.size());
        int[] pick = ask(d);
        if (pick == null) return null;
        List<Card> out = new ArrayList<>();
        for (int i : pick) out.add((Card) d.opts.get(i).payload);
        return out;
    }

    @Override
    public CardCollectionView chooseCardsForEffect(CardCollectionView sourceList, SpellAbility sa, String title, int min, int max, boolean isOptional, Map<String, Object> params) {
        if (mirror() || sourceList.isEmpty()) return super.chooseCardsForEffect(sourceList, sa, title, min, max, isOptional, params);
        List<Card> r = chooseCards(sourceList, null, h.r.card(sa.getHostCard()) + ": " + trim(title, 90), isOptional ? 0 : min, max, true);
        return r == null ? super.chooseCardsForEffect(sourceList, sa, title, min, max, isOptional, params) : new CardCollection(r);
    }

    @Override
    public <T extends GameEntity> T chooseSingleEntityForEffect(FCollectionView<T> optionList, DelayedReveal delayedReveal, SpellAbility sa, String title, boolean isOptional, Player relatedPlayer, Map<String, Object> params) {
        if (mirror() || optionList.isEmpty()) return super.chooseSingleEntityForEffect(optionList, delayedReveal, sa, title, isOptional, relatedPlayer, params);
        List<Decision.Opt> opts = new ArrayList<>();
        Set<Card> shown = shownCards(delayedReveal);
        List<T> items = new ArrayList<>(); for (T t : optionList) items.add(t);
        items.sort(Comparator.comparing((T t) -> t instanceof Player ? "0" : t instanceof Card c ? (h.k.visible(c) || shown.contains(c) ? c.getName() : "~hidden") : t.toString()));
        Set<String> seen = new HashSet<>();
        if (isOptional) opts.add(new Decision.Opt("none", "None", null));
        for (T t : items) {
            String lab = t instanceof Card c ? optionLabel(c, shown) : h.r.entity(t);
            if (lab.startsWith("a hidden card") && !seen.add(lab)) continue;
            opts.add(new Decision.Opt("entity", lab, t));
        }
        Decision d = new Decision(); d.id = ++h.decisionSeq; d.kind = "ChooseCards"; d.first = true;
        d.header = h.r.card(sa.getHostCard()) + ": " + trim(title, 90); d.opts.addAll(opts);
        int[] pick = ask(d);
        if (pick == null) return super.chooseSingleEntityForEffect(optionList, delayedReveal, sa, title, isOptional, relatedPlayer, params);
        @SuppressWarnings("unchecked") T res = (T) d.opts.get(pick[0]).payload;
        return res;
    }

    private Set<Card> shownCards(DelayedReveal dr) {
        Set<Card> s = new HashSet<>();
        if (dr != null) for (forge.game.card.CardView cv : dr.getCards()) { Card c = game.findById(cv.getId()); if (c != null) s.add(c); }
        return s;
    }

    @Override
    public Card chooseSingleCardForZoneChange(ZoneType destination, List<ZoneType> origin, SpellAbility sa, CardCollection fetchList, DelayedReveal delayedReveal, String selectPrompt, boolean isOptional, Player decider) {
        if (mirror() || !decider.equals(player) || fetchList.isEmpty()) return super.chooseSingleCardForZoneChange(destination, origin, sa, fetchList, delayedReveal, selectPrompt, isOptional, decider);
        Set<Card> shown = shownCards(delayedReveal);
        // the decider sees every card offered here (search results, reveal); library cards are shown by name only
        shown.addAll(fetchList);
        List<Card> r = chooseCards(fetchList, shown, h.r.card(sa.getHostCard()) + ": " + trim(selectPrompt, 90) + (isOptional ? " (may choose none)" : ""), isOptional ? 0 : 1, 1, true);
        if (r == null) return super.chooseSingleCardForZoneChange(destination, origin, sa, fetchList, delayedReveal, selectPrompt, isOptional, decider);
        return r.isEmpty() ? null : r.get(0);
    }

    @Override
    public CardCollection chooseCardsToDiscardFrom(Player p, SpellAbility sa, CardCollection validCards, int min, int max, CardCollectionView visibleToChooser) {
        if (mirror() || validCards.isEmpty()) return super.chooseCardsToDiscardFrom(p, sa, validCards, min, max, visibleToChooser);
        Set<Card> shown = new HashSet<>(); if (visibleToChooser != null) for (Card c : visibleToChooser) shown.add(c);
        List<Card> r = chooseCards(validCards, shown, (sa == null ? "discard" : h.r.card(sa.getHostCard()) + ": " + (p.equals(player) ? "you discard" : "opp discards")) + " " + min + (max > min ? "-" + max : ""), min, max, true);
        return r == null ? super.chooseCardsToDiscardFrom(p, sa, validCards, min, max, visibleToChooser) : new CardCollection(r);
    }

    @Override
    public CardCollectionView chooseCardsToDiscardToMaximumHandSize(int numDiscard) {
        if (mirror()) return super.chooseCardsToDiscardToMaximumHandSize(numDiscard);
        List<Card> r = chooseCards(player.getCardsIn(ZoneType.Hand), null, "discard to hand size: " + numDiscard, numDiscard, numDiscard, true);
        return r == null ? super.chooseCardsToDiscardToMaximumHandSize(numDiscard) : new CardCollection(r);
    }

    @Override
    public CardCollectionView choosePermanentsToSacrifice(SpellAbility sa, int min, int max, CardCollectionView validTargets, String message) {
        if (mirror() || validTargets.isEmpty()) return super.choosePermanentsToSacrifice(sa, min, max, validTargets, message);
        List<Card> r = chooseCards(validTargets, null, (sa == null ? "sacrifice" : h.r.card(sa.getHostCard()) + ": sacrifice") + " " + min, min, max, true);
        return r == null ? super.choosePermanentsToSacrifice(sa, min, max, validTargets, message) : new CardCollection(r);
    }

    @Override
    public boolean confirmAction(SpellAbility sa, PlayerActionConfirmMode mode, String message, List<String> options, Card cardToShow, Map<String, Object> params) {
        if (mirror()) return super.confirmAction(sa, mode, message, options, cardToShow, params);
        Decision d = new Decision(); d.id = ++h.decisionSeq; d.kind = "YesNo"; d.first = true;
        d.header = (sa == null ? "" : h.r.card(sa.getHostCard()) + ": ") + trim(message == null ? "confirm?" : message, 140);
        d.opts.addAll(options != null && options.size() == 2 ? yesNo(options.get(0), options.get(1)) : yesNo("Yes", "No"));
        int[] pick = ask(d);
        return pick == null ? super.confirmAction(sa, mode, message, options, cardToShow, params) : pick[0] == 0;
    }

    @Override
    public String chooseCardName(SpellAbility sa, Predicate<ICardFace> cpp, String valid, String message) {
        if (mirror()) return super.chooseCardName(sa, cpp, valid, message);
        // closed card pool: the names in the two decklists (sideboards included). Cards outside the pool cannot appear in these games.
        TreeSet<String> names = new TreeSet<>();
        for (String n : h.poolNames) {
            forge.item.PaperCard pc = forge.StaticData.instance().getCommonCards().getUniqueByName(n);
            if (pc != null && cpp.test(pc.getRules().getMainPart())) names.add(n);
        }
        if (names.isEmpty()) return super.chooseCardName(sa, cpp, valid, message);
        Decision d = new Decision(); d.id = ++h.decisionSeq; d.kind = "ChooseName"; d.first = true;
        d.header = h.r.card(sa.getHostCard()) + ": name a card (from the card pool)";
        for (String n : names) d.opts.add(new Decision.Opt("name", n, n));
        int[] pick = ask(d);
        return pick == null ? super.chooseCardName(sa, cpp, valid, message) : (String) d.opts.get(pick[0]).payload;
    }

    @Override
    public Object vote(SpellAbility sa, String prompt, List<Object> options, com.google.common.collect.ListMultimap<Object, Player> votes, Player forPlayer, boolean optional) {
        if (mirror() || options.isEmpty()) return super.vote(sa, prompt, options, votes, forPlayer, optional);
        Decision d = new Decision(); d.id = ++h.decisionSeq; d.kind = "ChooseCards"; d.first = true;
        d.header = h.r.card(sa.getHostCard()) + ": vote - " + trim(prompt, 90);
        List<Object> sorted = new ArrayList<>(options);
        sorted.sort(Comparator.comparing(o -> o instanceof Card c ? c.getName() : String.valueOf(o)));
        for (Object o : sorted) d.opts.add(new Decision.Opt("vote", o instanceof Card c ? h.r.card(c) + zoneTag(c) : h.r.entity(o), o));
        int[] pick = ask(d);
        return pick == null ? super.vote(sa, prompt, options, votes, forPlayer, optional) : d.opts.get(pick[0]).payload;
    }

    private int chooseOne(String header, List<String> labels, List<?> payloads, boolean first) {
        Decision d = new Decision(); d.id = ++h.decisionSeq; d.kind = "ChooseOption"; d.first = first; d.header = header;
        for (int i = 0; i < labels.size(); i++) d.opts.add(new Decision.Opt("option", labels.get(i), payloads.get(i)));
        int[] pick = ask(d);
        return pick == null ? -1 : pick[0];
    }

    @Override
    public String chooseProtectionType(SpellAbility sa, List<String> choices) {
        if (mirror() || choices.size() < 2) return super.chooseProtectionType(sa, choices);
        List<String> sorted = new ArrayList<>(choices); Collections.sort(sorted);
        int i = chooseOne(h.r.card(sa.getHostCard()) + ": choose a protection type", sorted, sorted, true);
        return i < 0 ? super.chooseProtectionType(sa, choices) : sorted.get(i);
    }

    @Override
    public forge.game.card.CounterType chooseCounterType(List<forge.game.card.CounterType> options, SpellAbility sa, String prompt, Map<String, Object> params) {
        if (mirror() || options.size() < 2) return super.chooseCounterType(options, sa, prompt, params);
        List<forge.game.card.CounterType> sorted = new ArrayList<>(options); sorted.sort(Comparator.comparing(Object::toString));
        List<String> labels = new ArrayList<>(); for (forge.game.card.CounterType t : sorted) labels.add(t.toString());
        int i = chooseOne((sa == null ? "" : h.r.card(sa.getHostCard()) + ": ") + trim(prompt == null ? "choose a counter type" : prompt, 90), labels, sorted, true);
        return i < 0 ? super.chooseCounterType(options, sa, prompt, params) : sorted.get(i);
    }

    @Override
    public int chooseNumber(SpellAbility sa, String title, int min, int max) {
        if (mirror() || max - min < 1 || max - min > 24) return super.chooseNumber(sa, title, min, max);
        List<String> labels = new ArrayList<>(); List<Integer> vals = new ArrayList<>();
        for (int v = min; v <= max; v++) { labels.add(String.valueOf(v)); vals.add(v); }
        int i = chooseOne((sa == null ? "" : h.r.card(sa.getHostCard()) + ": ") + trim(title, 90), labels, vals, true);
        return i < 0 ? super.chooseNumber(sa, title, min, max) : vals.get(i);
    }

    @Override
    public void reveal(forge.game.card.CardCollectionView cards, ZoneType zone, Player owner, String messagePrefix, boolean addSuffix) {
        if (zone == ZoneType.Hand && !owner.equals(player)) for (Card c : cards) h.k.markRevealedInOppHand(c);
        super.reveal(cards, zone, owner, messagePrefix, addSuffix);
    }
}
