package silo;

import com.google.common.eventbus.Subscribe;
import forge.game.Game;
import forge.game.card.Card;
import forge.game.card.CardView;
import forge.game.event.*;
import forge.game.player.Player;
import forge.game.zone.ZoneType;

import java.util.ArrayList;
import java.util.List;

/**
 * Redacted event stream for one observer. Every line is built from information the observer is entitled to:
 * moves into/out of hidden zones carry no card name unless the observer owns the card or it came from/went to a public zone.
 * Text is generated here from event fields only, never from Forge's own game log (which can contain hidden names).
 */
public final class EventLog {
    private final Game game; private final Knowledge k; private final Render render;
    private final List<String> lines = new ArrayList<>();
    public static final int KEEP = 14;
    public String firstTurnOwner;

    public EventLog(Game game, Knowledge k, Render render) { this.game = game; this.k = k; this.render = render; game.subscribeToEvents(this); }

    private String who(Player p) { return p.equals(k.me) ? "you" : "opp"; }
    private void add(String s) { lines.add(s); if (lines.size() > 200) lines.subList(0, 100).clear(); }

    /** Same lines drain() would return, without consuming them (used by tests that re-render the same state). */
    public List<String> peek() {
        int from = Math.max(0, lines.size() - KEEP);
        List<String> out = new ArrayList<>(lines.subList(from, lines.size()));
        if (from > 0) out.add(0, "(" + from + " earlier events omitted)");
        return out;
    }

    /** Lines since the last call. */
    public List<String> drain() {
        int from = Math.max(0, lines.size() - KEEP);
        List<String> out = new ArrayList<>(lines.subList(from, lines.size()));
        int dropped = from;
        lines.clear();
        if (dropped > 0) out.add(0, "(" + dropped + " earlier events omitted)");
        return out;
    }

    @Subscribe
    public void on(GameEvent e) {
        try { handle(e); } catch (RuntimeException ex) { add("(event log error: " + ex.getClass().getSimpleName() + ")"); }
    }

    private void handle(GameEvent e) {
        if (e instanceof GameEventTurnBegan t) {
            if (t.turnNumber() == 1) firstTurnOwner = t.turnOwner().getName().equals(k.me.getName()) ? "you" : "opp";
            add("-- turn " + t.turnNumber() + " (" + (t.turnOwner().getName().equals(k.me.getName()) ? "you" : "opp") + ") --");
        } else if (e instanceof GameEventCardChangeZone z) {
            Card c = game.findById(z.card().getId());
            if (c == null) return;
            ZoneType from = z.from() == null ? null : z.from().zoneType(), to = z.to() == null ? null : z.to().zoneType();
            if (from == to) return;
            Player owner = c.getOwner();
            boolean mine = owner.equals(k.me);
            // identity known to the observer after the move?
            if (to == ZoneType.Hand && from != null && from != ZoneType.Library && from != ZoneType.Hand && !mine) {
                k.markRevealedInOppHand(c); // public card returned to opp hand: observer knows what it is
            }
            if (from == ZoneType.Hand && !mine && to != ZoneType.Hand && to != ZoneType.Stack) k.forgetRevealed(c);
            boolean nameKnown = k.visible(c) || (mine && (to == ZoneType.Library || to == ZoneType.Hand))
                    || (from != null && from.isKnown() && !(from == ZoneType.Hand && !mine));
            String name = nameKnown ? (k.visible(c) ? k.ref(c) : c.getName()) : "a card";
            String o = mine ? "your" : "opp's";
            boolean nowHidden = !k.visible(c);
            if (nowHidden) k.forget(c); // a card that goes to a hidden zone returns as a new object (name above was rendered before this)
            if (from == ZoneType.Library && to == ZoneType.Hand) { add(mine ? "you draw " + name : "opp draws a card"); return; }
            if (to == ZoneType.Stack) return; // covered by the cast event
            if (from == ZoneType.Stack && to == ZoneType.Battlefield) { add(o + " " + name + " enters the battlefield"); return; }
            if (to == ZoneType.Battlefield) { add(o + " " + name + " enters the battlefield" + (from != null ? " from " + from.name().toLowerCase() : "")); return; }
            add(o + " " + name + ": " + (from == null ? "created" : from.name().toLowerCase()) + " -> " + (to == null ? "gone" : to.name().toLowerCase()));
        } else if (e instanceof GameEventSpellAbilityCast cast) {
            // describe the live top-of-stack item with our own renderer; Forge's targetDescription contains raw card ids
            forge.game.spellability.SpellAbilityStackInstance si = game.getStack().peek();
            if (si != null) add((si.isSpell() ? "cast " : "put on stack: ") + render.stackItem(si));
        } else if (e instanceof GameEventSpellResolved r) {
            if (r.hasFizzled()) add("(a spell or ability fizzled)");
        } else if (e instanceof GameEventLandPlayed lp) {
            // zone-change line already says it entered; keep a short marker with owner for clarity
        } else if (e instanceof GameEventPlayerLivesChanged l) {
            Player p = player(l.player().getId());
            if (p != null) add(who(p) + " life " + l.oldLives() + " -> " + l.newLives());
        } else if (e instanceof GameEventAttackersDeclared a) {
            List<String> names = new ArrayList<>();
            for (CardView cv : a.attackersMap().values()) { Card c = game.findById(cv.getId()); if (c != null) names.add(k.ref(c)); }
            if (!names.isEmpty()) add("attackers: " + String.join(", ", names));
        } else if (e instanceof GameEventMulligan m) {
            add(who(player(m.player().getId())) + " mulligan");
        }
    }

    private Player player(int id) { for (Player p : game.getPlayers()) if (p.getId() == id) return p; return null; }
}
