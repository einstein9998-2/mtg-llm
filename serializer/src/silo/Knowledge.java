package silo;

import forge.game.card.Card;
import forge.game.player.Player;
import forge.game.zone.Zone;
import forge.game.zone.ZoneType;

import java.util.HashMap;
import java.util.HashSet;
import java.util.Map;
import java.util.Set;

/**
 * What one observer legitimately knows, and the observer's private id space (doc 02 section 4.2).
 *
 * Forge card ids are never printed: they are assigned contiguously per card name and kept across zones, so an id of a hidden
 * card leaks which hidden cards share a name (forge-cloning/report.md section 3). Only cards that are visible to the observer
 * get a ViewId, in first-seen order. Opponent hand and every library card have no id at all, only counts.
 */
public final class Knowledge {
    public final Player me;
    private int next = 1;
    private final Map<Integer, Integer> viewIdOf = new HashMap<>();
    /** Forge ids of opponent-hand cards the observer has legitimately seen (public-zone card bounced to hand, hand reveal). */
    private final Set<Integer> revealedOppHand = new HashSet<>();

    public Knowledge(Player me) { this.me = me; }

    /** True when the observer may know this card's identity right now. Zone-based, plus revealed-card memory. */
    public boolean visible(Card c) {
        Zone z = c.getZone();
        if (z == null) return false;
        ZoneType t = z.getZoneType();
        switch (t) {
            case Battlefield: case Graveyard: case Stack: case Command:
                return !(c.isFaceDown() && !c.getController().equals(me));
            case Exile:
                return !c.isFaceDown() || c.getOwner().equals(me);
            case Hand:
                return c.getOwner().equals(me) || revealedOppHand.contains(c.getId());
            default:
                return false; // libraries: own library order and contents are not exposed in v1
        }
    }

    public void markRevealedInOppHand(Card c) { if (!c.getOwner().equals(me)) revealedOppHand.add(c.getId()); }
    public void forgetRevealed(Card c) { revealedOppHand.remove(c.getId()); }
    public boolean isRevealedOppHand(Card c) { return revealedOppHand.contains(c.getId()); }

    /**
     * A card that spends time in a hidden zone is a new object to the observer when it comes back (CR 400.7 and doc 02 section 4.2).
     * If the id survived, redrawing "the same physical copy" of a card after a shuffle would be visible in the output and would
     * correlate with the hidden library order. Called on every move into a zone the observer cannot see, and swept before each render.
     */
    public void forget(Card c) { viewIdOf.remove(c.getId()); }

    /** Drop ids of cards that are not visible right now. Backstop for moves that produced no event. */
    public void sweep(forge.game.Game game) {
        viewIdOf.keySet().removeIf(fid -> { Card c = game.findById(fid); return c == null || !visible(c); });
    }

    /** Number of ids held for currently hidden cards. Must be 0 right after a sweep; used by the non-interference test. */
    public int idsHeldForHiddenCards(forge.game.Game game) {
        int n = 0;
        for (int fid : viewIdOf.keySet()) { Card c = game.findById(fid); if (c == null || !visible(c)) n++; }
        return n;
    }

    /** The observer-space id of a visible card. Throws if asked for a hidden one: ids of hidden cards must not exist. */
    public int id(Card c) {
        if (!visible(c)) throw new IllegalStateException("ViewId requested for a card hidden from the observer");
        return viewIdOf.computeIfAbsent(c.getId(), k -> next++);
    }

    /** Printable reference: Name#id if visible, "a card" otherwise. */
    public String ref(Card c) { return visible(c) ? c.getName() + "#" + id(c) : "a card"; }
}
