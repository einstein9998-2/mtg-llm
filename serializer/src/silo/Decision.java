package silo;

import java.util.ArrayList;
import java.util.List;

/** One masked decision handed to a Policy. Options are the only things a player may pick. */
public final class Decision {
    public int id;
    public String kind;          // Priority, Mulligan, BottomCards, Target, Attack, Block, YesNo, ChooseCards, OrderTriggers, Mode, Number, Name
    public String header;        // one-line question
    public final List<Opt> opts = new ArrayList<>();
    public int min = 1, max = 1; // number of options to pick
    public boolean first;        // first decision after the game state changed: gets the full/diff state block
    public String prompt;        // fully rendered text for an LLM
    public int aiHint = -1;      // index of what the Forge AI would pick, -1 unknown. Never put in the prompt.

    public static final class Opt {
        public final String label;   // canonical short text, card names only
        public final String type;    // pass, land, spell, ability, target, card, yes, no, ...
        public Object payload;       // engine object, never serialized
        public Opt(String type, String label, Object payload) { this.type = type; this.label = label; this.payload = payload; }
    }

    public Decision add(String type, String label, Object payload) { opts.add(new Opt(type, label, payload)); return this; }
}
