# Turn sequencing

Use when: deciding the order of the land drop, cantrips and committal spells within a turn, or whether to act now or wait a turn.

## Land drop (P4)

- **TS1** [P4] Play a land every turn you have one. Brady refused to answer three positions (batch 1 #9, batch 2 #13 and #16) because the bot had skipped its land drop or cast Brainstorm in upkeep earlier in the turn, so the question was already wrong. Fix the earlier mistake first. (Brady, tough spots batch 2)
- **TS2** [P4] Make the land drop before a committal spell (Show and Tell, Aluren, Atraxa, Acererak, anything that spends a key card), but cast cantrips (Brainstorm, Ponder, Stock Up when it is not the combo turn) before playing the land: they find the land or show what you need, and a land played first reveals information. (Brady, tough spots batch 1)
- **TS3** [P4] Passing priority with a land in hand and mana to spend is "insane" (Brady; the bot did it in a turn-4 Main 1 with Vista in hand). The bot's recurring error was casting committal spells before the land drop, or into Daze and Force. (Brady, tough spots batch 1)
- Special cases of TS2, kept in their own files: a fetch played before Show and Tell to play around double Daze (show-and-tell.md ST5); City of Traitors' mana is spent before the land drop and City is played last (mana-and-life.md ML5); Hedge Maze rather than Boseiju before a Ponder (cantrips-and-shuffling.md CS15).

## Upkeep and main phase (P5)

- **TS4** [P5] Never cast Brainstorm in your own upkeep. In general, wait as long as possible to cast a spell if waiting is free (it costs no mana, tempo or card), because you get more information (the draw, their plays, a land drop you may need). This does not mean dawdling: when waiting costs you something (a window against Daze or Force, mana that would go unused, a turn), cast it. (Brady, tough spots batch 1)
- **TS5** [P5] Cast cantrips in your main phase, never in upkeep. (Brady, tough spots batch 1)

## Waiting a turn versus jamming (P5, P8, P9)

- **TS6** [P5] Waiting a turn instead of jamming is contextual (Brady): with no clock on you it is reasonable, but only if waiting helps you more than the opponent (you may draw mana, or a second combo card for another push). (Brady, LvL sign-off)
- **TS7** [P5, P8] When I have neither a spare mana nor Veil, a one-turn wait to build a spare mana was right (a41, a42). The cost of waiting was nothing in those games; the opponent mostly cantripped. (observed)
- **TS8** [P5, P8] If Daze is live and I cannot keep a spare, wait a turn. Going all-in with lethal on board next turn into two Daze (t18) loses; avoid getting there by not skipping land drops. (observed, t18)

Conflict: TS7 and TS8 are observed rules that say "wait"; TS6 is Brady's later qualifier that a wait is right only when it helps you more than the opponent. Read TS7 and TS8 as cases where that held (no clock, opponent cantripping). TS6 also does not override show-and-tell.md ST3: when you hold protection, jam Show and Tell now.
