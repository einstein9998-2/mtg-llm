# Turn sequencing

Use when: deciding the order of the land drop, cantrips and committal spells within a turn, or whether to act now or wait a turn. On the play (games 2 and 3 after a loss) re-read TS6 to TS10: sideboarding.md SB4.

Each rule is a default. Read its "Breaks when:" before applying it; if the situation matches, follow the principle instead.

## Land drop (P4)

- **TS1** [P4] Play a land every turn you have one. Brady refused to answer three positions (batch 1 #9, batch 2 #13 and #16) because the bot had skipped its land drop or cast Brainstorm in upkeep earlier in the turn, so the question was already wrong. Fix the earlier mistake first. (Brady, tough spots batch 2)
  Breaks when: you are not going to use the land and Wasteland threatens it: you can sometimes hold it (TS9, Brady 2026-10-08); City of Traitors while you still need land drops (ML5).
- **TS2** [P4] Make the land drop before a committal spell (Show and Tell, Aluren, Atraxa, Acererak, anything that spends a key card), but cast cantrips (Brainstorm, Ponder, Stock Up when it is not the combo turn) before playing the land: they find the land or show what you need, and a land played first reveals information. (Brady, tough spots batch 1; in m01 Brady said the same of the UR pilot twice, vs-ur-cutter.md URo4)
  Breaks when: against live Daze, Veil of Summer can go before the land drop: "veil, then play fetch petal show" (protecting-the-combo.md PR17, Brady m01 comments 2026-10-08). Land first is about as good: it plays around extra copies of Daze, while holding the land obscures information; unclear which is better (Brady, 2026-10-08 03:14Z; judgment note after PR17). Its special cases: a fetch played before Show and Tell to play around double Daze (show-and-tell.md ST5); City of Traitors' mana spent before the land drop (mana-and-life.md ML5); Hedge Maze rather than Boseiju before a Ponder (cantrips-and-shuffling.md CS15).
- **TS3** [P4] Passing priority with a land in hand and mana to spend is "insane" (Brady; the bot did it in a turn-4 Main 1 with Vista in hand). The bot's recurring error was casting committal spells before the land drop, or into Daze and Force. (Brady, tough spots batch 1)
  Breaks when: no known exception (holding a land you will not use is TS9, not a pass with mana to spend).
- **TS9** [P4, P7, P15] Do not hold a land you can use this turn just to protect it from Wasteland: you can still play only one a turn, and every turn they lack a Wasteland is a land kept. With only nonbasics in hand, play the one you can use this turn. (SYNTHESIS2 rule 12 with Brady's qualifier, 2026-10-08; g66 loss, g56)
  Breaks when: you are not going to use that land: sometimes hold it to play around Wasteland. Example (Brady): with an Island out, hold Ancient Tomb until the turn you cast your 3-drop, because being Wastelanded from 3 mana to 1 hurts more than from 3 to 2. Also City of Traitors (ML5), or a basic or fetch to play instead.

Turn 1, how the rules fit together (no new rule; checked after match m01): with no land in play nothing is castable before the land drop except off a Lotus Petal, and a Petal is not spent on a cantrip that could wait (mana-and-life.md ML13, cantrips-and-shuffling.md CS16), so the land (usually the fetch) comes first. Crack it only if this turn uses the mana (CS2, ML9), and fetch the colour the turn needs: white for Prismatic Ending on a turn-1 Dragon's Rage Channeler (vs-ur-cutter.md UR3). With one mana for a cantrip, cast Ponder, not Brainstorm (CS16, CS17).

## Upkeep and main phase (P5)

- **TS4** [P5] Never cast Brainstorm in your own upkeep. In general, wait as long as possible to cast a spell if waiting is free (it costs no mana, tempo or card), because you get more information (the draw, their plays, a land drop you may need). (Brady, tough spots batch 1)
  Breaks when: waiting costs you something (a window against Daze or Force, mana that would go unused, a turn): then cast it now. This is not dawdling (Brady, tough spots batch 1).
- **TS5** [P5] Cast cantrips in your main phase, never in upkeep. (Brady, tough spots batch 1)
  Breaks when: no known exception.
- **TS11** [P5] Stock Up and Ponder are sorceries: passing the main phase with them and open mana throws the turn's mana away. (Brady "Okay", 2026-10-08, conflict 9, on SYNTHESIS2 rule 14; g105 cost about 10 life)
  Breaks when: no known exception.

## Waiting a turn versus jamming (P5, P8, P9)

- **TS6** [P5] Waiting a turn instead of jamming is contextual (Brady): with no clock on you it is reasonable, but only if waiting helps you more than the opponent (you may draw mana, or a second combo card for another push). (Brady, LvL sign-off)
  Breaks when: you hold protection for Show and Tell: jam now (show-and-tell.md ST3); a visible untapped Wasteland faces you with no land in hand: jam more often than not (TS10).
- **TS7** [P5, P8] When I have neither a spare mana nor Veil, a one-turn wait to build a spare mana was right (a41, a42). The cost of waiting was nothing in those games; the opponent mostly cantripped. (observed)
  Breaks when: waiting does not help you more than the opponent, for example a clock on you (TS6); Wasteland pressure (TS10).
- **TS8** [P5, P8] If Daze is live and I cannot keep a spare, wait a turn. Going all-in with lethal on board next turn into two Daze (t18) loses; avoid getting there by not skipping land drops. (observed, t18)
  Breaks when: waiting does not help you more than the opponent (TS6); a visible untapped Wasteland and no land in hand (TS10).
- **TS10** [P5, P7] With a visible untapped Wasteland and no land in hand, jam more often than not: a wait is more likely to lose a mana than gain one. (Brady "Agree with jam more often than not", 2026-10-08, conflict 3; g59 lost by waiting; jams right in g73, g85, g93)
  Breaks when: there is no Wasteland pressure and waiting helps you more (TS6): waits were right in g75, g91 and g101 (two Daze).
