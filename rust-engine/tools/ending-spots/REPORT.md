# Prismatic Ending against tempo: is the 1-for-1 trade good, and when?

Written for Brady, 2026-10-14. Ending is a sideboard card (0 in the main deck), so everything here is **post-board games** with the current base plans (2 Ending in vs UR, UB and UW). There is no game-1 Ending to study.

## Short answer

- **Cast it, early, on a lone threat.** When Ending can exile a creature, casting it now beats never casting it by about 7 points of win probability against UR, 8 against UB and 9 against UW. Casting it one turn later gives back 2 to 4 of those points.
- **Your three instincts, checked:**
  1. *Worse the more creatures they have* is true against UR and UB, not UW. Win-probability gain from casting vs never casting, by number of opposing creatures: UR 1 creature +10.4, 2 creatures +3.8, 3+ +1.0. UB +12.3, +7.1, +5.8. UW +8.6, +10.8, +8.3.
  2. *Worse later in the game* is mostly the same effect. Later games have more creatures, and with creature count held fixed, game length barely matters (UR, one creature: +11.2 on turns 1 to 4, +10.0 on turn 5 and later). What does change with the clock is the cost of waiting: holding Ending one more turn costs 6 points at life 5 or less against UR, 1 point at life 16 or more.
  3. *At low life, resolve Show and Tell instead* is right, but only when you cannot do both. When Show and Tell is castable only if you skip Ending, and life is 7 or less, casting Ending was **8.8 points worse** (UR, 79 spots, ±6.6) and **4.2 points worse** (UB, 109 spots, ±5.0) than not casting it, and 10 and 6 points worse than waiting a turn. Pooled, about −6 ±4: the sign is consistent but each matchup alone is borderline. When mana covers both, Ending is +5 to +7 even at life 7 or less. When Show and Tell is not castable that turn, Ending is +6.5 to +7.
- **Low life, one creature: the best Ending spot there is.** Life 7 or less with one creature out: +15.8 (UR, 691 spots) and +13.9 (UB, 389 spots) over never, +11 to +12 over waiting a turn. Life 7 or less with two or more creatures: +1.6 (UR) and +5.2 (UB). At that point Ending does almost nothing against UR.
- **1 vs 2 vs 0 Ending in the deck:** the sim cannot separate them (differences under 2 points, noise ±0.5), see below. Dismember in the UW plan does about what Ending does. Hydroblast against UR does about what Ending does. Boseiju against Boros could not be tested with these tools.

## What I did

1. **Spot tool** (`endingspot`, new, in `rust-engine/crates/mtg-agent/src/bin/`). It plays post-board Alurentell games (the matchup net, net-sampled play at temperature 0.2 on both sides). At each of Alurentell's sorcery-speed turns with Ending castable and an opposing creature it could exile, it copies the true game state and plays it out 32 to 48 times for each of these arms:
   - **cast**: cast Ending now on the most dangerous creature it can exile (cheapest X that works; the opponent can still respond),
   - **late1 / late2**: hold it, then cast it on the first chance one or two own turns later,
   - **wait**: may not cast until the end of the opponent's next turn, then the net decides,
   - **never**: may not cast it for the rest of the game,
   - **free**: the net decides on its own.
   Checks that the rollout player is good enough: net-only games win 49.4% against UR where net plus 32-iteration search wins 50.4%. Net-only vs UB 40.2%, vs UW 45.8% (400-game checks).
2. **Spots:** 6,000 games per matchup (UR 2,676 usable spots, UB 2,790, UW 5,378), plus 30,000 games per matchup restricted to life 8 or less (UR 2,777 spots, UB 3,007). About 3.3 million rollouts.
3. **Deck tests:** 20,000 net-only games per cell (±0.35), different plans, same seeds.
4. **LLM games:** parsed the full logs of about 100 Sonnet-played match logs (`match-logs/postboard-matches`, m01, m02) for turns where Alurentell held Ending with a creature across the table (36 spots in 18 games).

Tables: `tables/*.md`. Spot data: `data/*.jsonl.gz`. Code: `rust-engine/tools/ending-spots/` and `endingspot.rs` (PR attached to this thread's branch).

## The numbers

Win-probability points, cast now minus the alternative, 95% interval on the cluster-adjusted mean in the tables.

### By number of opposing creatures (standard runs)

| | spots | cast − never | cast − 1 turn later |
|---|---|---|---|
| UR, 1 creature | 1,493 | +10.4 | +3.4 |
| UR, 2 | 797 | +3.8 | +2.5 |
| UR, 3+ | 386 | +1.0 | +0.2 |
| UB, 1 | 906 | +12.3 | +4.9 |
| UB, 2 | 826 | +7.1 | +3.2 |
| UB, 3+ | 1,058 | +5.8 | +3.4 |
| UW, 1 | 3,412 | +8.6 | +1.8 |
| UW, 2 | 1,556 | +10.8 | +3.5 |
| UW, 3+ | 410 | +8.3 | +4.0 |

All spots: UR +7.1 / +2.6, UB +8.3 / +3.8, UW +9.2 / +2.4.

### By my life: cost of waiting one turn (cast − late1)

| life | UR | UB | UW |
|---|---|---|---|
| 5 or less | +6.2 | +3.1 | +5.8 |
| 6 to 10 | +3.8 | +6.0 | +3.4 |
| 11 to 15 | +2.8 | +5.5 | +2.3 |
| 16 or more | +1.3 | +2.3 | +1.3 |

### By turn (cast − never)

UR: turns 1 to 3 +10.0, 4 to 6 +7.6, 7+ +6.1. UB: +13.0, +8.9, +6.1. UW: +9.5, +10.7, +8.9. Mostly the creature-count effect (see above).

### Low life, creatures, Show and Tell (extra life ≤ 8 runs, life ≤ 7 rows)

| life 7 or less | UR cast − never | UB cast − never |
|---|---|---|
| 1 creature | +15.8 (691) | +13.9 (389) |
| 2+ creatures | +1.6 (1,669) | +5.2 (2,166) |
| Show and Tell castable only without Ending | −8.8 ±6.6 (79) | −4.2 ±5.0 (109) |
| Show and Tell castable alongside Ending | +5.0 (316) | +6.6 (256) |
| Show and Tell not castable | +6.5 (1,965) | +7.0 (2,190) |

### What the LLM games show (UR, small sample)

Of 18 games with a spot, Ending was cast in 12 (8 wins) and held at every spot in 6 (5 wins). The four games where the player sat on Ending for two or more turns against a lone creature and did not win quickly were all losses (q408 g3, q425 g2, q433 g2, q463 g3). The games won with Ending held were won by the combo within a turn or two (q460 g2, q481 g3, q488 g2, q401 g2, m01 g2). This agrees with the sim but the hands are not matched, so it is anecdote.

### 1 vs 2 vs 0 Ending in the deck (win % of Alurentell, 20,000 net-only games per cell, ±0.35)

| Ending copies, what keeps the freed slot | UR | UB | UW |
|---|---|---|---|
| 2 (base plan) | 48.5 | 40.8 | 47.8 |
| 1, keep a Petal | 49.2 | 41.2 | 49.1 |
| 1, keep a Force of Will | 46.6 | 40.1 | 46.4 |
| 1, keep an Omniscience | 50.1 | 41.8 | 50.0 |
| 1, keep an Aluren | 49.6 | n/a | 51.8 |
| 0, keep 2 Petal | 50.4 | 41.8 | 50.3 |

Read these with a correction. The net plays Ending by itself in only 13% (UW), 22% (UR) and 45% (UB) of the spots where casting is good (the share of spots where the net itself cast Ending in the generating games), so the deck test undersells Ending. The spot tool says that casting at the first chance instead of what the net does adds roughly +1.3 (UR), +1.5 (UB) and +3.4 (UW) points per game for two copies (share of games with a spot times cast − free; rough). Adding that: 2 Ending is about even with 0 Ending plus 2 Petal against UR and UB, and slightly ahead against UW. One Ending is not distinguishable from two. A Force of Will in the slot is the one clearly worse use.

### Your other slot ideas

- **Dismember in the UW plan** (1 Dismember in, 3 Petal out, no Ending): 48.7. Ending ×1: 49.1, Ending ×2: 47.8, no Ending: 50.3. The net treats Dismember like Ending. It costs 4 life, which the net does not model well.
- **Hydroblast vs UR** (2 in instead of 2 Ending): 48.9. One Ending plus one Hydroblast: 48.6. Two Ending: 48.5. No Ending, 2 Petal: 50.4. As good as Ending in the net's hands. I did not test it against UB and UW, where it is a blank (no red).
- **Boseiju vs Boros:** Boros net-only play wins only 15% for Alurentell (14.8 with Ending, 15.3 with Boseiju, 15.4 with neither), too weak to read, and there is no Tron or Storm engine on main. Not tested in any useful way.

## Limits

- **The nets were trained on main decks.** They underplay Ending, Chant, Carpet and Veil. In the spot tool the cast arm scripts the target and X, so it removes that weakness for Ending itself, but the other post-board cards are still played badly. The deck tests have the same weakness for every sideboard card.
- **"Never" is a floor.** Real holding is closer to "late1" or "late2" (cast 2 to 4 points lower than casting now). Use the late columns for the cost of waiting, the never column for the value of the card.
- **Overlapping spots.** The same game can contribute several spots; intervals are clustered by game but are not exact.
- **Engine and nets.** Main engine (176 card definitions) with the m5chant nets; the later 222-definition engine used for the October 12 and 13 experiments is not in the repo, so these numbers are not directly comparable to those. UR here is `ur-cutter`, not the alt list.
- **Sample size** for the Show and Tell conflict (79 and 109 spots) is the weak point. The point estimates agree with you; each interval alone includes zero.
