# Forge interaction tests

Scripted game-state tests that check Forge's rules behaviour for the eight-deck Legacy pool (`../decks/legacy-deck-set.md`).
Each scenario builds a position, plays a fixed script of actions for both seats, then asserts the result the rules require.
230 pass, 3 are known Forge deviations marked `xfail:`. Findings are in `FINDINGS.md`.

## Run

```
cd harness
sh build.sh                        # compiles src/*.java against the Forge jar into /home/claude/itest-build
sh run.sh ../scenarios             # every scenario, about 20 s
sh run.sh -v --only brainstorm ../scenarios/13-cutter-vs-alurentell-d.scn   # one scenario, with the action trace
sh run.sh ../scenarios 2>&1 | sh filter.sh                                  # strips Forge's startup chatter
```

`FORGE_ROOT` (default `/home/claude/card-forge/forge`) is the Forge checkout; the jar must be built first
(build steps in `../forge-runner/README.md`). The harness runs from `forge-gui/` because Forge
reads `./res`. Output lines are `PASS`, `FAIL`, `ERROR` (script could not finish), `XFAIL` (expected failure happened) and `XPASS`
(an expected failure passed, so the deviation is fixed). A FAIL prints the failed expectation, the action trace, the game log and the
final position.

## Scenario format

A file holds scenarios separated by `=== name`. Keys at column 0, indented lines inside a section, `#` comments.

```
=== fow-alt-pitch-counters-stock-up
title: one line saying what the rules require
ref: Oracle text or CR rule the expectation comes from
xfail: only for a known Forge deviation: the reason (the scenario is then expected to FAIL)
swap-seats: yes            # optional, p1 becomes second in Forge's player list (seat order matters for some Forge code)
state:
  humanhand=Stock Up
  humanbattlefield=Island;Island;Island
  aihand=Force of Will;Brainstorm;Mountain
script:
  p1: cast Stock Up
  p1: pass
  p2: when-stack Stock Up
  p2: cast Force of Will ~alt -> Stock Up
  p2: pick Brainstorm
expect:
  life p2 == 19
  zone p2 exile has "Brainstorm"
```

### state

Forge `GameState` lines. `human` is p1, `ai` is p2 (`p1`/`p2` prefixes also work). Card lists use `;`, with modifiers after `|`
such as `Plains|Tapped` or `Moonshadow|Counters:M1M1=6`. Defaults when not stated: turn 3, p1's Main 1, 20 life, a library of ten
Islands (give `humanlibrary=` with nothing after it for an empty library). Tokens cannot be placed in the state; create them by
casting the card that makes them. Mana is paid by the Forge AI payer from untapped lands, so give exactly the lands the line needs.

### script

Each line is `p1:` or `p2:` plus a command. Each seat has its own queue, consumed whenever that seat gets priority or a decision.
Anything not scripted falls back to the Forge AI and shows as `DEFAULT` in the trace.

| command | meaning |
|---|---|
| `pass` | pass priority |
| `at [pN:]PHASE` | do nothing until that phase (`MAIN1`, `COMBAT_BEGIN`, `COMBAT_DECLARE_ATTACKERS`, `COMBAT_DAMAGE`, `END_OF_TURN`, ...); `p2:MAIN1` means in p2's turn |
| `wait-empty` | pass until the stack is empty |
| `when-stack Name` | pass until Name is the top item on the stack |
| `check <expectation>` | assert in the middle of the script |
| `cast` / `play` / `activate` `Name [~filter] [-> targets] [x=N]` | cast a spell, play a land or activate an ability |
| `try ...` | same, but it is fine if the action is not offered or cannot be paid (use for "this must be illegal") |
| `pick ...` | answer the next decision (see below) |
| `attack A;B -> p2` and `block A blocks B` | combat declarations |

`Name@p1` restricts to a permanent controlled by p1. `~filter` picks among a card's abilities: `~alt` (alternative cost such as Force of
Will, Daze, flashback, escape), `~hard` (the plain version), `~free`, `~cost:text`, `~#N`, or any text found in the ability
description (`~Evoke`, `~Ninjutsu`, `~Return target`, `~Add three`). Targets are card names, `p1`/`p2`, or `stack:Name` for an item on
the stack (a spell, an activated ability, a trigger). Several targeting parts (charm modes, sub-abilities) are separated by `|`:
`-> Phelia | p2`.

`pick` answers the next decision of that seat, whatever it is: a target for one of its own triggers, a card to exile or discard as
a cost, a search or surveil choice, a mode, a name, a number, `yes`/`no`, a colour. Names are matched against what the engine
offers; a name that is not offered fails the scenario (this is how "must not be legal" is tested: `pick !Name` asserts Name is
not offered or not targetable). `pick A;B;C` takes several cards at once, `pick none` and `pick all` do what they say,
`pick play` / `pick skip` decide a free-form "you may cast it" effect (Bilbo, Amped Raptor). Library orderings (Doomsday, Brainstorm
put-back via order prompts) are given top first. Decisions that arrive one card at a time (Doomsday's five picks) need one `pick`
line per card, in the order Forge asks.

### expect

`life p1 == 17`, `mana p1 == 3`, `draws p1 == 2`, `spells p1 == 1`, `zone p1 graveyard has "Name"` / `lacks` / `size == N` / `count "Name" == N`,
`owner-zone p1 exile has "Name"` (by owner, not controller), `controls p1 "Name"`, `tapped` / `untapped p1 "Name"`,
`counters p1 "Name" P1P1 == 2`, `pt p1 "Name" 3/4`, `creature` / `notcreature p1 "Name"`, `top p1 1 "Name"` (library, 1 is the top),
`stack empty` / `stack size == N`, `phase NAME`, `lost p2` / `alive p2` / `gameover`, `log has "text"` (game log and trace),
`trace has "CASTFAIL"` (a scripted cast that failed on cost or target), `not <expectation>`.

## How the harness works

`harness/src/Interact.java`: one JVM, one Forge `Game` per scenario, created with `Match.startGame` and a hook that applies the
`GameState`. Both seats are subclasses of `PlayerControllerAi` that read the script and otherwise defer to the AI.
Spells and abilities are played through a copy of Forge's own play sequence (move to stack, choose modes and targets, check
restrictions, pay costs) with a cost decider that takes scripted choices, so Force of Will's pitch card, Daze's Island or Doomsday's
pile are chosen by the script and not by the AI. Costs and effects that Forge would let the AI decide are therefore the only
unscripted parts. `harness/src/Expect.java` holds the expectation grammar.

## Files

| path | content |
|---|---|
| `scenarios/*.scn` | 233 scenarios, see the table in `FINDINGS.md` |
| `harness/` | `src/Interact.java`, `src/Expect.java`, `build.sh`, `run.sh`, `filter.sh` |
| `oracle-reference.txt` | Oracle text of the 144-card pool as stored in Forge's scripts (the reference for expectations) |
| `fixes/aluren-any-player.patch` | fix for the Aluren bug; apply with `cd forge-gui/res/cardsfolder/a && patch -p1 < .../aluren-any-player.patch`. Not applied to the shared Forge checkout. |
| `FINDINGS.md` | results, deviations found, limits, card coverage |
