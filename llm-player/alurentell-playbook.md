# Alurentell playbook (from Brady's notes plus what the games showed)

Source of the strategy points is Brady (2026-10-01). Lines marked "observed" come from my games a01 to a06.

## Acererak and Tomb of Annihilation

- Default: do **not** complete Tomb of Annihilation. Acererak then bounces to hand every time, and Aluren recasts him for free, so the loop keeps producing rooms (Goblins, Treasures, scry, cards, Dark Pool drain, life).
- Exception (Brady): if the opponent has a card that stops the Aluren loop, for example Sphere of Resistance, Damping Sphere, Trinisphere or Disruptor Flute, go through Tomb of Annihilation on purpose. Once the dungeon is completed Acererak stays on the battlefield by himself, and completing it also gives a 4/4 (The Atropal). Check the opponent's battlefield for tax and lock pieces before starting the loop and again whenever one appears.
- Tomb rooms hurt both players (Trapped Entry: each loses 1; Veils of Fear and Sandfall Cell: lose 2 or discard/sacrifice). Observed in a04: the opponent discarded Force of Will and sacrificed a creature to avoid the loss, and the engine chose my discard and sacrifice for me.
- Dungeon and room choices are now mine (prompts added). In a06 I took Lost Mine every time: Cave Entrance (scry), Goblin Lair (blocker), Dark Pool (drain 1) is the three-cast cycle that drains once per lap, so about three Acererak casts per point of life. Mad Mage and Tomb were never needed.

## Show and Tell and Omniscience

- Put Omniscience in with Show and Tell when I have Atraxa, Aluren or Acererak in hand: everything is free afterwards, and the put-in cannot be countered. Cast the free Atraxa second (observed a04: it blocked and gained 7 life).
- A free Force of Will through Omniscience costs nothing (no pitch, no life), but countering an artifact puts it in the opponent's graveyard (fourth card type, Dragon's Rage Channeler delirium). Count their graveyard types before countering artifacts.
- Hand size: loops draw many cards. Dump spare cards as free spells, or discard lands and duplicates at cleanup.

## Against UR Cutter (Forge AI)

- They hold Force of Will and Daze for my first Stock Up (a01, a04, a05). Veil of Summer first if I have it; otherwise a Stock Up into Force of Will plus Daze is a 2-for-1 for me.
- Do not tap out for Aluren when Daze may be in hand and I have no spare mana (a05).
- Wasteland takes Ancient Tomb and Tropical Island: prefer basics or fetches when I am short on lands, and keep a land drop for Hedge Maze.
- Omniscience costs ten mana ({7}{U}{U}{U}). Count it before planning around a hardcast: two Ancient Tombs, six other lands and a Petal are needed; each Tomb tap costs 2 life, so at 4 life or less the engine will not offer it (engine-oddities.md item 9, withdrawn as a bug). Show and Tell is the normal way to get it out.
- Prismari Charm (bounce) and Lightning Bolt are the cards that have hit Atraxa/me: Veil of Summer in response to the Charm worked in a06 and drew a card (they had cast a blue spell).
- Keep the menu discipline: indices shift after every action (a06 slip, report.md).

## Open decisions to test

- Whether to hold Acererak until a lock piece shows up or loop immediately.
- Whether Brady wants Tomb completed when the opponent is low and Dark Pool is not available.

## Lessons from the counted batch (a07 to a30, 20W-4L)

- Fetch cracks shuffle the library, so an unknown top card is just as likely to be replaced by a better one; the cost is 1 life and the real harm is cracking after a Ponder or Brainstorm that put a good card on top (a30 and a33 both lost an Atraxa to an early crack, with no way to know). Crack when I need the land or want to shuffle away a known bad top, not by reflex.
- A seven with no blue source and no payoff creature (a30: Tomb, City, Forest, Ponder, Show and Tell, two Omniscience) is a close mulligan; the keep would have won there, but the decision was reasonable. Hands that need two more cards (a creature plus an enabler) are the ones that lose to Force of Will, Daze and Wasteland.
- Two cards the opponent pitches to Force of Will on my Stock Up are worth the Stock Up. Veil of Summer cast in response to their Force of Will fizzles it and draws a card (a30).
- Process: use `play.py next` to look at a menu; `pick 0` always passes priority (a29 slips on turns 3 and 5).
- Veil of Summer also answers their Prismari Charm: in response, it gives my permanents hexproof from blue, so a Charm aimed at Aluren or Atraxa fizzles (a32), and it draws a card because their Charm is a blue spell. Keep a green source (Petal) open for it when the opponent has blue mana up.
- City of Traitors is sacrificed as soon as I play another land (a31). Play it as the last land, or not at all when I still need land drops.
- Opponent Wasteland: cast the key spell first with the lands that Wasteland can hit and hold the Petal back for Veil (a32).
- Ponder: count the cards. The prompt lists identical names once, so a three-card look can show two entries (a33: Aluren, Show and Tell, Show and Tell). If the list has fewer than three entries, assume the missing card is a duplicate of one shown and check it before keeping the order. With no creature among the three cards and a creature-dependent hand, shuffle.
- Wasteland (a36): the opponent can hold several (three in one game) and uses them in a burst at the end of my turn. When my only land drops are Ancient Tomb and Tropical Island, I cannot avoid them, but I can fetch basics with Misty, Strand and Delta and keep nonbasics for later. A second green source matters: with Forest as the only basic G, losing Tropical made Aluren uncastable for the rest of the game.
- One-land keeps (a36) with Petal and three cantrips on the draw were not punished early but never found a payoff; a creature (Acererak or Atraxa) in the opening seven matters more than extra cantrips.

## Lessons from a37 to a43 (7-0 on the play)

- Spare mana beats Daze, Veil beats Force of Will: a43 (opp held Force of Will and Daze) was won by casting Veil of Summer first (G), then Show and Tell with the other three mana; a37, a40, a41 won by casting Aluren with a Petal or second Ancient Tomb spare. When I have neither, a one-turn wait to build a spare mana was right (a41, a42). The cost of waiting was nothing in those games; the opponent mostly cantripped.
- Look at what the opponent has untapped: with only a Wasteland out, Daze is impossible (it returns an Island, and Wasteland is not one). a39 turn-2 Show and Tell exploited that.
- Show and Tell for Omniscience, then free Brainstorm, Ponder and Stock Up, finds the creature (a43). Cast Brainstorm before Ponder so Ponder's shuffle clears the put-backs.
- Keep one extra blue card as Force of Will pitch fodder when discarding to hand size (a40).
- Keeping Misty uncracked until the turn I need the mana costs nothing and gives Wasteland a decoy (a41, a42).
- Engine auto-passes my turn-1 when nothing is castable; the next prompt can be turn 3 main 1, not turn 1 main 2 (a41 slip: I passed my turn-3 main 1 by mistake).

## Lessons from the with-tools batch t07 to t18 (10-2)

- One pick per command, always. t18 was lost by sending "cast Petal, pass, pass" in one call: the engine had auto-passed to my turn-4 main 1 and the first pass skipped my land drop. When the last castable action of a turn is done, read the output before anything else; it may already be my next turn.
- Veil of Summer cast in response to their Force of Will on my spell makes it resolve and draws a card (t15), and cast first it beats Force of Will and Daze alike (t11). Hold the green source for it when I can pay it; the engine's payment taps all colours, so cast Veil first when the spare matters.
- When the opponent has only Wasteland (no Island), Daze is impossible: that was the window for an Aluren with no spare in t13, t16.
- Do not play a nonbasic into an untapped Wasteland just to use one more mana (t18: my only land was Wastelanded on turn 3). Prefer fetches and basics first.
- If Daze is live and I cannot keep a spare, wait a turn. Going all-in with lethal on board next turn into two Daze (t18) loses; avoid getting there by not skipping land drops.

## Lessons from t19 to t26 (tools batch, 8-0)

- When the opponent taps out on its own turn its lands stay tapped through my turn; only Force of Will and Daze (free) remain. t22, t23, t25 and t26 were won in exactly that window: Veil first or a spare mana plus my own Force of Will as backup.
- A basic fetched at their end step avoids Wasteland and gives me the mana on my own turn; Hedge Maze fetched at their end step enters tapped, untaps on my turn and still surveils (t22, t25).
- Their Force of Will on Aluren can be answered with my Force of Will when they are down to one card (t26); pitch the card that is dead without Show and Tell (Atraxa), keep Show and Tell as the second route.
- Brainstorm put-backs are drawn next turn unless a crack shuffles first; hold the fetch uncracked until their end step so it shuffles before my draw (t25), and never hold Petal/Stock Up on top by accident (t25).
- Under Omniscience, cast the redundant spell first as bait (Aluren, Stock Up); opponents here answered Goblin tokens with Unholy Heat (t22, t24, t26) instead of the loop.
- Process: do not use `pick 0` to look at a menu (it passes); use `tplay next`. And one pick per command, always (t26 lost nothing only because main 2 still allowed everything).

## Tips from Brady (2026-10-06, while planning sideboards; not from a played game)

- Show and Tell versus Aluren: decide by how much mana I have and which other combo pieces are in my hand. What the opponent is doing barely matters.
- Against Boros after a mulligan: Stock Up should put one card back, because I have to win the turn after Stock Up or their creatures kill me. In a blind three-land hand Stock Up is solid; the same card is just good, not a must, against a fast clock.
- Mirror: Aluren and Show and Tell are symmetric, so whoever blinks first loses (you are down the card and the mana), and whoever resolves more Stock Ups usually wins. If an Aluren is in play, somebody wins at instant speed almost immediately, so check whether the OPPONENT's Aluren lets me win first: example, the opponent resolved Aluren, cast Acererak in my upkeep so I would not draw; I cast my own Acererak in response, they Forced, I Forced back and won.
- Prismatic Ending does nothing against an Aluren in play; keep it out of the mirror.
- Orim's Chant (kicked): against the blue interactive decks (UR, UB, UW) it is a turn-of-the-combo tool on my own turn. Against Reanimator, a non-blue combo deck faster than I am, it is a time walk cast on THEIR turn: in their upkeep, or in response to Dark Ritual.
- Against Reanimator keep one Show and Tell for the high-resource line (Show in Atraxa, Chant them so they cannot answer, win next turn); otherwise Show is risky because they put in something big. Keep City of Traitors over a basic: two extra mana means one land fewer for the combo.

## Rules from Brady's tough-spot review (2026-10-06, first 10 flagged bot decisions; his words from the answer page)

- Never cast Brainstorm in your own upkeep. In general, wait as long as possible to cast a spell if waiting is free (it costs no mana, tempo or card), because you get more information (the draw, their plays, a land drop you may need). This does not mean dawdling: when waiting costs you something (a window against Daze or Force, mana that would go unused, a turn), cast it.
- Make the land drop before a committal spell (Show and Tell, Aluren, Atraxa, Acererak, anything that spends a key card), but cast cantrips (Brainstorm, Ponder, Stock Up when it is not the combo turn) before playing the land: they find the land or show what you need, and a land played first reveals information. Cast cantrips in your main phase, never in upkeep. Passing priority with a land in hand and mana to spend is "insane" (bot did it in a turn-4 Main 1 with Vista in hand), and the bot's recurring error was casting committal spells before the land drop. Fetch away the cards you put back with Brainstorm if they are bad.
- Do not cast a raw Show and Tell without a payoff or protection in hand when you can still dig. They may be holding Force of Will for it and get to surveil and turn on delirium. Better sequence (turn 4, Boseiju in hand): play the land, cast Veil of Summer first; if it resolves, Show and Tell for Atraxa; if it does not, Stock Up. If they Daze, pay 1 and draw a card.
- With no Show and Tell payoff on the board or in hand, play the card-selection land first (Hedge Maze) rather than Veil: you need to find something to put in.
- Brainstorm put-backs: Mishra's Bauble lets the opponent see the top card, so put back the card whose identity helps them least. Not Veil of Summer, not Show and Tell. (Brady chose Ponder.)
- Atraxa reveal: the card you pick first is usually the land you want untapped and green to cast Aluren next turn (Tropical Island); Forest is better when a Wasteland could hit a dual or City of Traitors might die.
- Mulligan: a 7 with no cantrips and no blue card for Force of Will, slow even if it finds Acererak, is a mulligan. A 7 with two Force of Will (second pitches to the first), Hedge Maze surveil and Ponder / Stock Up to dig toward a fast combo is a keep.
- Opponent at 2 cards in hand and I am about to reach 4 mana: Brainstorm first, then Veil plus Show and Tell wins.

Source and numbers: tough-spots/batch1 (bot agreed with Brady on 3 of 9 answered positions; the bot's recurring error was casting spells before the land drop or into Daze/Force).
