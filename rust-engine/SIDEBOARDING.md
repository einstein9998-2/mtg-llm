# Sideboarding and best-of-three matches (2026-10-05)

Layered above the frozen core (`core-frozen-m5`): **no core change, no RFC.** The core plays single
games and already takes a `DeckList { main, side }` per seat in `Game::new`; a match is a loop that
builds a new decklist per game and starts a fresh game. All new code is in `crates/mtg-match`
(library), `mtg-agent` (`bo3` binary) and `mtg-fuzz` (`sbfuzz` binary). Plans are in
`sideboard-plans/`.

## Rules the match loop follows

- Best of three; a seat with two wins takes the match. A drawn game (runaway-state draw, panic or
  the decision cap) is replayed, up to five games in all.
- Game one: main decks, first player given by the caller (alternate it across matches). Later
  games: the **loser plays first** (always chooses to play; no draw option yet).
- Between games each seat applies its plan for this matchup to its **base 75**, never to the
  previous game's list. Swaps are one for one, 60 main, at most 15 side, at most four copies of a
  nonbasic; anything else is an error, not a silent fix. No plan for a pair means no change.

## What an agent may know

Hidden-information rule unchanged: nobody is told the opponent's actual swaps. The belief model
(`ExpectedModel`) takes the list the observer *expects* the opponent to play and treats any card it
sees outside that list as a surprise that displaces one random unseen expected card. With no
surprise it equals the existing uniform model. Two expectations are offered (`OppView`):

- `Plan` (default): the opponent's base 75 with the **public** plan for this matchup applied (never the
  list the opponent actually plays: `play_match` takes a separate `public` book per seat, and
  `bo3 --public <dir>` / `run_variants.sh` pass the baseline set, so a variant under test cannot reach
  the other seat's belief; fixed 2026-10-06 after review of PR #4, UR variants rerun). This is
  the prepared-player assumption: the plans are fixed public knowledge, like the decklists
  themselves. It would leak the opponent's choices if sideboarding ever becomes learned or chosen
  per game, so switch to `Main` before that.
- `Main`: the opponent's game-one main deck; their sideboard cards are surprises when first seen.

Tests: forks succeed at every decision of boarded games under both views
(`crates/mtg-match/tests/matches.rs`).

## Plans

13 plans, all provisional and mine, not from tournament data: Alurentell against UR Cutter, Dimir
Tempo, UWx Control, Doomsday, Reanimator, Boros Aggro and the mirror, plus each of those
opponents' plan back against Alurentell. Each file has a one-line reason per matchup. The plans
only use the 15 cards in the tournament sideboard of each list. `bo3 <decks> <plans> --check`
prints every swap and validates it.

Not covered: the other 21 pairs of the eight decks (they play main decks), BW Death and Taxes
vs Alurentell, plans that depend on being on the play or the draw, and plans that change between
games two and three.

## Limits worth knowing

- The nets were trained on main-deck games only. Sideboard cards are in the vocabulary (the pool is
  all 75-card lists) but their embeddings are almost untrained, so a net plays a boarded card worse
  than a main-deck card. A boarded-game result with a net therefore understates what a trained
  player would get from a plan. Self-play data from boarded matches would fix this; that is a
  training job for the PC.
- Rollout-MCTS and nets play both seats with the same strength, so the with/without-boarding
  difference measures how much a plan helps *this* player, not a human.
- Game results are noisy: 1,000 matches give about ±1.5 points of standard error on the match win
  rate.

## How to run

```
cargo build --release -p mtg-agent --bin bo3 -p mtg-fuzz --bin sbfuzz
bo3 decks sideboard-plans --check
bo3 decks sideboard-plans alurentell ur-cutter 1000 32 --net <net.bin> --threads 4 --board both|a|b|none
sbfuzz decks sideboard-plans 3000 1 500 50     # invariants every decision, replay check every 50th game
```

`--board a` boards only deck A (seat 0), so `--board none` vs `--board a` isolates what A's plan is
worth, and `--board b` the opponent's plan.

## First numbers (2026-10-05, cloud sandbox)

**Boarded-deck fuzz** (`sbfuzz decks sideboard-plans 3000 1 500 50`): 13 boarded matchups x 3,000 random games = 39,000 games, 26.2M decisions, invariants checked at every decision, replay check on every 50th game: 0 violations, 0 truncated, 2 draws. All 15 distinct sideboard cards that the plans bring in were cast or entered play. `cargo test --release --workspace` is green with the new crate.

**Does boarding help?** Alurentell vs UR Cutter, 1,000 matches per row, both seats the matchup-1 gen-3 net with 32 iterations, same match seeds (game one is identical in every row: A 569 of 1,000, a determinism check):

| Who boards | Match win rate, Alurentell | Alurentell, games 2+ |
|---|---|---|
| nobody | 58.9% | 830 of 1,497 (55.4%) |
| Alurentell only | 60.3% | 830 of 1,466 (56.6%) |
| UR Cutter only | 56.4% | 797 of 1,505 (53.0%) |
| both | 58.2% | 803 of 1,477 (54.4%) |

Reading: each plan moves its owner by 1 to 2 points in the expected direction (Alurentell +1.2 on games 2+, UR Cutter +2.4 for itself), but each row has about ±1.3 points of standard error on games 2+, so this is suggestive for UR Cutter's Pyroblast plan and not distinguishable from zero for Alurentell's. The net has never seen Carpet of Flowers, Defense Grid or Pyroblast in training, so these numbers underestimate what the plans are worth to a player who knows the cards. Treat them as a plumbing check, not a verdict on the plans.

## Update 2026-10-06: Brady's Alurentell 75, plans and variants

- Engine deck `decks/alurentell.txt` is now Brady's 75 (Tundra, Savannah, Orim's Chant in the
  sideboard); RFC 0005 added Orim's Chant to the engine.
- `sideboard-plans/alurentell.txt` has Brady's plans for Dimir Tempo, UR Cutter (baseline), UW Phelia
  (`uwx-control`), Boros (tentative), Reanimator, and a mirror baseline; his reasons are in the
  comments. Plans for Doomsday are still my draft. `sideboard-pending/` holds the Storm and Colorless
  Tron plans until those decks are in the engine.
- Plan files can differ on the play and on the draw (`vs ur-cutter on-play`), and `!oversize` allows a
  61-card main deck for experiments. `bo3 --plans-b <dir>` gives deck B its own plans (mirror: a
  variant against the baseline).
- Variant sets for testing: `sideboard-variants/` (UR Cutter), `-boros`, `-mirror`, `-uwx`.
  `tools/run_variants.sh <net> <matches> <iterations> <out dir> [threads]` plays them (set `VARIANTS=`,
  `OPP=`, `PLANS_B=`), `bo3 --trace` writes every game, `sbtrace` replays each game and records when
  each Alurentell card was drawn, cast or used, and `tools/sbvariants.py` writes the report (win rates
  with 95% intervals, differences from the baseline, card evidence, example games).
- `netremap <old snapshot.ron> <net.bin> <out.bin>` moves a trained net to a card database that gained
  cards (Chant moved 60 card indices; an old net without it would read the wrong rows). New cards get
  zero weights.
- The nets were trained on the old Alurentell list. Per-deck-pair nets exist only for pairs that were
  trained; a net that never saw a deck is no use for testing plans against it.
