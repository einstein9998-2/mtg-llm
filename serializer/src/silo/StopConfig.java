package silo;

import forge.game.phase.PhaseType;

import java.util.EnumSet;
import java.util.Set;

/**
 * Which empty-stack priority windows are worth a prompt. A window is only ever skipped when passing is the player's
 * default and the cost is a missed instant-speed play at that step, so the presets are explicit and measured.
 * Whenever the opponent has something on the stack the player is always prompted (if any non-pass action exists).
 */
public final class StopConfig {
    public final Set<PhaseType> mine, theirs;
    public final boolean promptOnOwnStack;   // prompt when the stack holds only your own items (needed for storm-style chains)
    public final String name;

    private StopConfig(String name, Set<PhaseType> mine, Set<PhaseType> theirs, boolean ownStack) {
        this.name = name; this.mine = mine; this.theirs = theirs; this.promptOnOwnStack = ownStack;
    }

    public static StopConfig parse(String s) {
        switch (s) {
            case "all": return new StopConfig("all", EnumSet.allOf(PhaseType.class), EnumSet.allOf(PhaseType.class), true);
            case "minimal": return new StopConfig("minimal", EnumSet.of(PhaseType.MAIN1, PhaseType.MAIN2), EnumSet.noneOf(PhaseType.class), false);
            case "default": return new StopConfig("default",
                    EnumSet.of(PhaseType.MAIN1, PhaseType.COMBAT_DECLARE_BLOCKERS, PhaseType.MAIN2),
                    EnumSet.of(PhaseType.COMBAT_DECLARE_ATTACKERS, PhaseType.COMBAT_DECLARE_BLOCKERS, PhaseType.END_OF_TURN), false);
            default: throw new IllegalArgumentException("stops: all | default | minimal");
        }
    }
}
