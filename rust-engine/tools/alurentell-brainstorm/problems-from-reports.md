# Alurentell recurring problems, from 253 Alurentell-side reports (145 W / 108 L)
Sources: /tmp/claude-0/rep/{overnight-waves 56, postboard-wave2 63, postboard-wave3 134}, files *a.md only. I read all 253 in full and hand-coded each one
(per-game tag file: /tmp/claude-0/-home-claude-mtg-llm/3fbd6638-b293-506a-921f-6b8fb9b1d6f0/scratchpad/codes2.tsv). Counts are "number of games where the
report says it happened", W/L = Alurentell result. They are judgement calls, so treat as approximate (+/- 3). Tags overlap, so one game can sit in several rows.
Ids are written w118 for w118g2a. Keyword greps were used only as cross-checks, and where they differ I say so.
Record by opponent (games.tsv): UR main 41-26, UR alt 40-27, Boros aggro 9-12, Boros energy 25-22, mirror 28-21 (+2 m01 reports, both W).

## 1. Mana problems
- Wasteland hit a key land (Tomb, or the only coloured/green/white dual): 91 games, 46W/45L (UR 63 games 34W/29L, Boros 28 games 12W/16L, mirror 0).
  Wasteland is mentioned in 123 reports; land named in the same sentence (overlapping): Tomb 49, Tropical 37, Hedge Maze 31, Savannah 17, Tundra 10. Stifle: 0 reports.
  On its own it is survivable (46 wins); it kills when stacked with few lands. w97 "3 Wastelands, one on each of Hedge Maze, Ancient Tomb and Tropical Island, which left me on 2 lands"
  w89 "three Wastelands that hit Tropical, Tundra and Savannah in turn" (so no white for a hardcast Atraxa). w176 "Wastelanded my Tropical Island, Hedge Maze and Hedge Maze again on three consecutive turns".
- Too few lands (mulligan to 5 or fewer, one-land keeps that missed, no land drops for 3+ turns): 39 games, 5W/34L. The strongest loss signal. Keyword cross-check: "mulligan to 5 or fewer" 22 reports (7W/15L).
  w151 "I drew no land in turns 2-5" (after a 5-card keep). w104 kept a one-lander with two Brainstorms, found no land. w216 mulled twice, "Ancient Tomb was Wastelanded before I found a second land".
- No blue source (every spell is blue): 14 games, 1W/13L. w20 "never saw a blue source in 3 draws (13 outs)" (kept 2 Tombs + all-blue hand). w148 "I drew no land in 4 draws" (2 Tombs, no coloured land). w241, w249, w83, w173 similar.
- No green source (Aluren GG, Veil, Carpet): 10 games, 2W/8L. w107 "I never found a green source (Island, Tundra, Ancient Tomb were my only lands)": Carpet, Veil and 2 Aluren dead. w123 "Veil was dead in hand all game (no G source)" (won anyway). Also w19, w57, w128, w149, w212, w240.
- No usable white (Chant/Ending): 48 games, 19W/29L (many are "only one W source so kicked Chant needs WW"; keyword cross-check "no white source" 45 reports, 20W/25L).
  w233 "I had no white source in play all game (Hedge Maze, Tropical Island, Ancient Tomb), so both were dead cards". w105 "only Tundra and Savannah make white". w212, w149, w131, w57 same story.
- Flood: rare, 4 games (2W/2L): w81, w155 (mulled a six-land 7), w210, w256 "I drew 8 lands in 11 cards with no payoff until turn 5". Not a real problem.
- Other mana leak (not asked, but common): auto-payer / pay= taps Ancient Tomb (2 life) or over-taps and strands floating Carpet mana: 32 reports (23W/9L). Life spent on Tomb+fetches then mattered against Bolt or exact-lethal swings in w5, w8, w22, w75, w188.

## 2. Dead cards in hand (cards drawn and useless)
- Orim's Chant dead: 46 games, 14W/32L. Reasons: no/too few white (kicker WW) at least 24, Voice of Victory stops casting on Boros's turn 6, discarded to hand size 8. By opponent: UR 19, Boros 22, mirror 5.
  w172/w240 "dead: no WW source, Savannah and Tundra are the only white". w177 and w245: Voice of Victory blanks it.
- Prismatic Ending dead: 17 games, 5W/12L. Causes: no white (w6, w57, w125), cannot hit Murktide (w5 "would trim to 1 Ending since it can't hit Murktide"), no target vs Boros (w97), needs two colours for MV 2 (w83, w213).
- Veil of Summer drawn but never cast/useless: 19 games, 3W/16L. Causes: no green (w19, w107, w128, w148, w212), tapped out (w62, w134, w174), no payoff to protect (w188 "Veils were dead weight (drew 4 of 4 by turn 5)"), believed not to stop Pyroblast (w176 "which Veil did not stop"; but w163 shows Veil beating Pyroblast on Stock Up and w131 lost by misreading it; Veil cannot stop Pyroblast damage on Atraxa).
- Carpet of Flowers dead: 14 games, 4W/10L. Causes: nothing to spend mana on (w26 "I had no cantrips or payoff to spend it on"; w188), few opposing Islands/Wasteland (w67, w103, w199, w215), no green to cast it (w107, w212). Net: Carpet is the best-rated card in 83 reports, but 14 games it was a blank.
- Force of Will dead (no spare blue pitch card, or Voice on their turn): 8 games, 0W/8L. w22 "Force of Will was dead in hand for most of the game (no spare blue card to pitch)". w42 "both Forces became dead cards" (Voice of Victory).
- Second/extra combo pieces: Aluren stranded with no Acererak or creature 8 games, 0W/8L (w7, w68, w77, w102, w107, w159, w181, w233). w102 "3 Alurens in hand, no Acererak, no third land by turn 3".
  Extra Acererak with no enabler: w15 "I held three Acererak and no enabler", w38, w175 (3 Atraxa + 2 Acererak clogged the hand). Second Atraxa dead: w24, w76 ("a second Atraxa was a dead card"; hardcast needs W and B), w175.
  Show and Tell with nothing to put in: w95, w181, w245. Second Omniscience: w233. Mirror: a 2nd S&T/Atraxa/Omniscience is simply boarded out (no complaints there).
  Unique games with a stranded Aluren/Acererak/Atraxa/S&T/Omniscience: 15, all 15 lost (they show up as the "no enabler" losses in section 4).

## 3. Hand size / overdraw
- Discarded to 7 at cleanup: 25 games, 12W/13L.
  After Atraxa's trigger: 8 games, 7W/1L (w10, w12, w100, w110, w112, w113, w204, w205). Almost always harmless: w100 "ending with 9 cards in hand and discarding 2 at cleanup (second Carpet and Orim's Chant)", w204 "discarded two cards at cleanup (Acererak, Stock Up)".
  After Stock Up/Brainstorm/holding Tomb: 14 games, 5W/9L (w50, w62, w82, w97, w98, w150, w154, w156, w170, w184, w197, w202, w206, w250). Mostly protection or answers thrown away: Veil (w62, w98, w206, w250), Chant (w82, w97, w156), Ending (w197), FoN (w170), Consign (w154).
  Mattered in the mirror: w154 "I discarded Consign, which was my only answer to the Acererak bounce trigger" (L). w62 "discarded one to hand size" (Veil; L). Mirror total 4W/6L.
  Stuck with uncastable hand (no lands): 3 games, 3L (w128, w148, w151).
- Cards picked off Atraxa's reveal and never cast: keyword hits in 19 reports (e.g. w44 Carpet, w187 Chant); players rarely say it cost them.
- Cheap fix: w112 "discarded to hand size on turn 2 after taking 3 Brainstorm/Aluren from Atraxa's reveal"; w98 "I took Stock Up and pulled my hand to 8 cards, then discarded a Veil of Summer at cleanup". Count hand size before Stock Up (cast Ponder first).

## 4. How the 108 losses happened
Non-exclusive (a loss can appear in several): opponent's early clock killed me 85 (UR 51 of 53 losses, Boros 34 of 34; mirror 0); combo-turn spell countered 38 (UR 32, Boros 5, mirror 1);
answered after resolving 28 (UR 16, Boros 12); no enabler/payoff all game 32; mirror loop 18 of 21 mirror losses.
Exclusive primary cause (my rule, first match: mirror > mana-starved [no blue/green/too few lands] > combo turn countered > answered after resolving > no enabler > clock only):
  mirror loop 18 | mana-starved 38 (UR 26, Boros 11, mirror 1) | combo turn countered 22 (UR 18, Boros 3) | answered after resolving 13 (Boros 9, UR 4) | no enabler/payoff 15 (Boros 11) | clock only 2.
- Early clock, UR (53 losses; keyword counts among losses): Dragon's Rage Channeler 36, Bolt 30, Cori-Steel Cutter/Monk 28, Murktide 23, Stormchaser's Talent/Otter 18.
  w32 "4/4 trample haste Monk + two 3/3 fliers was exactly lethal from 10". w199 "took me from 13 to 0 in one turn" (Otters + DRC + Bolt). w34 and w183 also 2-3 DRCs with delirium by turn 3-4.
- Early clock, Boros (34 losses): Guide of Souls 23, Ocelot Pride 21, Ajani 17, Voice of Victory 16, Amped Raptor 16, Phlage 6, Deafening Silence 4. w229 "I faced 23 attackers on their turn 5". w177 "11 attackers killed me from 11 life on their 4th turn".
- Countered on the combo turn: Pyroblast (blue-spell mode) on S&T/Atraxa/Stock Up/Brainstorm is the main killer: Pyroblast in 68 reports (UR 56, Boros 12), S&T hit by it in 21 (11W/10L), cantrips/Carpet hit in 18 (7W/11L). Then Daze with no spare mana (23 reports, 8W/15L) and Force of Will/FoN.
  w176 "Three Pyroblasts in one turn was a surprise". w200 "both Show and Tells were Dazed". w50 "they Forced my Force with a free pitch". w34 "it was countered by Pyroblast (blue-spell mode)" (hardcast Atraxa). No discard spells seen from either opponent.
- Answered after resolving (28 L / 43 games total, 15 of them W): Torpor Orb in 20 UR reports (UR alt sideboard, one main-deck), 7W/13L (kills Acererak bounce and Atraxa reveal; Ending or Boseiju is the only out). Static Prison on Atraxa/Omniscience 12 games (8W/4L).
  Karakas bounce 15 mentions (6W/9L), Swords 31 (25W/6L), Mindbreak Trap 8 (5W/3L), Containment Priest 3, Boomerang 9, Erode.
  w101 "they put Static Prison in with my Show and Tell on turn 3 (exiled Atraxa)". w97 "Karakas (bounced Atraxa in response to every Show and Tell". w45 "the second exiled Acererak with his trigger on the stack". w109 "exiled by a free Mindbreak Trap (3 spells in one turn)".
  w175 "they sideboarded Torpor Orb (stops Acererak's and Atraxa's enter triggers". w159 "the deck needs a way to answer Torpor Orb (Ending)".
- Mirror (49 games, 28W/21L): 18 of the 21 losses are the opponent resolving Aluren + Acererak first (Veil backs Force of Will; Chant before Aluren; Boseiju channel). w182 "jamming Aluren first handed them the loop". w186 "My Island fed their Carpet" (lost on their turn 2).
  Carpet feeds the other side (13 reports), Hedge Maze counts as an Island. Wins: Chant first, then Aluren (c5 below).

## 5. Chant or Veil to force through Show and Tell / Atraxa, then Atraxa answered
- Games where Chant (12) or Veil (7) cleared the way for S&T-Atraxa or a hardcast Atraxa: 19 games, 16W/3L. Atraxa then survived and won in 12 of the wins (w10, w55, w72, w79, w124, w147, w166, w192, w224, w235, w236, w247).
- Atraxa answered afterwards in 6 of them (4W/2L), plus one more loss to a raw clock: w71 L "Show and Tell then resolved but they put in a Volcanic and Pyroblasted Atraxa"; w187 L "Atraxa died to Pyroblast on their next turn and Torpor Orb came down";
  w232 W "they Pyroblasted Atraxa at my beginning of combat"; w195 W "Atraxa died anyway"; w100 W (Murktide traded with Atraxa in a block); w46 W (Torpor Orb blanked the trigger, 7/7 body still won); w255 L "Show and Tell gave them a free Cori-Steel Cutter" (14 damage at 3 life).
- Chant/Veil used the wrong way (not the same as 5): w120, w32, w131 Veil spent as a cantrip or held, so S&T died to Pyroblast. w131 "Veil in response would have saved Show and Tell" (Veil does stop Pyroblast-as-counter).
- Wider view: Atraxa or Omniscience answered after it resolved (any protection): 30 games, 17W/13L. Pyroblast 7 games (4W/3L), Static Prison 12 (8W/4L), Karakas/Swords/Erode/Boomerang/Priest the rest. When it happens Atraxa's reveal usually still paid for itself.
- Chant first on the combo turn (before S&T, Atraxa or Aluren; my c5 tag) is the best-rated use: 30 games, 27W/3L, of which mirror Chant-then-Aluren 10W/0L. The 3 losses are w1, w71, w255.

## 6. Cards players said they wanted, or would cut/add (explicit statements; approximate counts)
CUT / DOUBT
- Orim's Chant: 20 reports call it weak/dead/only worth it with more white; explicit "cut/swap" in 9: w105 ("I would swap it for something else"), w109, w172 ("would cut one Chant"), w184 ("I would trim them"), w189 ("swap it for a second Ending"), w193 ("consider cutting Chant for a Veil or Stock Up"), w233, w240 ("I would cut Chant for something castable"), w77 ("Would keep one Chant at most"). But 26 reports rate it key when white is available (w43, w66, w84, w146, w147, w185, w224, w235, w236).
- Prismatic Ending: trim to 1 or 0 vs Murktide/Talent/Otters: w5, w103, w156, w184, w233 (5). Wanted MORE of it as the answer to Torpor Orb/DRC/Priest/Static Prison: w16, w22, w24, w25, w75, w95, w99, w159, w175, w189, w207, w215 (12). Ending needs W plus a second colour.
- Veil of Summer: judged weak vs Pyroblast (partly a misread, see above), dead without green: w75, w176 ("I would drop a Veil for the Force of Negation or another blue pitch card"), w188, w211, w244 (5). Strongly positive in the other 40+ reports.
- Carpet in the mirror: reconsider w130, w134, w138, w150 (4) because it feeds the opponent's Carpet.
ADD / WANT BACK
- Boseiju or other artifact/enchantment answer for Torpor Orb and Static Prison: w25, w187, w215 (and w86, w170, w174 want Boseiju mana in the mirror).
- More white sources (or a way to make WW): w95, w97, w109, w187, w233, w240 (6). More green sources: w107 (1). A black source for hardcast Atraxa/Acererak: w76, w229, w177 (3).
- Force of Will / blue pitch cards back (boarded out vs UR): w17 ("Boarded-out Force of Will x2 was missed once"), w22 (2 main FoW "thin"), w176, w200 (bottomed FoW in mulligan) (4). Spare blue cards to pitch: w16, w28, w22.
- Lotus Petal / fast mana: w103 ("would want Petal/fast mana and a cheaper clock back"); w100 and w98 did NOT miss the Petals (2).
- Second Aluren back in for the UR plan: w207 (1). Mirror: Consign was the only answer to the loop (w82, w90, w154, w126), but is pitch fodder in 7 others.
- Cheap early interaction/sweeper/blocker vs Ocelot Pride and Guide boards, and "Force Voice with a spare blue": w9, w12, w42, w93, w137, w245 (6).
- Take-away: the loss pattern is (a) mana (few lands, Wasteland, no coloured source), (b) Pyroblast/Daze on the combo turn, (c) Torpor Orb/Static Prison post-resolution, (d) Boros/UR clocks faster than the deck can dig.
