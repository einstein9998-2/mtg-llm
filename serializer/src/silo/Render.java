package silo;

import com.google.common.collect.Multiset;
import forge.game.GameEntity;
import forge.game.card.Card;
import forge.game.card.CounterType;
import forge.game.player.Player;
import forge.game.spellability.SpellAbility;
import forge.game.spellability.SpellAbilityStackInstance;
import forge.game.spellability.TargetChoices;

import java.util.ArrayList;
import java.util.List;

/** Observer-safe text for single objects. Everything printable about a game object goes through here. */
public final class Render {
    final Knowledge k;
    public Render(Knowledge k) { this.k = k; }

    public String player(Player p) { return p.equals(k.me) ? "you" : "opp"; }

    public String card(Card c) { return k.ref(c); }

    /** Permanent with state flags, e.g. Thalia, Guardian of Thraben#9 2/1 (T, sick). */
    public String permanent(Card c) {
        StringBuilder sb = new StringBuilder(card(c));
        if (c.isCreature()) sb.append(' ').append(c.getNetPower()).append('/').append(c.getNetToughness());
        List<String> f = new ArrayList<>();
        if (c.isTapped()) f.add("T");
        if (c.isCreature() && c.isSick() && c.getController().equals(k.me) && c.getGame().getPhaseHandler().isPlayerTurn(k.me)) f.add("sick"); // only matters on your own turn
        if (c.getGame().getCombat() != null && c.isAttacking()) f.add("att");
        if (c.getDamage() > 0) f.add("dmg" + c.getDamage());
        if (c.isToken()) f.add("token");
        Multiset<CounterType> cs = c.getCounters();
        for (CounterType t : cs.elementSet()) f.add(cs.count(t) + "x" + t.toString());
        if (c.getEquipping() != null) f.add("on " + card(c.getEquipping()));
        else if (c.getEnchantingCard() != null) f.add("on " + card(c.getEnchantingCard()));
        if (!f.isEmpty()) sb.append(" (").append(String.join(", ", f)).append(')');
        return sb.toString();
    }

    public String entity(Object o) {
        if (o instanceof Player p) return player(p);
        if (o instanceof Card c) return card(c);
        if (o instanceof SpellAbility sa) return "spell:" + sa.getHostCard().getName();
        return String.valueOf(o);
    }

    public String targets(SpellAbility sa) {
        List<String> out = new ArrayList<>();
        for (SpellAbility s = sa; s != null; s = s.getSubAbility()) {
            TargetChoices tc = s.getTargets();
            if (tc == null) continue;
            for (GameEntity e : tc.getTargetEntities()) out.add(entity(e));
            for (SpellAbility t : tc.getTargetSpells()) out.add(entity(t));
        }
        return String.join(", ", out);
    }

    public String stackItem(SpellAbilityStackInstance si) {
        SpellAbility sa = si.getSpellAbility();
        Card host = sa.getHostCard();
        String kind = si.isSpell() ? "" : (si.isTrigger() ? "trigger of " : "ability of ");
        String tg = targets(sa);
        return kind + card(host) + " (" + player(si.getActivatingPlayer()) + ")" + (tg.isEmpty() ? "" : " -> " + tg);
    }
}
