RESULT: LOST
TURNS: 4
SIDEBOARD: None boarded in (the 60-card game deck was identical to the base main deck, so nothing to evaluate). I could not use the sideboard either: Karn, the Great Creator only offered its +1 in the menu, so there was no wish for Ensnaring Bridge or other sideboard artifacts.
EMRAKUL: n/a
- Game flow: turn 1 Ancient Tomb, Grim Monolith, Trinisphere. On their turn 2 the opponent still cast Show and Tell (3 mana with Trinisphere) and I put in Urza's Workshop; they put in Atraxa, Grand Unifier. I played Karn (killed by Atraxa), then Mishra's Research Desk and Urza's Saga. I never had 7 mana for Ugin. Atraxa hit me 14 over two attacks (Ancient Tomb and Karn damage plus those hits left me on 7) while lifelink took them to 33.
- Mistake, turn 4: I held up Kozilek's Command to cut Atraxa's damage, assuming it had a -X/-X mode. It does not: its modes are Eldrazi Spawn, scry X then draw, exile creature with MV X or less, and exile graveyard cards. Exiling Atraxa needed X=7 (9 mana), so the Command did nothing and I died at exactly 7 life. I should have checked the card text before planning around it (get_card does not contain my own cards).
- Surprise: the opponent held a second Show and Tell and a second Atraxa for several turns without casting them, and discarded Aluren and Acererak to hand size.
- Engine notes: get_card does not know my deck's cards (Karn, Ugin, Trinisphere, Urza's lands, Desk). Karn's sideboard wish -1 was not offered. Desk's choose-and-play-from-exile worked fine.
