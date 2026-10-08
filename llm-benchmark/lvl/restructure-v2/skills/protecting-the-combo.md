# Protecting the combo

Use when: the opponent may have Force of Will, Daze or Prismari Charm as you cast a combo spell, or you are deciding how to use your own Force of Will, Veil of Summer or Orim's Chant.

Each rule is a default. Read its "Breaks when:" before applying it; if the situation matches, follow the principle instead.

## Can they pay? (P8, P12)

- **PR1** [P8, P12] Daze is live whenever they control any Island-typed land, tapped or untapped. Only an untapped land or a Petal already on the battlefield pays for it; a Petal in hand cannot be cast in response. (Brady, LvL sign-off)
  Breaks when: no known exception.
- **PR2** [P8, P12] Look at what the opponent has: with only a Wasteland out, Daze is impossible (it returns an Island, and Wasteland is not one). a39 turn-2 Show and Tell exploited that, and it was the window for an Aluren with no spare in t13 and t16. (observed; the case of PR1 where they control no Island-typed land)
  Breaks when: no known exception.
- **PR3** [P12] When the opponent taps out on its own turn its lands stay tapped through my turn; only Force of Will and Daze (free) remain. t22, t23, t25 and t26 were won in exactly that window: Veil first or a spare mana plus my own Force of Will as backup. (observed)
  Breaks when: no known exception.

## Beating Daze and Force of Will (P8)

- **PR4** [P8] Do not tap out for Aluren when Daze may be in hand and I have no spare mana. (a05)
  Breaks when: they control no Island-typed land, so Daze is impossible (PR2; t13, t16).
- **PR5** [P8] Spare mana beats Daze, Veil beats Force of Will: a43 (opponent held Force of Will and Daze) was won by casting Veil of Summer first (G), then Show and Tell with the other three mana; a37, a40, a41 won by casting Aluren with a Petal or second Ancient Tomb spare. (observed) When you have neither, see turn-sequencing.md TS6 to TS10.
  Breaks when: no known exception.
- **PR6** [P8] Two cards the opponent pitches to Force of Will on my Stock Up are worth the Stock Up. (a30) The UR Cutter version (they hold Force and Daze for my first Stock Up) is vs-ur-cutter.md UR1.
  Breaks when: no known exception.
- **PR7** [P8] Veil of Summer cast in response to their Force of Will on my spell fizzles the Force, makes my spell resolve and draws a card (a30, t15); cast first, it beats Force of Will and Daze alike (t11). Hold the green source for it when I can pay it; the engine's payment taps all colours, so cast Veil first when the spare matters. (observed; two lessons merged)
  Breaks when: no Daze payment would be left after Veil in response: Veil itself can be Dazed (PR15, g71).
- **PR8** [P8, P12] Veil of Summer also answers their Prismari Charm: in response, it gives my permanents hexproof from blue, so a Charm aimed at Aluren or Atraxa fizzles (a06, a32), and it draws a card because their Charm is a blue spell. Keep a green source (Petal) open for it when the opponent has blue mana up. (observed; two lessons merged)
  Breaks when: no known exception.
- **PR9** [P5, P8, P15] Veil timing is situational: play it first to play around Daze; otherwise play it later to surprise the opponent and draw a card. Veil first is right probably 95%+ of the time. (Brady, LvL sign-off; Brady, 2026-10-08, conflict 5; m01 game 1 turn 3, where Brady wanted Veil first: PR17)
  Breaks when: the card draw is important: they do not always have Daze (Brady, 2026-10-08).
- **PR17** [P8, P15] With Daze live and Show and Tell to cast, cast Veil of Summer first, off the lands already in play, then make the land drop (crack the fetch), cast Lotus Petal, then Show and Tell. Veil first plays around Daze, and casting it before the land and the Petal also disguises how much mana you have if they do have Daze. Brady: "we can veil to play around daze. so veil, then play fetch petal show. also disguises how much mana we have if they do have daze". (Brady, m01 comments, 2026-10-08; m01 game 1 turn 3: fetch, Petal, Show and Tell, with Veil cast only after Show and Tell resolved; the m01 replay player cast Veil first but after the land drop and the Petal)
  Breaks when: the card draw is important: they do not always have Daze, so you do not always do it, but it is probably 95%+ (PR9, Brady 2026-10-08, ruling 5).
- **PR15** [P6, P8] Veil can itself be Dazed: with your spell on the stack and Daze live, keep the untapped land that pays for it, and cast Veil in response to their Force only if a Daze payment is still left. (SYNTHESIS2 rule 26; Brady "Yes, Veil first plays around Daze", 2026-10-08, conflict 5; g71 loss: Force, Atraxa and Veil lost)
  Breaks when: no known exception.
- Cast Veil only if the combo spell is still castable this turn: mana-and-life.md ML2. Hold the Petal back for Veil against Wasteland: ML11.

PR9 is the general Veil timing rule. PR5, PR7's "cast Veil first when the spare matters", PR17 (Veil before the land drop and the Petal) and show-and-tell.md ST4 are its "first, to play around Daze" case (ST4 plays the land before Veil: open question in CHANGES.md, m01 open question 2); PR7's in-response line and PR8 are its "later, to surprise and draw" case.

## Your own Force of Will (P11, P3)

- **PR10** [P11] Spend Force of Will on what breaks the loop (removal on Acererak while his enter trigger is on the stack), not on Swords for a Show-and-Tell Atraxa and not on face burn when you are tapped out and they have Daze. (Brady, LvL sign-off)
  Breaks when: Voice of Victory is on the stack and you have a spare blue pitch (vs-boros.md BO4, Brady 2026-10-08); for Swords on a Show-and-Tell Atraxa when the loop turn is already safe, see the judgment note below.
- **PR11** [P11, P3] Their Force of Will on Aluren can be answered with my Force of Will when they are down to one card (t26); pitch the card that is dead without Show and Tell (Atraxa), keep Show and Tell as the second route. (t26)
  Breaks when: no known exception.
- **PR12** [P11, P12] A free Force of Will through Omniscience costs nothing (no pitch, no life), but countering an artifact puts it in the opponent's graveyard (fourth card type, Dragon's Rage Channeler delirium). Count their graveyard types before countering artifacts. (observed, a01 to a06)
  Breaks when: no known exception.
- **PR13** [P3] Keep one extra blue card as Force of Will pitch fodder when discarding to hand size. (a40)
  Breaks when: no known exception. PR16 extends it beyond cleanup.
- **PR16** [P3, P8] Only blue cards pitch to Force of Will (Acererak is black, Aluren and Veil green, Petal colourless). Count your pitches before you rely on Force; when you Brainstorm or pick with Force in hand, keep one blue card per Force. No "Cast Force of Will" in the menu means no backup. (Brady "Okay", 2026-10-08, conflict 11, on SYNTHESIS2 rule 6; losses g45, g95; g111 kept Omniscience as the pitch and won the Force war)
  Breaks when: no known exception.
- Pitching your only payoff to resolve Show and Tell: show-and-tell.md ST16.

Judgment, not a rule (Brady: "Depends but probably doesn't matter that much", 2026-10-08, conflict 14): Forcing their Swords on a Show-and-Tell Atraxa when the loop turn is already safe (g100, one game). PR10 stays the default.

## Orim's Chant (P13)

- **PR14** [P13] Orim's Chant (kicked): against the blue interactive decks (UR, UB, UW) it is a turn-of-the-combo tool on my own turn. Against Reanimator, a non-blue combo deck faster than I am, it is a time walk cast on THEIR turn: in their upkeep, or in response to Dark Ritual. (Brady, tips 2026-10-06, while planning sideboards)
  Breaks when: no known exception.
- Orim's Chant in the high-resource line against Reanimator: mirror-and-other-matchups.md MM3. Orim's Chant is a sideboard card: sideboarding.md.
