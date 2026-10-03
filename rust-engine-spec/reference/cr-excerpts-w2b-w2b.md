# CR excerpts retrieved by batch W2b (state-based actions, replacement effects, layers)

Retrieved 2026-10-01 from the Savecraft `rules_search` module (game_id magic), which serves the Comprehensive Rules effective November 14, 2025. Text is copied verbatim (rule examples kept where retrieved; the module's appended "reasoning guide" and "interaction patterns" are NOT CR text and are not copied).

## 704 (state-based actions)

704.4 Unlike triggered abilities, state-based actions pay no attention to what happens during the resolution of a spell or ability.
  Example: A player controls Maro, a creature with the ability "Maro's power and toughness are each equal to the number of cards in your hand" and casts a spell whose effect is "Discard your hand, then draw seven cards." Maro will temporarily have toughness 0 in the middle of the spell's resolution but will be back up to toughness 7 when the spell finishes resolving. Thus Maro will survive when state-based actions are checked. In contrast, an ability that triggers when the player has no cards in hand goes on the stack after the spell resolves, because its trigger event happened during resolution.

704.5 The state-based actions are as follows:

704.5a If a player has 0 or less life, that player loses the game.

704.5b If a player attempted to draw a card from a library with no cards in it since the last time state-based actions were checked, that player loses the game.

704.5d If a token is in a zone other than the battlefield, it ceases to exist.

704.5e If a copy of a spell is in a zone other than the stack, it ceases to exist. If a copy of a card is in any zone other than the stack or the battlefield, it ceases to exist.

704.5f If a creature has toughness 0 or less, it's put into its owner's graveyard. Regeneration can't replace this event.

704.5g If a creature has toughness greater than 0, it has damage marked on it, and the total damage marked on it is greater than or equal to its toughness, that creature has been dealt lethal damage and is destroyed. Regeneration can replace this event.

704.5h If a creature has toughness greater than 0, and it's been dealt damage by a source with deathtouch since the last time state-based actions were checked, that creature is destroyed. Regeneration can replace this event.

704.5j If two or more legendary permanents with the same name are controlled by the same player, that player chooses one of them, and the rest are put into their owners' graveyards. This is called the "legend rule."

704.5m If an Aura is attached to an illegal object or player, or is not attached to an object or player, that Aura is put into its owner's graveyard.

704.5n If an Equipment or Fortification is attached to an illegal permanent or to a player, it becomes unattached from that permanent or player. It remains on the battlefield.

704.5q If a permanent has both a +1/+1 counter and a -1/-1 counter on it, N +1/+1 and N -1/-1 counters are removed from it, where N is the smaller of the number of +1/+1 and -1/-1 counters on it.

704.5t If a player's venture marker is on the bottommost room of a dungeon card, and that dungeon card isn't the source of a room ability that has triggered but not yet left the stack, the dungeon card's owner removes it from the game. See rule 309, "Dungeons."

## 614 (replacement effects)

614.1 Some continuous effects are replacement effects. Like prevention effects (see rule 615), replacement effects apply continuously as events happen—they aren't locked in ahead of time. Such effects watch for a particular event that would happen and completely or partially replace that event with a different event. They act like "shields" around whatever they're affecting.

614.1a Effects that use the word "instead" are replacement effects. Most replacement effects use the word "instead" to indicate what events will be replaced with other events.

614.1c Effects that read "[This permanent] enters with . . . ," "As [this permanent] enters . . . ," or "[This permanent] enters as . . . " are replacement effects.

614.1d Continuous effects that read "[This permanent] enters . . ." or "[Objects] enter [the battlefield] . . ." are replacement effects.

614.5 A replacement effect doesn't invoke itself repeatedly; it gets only one opportunity to affect an event or any modified events that may replace that event.

614.6 If an event is replaced, it never happens. A modified event occurs instead, which may in turn trigger abilities. Note that the modified event may contain instructions that can't be carried out, in which case the impossible instruction is simply ignored.

614.12 Some replacement effects modify how a permanent enters the battlefield. (See rules 614.1c–d.) Such effects may come from the permanent itself if they affect only that permanent (as opposed to a general subset of permanents that includes it). They may also come from other sources. To determine which replacement effects apply and how they apply, check the characteristics of the permanent as it would exist on the battlefield, taking into account replacement effects that have already modified how it enters the battlefield (see rule 616.1), continuous effects from the permanent's own static abilities that would apply to it once it's on the battlefield, and continuous effects that already exist and would apply to the permanent.

614.15 Some replacement effects are not continuous effects. Rather, they are an effect of a resolving spell or ability that replace part or all of that spell or ability's own effect(s). Such effects are called self-replacement effects. The text creating a self-replacement effect is usually part of the ability whose effect is being replaced, but the text can be a separate ability, particularly when preceded by an ability word. When applying replacement effects to an event, self-replacement effects are applied before other replacement effects.

614.17 Some effects state that something can't happen. These effects aren't replacement effects, but follow similar rules.

## 616 (interaction of replacement effects)

616.1 If two or more replacement and/or prevention effects are attempting to modify the way an event affects an object or player, the affected object's controller (or its owner if it has no controller) or the affected player chooses one to apply, following the steps listed below. If two or more players have to make these choices at the same time, choices are made in APNAP order (see rule 101.4).

616.1a If any of the replacement and/or prevention effects are self-replacement effects (see rule 614.15), one of them must be chosen. If not, proceed to rule 616.1b.

616.1b If any of the replacement and/or prevention effects would modify under whose control an object would enter the battlefield, one of them must be chosen. If not, proceed to rule 616.1c.

616.1c If any of the replacement and/or prevention effects would cause an object to become a copy of another object as it enters the battlefield, one of them must be chosen. If not, proceed to rule 616.1d.

616.1d If any of the replacement and/or prevention effects would cause a card to enter the battlefield with its back face up, one of them must be chosen (See rule 701.27, "Transform," and rule 701.28, "Convert."). If not, proceed to 616.1e.

616.1e Any of the applicable replacement and/or prevention effects may be chosen.

616.1f Once the chosen effect has been applied, this process is repeated (taking into account only replacement or prevention effects that would now be applicable) until there are no more left to apply.

616.1g While following the steps in 616.1a–f, one replacement or prevention effect may apply to an event, and another may apply to an event contained within the first event. In this case, the second effect can't be chosen until after the first effect has been chosen.

## 611 and 613 (continuous effects, layers, timestamps)

611.2a A continuous effect generated by the resolution of a spell or ability lasts as long as stated by the spell or ability creating it (such as "until end of turn"). If no duration is stated, it lasts until the end of the game.

611.2c If a continuous effect generated by the resolution of a spell or ability modifies the characteristics or changes the controller of any objects, the set of objects it affects is determined when that continuous effect begins. After that point, the set won't change. (Note that this works differently than a continuous effect from a static ability.) A continuous effect generated by the resolution of a spell or ability that doesn't modify the characteristics or change the controller of any objects modifies the rules of the game, so it can affect objects that weren't affected when that continuous effect began. If a single continuous effect has parts that modify the characteristics or changes the controller of any objects and other parts that don't, the set of objects each part applies to is determined independently.
  Example: An effect that reads "All white creatures get +1/+1 until end of turn" gives the bonus to all permanents that are white creatures when the spell or ability resolves—even if they change color later—and doesn't affect those that enter the battlefield or turn white afterward.

611.3a A continuous effect generated by a static ability isn't "locked in"; it applies at any given moment to whatever its text indicates.

611.3b The effect applies at all times that the permanent generating it is on the battlefield or the object generating it is in the appropriate zone.
  Example: A permanent with the static ability "All white creatures get +1/+1" generates an effect that continuously gives +1/+1 to each white creature on the battlefield. If a creature becomes white, it gets this bonus; a creature that stops being white loses it.

611.3c Continuous effects that modify characteristics of permanents do so simultaneously with the permanent entering the battlefield. They don't wait until the permanent is on the battlefield and then change it. Because such effects apply as the permanent enters the battlefield, they are applied before determining whether the permanent will cause an ability to trigger when it enters the battlefield.

613.1 The values of an object's characteristics are determined by starting with the actual object. For a card, that means the values of the characteristics printed on that card. For a token or a copy of a spell or card, that means the values of the characteristics defined by the effect that created it. Then all applicable continuous effects are applied in a series of layers in the following order:

613.1d Layer 4: Type-changing effects are applied. These include effects that change an object's card type, subtype, and/or supertype.

613.1f Layer 6: Ability-adding effects, keyword counters, ability-removing effects, and effects that say an object can't have an ability are applied.

613.1g Layer 7: Power- and/or toughness-changing effects are applied.

613.4 Within layer 7, apply effects in a series of sublayers in the order described below. Within each sublayer, apply effects in timestamp order. (See rule 613.7.) Note that dependency may alter the order in which effects are applied within a sublayer. (See rule 613.8.)

613.4a Layer 7a: Effects from characteristic-defining abilities that define power and/or toughness are applied. See rule 604.3.

613.4b Layer 7b: Effects that set power and/or toughness to a specific number or value are applied. Effects that refer to the base power and/or toughness of a creature apply in this layer.

613.4c Layer 7c: Effects and counters that modify power and/or toughness (but don't set power and/or toughness to a specific number or value) are applied.

613.7 Within a layer or sublayer, determining which order effects are applied in is usually done using a timestamp system. An effect with an earlier timestamp is applied before an effect with a later timestamp.

613.7a A continuous effect generated by a static ability has the same timestamp as the object the static ability is on, or the timestamp of the effect that created the ability, whichever is later. If the effect that created the ability has the later timestamp and the object the ability is on receives a new timestamp, each continuous effect generated by static abilities of that object receives a new timestamp as well, but the relative order of those timestamps remains the same.

613.7b A continuous effect generated by the resolution of a spell or ability receives a timestamp at the time it's created.

613.7c Each counter receives a timestamp as it's put on an object or player. If that object or player already has a counter of that kind on it, each counter of that kind receives a new timestamp identical to that of the new counter.

613.7d An object receives a timestamp at the time it enters a zone.

613.7e An Aura, Equipment, or Fortification receives a new timestamp each time it becomes attached to an object or player.

613.7m If two or more objects would receive a timestamp simultaneously, such as by entering a zone simultaneously or becoming attached simultaneously, their relative timestamps are determined in APNAP order (see rule 101.4). Objects controlled by the active player (or owned by the active player, if they have no controller) have an earlier relative timestamp in the order of that player's choice, followed by each other player in turn order.

613.8 Within a layer or sublayer, determining which order effects are applied in is sometimes done using a dependency system. If a dependency exists, it will override the timestamp system.

613.10 Some continuous effects affect players rather than objects. For example, an effect might give a player protection from red. All such effects are applied in timestamp order after the determination of objects' characteristics. See also the rules for timestamp order and dependency (rules 613.7 and 613.8).

613.11 Some continuous effects affect game rules rather than objects. For example, effects may modify a player's maximum hand size, or say that a creature must attack this turn if able. These effects are applied after all other continuous effects have been applied. Continuous effects that affect the costs of spells or abilities are applied according to the order specified in rule 601.2f. All other such effects are applied in timestamp order. See also the rules for timestamp order and dependency (rules 613.7 and 613.8).

## Keywords and permanents (hexproof, deathtouch, Equipment, equip, damage)

702.11b "Hexproof" on a permanent means "This permanent can't be the target of spells or abilities your opponents control."

702.11c "Hexproof" on a player means "You can't be the target of spells or abilities your opponents control."

702.11d "Hexproof from [quality]" is a variant of the hexproof ability. "Hexproof from [quality]" on a permanent means "This permanent can't be the target of [quality] spells your opponents control or abilities your opponents control from [quality] sources." A "hexproof from [quality]" ability is a hexproof ability.

702.11f "Hexproof from [quality A] and from [quality B]" is shorthand for "hexproof from [quality A]" and "hexproof from [quality B]"; it behaves as two separate hexproof abilities.

702.2b A creature with toughness greater than 0 that's been dealt damage by a source with deathtouch since the last time state-based actions were checked is destroyed as a state-based action. See rule 704.

702.2d The deathtouch rules function no matter what zone an object with deathtouch deals damage from.

301.5 Some artifacts have the subtype "Equipment." An Equipment can be attached to a creature. It can't legally be attached to anything that isn't a creature.

301.5b Equipment spells are cast like other artifact spells. Equipment enter the battlefield like other artifacts. They don't enter the battlefield attached to a creature. The equip keyword ability attaches the Equipment to a creature you control (see rule 702.6, "Equip"). Control of the creature matters only when the equip ability is activated and when it resolves. Spells and other abilities may also attach an Equipment to a creature. If an effect attempts to attach an Equipment to an object that can't be equipped by it, the Equipment doesn't move.

301.5c An Equipment that's also a creature can't equip a creature unless that Equipment has reconfigure (see rule 702.151, "Reconfigure"). An Equipment that loses the subtype "Equipment" can't equip a creature. An Equipment can't equip itself. An Equipment that equips an illegal or nonexistent permanent becomes unattached from that permanent but remains on the battlefield. (This is a state-based action. See rule 704.) An Equipment can't equip more than one creature. If a spell or ability would cause an Equipment to equip more than one creature, the Equipment's controller chooses which creature it equips.

301.5f An ability of a permanent that refers to the "equipped creature" refers to whatever creature that permanent is attached to, even if the permanent with the ability isn't an Equipment.

702.6a Equip is an activated ability of Equipment cards. "Equip [cost]" means "[Cost]: Attach this permanent to target creature you control. Activate only as a sorcery."

120.6 Damage marked on a creature remains until the cleanup step, even if that permanent stops being a creature. If the total damage marked on a creature is greater than or equal to its toughness, that creature has been dealt lethal damage and is destroyed as a state-based action (see rule 704). All damage marked on a permanent is removed when it regenerates (see rule 701.19, "Regenerate") and during the cleanup step (see rule 514.2).

514.2 Second, the following actions happen simultaneously: all damage marked on permanents (including phased-out permanents) is removed and all "until end of turn" and "this turn" effects end. This turn-based action doesn't use the stack.

## Card types, ability words, tokens, summoning sickness, ETB and look-back triggers

205.2a The card types are artifact, battle, conspiracy, creature, dungeon, enchantment, instant, kindred, land, phenomenon, plane, planeswalker, scheme, sorcery, and vanguard. See section 3, "Card Types."

207.2c An ability word appears in italics at the beginning of some abilities. Ability words are similar to keywords in that they tie together cards that have similar functionality, but they have no special rules meaning and no individual entries in the Comprehensive Rules. The ability words are adamant, addendum, alliance, battalion, bloodrush, celebration, channel, chroma, cohort, constellation, converge, council's dilemma, coven, delirium, descend 4, descend 8, domain, eerie, eminence, enrage, fateful hour, fathomless descent, ferocious, flurry, formidable, grandeur, hellbent, heroic, imprint, inspired, join forces, kinship, landfall, lieutenant, magecraft, metalcraft, morbid, pack tactics, paradox, parley, radiance, raid, rally, renew, revolt, secret council, spell mastery, strive, survival, sweep, tempting offer, threshold, undergrowth, valiant, void, and will of the council.

111.7 A token that's in a zone other than the battlefield ceases to exist. This is a state-based action; see rule 704. (Note that if a token changes zones, applicable triggered abilities will trigger before the token ceases to exist.)

302.6 A creature's activated ability with the tap symbol or the untap symbol in its activation cost can't be activated unless the creature has been under its controller's control continuously since their most recent turn began. A creature can't attack unless it has been under its controller's control continuously since their most recent turn began. This rule is informally called the "summoning sickness" rule.

603.6a Enters-the-battlefield abilities trigger when a permanent enters the battlefield. These are written, "When [this object] enters, . . . " or "Whenever a [type] enters, . . ." Each time an event puts one or more permanents onto the battlefield, all permanents on the battlefield (including the newcomers) are checked for any enters-the-battlefield triggers that match the event.

603.10a Some zone-change triggers look back in time. These are leaves-the-battlefield abilities, abilities that trigger when a card leaves a graveyard, and abilities that trigger when an object that all players can see is put into a hand or library.

## Golden rules, game loss, drawing, attack requirements, triggering

101.2 When a rule or effect allows or directs something to happen, and another effect states that it can't happen, the "can't" effect takes precedence.

101.2a Adding abilities to objects and removing abilities from objects don't fall under this rule. (See rule 113.10.)

113.10 Effects can add or remove abilities of objects. An effect that adds an ability will state that the object "gains" or "has" that ability, or similar. An effect that removes an ability will state that the object "loses" that ability.

104.4a If all the players remaining in a game lose simultaneously, the game is a draw.

121.4 A player who attempts to draw a card from a library with no cards in it loses the game the next time a player would receive priority. (This is a state-based action. See rule 704.)

508.1d The active player checks each creature they control to see whether it's affected by any requirements (effects that say a creature attacks if able, or that it attacks if some condition is met). If the number of requirements that are being obeyed is fewer than the maximum possible number of requirements that could be obeyed without disobeying any restrictions, the declaration of attackers is illegal. If a creature can't attack unless a player pays a cost, that player is not required to pay that cost, even if attacking with that creature would increase the number of requirements being obeyed. If a requirement that says a creature attacks if able during a certain turn refers to a turn with multiple combat phases, the creature attacks if able during each declare attackers step in that turn.

603.2 Whenever a game event or game state matches a triggered ability's trigger event, that ability automatically triggers. The ability doesn't do anything at this point.

603.2c An ability triggers only once each time its trigger event occurs. However, it can trigger repeatedly if one event contains multiple occurrences.

603.2g An ability triggers only if its trigger event actually occurs. An event that's prevented or replaced won't trigger anything.
  Example: An ability that triggers on damage being dealt won't trigger if all the damage is prevented.

700.4 The term dies means "is put into a graveyard from the battlefield."

## Zones, countering, trample, haste, counters on entry

400.7 An object that moves from one zone to another becomes a new object with no memory of, or relation to, its previous existence. This rule has the following exceptions.

400.7d An ability of a permanent can reference information about the spell that became that permanent as it resolved, including what costs were paid to cast that spell or what mana was spent to pay those costs.

701.6a To counter a spell or ability means to cancel it, removing it from the stack. It doesn't resolve and none of its effects occur. A countered spell is put into its owner's graveyard.

702.19b The controller of an attacking creature with trample first assigns damage to the creature(s) blocking it. Once all those blocking creatures are assigned lethal damage, any excess damage is assigned as its controller chooses among those blocking creatures and the player, planeswalker, or battle the creature is attacking. When checking for assigned lethal damage, take into account damage already marked on the creature and damage from other creatures that's being assigned during the same combat damage step, but not any abilities or effects that might change the amount of damage that's actually dealt. The attacking creature's controller need not assign lethal damage to all those blocking creatures but in that case can't assign any damage to the player or planeswalker it's attacking.

702.10b If a creature has haste, it can attack even if it hasn't been controlled by its controller continuously since their most recent turn began. (See rule 302.6.)

122.6 Some spells and abilities refer to counters being put on an object. This refers to putting counters on that object while it's on the battlefield and also to an object that's given counters as it enters the battlefield.

122.6a If an object enters the battlefield with counters on it, the effect causing the object to be given counters may specify which player puts those counters on it. If the effect doesn't specify a player, the object's controller puts those counters on it.

## Dungeons (SBA only), ability zones, end of resolution

309.4c Each room has a triggered ability called a room ability whose effect is printed on the card. They all have the same trigger condition not printed on the card. The full text of each room ability is "When you move your venture marker into this room, [effect.]" As long as a dungeon card is in the command zone, its abilities may trigger. Each room ability is controlled by the player who owns the dungeon card that is that ability's source.

309.6 If a player's venture marker is on the bottommost room of a dungeon card, and that dungeon card isn't the source of a room ability that has triggered but not yet left the stack, the dungeon card's owner removes it from the game. (This is a state-based action. See rule 704.)

113.6 Abilities of an instant or sorcery spell usually function only while that object is on the stack. Abilities of all other objects usually function only while that object is on the battlefield. The exceptions are as follows:

113.6g An object's ability that states it can't be countered or can't be copied functions on the stack.

113.6h An object's ability that modifies how that particular object enters the battlefield functions as that object is entering the battlefield. See rule 614.12.

608.2n As the final part of an instant or sorcery spell's resolution, the spell is put into its owner's graveyard. As the final part of an ability's resolution, the ability is removed from the stack and ceases to exist.

(Keyword search "until your next turn" on the same module returned no rule that defines when an "until your next turn" duration ends for a player still in the game; only 800.4m (multiplayer, player leaves) mentions it. Scenarios here therefore assert only that such an effect still applies during the opponent's following turn and no longer applies in the controller's next main phase.)

## Combat damage, blocking, prowess, mana abilities

510.2 Second, all combat damage that's been assigned is dealt simultaneously. This turn-based action doesn't use the stack. No player has the chance to cast spells or activate abilities between the time combat damage is assigned and the time it's dealt.

509.1a The defending player chooses which creatures they control, if any, will block. The chosen creatures must be untapped and they can't also be battles. For each of the chosen creatures, the defending player chooses one creature for it to block that's attacking that player, a planeswalker they control, or a battle they protect.

702.108a Prowess is a triggered ability. "Prowess" means "Whenever you cast a noncreature spell, this creature gets +1/+1 until end of turn."

605.3a A player may activate an activated mana ability whenever they have priority, whenever they are casting a spell or activating an ability that requires a mana payment, or whenever a rule or effect asks for a mana payment, even if it's in the middle of casting or resolving a spell or activating or resolving an ability.

605.3b An activated mana ability doesn't go on the stack, so it can't be targeted, countered, or otherwise responded to. Rather, it resolves immediately after it is activated. (See rule 405.6c.)

405.6c Mana abilities resolve immediately. If a mana ability both produces mana and has another effect, the mana is produced and the other effect happens immediately. If a player had priority before a mana ability was activated, that player gets priority after it resolves. (See rule 605, "Mana Abilities.")
