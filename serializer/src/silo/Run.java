package silo;

import forge.GuiDesktop;
import forge.LobbyPlayer;
import forge.deck.Deck;
import forge.deck.DeckSection;
import forge.deck.io.DeckSerializer;
import forge.game.*;
import forge.game.player.RegisteredPlayer;
import forge.gui.GuiBase;
import forge.item.PaperCard;
import forge.model.FModel;
import forge.player.GamePlayerUtil;
import forge.util.MyRandom;
import forge.view.TimeLimitedCodeBlock;

import java.io.File;
import java.util.*;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

/**
 * Plays N games: seat A is driven by a Policy through SeatController (serialized prompts, masked actions),
 * seat B is Forge's built-in AI. Writes decisions.jsonl, games.jsonl and summary.json into --out.
 *
 *   java -cp serializer.jar:forge.jar silo.Run --decks DIR --a dnt.dck --b jund75.dck --games 10 --seed 1 --policy random|first|mirror --stops default
 */
public final class Run {
    public static void main(String[] args) throws Exception {
        Map<String, String> o = new HashMap<>();
        for (int i = 0; i < args.length; i += 2) o.put(args[i].replaceFirst("^--", ""), args[i + 1]);
        String decksDir = o.getOrDefault("decks", "."), a = o.get("a"), b = o.get("b");
        int games = Integer.parseInt(o.getOrDefault("games", "5"));
        long seed = Long.parseLong(o.getOrDefault("seed", "1"));
        String policyName = o.getOrDefault("policy", "random");
        boolean logPrompts = Boolean.parseBoolean(o.getOrDefault("log-prompts", "false"));
        final boolean leak = Boolean.parseBoolean(o.getOrDefault("leak-test", "false"));
        int timeout = Integer.parseInt(o.getOrDefault("timeout", "120"));
        File out = new File(o.getOrDefault("out", "out")); out.mkdirs();

        System.setProperty("java.awt.headless", "true");
        GuiBase.setInterface(new GuiDesktop());
        FModel.initialize(null, null);
        StopConfig stops = StopConfig.parse(o.getOrDefault("stops", "default")); // PhaseType needs the localizer, so after FModel

        Deck da = DeckSerializer.fromFile(new File(decksDir, a)), db = DeckSerializer.fromFile(new File(decksDir, b)); // pool/glossary only
        Set<String> pool = new TreeSet<>();
        for (Deck d : List.of(da, db)) for (DeckSection s : List.of(DeckSection.Main, DeckSection.Sideboard))
            if (d.has(s)) for (Map.Entry<PaperCard, Integer> e : d.get(s)) pool.add(e.getKey().getName());
        String glossary = Glossary.build(pool);
        try (java.io.PrintWriter pw = new java.io.PrintWriter(new File(out, "glossary.txt"))) { pw.print(glossary); }

        Stats stats = new Stats();
        final Policy.External ext = policyName.equals("external") ? new Policy.External(o.get("pipe-out"), o.get("pipe-in")) : null; // --pipe-out: engine->driver, --pipe-in: driver->engine
        long wins = 0, aiWins = 0, draws = 0, onPlayWins = 0, onPlayGames = 0;
        long totalMs = 0;
        try (Jsonl dec = new Jsonl(new File(out, "decisions.jsonl")); Jsonl gl = new Jsonl(new File(out, "games.jsonl"))) {
            for (int g = 0; g < games; g++) {
                final int gameNo = g; final long gSeed = seed * 1000 + g;
                MyRandom.setRandom(new Random(gSeed));
                // load the decks after seeding, like `forge sim`: building decks and players may consume randomness, and a mirror run must see the same stream
                final Deck dA = DeckSerializer.fromFile(new File(decksDir, a)), dB = DeckSerializer.fromFile(new File(decksDir, b));
                final boolean plain = policyName.equals("plain"); // no SeatController at all: both seats are stock Forge AI (framework control)
                final Policy pol = switch (policyName) {
                    case "plain" -> new Policy.Mirror();
                    case "first" -> new Policy.First();
                    case "mirror" -> new Policy.Mirror();
                    case "hybrid" -> new Policy.Hybrid(gSeed);
                    case "external" -> { ext.game = gameNo; yield ext; }
                    case "random" -> new Policy.RandomPolicy(gSeed, Double.parseDouble(o.getOrDefault("pass-bias", "0.5")));
                    default -> throw new IllegalArgumentException("policy: random|first|mirror|hybrid|external|plain");
                };
                SeatLobby seatA = plain ? null : new SeatLobby("Seat-A", game -> { Harness hh = new Harness(pol, stats, dec, stops, pool, logPrompts, gameNo, gSeed); hh.leakTest = leak; return hh; });
                // same AI profile (and the same RNG calls) as `forge sim` uses, so a mirror run can be compared with it
                LobbyPlayer lobbyA = seatA;
                if (plain) lobbyA = GamePlayerUtil.createAiPlayer(o.containsKey("simnames") ? "Ai(1)-" + da.getName() : "Seat-A", 0, "");
                else seatA.setAiProfile(((forge.ai.LobbyPlayerAi) GamePlayerUtil.createAiPlayer("profile-source", 0, "")).getAiProfile());
                LobbyPlayer lobbyB = GamePlayerUtil.createAiPlayer(o.containsKey("simnames") ? "Ai(2)-" + db.getName() : "ForgeAI-B", 0, "");
                boolean swap = g % 2 == 1; // alternate which seat is created first; the die roll still decides who plays first
                List<RegisteredPlayer> pp = new ArrayList<>();
                RegisteredPlayer ra = new RegisteredPlayer(dA), rb = new RegisteredPlayer(dB);
                ra.setPlayer(lobbyA); rb.setPlayer(lobbyB);
                if (swap) { pp.add(rb); pp.add(ra); } else { pp.add(ra); pp.add(rb); }
                GameRules rules = new GameRules(GameType.Constructed);
                rules.setAppliedVariants(EnumSet.of(GameType.Constructed));
                rules.setSimTimeout(timeout);
                Match mc = new Match(rules, pp, "Test");
                Game game = mc.createGame();
                game.setNoGUIUser();
                long t0 = System.currentTimeMillis();
                String err = null;
                try {
                    TimeLimitedCodeBlock.runWithTimeout(() -> mc.startGame(game), timeout, TimeUnit.SECONDS);
                } catch (TimeoutException e) { err = "timeout"; }
                catch (Throwable e) { err = e.getClass().getSimpleName() + ": " + e.getMessage(); e.printStackTrace(System.err); }
                finally { game.setGameOver(GameEndReason.Draw); }
                long ms = System.currentTimeMillis() - t0; totalMs += ms;
                GameOutcome oc = game.getOutcome();
                String winner = oc == null || oc.isDraw() ? "DRAW" : oc.getWinningLobbyPlayer().getName();
                boolean aWon = winner.equals("Seat-A") || winner.startsWith("Ai(1)");
                if (winner.equals("DRAW")) draws++; else if (aWon) wins++; else aiWins++;
                boolean aFirst = seatA != null && seatA.last != null && "you".equals(seatA.last.events.firstTurnOwner);
                onPlayGames += aFirst ? 1 : 0; onPlayWins += aFirst && aWon ? 1 : 0;
                stats.games++;
                gl.write("g", gameNo, "seed", gSeed, "winner", winner, "turns", game.getPhaseHandler().getTurn(), "ms", ms, "a_on_play", aFirst, "error", err);
                System.out.printf("game %d seed %d winner %s turn %d %d ms %s%n", gameNo, gSeed, winner, game.getPhaseHandler().getTurn(), ms, err == null ? "" : err);
                dec.flush(); gl.flush();
            }
        }
        Summary.write(new File(out, "summary.json"), policyName, stops.name, a, b, games, wins, aiWins, draws, onPlayGames, onPlayWins, totalMs, stats);
        Summary.print(policyName, stops.name, games, wins, aiWins, draws, totalMs, stats);
        if (ext != null) ext.close();
        System.exit(0);
    }

}
