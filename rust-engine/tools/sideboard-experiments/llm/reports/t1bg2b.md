RESULT: LOST
TURNS: 2
SIDEBOARD: Game deck was identical to the base 75 (no boarding). None of the sideboard cards were drawn or cast.
- Mistake: probably none avoidable. Kept 2 lands plus Ancient Tomb, One Ring, Tezzeret x2, Ugin on the draw. Turn 2 I played Tomb and cast Tezzeret, Cruel Captain (Saga plus Tomb) into one open Tropical Island. The opponent then won on their turn 3 (Aluren plus Acererak loop). I had no interaction or hand disruption in hand to stop it, and no window to respond.
- Surprise: the opponent comboed out on their turn 3, which was my pick 23 resolving. After my Tezzeret cast the game ended with no further prompt for me. The opponent used cantrips (Ponder, Brainstorm) and fetch/dual mana on turns 1-2 and had an Aluren loop ready by turn 3.
- Engine/menu: get_card said "not in the card pool" for Tezzeret, Cruel Captain, Ugin, Eye of the Storms, The One Ring, Kozilek's Command and Portable Hole, so I could not read their text. Also, `lg.sh show` failed with a missing prompt.txt until I ran `lg.sh wait` (game start was delayed). The game over line reported 148 auto picks by the opponent's side, which looks like the Aluren loop being run by a macro.
