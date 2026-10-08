# Mana and life

Use when: planning a turn's mana, paying life for Ancient Tomb or a fetch, choosing which land to play or fetch, casting Lotus Petal, or the opponent has Wasteland.

Each rule is a default. Read its "Breaks when:" before applying it; if the situation matches, follow the principle instead.

## Counting (P6)

- **ML1** [P6] Count mana before planning. Show and Tell is {2}{U}; Aluren is {2}{G}{G}; Ancient Tomb counts as 2 and a Petal on the battlefield as one coloured mana. (Brady, LvL sign-off)
  Breaks when: no known exception.
- **ML2** [P6, P8] Cast Veil only if the combo spell is still castable this turn (Veil + Show and Tell = 4 mana, Veil + Aluren = 5 including GG). (Brady, LvL sign-off)
  Breaks when: Veil is cast in response to their spell to save a permanent already in play, such as a Prismari Charm aimed at Aluren or Atraxa (PR8; a06, a32).
- **ML3** [P6] Omniscience costs ten mana ({7}{U}{U}{U}). Count it before planning around a hardcast: two Ancient Tombs, six other lands and a Petal are needed; each Tomb tap costs 2 life, and the engine can still offer Tomb-only casts at 2 or 3 life (seen in LvL games g8 and g19), so check your life yourself (engine-oddities.md item 9, withdrawn as a bug). Show and Tell is the normal way to get it out. (observed)
  Breaks when: no known exception. Engine: the second night found the auto-payer taps Ancient Tomb even when a painless payment exists, so item 9 may need reopening (process-and-engine.md EF1).

## Life is mana (P6, P12)

- **ML4** [P6, P12] Before paying Tomb or fetch life, check your life afterwards against their instant reach (Goblin Bombardment = their creature count; a second Ajani flip; free Phlage = 3 under your Aluren; burn to 2 or less turns off your Tombs and Force of Will). (Brady, LvL sign-off)
  Breaks when: no known exception.
- **ML15** [P6, P15] Against a fast clock, waiting on Ancient Tomb can be useful: its 2 damage is real. Brady: "hard to tell". (Brady, 2026-10-08, conflict 15; g98: Tomb plus Stock Up versus Ponder plus a tapped Hedge Maze)
  Breaks when: no known exception (Brady: hard to tell).
- See also aluren-and-the-loop.md AL6: do not cast Aluren at 3 life or less against Boros.

## Lotus Petal (P6)

- **ML13** [P6] Lotus Petal is next turn's combo mana only if the combo is castable with it. Do not spend it on a cantrip that could wait a turn (g76, g83, losses); with no green land it is not Aluren mana, so spend it as the Daze spare or on Veil (g107). On turn 1 that includes Brainstorm: "The petal will only ever be used once anyway" (cantrips-and-shuffling.md CS16; m01 game 1 spent it on a turn-1 Brainstorm). (Brady "okay", 2026-10-08, conflict 10, on SYNTHESIS2 rule 18; Brady, m01 comments, 2026-10-08)
  Breaks when: against Spider-Woman or Thalia, cast Petals early (ML14); you are out of cantrips, have plenty of lands and are not spending mana this turn: Brady's most common Acererak hardcast turns a Petal and 2 life into a scry 1 (AL9).
- **ML14** [P6, P12] Usually cast Lotus Petals as early as possible to play around Spider-Woman (Boros) or Thalia. Against Boros that means even before Spider-Woman is seen (g90, g104 missed it; g44, g108 did it). (Brady, 2026-10-08, on SYNTHESIS2 rule 19)
  Breaks when: you have a lot of lands (Brady, 2026-10-08).

## City of Traitors (P4, P6)

- **ML5** [P4, P6] Spend City of Traitors' mana before the land drop (Brady, LvL sign-off). City is sacrificed as soon as I play another land (a31): play it as the last land, or not at all when I still need land drops.
  Breaks when: no known exception. Engine: City's mana cannot be floated across your land drop here (process-and-engine.md EF3).

## Wasteland (P7)

- **ML6** [P7] Wasteland takes Ancient Tomb and Tropical Island: prefer basics or fetches when I am short on lands, and keep a land drop for Hedge Maze. (observed, UR Cutter)
  Breaks when: no known exception.
- **ML7** [P7] Do not play a nonbasic into an untapped Wasteland just to use one more mana (t18: my only land was Wastelanded on turn 3). Prefer fetches and basics first. (t18)
  Breaks when: that mana casts your combo this turn and Wasteland is coming anyway (g73, g93); Brady agrees with jamming more often than not there (turn-sequencing.md TS10).
- **ML8** [P7] The opponent can hold several Wastelands (three in one game) and uses them in a burst at the end of my turn. When my only land drops are Ancient Tomb and Tropical Island, I cannot avoid them, but I can fetch basics with Misty, Strand and Delta and keep nonbasics for later. A second green source matters: with Forest as the only basic G, losing Tropical made Aluren uncastable for the rest of the game. (a36)
  Breaks when: this turn needs a colour only a dual gives: with Prismatic Ending in hand against a turn-1 Dragon's Rage Channeler, fetch Tundra or Savannah for the white (vs-ur-cutter.md UR3, Brady m01 comments 2026-10-08; the m01 game and its replay both fetched the basic Island, the replay citing Wasteland).
- **ML9** [P5, P7] Keeping Misty uncracked until the turn I need the mana costs nothing and gives Wasteland a decoy. (a41, a42)
  Breaks when: the turn you need the mana: crack it then.
- **ML10** [P7] A basic fetched at their end step avoids Wasteland and gives me the mana on my own turn. (t22, t25)
  Breaks when: no known exception.
- **ML11** [P7, P8] Opponent Wasteland: cast the key spell first with the lands that Wasteland can hit and hold the Petal back for Veil. (a32)
  Breaks when: no known exception.
- **ML12** [P7] Do not channel Boseiju on a dual: they fetch an untapped replacement and the dead land is Murktide delve fuel. (Brady, 2026-10-08, conflict 4; g45, g81)
  Breaks when: no known exception.
- Holding a land you will not use, and holding Ancient Tomb until your 3-drop: turn-sequencing.md TS9.

Judgment, not a rule (Brady: "Depends", 2026-10-08, conflict 12): once Wasteland has been seen, fetching Hedge Maze at their end step (CS13) and fetching basics or holding the fetch (ML8, ML9) pull opposite ways. g99 and g83 lost a land that way; g51, g63, g85 and g101 fetched Maze at the end step and won. Weigh it each time.

## Elsewhere

- Paying for Daze (only an untapped land or a Petal already on the battlefield): protecting-the-combo.md PR1.
- Which land to take from Atraxa (Tropical Island or Forest): show-and-tell.md ST15.
- Which land to fetch for Prismatic Ending (white: Tundra or Savannah only): vs-ur-cutter.md UR3. Veil before the land drop and the Petal against Daze: protecting-the-combo.md PR17.
- Spider-Woman makes Petals enter tapped: vs-boros.md BO3.
- City of Traitors over a basic against Reanimator: mirror-and-other-matchups.md MM4.
