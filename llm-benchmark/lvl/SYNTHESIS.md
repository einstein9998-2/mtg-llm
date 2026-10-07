# LvL overnight synthesis (reviews w1g1-w1g4, g5-g40; g21 tainted, A-side play still used)

Sources: 40 reviews in reviews/ (w1g1-w1g4, g5-g40, including late-arriving g37, g38, g40). g41-g43 have no review; g42 has player reports only and is not used.
Line numbers (L) refer to alurentell-playbook.md as of this morning. "Brady line" = lines 71-102 or the Brady-sourced top sections.

## 1. Candidate Alurentell playbook rules

Sorted by number of supporting games, then confidence.

1. **Lost Mine lap = 4 Acererak casts, 1 drain, 1 draw.** Each lap is Cave Entrance, Goblin Lair, Dark Pool (drain 1), Temple of Dumathoin (draw 1), so before starting count opponent life <= library + 1, and do not cast free Brainstorm/Ponder mid-loop.
   - Games (13): w1g1, g5, g7, g12, g13, g15, g18, g22, g23, g24, g30, g36, g38. Confidence: high (counted from logs).
   - Evidence: g22 ended at library 0 vs 36 life (143 casts = 35x4+3); g38 library 34 vs 25 life.
   - CORRECTS L10: "Dark Pool (drain 1) is the three-cast cycle that drains once per lap, so about three Acererak casts per point of life".
2. **Auto macro for the loop: rule order and budget.** Put `stack:Pass priority` before `Cast Acererak`, set one budget of about 20 picks per point of opponent life (about 400 for 20), and stop when library <= opponent life.
   - Games (11): g15, g17, g18, g22, g23, g24, g25, g26, g28, g35, g36. Confidence: high.
   - Evidence: cast-first stacked 15-35 room triggers (g17, g23, g25, g26, g28), and in g24 a second Acererak died to the legend rule. Small budgets caused 3-6 relaunches in nearly every macro game.
   - EXTENDS process lines L56/L69 ("One pick per command, always"); the playbook has no macro line yet.
3. **Count mana before planning.** Show and Tell is {2}{U} and Aluren is {2}{G}{G}; Ancient Tomb counts as 2 and a Petal on the battlefield as one coloured mana; cast Veil only if the combo spell is still castable this turn (Veil + Show and Tell = 4 mana, Veil + Aluren = 5 with GG+G); spend City of Traitors' mana before the land drop, because the engine cannot float mana.
   - Games (10): w1g2, w1g3, w1g4, g10, g15, g17, g20, g30, g35, g37. Confidence: high.
   - Evidence: g15 assumed UU and passed a turn-3 win; g17 wasted Veil after reading Aluren as {2}{G}; g37 missed Tomb + Petal = Show and Tell; in g10 and w1g3 the first fetch or reveal left no GG; in g30 and w1g2 City was sacrificed unused.
   - EXTENDS L23 ("Omniscience costs ten mana ({7}{U}{U}{U})"), the only cost listed, and L39 ("City of Traitors is sacrificed as soon as I play another land").
   - The g10 sub-point "fetch Tropical first when Tomb/City is in hand" is low confidence and in tension with L22 ("prefer basics or fetches when I am short on lands").
   - Deck note: the current list has no basic Forest and no black land (decks/alurentell.txt; g9, w1g3), so L42 and L87, which mention Forest, are out of date.
4. **Life is mana; count their reach.** Before paying Tomb or fetch life, check that your life afterwards stays above their instant reach (Goblin Bombardment = their creature count; a second Ajani flip = creature count with a red permanent; free Phlage = 3 under your Aluren; burn to 2 or less turns off your Tombs and Force); Force of Will the card that makes it lethal.
   - Games (10): g8, g10, g11, g12, g16, g19, g26, g29, g31, g34. Confidence: medium.
   - Evidence: g16 cast Aluren at 8 life into 9 Bombardment creatures; g11 and g34 died to a second Ajani; g26 passed on Ajani #2 at 6 life and survived only by B's misclick; g31 did not Force a Bolt that took A to 1; g29 let Cutter resolve and went 14 to 5.
   - CORRECTS L23 "at 4 life or less the engine will not offer it": g8 (2 life) and g19 (3 life) were offered Tomb-only casts. Unverified whether the engine refuses any case.
5. **Second Acererak.** On the combo turn take every card Atraxa offers, a spare Acererak first, and when discarding to hand size with Aluren in hand keep Acererak and drop a second Atraxa.
   - Games (8): g6, g7, g12, g18, g24, g32, g36, g37. Confidence: high.
   - Evidence: in g24 the spare would have covered the legend-rule loss plus Swords; in g32 and g36 the second copy beat removal on the first.
   - EXTENDS L99: "Atraxa reveal: do not take 0 cards. The only reason to take fewer than allowed is ... hand size". On the combo turn hand size never applies.
6. **Daze math.** Daze is live whenever they control any Island-typed land, tapped or untapped (Volcanic, Thundering Falls), and only an untapped land or a Petal already on the battlefield pays for it, because a Petal in hand cannot be cast in response.
   - Games (8): w1g1, g8, g13, g21, g25, g27, g35, g39. Confidence: high.
   - Evidence: g27 flagged "Daze impossible" with only a tapped Falls out; w1g1 and g39 had Stock Up Dazed with a Petal still in hand; g35 faced no land at all, so Daze was impossible.
   - CORRECTS L48 ("Look at what the opponent has untapped: with only a Wasteland out, Daze is impossible"; the word "untapped" misleads) and EXTENDS L47 ("Spare mana beats Daze ... with a Petal ... spare").
7. **When the loop is available, run it now and to the kill.** With Aluren or Omniscience in play (or castable) and Acererak in hand, loop at once and keep choosing Lost Mine until they are dead.
   - Do not cantrip, attack or take Tomb of Annihilation first. Use Trapped Entry only for the last point, with the opponent at 1. If you do enter Tomb, take Veils of Fear and Sandfall Cell, never Oubliette (g6). Force their Force on the winning spell (w1g2).
   - Games (7): w1g2, g5, g6, g7, g10, g11, g12. Confidence: high.
   - Evidence: g7 passed with the loop available three times; g11 held Aluren + Acererak and died; g12 took Tomb "for an Atraxa swing".
   - EXTENDS L7 ("Default: do not complete Tomb of Annihilation") and answers open question L30. Note: w1g1 (before the macro existed) advised "stop after a lap or two and win by attacking"; later games supersede it.
8. **Spend Force of Will on what breaks the loop.** Removal on Acererak while his ETB trigger is on the stack ends the loop, so hold Force for that, not for Swords on a Show-and-Tell Atraxa (it gives 7 life and the reveal still resolves) or for face burn when you are tapped out and they have Daze.
   - Games (7): w1g1, g8, g22, g24, g28, g32, g36. Confidence: medium-high.
   - Evidence: g22 held Force through Swords, then countered Erode on Acererak; in g24 Force saved the loop from Swords; in g8 Force on Bolt was Dazed and A went 15 to 2; in w1g1 Bolt + Unholy Heat in response to the ETB stopped the loop; in g32 and g36 removal killed the first copy and the second copy won.
   - New; nearest is L66 ("Their Force of Will on Aluren can be answered with my Force of Will").
9. **Veil of Summer is a blank against Boros in game 1.** After a mulligan, bottom the matchup-dead card (Veil against Boros) and keep the Show and Tell payoff (Omniscience, Atraxa), which is also Force fodder.
   - Games (7): g20, g24, g26, g33, g34, g36, g38. Confidence: medium-high.
   - Evidence: g38 bottomed Omniscience and kept Veil, about 6 turns lost; g33 and g34 kept Veil over a payoff or Aluren.
   - EXTENDS L86 ("Not Veil of Summer, not Show and Tell"), which applies to Bauble decks, not Boros.
10. **Boros permanents shape the combo turn.** Voice of Victory stops your spells on their turn, not theirs on yours; count their untapped white (and Lazotep Quarry with a creature) as Swords before looping; Karakas bounces a legendary Atraxa/Acererak; Spider-Woman makes your Petals and the Show-and-Tell Atraxa enter tapped (cast Petals a turn early).
    - Games (7): g9, g11, g14, g16, g18, g20, g30. Confidence: medium-high (card text).
    - Evidence: g9 misread Voice; in g14 the Petals entered tapped; in g30 Karakas bounced Atraxa.
    - New; the only Boros line is L74 ("Against Boros after a mulligan: Stock Up should put one card back").
11. **Hedge Maze timing.** At their end step fetch Hedge Maze over a dual or basic when either gives the needed colours, and play Maze from hand on a turn the untapped land's mana would go unused.
    - Games (6): g5, g28, g34, g36, g39, g40. Confidence: medium.
    - Evidence: g28, g36 and g39 fetched Savannah/Tropical instead; g40 played Boseiju before Ponder and left its G idle.
    - EXTENDS L65 ("Hedge Maze fetched at their end step enters tapped, untaps on my turn and still surveils").
12. **Brainstorm, then crack the fetch, then Ponder.** Put back duplicates (a second Atraxa, a second Show and Tell, extra lands), never your only Aluren or Acererak, then crack the fetch before Ponder so Ponder sees three fresh cards.
    - Games (5): w1g4, g21, g25, g29, g30. Confidence: high on sequencing, medium on the put-back list.
    - Evidence: w1g4 and g21 wasted Ponder on the put-backs; g25 shuffled away its only Aluren; g30 put back Acererak and kept 2 Atraxa.
    - EXTENDS Brady L83 ("Fetch away the cards you put back with Brainstorm if they are bad"). Partly CONTRADICTS L49 ("Cast Brainstorm before Ponder so Ponder's shuffle clears the put-backs"), an observed line written for Omniscience.
13. **Aluren is symmetric.** Once it resolves, the opponent casts MV<=3 creatures free at instant speed (free Phlage deals 3; Amped Raptor can discover Swords/Erode/Bombardment), so do not cast Aluren at 3 life or less against Boros.
    - Games (5): w1g1, g10, g28, g35, g36. Confidence: high (fact), medium (advice).
    - Evidence: in g10 B passed 33 times on a free lethal Phlage; g36 Raptor flips; in w1g1 and g35 B was offered a free DRC.
    - EXTENDS Brady L75 ("Mirror: Aluren and Show and Tell are symmetric"), which covers the mirror only.
14. **Ponder shuffle.** Without an enabler in hand, keep a known Aluren/Show and Tell on top, and otherwise shuffle any look with no enabler, creature or cantrip; a land alone is not enough unless you are stuck on one land with none in hand (then shuffle any landless look).
    - Games (5): g19, g21, g27, g31, g33. Confidence: medium.
    - Evidence: g19 kept Island/Petal/Petal and lost; g21 binned and shuffled away Aluren; g27 kept a landless look on one land.
    - EXTENDS L41 ("With no creature among the three cards and a creature-dependent hand, shuffle").
15. **Show and Tell with Acererak in hand puts in Aluren, not Atraxa.** Then cast the free Acererak and loop; this includes a known Acererak on top with a cantrip and a spare mana to draw it.
    - Games (4): g20, g30, g32, g35. Confidence: high against a tapped-out opponent, medium against Boros with W open (see rule 13).
    - Evidence: g35 did it and won on turn 1; g30 and g32 lost 2-4 turns by taking Atraxa.
    - EXTENDS Brady L73 ("Show and Tell versus Aluren: decide by how much mana I have and which other combo pieces are in my hand").
16. **Stock Up with two or fewer lands and none in hand: take a land first.** Prefer one Wasteland cannot hit, take a creature only if Aluren or Show and Tell can cast it next turn, and with Aluren + Acererak already in hand dig for mana.
    - Games (4): g14, g20, g36, g40. Confidence: high (g20), medium otherwise.
    - Evidence: g20 took Acererak over Tomb and was stranded on 2 lands; g36 kept a Stock Up on top instead of mana.
    - EXTENDS Brady L101 ("with enough lands in hand ... then Atraxa or Force, not another land"), the converse case.
17. **A keep needs a route to a payoff.** A seven whose only spells are Show and Tell + Acererak, or one land + Ponder with no creature, is a mulligan; run the keep sim on the missing card, not on lands, and do not keep a six that is worse than the seven thrown back.
    - Games (4): w1g1, g19, g34, g39. Confidence: medium-high.
    - Evidence: g19's sim measured lands at 90% and the hand never found a creature.
    - EXTENDS Brady L88 ("a 7 with no cantrips and no blue card for Force of Will ... is a mulligan") and L102.

### Needs Brady's ruling (contradict or limit Brady's own lines)

- **B1. Hold Veil for their first counter.** On the combo turn, hold Veil for their first Daze/Force instead of casting it first: in response it makes every spell that turn uncounterable (each free Acererak included) and draws a card. With exactly Show and Tell + 1 mana, cast Show and Tell first and hold the 1.
  - Games (4): w1g1, w1g2, g29, g31. Confidence: medium.
  - Evidence: in g29 Veil in response beat Daze twice; in g31 Veil in response to Daze was answered with no Force.
  - CONTRADICTS Brady L97 ("Jam line with Veil: land (Ancient Tomb), Veil of Summer, then Show and Tell"). Self-flagged g29a 112.
- **B2. One-land and no-cantrip keeps.**
  - Exception wins: g23, g37 (one land + Petal + 2 cantrips + both combo halves); g12 (one land + Ponder); g9, w1g1 (no cantrip). The g33 reviewer leaned keep on a one-land Ponder seven; the g28 reviewer called a no-cantrip seven a keep against Boros when it already has turn-2 Aluren mana plus Acererak. Rule-confirming losses: g19, g34.
  - Games (8): g9, g12, g23, g28, g33, g37 vs g19, g34. Confidence: low.
  - CONTRADICTS Brady L102 ("One land with Ponder is a mulligan unless ... Storm or Reanimator") and L88.
- **B3. Wait a turn rather than jam with Veil as the only spare.** With Veil as the only spare against an opponent at 5+ cards and no clock, cantrip and wait a turn for one more mana.
  - Games: w1g1 (B held Daze + FoW + FoW); g37 waited without a spare and lost two turns. w1g2's reviewer argues the other way: the wait broke Brady's rule and cost two turns. Confidence: low (conflicting reviews).
  - LIMITS Brady L96 ("when you hold protection ... jam it now").

## 2. UR Cutter notes (opponent side)

- **Keep one blue pitch card per Force of Will.** Never Brainstorm, Preordain-bottom or surveil away Daze, Force or the last pitch card against a combo deck (w1g1, w1g4, g6, g21, g25, g37). In g6 and g25 B lost with a pitchless Force in hand.
- **Counter the Veil, not the spell behind it.** Force a Veil cast on their own turn: once it resolves, every spell that turn is uncounterable (g6, g7, g31).
- **Wasteland targets.** Ancient Tomb when Show and Tell is the threat; the green dual once Aluren is seen (GG); their only land on turn 2 against a one-land keep; the land that would give them their third mana (w1g2, g5, g17, g19, g25, g27, g33, g37).
- **Cast cantrips before the land drop** when your lands are fetches or a Wasteland could be found (g17, g19, g27). In g17 a late Wasteland would have stopped Aluren.
- **Do not tap out on your own turn for Murktide/Cutter once they can reach 4 mana.** Hold U(+R) for Prismari Charm, which bounces Aluren in response to the free Acererak, or Unholy Heat (g5, g13). Brainstorm at their end step or in response, not main phase (g15).
- **Acererak answer.** Bolt + Unholy Heat (delirium) on Acererak with his ETB on the stack ends the loop (w1g1). g23's note "Bolt does nothing to the loop, Acererak returns by himself" holds only for Bolt alone (3 < 5 toughness); it conflicts with w1g1, g32 and g36, where removal in response stopped the return.
- **Daze timing.** Use Daze on a tapped-out Stock Up or Show and Tell (w1g1, g23, g39) or on their Force when they are tapped out (g8). It is nearly dead into open green mana, because Veil answers it (g29).
- **Mishra's Bauble.** Crack it early: free card plus the artifact type for DRC delirium (g13, g23, g33).
- **Keep and sequencing.** Keep a one-lander on the draw with Brainstorm plus a free counter, counting Brainstorm's 3 cards in the land odds (g15); cast Cutter before the second spell of the turn (g39).
- **Against Atraxa.** Chump early when it turns on delirium; Heat at the end step on the turn she took block damage; use Charm's bounce mode, not surveil; burn face to 2 or less to turn off their Tombs and Force (w1g2, w1g3, g31).

## 3. Boros Aggro notes (opponent side)

- **Wasteland timing and targets.** With no 1-drop, play Wasteland as the turn-1 land and fire it in response to their first spell; hit Ancient Tomb or their only blue/green dual, not an uncracked fetch; never surveil Wasteland away while they are short of lands (g9 never activated it and lost; g14, g20, g24, g30, g34, g38, g40).
- **Swords on Acererak.** Keep W open (or Quarry plus a creature) for Swords on Acererak with his ETB trigger on the stack, and fire on the first cast, because each lap draws them a card (g9, g22, g28, g32, g36). A second copy beats it (g32, g36).
- **Under their Aluren, cast your MV<=3 creatures free at instant speed.** Free Phlage kills at 3 life or less (g10, missed 33 times); cast Amped Raptor last so its discover resolves above Swords (g28, g36).
- **Goblin Bombardment.** Hold it for their combo turn: let them pay Tomb and fetch life, then sacrifice everything in response (g16); ping Atraxa after combat (g12); kill the first Acererak on its ETB (g32).
- **Second Ajani is reach.** Cast the second Ajani while the first is out: the legend-rule death flips the survivor, and Avenger's 0 deals damage equal to your creature count with a red permanent (g11, g26, g34, g40). The menu's "ability 1" is the +2 and "ability 2" is the 0; pick by elimination (g26 lost lethal to this).
- **Voice of Victory first among 2-drops against Force decks** (g11, g18, g20). Keep Karakas untapped or saved for a legendary Atraxa (g18, g30); Spider-Woman early makes their Petals and Atraxa enter tapped (g14).
- **Swords the Show-and-Tell Atraxa before attacking even though they gain 7** (g24, g38). g22 suggests first counting your removal against their Force, since Acererak needs an answer too.
- **Ascend and token timing.** Count to ten permanents before the end step (g16); order Voice's sacrifice trigger first so Ocelot Pride copies the mobilize Warriors (g36).
- **One pick per command.** A scripted Bombardment index loop hit The Atropal instead of the player and lost a won game (g12).

## 4. Harness and engine issues

- **Planeswalker ability labels** (menu/labelling): "ability 1/2" with no loyalty cost or text, and no index 0 (g11, g26; this decided g26). Karakas likewise shows only "ability 1" (g30).
- **Aluren and Show and Tell auto-payment over-taps** (likely engine bug, payment): it taps an extra source and leaves 1 floating (g17, g23, g26, g37) and prefers Ancient Tomb over painless sources (2 life lost in g32 and g38). Request least-cost/painless payment or manual payment.
- **Possible hidden-information leak** (possible engine bug, unverified): after Atraxa sent a revealed Acererak to the bottom and Erode shuffled A's library, B's known-cards list showed Acererak in A's hand after A drew one (g32 B prompt 49). Known-card memory should clear on a shuffle.
- **Dungeon completion timing** (possible engine rules deviation, unverified): completion is counted while Temple of Dumathoin abilities are still on the stack (g17 A prompt 55; CR 309.7). No effect on the loop.
- **Missing priority windows** (likely engine/auto-pass bug): no window in the declare-blockers or combat-damage step (g7, g12); no upkeep window for the non-active player holding Wasteland (g9); no end-step stop for the non-active player when the stack is empty (g12, from STATE); free castable spells skipped after the Oubliette trigger (g6).
- **Duplicate unlabeled menu entries** (menu/labelling):
  - Hard versus free (Aluren/Omniscience) "Cast X" entries (w1g1, g7, g9, g13; a hard cast happened by mistake in g13).
  - Legend-rule choices with no ids (g11, g24, g26, g34); Daze PayCost lists tapped and untapped Volcanic identically (g25, g31).
  - Ponder/Stock Up/Brainstorm/Atraxa lists collapse duplicate names (w1g4, g11, g12, g19, g21, g22, g25, g40); "#0" ids (g18, g24, g39, g40).
- **Lazotep Quarry menu noise** (menu): 15-35 "Tap Quarry for X, sacrificing Y" entries in every B menu (g11, g14, g20, g22, g24, g34, g40). g20 also notes no plain colourless tap is offered; unverified against card text (possible engine issue).
- **The loser sees no final events after GAME OVER** (harness limitation): g11, g14, g15, g17, g18, g22, g27, g33. Players misreport how they died.
- **Mana abilities are never offered** (harness limitation), so mana cannot be floated (for example City of Traitors before a land drop) (g30).
- **Sim returns 0.0% at a mulligan prompt** (harness tool issue, unverified): g25 B, g34 A; g38 A got "unhelpful, 0 results". g34 notes 0% may be the true answer if `turn` stops before A's first land drop; g12 A claimed lands_playable ignored the land in hand. Make the sim warn at Mulligan prompts and print the simulated turns.
- **Daze "Pay to avoid?" prompt still asked after Veil made the spell uncounterable** (menu; legal but misleading): g29 paid and wasted Boseiju's mana; g31 prompt 118 shows the same.
- **Tomb-only casts are offered at lethal life** (menu/engine; legal by the rules): g8 at 2 life, g19 at 3. This contradicts playbook L23.
- **Unlabelled prompts** (labelling):
  - Bare "Do it?" May prompts: Ponder shuffle, Veils of Fear, Acererak attack trigger, Erode search (g7, g8, g12, g14, g16, g19).
  - Delve shows "remaining: 1" with no Done (w1g3, g5, g27, g31, g33, g39); g5 asks whether over-delving can be forced (unverified).
  - No mana costs in "Cast X" (g15); no static-effect notice, so Spider-Woman taps looked like a bug (g14).
- **Log footer counters are game-wide and printed on one seat's line** (labelling); players misattribute them (w1g1, g8, g14, g15, g19, g21, g22, g37, g39).
- **Isolation** (harness): a shared /tmp helper collision printed another game's prompts (g10, g13); `show` of the other seat leaked A's hand and tainted g21; `ps aux` showed other sessions (g5).
- **Loop cost** (harness):
  - Before the macro: 350-650 prompts and about 4 h per loop (g5, g9, g10, g13).
  - The macro passes over an opponent's Force on Acererak, because `stack:Pass priority` fires on any stack (g15); it has no deck-out guard (g22); budget stops in most macro games.
  - The `pick 0` slip recurred (g38 prompt 131).
- **Cosmetic:** "spell #0 countered" does not name the spell (w1g2, g39); "(gone)" targets (g32); the completed-dungeons list grows without limit (g23, g26); get_card says Tomb of Annihilation is not in the pool while the engine runs it (g7).

## 5. Matchup observations

- **When A wins.** A's 24 wins (24 reviews plus STATE for g38's turn) ended by game turn:
  - turn 1: g35; turn 4: g28; turn 5: g13, g15, g23; turn 6: g18, g32, g36;
  - turn 7: g9, g17, g37; turn 8: g10, g24, g26, g30; turn 9: g5, g29;
  - turns 10-14: g22, g7, g25, g6, g38; turn 15 or later: w1g1, w1g2, g12.
  - Every win from g13 on was the Aluren/Omniscience + Acererak Lost Mine drain. The earlier long wins were Atraxa beats (w1g1, w1g2) or Atropal (g6), plus the 26-turn g12.
- **When A loses.** Losses ended at turn 6 (g16), turn 7 (g8, g40), turn 9 (w1g4, g14), turn 10 (g11, g27, g31, g33, g39), turn 11 (g20, g34), turn 12 (g19; g21 tainted) and turn 16 (w1g3). A almost never loses once a combo spell resolves with Acererak available; losses are games where A never resolves one.
- **Against UR, the clock plus counters.** DRC delirium, Murktide and Cori-Steel Cutter Monks kill by turns 9-10 (w1g3, w1g4, g8, g19, g27, g31, g33, g39). Force of Will on Aluren or Show and Tell (g19, g27, w1g4) buys those turns.
- **Wasteland on Ancient Tomb, Tropical Island or Hedge Maze** is the most common single cause of A losses: g19, g27, g33 (UR); g20, g34, g40 (Boros). It also delayed wins (w1g2, g24, g25, g37). A's one-land and Tomb-dependent keeps are what it punishes.
- **Against Boros, the curve and the reach.**
  - The Ocelot Pride / Ajani / Guide / Spider-Woman curve (g14, g16, g40); second-Ajani flips (g11, g34; near-lethal in g26); Goblin Bombardment at instant speed (g12, g16).
  - Answers to Atraxa: Swords (g12, g22, g24, g38), Karakas (g30), Bombardment (g12).
  - Voice of Victory turns off A's Force and Veil on Boros turns (g11, g20).
- **Swords or removal on Acererak's ETB** is the only way the loop was broken. Each time it lost to a second Acererak or a Force: g22, g24, g32, g36 (and the w1g1 Bolt + Heat delayed the loop a turn).
- **Self-inflicted A losses or delays.**
  - Not looping when able (g6, g7, g11, g12); Atraxa put in over Aluren (g30, g32); card-cost misreads (g15, g17, g37).
  - Force on face burn (g8); no Force on Ajani #2 (g26); weak keeps (g19, g34, g39).
  - g11 is the only loss with the full combo in hand.
- **Opponent-side mistakes that handed A games.**
  - UR: tapping out on its own turn (g5, g13), pitchless or binned counters (g6, g25, g37), unused Baubles (g13, g23).
  - Boros: missed free Phlage (g10), unused Wasteland (g9), Bombardment misclick (g12), wrong planeswalker ability (g26).
  - Opposing sevens with no Force lost to unopposed early combos (g13, g15, g23, g35).
