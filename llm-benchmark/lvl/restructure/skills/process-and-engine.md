# Process and engine

Use when: before your first command of a game, and whenever a menu, prompt or engine behaviour surprises you.

## Commands (P14)

- **PE1** [P14] One pick per command, always. t18 was lost by sending "cast Petal, pass, pass" in one call: the engine had auto-passed to my turn-4 main 1 and the first pass skipped my land drop. When the last castable action of a turn is done, read the output before anything else; it may already be my next turn. (t18; t26 lost nothing only because main 2 still allowed everything; on the opponent side a scripted Bombardment loop once hit The Atropal instead of the player.)
- **PE2** [P14] To look at a menu use the look command (`play.py next`; `tplay next` in the tools harness). Do not use `pick 0` to look: `pick 0` always passes priority. (a29 slips on turns 3 and 5; repeated as a lesson in t19 to t26)
- **PE3** [P14] Keep the menu discipline: indices shift after every action. (a06 slip, report.md)
- **PE4** [P14] The engine auto-passes my turn 1 when nothing is castable; the next prompt can be turn 3 main 1, not turn 1 main 2. (a41 slip: I passed my turn-3 main 1 by mistake)

## Engine behaviour to know

- **PE5** [P10] Dungeon and room choices are mine (prompts added); see aluren-and-the-loop.md AL1 and AL2 for which to take.
- **PE6** [P10] In Tomb rooms the engine chose my discard and sacrifice for me. (a04)
- The engine can offer Tomb-only casts at 2 or 3 life; check your life yourself: mana-and-life.md ML3.
- The Ponder prompt lists identical names once: cantrips-and-shuffling.md CS7.
- The engine's payment taps all colours, so cast Veil first when the spare matters: protecting-the-combo.md PR7.
- Ajani's menu: "ability 1" is the +2 and "ability 2" is the 0 (opponent note): vs-boros.md BOo5.
