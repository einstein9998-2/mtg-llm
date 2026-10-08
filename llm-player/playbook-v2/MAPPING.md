# Mapping: old playbook to new files

Version 1 mapping, copied unchanged into version 2 except for this note. Rule ids are the same in version 2. The conflict and "Open" notes listed at the end were replaced in version 2 by "Breaks when:" lines or resolved by Brady's 2026-10-08 rulings: see MAPPING-v2.md and CHANGES.md.

Old file: /mnt/project-files/llm-player/alurentell-playbook.md (150 lines). New files: playbook.md (principles P1 to P14) and skills/*.md. Rule ids name the skill file:

OH opening-hand-and-mulligan, TS turn-sequencing, CS cantrips-and-shuffling, ML mana-and-life, ST show-and-tell, AL aluren-and-the-loop, PR protecting-the-combo, UR/URo vs-ur-cutter (URo = their pilot's notes), BO/BOo vs-boros (BOo = their pilot's notes), MM mirror-and-other-matchups, PE process-and-engine.

Where an old line was split, each part is listed. "Pointer" means the other file links to the rule rather than repeating it.

## Line by line

| Old line | Content (short) | New home | Principle | Notes |
|---|---|---|---|---|
| 1 | Title | playbook.md title | | |
| 2 | blank | | | |
| 3 | Source: Brady 2026-10-01; "observed" = a01 to a06 | playbook.md Sources; AL3, AL4 carry "Brady, 2026-10-01"; a01 to a06 rules carry "observed" | | |
| 4 | blank | | | |
| 5 | Heading: Acererak and Tomb | aluren-and-the-loop.md | | |
| 6 | blank | | | |
| 7 | Default: do not complete Tomb | AL3 | P10 | Also the "why" of P10 |
| 8 | Exception (Brady): lock pieces, go through Tomb; check for lock pieces | AL4 | P10, P12 | Special case of AL1 and AL3, noted in file |
| 9 | Tomb rooms hurt both players; a04 opponent discarded Force | AL5 | P10 | |
| 9 (end) | Engine chose my discard and sacrifice | PE6 | P10 | |
| 10 (1st sentence) | Dungeon and room choices are mine (prompts added) | PE5 | P10 | |
| 10 (rest) | Lost Mine rooms; Aluren technique (Brady 2026-10-07); Omniscience lap; "three casts per point" was wrong; Mad Mage and Tomb never needed | AL2 | P10 | The withdrawn estimate is kept only as "this replaces the earlier estimate" |
| 11 | blank | | | |
| 12 | Heading: Show and Tell and Omniscience | show-and-tell.md | | |
| 13 | blank | | | |
| 14 | Omniscience with Atraxa, Aluren or Acererak in hand; free Atraxa second (a04) | ST7 | P1 | Conflict with line 122 (ST8), noted in show-and-tell.md |
| 15 | Free Force through Omniscience; artifact feeds delirium | PR12 | P11, P12 | Pointer from ST and UR |
| 16 | Hand size: dump spare cards, discard lands and duplicates | ST11 | P3 | |
| 17 | blank | | | |
| 18 | Heading: Against UR Cutter (Forge AI) | vs-ur-cutter.md | | |
| 19 | blank | | | |
| 20 | They hold Force and Daze for my first Stock Up; Veil first or 2-for-1 | UR1 | P8, P12 | Overlaps line 36 (PR6); both kept, cross-linked |
| 21 | Do not tap out for Aluren into Daze with no spare (a05) | PR4 | P8 | Pointer from AL and UR; see waiting conflict under line 60 |
| 22 | Wasteland takes Tomb and Tropical; prefer basics/fetches; keep a drop for Hedge Maze | ML6 | P7 | Same idea as 59 and 42; kept as three bullets because each adds a detail |
| 23 | Omniscience costs ten; hardcast needs; Tomb-only casts at low life; engine-oddities item 9 | ML3 | P6 | Pointer from PE |
| 24 | Prismari Charm and Bolt hit Atraxa/me | UR2 | P12 | |
| 24 (end) | Veil in response to Charm worked in a06, drew a card | PR8 | P8, P12 | Merged with line 38 (duplicate); a06 kept as evidence |
| 25 | Menu discipline: indices shift (a06) | PE3 | P14 | |
| 26 | blank | | | |
| 27 | Heading: Open decisions to test | aluren-and-the-loop.md "Open" | | |
| 28 | blank | | | |
| 29 | Hold Acererak for a lock piece or loop now? | AL "Open" | P10 | Kept as open; note added that line 114 (AL1, Brady-signed) says loop now |
| 30 | Tomb completed when opponent is low and no Dark Pool? | AL "Open" | P10 | Kept as open; line 114 settles only the last point (Trapped Entry) |
| 31 | blank | | | |
| 32 | Heading: counted batch a07 to a30 | dropped | | Heading only; batch record is 20W-4L, no rule |
| 33 | blank | | | |
| 34 | Fetch cracks shuffle; crack only for land or to shuffle a known bad top | CS2 | P2 | Instance of Brady's CS1 |
| 35 (1st sentence) | a30 seven: close mulligan, keep would have won | OH6 | P1 | |
| 35 (2nd sentence) | Hands needing two more cards lose to Force, Daze, Wasteland | OH2 | P1 | Also the "why" of P1 |
| 36 (1st sentence) | Two cards pitched to Force on my Stock Up are worth it | PR6 | P8 | Overlaps line 20 |
| 36 (2nd sentence) | Veil in response to their Force fizzles it, draws (a30) | PR7 | P8 | Merged with line 57 (duplicate) |
| 37 | Process: `play.py next` to look; `pick 0` passes (a29) | PE2 | P14 | Merged with line 69 (duplicate) |
| 38 | Veil answers Prismari Charm (a32); keep Petal open vs blue | PR8 | P8, P12 | Merged with line 24 end |
| 39 | City of Traitors sacrificed on next land; play it last (a31) | ML5 | P4, P6 | Merged with line 110 last sentence |
| 40 | Opponent Wasteland: key spell with hittable lands, Petal back for Veil (a32) | ML11 | P7, P8 | Pointer from PR |
| 41 (count) | Ponder prompt lists identical names once; count to three | CS7 | P2, P14 | Pointer from PE |
| 41 (last sentence) | No creature in the look and creature-dependent hand: shuffle | CS6 | P2 | Special case of line 121 (CS5), noted |
| 42 | Wasteland bursts (a36); fetch basics; second green source | ML8 | P7 | Also the "why" of P7 |
| 43 | One-land keeps with Petal and cantrips never found a payoff; creature matters more | OH7 | P1 | Tension with lines 88 and 102 (Brady), noted in OH file |
| 44 | blank | | | |
| 45 | Heading: a37 to a43 (7-0) | dropped | | Heading only, no rule |
| 46 | blank | | | |
| 47 (1st part) | Spare mana beats Daze, Veil beats Force (a43, a37, a40, a41) | PR5 | P8 | Also the "why" of P8 |
| 47 (2nd part) | With neither, a one-turn wait was right (a41, a42) | TS7 | P5, P8 | Conflict with Brady's line 126, noted |
| 48 | Only Wasteland out: Daze impossible (a39) | PR2 | P8, P12 | Merged with line 58 (duplicate); special case of line 113 (PR1) |
| 49 | Show and Tell for Omniscience, free cantrips find the creature; Brainstorm before Ponder | ST9 | P2 | Brainstorm-before-Ponder also in CS4 |
| 50 | Keep an extra blue card as Force pitch when discarding (a40) | PR13 | P3 | |
| 51 | Misty uncracked: costs nothing, Wasteland decoy (a41, a42) | ML9 | P5, P7 | Pointer from CS |
| 52 | Engine auto-passes turn 1; next prompt can be turn 3 (a41) | PE4 | P14 | |
| 53 | blank | | | |
| 54 | Heading: t07 to t18 (10-2) | dropped | | Heading only, no rule |
| 55 | blank | | | |
| 56 | One pick per command; t18 lost; read output | PE1 | P14 | Merged with lines 69 and 150 (duplicates); also the "why" of P14 |
| 57 | Veil in response to Force makes my spell resolve (t15); first beats both (t11); hold green; engine taps all colours | PR7 | P8 | Merged with line 36; engine note also pointed to from PE |
| 58 | Only Wasteland: window for Aluren with no spare (t13, t16) | PR2 | P8, P12 | Merged with line 48 |
| 59 | Do not play a nonbasic into an untapped Wasteland (t18) | ML7 | P7 | Overlaps lines 22, 42 |
| 60 | If Daze is live and no spare, wait a turn; do not skip land drops (t18) | TS8 | P5, P8 | Conflict: Brady's line 126 qualifies it (wait only when it helps you more); noted in TS |
| 61 | blank | | | |
| 62 | Heading: t19 to t26 (8-0) | dropped | | Heading only, no rule |
| 63 | blank | | | |
| 64 | Opponent tapped out on its turn: only Force and Daze remain | PR3 | P12 | |
| 65 (1st part) | Basic fetched at their end step avoids Wasteland | ML10 | P7 | |
| 65 (2nd part) | Hedge Maze fetched at end step enters tapped, still surveils | CS13 | P2, P7 | Merged with line 118 first part |
| 66 | My Force answers their Force on Aluren when they are at one card; pitch Atraxa | PR11 | P11, P3 | |
| 67 | Put-backs drawn next turn unless a crack shuffles; hold fetch to their end step; never hold Petal/Stock Up on top by accident | CS3 | P2 | Merged with line 83 last sentence; relation to line 100 noted in CS |
| 68 | Under Omniscience bait with the redundant spell; Unholy Heat on Goblins | ST10 | P12 | Pointer from UR |
| 69 (1st part) | Do not use `pick 0` to look; use `tplay next` | PE2 | P14 | Merged with line 37 |
| 69 (2nd part) | One pick per command (t26) | PE1 | P14 | Merged with line 56 |
| 70 | blank | | | |
| 71 | Heading: Tips from Brady 2026-10-06 | sources kept as "Brady, tips 2026-10-06" | | |
| 72 | blank | | | |
| 73 | Show and Tell vs Aluren: decide by your mana and pieces | ST1 | P9, P13 | |
| 74 | Boros after a mulligan: Stock Up puts one back | BO1 | P1, P13 | Pointer from OH and CS |
| 75 | Mirror: symmetric, more Stock Ups wins, use the opponent's Aluren (example) | MM1 | P12 | Also the "why" of P12 (their Aluren works for them too) |
| 76 | Prismatic Ending out of the mirror | MM2 | P13 | Also the "why" of P13 |
| 77 | Orim's Chant: combo tool vs blue, time walk vs Reanimator | PR14 | P13 | Pointer from MM; also the "why" of P13 |
| 78 (1st sentence) | Reanimator: keep one Show and Tell for the high-resource line | MM3 | P9, P13 | |
| 78 (2nd sentence) | Reanimator: City of Traitors over a basic | MM4 | P6, P13 | Pointer from ML |
| 79 | blank | | | |
| 80 | Heading: tough-spot review batch 1 | sources kept as "Brady, tough spots batch 1" | | |
| 81 | blank | | | |
| 82 | Never Brainstorm in upkeep; wait while waiting is free; not dawdling | TS4 | P5 | Also the source of P5 |
| 83 (land vs committal) | Land before committal spells, cantrips before the land | TS2 | P4 | Source of P4 |
| 83 (main phase) | Cantrips in main phase, never upkeep | TS5 | P5 | Duplicate of part of line 82; kept as its own bullet |
| 83 ("insane", recurring error) | Passing with a land in hand is "insane"; committal before land | TS3 | P4 | |
| 83 (last sentence) | Fetch away bad Brainstorm put-backs | CS3 | P2 | Merged with line 67 |
| 84 | No raw Show and Tell without payoff or protection; Veil-first sequence | ST2 | P9 | Same line as 97 (ST4); both kept, relation noted |
| 85 | No payoff: Hedge Maze before Veil | CS14 | P1, P2 | Pointer from ST |
| 86 | Bauble: put back the card that helps them least (Ponder) | CS10 | P3, P12 | |
| 87 | Atraxa first pick: Tropical, or Forest vs Wasteland/City | ST15 | P3, P7 | Pointer from ML |
| 88 | Mulligan: no cantrips and no blue = mull; two Force + Maze + dig = keep | OH3 | P1 | |
| 89 | Opponent at 2 cards, about to reach 4 mana: Brainstorm, then Veil + Show | ST6 | P9, P12 | |
| 90 | blank | | | |
| 91 | Source and numbers: batch 1, 3 of 9 agreed; recurring error | TS3 (recurring error, incl. "or into Daze/Force") | P4 | The agreement count (3 of 9) is dropped: a score, not a rule |
| 92 | blank | | | |
| 93 | Heading: tough-spot review batch 2 | sources kept as "Brady, tough spots batch 2" | | |
| 94 | blank | | | |
| 95 | Play a land every turn; Brady refused three moot positions | TS1 | P4 | Also the "why" of P4 |
| 96 | With protection, jam Show and Tell; Acererak is "plan F" | ST3 | P9 | "plan F" also in playbook.md plan |
| 97 | Jam line with Veil (Tomb, Veil, Show for Atraxa) | ST4 | P9, P8 | See line 84 |
| 98 | Fetch before Show and Tell plays around double Daze | ST5 | P9, P4, P8 | Special case of TS2, pointed to from TS |
| 99 | Atraxa: do not take 0; take green source, Force, then Aluren | ST12 | P3 | General case of line 112 (ST13) |
| 100 | Brainstorm with 4 lands: put back Petal, not a land | CS9 | P3 | |
| 101 | Stock Up with enough lands: take the other Stock Up, then Atraxa or Force | CS11 | P3 | Complements line 123 (CS12): opposite land counts, no conflict |
| 102 (1st, 2nd sentence) | Two surveil lands with live Force: keep (close); turn-1 Stock Up: keep (close) | OH4 | P1, P2 | |
| 102 (3rd, 4th sentence) | One land with Ponder: mull unless Storm or Reanimator; no coloured source or no cantrips with one land: mull | OH5 | P1, P13 | Pointer from MM; relation to line 124 noted |
| 103 | blank | | | |
| 104 | Source: batch 2, 4 of 14 agreed, 2 moot | dropped | | Agreement score, no rule; the moot positions are in TS1 |
| 105 | blank | | | |
| 106 | Heading: LvL rules, Brady signed off 2026-10-07 | sources kept as "Brady, LvL sign-off" | | |
| 107 | blank | | | |
| 108 | Source: llm-benchmark/lvl/SYNTHESIS.md; Lost Mine technique is above | dropped | | Pointer only; evidence lives in SYNTHESIS.md; technique is AL2 |
| 109 | blank | | | |
| 110 (costs) | Count mana: Show {2}{U}, Aluren {2}{G}{G}, Tomb 2, Petal 1 | ML1 | P6 | Also the "why" of P6 |
| 110 (Veil) | Cast Veil only if the combo spell is still castable | ML2 | P6, P8 | Pointer from PR |
| 110 (City) | Spend City's mana before the land drop | ML5 | P4, P6 | Merged with line 39 |
| 111 | Life is mana: check life against their instant reach | ML4 | P6, P12 | Pointer from BO |
| 112 (1st sentence) | Combo turn: take every card Atraxa offers, spare Acererak first; hand size never applies | ST13 | P3 | Special case of line 99 |
| 112 (2nd sentence) | Discarding with Aluren in hand: keep Acererak, drop second Atraxa | ST14 | P3 | Pointer from AL |
| 113 | Daze live with any Island-typed land; what pays for it | PR1 | P8, P12 | General case of lines 48/58 |
| 114 | Loop now on Lost Mine; no cantrip, attack or Tomb first; Trapped Entry last; Tomb rooms to take | AL1 | P10 | Source of P10; answers most of line 29 |
| 115 | Spend Force on what breaks the loop | PR10 | P11 | Source of P11 |
| 116 (1st sentence) | Veil is a blank vs Boros in game 1 | BO2 | P13 | |
| 116 (2nd sentence) | After a mulligan bottom the dead card, keep the payoff | OH8 | P3, P13 | Pointer from BO |
| 117 | Boros permanents: Voice, untapped white as Swords, Karakas, Spider-Woman | BO3 | P12 | Pointer from ML (Petals) |
| 118 (end step) | Fetch Hedge Maze at their end step when it gives the colours | CS13 | P2, P7 | Merged with line 65 second part |
| 118 (from hand) | Play Hedge Maze when the untapped land's mana would go unused | CS15 | P2, P4 | Pointer from TS |
| 119 (1st sentence) | Put back duplicates, never the only Aluren or Acererak | CS8 | P3 | Also the "why" of P3 |
| 119 (rest) | Sometimes Brainstorm then Ponder; weave in a fetch | CS4 | P2 | Brady's 2026-10-07 note (CS1) is the governing principle this falls under |
| 120 | Aluren is symmetric; no Aluren at 3 life or less vs Boros | AL6 | P12 | Pointer from ML and BO |
| 121 | Ponder: keep a known Aluren or Show on top without an enabler; else shuffle | CS5 | P2 | General Ponder rule; line 41 is a special case |
| 122 | Show and Tell with Acererak in hand: Aluren, not Atraxa; vs Boros weigh Swords | ST8 | P10 | Conflict/open with line 14, noted |
| 123 | Stock Up with two or fewer lands: land first | CS12 | P3 | |
| 124 | A keep needs a route to a payoff; exception: the combo in hand | OH1 | P1 | Source of P1 |
| 125 | Veil timing is situational (Brady) | PR9 | P5, P8 | General Veil timing rule; lines 47, 57, 97 are its cases, noted |
| 126 | Waiting a turn is contextual (Brady) | TS6 | P5 | Qualifies lines 47 and 60 |
| 127 | blank | | | |
| 128 | Heading: notes for the opponent decks | vs-ur-cutter.md and vs-boros.md "Their pilot's notes" | | |
| 129 | blank | | | |
| 130 | UR Cutter: | vs-ur-cutter.md | | |
| 131 | Keep one pitch card per Force; never bin Daze/Force | URo1 | P12 | Opponent rule, text unchanged |
| 132 | Force a Veil cast on their own turn | URo2 | P12 | Opponent rule |
| 133 | Their Wasteland targets | URo3 | P12 | Opponent rule |
| 134 | Cantrips before the land drop with fetches | URo4 | P12 | Opponent rule (same idea as TS2 for us) |
| 135 | Do not tap out for Murktide/Cutter; hold Charm or Heat | URo5 | P12 | Opponent rule |
| 136 | Bolt plus Heat on Acererak ends the loop | URo6 | P12 | Opponent rule; pointed to from AL |
| 137 | Their Daze use; dead into open green | URo7 | P12 | Opponent rule |
| 138 | Their Mishra's Bauble | URo8 | P12 | Opponent rule |
| 139 | Against Atraxa: chump, Heat, Charm bounce, burn to 2 | URo9 | P12 | Opponent rule |
| 140 | blank | | | |
| 141 | Boros Aggro: | vs-boros.md | | |
| 142 | Their Wasteland | BOo1 | P12 | Opponent rule |
| 143 | Keep W open for Swords on Acererak | BOo2 | P12 | Opponent rule; pointed to from AL |
| 144 | Under our Aluren they cast MV 3 or less free | BOo3 | P12 | Opponent rule; matches AL6 |
| 145 | Goblin Bombardment timing | BOo4 | P12 | Opponent rule |
| 146 | Second Ajani; menu ability 1/2 | BOo5 | P12 | Opponent rule; menu note pointed to from PE |
| 147 | Voice first, Karakas, Spider-Woman | BOo6 | P12 | Opponent rule |
| 148 | Swords the Show-and-Tell Atraxa; count removal | BOo7 | P12 | Opponent rule |
| 149 | Ascend count; Voice trigger order | BOo8 | P12 | Opponent rule |
| 150 | One pick per command (Bombardment hit The Atropal) | BOo9 and PE1 | P14 | Merged into PE1 as evidence |

## New material that is not from the old playbook

- CS1 and principle P2's "why": Brady's 2026-10-07 note on how to organize the playbook ("Brainstorm, then fetch, then Ponder" falls under "see as many new cards as possible"; sometimes you Brainstorm and like all the cards, so you do not need to shuffle). It is Brady's, so it is included and tagged as his.
- playbook.md's plan paragraph and the one-line "why" under each principle are summaries of the rules mapped above, not new rules.
- Not added: a rule that a Ponder "Do it?" prompt means shuffle. It was suggested for process-and-engine.md but is not in the old playbook.

## Dropped, with reasons

| Old line | Dropped text | Reason |
|---|---|---|
| 32, 45, 54, 62 | Batch headings with records (20W-4L, 7-0, 10-2, 8-0) | History, no rule; the game ids stay on the rules as evidence |
| 91 (part) | "bot agreed with Brady on 3 of 9 answered positions" | Score of the bot, no rule |
| 104 | "bot agreed with Brady on 4 of 14; 2 more declined as moot" | Score, no rule; the moot positions are in TS1 |
| 108 | Pointer to SYNTHESIS.md and to the Acererak section | Navigation only; the technique is AL2 |
| 10 (part) | "The earlier 'about three casts per point of life' was wrong." | Kept only as a note in AL2 that the old estimate is replaced; the wrong number is no longer a rule |

## Conflicts and special cases (all noted in the skill files, none resolved)

1. Line 14 (ST7: Omniscience when Acererak is in hand) vs line 122 (ST8: Aluren, not Atraxa, with Acererak). Brady has not compared Aluren with Omniscience. Open.
2. Lines 47 and 60 (TS7, TS8: wait a turn) vs line 126 (TS6, Brady: waiting is contextual, only if it helps you more). TS7 and TS8 read as cases of TS6; line 96 (ST3: jam with protection) is not overridden.
3. Line 43 (OH7: a creature matters more than cantrips) vs lines 88 and 102 (OH3, OH4: some creature-less dig hands are keeps, Brady). Brady's rulings are on specific hands; OH7 is about one-land hands.
4. Line 41 (CS6) is a special case of line 121 (CS5).
5. Lines 48 and 58 (PR2) are a special case of line 113 (PR1).
6. Line 112 (ST13) is the combo-turn special case of line 99 (ST12).
7. Line 8 (AL4) is the only exception to lines 7 and 114 (AL3, AL1).
8. Lines 47, 57 and 97 (Veil first) and lines 36 and 38 (Veil in response) are the two cases of line 125 (PR9, Veil timing).
9. Line 67 ("never hold Petal on top by accident") and line 100 (CS9: put back Petal on purpose) do not conflict; noted because they read alike.
10. Lines 34, 49, 67, 83 and 119 (fetch and shuffle rules) all fall under Brady's CS1: shuffle only when the known cards are worse than a random one.
11. Lines 29 and 30 (open decisions) are mostly answered by line 114 (AL1) but kept as open, since Brady has not closed them in those words.
