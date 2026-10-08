# Against UR Cutter

Use when: the opponent is UR Cutter (Forge AI or an LLM pilot): Force of Will, Daze, Wasteland, Prismari Charm, Lightning Bolt, Unholy Heat, Dragon's Rage Channeler, Murktide, Cutter. Brady is also mixing in a Stormchaser's Talent UR deck (decks/ur-cutter-alt.txt); it has no rules or sideboard plan of its own yet.

Each of our rules is a default. Read its "Breaks when:" before applying it; if the situation matches, follow the principle instead.

## What has happened in our games (P8, P12)

- **UR1** [P8, P12] They hold Force of Will and Daze for my first Stock Up (a01, a04, a05). Veil of Summer first if I have it; otherwise a Stock Up into Force of Will plus Daze is a 2-for-1 for me. (observed)
  Breaks when: no known exception.
- **UR2** [P12] Prismari Charm (bounce) and Lightning Bolt are the cards that have hit Atraxa/me. (observed, a01 to a06) The answer to the Charm is Veil in response: protecting-the-combo.md PR8.
  Breaks when: no known exception.
- **UR3** [P12, P13, P6] With Prismatic Ending in hand (games 2 and 3) and a Dragon's Rage Channeler on their turn 1, your turn 1 is Prismatic Ending on the Channeler. The fetch choice is part of the play: Ending needs white, and only Tundra or Savannah give it (the basic Island and Tropical Island do not), so crack the fetch for one of them. Brady: "Our turn 1 should be prismatic ending to kill the channeler." (Brady, m01 comments, 2026-10-08; m01 game 2 turn 1: Misty fetched the basic Island and Brainstorm was cast; the m01 replay player also fetched Island, for Wasteland and an untapped blue, and called white "only for Prismatic Ending")
  Breaks when: no known exception from Brady. It is the exception to fetching basics first against Wasteland (mana-and-life.md ML8), and that turn's one mana goes to Ending, not to a cantrip (cantrips-and-shuffling.md CS17).
- **UR4** [P6, P12, P15] With Atraxa in play facing their Murktide Regent, attack: Atraxa's lifelink gains 7 even if Murktide blocks and trades. Before you settle for holding Atraxa back, count what one removal spell they topdeck does to your plan: if one removal spell on your blocker loses the game, the life from attacking matters. Brady, after the loss: "we should have attacked I think". This supersedes his earlier advice in the same game to hang back (Atraxa home as a deterrent, their hand empty, Bolt the only threat to play around). (Brady, interactive match m02 game 2, 2026-10-08, as recorded in BRADY-RULINGS2.md; m02 game 2 turns 5 to 8: Atraxa never attacked, and at 6 life Pyroblast on Atraxa followed by Murktide and the Monk tokens was lethal)
  Breaks when: one removal spell on Atraxa would not lose you the game: then the superseded reasoning can still apply (attacking lets them attack back; Atraxa at home is a deterrent). Brady's own word was "I think".
- **UR5** [P12, P13] Pyroblast and Red Elemental Blast are red spells. Veil of Summer gives hexproof from blue and black only, so Pyroblast ("destroy target permanent if it's blue") kills Atraxa through Veil. Only Force of Will, Daze and Counterspell-style cards are blue. UR Cutter boards in 4 Pyroblast for games 2 and 3 (m02 game 2), and its other mode counters a blue spell (in m02 game 2 it countered your Force of Will). So against UR after sideboarding, Veil is not protection for Atraxa or Aluren on the battlefield, and you do not scry Veil to the top as if it were (Brady's third listed mistake of m02 game 2: Veil kept on top from a Lost Mine scry, then cast in response to Pyroblast on Atraxa, which still died). Veil cast before a spell still makes that spell uncounterable that turn, Pyroblast included (protecting-the-combo.md PR17). (Brady, m02 2026-10-08, as recorded in BRADY-RULINGS2.md: "Pyroblast is a red removal spell: Veil of Summer does not stop it (Brady and I both missed it)")
  Breaks when: no known exception from Brady.

## Rules that matter most here (full text in their files)

- Daze and what pays for it: protecting-the-combo.md PR1, PR2. Their tapped-out window: PR3. Spare mana or Veil: PR5, PR7. Do not tap out for Aluren into Daze: PR4.
- Wasteland: mana-and-life.md ML6 to ML11.
- Free Force of Will against an artifact feeds their delirium: protecting-the-combo.md PR12. Their turn-1 fetch puts a land in their graveyard toward delirium too (URo10).
- Veil before Show and Tell against Daze; the land drop before or after Veil is a judgment call: protecting-the-combo.md PR17 and the note after it. Veil does not stop their red Pyroblast on a permanent: UR5, PR8.
- Brady's four mistakes in m02 game 2, each now a rule: fetching before the draw shuffled away a known Carpet (cantrips-and-shuffling.md CS19); Brainstorm on the Acererak turn left no Daze payment (protecting-the-combo.md PR18, a hard call); Veil scried to the top (UR5); possibly the Ponder order (CS18). Also: attack with Atraxa (UR4), plan Carpet on turn 3 (sideboarding.md SB5), floating mana is spent first (process-and-engine.md EF12).
- Under Omniscience, bait with the redundant spell; they answered Goblin tokens with Unholy Heat: show-and-tell.md ST10.
- Orim's Chant against blue decks is a turn-of-the-combo tool: protecting-the-combo.md PR14.
- A Show and Tell put-in land can open up their Pyroblast: show-and-tell.md ST17.
- Sideboard plan for games 2 and 3: sideboarding.md.

## Their pilot's notes (from LvL reviews; Brady signed off 2026-10-07; URo4 and URo10 from his m01 comments, URo11 from his LLM-game tough spots, 2026-10-08)

These rules were written for the LLM playing UR Cutter against Alurentell. Read them as what a well-played UR Cutter will do to you (P12). If the opponent's notes are ever loaded from this file, this section is their copy: unchanged from the LvL reviews except URo4 (sharpened, and the Thundering Falls line) and URo10 (new) from Brady's m01 comments and URo11 (new) from his LLM-game tough spots, the only three that carry principle tags and "Breaks when:" lines. The others carry no "Breaks when:" clauses. In these notes "they" and "their" mean Alurentell (you) and "your" means the UR Cutter player.

- **URo1** Keep one blue pitch card per Force of Will. Never Brainstorm, Preordain-bottom or surveil away Daze, Force or the last pitch card against a combo deck.
- **URo2** Force a Veil cast on their own turn: once it resolves, every spell that turn is uncounterable.
- **URo3** Wasteland: Ancient Tomb when Show and Tell is the threat; the green dual once Aluren is seen (GG); their only land on turn 2 against a one-land keep. Use it before they untap, to cut the most mana.
- **URo4** [P4, P12, P2] Cast cantrips (Brainstorm) before the land drop: they show which land to play, and a land played first gives away information. With Thundering Falls as the land, Brainstorm first also lets the Falls' surveil 1 put the worst card (often a put-back) into the graveyard: "UR: Brainstorm first, then play Thundering Falls, so the Falls surveil can put the worst card in the graveyard" (Brady, m01 comments, 2026-10-08). This is the same Brady rule as Alurentell's own (turn-sequencing.md TS2). The LvL version said "when your lands are fetches or a Wasteland could be found"; Brady's m01 comments state it with no condition: "cutter turn 2 why are we playing a land before brainstorm" (game 1: Volcanic Island played, then Brainstorm, with only Volcanic Island and Thundering Falls as lands in hand), and "cutter: play brainstorm before playing a land" (game 2: Wasteland played, then Brainstorm). (LvL reviews; Brady, m01 comments, 2026-10-08; the m01 replay player again played a land first in game 1 and cast Brainstorm first in game 2)
  Breaks when: no known exception from Brady.
- **URo10** [P12] Crack a fetch on turn 1: the fetchland in the graveyard is a land card toward Dragon's Rage Channeler's delirium. Brady: "Good that cutter fetched on turn 1 to get closer to delirium." (Brady, m01 comments, 2026-10-08; m01 games 1 and 2: Polluted Delta cracked for Volcanic Island on turn 1, then Dragon's Rage Channeler)
  Breaks when: no known exception from Brady.
- **URo11** [P5, P4] When you are the attacking side, attack first, then cast your spells in the second main phase; Brainstorm too. Brady: "attack first, then cast spells second main" (and the same for Brainstorm). This is about combat; URo4 is about the land drop, and both hold: Brainstorm still goes before the land, so a turn that follows both attacks, then casts Brainstorm in the second main and plays the land after it. (Brady, LLM-game tough spots, 2026-10-08, said of both UR Cutter and Boros, vs-boros.md BOo10)
  Breaks when: no known exception from Brady.
- **URo5** Do not tap out on your own turn for Murktide or Cutter once they can reach 4 mana: hold U (and R) for Prismari Charm, which bounces Aluren in response to the free Acererak, or Unholy Heat. Brainstorm at their end step or in response.
- **URo6** Bolt plus Unholy Heat (delirium) on Acererak with his trigger on the stack ends the loop; Bolt alone does not.
- **URo7** Daze: use it on a tapped-out Stock Up or Show and Tell, or on their Force when they are tapped out. It is nearly dead into open green mana, because Veil answers it.
- **URo8** Mishra's Bauble: target yourself if you can influence your library (most often a fetchland), or hold it for a DRC or Cutter trigger.
- **URo9** Against Atraxa: chump early when it turns on delirium; Heat at the end step on the turn she took block damage; use Charm's bounce mode, not surveil; burn face to 2 or less to turn off their Tombs and Force.
