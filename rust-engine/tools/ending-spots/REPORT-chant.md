# Swapping Prismatic Ending for more Orim's Chant against tempo (2026-10-14)

Brady's question: against blue tempo, does 1 Ending + 3 Chant or 0 Ending + 4 Chant beat the current 2 Ending + 2 Chant post-board plan? Brady then asked to skip the net analysis and use LLM games, so this report is 12 LLM-vs-LLM post-board games (each seat a separate Sonnet agent through the engine menus), plus a short note on why the net tests were dropped.

## Short answer

- **The games cannot separate the three plans.** 1 Ending + 3 Chant went 4-2, 0 Ending + 4 Chant went 2-4. With six games per arm that is well inside noise (a 60% deck produces either result). The earlier LLM waves with the current plan (2 Ending + 2 Chant) won about 63% against UR main (31 of 49) and 59% against UR alt (29 of 49); there is no matching UB or UW baseline.
- **The players' reports agree on one thing: Chant is limited by white mana, not by count.** The 60 has two white lands, Savannah and Tundra (Carpet of Flowers can also make white). In at least seven of the 12 games Chant sat dead in hand or was cast at a bad moment. Most players asked for fewer Chant (1 to 3) or more white sources; one (j5, who won with it) said 4 is right. Kicked Chant (WW) was never castable in any game.
- **Where Chant worked, it worked as a decoy, not a fog.** Cast on the combo turn before Show and Tell or Aluren it ate a Force of Will or Daze and let the real spell through (k1, j5), or made the opponent spend Daze (j3). In j5 it was the whole game: one Chant resolved, the opponent could cast nothing, Aluren resolved and the Acererak loop killed the same turn.
- **Ending was almost never seen.** It was drawn in 0 of the 6 games with 1 copy (once in a discarded opening seven), so there is no read on it from the games. Three players (k4, k5, j4) said they wanted it against Moonshadow, Bowmasters and Lavinia, but that is a wish, not an observation.
- **My reading, not a result:** the data does not support swapping Ending for Chant. If anything the reports argue for 2 to 3 Chant and more white access, not 4 Chant.

## The games

Post-board game 2 of a match, same 60 and sideboard plans as the earlier waves except the Ending/Chant counts. Seats: Alurentell on the play in the "A" games, on the draw in the "B" games. Opponent boards per Brady's earlier plans (UR Cutter +3 Pyroblast, Dimir Tempo +Consign +Force of Negation, UW Control +Lavinia/Erode/Pyroblast/Ending).

| Game | Opponent | Alurentell | 1 Ending + 3 Chant (k) / 0 Ending + 4 Chant (j) | Result |
|---|---|---|---|---|
| k1 | UR | play | 1E + 3C | **Won**, turn 5 |
| k2 | UR | draw | 1E + 3C | **Won**, turn 6 |
| k3 | UB | play | 1E + 3C | **Won**, turn 8 |
| k4 | UB | draw | 1E + 3C | Lost, turn 5 |
| k5 | UW | play | 1E + 3C | Lost, turn 9 |
| k6 | UW | draw | 1E + 3C | **Won**, turn 9 |
| j1 | UR | play | 0E + 4C | Lost, turn 7 |
| j2 | UR | draw | 0E + 4C | **Won**, turn 6 |
| j3 | UB | play | 0E + 4C | Lost, turn 6 |
| j4 | UB | draw | 0E + 4C | Lost, turn 3 |
| j5 | UW | play | 0E + 4C | **Won**, turn 5 |
| j6 | UW | draw | 0E + 4C | Lost, turn 12 |

By arm and opponent: 1E + 3C: UR 2-0, UB 1-1, UW 1-1. 0E + 4C: UR 1-1, UB 0-2, UW 1-1. Both UB losses with 4 Chant (j3, j4) came against Moonshadow beats (plus Orcish Bowmasters in j4) while the combo hand stumbled on mana; no Chant was drawn in j4 and in j3 the one Chant was a late bait.

## What happened to the Chants (from the Alurentell reports)

| Game | Chant drawn | Cast | What it did |
|---|---|---|---|
| k1 | 1 | 1 | Cast first on the Aluren turn with Carpet's W; ate their Force of Will (pitching Daze). Show and Tell then lost to Daze plus Wasteland, but they were stripped, and Aluren resolved the next turn. |
| k2 | 3 (2 bottomed) | 0 | No white source; won on Veil plus Aluren. |
| k3 | 1 | 0 | Never any white (Tundra and Savannah not drawn). |
| k4 | 0 | 0 | Mulligan; none seen. |
| k5 | 3 | 0 | Wasteland killed the only white source, a Stifle hit the Strand. Died holding two. |
| k6 | 1 (+1 seen) | 1 | Cast with Ancient Tomb and Savannah exposed to two Wastelands; they hit both. The line cost two lands. |
| j1 | 1 | 0 | Only white source was Savannah, needed for Atraxa; Tundra was binned. |
| j2 | 3 seen (2 in a mulliganed seven) | 0 | Bottomed on the mulligan, no WW. Won anyway. |
| j3 | 1 | 1 | Turn 11 at 2 life, cast to bait; they Dazed it. Aluren resolved, but the game ended on the free Acererak cast (the player thinks they responded). |
| j4 | 0 | 0 | none |
| j5 | 2 (+1 seen) | 1 | Cast before Aluren with six mana; resolved, opponent cast nothing, loop killed from 19. |
| j6 | 3 | 2 | First enabled Show and Tell with the opponent tapped out; second was Force of Negation'd; third dead (one white source). |

Five casts in 12 games, two of them decisive for the better (j5, and k1 as a decoy), one that cost two lands (k6). In seven games Chant was never castable.

## Other things the reports raised

- **White mana.** Savannah and Tundra are the only white lands in 60 cards; Flooded Strand (3) and Misty Rainforest (4) fetch them if still in the library. Wasteland on the white land (k5, k6, j1 in effect) turns off Chant for the game. Several players would rather add white access than Chant copies.
- **Pyroblast** (UR and UW) is the opponent card the players noticed most: it answers Force of Will, Show and Tell and Atraxa and ignores Veil of Summer (j1, j2, j6). The j2 player thought a Chant cast on his own turn would have beaten two Pyroblasts plus Daze and Force of Will, but he had no WW.
- **Mana payment.** `pay=` was reported as ignored in about eight reports (named lands not tapped, floating Carpet mana unspent, Aluren over-tapped). It cost Pyroblast mana in j5b and j6b and a Carpet pool in k5. This is the engine's known preference behavior, not new.
- **Moonshadow and Orcish Bowmasters** won k4, j3 and j4 for UB; Bowmasters punishes Brainstorm and Ponder (j4).

## Why the net tests were dropped

Before Brady asked for LLM games I ran net-only deck tests (20,000 games per cell, ±0.35). They do not help here. With 2 Ending + 2 Chant as the base:

| | base (2E + 2C) | 1E + 3C | 0E + 4C |
|---|---|---|---|
| UR | 48.5 | 49.4 | 49.0 |
| UB | 40.8 | 48.4 | 47.7 |
| UW | 47.8 | 42.6 | 43.1 |

UB jumps by +7.6 and UW falls by −5.2 for the same one-card swap, and a Chant-count sweep with the Petals held back (UB: 0 Chant 52.1, 1 Chant 51.1, 3 Chant 48.8) shows the number mostly follows how many Petals stay in, not Chant. The nets were trained on main decks and play sideboard cards poorly, so I did not use these.

## Limits

- Twelve games, one game per matchup and seat, so the 4-2 and 2-4 are noise. Seat and opponent are confounded by the rotation.
- Agents played from reading the brief; they drew many different hands, and a few outcomes were decided by mulligans (k4, j4, j2).
- The opponent players had no playbook; several used Pyroblast, Force of Will and Wasteland well, others sat on counters (k1, k5).
- Prismatic Ending was effectively untested.
- Reports are in `llm-chant/reports/` (`kNg2a` and `jNg2a` are Alurentell, `…b` the opponent); the plans, brief and launcher are alongside. The launcher needs the `llmgame` binary from `llm-benchmark/harness/llmgame.rs` (add `[[bin]] name = "llmgame"` to `crates/mtg-agent/Cargo.toml`, as RESULTS.md describes) and the base 75 with four Chant in the sideboard (`alurentell-base-75-4chant.txt`, two Consign to Memory out).
