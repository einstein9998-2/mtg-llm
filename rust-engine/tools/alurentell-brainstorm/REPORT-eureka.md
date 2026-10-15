# Eureka in Alurentell, game 1 and post-board (2026-10-14/15)

Brady's request: "also try out playing some number of eureka", "preboard and post". Eureka ({2}{G}{G} sorcery: starting with you, each player may put a permanent card from their hand onto the battlefield; repeat until no one does) is not in the engine's card pool, so I added it in a scratch engine (see below) and ran two waves.

**Defaults I chose (Brady gave only "some number"):** 2 Eureka in place of 2 Ponder in the 60 (so 2 Ponder + 2 Eureka). Game 1 used the unboarded main deck. The post-board games used the usual sideboard plan for each matchup (not the Petal/Chant list from `REPORT-pe-list.md`), with the same 2 Eureka kept in. 12 games each, 4 per opponent (UR Cutter, UB Dimir, UW Phelia), 2 on the play and 2 on the draw, Sonnet agents on both seats. No same-build control arm was run.

## Result
| Opponent | Game 1 (main + 2 Eureka) | Post-board (+ usual plan) |
|---|---|---|
| UR Cutter | 3-1 | 2-2 |
| UB Dimir | 3-1 | 3-1 |
| UW Phelia | 4-0 | 2-2 |
| **Total** | **10-2** | **7-5** |

17-7 over 24 games. Twelve games cannot give a win rate (95% interval about ±28 to 37 points), and the win rate is mostly not about Eureka: it was never cast in 19 of 24 games.

## How often Eureka showed up and what it did
| | Saw one | Cast | Resolved | Won when seen |
|---|---|---|---|---|
| Game 1 | 7 of 12 | 2 | 0 | 6-1 |
| Post-board | 9 of 12 | 3 | 3 | 5-4 |

- **Game 1, both casts were countered.** f5 (UB): hard cast on turn 3 with exactly 4 mana (GG from two Hedge Mazes, Tomb for the rest) into Daze, no spare mana; it ate the Daze and the game was lost. f9 (UW): hard cast on turn 4 into a tapped-out opponent; Force of Will (pitching Quantum Riddler) countered it. In both games the deck won or lost on other cards. In the other five games where it was seen it sat in hand: f1 and f7 chose Show and Tell (3 mana, a spare for Daze) over a 4-mana tap-out; f11 had one green source; f12 already had Omniscience and a winning loop; f10's player mulliganed it away.
- **Post-board, all three resolved Eureka were wins, all against UB, all with two or three Carpets of Flowers on the battlefield** (Carpet makes the GG and the fourth mana easy, since the opponent controls Islands):
  - **h5:** hard cast on turn 6 after a Veil of Summer, the opponent empty-handed. Four rounds: Aluren, Acererak, Ancient Tomb, Ancient Tomb. Acererak's trigger waited until Eureka finished, then the loop killed from 10. The player would keep 2.
  - **h6:** hard cast on turn 5 main 2 with Tropical up for Daze. Rounds: Atraxa (the opponent put in Orcish Bowmasters), Atraxa, Misty Rainforest, Acererak. The legend rule ran afterwards, but all three Atraxa triggers resolved: 20 cards revealed, hand 0 to 8. Won next turn. The player would keep 2; the UB player on the other seat called it "excellent against me. I would run 2 Eureka in a deck like theirs."
  - **h8:** cast on turn 6 with GGGG from two Carpets, the opponent tapped out with one card. Rounds: Aluren, Acererak. The opponent put in nothing. The player: "no better than a second Aluren", so 0 or 1.
  - What the opponent got out of it: Bowmasters (a ping and an Army) in h6; nothing in the other two (they were empty-handed or had no permanent).
- **Not cast post-board (9 games where one was seen, 6 not cast):** h1 (UR, stuck on lands), h2 (UR), h3 (UR, bottomed on a mulligan; won with Carpet and Veil), h7 (UB, one-land keep), h10 (UW, Lavinia plus no lands made MV 4 uncastable), h12 (UW, won with Aluren hardcast under Veil/Chant). Unseen in h4, h9, h11.

## What the players said
Of the 24 Alurentell players, 22 would keep 0 or at most 1 Eureka and put the Ponder back (the two who would keep 2 are h5 and h6, below). The reasons, over and over:
- **It costs one more than Show and Tell and needs GG,** which Ancient Tomb, City of Traitors and a fetch-heavy base do not give early. It was a dead card in most hands that held it.
- **It is a sorcery that Daze, Force of Will and Force of Negation hit just like Show and Tell,** and it taps you out at 4.
- **It is symmetric over several rounds,** so against a deck with lands, Murktide, Channelers and Bowmasters it hands them permanents each round, where Show and Tell gives them one.
- **Its upsides:** it puts in several things at once (Aluren plus Acererak, or two Atraxa for two reveals), and with Carpet of Flowers on the table or the opponent empty-handed it is better than Show and Tell. h5, h6 and h12 said it can be a second route when the opponent is tapped out and Carpet is out (h12: "1 at most"), but no one said it should replace a Show and Tell.
- The three who resolved it split: two keep 2, one keeps 0 or 1.

## Read
Not enough evidence to put Eureka in, and no evidence that it hurts: the 10-2 and 7-5 are mostly the base deck, and the two game-1 casts did nothing but eat a counter. The one pattern that looks real is that Eureka is a good second route after Carpet is out (mana) and when the opponent is tapped out or empty-handed (3-0), and a bad card in the hand where it competes with Show and Tell for the same mana. A single copy in place of the fourth Ponder is the version the players kept pointing at; I did not test it.

## Engine notes
- Cards are put onto the battlefield as each is chosen (one at a time, which is how the real card works); triggers wait until Eureka has finished resolving. My game brief said "every card enters at once" which was wrong; two players noticed (h5, h8). The legend rule runs after resolution (h6: three Atraxa triggered).
- Other engine or menu items from the 24 reports: Brainstorm and Ponder order prompts do not say the first pick ends on top; Carpet of Flowers asks twice per main phase per copy; menu option indices shift when a castable option disappears (h11b mis-clicked); `pay=` by name fails on "Boseiju, Who Endures" (use the v-id); a `;` in a pick reason runs as a shell command (harmless).
- **Possible engine bug, not confirmed:** h7b says Undercity Sewers fetched with "two other lands" came in tapped. The card enters tapped unless you control two or fewer other lands, so with exactly two it should be untapped; the player may have had three. Not checked against the log.

## Scratch implementation
`eureka-engine/` has the card (`eureka.cards.ron`), the new `Effect::PutLoop` excerpts (sequential, caster first, repeat until a whole round puts nothing in) and the two scenarios that pass (41 of 41 in the scratch tree). It was built in a copy of the payfix2 tree because core is frozen; nothing under `rust-engine/crates` changed. It can go through the RFC process if Brady wants Eureka in the pool.

Files: `eureka-g1-games/` (24 reports `fNg1a.md`/`fNg1b.md`, the 75-card list, `mkeu.sh` and the game-1 brief), `eureka-post-games/` (24 reports `hNg2a.md`/`hNg2b.md`, post-board brief).
