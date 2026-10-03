package silo;

import forge.StaticData;
import forge.card.CardRules;
import forge.item.PaperCard;

import java.util.*;

/** Cached-prefix glossary: oracle text for every card in the two decks, printed once per game, never per decision. */
public final class Glossary {
    public static String build(Collection<String> names) {
        StringBuilder sb = new StringBuilder("GLOSSARY (oracle text; per-decision prompts use names only)\n");
        for (String n : new TreeSet<>(names)) {
            PaperCard pc = StaticData.instance().getCommonCards().getUniqueByName(n);
            if (pc == null) { sb.append(n).append(": (unknown)\n"); continue; }
            CardRules r = pc.getRules();
            sb.append(n);
            if (!r.getManaCost().isNoCost()) sb.append(' ').append(r.getManaCost());
            sb.append(" | ").append(r.getType());
            if (r.getMainPart().getPower() != null && !r.getMainPart().getPower().isEmpty()) sb.append(' ').append(r.getMainPart().getPower()).append('/').append(r.getMainPart().getToughness());
            sb.append(": ").append(r.getOracleText().replace("\\n", " / ").replace("\n", " / ")).append('\n');
        }
        return sb.toString();
    }
}
