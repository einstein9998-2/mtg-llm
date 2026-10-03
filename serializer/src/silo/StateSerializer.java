package silo;

import forge.game.Game;
import forge.game.card.Card;
import forge.game.card.CardCollectionView;
import forge.game.combat.Combat;
import forge.game.phase.PhaseHandler;
import forge.game.player.Player;
import forge.game.spellability.SpellAbilityStackInstance;
import forge.game.zone.ZoneType;

import java.util.*;

/**
 * Builds the per-decision text for one observer. Reads the engine only through Knowledge.visible()-guarded accessors:
 * opponent hand, opponent library and own library are touched only for their sizes.
 * Sections are printed in full at turn start and as changed-only diffs afterwards (the unchanged ones are omitted).
 */
public final class StateSerializer {
    private final Game game; private final Player me; private Player oppCache;
    private final Knowledge k; private final Render r; private final EventLog events;
    private static final String INJECT = System.getProperty("silo.injectLeak"); // test-only: see LeakCheck negative controls
    private final Map<String, String> last = new LinkedHashMap<>();
    private int lastTurn = -1;

    public StateSerializer(Game game, Player me, Knowledge k, Render r, EventLog events) {
        this.game = game; this.me = me; this.k = k; this.r = r; this.events = events;
    }

    /** The other player; resolved lazily because the serializer is built while players are still being created. */
    private Player opp() { if (oppCache == null) for (Player p : game.getPlayers()) if (!p.equals(me)) oppCache = p; return oppCache; }

    private static final Comparator<Card> BY_NAME_THEN_ZONE_ORDER = Comparator.comparing(Card::getName);

    /** Observer-visible cards sorted canonically: by name, ties keep the zone's own (public or own-draw) order. Ids are allocated in this order. */
    private List<Card> sorted(CardCollectionView cards) {
        List<Card> l = new ArrayList<>();
        for (Card c : cards) l.add(c);
        l.sort(BY_NAME_THEN_ZONE_ORDER); // List.sort is stable
        return l;
    }

    private String list(List<Card> cards, boolean full) {
        StringBuilder sb = new StringBuilder();
        for (Card c : cards) { if (sb.length() > 0) sb.append(", "); sb.append(full ? r.permanent(c) : r.card(c)); }
        return sb.toString();
    }

    private String battlefield(Player p) {
        List<Card> lands = new ArrayList<>(), rest = new ArrayList<>();
        for (Card c : sorted(p.getCardsIn(ZoneType.Battlefield))) {
            (c.isLand() && !c.isCreature() && c.getCounters().isEmpty() && c.getEquippedBy().isEmpty() ? lands : rest).add(c);
        }
        String s = list(rest, true);
        if (!lands.isEmpty()) s = (s.isEmpty() ? "" : s + "; ") + "lands: " + list(lands, true);
        return s.isEmpty() ? "-" : s;
    }

    private String graveyard(Player p) {
        List<Card> l = new ArrayList<>(); for (Card c : p.getCardsIn(ZoneType.Graveyard)) l.add(c); // true public order, oldest first
        return l.isEmpty() ? "-" : list(l, false);
    }

    private Map<String, String> sections() {
        k.sweep(game);
        Map<String, String> s = new LinkedHashMap<>();
        final Player opp = opp();
        PhaseHandler ph = game.getPhaseHandler();
        boolean pregame = ph.getPlayerTurn() == null; // starting-player and mulligan decisions happen before turn 1
        boolean myTurn = !pregame && ph.getPlayerTurn().equals(me);
        s.put("hdr", (pregame ? "pre-game" : "T" + ph.getTurn() + " " + (myTurn ? "your" : "opp") + " turn, " + ph.getPhase()) + " | life you " + me.getLife() + " opp " + opp.getLife()
                + (me.getPoisonCounters() > 0 || opp.getPoisonCounters() > 0 ? " | poison " + me.getPoisonCounters() + "/" + opp.getPoisonCounters() : ""));
        s.put("hand", "YOUR HAND (" + me.getCardsIn(ZoneType.Hand).size() + "): " + list(sorted(me.getCardsIn(ZoneType.Hand)), false));
        s.put("ybf", "YOUR BF: " + battlefield(me));
        s.put("obf", "OPP BF: " + battlefield(opp));
        s.put("ygy", "YOUR GY: " + graveyard(me));
        s.put("ogy", "OPP GY: " + graveyard(opp));
        String yex = exile(me), oex = exile(opp);
        if (!yex.isEmpty() || !oex.isEmpty()) s.put("ex", "EXILE: yours " + (yex.isEmpty() ? "-" : yex) + " | opp " + (oex.isEmpty() ? "-" : oex));
        // hidden zones: sizes only; known (revealed) opponent cards listed by name
        List<Card> known = new ArrayList<>();
        for (Card c : opp.getCardsIn(ZoneType.Hand)) if (k.isRevealedOppHand(c)) known.add(c);
        int oppHand = opp.getCardsIn(ZoneType.Hand).size();
        s.put("cnt", "OPP HAND " + oppHand + (known.isEmpty() ? "" : " (known: " + list(sorted(new forge.game.card.CardCollection(known)), false) + ")")
                + ", OPP LIB " + opp.getCardsIn(ZoneType.Library).size() + " | YOUR LIB " + me.getCardsIn(ZoneType.Library).size());
        StringBuilder st = new StringBuilder();
        for (SpellAbilityStackInstance si : game.getStack()) { if (st.length() > 0) st.append("; "); st.append(r.stackItem(si)); } // iteration is top-first
        s.put("stack", "STACK (top first): " + (st.length() == 0 ? "empty" : st));
        if (INJECT != null) { // negative control for LeakCheck: deliberately leak, the test must then fail
            Card oh = opp.getCardsIn(ZoneType.Hand).isEmpty() ? null : opp.getCardsIn(ZoneType.Hand).get(0);
            Card lt = me.getCardsIn(ZoneType.Library).isEmpty() ? null : me.getCardsIn(ZoneType.Library).get(0);
            if (INJECT.equals("opphand") && oh != null) s.put("leak", "LEAK " + oh.getName());
            if (INJECT.equals("libtop") && lt != null) s.put("leak", "LEAK " + lt.getName());
            if (INJECT.equals("forgeid")) { StringBuilder b = new StringBuilder("LEAK"); for (Card x : opp.getCardsIn(ZoneType.Hand)) b.append(' ').append(x.getId()); s.put("leak", b.toString()); }
            if (INJECT.equals("ophandsize-from-lib") ) s.put("leak", "LEAK " + opp.getCardsIn(ZoneType.Library).get(0).getName().length());
        }
        Combat c = ph.getCombat();
        if (c != null && !c.getAttackers().isEmpty()) {
            StringBuilder cb = new StringBuilder();
            for (Card a : c.getAttackers()) {
                if (cb.length() > 0) cb.append("; ");
                cb.append(r.card(a)).append(" -> ").append(r.entity(c.getDefenderByAttacker(a)));
                if (c.isBlocked(a)) { cb.append(" blocked by "); List<String> bl = new ArrayList<>(); for (Card b : c.getBlockers(a)) bl.add(r.card(b)); cb.append(String.join("+", bl)); }
            }
            s.put("combat", "COMBAT: " + cb);
        }
        return s;
    }

    private String exile(Player p) {
        List<Card> l = new ArrayList<>(); for (Card c : game.getCardsIn(ZoneType.Exile)) if (c.getOwner().equals(p) && k.visible(c)) l.add(c);
        return list(l, false);
    }

    /** The state part of a prompt. full=true prints everything; otherwise only the sections that changed since the previous call. */
    public String state(boolean forceFull) {
        int turn = game.getPhaseHandler().getTurn();
        boolean full = forceFull || turn != lastTurn;
        Map<String, String> now = sections();
        StringBuilder sb = new StringBuilder();
        for (Map.Entry<String, String> e : now.entrySet()) {
            boolean changed = !e.getValue().equals(last.get(e.getKey()));
            if (full || changed || e.getKey().equals("hdr")) sb.append(e.getValue()).append('\n');
        }
        for (String key : last.keySet()) if (!now.containsKey(key)) sb.append(key.equals("ex") ? "EXILE: -\n" : key.equals("combat") ? "COMBAT: none\n" : "").append("");
        List<String> ev = events.drain();
        if (!ev.isEmpty()) sb.append("LOG: ").append(String.join(" | ", ev)).append('\n');
        last.clear(); last.putAll(now); lastTurn = turn;
        return sb.toString();
    }

    /** Pure full rendering of the current state (no diff cache update, no event draining). For tests that re-render a perturbed world. */
    public String peekFull() {
        StringBuilder sb = new StringBuilder();
        for (String v : sections().values()) sb.append(v).append('\n');
        List<String> ev = events.peek();
        if (!ev.isEmpty()) sb.append("LOG: ").append(String.join(" | ", ev)).append('\n');
        return sb.toString();
    }

    /** Full prompt for a decision. */
    public String prompt(Decision d, boolean withState) {
        StringBuilder sb = new StringBuilder();
        if (withState) sb.append(state(false));
        sb.append("D").append(d.id).append(" ").append(d.kind).append(": ").append(d.header);
        if (d.max > 1 || d.min != 1) sb.append(" (pick ").append(d.min == d.max ? d.min + "" : d.min + "-" + d.max).append(")");
        sb.append('\n');
        for (int i = 0; i < d.opts.size(); i++) sb.append(i).append(' ').append(d.opts.get(i).label).append('\n');
        return sb.toString();
    }
}
