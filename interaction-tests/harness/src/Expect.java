package itest;

import forge.game.*;
import forge.game.card.*;
import forge.game.player.Player;
import forge.game.zone.ZoneType;

import java.util.*;
import java.util.regex.*;
import java.util.stream.*;

/** Expectation grammar. Returns null when the expectation holds, else a short description of what was found. */
final class Expect {
    static final Pattern CMP = Pattern.compile("^(.*?)\\s*(==|>=|<=|!=|>|<)\\s*(-?\\d+)$");

    static Player pl(Interact.Run r, String s) { return r.pl(s.trim().equals("p1") ? 0 : 1); }
    static ZoneType zone(String z) { return switch (z) { case "hand" -> ZoneType.Hand; case "battlefield", "bf" -> ZoneType.Battlefield; case "graveyard", "gy" -> ZoneType.Graveyard; case "exile" -> ZoneType.Exile; case "library", "lib" -> ZoneType.Library; case "command" -> ZoneType.Command; default -> throw new IllegalArgumentException("zone " + z); }; }

    static boolean cmp(int have, String op, int want) {
        return switch (op) { case "==" -> have == want; case "!=" -> have != want; case ">=" -> have >= want; case "<=" -> have <= want; case ">" -> have > want; case "<" -> have < want; default -> false; };
    }

    static List<Card> cards(Interact.Run r, Player p, ZoneType z, String name) {
        List<Card> out = new ArrayList<>();
        for (Card c : p.getCardsIn(z)) if (name == null || c.getName().equals(name)) out.add(c);
        return out;
    }

    static String eval(Interact.Run r, String e) {
        e = e.trim();
        String[] w = e.split("\\s+", 2); String cmd = w[0]; String rest = w.length > 1 ? w[1].trim() : "";
        switch (cmd) {
            case "life": {
                Matcher m = CMP.matcher(rest); if (!m.matches()) return "bad syntax"; Player p = pl(r, m.group(1));
                return cmp(p.getLife(), m.group(2), Integer.parseInt(m.group(3))) ? null : "life is " + p.getLife();
            }
            case "zone": { // zone p1 graveyard has "Name" | lacks "Name" | size == N | count "Name" == N
                String[] t = rest.split("\\s+", 3); Player p = pl(r, t[0]); ZoneType z = zone(t[1]); String tail = t[2].trim();
                if (tail.startsWith("has ")) { String n = unq(tail.substring(4)); return cards(r, p, z, n).isEmpty() ? "not found; zone has: " + names(p, z) : null; }
                if (tail.startsWith("lacks ")) { String n = unq(tail.substring(6)); return cards(r, p, z, n).isEmpty() ? null : "found " + n; }
                if (tail.startsWith("size")) { Matcher m2 = Pattern.compile("^size\\s*(==|>=|<=|!=|>|<)\\s*(-?\\d+)$").matcher(tail); if (!m2.matches()) return "bad syntax"; int have = p.getCardsIn(z).size(); return cmp(have, m2.group(1), Integer.parseInt(m2.group(2))) ? null : "size is " + have + ": " + names(p, z); }
                if (tail.startsWith("count ")) { Matcher m2 = Pattern.compile("^count\\s+(\"[^\"]+\"|\\S+)\\s*(==|>=|<=|!=|>|<)\\s*(-?\\d+)$").matcher(tail); if (!m2.matches()) return "bad syntax"; int have = cards(r, p, z, unq(m2.group(1))).size(); return cmp(have, m2.group(2), Integer.parseInt(m2.group(3))) ? null : "count is " + have + ": " + names(p, z); }
                return "bad zone clause";
            }
            case "owner-zone": { // cards OWNED by a player in a zone regardless of controller
                String[] t = rest.split("\\s+", 3); Player p = pl(r, t[0]); ZoneType z = zone(t[1]); String tail = t[2].trim();
                boolean has = false; String n = unq(tail.replaceFirst("^(has|lacks)\\s+", ""));
                for (Player q : r.game.getPlayers()) for (Card c : q.getCardsIn(z)) if (c.getOwner() == p && c.getName().equals(n)) has = true;
                return tail.startsWith("has") == has ? null : (has ? "found" : "not found");
            }
            case "stack": {
                if (rest.equals("empty")) return r.game.getStack().isEmpty() ? null : "stack has " + r.game.getStack().size();
                Matcher m = Pattern.compile("^size\\s*(==|>=|<=)\\s*(\\d+)$").matcher(rest); if (m.matches()) return cmp(r.game.getStack().size(), m.group(1), Integer.parseInt(m.group(2))) ? null : "stack size " + r.game.getStack().size();
                return "bad stack clause";
            }
            case "tapped": case "untapped": {
                String[] t = rest.split("\\s+", 2); Player p = pl(r, t[0]); String n = unq(t[1]);
                List<Card> cs = cards(r, p, ZoneType.Battlefield, n); if (cs.isEmpty()) return "no such permanent";
                boolean any = cs.stream().anyMatch(c -> c.isTapped() == cmd.equals("tapped"));
                return any ? null : "state is " + (cmd.equals("tapped") ? "untapped" : "tapped");
            }
            case "counters": { // counters p1 "Name" P1P1 == 2
                Matcher m = Pattern.compile("^(p[12])\\s+(\"[^\"]+\"|\\S+)\\s+(\\S+)\\s*(==|>=|<=|!=)\\s*(\\d+)$").matcher(rest); if (!m.matches()) return "bad syntax";
                List<Card> cs = cards(r, pl(r, m.group(1)), ZoneType.Battlefield, unq(m.group(2))); if (cs.isEmpty()) return "no such permanent";
                var ct = CounterType.getType(m.group(3)); int have = cs.get(0).getCounters(ct);
                return cmp(have, m.group(4), Integer.parseInt(m.group(5))) ? null : "counters " + have;
            }
            case "pt": { // pt p1 "Name" 3/4
                Matcher m = Pattern.compile("^(p[12])\\s+(\"[^\"]+\"|\\S+)\\s+(-?\\d+)/(-?\\d+)$").matcher(rest); if (!m.matches()) return "bad syntax";
                List<Card> cs = cards(r, pl(r, m.group(1)), ZoneType.Battlefield, unq(m.group(2))); if (cs.isEmpty()) return "no such permanent";
                Card c = cs.get(0); return c.getNetPower() == Integer.parseInt(m.group(3)) && c.getNetToughness() == Integer.parseInt(m.group(4)) ? null : "P/T is " + c.getNetPower() + "/" + c.getNetToughness();
            }
            case "creature": case "notcreature": {
                String[] t = rest.split("\\s+", 2); List<Card> cs = cards(r, pl(r, t[0]), ZoneType.Battlefield, unq(t[1])); if (cs.isEmpty()) return "no such permanent";
                return cs.get(0).isCreature() == cmd.equals("creature") ? null : "isCreature=" + cs.get(0).isCreature();
            }
            case "controls": {
                String[] t = rest.split("\\s+", 2); return cards(r, pl(r, t[0]), ZoneType.Battlefield, unq(t[1])).isEmpty() ? "not controlled" : null;
            }
            case "log": { // log has "text" / log lacks "text": game log + script trace, lowercase substring
                String n = unq(rest.replaceFirst("^(has|lacks)\\s+", "")).toLowerCase();
                String all = r.game.getGameLog().getLogEntries(null).stream().map(Object::toString).collect(Collectors.joining("\n")).toLowerCase();
                boolean has = all.contains(n);
                return rest.startsWith("has") == has ? null : (has ? "present in game log" : "absent in game log");
            }
            case "trace": { // script trace only (CASTFAIL markers, choices offered)
                String n = unq(rest.replaceFirst("^(has|lacks)\\s+", "")).toLowerCase(); boolean has = String.join("\n", r.res.trace).toLowerCase().contains(n);
                return rest.startsWith("has") == has ? null : (has ? "present in trace" : "absent in trace");
            }
            case "lost": case "alive": { Player p = pl(r, rest); return p.hasLost() == cmd.equals("lost") ? null : (p.hasLost() ? "lost" : "still in game"); }
            case "gameover": return r.game.isGameOver() ? null : "game still running";
            case "phase": { return r.game.getPhaseHandler().getPhase().name().equalsIgnoreCase(rest) ? null : "phase " + r.game.getPhaseHandler().getPhase(); }
            case "mana": {
                Matcher m = CMP.matcher(rest); if (!m.matches()) return "bad syntax"; Player p = pl(r, m.group(1)); int have = p.getManaPool().totalMana();
                return cmp(have, m.group(2), Integer.parseInt(m.group(3))) ? null : "pool " + have;
            }
            case "draws": {
                Matcher m = CMP.matcher(rest); if (!m.matches()) return "bad syntax"; Player p = pl(r, m.group(1)); int have = p.getNumDrawnThisTurn();
                return cmp(have, m.group(2), Integer.parseInt(m.group(3))) ? null : "drawn " + have;
            }
            case "spells": { Matcher m = CMP.matcher(rest); if (!m.matches()) return "bad syntax"; Player p = pl(r, m.group(1)); int have = p.getSpellsCastThisTurn(); return cmp(have, m.group(2), Integer.parseInt(m.group(3))) ? null : "spells cast " + have; }
            case "top": { // top p1 library 1 "Name": the Nth card from the top of a library
                Matcher m = Pattern.compile("^(p[12])\\s+(\\d+)\\s+(\"[^\"]+\"|\\S+)$").matcher(rest); if (!m.matches()) return "bad syntax";
                List<Card> lib = new ArrayList<>(pl(r, m.group(1)).getCardsIn(ZoneType.Library)); int i = Integer.parseInt(m.group(2)) - 1;
                if (i >= lib.size()) return "library has only " + lib.size() + " cards";
                return lib.get(i).getName().equals(unq(m.group(3))) ? null : "card " + (i + 1) + " is " + lib.get(i).getName() + "; top cards: " + lib.stream().limit(6).map(Card::getName).collect(Collectors.joining(", "));
            }
            case "not": { String bad = eval(r, rest); return bad == null ? "expectation unexpectedly true: " + rest : null; }
            default: return "unknown expectation '" + cmd + "'";
        }
    }

    static String unq(String s) { s = s.trim(); if (s.startsWith("\"") && s.endsWith("\"") && s.length() >= 2) s = s.substring(1, s.length() - 1); return s; }
    static String names(Player p, ZoneType z) { return p.getCardsIn(z).stream().map(Card::getName).collect(Collectors.joining(", ")); }
}
