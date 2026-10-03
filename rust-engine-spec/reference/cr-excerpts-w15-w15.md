# CR excerpts for batch w15 (Dimir Tempo vs UWx Control matchup)

Rules copied verbatim from reference/cr-excerpts*.md (which were retrieved from the Savecraft rules_search module, CR effective 2025-11-14), plus rules newly retrieved from Savecraft rules_search on 2026-10-01 (marked NEW).

117.3b The active player receives priority after a spell or ability (other than a mana ability) resolves.

117.3c If a player has priority when they cast a spell, activate an ability, or take a special action, that player receives priority afterward.

117.3d If a player has priority and chooses not to take any actions, that player passes. If any mana is in that player's mana pool, they announce what mana is there. Then the next player in turn order receives priority.

117.4 . If all players pass in succession (that is, if all players pass without taking any actions in between passing), the spell or ability on top of the stack resolves or, if the stack is empty, the phase or step ends.

118.9a Only one alternative cost can be applied to any one spell as it's being cast. The controller of the spell announces their intentions to pay that cost as described in rule 601.2b.

118.9d If an alternative cost is being paid to cast a spell, any additional costs, cost increases, and cost reductions that affect that spell are applied to that alternative cost. (See rule 601.2f.)

121.1 . A player draws a card by putting the top card of their library into their hand. This is done as a turn-based action during each player’s draw step. It may also be done as part of a cost or effect of a spell or ability.

121.2 Cards may only be drawn one at a time. If a player is instructed to draw multiple cards, that player performs that many individual card draws.

121.2a An instruction to draw multiple cards can be modified by replacement effects that refer to the number of cards drawn. This modification occurs before considering any of the individual card draws. See rule 616.1g.

121.5 . If an effect moves cards from a player’s library to that player’s hand without using the word “draw,” the player has not drawn those cards. This makes a difference for abilities that trigger on drawing cards and effects that replace card draws, as well as if the player’s library is empty.

614.1a Effects that use the word “instead” are replacement effects. Most replacement effects use the word “instead” to indicate what events will be replaced with other events.

614.5 A replacement effect doesn't invoke itself repeatedly; it gets only one opportunity to affect an event or any modified events that may replace that event.

614.6 If an event is replaced, it never happens. A modified event occurs instead, which may in turn trigger abilities. Note that the modified event may contain instructions that can’t be carried out, in which case the impossible instruction is simply ignored.

504.1 First, the active player draws a card. This turn-based action doesn’t use the stack.

701.6a To counter a spell or ability means to cancel it, removing it from the stack. It doesn’t resolve and none of its effects occur. A countered spell is put into its owner’s graveyard.

701.6b The player who cast a countered spell or activated a countered ability doesn’t get a “refund” of any costs that were paid.

603.2 . Whenever a game event or game state matches a triggered ability's trigger event, that ability automatically triggers. The ability doesn't do anything at this point.

603.2c An ability triggers only once each time its trigger event occurs. However, it can trigger repeatedly if one event contains multiple occurrences.

603.3 Once an ability has triggered, its controller puts it on the stack as an object that's not a card the next time a player would receive priority. See rule 117, "Timing and Priority." The ability becomes the topmost object on the stack. It has the text of the ability that created it, and no other characteristics. It remains on the stack until it's countered, it resolves, a rule causes it to be removed from the stack, or an effect moves it elsewhere.

603.3b If multiple abilities have triggered since the last time a player received priority, the abilities are placed on the stack in a two-part process. First, each player, in APNAP order, puts each triggered ability they control with a trigger condition that isn't another ability triggering on the stack in any order they choose. (See rule 101.4.) Second, each player, in APNAP order, puts all remaining triggered abilities they control on the stack in any order they choose. Then the game once again checks for and performs state-based actions until none are performed, then abilities that triggered during this process go on the stack. This process repeats until no new state-based actions are performed and no abilities trigger. Then the appropriate player gets priority.

603.4 A triggered ability may read "When/Whenever/At [trigger event], if [condition], [effect]." When the trigger event occurs, the ability checks whether the stated condition is true. The ability triggers only if it is; otherwise it does nothing. If the ability triggers, it checks the stated condition again as it resolves. If the condition isn't true at that time, the ability is removed from the stack and does nothing. Note that this mirrors the check for legal targets.

603.7a Delayed triggered abilities are created during the resolution of spells or abilities, as the result of a replacement effect being applied, or as a result of a static ability that allows a player to take an action. A delayed triggered ability won’t trigger until it has actually been created, even if its trigger event occurred just beforehand. Other events that happen earlier may make the trigger event impossible.

603.7b A delayed triggered ability will trigger only once—the next time its trigger event occurs—unless it has a stated duration, such as “this turn.” If its trigger event occurs more than once simultaneously and the ability doesn’t have a stated duration, the controller of the delayed triggered ability chooses which event causes the ability to trigger.

603.7c A delayed triggered ability that refers to a particular object still affects it even if the object changes characteristics. However, if that object is no longer in the zone it’s expected to be in at the time the delayed triggered ability resolves, the ability won’t affect it. (Note that if that object left that zone and then returned, it’s a new object and thus won’t be affected. See rule 400.7.)

603.7d If a spell creates a delayed triggered ability, the source of that delayed triggered ability is that spell. The controller of that delayed triggered ability is the player who controlled that spell as it resolved.

702.185a Warp represents two static abilities that function while the card with warp is on the stack, one of which may create a delayed triggered ability. “Warp [cost]” means “You may cast this card from your hand by paying [cost] rather than its mana cost” and “If this spell’s warp cost was paid, exile the permanent this spell becomes at the beginning of the next end step. Its owner may cast this card after the current turn has ended for as long as it remains exiled.” Casting a spell for its warp cost follows the rules for paying alternative costs in rules 601.2b and 601.2f–h.

608.3g If the object that’s resolving has a static ability that functions on the stack and creates a delayed triggered ability, that delayed triggered ability is created as that permanent is put onto the battlefield in any of the steps above. (See rules 702.109, “Dash,” 702.152, “Blitz,” and 702.185, “Warp.”)

400.7 . An object that moves from one zone to another becomes a new object with no memory of, or relation to, its previous existence. This rule has the following exceptions.

702.49a Ninjutsu is an activated ability that functions only while the card with ninjutsu is in a player’s hand. “Ninjutsu [cost]” means “[Cost], Reveal this card from your hand, Return an unblocked attacking creature you control to its owner’s hand: Put this card onto the battlefield from your hand tapped and attacking.”

702.49c A ninjutsu ability may be activated only while a creature on the battlefield is unblocked (see rule 509.1h). The creature with ninjutsu is put onto the battlefield unblocked. It will be attacking the same player, planeswalker, or battle as the creature that was returned to its owner’s hand.

702.11a Hexproof is a static ability.

702.11b “Hexproof” on a permanent means “This permanent can’t be the target of spells or abilities your opponents control.”

509.1h An attacking creature with one or more creatures declared as blockers for it becomes a blocked creature; one with no creatures declared as blockers for it becomes an unblocked creature. This remains unchanged until the creature is removed from combat, an effect says that it becomes blocked or unblocked, or the combat phase ends, whichever comes first. A creature remains blocked even if all the creatures blocking it are removed from combat.

702.138a Escape represents a static ability that functions while the card with escape is in a player’s graveyard. “Escape [cost]” means “You may cast this card from your graveyard by paying [cost] rather than paying its mana cost.” Casting a spell using its escape ability follows the rules for paying alternative costs in rules 601.2b and 601.2f–h.

702.138b A spell or permanent “escaped” if that spell or the spell that became that permanent as it resolved was cast from a graveyard with an escape ability.

202.3 The mana value of an object is a number equal to the total amount of mana in its mana cost, regardless of color.

202.3e When calculating the mana value of an object with an {X} in its mana cost, X is treated as 0 while the object is not on the stack, and X is treated as the number chosen for it while the object is on the stack.

608.2b If the spell or ability specifies targets, it checks whether the targets are still legal. A target that's no longer in the zone it was in when it was targeted is illegal. Other changes to the game state may cause a target to no longer be legal; for example, its characteristics may have changed or an effect may have changed the text of the spell. If the source of an ability has left the zone it was in, its last known information is used during this process. If all its targets, for every instance of the word "target," are now illegal, the spell or ability doesn't resolve. It's removed from the stack and, if it's a spell, put into its owner's graveyard. Otherwise, the spell or ability will resolve normally. Illegal targets, if any, won't be affected by parts of a resolving spell's effect for which they're illegal. Other parts of the effect for which those targets are not illegal may still affect them. If the spell or ability creates any continuous effects that affect game rules (see rule 613.11), those effects don't apply to illegal targets. If part of the effect requires information about an illegal target, it fails to determine any such information. Any part of the effect that requires that information won't happen.

608.2h If an effect requires information from the game (such as the number of creatures on the battlefield), the answer is determined only once, when the effect is applied. If the effect requires information from a specific object, including the source of the ability itself, the effect uses the current information of that object if it’s in the public zone it was expected to be in; if it’s no longer in that zone, or if the effect has moved it from a public zone to a hidden zone, the effect uses the object’s last known information. See rule 113.7a. If an ability states that an object does something, it’s the object as it exists—or as it most recently existed—that does it, not the ability.

608.2n As the final part of an instant or sorcery spell’s resolution, the spell is put into its owner’s graveyard. As the final part of an ability’s resolution, the ability is removed from the stack and ceases to exist.

701.47a To amass [subtype] N means “If you don’t control an Army creature, create a 0/0 black [subtype] Army creature token. Choose an Army creature you control. Put N +1/+1 counters on that creature. If it isn’t a [subtype], it becomes a [subtype] in addition to its other types.”

111.7 . A token that’s in a zone other than the battlefield ceases to exist. This is a state-based action; see rule 704. (Note that if a token changes zones, applicable triggered abilities will trigger before the token ceases to exist.)

111.8 . A token that has left the battlefield can’t move to another zone or come back onto the battlefield. If such a token would change zones, it remains in its current zone instead. It ceases to exist the next time state-based actions are checked; see rule 704.

120.3a Damage dealt to a player by a source without infect causes that player to lose that much life.

704.5f If a creature has toughness 0 or less, it’s put into its owner’s graveyard. Regeneration can’t replace this event.

704.5g If a creature has toughness greater than 0, it has damage marked on it, and the total damage marked on it is greater than or equal to its toughness, that creature has been dealt lethal damage and is destroyed. Regeneration can replace this event.

701.21a To sacrifice a permanent, its controller moves it from the battlefield directly to its owner’s graveyard. A player can’t sacrifice something that isn’t a permanent, or something that’s a permanent they don’t control. Sacrificing a permanent doesn’t destroy it, so regeneration or other effects that replace destruction can’t affect this action.

702.56a Replicate is a keyword that represents two abilities. The first is a static ability that functions while the spell with replicate is on the stack. The second is a triggered ability that functions while the spell with replicate is on the stack. "Replicate [cost]" means "As an additional cost to cast this spell, you may pay [cost] any number of times" and "When you cast this spell, if a replicate cost was paid for it, copy it for each time its replicate cost was paid. If the spell has any targets, you may choose new targets for any of the copies." Paying a spell's replicate cost follows the rules for paying additional costs in rules 601.2b and 601.2f-h.

707.10c Some effects copy a spell or ability and state that its controller may choose new targets for the copy. The player may leave any number of the targets unchanged, even if those targets would be illegal. If the player chooses to change some or all of the targets, the new targets must be legal. Once the player has decided what the copy's targets will be, the copy is put onto the stack with those targets.

601.3 A player can begin to cast a spell only if a rule or effect allows that player to cast it and no rule or effect prohibits that player from casting it.

601.2f The player determines the total cost of the spell. Usually this is just the mana cost. Some spells have additional or alternative costs. Some effects may increase or reduce the cost to pay, or may provide other alternative costs. Costs may include paying mana, tapping permanents, sacrificing permanents, discarding cards, and so on. The total cost is the mana cost or alternative cost (as determined in rule 601.2b), plus all additional costs and cost increases, and minus all cost reductions. If multiple cost reductions apply, the player may apply them in any order. If the mana component of the total cost is reduced to nothing by cost reduction effects, it is considered to be {0}. It can't be reduced to less than {0}. Once the total cost is determined, any effects that directly affect the total cost are applied. Then the resulting total cost becomes "locked in." If effects would change the total cost after this time, they have no effect.

601.2h The player pays the total cost. First, they pay all costs that don’t involve random elements or moving objects from the library to a public zone, in any order. Then they pay all remaining costs in any order. Partial payments are not allowed. Unpayable costs can’t be paid.

119.4 If a cost or effect allows a player to pay an amount of life greater than 0, the player may do so only if their life total is greater than or equal to the amount of the payment. If a player pays life, the payment is subtracted from their life total; in other words, the player loses that much life.

603.6a Enters-the-battlefield abilities trigger when a permanent enters the battlefield. These are written, "When [this object] enters, . . . " or "Whenever a [type] enters, . . ." Each time an event puts one or more permanents onto the battlefield, all permanents on the battlefield (including the newcomers) are checked for any enters-the-battlefield triggers that match the event.

115.1d A triggered ability is targeted if it identifies something it will affect by using the phrase "target [something]," where the "something" is a phrase that describes an object and/or player. The target(s) are chosen as the ability is put on the stack; see rule 603.3d.

302.6 A creature’s activated ability with the tap symbol or the untap symbol in its activation cost can’t be activated unless the creature has been under its controller’s control continuously since their most recent turn began. A creature can’t attack unless it has been under its controller’s control continuously since their most recent turn began. This rule is informally called the “summoning sickness” rule.

506.4 . A permanent is removed from combat if it leaves the battlefield, if its controller changes, if it phases out, if an effect specifically removes it from combat, if it’s a planeswalker that’s being attacked and stops being a planeswalker, if it’s a battle that’s being attacked and stops being a battle, or if it’s an attacking or blocking creature that regenerates (see rule 701.19), stops being a creature, or becomes a battle. A creature that’s removed from combat stops being an attacking, blocking, blocked, and/or unblocked creature. A planeswalker or battle that’s removed from combat stops being attacked.

603.6c Leaves-the-battlefield abilities trigger when a permanent moves from the battlefield to another zone, or when a phased-in permanent leaves the game because its owner leaves the game. These are written as, but aren’t limited to, “When [this object] leaves the battlefield, . . .” or “Whenever [something] is put into a graveyard from the battlefield, . . . .” (See also rule 603.10.) An ability that attempts to do something to the card that left the battlefield checks for it only in the first zone that it went to. An ability that triggers when a card is put into a certain zone “from anywhere” is never treated as a leaves-the-battlefield ability, even if an object is put into that zone from the battlefield.

702.8a Flash is a static ability that functions in any zone from which you could play the card it’s on. “Flash” means “You may play this card any time you could cast an instant.”

701.23b If a player is searching a hidden zone for cards with a stated quality, such as a card with a certain card type or color, that player isn’t required to find some or all of those cards even if they’re present in that zone.

704.3 Whenever a player would get priority (see rule 117, "Timing and Priority"), the game checks for any of the listed conditions for state-based actions, then performs all applicable state-based actions simultaneously as a single event. If any state-based actions are performed as a result of a check, the check is repeated; otherwise all triggered abilities that are waiting to be put on the stack are put on the stack, then the check is repeated. Once no more state-based actions have been performed as the result of a check and no triggered abilities are waiting to be put on the stack, the appropriate player gets priority. This process also occurs during the cleanup step (see rule 514), except that if no state-based actions are performed as the result of the step's first check and no triggered abilities are waiting to be put on the stack, then no player gets priority and the step ends.

405.5 . When all players pass in succession, the top (last-added) spell or ability on the stack resolves. If the stack is empty when all players pass, the current step or phase ends and the next begins.

117.1a A player may cast an instant spell any time they have priority. A player may cast a noninstant spell during their main phase any time they have priority and the stack is empty.

305.6 The basic land types are Plains, Island, Swamp, Mountain, and Forest. If an object uses the words “basic land type,” it’s referring to one of these subtypes. An object with the land card type and a basic land type has the intrinsic ability “{T}: Add [mana symbol],” even if the text box doesn’t actually contain that text or the object has no text box. For Plains, [mana symbol] is {W}; for Islands, {U}; for Swamps, {B}; for Mountains, {R}; and for Forests, {G}. See rule 107.4a. See also rule 605, “Mana Abilities.”

701.25a To "surveil N" means to look at the top N cards of your library, then put any number of them into your graveyard and the rest on top of your library in any order.

202.3a The mana value of an object with no mana cost is 0, unless that object is the back face of a nonmodal double-faced permanent or spell, or it is a melded permanent.

603.3a A triggered ability is controlled by the player who controlled its source at the time it triggered, unless it's a delayed triggered ability. To determine the controller of a delayed triggered ability, see rules 603.7d-f.

117.3a The active player receives priority at the beginning of most steps and phases, after any turn-based actions (such as drawing a card during the draw step; see rule 703) have been dealt with and abilities that trigger at the beginning of that phase or step have been put on the stack. No player receives priority during the untap step. Players usually don't get priority during the cleanup step (see rule 514.3).

111.4 A spell or ability that creates a token sets both its name and its subtype(s). If the spell or ability doesn’t specify the name of the token, its name is the same as its subtype(s) plus the word “Token.” Once a token is on the battlefield, changing its name doesn’t change its subtype(s), and vice versa.

306.5b A planeswalker has the intrinsic ability “This permanent enters with a number of loyalty counters on it equal to its printed loyalty number.” This ability creates a replacement effect (see rule 614.1c).

306.5c The loyalty of a planeswalker on the battlefield is equal to the number of loyalty counters on it.

702.2b A creature with toughness greater than 0 that's been dealt damage by a source with deathtouch since the last time state-based actions were checked is destroyed as a state-based action. See rule 704.

509.1a The defending player chooses which creatures they control, if any, will block. The chosen creatures must be untapped and they can't also be battles. For each of the chosen creatures, the defending player chooses one creature for it to block that's attacking that player, a planeswalker they control, or a battle they protect.

508.1a The active player chooses which creatures that they control, if any, will attack. The chosen creatures must be untapped, they can't also be battles, and each one must either have haste or have been controlled by the active player continuously since the turn began.

105.2 An object can be one or more of the five colors, or it can be no color at all. An object is the color or colors of the mana symbols in its mana cost, regardless of the color of its frame. An object's color or colors may also be defined by a color indicator or a characteristic-defining ability. See rule 202.2.

202.2 . An object is the color or colors of the mana symbols in its mana cost, regardless of the color of its frame.

509.1h An attacking creature with one or more creatures declared as blockers for it becomes a blocked creature; one with no creatures declared as blockers for it becomes an unblocked creature. This remains unchanged until the creature is removed from combat, an effect says that it becomes blocked or unblocked, or the combat phase ends, whichever comes first. A creature remains blocked even if all the creatures blocking it are removed from combat.

117.1b A player may activate an activated ability any time they have priority.

605.3a A player may activate an activated mana ability whenever they have priority, whenever they are casting a spell or activating an ability that requires a mana payment, or whenever a rule or effect asks for a mana payment, even if it’s in the middle of casting or resolving a spell or activating or resolving an ability.

## NEW: retrieved from Savecraft rules_search 2026-10-01

121.6 . Some effects replace card draws.

121.6a An effect that replaces a card draw is applied even if no cards could be drawn because there are no cards in the affected player’s library.

121.6b If an effect replaces a draw within a sequence of card draws, the replacement effect is completed before resuming the sequence.

614.11 . Some effects replace card draws. These effects are applied even if no cards could be drawn because there are no cards in the affected player’s library.

614.11a If an effect replaces a draw within a sequence of card draws, all actions required by the replacement are completed, if possible, before resuming the sequence.

702.49b The card with ninjutsu remains revealed from the time the ability is announced until the ability leaves the stack.

702.11c “Hexproof” on a player means “You can’t be the target of spells or abilities your opponents control.”

601.2b If the spell is modal, the player announces the mode choice (see rule 700.2). If the player wishes to splice any cards onto the spell (see rule 702.47), they reveal those cards in their hand. If the spell has alternative or additional costs that will be paid as it’s being cast such as buyback or kicker costs (see rules 118.8 and 118.9), the player announces their intentions to pay any or all of those costs (see rule 601.2f). A player can’t apply two alternative methods of casting or two alternative costs to a single spell. If the spell has a variable cost that will be paid as it’s being cast (such as an {X} in its mana cost; see rule 107.3), the player announces the value of that variable. If the value of that variable is defined in the text of the spell by a choice that player would make later in the announcement or resolution of the spell, that player makes that choice at this time instead of that later time. If a cost that will be paid as the spell is being cast includes hybrid mana symbols, the player announces the nonhybrid equivalent cost they intend to pay. If a cost that will be paid as the spell is being cast includes Phyrexian mana symbols, the player announces whether they intend to pay 2 life or a corresponding colored mana cost for each of those symbols. Previously made choices (such as choosing to cast a spell with flashback from a graveyard or choosing to cast a creature with morph face down) may restrict the player’s options when making these choices.
