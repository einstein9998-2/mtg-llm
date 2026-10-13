# Emrakul, the Aeons Torn: LLM-piloted games, game one and post-board (2026-10-13)

Twelve games, each played by two Claude agents (one per deck) through the same engine menus as the earlier LLM wave. Alurentell had **one Emrakul in the main deck in place of one Ponder** (the sim's best cut), the opponent had its usual list. Game one is unboarded on both sides; the post-board games use Brady's plan for the matchup (`plans/<m>/base`), applied to the same 75 with Emrakul in the main. Each opponent was piloted with no playbook. The nets were not consulted (`--no-consult`). Players read the card from memory: the card-lookup tool does not have Emrakul in its pool, which every Alurentell player reported. Reports are in `llm/emrakul-games/reports/` (`e<N>g<G>a.md` is Alurentell, `…b.md` the opponent).

Twelve games cannot give a win rate. The point is what Emrakul actually does and what the players say about it.

| Game | Opponent | Round | Seat | Result | Emrakul |
|---|---|---|---|---|---|
| e1 | UR Cutter | game 1 | play | **Won**, turn 3 | In the opening seven. Show and Tell put Omniscience in, Emrakul cast free (extra turn), annihilator 6 wiped their board, 22 flying damage |
| e2 | UR Cutter | game 1 | draw | Lost | Never drawn; Show and Tell met Force of Will on turn 4 |
| e3 | Tron | game 1 | play | Lost | Never drawn; Omniscience from Show and Tell was destroyed by Boseiju, then Karn and a Trinisphere beat down |
| e4 | Tron | game 1 | draw | Won | Never drawn (not in the Atraxa reveal either) |
| e5 | Storm | game 1 | play | Won | Never drawn |
| e6 | Storm | game 1 | draw | Won | Never drawn |
| e7 | UR Cutter | post-board | play | Won | Opening seven was Emrakul plus one land, so it was mulliganed away |
| e8 | UR Cutter | post-board | draw | Lost | Drawn on turn 4, put in with Show and Tell after a Force of Will fight, died the next turn to Murktide and Channelers (Emrakul could block one) |
| e9 | Tron | post-board | play | Won | Never drawn |
| e10 | Tron | post-board | draw | **Won**, turn 3 | In the opening seven. Turn 2 Show and Tell put it in, annihilator on turn 3 for exactly lethal |
| e11 | Storm | post-board | play | Won | Never drawn |
| e12 | Storm | post-board | draw | Won | Never drawn |

**Game one 4-2, post-board 5-1.** Emrakul was in the starting hand or seen in 4 of 12 games (e1, e7, e8, e10); in the other 8 it was never drawn.

## What the games show

- **When it lands it ends the game at once.** e1 and e10 were turn-3 kills: free cast under Omniscience (extra turn, so the opponent gets no untap) or put in with Show and Tell (no extra turn, annihilator on the next attack). Neither opponent had an answer to a protection-from-colored-spells creature (the Tron player's only listed outs were Karakas and Expedition Map).
- **It is slow and only a Show and Tell/Omniscience card.** Without Omniscience, Show and Tell is the only route (15 mana otherwise), which is why the players kept calling it a dead card in hand, especially vs UR (Omniscience is boarded out post-board) and vs Storm (the games are decided by turn 4 anyway).
- **vs UR tempo it can arrive too late (e8).** Put in with Show and Tell on turn 4 at 11 life after paying Tomb and Force of Will life, it blocked one attacker and the other two killed Alurentell. The player's note: it needs a healthy life total.
- **vs Tron it dodges what Tron has.** Colorless and protection from colored spells, so Ugin's exile and colored removal miss it (the e10 player noted Karakas as the only answer). Boseiju killed an Omniscience in e3, which is a reason the free-cast route is less safe than Show and Tell onto the battlefield there.
- **What they would cut.** Nobody wanted a combo piece out. Every player who commented said the Ponder it already replaced (or the fourth Stock Up) is the cheapest slot; three players (e2, e5, e6) said they would rather have Ponder back in the main. That matches the sim: Ponder +4.6, Stock Up +2.8 on average for Emrakul swaps, at noise level.
- **No dead-card tax visible.** In the 8 games where it was not drawn, the results (6-2) look like ordinary Alurentell results; the one pre-board loss to UR and the Tron loss did not involve it.

## Engine and menu notes that came up (not Emrakul-specific)

- The card-lookup tool does not know Emrakul (or most Storm and Tron cards), so players relied on memory.
- Opponent-side agents reported engine oddities in their own decks: Karn, the Great Creator offered only its +1; Beseech the Mirror was not offered as castable when Lotus Petal or Lion's Eye Diamond mana was needed; `pay=` is a preference and an extra Ancient Tomb got tapped (2 life lost) in two Tron games; Planar Nexus counted as one Tron land. These are in the e3/e4/e9/e10 and e6/e11 reports.
- Emrakul showed as 15/15, which is its printed size.

## Files

`llm/emrakul-games/` in this directory: `mk2.sh`, `mkgame2.sh` (game 1 or post-board, Emrakul base 75, Emrakul engine build), `BRIEF-g1.md`, `BRIEF-post-emr.md`, `decks-base-emr/alurentell.txt`, the 12 task files, and the 24 reports.
