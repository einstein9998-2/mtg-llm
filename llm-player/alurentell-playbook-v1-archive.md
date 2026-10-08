# Alurentell playbook (from Brady's notes plus what the games showed)

Source of the strategy points is Brady (2026-10-01). Lines marked "observed" come from my games a01 to a06.

## Acererak and Tomb of Annihilation

- Default: do **not** complete Tomb of Annihilation. Acererak then bounces to hand every time, and Aluren recasts him for free, so the loop keeps producing rooms (Goblins, Treasures, scry, cards, Dark Pool drain, life).
- Exception (Brady): if the opponent has a card that stops the Aluren loop, for example Sphere of Resistance, Damping Sphere, Trinisphere or Disruptor Flute, go through Tomb of Annihilation on purpose. Once the dungeon is completed Acererak stays on the battlefield by himself, and completing it also gives a 4/4 (The Atropal). Check the opponent's battlefield for tax and lock pieces before starting the loop and again whenever one appears.
- Tomb rooms hurt both players (Trapped Entry: each loses 1; Veils of Fear and Sandfall Cell: lose 2 or discard/sacrifice). Observed in a04: the opponent discarded Force of Will and sacrificed a creature to avoid the loss, and the engine chose my discard and sacrifice for me.
- Dungeon and room choices are now mine (prompts added). In a06 I took Lost Mine every time: Cave Entrance (scry), Goblin Lair (blocker), Dark Pool (drain 1) is the room that drains. With Aluren in play (Brady, 2026-10-07): cast Acererak at instant speed and leave every room trigger on the stack except the Dark Pool drain, so no cards are drawn and the library is not touched. Only under Omniscience must the triggers resolve (you will eventually draw Aluren); then a lap is four casts, one drain and one draw, so check library size against their life. The earlier "about three casts per point of life" was wrong. Mad Mage and Tomb were never needed.

## Show and Tell and Omniscience

- Put Omniscience in with Show and Tell when I have Atraxa, Aluren or Acererak in hand: everything is free afterwards, and the put-in cannot be countered. Cast the free Atraxa second (observed a04: it blocked and gained 7 life).
- A free Force of Will through Omniscience costs nothing (no pitch, no life), but countering an artifact puts it in the opponent's graveyard (fourth card type, Dragon's Rage Channeler delirium). Count their graveyard types before countering artifacts.
- Hand size: loops draw many cards. Dump spare cards as free spells, or discard lands and duplicates at cleanup.

## Against UR Cutter (Forge AI)

- They hold Force of Will and Daze for my first Stock Up (a01, a04, a05). Veil of Summer first if I have it; otherwise a Stock Up into Force of Will plus Daze is a 2-for-1 for me.
- Do not tap out for Aluren when Daze may be in hand and I have no spare mana (a05).
- Wasteland takes Ancient Tomb and Tropical Island: prefer basics or fetches when I am short on lands, and keep a land drop for Hedge Maze.
- Omniscience costs ten mana ({7}{U}{U}{U}). Count it before planning around a hardcast: two Ancient Tombs, six other lands and a Petal are needed; each Tomb tap costs 2 life, and the engine can still offer Tomb-only casts at 2 or 3 life (seen in LvL games g8 and g19), so check your life yourself (engine-oddities.md item 9, withdrawn as a bug). Show and Tell is the normal way to get it out.
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

## Rules from Brady's tough-spot review, batch 2 (2026-10-07)

- Play a land every turn you have one. Brady refused to answer three positions (batch 1 #9, batch 2 #13 and #16) because the bot had skipped its land drop or cast Brainstorm in upkeep earlier in the turn, so the question was already wrong. Fix the earlier mistake first.
- Show and Tell: when you hold protection (Force of Will, Veil of Summer), jam it now; it will not get better by waiting. Never cast Acererak instead of Show and Tell: Acererak is "plan F" because it achieves so little, and a resolved Show and Tell wins the game.
- Jam line with Veil: land (Ancient Tomb), Veil of Summer, then Show and Tell for Atraxa. If Veil resolves, you probably win. If it is Dazed, pay 1 and draw. If it is forced, try again next turn.
- Land drop vs Atraxa: playing a fetch before Show and Tell plays around double Daze, which is worth slightly more than holding the land drop for a possible Atraxa trigger, because a resolved Show and Tell usually wins anyway.
- Atraxa reveal: do not take 0 cards. The only reason to take fewer than allowed is to put back cards you would discard to hand size anyway. Take the green source and Force of Will, then Aluren, and win.
- Brainstorm put-back with plenty of lands (4 across battlefield and hand): put back Lotus Petal, not a land.
- Stock Up pick (look at 5, take 2): with enough lands in hand and no Show and Tell or Acererak in hand to complete a pair, take the strong cantrip (the other Stock Up) first, then Atraxa or Force, not another land.
- Mulligan: Two surveil lands toward Acererak or Atraxa or more cantrips, with live Force, is a keep (close). Stock Up on turn 1 that likely finds a permanent mana source is a keep (close). One land with Ponder is a mulligan unless you know the opponent is Storm or Reanimator (then keep). No permanent coloured source or no cantrips with one land is a mulligan.

Source: tough-spots/batch2 (bot agreed with Brady on 4 of 14 answered positions; 2 more declined as moot).

## Rules from LvL reviews (LLM vs LLM overnight run, 40 games; Brady signed off 2026-10-07 13:06Z, with his corrections)

Source: llm-benchmark/lvl/SYNTHESIS.md (games and evidence per rule). The Lost Mine technique is in the Acererak section above.

- Count mana before planning. Show and Tell is {2}{U}; Aluren is {2}{G}{G}; Ancient Tomb counts as 2 and a Petal on the battlefield as one coloured mana. Cast Veil only if the combo spell is still castable this turn (Veil + Show and Tell = 4 mana, Veil + Aluren = 5 including GG). Spend City of Traitors' mana before the land drop.
- Life is mana: before paying Tomb or fetch life, check your life afterwards against their instant reach (Goblin Bombardment = their creature count; a second Ajani flip; free Phlage = 3 under your Aluren; burn to 2 or less turns off your Tombs and Force of Will).
- On the combo turn, take every card Atraxa offers, a spare Acererak first. When discarding to hand size with Aluren in hand, keep Acererak and drop a second Atraxa (on the combo turn hand size never applies to the reveal).
- Daze is live whenever they control any Island-typed land, tapped or untapped. Only an untapped land or a Petal already on the battlefield pays for it; a Petal in hand cannot be cast in response.
- With Aluren or Omniscience in play (or castable) and Acererak in hand, loop now and keep choosing Lost Mine until they are dead. Do not cantrip, attack or take Tomb first. Trapped Entry only for the last point. If you do enter Tomb, take Veils of Fear and Sandfall Cell, never Oubliette.
- Spend Force of Will on what breaks the loop (removal on Acererak while his enter trigger is on the stack), not on Swords for a Show-and-Tell Atraxa and not on face burn when you are tapped out and they have Daze.
- Against Boros, Veil of Summer is a blank in game 1. After a mulligan, bottom the card that is dead in the matchup (Veil) and keep the Show and Tell payoff (Omniscience, Atraxa).
- Boros permanents: Voice of Victory stops your spells on their turn, not theirs on yours; count their untapped white (and Lazotep Quarry with a creature) as Swords before looping; Karakas bounces a legendary Atraxa or Acererak; Spider-Woman makes Petals and the Show-and-Tell Atraxa enter tapped (cast Petals a turn early).
- Hedge Maze: at their end step fetch it over a dual or basic when either gives the colours you need; from hand, play it on a turn where the untapped land's mana would go unused (play Hedge Maze rather than Boseiju before a Ponder).
- Brainstorm, fetch and Ponder: put back duplicates (a second Atraxa or Show and Tell, extra lands), never your only Aluren or Acererak. Sometimes Brainstorm then Ponder to shuffle; with a fetch in hand, weave it in so Ponder sees fresh cards.
- Aluren is symmetric: the opponent casts MV 3 or less creatures free at instant speed (free Phlage deals 3; Amped Raptor can discover Swords, Erode or Bombardment). Do not cast Aluren at 3 life or less against Boros.
- Ponder: without an enabler in hand, keep a known Aluren or Show and Tell on top; otherwise shuffle any look with no enabler, creature or cantrip. A lone land is not enough unless you are stuck on one land with none in hand (then shuffle any landless look).
- Show and Tell with Acererak in hand: put in Aluren, not Atraxa, then cast the free Acererak and loop. This includes a known Acererak on top with a cantrip and a spare mana to draw it. Against Boros with white open, weigh Swords first.
- Stock Up with two or fewer lands and none in hand: take a land first. Take a creature only if Aluren or Show and Tell can cast it next turn; with Aluren and Acererak already in hand, dig for mana.
- A keep needs a route to a payoff. A seven whose only spells are Show and Tell and Acererak, or one land plus Ponder with no creature, is a mulligan. Exception (Brady): if you have the combo you do not need cantrips and can keep a one-lander.
- Veil timing is situational (Brady): play it first to play around Daze; otherwise play it later to surprise the opponent and draw a card.
- Waiting a turn instead of jamming is contextual (Brady): with no clock on you it is reasonable, but only if waiting helps you more than the opponent (you may draw mana, or a second combo card for another push).

## Notes for the opponent decks in Alurentell matchups (from LvL reviews; Brady signed off 2026-10-07)

UR Cutter:
- Keep one blue pitch card per Force of Will. Never Brainstorm, Preordain-bottom or surveil away Daze, Force or the last pitch card against a combo deck.
- Force a Veil cast on their own turn: once it resolves, every spell that turn is uncounterable.
- Wasteland: Ancient Tomb when Show and Tell is the threat; the green dual once Aluren is seen (GG); their only land on turn 2 against a one-land keep. Use it before they untap, to cut the most mana.
- Cast cantrips before the land drop when your lands are fetches or a Wasteland could be found.
- Do not tap out on your own turn for Murktide or Cutter once they can reach 4 mana: hold U (and R) for Prismari Charm, which bounces Aluren in response to the free Acererak, or Unholy Heat. Brainstorm at their end step or in response.
- Bolt plus Unholy Heat (delirium) on Acererak with his trigger on the stack ends the loop; Bolt alone does not.
- Daze: use it on a tapped-out Stock Up or Show and Tell, or on their Force when they are tapped out. It is nearly dead into open green mana, because Veil answers it.
- Mishra's Bauble: target yourself if you can influence your library (most often a fetchland), or hold it for a DRC or Cutter trigger.
- Against Atraxa: chump early when it turns on delirium; Heat at the end step on the turn she took block damage; use Charm's bounce mode, not surveil; burn face to 2 or less to turn off their Tombs and Force.

Boros Aggro:
- Wasteland: with no 1-drop, play it as the turn-1 land. Hit Ancient Tomb or their only blue or green dual, not an uncracked fetch. Use it before they untap, not after a spell. Never surveil Wasteland away while they are short of lands.
- Keep W open (or Quarry plus a creature) for Swords on Acererak with his trigger on the stack, and fire on the first cast: each lap draws them a card.
- Under their Aluren, cast your MV 3 or less creatures free at instant speed (free Phlage kills at 3 life or less); cast Amped Raptor last so its discover resolves above Swords.
- Goblin Bombardment: hold it for their combo turn; let them pay Tomb and fetch life, then sacrifice everything in response; kill the first Acererak on its enter trigger.
- Cast the second Ajani while the first is out: the legend-rule death flips the survivor, and Avenger's 0 deals damage equal to your creature count with a red permanent. In the menu "ability 1" is the +2 and "ability 2" is the 0.
- Voice of Victory first among 2-drops against Force decks. Keep Karakas untapped for a legendary Atraxa. Spider-Woman early makes their Petals and Atraxa enter tapped.
- Swords the Show-and-Tell Atraxa before attacking even though they gain 7, but count your removal against their Force, since Acererak needs an answer too.
- Count to ten permanents for Ascend before the end step. Order Voice's sacrifice trigger first so Ocelot Pride copies the mobilize Warriors.
- One pick per command: a scripted Bombardment loop once hit The Atropal instead of the player.
