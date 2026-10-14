# Archon of Cruelty for one Ponder, game 1 (2026-10-14)

Brady's idea: Archon of Cruelty as another Show and Tell / Omniscience target instead of (or besides) Emrakul. Test: unboarded game 1, Alurentell main with 3 Ponder + 1 Archon of Cruelty ({6}{B}{B} in the engine, 6/6 flier), against UR Cutter, Dimir Tempo (UB), UW Phelia. 12 LLM-vs-LLM games (4 per opponent, 2 on the play and 2 on the draw), Sonnet agents on both seats, same menus and playbook as the earlier Emrakul tests. No control arm was run on this build.

## Result
| Opponent | Alurentell with Archon |
|---|---|
| UR Cutter | 3-1 |
| UB Dimir | 0-4 |
| UW Phelia | 2-2 |
| **Total** | **5-7** |

Twelve games cannot give a win rate (a 95% interval is about ±37 points). For scale, the earlier plain-main control on a different build was 7/12, and the Emrakul variants were 10/12 (for 2nd Veil), 9/12 (for City), 5/12 (61st card).

## What Archon did
- Drawn or seen in 4 of 12 games (a1, a3, a9, a11), cast in exactly one.
- **a1 (UR, won):** Show and Tell was Forced, Force of Will back, then Archon put in instead of Aluren. Enter trigger made them discard a Channeler and lose 3; next turn the attack trigger stripped a Force of Will; 13 flying damage won. The player would keep it vs UR ("uncounterable via Show and Tell, triggers strip Force and Daze from a hand that holds them").
- **a3 (UR, won):** held it; Atraxa took the single Show and Tell, Aluren + Acererak won. Dead card in hand.
- **a11 (UW, won):** drawn off Atraxa's reveal, never used; Show and Tell went to Aluren and the loop won.
- **a9 (UW, lost):** drawn twice, uncastable (no Show and Tell, Omniscience or black mana). Lost to Tamiyo plus Phelia.
- The seven games where it was never drawn say nothing about it. All four UB losses came against a fast Moonshadow / Nethergoyf / Bilbo / Kaito / Bowmasters start before any payoff was reachable (a8 was also stuck on two lands; a6 took Bowmasters pings off Brainstorm).

## What the players said
Ten of the 12 Alurentell players said to put the Ponder back; a1 (drew and used it) would keep it against UR, and a2 was undecided. Reasons given:
- It is only live through Show and Tell or Omniscience; Atraxa's reveal pays for itself even when Atraxa is Swords'd, Archon's value is only its enter trigger.
- It is black, so Swords, Path, Heat with delirium (a3: 6 toughness dies to one delirium Heat) and Prismari Charm answer the body; Atraxa needs two Bolts after blocking.
- Stifle (4 in UW Phelia) and Consign hit the trigger; the opponent chooses what to sacrifice (a Clue, Tamiyo or a Monk token is enough).
- It does not stabilise a race (no lifelink; 3 life per trigger) and Atraxa blocks better.
- Where it shines: lone big threat or a hand holding Force/Daze, and it cannot be Dazed or countered once put in, same as any Show and Tell target.
Most of this reasoning comes from players who never drew it, so weigh it as opinion.

## Read
Nothing here shows Archon is better than the Ponder it replaced, and the one game it won is exactly the case it was designed for (Show and Tell resolves, opponent holds Force and a lone creature). The sims for Emrakul rated Ponder as the best card to lose to a new payoff (+4.6), so it is not free either. Against Dimir the deck simply needs to dig and survive, not another eight-drop. Not recommended as a main-deck card; at best a 1-of over Emrakul if white/Show-and-Tell lines are what Brady wants to improve, and only against decks without Stifle/Swords.

## Caveats
- Game a10: two Alurentell agents were started by mistake on the same seat (I launched it twice). Both made the same plays and reported the same win; counted, but do not lean on it.
- Opponents had no playbook. Dimir played as a pure aggro deck (Moonshadow, Goyf, Kaito), which is not how the earlier nets or waves played it.
- Engine oddities the opponents noticed, not caused by Archon: Show and Tell offers lands (Wasteland, Volcanic Island) as put-in choices, which the real card does not allow (a1b, a2b, a12b); Mishra's Bauble does not show the card it looks at; `pay=` with a name shared by several lands matches the wrong land (a9b).

Files: `archon-games/` (24 reports `aNg1a.md` Alurentell, `aNg1b.md` opponent, the brief, launcher `mkx.sh`).
