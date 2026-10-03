package silo;

import forge.LobbyPlayer;
import forge.ai.LobbyPlayerAi;
import forge.game.Game;
import forge.game.player.IGameEntitiesFactory;
import forge.game.player.Player;
import forge.game.player.PlayerController;

import java.util.function.Function;

/** Lobby entry for the seat our controller drives; creates a fresh Harness per game. */
/**
 * Extends LobbyPlayerAi on purpose: Forge looks up AI settings (profile properties) only for LobbyPlayerAi, so a plain LobbyPlayer
 * would silently run the delegated AI on built-in defaults instead of the Default profile and break comparison with `forge sim`.
 */
public final class SeatLobby extends LobbyPlayerAi implements IGameEntitiesFactory {
    private final Function<Game, Harness> harnessFactory;
    public Harness last;

    public SeatLobby(String name, Function<Game, Harness> harnessFactory) { super(name, null); this.harnessFactory = harnessFactory; }

    @Override public PlayerController createMindSlaveController(Player master, Player slave) { return new SeatController(slave.getGame(), slave, this, last); }

    @Override public Player createIngamePlayer(Game game, int id) {
        Player p = new Player(getName(), game, id);
        last = harnessFactory.apply(game);
        last.init(game, p);
        SeatController ctl = new SeatController(game, p, this, last);
        if (last.leakTest) last.onPrompt = new LeakCheck(game, p, last, ctl, last.seed)::check;
        p.setFirstController(ctl);
        return p;
    }

    @Override public void hear(LobbyPlayer player, String message) { }
}
