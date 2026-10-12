# Alurentell deck construction: net simulation, metrics and LLM games (2026-10-12)

For Brady. Branch `claude/project-thread-k5zuj1`, scripts in `rust-engine/tools/sideboard-experiments/`.

## Short version

- **Do not trust a net win rate for sideboard cards.** The nets were trained on main decks only, so they play Chant, Ending, Consign, Force of Negation and Emrakul badly. Every "take the sideboard card out" test comes out positive for the unboarded list (`none`, nobody boards, is +1 to +10 against five of six opponents). The same goes for "add Petals or Force of Will": the nets say worse, monotonically with copies, in UR, UB and UW. I read those as "the net does not value the swap", not as proof the swap is bad. The metrics and the card-fate counts below are the part that does not depend on how the net plays.
- **Petal and Force of Will coming in over Atraxa or Show and Tell** gives the earlier mana (+0.2 mana on turn 3 per Petal) but not an earlier combo: the share of games with Show and Tell plus payoff (or Aluren plus Acererak) by turn 4 falls (UR 42% to 36% with one Petal over Atraxa, 32% with two). More Force of Will raises protection at the first combo turn (1.27 to 1.46 and 1.53) and makes that turn later (4.48 to 4.74 and 5.12). Petal over a land is about neutral on the combo turn (39% vs 42%) and +0.03 mana. The net numbers agree in direction.
- **Chant and Ending are dead or late a lot.** The only white sources are Tundra and Savannah (plus Petal and Carpet), so in 5 to 20% of the games where you draw them (more against UR and UB) there is no way to cast them, and a drawn Ending is cast only 34% to 69% of the time depending on the matchup. Details below.
- **Consign looks like the best sideboard card against Tron and Ending a dead card there.** In the 8 Tron LLM games on the Consign and FoN-out plans, Consign countered Ugin once and Karn once, and Ending was cast once (to exile a Disruptor Flute that named Aluren). Against Storm in the LLM games Ending was a blank, apart from killing Hexing Squelcher, which cost the player the 2 life that lost a game (ward). The fate counts agree: Ending's cast games win no more often than games without it against Storm, Tron or Boros. I would not board Ending against Tron or Storm.
- **Omniscience:** keep it against Storm and Boros, take it out against UR and UB; against UW and Tron keeping one costs nothing and keeping both costs 2 to 4 points (nets). In the mirror you take it out.
- **61 cards:** nothing clearly beats the 60-card plan. Best 61st cards by the nets: Atraxa and Stock Up against Storm (+5.3, +3.0), Emrakul against Storm (+7.1, below), Aluren against UW (+1.4).
- **Emrakul** works in the engine (cast free under Omniscience with the extra turn, or put in by Show and Tell). As a 61st card it is positive in Storm (+7.1), Boros (+4.5), Tron (+3.9), UW (+3.1) and UB (+1.9) and slightly negative in UR (-1.9, noise); replacing an Atraxa is mixed (UR -4.8, Boros -2.7, Tron +2.3). Against UW it is the better Show and Tell target (95% vs 52% wins when put in) because colored removal cannot target it.
- **LLM games (16 new, plus the 24 from before):** too few to tell the plans apart. Tron went 13-1 across all arms (4-0 Consign, 4-0 FoN out, 3-0 base, 2-1 no Consign); the LLM Tron deck is weak (no notes). They are most useful for the card notes and the engine/menu issues listed at the end.

## What I ran

1. **Net vs net** (the pair net for each matchup, 32 iterations), one postboard game per cell, 500 per side (play and draw), 300 for Boros and Reanimator (net vs rollout, because there is no Boros or Reanimator net). Seeds shared across configs, so differences are paired. Anything under about 4 points is noise (95% intervals are given).
2. **Metrics from the traces.** Every game is replayed with `sbtrace` (each own main phase: hand and permanents). "Combo line" = Show and Tell plus a payoff (Atraxa, Omniscience, Emrakul) in hand with at least 3 mana, or Aluren available plus Acererak in hand. Mana counts Ancient Tomb and City as 2, plus the land drop and Petals. Protection = Veil, Chant, Consign, FoN, plus Force of Will if there is a card to pitch or 5 mana.
3. **Per-card fate** for Chant and Ending: drawn and cast, drawn and not cast with a white source available, drawn with no white source, drawn too late.
4. **LLM games**: Sonnet plays both sides with the same engine (tags t4 to t11, u4 to u7, s4 to s7).

All configs are deltas on your post-board 60 for that matchup; swap-ins can come out of the main deck as well as the sideboard. Carpet and Veil were never swapped out against tempo; Ending stays in against tempo.

## Your list, one by one

### 0 / 1 / 2 Petal after boarding, tempo decks

Petal comes in over Atraxa (a), Show and Tell (s) or a land (l). Base has 0. Net win rate change vs base (play/draw average, ± 95%):

| Matchup | 1 Petal (a / s / l) | 2 Petals (a / s / l) |
|---|---|---|
| UR Cutter | -5.4 ±3.0 / -5.2 ±3.7 / -6.7 ±4.0 | -10.4 ±3.2 / -9.4 ±3.8 / -5.2 ±4.1 |
| UB Dimir | -3.3 ±2.8 / -0.8 ±3.4 / -2.9 ±3.8 | -8.1 ±3.0 / -4.8 ±3.6 / -2.0 ±3.8 |
| UW Phelia | -0.5 ±3.1 / 0.0 ±3.8 / +1.3 ±4.1 | -8.1 ±3.2 / -1.5 ±4.1 / -3.9 ±4.0 |

Metrics (UR): 0 Petal: mana T3 3.08, combo line by T4 42%. 1 Petal over Atraxa: 3.23, 36%. 2 over Atraxa: 3.42, 32%. 1 over a land: 3.11, 39%; 2 over lands: 3.22, 39%. UB and UW look the same. Verdict: Petals as mana do not make up for what they replace, and the least bad version of the swap is over a land (one copy: UR -6.7, UB -2.9, UW +1.3).

### 0 / 1 / 2 Force of Will after boarding, tempo decks

UR and UW base keep 2 Force of Will (your plan), UB base keeps 0. "1" for UR and UW means 1 Force of Will plus Force of Negation. Change vs base:

| Matchup | 1 FoW + FoN (from 2) | 3 FoW (over Atraxa / S&T) | 4 FoW (over Atraxa / S&T) |
|---|---|---|---|
| UR | -1.1 ±3.6 | -5.4 ±2.5 / -6.1 ±3.1 | -14.6 ±2.7 / -11.3 ±3.5 |
| UW | -0.4 ±3.9 | -3.3 ±2.9 / -0.3 ±3.6 | -6.6 ±3.0 / -4.4 ±3.7 |

| UB (base 0) | 1 FoW (over Atraxa / S&T) | 2 FoW (over Atraxa / S&T) |
|---|---|---|
| change | -5.3 ±2.6 / -2.3 ±3.1 | -11.0 ±2.6 / -10.1 ±3.4 |

Metrics, UR: protection at the first combo turn 1.27 (2 FoW), 1.46 (3), 1.53 (4); first combo turn 4.48, 4.74, 5.12. Over Show and Tell it is the same shape with a worse hit. So a pitch spell is more protection but costs the card that makes the combo; against tempo the net says no. 0 Force of Will for UR and UW was not in the grid.

### How good is Chant

Games where Chant was drawn (base, 600 games per matchup, 300+300 play and draw), and what happened to it:

| Matchup | drawn | cast | held, white mana available | no white source | too late |
|---|---|---|---|---|---|
| UR | 318 | 131 (41%) | 79 (25%) | 63 (20%) | 45 (14%) |
| UB | 280 | 141 (50%) | 54 (19%) | 57 (20%) | 28 (10%) |
| UW | 365 | 252 (69%) | 69 (19%) | 28 (8%) | 16 (4%) |
| Storm | 341 | 216 (63%) | 55 (16%) | 44 (13%) | 26 (8%) |
| Mirror | 497 | 246 (50%) | 219 (44%) | 25 (5%) | 7 (1%) |
| Boros | 404 | 333 (82%) | 36 (9%) | 20 (5%) | 15 (4%) |

Alurentell win rate by what happened to Chant (cast / no white source / never drawn): UR 47% / 30% / 38%, UB 30% / 28% / 41%, UW 40% / 29% / 28%, Storm 60% / 43% / 57%, Mirror 52% / 40% / 50%, Boros 54% / 10% / 43%. The "no white source" games are bad everywhere (the dead card), the cast games are better than never-drawn in UR, UW and Boros and worse in UB. Net "no Chant" tests: UR +4.0, UB +2.0, UW +5.4, Storm +8.7, Mirror +2.9, Boros -2.2 (chant0) and -3.2 (chant1): the nets prefer the main-deck card that Chant replaces, which I do not trust. In the UR LLM games Chant was drawn twice: cast once on the kill turn so they could not respond, once uncastable at 1 life (no white source).

### How good is Ending, and vs Storm / Tron

| Matchup | drawn | cast | held with mana | no white source | too late | win: cast vs never drawn |
|---|---|---|---|---|---|---|
| UR | 309 | 121 (39%) | 70 (23%) | 79 (26%) | 39 (13%) | 53% vs 39% |
| UB | 285 | 97 (34%) | 81 (28%) | 54 (19%) | 53 (19%) | 39% vs 35% |
| UW | 358 | 212 (59%) | 85 (24%) | 38 (11%) | 23 (6%) | 40% vs 33% |
| Storm | 343 | 160 (47%) | 103 (30%) | 50 (15%) | 30 (9%) | 54% vs 54% |
| Tron | 185 | 91 (49%) | 60 (32%) | 18 (10%) | 16 (9%) | 49% vs 48% |
| Boros | 416 | 285 (69%) | 70 (17%) | 31 (7%) | 30 (7%) | 49% vs 49% |

Ending is fine against UR and UW (cast games win more) and shows no effect against Storm, Tron and Boros. Net "no Ending": UR +3.4, UB -0.9, UW +2.6, Storm +7.0, Tron -0.9, Boros -0.9 (ending0) and -2.2 (ending1); Storm -1 Ending +1 Stock Up (`ending1`) is +1.6 ±3.5, and Tron -1 Ending +1 Omniscience is +1.0 ±3.3. LLM notes: Ending exiled a Cori-Steel Cutter and a Monk token against UR (fine), a Disruptor Flute against Tron (won the game), and Hexing Squelcher against Storm at the price of 2 life for ward, which was exactly the margin in a lost game (s6). It was dead or never cast in most of its other draws against Tron and Storm.

### 4th Aluren and 4th Force of Will after boarding vs Boros

Base takes out 1 Aluren and 1 Stock Up and leaves 4 Force of Will. Net vs rollout, 300 per side, change vs base (47.2%):

| Plan | change |
|---|---|
| 61 cards: 4th Aluren back (-1 Stock Up only) `al61` | +0.8 ±5.4 |
| 4th Aluren, no Chant (v-d) | -1.3 ±5.0 |
| 4th Aluren, 3 Force of Will (v-c) | -0.7 ±5.2 |
| 4th Aluren, 3 Petal (v-e) | +3.5 ±5.3 |
| 2 Consign, 3 Force of Will (v-a, your weekend list) | -1.0 ±5.4 |
| 1 Chant 1 Macabre (v-f) | +0.5 ±5.1 |

Nothing here is separable from noise. The 4th Force of Will question is v-c: cutting the 4th Force for the 4th Aluren is -0.7 ±5.2, i.e. no difference either way. The metrics are clearer: with a 4th Aluren the Aluren line by turn 4 goes from 29% to 38 to 43% (v-a, v-c, al61 and v-d) and the first combo turn from 4.58 to 3.9 to 4.2, and cutting Force of Will for it (v-c) does that without costing protection much (0.81 vs 0.92). So the 4th Aluren gets the combo earlier; whether Boros lets it matter I cannot say from this sim.

### 2 vs 3 Consign vs Tron

`consign3` = the 3rd Consign comes in over the 2nd Ending (needs a sideboard of 3 Consign, one Macabre fewer). Over Atraxa (`a`) and over Stock Up (`s`) are the alternatives.

| Config | net change | LLM games |
|---|---|---|
| 2 Consign (base) | - (45.8%) | 3-0 |
| 3 Consign, over Ending | +0.4 ±3.0 | 4-0 |
| 3 Consign, over Stock Up | +2.2 ±3.6 | - |
| 3 Consign, over Atraxa | -2.3 ±3.7 | - |
| No Consign | +0.3 ±3.4 | 2-1 |

Consign in the LLM games: it countered Ugin (T5) for one mana in one game and Karn in another; two more games drew it and held it. Metrics: protection at the first combo turn 1.00 base, 1.16 and 1.21 with a 3rd Consign, with no change in the first combo turn (4.2 to 4.4). The nets are not separable here and neither are the LLM games (Tron went 11-1 across all arms).

### How good is Force of Negation vs Storm and Tron

Net change from taking FoN out: Storm +2.9 ±3.6, Tron -0.5 ±3.9 (replaced by Island) and -1.6 ±3.2 (replaced by Omniscience). When drawn and cast FoN wins about as often as when not drawn (Storm 56% vs 59%, Tron 45% vs 51%). FoN was drawn in 34% of Storm games and 39% of Tron games. LLM: FoN exiled a turn-2 Karn off Tron's 7 mana (t5). Evidence says it is not important in either matchup.

### Trimming lands

| Matchup | cut | change |
|---|---|---|
| UR | Island, +Aluren | -2.8 ±3.9 |
| UB | Island, +Force of Will | -3.6 ±3.5 |
| UW | Island, +Aluren | +0.8 ±4.1 |
| Mirror | Island, +Show and Tell | +4.2 ±4.2 |
| Storm | Hedge Maze, +Acererak | +3.5 ±3.5 |
| Tron | Hedge Maze, +Omniscience | +0.1 ±4.0 |
| Boros | Island, +Aluren | -3.1 ±5.3 |
| Reanimator | Hedge Maze, +Stock Up | +1.2 ±4.9 |

Mana on turn 3 falls by 0.1 to 0.2 per cut (UR 3.08 to 2.93). A wash against UW, Tron and Reanimator, a small cost against UR, UB and Boros, a small gain in the mirror and Storm.

### Which matchups keep Omniscience

Net change vs base, base shown in brackets. UR (0 Omni): keep 1 -4.7 ±3.7, keep 2 -9.3 ±4.0. UB (0): -2.0, -7.6. UW (0): +1.1, -2.1. Tron (0): -0.2, -4.4. Storm (2): 1 out +1 Veil -1.0 ±3.4, both out -4.7 ±3.6. Boros (2): one out -6.2 ±4.7 (also v-g -6.8). Mirror: out (your rule). So: keep against Storm and Boros, take out against UR and UB; UW and Tron are close (keeping 1 is free, keeping both costs 2 to 4). Same sign as the combo-line metric (Storm line by T4 50% with 2 Omni, 45% with 1, 44% with 0).

### 61-card variants

61st card added on top of your plan, change vs base:

| Matchup | results |
|---|---|
| UR | Aluren -1.4, Omniscience -0.9, Petal -1.3, Force of Will -4.5, Emrakul -1.9 |
| UB | Omniscience -2.5, Petal -2.6, Force of Will -1.3, Emrakul +1.9 |
| UW | Aluren +1.4, Omniscience +0.4, Petal +0.3, Force of Will +0.4, Emrakul +3.1 |
| Storm | Atraxa +5.3, Stock Up +3.0, Veil -0.7, Emrakul +7.1 |
| Tron | Consign -1.0, Omniscience +0.1, Emrakul +3.9 |
| Mirror | Show and Tell +3.0, City of Traitors +3.3 |
| Boros | Aluren +0.8, Stock Up -0.3, Veil -1.3 |

All but the Storm Atraxa and Emrakul are inside the noise.

## Emrakul, the Aeons Torn

Implemented in the engine (flying, uncounterable, protection from colored spells, cast trigger extra turn, annihilator 6, graveyard shuffles) and tested in scenarios; it is cast free under Omniscience and takes the extra turn there (22 to 67 games per matchup in the cells below). It goes in Brady's way: as the 61st card (Macabre out of the sideboard) or in place of one Atraxa (`emr1a`). Net win rate change vs base, 500 games per side (Boros: 300, vs rollout):

| Matchup | Emrakul as 61st card | Emrakul instead of 1 Atraxa |
|---|---|---|
| UR Cutter | -1.9 ±4.0 | -4.8 ±3.8 |
| UB Dimir | +1.9 ±3.9 | +1.8 ±3.5 |
| UW Phelia | +3.1 ±4.0 | -0.8 ±4.1 |
| Storm | +7.1 ±4.0 | +2.0 ±3.7 |
| Tron | +3.9 ±4.1 | +2.3 ±3.7 |
| Boros | +4.5 ±5.7 | -2.7 ±5.4 |

A control with Emrakul only in the sideboard (`emr0`, UR) plays exactly like base, so the card does not disturb the nets. How often Emrakul comes up and what happens when Show and Tell puts it in, against putting Atraxa in:

| Matchup | plan | games | Emrakul drawn | put in by Show and Tell | cast (Omniscience) | wins: S&T put Emrakul only | wins: S&T put Atraxa only | wins: Emrakul never drawn |
|---|---|---|---|---|---|---|---|---|
| ur | emr1a | 1000 | 341 | 55 | 7 | 93% of 44 | 86% of 315 | 38% of 659 |
| ur | emr61 | 1000 | 322 | 46 | 7 | 89% of 35 | 89% of 360 | 41% of 678 |
| ub | emr1a | 1000 | 288 | 61 | 4 | 85% of 52 | 86% of 304 | 37% of 712 |
| ub | emr61 | 1000 | 297 | 45 | 4 | 92% of 36 | 86% of 335 | 38% of 703 |
| uw | emr1a | 1000 | 409 | 83 | 22 | 95% of 59 | 52% of 346 | 32% of 591 |
| uw | emr61 | 1000 | 395 | 65 | 22 | 87% of 47 | 55% of 452 | 38% of 605 |
| tron | emr1a | 1000 | 420 | 62 | 32 | 63% of 51 | 54% of 318 | 46% of 580 |
| tron | emr61 | 1000 | 403 | 48 | 26 | 79% of 39 | 61% of 371 | 48% of 597 |
| storm | emr1a | 1000 | 383 | 60 | 47 | 94% of 51 | 91% of 254 | 56% of 617 |
| storm | emr61 | 1000 | 339 | 54 | 46 | 91% of 43 | 90% of 322 | 64% of 661 |
| boros | emr1a | 600 | 261 | 44 | 62 | 48% of 29 | 54% of 166 | 40% of 339 |
| boros | emr61 | 600 | 258 | 42 | 67 | 70% of 27 | 63% of 233 | 51% of 342 |

Reading: Emrakul is drawn or seen in 29% to 43% of games and lands on the battlefield through Show and Tell or Omniscience in 5% to 7% (UR, UB), 9% to 11% (UW, Tron, Storm) and about 18% (Boros, where Omniscience stays in). When Show and Tell puts it in, the win rate is the same as with Atraxa against UR, UB and Storm (about 85% to 94%) and better against UW (95% and 87% vs 52% and 55%, protection from colored spells) and Tron (63% and 79% vs 54% and 61%). These are selected games (Show and Tell resolved), so read them as "Emrakul is at least as good a payoff", not as a win-rate gain. As a 61st card it is positive or neutral everywhere but UR (-1.9, inside the noise), best in Storm; replacing an Atraxa costs against UR and Boros. I did not run the mirror (Emrakul does not fit its plan).

## LLM games

Records for the arms (Alurentell side, one game per cell, both sides played by an LLM; the opponent has no notes):

| Matchup | arm | record |
|---|---|---|
| Tron | base / 3 Consign / no FoN / no Consign | 3-0 / 4-0 / 4-0 / 2-1 |
| UR | base / 1 Petal over Atraxa / no Veil | 3-0 / 2-2 / 1-2 |
| Storm | base / no FoN / no Consign | 1-2 / 2-2 / 3-0 |
| Mirror | base / no Consign no FoN | 0-3 / 2-1 |

These are far below what it takes to rank arms. What they do give is how the cards play:

- **Carpet** was cast on turn 2 in three UR games (two wins) and the players called it the best card in all of them (3 to 6 mana a turn, a turn-3 Aluren); the loss drew no payoff in about 22 cards. Keep all four.
- **Consign** is live against Tron (Karn, Ugin). Against Storm it is strong but wants mana for replicate: one game was lost with a single Consign on the Tendrils storm trigger and no mana to copy it.
- **Veil** beat a Force of Will in one UR game; Chant was drawn dead once (no white source).
- **Prismatic Ending** and **Portable Hole** (Tron's side) were the cards most often uncastable or without a target.
- UR boards Pyroblast and used it once as a counter on Show and Tell.

Reports for each game are in `rust-engine/tools/sideboard-experiments/llm/reports/` and the results in `llm/ledger.tsv`.

## Things the LLM players reported about the engine and menus (not checked by me)

- The castable list ignores mana that a Lion's Eye Diamond or other mana ability would make available, and the engine moves from Main 1 to combat when only such an ability could enable a spell (Echo of Eons flashback, Tendrils). It cost the Storm player turns in two games.
- Karn's minus (sideboard wish) was never offered; Kozilek's Command asks for graveyard targets for a mode that was not chosen; Disruptor Flute's chosen name is not shown anywhere.
- `pay=` does not accept names with a comma or apostrophe (Boseiju, Orim's Chant); the auto-payment tapped Boseiju for Grim Monolith once and left no green.
- The Prismatic Ending X prompt comes before the target prompt; the Ponder order prompt does not say that the first card chosen ends up on top.
- `get_card` does not know Ugin, Karn, Tezzeret, Trinisphere, Hexing Squelcher, Peer into the Abyss, Runehorn Hellkite.
- A semicolon or apostrophe in a pick reason is run by the shell (harmless but it breaks the reason).

## Caveats

- Net-vs-net: one net per matchup, trained before the new cards and the sideboard plans existed; card values for sideboard cards are almost untrained. The metrics and fate counts use the same games, so they inherit the net's play, but they measure what happened (turn of the combo, mana, whether a card was castable) rather than who won.
- A drawn card that is "held with mana" wins more because games that were won early had no reason to cast it. Compare "cast" and "no white source", not the held column.
- Every comparison uses 500 games per side (300 for Boros and Reanimator); intervals are ±3 to ±5 points.
- The first Emrakul run crashed in 2 to 3% of UW games (colored spells could be cast at Emrakul with no legal target); fixed and rerun, those numbers are discarded.

## Files

`rust-engine/tools/sideboard-experiments/`: `gen.py`, `gen2.py` (all plans and decks), `run1.sh`, `queue.sh`, `analyze.py`, `metrics.py`, `cardfate.py`, `tracestats.py`, `pending.sh`, `waitq.sh`, `llm/` (harness changes, reports, ledger). Full tables: `tables-net.md`, `tables-nr.md`, `tables-metrics.md`, `tables-fate.md`, `tables-cardstats.md`. The Emrakul engine files are in `emrakul-engine/` (they were developed on the merged engine tree, not on this repo's main).
