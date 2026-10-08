# Alurentell playbook

Read this every game. It holds the deck's plan and the principles every decision comes from. For the details of a situation, read the skill file the index at the end names. Every rule in a skill file is tagged with the principle it applies (P1 to P15): it is that principle worked out for one situation.

Sources: rules marked "Brady" are his (tips 2026-10-06, tough-spot reviews 2026-10-06 and 2026-10-07, LvL sign-off 2026-10-07, rulings on the second-night synthesis 2026-10-08, comments on match m01 and answers to its open questions 2026-10-08, rulings from the interactive match m02 2026-10-08, rulings on the LLM-game tough spots 2026-10-08). Game ids are evidence: a = games against the Forge AI, t = the with-tools batch, g = LLM against LLM, m = a best-of-three match (m01: against UR Cutter; m02: the interactive match against UR Cutter). Candidate rules Brady has not ruled on are in PENDING.md, not in the skills.

## How to use this

1. Principles first. They say why every rule exists, and they decide when no rule fits.
2. Each rule in a skill file is a default for one situation, not a law.
3. Before applying a rule, read its "Breaks when:" line.
4. If the situation matches a "Breaks when:", follow the principle instead of the rule. P15 says how to weigh it.
5. A "Judgment, not a rule" note marks a spot where Brady said "depends": there is no default, so weigh it each time.

## The plan

Alurentell is a Legacy combo deck with two routes to the same payoffs. Show and Tell puts in Omniscience, Aluren or Atraxa, and the put-in cannot be countered; a resolved Show and Tell usually wins. Aluren lets Acererak be cast for free again and again: without a completed Tomb of Annihilation he returns to hand every time, and choosing Lost Mine every lap drains with Dark Pool until they are dead.

Cantrips (Brainstorm, Ponder, Stock Up, Mishra's Bauble, Hedge Maze) dig for the pieces. Force of Will and Veil of Summer protect the turn you go off. Fast mana (Ancient Tomb, City of Traitors, Lotus Petal) lets you go off early, paid for in life and in exposure to Wasteland. Casting Acererak instead of Show and Tell is "plan F" because it achieves so little (Brady).

## Principles

1. **Have a route to a payoff.** Keep hands and make plays that lead to a resolved Show and Tell or Aluren with something to put in or loop.
   Why: Brady counts all four combo cards as payoffs, and you need to assemble the right two. Hands that need two more cards (a creature plus an enabler) lose to Force of Will, Daze and Wasteland. Brady (m02): a hand is a keep only if the plan you write down for its first two or three turns works.
2. **See as many new cards as possible; shuffle only when the cards you already know are worse than a random card.**
   Why: cantrips are how the deck finds its pieces. Brady: sometimes you Brainstorm and you like all of the cards, so you do not need to shuffle. The other side (Brady, m01): "Brainstorm gets better if we can shuffle away 2 cards with a fetch", so do not Brainstorm on turn 1 without a shuffle. With two lands: Brainstorm off the land in play, then play and crack the fetch, then Ponder; with one land you have no fetch until next turn, so Ponder first. With a card you want known on top, crack the fetch after your draw, not at their end step (Brady, m02).
3. **Keep what is unique to the combo or pays for Force of Will; give up duplicates and dead cards; take everything that is free.**
   Why: a second Atraxa or a spare land is replaceable, your only Aluren or Acererak and your last pitch card are not. This covers put-backs, Stock Up and Atraxa picks, discards and bottoms; with Atraxa on the battlefield, Acererak beats a second Atraxa (Brady).
4. **Play a land every turn: dig before the land drop, commit after it.**
   Why (Brady): a skipped land drop makes every later decision in the turn wrong; cantrips may find the land, and a land played first gives away information. Veil of Summer against Daze may go before or after the land: Brady calls them similar (P8).
5. **Wait as long as waiting is free; act as soon as waiting costs something.**
   Why (Brady): waiting gets you the draw, their plays and a land drop, but this is not dawdling: a window against Daze or Force, unused mana or a turn are real costs.
6. **Count mana and life before you plan; life is mana.**
   Why: the costs are exact (Show and Tell {2}{U}, Aluren {2}{G}{G}, Omniscience ten), and every Tomb or fetch payment is life their burn and Bombardment can reach. Brady (m02): before a cantrip on a turn that chains spells into Daze, count the Daze payment.
7. **Keep your mana safe from Wasteland: basics and fetches first, nonbasics later, a second green source.**
   Why: opponents hold several Wastelands and fire them in a burst; losing the only green dual once made Aluren uncastable for the rest of the game.
8. **Beat free counters with spare mana or Veil of Summer, and check whether they can pay for them.**
   Why: spare mana beats Daze and Veil beats Force of Will; Daze is live only while they control an Island-typed land. Brady (m01): "veil, then play fetch petal show": Veil first plays around Daze and hides how much mana you have. Playing the land before Veil plays around extra copies of Daze instead; which is better is unclear (Brady). Veil's hexproof is from blue and black only: red Pyroblast still kills Atraxa (Brady, m02). Know what you cast after Veil (Brady).
9. **A resolved Show and Tell wins: jam it with protection, dig when you have no payoff or protection.**
   Why (Brady): a protected Show and Tell will not get better by waiting, and a raw one into Force of Will hands them a surveil and delirium.
10. **With Aluren or Omniscience and Acererak, loop now on Lost Mine; go through Tomb of Annihilation only to beat a lock piece.**
    Why: with Tomb uncompleted Acererak returns to hand and is recast for free every lap; a completed Tomb keeps him on the battlefield, which only matters against tax and lock pieces (Brady). With Omniscience or Aluren out he is free, so cast every Acererak you hold: "we should probably win the game with Acererak"; an Atropal with no lock piece means "the wrong dungeon" (Brady).
11. **Spend your Force of Will on what breaks your combo, not on what only hurts.**
    Why: removal on Acererak while his enter trigger is on the stack ends the loop; Swords on a Show-and-Tell Atraxa or burn to the face usually does not.
12. **Read the opponent's battlefield and untapped mana before every committal play.**
    Why: their open colours tell you which answers are live (Swords, Prismari Charm, Daze), lock pieces change the loop, and their Aluren works for them too.
13. **Value every card for this matchup and this moment.**
    Why: Veil is a blank against Boros in game 1, Orim's Chant is a combo-turn tool against blue decks but a time walk against Reanimator, and Prismatic Ending does nothing against an Aluren in play. Sideboarding applies the same idea between games.
14. **One pick per command, and read the output before the next one.**
    Why: the engine auto-passes and menu indices shift after every action; t18 was lost to a command that held three picks.
15. **Nothing is "always": weigh how often their answer is there and what being wrong costs.** (new in version 2)
    Why (Brady, 2026-10-08): "Don't think anything is always." Veil first plays around Daze probably 95%+ of the time, but not when the card draw matters, because they do not always have Daze. Sometimes hold Ancient Tomb against Wasteland until your 3-drop, because being cut from 3 mana to 1 hurts more than from 3 to 2. Casting Aluren is often better than Show and Tell, but Show and Tell fixes mana and costs 1 less.

## Index: which skill to read

| Situation | Skill file |
|---|---|
| Keep or mulligan; what to bottom | skills/opening-hand-and-mulligan.md |
| Order of land drop, cantrips and combo spells (turn 1 too); whether to wait a turn | skills/turn-sequencing.md |
| Brainstorm, Ponder, Stock Up, Bauble, fetch cracks, Hedge Maze | skills/cantrips-and-shuffling.md |
| Counting mana, paying life, City of Traitors, Petals, facing Wasteland | skills/mana-and-life.md |
| Show and Tell in hand; Atraxa's reveal; Omniscience in play | skills/show-and-tell.md |
| Aluren cast or in play; Acererak, dungeon and room choices | skills/aluren-and-the-loop.md |
| Their Force, Daze or Charm is live; using your Force, Veil, Orim's Chant | skills/protecting-the-combo.md |
| Opponent is UR Cutter | skills/vs-ur-cutter.md |
| Opponent is Boros Aggro | skills/vs-boros.md |
| Mirror, Reanimator, Storm or another deck | skills/mirror-and-other-matchups.md |
| Between games of a best-of-three match; who plays first | skills/sideboarding.md |
| Before the first command; any menu or engine surprise | skills/process-and-engine.md |

When a rule and its principle seem to pull apart, the rule's "Breaks when:" usually says why; if it does not, the principle wins.
