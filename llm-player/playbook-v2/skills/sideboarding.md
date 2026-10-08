# Sideboarding (best-of-three matches)

Use when: between games of a best-of-three match, and at the start of games 2 and 3.

Sources: plans from /home/claude/work/live/engine/sideboard-plans/alurentell.txt ([Brady] = locked by Brady in the project chat, 2026-10-06; [draft] = not reviewed by Brady); list from /home/claude/work/llm2/decks/alurentell.txt (Brady's personal list, 2026-10-06); match rules from the engine's SIDEBOARDING.md. Brady, 2026-10-08: "Start playing matches: best of 3 with sideboarding, loser of the previous game goes first."

Each rule is a default. Read its "Breaks when:" before applying it; if the situation matches, follow the principle instead.

## Match rules (P13)

- **SB1** [P13] Apply the plan for this matchup to the base 75 below, never to the previous game's list. Swaps are one for one: the main deck stays at 60 and the sideboard at 15. (engine match rules)
  Breaks when: no known exception.
- **SB2** [P13] Use Brady's plans ([Brady]) as written. A [draft] plan is an adaptation he has not reviewed.
  Breaks when: no known exception.
- **SB3** [P13, P1] With no plan for the matchup, decide each swap by what the card does in this matchup (the table below): bring a card in only for a job it does here, take out what is dead or slow here, and keep a route to a payoff (P1) and one blue pitch card per Force of Will (protecting-the-combo.md PR16).
  Breaks when: no known exception.
- **SB4** [P5, P13] The loser of the previous game goes first and plays first (there is no draw option yet). So after a loss Alurentell is on the play in the next game, and after a win on the draw. Being on the play changes some keep and wait decisions: re-read the rules that mention the play or the draw (opening-hand-and-mulligan.md OH7) and the wait rules (turn-sequencing.md TS6 to TS10). There are no other play or draw rules yet; do not invent them.
  Breaks when: no known exception.
- **SB5** [P6, P7, P1] With Carpet of Flowers in the deck (games 2 and 3 against a deck with Islands), plan to cast it on turn 3. Brady (as recorded in BRADY-RULINGS2.md): "Always plan to cast Carpet on turn 3." It is really good into Wasteland: its mana comes from the Islands the opponent controls, so a Wasteland on your land does not cut it, and Carpet is cheap to replay. If the opponent has no Wasteland, Carpet's mana gets close to hardcasting Atraxa. This plan sets your turn-1-to-3 plan when you keep (opening-hand-and-mulligan.md OH10) and when you order a Ponder or time a fetch (cantrips-and-shuffling.md CS18, CS19). "Always" is Brady's own plan statement; playbook P15 ("nothing is always") still applies to it. (Brady, interactive match m02, 2026-10-08; m02 game 2: Carpet known on top from Ponder was shuffled away by an end-step fetch, Wasteland took Tropical Island, and Carpet came down only on turn 5)
  Breaks when: Brady named no exception. His reasons rest on the opponent controlling Islands (Carpet's mana is their Island count), so with no Island on their side it makes no mana; that is why it stays out against Boros. Against an opponent without Wasteland the plan stands, and Brady's reason changes from beating Wasteland to mana toward a hardcast Atraxa.

Note: on the second night the seat was tied to the deck, so every game against UR Cutter was on the play and every game against Boros on the draw (SYNTHESIS2). Rules learned from those games were learned from one seat only.

You are not told the opponent's swaps. Their sideboard cards show up as cards outside their game-1 list.

## Base 75 (Brady's list, 2026-10-06)

Main (60): 4 Ancient Tomb, 1 Boseiju, Who Endures, 1 City of Traitors, 3 Flooded Strand, 2 Hedge Maze, 1 Island, 4 Misty Rainforest, 1 Savannah, 1 Tundra, 2 Tropical Island, 4 Acererak the Archlich, 4 Atraxa, Grand Unifier, 4 Brainstorm, 4 Force of Will, 4 Ponder, 4 Show and Tell, 4 Stock Up, 2 Veil of Summer, 4 Aluren, 4 Lotus Petal, 2 Omniscience.

Sideboard (15): 4 Carpet of Flowers, 1 Force of Negation, 2 Consign to Memory, 2 Faerie Macabre, 2 Orim's Chant, 2 Prismatic Ending, 2 Veil of Summer.

## What each sideboard card is for (Brady's plan notes)

| Card | Job | Comes in against |
|---|---|---|
| Carpet of Flowers (4) | Mana against decks with Islands; Stock Up is very good with Carpet out; plan to cast it on turn 3 (SB5); its mana is spent first by the auto-payer (process-and-engine.md EF12) | Dimir Tempo, UR Cutter, UWx Control, the mirror (3 against Doomsday, draft). Not Boros: no Islands |
| Veil of Summer (2 more) | Beats Force of Will, Force of Negation and Daze; beats Thoughtseize and Unmask and draws | Dimir, UR, UWx, Reanimator, the mirror, Doomsday (draft). Against Boros the 2 main-deck Veils go out (blank, vs-boros.md BO2) |
| Orim's Chant (2) | Against blue decks a combo-turn tool on your turn; against Reanimator a time walk on their turn (protecting-the-combo.md PR14); against Boros it covers Swords to Plowshares, Pyroblast and Flute | Dimir, UR, UWx, Reanimator, Boros, the mirror |
| Prismatic Ending (2) | Dimir's 1-mana 3/3; UR's turn-1 Dragon's Rage Channeler (vs-ur-cutter.md UR3; fetch Tundra or Savannah for the white); Tamiyo and Lavinia (UWx); Boros hate pieces (Deafening Silence, Teeg, Containment Priest). Does nothing against an Aluren in play | Dimir, UR, UWx, Boros. Not the mirror (mirror-and-other-matchups.md MM2) |
| Force of Negation (1) | Covers their noncreature spells | Reanimator, the mirror, Doomsday (draft) |
| Consign to Memory (2) | In the mirror plan. Not against Boros: "no colorless static prison" | The mirror |
| Faerie Macabre (2) | Answers Reanimate and Animate Dead | Reanimator |

## Brady's plans

- **vs UR Cutter** [Brady, baseline]. Out: 2 Force of Will, 2 Omniscience, 1 Boseiju, 1 Aluren, 4 Lotus Petal. In: 2 Veil of Summer, 4 Carpet of Flowers, 2 Orim's Chant, 2 Prismatic Ending. Why: their clock is faster, so the combo has to fit into one turn; keep 2 Force of Will, no Petal; 4 Stock Up stay (very good when Carpet is out and they are threat-light but interaction-heavy).
- **vs Boros Aggro** [Brady, tentative]. Out: 2 Veil of Summer, 1 Stock Up, 1 Aluren. In: 2 Prismatic Ending, 2 Orim's Chant. Why: Ending is an easy swap for Veil and answers their hate pieces; Chant covers Swords, Pyroblast and Flute; all 4 Force and all 4 Petal stay (3 Pyroblast around, and speed matters); no Carpet (no Islands), no Consign. Stock Up after a mulligan should put 1 card back (vs-boros.md BO1).
- **vs Dimir Tempo (UB)** [Brady]. Out: 4 Lotus Petal, 4 Force of Will, 2 Omniscience. In: 2 Veil of Summer, 4 Carpet of Flowers, 2 Orim's Chant, 2 Prismatic Ending. Why: Veil is good because of Force of Will, Force of Negation and Daze; Ending (always in) answers their 1-mana 3/3.
- **vs UWx Control** [Brady]. Out: 4 Lotus Petal, 2 Force of Will, 2 Omniscience, 2 Aluren. In: 2 Veil of Summer, 4 Carpet of Flowers, 2 Orim's Chant, 2 Prismatic Ending. Why: like UR, but they kill slower and their mana denial is stronger; they have many Swords, so go for Atraxa first and set up several Acereraks; Ponder and City of Traitors stay; Ending has to hit Tamiyo and Lavinia.
- **vs Reanimator** [Brady]. Out: 2 Omniscience, 1 Stock Up, 3 Show and Tell, 1 Island. In: 2 Veil of Summer, 2 Faerie Macabre, 1 Force of Negation, 2 Orim's Chant. Why: Show and Tell is risky because they put in something big; one stays for the high-resource line (mirror-and-other-matchups.md MM3); City of Traitors stays over the Island (MM4); Chant is a time walk on their turn.
- **vs the mirror (Alurentell)** [Brady, baseline]. Out: 1 City of Traitors, 4 Lotus Petal, 4 Show and Tell, 2 Omniscience. In: 4 Carpet of Flowers, 2 Orim's Chant, 2 Veil of Summer, 2 Consign to Memory, 1 Force of Negation. Why: whoever blinks first loses and more Stock Ups usually wins (MM1); Ending does not matter against Aluren. Cutting City is unsure.
- **vs Doomsday** [draft]. Out: 4 Stock Up, 2 Omniscience. In: 3 Carpet of Flowers, 2 Veil of Summer, 1 Force of Negation. Why: a race between two combo decks that both fight with Force of Will and Daze.
- **No plan yet:** BW Death and Taxes, and the Stormchaser's Talent UR deck (decks/ur-cutter-alt.txt). With no plan the match runner plays the main deck; if you choose swaps yourself, use SB3.
