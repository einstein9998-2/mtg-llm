# Process and engine

Use when: before your first command of a game, and whenever a menu, prompt or engine behaviour surprises you.

Each rule is a default. Read its "Breaks when:" before applying it; if the situation matches, follow the principle instead.

## Commands (P14)

- **PE1** [P14] One pick per command, always. t18 was lost by sending "cast Petal, pass, pass" in one call: the engine had auto-passed to my turn-4 main 1 and the first pass skipped my land drop. When the last castable action of a turn is done, read the output before anything else; it may already be my next turn. (t18; t26 lost nothing only because main 2 still allowed everything; on the opponent side a scripted Bombardment loop once hit The Atropal instead of the player.)
  Breaks when: no known exception.
- **PE2** [P14] To look at a menu use the look command (`play.py next`; `tplay next` in the tools harness). Do not use `pick 0` to look: `pick 0` always passes priority. (a29 slips on turns 3 and 5; repeated as a lesson in t19 to t26)
  Breaks when: no known exception.
- **PE3** [P14] Keep the menu discipline: indices shift after every action. (a06 slip, report.md)
  Breaks when: no known exception.
- **PE4** [P14] The engine auto-passes my turn 1 when nothing is castable; the next prompt can be turn 3 main 1, not turn 1 main 2. (a41 slip: I passed my turn-3 main 1 by mistake)
  Breaks when: no known exception.

## Engine behaviour to know

- **PE5** [P10] Dungeon and room choices are mine (prompts added); see aluren-and-the-loop.md AL1 and AL2 for which to take.
  Breaks when: no known exception.
- **PE6** [P10] In Tomb rooms the engine chose my discard and sacrifice for me. (a04)
  Breaks when: no known exception.
- The engine can offer Tomb-only casts at 2 or 3 life; check your life yourself: mana-and-life.md ML3 and EF1 below.
- The Ponder prompt lists identical names once: cantrips-and-shuffling.md CS7.
- The engine's payment taps all colours, so cast Veil first when the spare matters: protecting-the-combo.md PR7.
- Ajani's menu: "ability 1" is the +2 and "ability 2" is the 0 (opponent note): vs-boros.md BOo5.

## Engine and menu facts (verified in reviews, not from Brady)

These come from the second-night reviews (SYNTHESIS2 rules 20, 34, 35, 36 and section 4); EF12 from the interactive match m02. They describe the engine and the harness, not strategy. [P14] unless marked.

- **EF1** [P6, P14] The auto-payer taps Ancient Tomb (2 life) even when a painless payment exists. It killed the bot at 2 life in g65 and at 1 in g72; also g49, g52, g79, g105, g111. Count Tomb's 2 life on every cast while Tomb is untapped.
  Breaks when: no known exception.
- **EF2** [P6, P14] The auto-payer over-taps: casting Aluren tapped a fourth land and left 1 floating (g81, g99, g103, g105, g111), so the spare land for Daze was lost. It also spends a Petal on the battlefield before an untapped land (g50).
  Breaks when: no known exception.
- **EF3** The menu has no land mana abilities, so City of Traitors cannot be tapped in response to its own sacrifice trigger or floated across your land drop (g52, g93): count next turn as City plus untapped lands with no land drop. The engine does have a mana pool (1 floating shown in g81, g99, g111).
  Breaks when: no known exception.
- **EF4** Under Omniscience the menu lists "Cast X" twice: the first entry is the paid cast (it taps lands and sacrifices Petals), the later duplicate is free. g86 lost a Petal; duplicates seen in g44, g48, g58, g78, g87, g89, g91.
  Breaks when: no known exception.
- **EF5** On the opponent's turn read the label before every pick: Force of Will is listed only when a spell is on the stack, so Veil's index moves between windows with no action of yours (g71, decisive).
  Breaks when: no known exception.
- **EF6** If the game ends with no events after your last prompt, the opponent finished it while you had no legal action (the loser sees no events after its last prompt). Rebuild it from life (2 per Tomb tap, 1 per fetch, 3 per Bolt, their attackers) before you report an engine bug. (g77; the same misreport in g69, g83, g97, g109)
  Breaks when: no known exception.
- **EF7** No priority window after blockers is given to a player whose only mana is a fetch or who seems to have nothing castable (g105, g111).
  Breaks when: no known exception.
- **EF8** Legal but misleading: Daze and Force are offered against a spell Veil made uncounterable, with a Daze payment prompt for you (g49, g53, g91); Force's pitch menu is skipped when only one blue card exists (g95).
  Breaks when: no known exception.
- **EF9** Unlabelled prompts: Ponder's shuffle is a bare "Do it?" (g48, g52, g63, g98, g110); Ponder asks for the order before the shuffle (g82); the top-order prompt does not say which card ends on top (g54).
  Breaks when: no known exception.
- **EF10** Mishra's Bauble's reveal is not shown to its controller (g57, g89). The summary-line counters (auto picks, odds, sim) are game-wide, not yours (g54, g55, g60, g61, g66, g89, g95, g109, g111).
  Breaks when: no known exception.
- **EF11** The macro's pick budget can stop in the middle of a loop; restart it (g47, g64, g68, g84, g88, g92, g101, g103, g106, g109).
  Breaks when: no known exception.
- **EF12** [P6, P14] The automatic payment spends floating mana (for example Carpet of Flowers' mana) before lands, and spends it on generic costs too. So Carpet's 3 mana of one colour cannot pay the coloured pip of three separate spells: the first spell eats all of it. Cast first a spell whose generic part can soak up the floating mana, so the next coloured pip is paid from what is left. The m02 log also records that while mana was floating each spell had an extra menu option "pay generic from lands, keep floating mana"; when it is listed, it keeps the pool for the coloured pips. (Tool note from interactive match m02 game 2, 2026-10-08, in BRADY-RULINGS2.md; the menu option from the m02 log header)
  Breaks when: no known exception.
