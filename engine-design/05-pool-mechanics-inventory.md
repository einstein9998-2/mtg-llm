# MTG Engine — Mechanics Inventory for the 8-Deck Pool

Status: DRAFT v0.2, re-scoped to the **final eight decks** in `/mnt/project-files/decks/` (deck-set thread, 2026-10-01): UR Cutter, Alurentell (Aluren + Show and Tell + Omniscience), Boros Aggro, BW Death and Taxes, Dimir Tempo, UWx Control, Doomsday, Reanimator. 144 unique non-basic cards counting sideboards (per `legacy-deck-set.md`). This replaces v0.1, which was scoped to the earlier pool and is obsolete.

**How this was produced.**
- I read the eight decklists (main and sideboard) and went through the pool card by card.
- Oracle text came from the card scripts in a local Forge checkout (read-only, used for checking; nothing copied). The pinned Oracle snapshot in the card DB wins over a script's text, and the spec-writer agent re-reads it.
- Rules text was checked against the Comprehensive Rules effective 2025-11-14 for dungeons (CR 309, 701.49, 704.5t), The Ring (701.54), warp (702.185), mobilize (702.181) and impending (702.176).
- Not checked against the CR text: energy, ninjutsu, escape, ward, miracle, replicate, stun counters, ascend, converge, flurry. They are listed with the engine impact I expect; the spec-writer confirms each.

This is the "mechanics inventory" doc 01 section 16 calls for. Regenerate it when the pool changes.

---

## 1. Headline results

1. **Layer 4 is needed. A dependency system is still probably not.** Layers needed: **4** (Kaito is a creature on your turn; Overlord of the Balemurk is not a creature while it has time counters; Magus of the Moon, a Reanimator sideboard card, makes nonbasic lands Mountains), **6** (Magus strips land abilities; Kaito gains hexproof; Guide of Souls' flying counter), **7a** (Barrowgoyf, Nethergoyf CDAs), **7b** (Kaito's 3/4), **7c** (counters, Cori-Steel Cutter, Kaito's emblem). Layers 1 (as a continuous effect), 2, 3 and 5 are not needed. Copiable values are needed to create token copies (Ocelot Pride, Lazotep Quarry). Dependency ordering (CR 613.8) is **not needed by my manual read**: Kaito, Overlord and Magus affect disjoint objects (a planeswalker, an impending permanent, nonbasic lands). The `deps-scan` tool (doc 01 section 10.4) must confirm.
2. **Targeting abilities on the stack is needed.** Stifle (UWx Control, 4 main) targets activated or triggered abilities, and Consign to Memory (UWx, Dimir, Alurentell) counters triggered abilities. Mana abilities are not targetable. v0.1 said this was not needed; that was for the old pool.
3. **New mechanics outside the old plan** (section 2.3): dungeons (Acererak; the deck ventures into Lost Mine or Mad Mage, not Tomb), The Ring tempts you (Samwise), energy, ninjutsu, escape, impending, warp, mobilize, replicate, miracle, modal double-faced lands, transforming walkers (Ajani, Tamiyo), ward, stun counters, ascend, converge, flurry, investigate, surveil, storm (Flusterstorm, sideboard), and Aluren's any-player free casting with flash.
4. **Hidden information remains the main test surface** (section 4): Doomsday, Show and Tell, Stronghold Gambit, Mishra's Bauble, Flow State, Stock Up, Personal Tutor, Atraxa, Raph & Mikey, Thassa's Oracle, the surveil lands, and discard effects.
5. **Randomness** (section 5): library shuffles and random-order bottoming only. No coin flips, dice, or random discard in this pool as far as I read.
6. **Estimated custom (hand-written) cards: roughly 10-14 of 144** (section 6). A judgment, not a measurement.
7. **Volume cards:** Wasteland (four copies in five decks), fetchlands (every deck), Force of Will (five decks), Daze, Brainstorm and Ponder, Thoughtseize, Swords to Plowshares, Bowmasters. They dominate real games, so they get the earliest and heaviest differential testing.

---

## 2. Feature inventory

Tier: **C** = core, **P** = needed by a main deck, **S** = sideboard-only (gate after the main 60), **X** = not needed (loader rejects).

### 2.1 Casting, costs, and timing

| Feature | Pool cards | Tier |
|---|---|---|
| Alternative costs | Force of Will, Force of Negation (only on the opponent's turn), Daze (return an Island: any land with the Island type, which covers Volcanic Island, Underground Sea, Tundra, Tropical Island, Hedge Maze, Meticulous Archive, Undercity Sewers, Thundering Falls, snow Islands), Snuff Out (4 life if you control a Swamp), Massacre (free if an opponent controls a Plains and you control a Swamp), Unmask (exile a black card), Solitude (evoke) | P |
| Additional costs | Bitter Triumph (discard a card or pay 3 life, chosen at cast), Dismember and Surgical Extraction (Phyrexian mana), delve (Murktide Regent) | P |
| Free-cast permissions | **Aluren** (creature spells with mana value 3 or less, for any player, without paying mana costs and as though they had flash), **Omniscience**, Amped Raptor (cast the exiled nonland card by paying energy equal to its mana value) | P |
| X and converge | Prismatic Ending (converge), Hide on the Ceiling (X targets), Wrath of the Skies (X energy), Meltdown (sb) | P/S |
| Replicate | Consign to Memory | P |
| Storm | Flusterstorm (Doomsday sb) | S |
| Flurry | Cori-Steel Cutter (second spell each turn) | P |
| Flash and instant-speed play | Bowmasters, Phelia, Samwise, Solitude, Brazen Borrower, Containment Priest, Disruptor Flute, Faerie Macabre (discard ability), all instants; Aluren grants flash to small creatures for both players | C |
| Warp | Quantum Riddler | P |
| Impending | Overlord of the Balemurk | P |
| Escape | Nethergoyf, Phlage | P |
| Flashback | Faithless Looting, Cabal Therapy (sacrifice a creature) | P |
| Alternate cycling costs | Street Wraith (pay 2 life), Edge of Autumn (sacrifice a land) | P |
| Ninjutsu | Kaito, Bane of Nightmares | P |
| Miracle | Triumph of Saint Katherine (UWx sb, Doomsday sb) | S |
| Modal and escalate | Prismari Charm (three modes; mode 2 has one or two targets), Hydroblast and Pyroblast (two modes), Abrade (sb), Collective Brutality (escalate) | P |
| Adventure | Brazen Borrower (Petty Theft) | P |
| Modal double-faced cards | Boggart Trawler // Boggart Bog, Witch Enchanter // Witch-Blessed Meadow (the land back enters tapped unless 3 life is paid) | P |
| Cast-record dependent effects | Lavinia (counter a spell if no mana was spent), Bilbo (spells cast from anywhere but hand cost {1} less), Grafdigger's Cage (no casting from graveyards or libraries), Veil of Summer (can't be countered). The cast record stores: alternative cost used, mana spent (amount and colors), zone cast from, additional costs, X, converge | P |
| Cost increases and cast restrictions | Defense Grid (+3 except during the caster's turn), Disruptor Flute (named card +3, named-source activations restricted), Deafening Silence (one noncreature spell per player per turn), Gaddock Teeg (noncreature spells with mana value 4+ or X), Voice of Victory (opponents can't cast spells during your turn), Lavinia (noncreature spells above the opponent's land count), Null Rod (artifact activations) | P |
| Counterspell shapes | Daze, Force of Negation (noncreature, exile instead), Flusterstorm, Hydroblast and Pyroblast (color checks), Consign (triggered ability or colorless spell), Stifle (abilities), ward, uncounterable (Koma, Veil of Summer) | P |
| Costed mana abilities | Ancient Tomb (damage), City of Traitors (sacrifice when you play another land), Lazotep Quarry (sacrifice a creature for one mana of any color), Lotus Petal, Lion's Eye Diamond (discard hand), Dark Ritual | C/P |

### 2.2 Zones, objects, abilities, combat

| Feature | Pool cards | Tier |
|---|---|---|
| **Command zone objects** | Dungeon cards (Lost Mine of Phandelver, Dungeon of the Mad Mage, Tomb of Annihilation), The Ring emblem, Kaito's emblem | P |
| Planeswalkers | Kaito (also a creature on your turn), Jace Wielder of Mysteries, Ajani Nacatl Avenger and Tamiyo Seasoned Scholar (back faces) | P |
| Transform | Ajani, Nacatl Pariah (exile, return transformed when one or more other Cats die), Tamiyo, Inquisitive Student (when you draw your third card in a turn); both exile-and-return | P |
| Equipment | Cori-Steel Cutter (+1/+1, trample, haste; flurry attaches it to the Monk token) | P |
| Aura | Animate Dead | P (custom) |
| Tokens | Monk (prowess), Clue, Cat Warrior, Sand Warrior, Zombie (Acererak), Warrior (mobilize), Orc Army (Bowmasters), Koma's Coil, The Atropal, Illusion X/X (Skyclave Apparition), token copies (Ocelot Pride copies tokens that entered this turn once you have the city's blessing; Lazotep Quarry makes a 4/4 Zombie copy of a graveyard creature) | P |
| Exile-until and flicker | Skyclave Apparition, Cloak and Dagger (exile until it leaves), Phelia (returns at the next end step), Flickerwisp-style effects, Hide on the Ceiling (returns at next end step), Shallow Grave (exile at the end step) | P |
| Delayed triggers | Mishra's Bauble (draw at the next upkeep), Shallow Grave, Phelia, Hide on the Ceiling, mobilize Warriors (sacrifice), warp (exile) | P |
| Replacement effects | Containment Priest (nontoken creature that enters without being cast is exiled; hits Reanimator, Phelia's returns, Show and Tell), Grafdigger's Cage (creature cards in graveyards and libraries can't enter), Spider-Woman (opponents' artifacts and creatures enter tapped), Quantum Riddler (one extra draw when hand is 0-1), Jace Wielder (win instead of drawing from an empty library), stun counters (untap replaced), MDFC land back ("pay 3 life or tapped"), "as enters" choices (Disruptor Flute, Cavern of Souls) | P |
| Triggers | Ocelot Pride (end step, life gained this turn, city's blessing), Guide of Souls (creature enters; attack with energy payment and a reflexive "when you do"), Bowmasters (extra draws, ETB, amass), Sand Scout (once each turn when land cards hit your graveyard; Desert search on entry), Moonshadow (permanent cards to graveyard), Barrowgoyf (combat damage, mill, return a creature), Phelia, Overlord (enter or attack), Samwise, batch "one or more" triggers (Ajani, Moonshadow, Sand Scout) | P |
| P/T and characteristic statics | Barrowgoyf (all graveyards), Nethergoyf (own graveyard), Kaito, Cori-Steel Cutter, Moonshadow (counters), delirium (Unholy Heat, DRC) | P |
| Protection family | Koma (ward {4}), Veil of Summer (hexproof from blue and from black), Kaito (hexproof on your turn) | P |
| Energy, Ring, dungeons, ascend | Amped Raptor, Guide of Souls, Wrath of the Skies; Samwise; Acererak; Ocelot Pride (the city's blessing is a permanent player designation) | P |
| Triggered mana | Carpet of Flowers (beginning of each of your main phases, optional, X from target opponent's Islands): a triggered ability that adds mana, not a mana ability, so it uses the stack | P |
| Land features | Fetchlands and Prismatic Vista, land-type searches (Marsh Flats finds Scrubland and other Plains or Swamp duals), seven surveil lands, Boseiju channel (discard from hand, cheaper per legendary creature, opponent may fetch a basic-typed land), Karakas, Wasteland (nonbasic land destruction), Lazotep Quarry (a Desert), Sand Scout's Desert search, Cavern of Souls | C/P |
| Legend rule | Many legendary permanents (Phelia, Samwise, Spider-Woman, Cloak and Dagger, Acererak, Griselbrand, Koma, Raph & Mikey, Boseiju, Karakas, Bilbo, Lavinia, Loran, Gaddock Teeg, Ajani, Tamiyo, Kaito, Jace, Phlage) | C |
| Combat | First strike, flying, deathtouch, lifelink, trample, menace, haste, swampwalk (Street Wraith), mobilize and Raph & Mikey (tokens tapped and attacking), ninjutsu, attack triggers, damage assignment per CR 510.1c (no blocker ordering) | P |
| Opponent decisions inside my effects | Acererak (attack: opponent sacrifices a creature or I make a Zombie), Tomb of Annihilation rooms (each player discards or sacrifices or loses life), used by Alurentell when taxes shut the loop off, Archon of Cruelty, Boseiju, Path, Erode and White Orchid Phantom (controller may search for a basic) | P |
| History queries | Ocelot Pride (life gained this turn), Kaito (an opponent lost life this turn), Bowmasters (draw counts), Tamiyo (third draw), Sand Scout (once per turn), Deafening Silence and flurry (spells cast this turn), Fatal Push (revolt: a permanent left the battlefield this turn) | P |

### 2.3 New mechanics, with rules and engine impact

| Mechanic | Cards | Engine impact (CR cites where I checked) |
|---|---|---|
| **Dungeons / venture** | Acererak | **How the card is actually played:** Acererak says "when it enters, if you haven't completed Tomb of Annihilation, return it to its owner's hand and venture into the dungeon." The Alurentell player ventures into **Lost Mine of Phandelver or Dungeon of the Mad Mage, never Tomb of Annihilation**, so the condition stays true and Acererak keeps returning to hand. With Aluren (creatures with mana value 3 or less cast free, at instant speed, for any player) each return is a free recast: cast Acererak, ETB trigger, bounce and venture (a room ability triggers), cast Acererak again. Each loop produces a room trigger and gives Aluren-castable creatures extra enters. Venturing into Tomb of Annihilation is the **alternative line**, chosen when something stops the free-cast loop: cost increases still apply to a free cast (CR 118.9d, 601.2f), so a tax effect on Acererak (Sphere of Resistance, Damping Sphere, Trinisphere, or in this pool Disruptor Flute naming Acererak, or Defense Grid outside the caster's turn) means each recast costs real mana. Then it is better to finish Tomb of Annihilation, which turns the bounce off so Acererak stays on the battlefield, and completing it also makes The Atropal, a 4/4 deathtouch token. So **which dungeon to choose is a real decision with a different best answer depending on the opponent's board**; the engine presents all three every time and never pre-selects. Rules (CR 309, 701.49, checked against the 2025-11-14 CR): dungeon cards begin outside the game (309.2); venturing with no dungeon in the command zone makes the owner **choose** a dungeon card they own from outside the game and put it there with the marker on the top room (309.2a, 309.4a, 701.49a); otherwise the marker moves along an arrow, and the owner chooses if there are several (309.5a, 701.49b); venturing from the bottom room removes the dungeon, completes it, and the owner chooses again (309.5b, 701.49c, 309.7); each room has a triggered ability controlled by the dungeon's owner (309.4c); a state-based action removes a dungeon whose marker is on the bottom room when none of its room abilities is on the stack (309.6, 704.5t). **All three dungeons must be implemented**, because the choice among them is the point (rooms per Forge's scripts): *Lost Mine of Phandelver*: Cave Entrance (scry 1) to Goblin Lair (1/1 Goblin) or Mine Tunnels (Treasure); Goblin Lair to Storeroom (+1/+1 counter on target creature) or Dark Pool (each opponent loses 1, you gain 1); Mine Tunnels to Dark Pool or Fungi Cavern (target creature gets -4/-0 until your next turn); all of these lead to Temple of Dumathoin (draw a card). *Dungeon of the Mad Mage*: Yawning Portal (gain 1) to Dungeon Level (scry 1) to Goblin Bazaar (Treasure) or Twisted Caverns (target creature can't attack until your next turn) to Lost Level (scry 2) to Runestone Caverns (exile the top two cards, you may play them) or Muiral's Graveyard (two 1/1 Skeletons) to Deep Mines (scry 3) to Mad Wizard's Lair (draw three, reveal them, may cast one free). *Tomb of Annihilation*: Trapped Entry (each player loses 1) to Veils of Fear (each player loses 2 unless they discard) to Sandfall Cell (each player loses 2 unless they sacrifice an artifact, creature or land) to Cradle of the Death God (The Atropal, 4/4 deathtouch), or Trapped Entry to Oubliette (discard a card, sacrifice an artifact, a creature and a land) to Cradle. Needs: command-zone dungeon with a venture marker, a per-player record of **which named dungeons are completed** (Acererak checks Tomb specifically), a choose-a-dungeon decision on every fresh venture, room-choice decisions, opponent "pay or lose life" decisions, and room triggers whose source is the command zone. **Loop handling:** the Aluren/Acererak loop is optional and every iteration changes state (marker position, cards drawn, tokens), so the same-state-hash repeat counter will not catch it; the per-game action budget (doc 01 section 6) bounds it, and the loop gets its own scenario set (doc 04 `aluren-acererak-loop`). The Stifle and Consign to Memory responses to Acererak's ETB trigger and to room triggers are the interactive points |
| **The Ring tempts you** | Samwise | An emblem called The Ring per player, plus a Ring-bearer designation on a creature (701.54). Levels by number of temptations: the Ring-bearer is legendary and can't be blocked by creatures with greater power; loot when it attacks; a creature that blocks it is sacrificed at end of combat; opponents lose 3 life when it deals combat damage to a player. Needs player emblem state, a designation on an object, four small triggers, and a "choose a creature" decision on each tempt |
| **Warp** | Quantum Riddler | Alternative cost; a delayed trigger exiles the permanent at the next end step; it can be cast from exile later (702.185). Needs "may be cast from exile" tracking |
| **Impending** | Overlord of the Balemurk | Alternative cost; enters with five time counters; not a creature while a time counter is on it (a layer-4 effect); an end-step trigger removes a counter (702.176) |
| **Mobilize** | Voice of Victory | On attack, create N tapped-and-attacking Warrior tokens, sacrificed at the next end step (702.181) |
| **Energy** | Amped Raptor, Guide of Souls, Wrath of the Skies | Player counter; paying energy as a cost or effect; Amped Raptor exiles until a nonland card and offers a cast for energy, only if it was cast from hand; Wrath of the Skies gains X, then pays any amount |
| **Ninjutsu** | Kaito | Activated ability from hand after blockers are declared: return an unblocked attacker you control and put this onto the battlefield tapped and attacking. What Kaito attacks is for the spec-writer to confirm |
| **Escape** | Nethergoyf, Phlage | Cast from the graveyard with an exile cost. Nethergoyf needs "any number of other cards with four or more card types among them", so enumerate picks with a "done" option valid only once the constraint holds |
| **Ward** | Koma | Triggered counter-unless-pays when it becomes the target; ward {4} |
| **Stun counters** | Kaito's -2 | Replacement of untap: remove a stun counter instead |
| **Ascend** | Ocelot Pride | Permanent designation once you control ten permanents |
| **Converge** | Prismatic Ending | The cast record stores the colors of mana spent |
| **Replicate** | Consign to Memory | One copy per payment; copies may choose new targets |
| **Flurry** | Cori-Steel Cutter | Trigger on the second spell cast each turn |
| **Investigate, surveil, amass** | Tamiyo, surveil lands, Consider, Prismari Charm, Bowmasters | Clue tokens, surveil decision, Army counters |
| **Card-type counts** | Unholy Heat, DRC, Barrowgoyf, Nethergoyf | Distinct card types in graveyards (own or all) |
| **Miracle** | Triumph of Saint Katherine (sb) | Reveal-on-first-draw special trigger window |
| **Storm** | Flusterstorm (sb) | One copy per spell cast earlier this turn |

### 2.4 Not needed by this pool (tier X; the loader rejects them)

Layers 1 (continuous), 2, 3 and 5; dependency ordering (unless `deps-scan` finds a case); control-changing effects; regeneration; morph; suspend; madness; convoke; cascade; sagas; battles; day/night; monarch; initiative; poison; multiplayer and commander rules. Revisit when the pool changes.

---

## 3. Rules points the pool exercises

- **Triggers:** APNAP in two parts (CR 603.3b); no damage assignment order (CR 510.1c). Forge diverges on the latter (kd-0001, doc 04 section 9).
- **Replacement ordering** (CR 616.1): Containment Priest and Grafdigger's Cage versus Reanimate, Animate Dead, Shallow Grave, Show and Tell, Stronghold Gambit, Phelia's and Hide on the Ceiling's returns; Spider-Woman's enters-tapped versus returns; Quantum Riddler's extra draw versus Jace's win replacement on the same draw (the affected player chooses the order).
- **Layers:** Kaito (type, hexproof, 3/4 on your turn while he has a loyalty counter), Overlord (not a creature), Magus of the Moon (nonbasic lands become Mountains in layer 4 and lose their other abilities, including surveil triggers, fetch abilities and Wasteland's, by CR 305.7); CDAs in 7a; counters in 7c.
- **Alternative-cost interplay:** casting without paying a mana cost is an alternative cost (CR 118.9), so Aluren's or Omniscience's permission cannot combine with another alternative cost on the same spell, but additional costs still apply. Lavinia counters any spell cast with no mana spent, which includes Force of Will, Daze-style free alternatives, and Aluren and Omniscience casts.
- **Ward and Stifle:** ward is a triggered ability, so Stifle and Consign can respond to it.
- **Triumph of Saint Katherine:** exiles itself and the top six cards face down, shuffles the pile and puts it on top (section 4).

---

## 4. Hidden-information inventory (doc 02 test corpus)

| Category | Cards | What the knowledge model must do |
|---|---|---|
| Draw and dig in own library | Brainstorm, Ponder, Preordain, Consider, Flow State (top three, one to hand, rest on the bottom in any order), Stock Up (top five, two to hand, rest on the bottom in any order), surveil lands, Prismari Charm, Street Wraith | Owner learns cards and positions; cards put on the bottom in a chosen order are **known to the owner with known bottom positions** |
| Look at a library's top card | Mishra's Bauble (target player's library) | Looker learns the top card; position known until shuffle or draw |
| Search to the top | Personal Tutor (reveal a sorcery, shuffle, put on top) | Identity public, position known (top) to both |
| Reveals from the library | Atraxa (top ten, rest to the bottom in random order), Raph & Mikey (reveal until a creature, rest to the bottom randomly), Amped Raptor (exile until nonland), Overlord and Barrowgoyf (mill), Thassa's Oracle (top X) | Identity public; positions known only until random-order placement |
| Face-down pile put back on top | Triumph of Saint Katherine (sb) | New knowledge form: **known to be within the top K** with unknown order; the determinizer must respect it |
| Hand reveals and choices | Thoughtseize, Duress, Unmask, Cloak and Dagger, Cabal Therapy, Collective Brutality, Bitter Triumph (discard cost) | Caster learns the hand at that moment; discarded and exiled cards become public |
| Simultaneous secret choices | Show and Tell, Stronghold Gambit (each player chooses a card, then reveals) | Paired secret decision (doc 02 section 9.1) |
| Library and graveyard piles | Doomsday (choose five from library and graveyard, order them, exile the rest) | Owner knows the order; the opponent sees counts and the exiled cards |
| Search | Fetchlands, Boseiju's opponent fetch, Path, Erode and Phantom basic fetches, Recruiter of the Guard, Edge of Autumn, Sand Scout, Surgical Extraction (graveyard, hand and library; exact wording left to the spec-writer) | Searcher sees a sorted multiset; the opponent sees only that a shuffle happened |
| Win or lose on library size | Thassa's Oracle, Jace Wielder, Doomsday lines | Library count public; the outcome depends on the owner's own hidden order |
| Naming | Cabal Therapy, Disruptor Flute, Cavern of Souls (type) | `ChooseName` restricted to pool card names (scope cut) |
| Public choices | Samwise (Ring), Acererak (rooms) | Public |

Knowledge model additions needed in doc 02 section 6: known bottom positions, "identity public, order unknown", and a top-K membership constraint.

---

## 5. Randomness inventory

RNG-using effects: library shuffles (every fetchland, Prismatic Vista, Personal Tutor, Boseiju, Edge of Autumn, Sand Scout, Path-style fetches, Surgical Extraction, Triumph's pile) and random-order bottoming (Atraxa and Raph & Mikey; check Thassa's Oracle's pinned text). All draw from the single in-state RNG (doc 01 section 12) and go through the scripted-`Random` protocol in doc 04 section 5.3 so Forge follows our outcomes.

---

## 6. Hand-written card candidates

Likely **custom ops** (the DSL lacks the semantics):

| Card | Why |
|---|---|
| Doomsday | Choose five from library and graveyard, order them, exile the rest, lose half life |
| Show and Tell, Stronghold Gambit | Paired secret choices, simultaneous reveal and entry |
| Animate Dead | Aura that targets a card in a graveyard, rewrites its own enchant restriction, -1/-0, enters under your control |
| Acererak the Archlich | Dungeon state machine across three dungeons, conditional return to hand and venture on entry (the Aluren loop), opponent choice on attack |
| Samwise the Stouthearted | The Ring emblem and Ring-bearer levels |
| Kaito, Bane of Nightmares | Planeswalker that is a creature on your turn, ninjutsu, emblem, stun counters |
| Aluren | Any-player free-cast permission with flash; with Acererak the loop is long when played stepwise; an optional loop shortcut (doc 02 section 9.6, shelved, not in initial scope) could shorten it |
| Atraxa, Grand Unifier | Pick one of each card type from ten revealed cards, rest to the bottom randomly |
| Cloak and Dagger, Entwined | Reveal hand, choose a nonland card or the chosen creature, exile until it leaves |
| Ajani and Tamiyo flips | Exile-and-return-transformed on a batch or counted trigger |

Likely **hybrid** (DSL plus engine support): Lion's Eye Diamond timing, Omniscience, Cavern of Souls, Surgical Extraction, Jace Wielder (draw replacement and ultimate), Amped Raptor, Nethergoyf and Phlage (escape subsets), Skyclave Apparition (last-known mana value feeds the token), Overlord (impending), Quantum Riddler (warp), Consign to Memory (replicate and ability targets), Stifle, Cori-Steel Cutter (flurry), Lazotep Quarry and Ocelot Pride (token copies), Boseiju (channel), MDFC lands, Collective Brutality (escalate), Flusterstorm (storm).

Everything else I expect to compile from the DSL. A judgment, not a measurement.

---

## 7. Staged rollout

Rules features are cumulative across stages. Stages are by engine difficulty and need not match the order of the Forge baseline pipeline (Brady's first matchup, UR Cutter vs Alurentell, is stage C; building the core and stages A and B first still comes first, since every deck uses them). Sideboard cards (S) are gated after the main 60.

| Stage | Decks | New features beyond the shared core |
|---|---|---|
| Core (all decks) | all | Fetchlands, dual and surveil lands, Wasteland, Force of Will, Daze, Brainstorm, Ponder, Thoughtseize, Swords to Plowshares, Bowmasters, basic combat and tokens |
| A | Boros Aggro, BW Death and Taxes | First strike, flash, energy, ascend, token copies, mobilize, Spider-Woman and Voice of Victory statics, Karakas, Sand Scout and Desert search, Lazotep Quarry, Goblin Bombardment, Solitude evoke, Skyclave Apparition, Phelia and flicker returns, Aether Vial, MDFC lands, Overlord impending, Samwise and the Ring, Ajani transform, Phlage escape, Cloak and Dagger, Containment Priest and Cage (S) |
| B | Dimir Tempo, UWx Control | Stifle and Consign (abilities on the stack, replicate), Force of Negation, Fatal Push (revolt), Snuff Out, Kaito (ninjutsu, creature planeswalker, stun, emblem, layers 4/6/7b), Tamiyo (transform, investigate), Nethergoyf and Barrowgoyf (7a, escape), Moonshadow, Bilbo, Flow State, Quantum Riddler (warp), Prismatic Ending (converge), Murktide (delve), Lavinia, Hydroblast and Pyroblast (S) |
| C | UR Cutter, Alurentell | Cori-Steel Cutter flurry, DRC delirium, Prismari Charm, Mishra's Bauble, Lightning Bolt, Unholy Heat, Aluren (any-player free instant-speed casting), Show and Tell (secret simultaneous choice), Omniscience, Acererak (all three dungeons, the Aluren loop), Atraxa, Stock Up, Ancient Tomb and City of Traitors, Boseiju channel, Carpet of Flowers, Defense Grid, Disruptor Flute |
| D | Doomsday | Doomsday piles, Thassa's Oracle, Jace Wielder, Personal Tutor, Street Wraith and Edge of Autumn costs, Dark Ritual, Lion's Eye Diamond, ordered-library knowledge, Flusterstorm (S) |
| E | Reanimator | Reanimate, Animate Dead, Shallow Grave, Unmask, Faithless Looting flashback, Cabal Therapy, Collective Brutality, Griselbrand, Archon of Cruelty, Koma (ward, uncounterable), Raph & Mikey, Stronghold Gambit, Magus of the Moon (S, layers 4 and 6 on lands) |

---

## 8. Scope cuts and scope-cut divergences

A scope cut is a documented decision not to implement a legal action or rule. Forge will still offer it, so the differential harness would flag an action-set mismatch. Each cut gets a `scope-cut` entry in the known-divergence registry (doc 04 section 9) stating what is cut, why it is acceptable here, and the revisit condition. Candidates:

- Venture: **none** (all three dungeons are implemented; choosing among them is how Alurentell avoids completing Tomb).
- Naming any card (Cabal Therapy, Disruptor Flute) restricted to pool card names.
- Two-player paths only (venture "each player", Ring, ward).
- Wishes and sideboard-fetching: none in the pool.
- Miracle and storm: implemented only when their sideboard stages are gated.

---

## 9. Open points for the spec-writer agent and Brady

- Confirm exact Oracle text and rulings for each custom and hybrid card, including Ring levels, ninjutsu with a planeswalker-creature (what Kaito attacks), and Thassa's Oracle's bottoming wording.
- Surgical Extraction's hidden-zone search knowledge rule.
- Show and Tell and Stronghold Gambit secrecy and simultaneity rulings.
- Dungeon edge cases (Brady mentioned some, "barring some edge cases"): whether a completed dungeon can be chosen again, forced Tomb choice if the others are unavailable, and what the opponent's Tomb-related effects can do. The spec-writer to collect rulings.
- Aluren: which zones the permission covers (hand only, or also exile and graveyard casts through Bilbo, escape, warp and adventure).
- Deck-gate card checklists (doc 04 section 8) generated from this inventory per stage.
