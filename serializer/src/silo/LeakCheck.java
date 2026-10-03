package silo;

import forge.game.Game;
import forge.game.card.Card;
import forge.game.card.CardCollection;
import forge.game.player.Player;
import forge.game.zone.Zone;
import forge.game.zone.ZoneType;

import java.util.*;

/**
 * Non-interference test (doc 02 section 1), run at every prompt of a real game.
 *
 * Take the live game state, render what the observer sees, then move to another world the observer cannot tell apart:
 * shuffle both libraries, shuffle the opponent's hand, and swap random unrevealed opponent-hand cards with random
 * opponent-library cards. Render again. The two renderings (and the priority menu) must be identical. Then restore the exact original zones.
 * Any difference means some output depends on hidden state: a card name, an id, an order, or a count that should not be exposed.
 */
public final class LeakCheck {
    private final Game game; private final Player me; private final Harness h; private final SeatController ctl; private final Random rng;

    public LeakCheck(Game game, Player me, Harness h, SeatController ctl, long seed) { this.game = game; this.me = me; this.h = h; this.ctl = ctl; this.rng = new Random(seed); }

    private Player opp() { for (Player p : game.getPlayers()) if (!p.equals(me)) return p; return null; }

    public void check(Decision d) {
        Player opp = opp();
        if (opp == null) return;
        Zone myLib = me.getZone(ZoneType.Library), oLib = opp.getZone(ZoneType.Library), oHand = opp.getZone(ZoneType.Hand);
        CardCollection myLibOrig = new CardCollection(myLib.getCards()), oLibOrig = new CardCollection(oLib.getCards()), oHandOrig = new CardCollection(oHand.getCards());

        String before = h.ser.peekFull();
        List<String> menuBefore = d.kind.equals("Priority") ? ctl.priorityMenuLabels() : null;
        List<String> optsBefore = new ArrayList<>(); for (Decision.Opt o : d.opts) optsBefore.add(o.label);

        // another world with the same observer information
        List<Card> myL = new ArrayList<>(myLibOrig), oL = new ArrayList<>(oLibOrig), oH = new ArrayList<>(oHandOrig);
        List<Card> swappable = new ArrayList<>(); for (Card c : oH) if (!h.k.isRevealedOppHand(c)) swappable.add(c);
        Collections.shuffle(swappable, rng);
        int swaps = Math.min(swappable.size(), Math.min(3, oL.size()));
        for (int i = 0; i < swaps; i++) {
            Card a = swappable.get(i), b = oL.get(rng.nextInt(oL.size()));
            int ia = oH.indexOf(a), ib = oL.indexOf(b);
            oH.set(ia, b); oL.set(ib, a);
        }
        Collections.shuffle(myL, rng); Collections.shuffle(oL, rng);
        // hand order is hidden too; keep revealed cards in place relative to each other is unnecessary because they are listed by name
        Collections.shuffle(oH, rng);
        try {
            myLib.setCards(myL); oLib.setCards(oL); oHand.setCards(oH);
            String after = h.ser.peekFull();
            List<String> menuAfter = d.kind.equals("Priority") ? ctl.priorityMenuLabels() : null;
            h.stats.leakChecks++;
            if (!before.equals(after)) fail("state text", before, after);
            if (menuBefore != null && !menuBefore.equals(menuAfter)) fail("priority menu", String.join("\n", menuBefore), String.join("\n", menuAfter));
        } finally {
            myLib.setCards(myLibOrig); oLib.setCards(oLibOrig); oHand.setCards(oHandOrig);
        }
        String restored = h.ser.peekFull();
        if (!before.equals(restored)) fail("restore", before, restored);
        int stale = h.k.idsHeldForHiddenCards(game);
        if (stale != 0) fail("stale ids for hidden cards", "held=" + stale, "expected 0");
    }

    private void fail(String what, String a, String b) {
        h.stats.leakFailures++;
        h.stats.problem("LEAK:" + what, "turn " + game.getPhaseHandler().getTurn() + "\n--- before\n" + a + "\n--- after\n" + b);
    }
}
