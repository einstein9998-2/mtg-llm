# CR excerpts for batch W14 (matchup 2: Boros Aggro vs BW Death and Taxes)

Rules under 'Retrieved' were retrieved 2026-10-01 from the Savecraft `rules_search` module (MTG Comprehensive Rules effective November 14, 2025) and copied verbatim (examples omitted). Rules under 'Copied' are copied verbatim from earlier batches' excerpt files in reference/ (source file named after each rule), which were retrieved from the same module.

## Retrieved

603.12 A resolving spell or ability may allow or instruct a player to take an action and create a triggered ability that triggers “when [a player] [does or doesn’t]” take that action or “when [something happens] this way.” These reflexive triggered abilities follow the rules for delayed triggered abilities (see rule 603.7), except that they’re checked immediately after being created and trigger based on whether the trigger event or events occurred earlier during the resolution of the spell or ability that created them.

603.12a Normally, if the trigger event or events occur multiple times during the resolution of the spell or ability that created it, the reflexive triggered ability will trigger once for each of those times. However, if a resolving spell or ability includes a choice to pay a cost “any number of times” and creates a triggered ability that triggers “when [a player] pays [that cost] one or more times,” paying that cost one or more times causes the reflexive triggered ability to trigger only once.

702.138c An ability that reads “[This permanent] escapes with [one or more of a kind of counter]” means “If this permanent escaped, it enters with [those counters]” That ability may have a triggered ability linked to it that triggers “When it enters this way.” (See rule 603.11.) Such a triggered ability triggers when that permanent enters the battlefield after its replacement effect was applied, even if that replacement effect had no effect.

113.7a Once activated or triggered, an ability exists on the stack independently of its source. Destruction or removal of the source after that time won’t affect the ability. Note that some abilities cause a source to do something (for example, “This creature deals 1 damage to any target”) rather than the ability doing anything directly. In these cases, any activated or triggered ability that references information about the source for use while announcing an activated ability or putting a triggered ability on the stack checks that information when the ability is put onto the stack. Otherwise, it will check that information when it resolves. In both instances, if the source is no longer in the zone it’s expected to be in at that time, its last known information is used. The source can still perform the action even though it no longer exists.

608.2b If the spell or ability specifies targets, it checks whether the targets are still legal. A target that’s no longer in the zone it was in when it was targeted is illegal. Other changes to the game state may cause a target to no longer be legal; for example, its characteristics may have changed or an effect may have changed the text of the spell. If the source of an ability has left the zone it was in, its last known information is used during this process. If all its targets, for every instance of the word “target,” are now illegal, the spell or ability doesn’t resolve. It’s removed from the stack and, if it’s a spell, put into its owner’s graveyard. Otherwise, the spell or ability will resolve normally. Illegal targets, if any, won’t be affected by parts of a resolving spell’s effect for which they’re illegal. Other parts of the effect for which those targets are not illegal may still affect them. If the spell or ability creates any continuous effects that affect game rules (see rule 613.11), those effects don’t apply to illegal targets. If part of the effect requires information about an illegal target, it fails to determine any such information. Any part of the effect that requires that information won’t happen.

510.4 If at least one attacking or blocking creature has first strike (see rule 702.7) or double strike (see rule 702.4) as the combat damage step begins, the only creatures that assign combat damage in that step are those with first strike or double strike. After that step, instead of proceeding to the end of combat step, the phase gets a second combat damage step. The only creatures that assign combat damage in that step are the remaining attackers and blockers that had neither first strike nor double strike as the first combat damage step began, as well as the remaining attackers and blockers that currently have double strike. After that step, the phase proceeds to the end of combat step.

702.15b Damage dealt by a source with lifelink causes that source’s controller, or its owner if it has no controller, to gain that much life (in addition to any other results that damage causes). See rule 120.3.

702.2b A creature with toughness greater than 0 that’s been dealt damage by a source with deathtouch since the last time state-based actions were checked is destroyed as a state-based action. See rule 704.

704.5a If a player has 0 or less life, that player loses the game.

707.2 When copying an object, the copy acquires the copiable values of the original object’s characteristics and, for an object on the stack, choices made when casting or activating it (mode, targets, the value of X, whether it was kicked, how it will affect multiple targets, and so on). The copiable values are the values derived from the text printed on the object (that text being name, mana cost, color indicator, card type, subtype, supertype, rules text, power, toughness, and/or loyalty), as modified by other copy effects, by its face-down status, and by “as . . . enters” and “as . . . is turned face up” abilities that set power and toughness (and may also set additional characteristics). Other effects (including type-changing and text-changing effects), status, counters, and stickers are not copied.

707.2a A copy acquires the color of the object it’s copying because that value is derived from its mana cost or color indicator. A copy acquires the abilities of the object it’s copying because those values are derived from its rules text. A copy doesn’t wind up with two values of each ability (that is, it doesn’t copy the object’s abilities and its rules text, then have that rules text define a new set of abilities).

603.7e If an activated or triggered ability creates a delayed triggered ability, the source of that delayed triggered ability is the same as the source of that other ability. The controller of that delayed triggered ability is the player who controlled that other ability as it resolved.

108.4a If anything asks for the controller of a card that doesn’t have one (because it’s not a permanent or spell), use its owner instead.
(added by reviewer, retrieved 2026-10-01 from the Savecraft rules_search module)

122.1b A keyword counter on a permanent or on a card in a zone other than the battlefield causes that object to gain that keyword. The keywords that a keyword counter can be are flying, first strike, double strike, deathtouch, decayed, exalted, haste, hexproof, indestructible, lifelink, menace, reach, shadow, trample, and vigilance, as well as any variants of those keywords. See rule 613.1f.
(added by reviewer, retrieved 2026-10-01 from the Savecraft rules_search module)

## Copied from earlier excerpt files

101.2 When a rule or effect allows or directs something to happen, and another effect states that it can’t happen, the “can’t” effect takes precedence.
(copied from cr-excerpts-w12-w12.md)

117.3b The active player receives priority after a spell or ability (other than a mana ability) resolves.
(copied from cr-excerpts-w10-.md)

117.3c If a player has priority when they cast a spell, activate an ability, or take a special action, that player receives priority afterward.
(copied from cr-excerpts-w10-.md)

117.4 If all players pass in succession (that is, if all players pass without taking any actions in between passing), the spell or ability on top of the stack resolves or, if the stack is empty, the phase or step ends.
(copied from cr-excerpts-w1a-w1a.md)

111.1 Some effects put tokens onto the battlefield. A token is a marker used to represent any permanent that isn't represented by a card.
(copied from cr-excerpts-w7-w7.md)

111.7 A token that’s in a zone other than the battlefield ceases to exist. This is a state-based action; see rule 704. (Note that if a token changes zones, applicable triggered abilities will trigger before the token ceases to exist.)
(copied from cr-excerpts-w11-w11.md)

115.4 Some spells and abilities that refer to damage require “any target,” “another target,” “two targets,” or similar rather than “target [something].” These targets may be creatures, players, planeswalkers, or battles. Other game objects, such as noncreature artifacts or spells, can’t be chosen.
(copied from cr-excerpts-w10-w10.md)

120.3a Damage dealt to a player by a source without infect causes that player to lose that much life.
(copied from cr-excerpts-w10-w10.md)

202.3 The mana value of an object is a number equal to the total amount of mana in its mana cost, regardless of color.
(copied from cr-excerpts-w10-w10.md)

302.6 A creature’s activated ability with the tap symbol or the untap symbol in its activation cost can’t be activated unless the creature has been under its controller’s control continuously since their most recent turn began. A creature can’t attack unless it has been under its controller’s control continuously since their most recent turn began. This rule is informally called the “summoning sickness” rule.
(copied from cr-excerpts-w10-w10.md)

307.1 A player who has priority may cast a sorcery card from their hand during a main phase of their turn when the stack is empty. Casting a sorcery as a spell uses the stack. (See rule 601, “Casting Spells.”)
(copied from cr-excerpts-w10-w10-review.md)

400.7 An object that moves from one zone to another becomes a new object with no memory of, or relation to, its previous existence. This rule has the following exceptions.
(copied from cr-excerpts-w11-w11.md)

506.4 A permanent is removed from combat if it leaves the battlefield, if its controller changes, if it phases out, if an effect specifically removes it from combat, if it’s a planeswalker that’s being attacked and stops being a planeswalker, if it’s a battle that’s being attacked and stops being a battle, or if it’s an attacking or blocking creature that regenerates (see rule 701.19), stops being a creature, or becomes a battle. A creature that’s removed from combat stops being an attacking, blocking, blocked, and/or unblocked creature. A planeswalker or battle that’s removed from combat stops being attacked.
(copied from cr-excerpts-w11-w11.md)

508.3d An ability that reads "Whenever [a player] attacks, . . ." triggers if one or more creatures that player controls are declared as attackers.
(copied from cr-excerpts-w8-w8.md)

508.8 If no creatures are declared as attackers or put onto the battlefield attacking, skip the declare blockers and combat damage steps.
(copied from cr-excerpts-w1a-w1a.md)

509.1a The defending player chooses which creatures they control, if any, will block. The chosen creatures must be untapped and they can't also be battles. For each of the chosen creatures, the defending player chooses one creature for it to block that's attacking that player, a planeswalker they control, or a battle they protect.
(copied from cr-excerpts-w2b-w2b.md)

510.2 Second, all combat damage that's been assigned is dealt simultaneously. This turn-based action doesn't use the stack. No player has the chance to cast spells or activate abilities between the time combat damage is assigned and the time it's dealt.
(copied from cr-excerpts-w2b-w2b.md)

513.1 The end step has no turn-based actions. Once it begins, the active player gets priority. (See rule 117, "Timing and Priority.")
(copied from cr-excerpts-w1a-w1a.md)

601.2f The player determines the total cost of the spell. Usually this is just the mana cost. Some spells have additional or alternative costs. Some effects may increase or reduce the cost to pay, or may provide other alternative costs. Costs may include paying mana, tapping permanents, sacrificing permanents, discarding cards, and so on. The total cost is the mana cost or alternative cost (as determined in rule 601.2b), plus all additional costs and cost increases, and minus all cost reductions. If multiple cost reductions apply, the player may apply them in any order. If the mana component of the total cost is reduced to nothing by cost reduction effects, it is considered to be {0}. It can't be reduced to less than {0}. Once the total cost is determined, any effects that directly affect the total cost are applied. Then the resulting total cost becomes "locked in." If effects would change the total cost after this time, they have no effect.
(copied from cr-excerpts-w10-.md)

601.3 A player can begin to cast a spell only if a rule or effect allows that player to cast it and no rule or effect prohibits that player from casting it.
(copied from cr-excerpts-w10-w10-review.md)

602.5 A player can’t begin to activate an ability that’s prohibited from being activated.
(copied from cr-excerpts-w4-w4.md)

603.2 Whenever a game event or game state matches a triggered ability's trigger event, that ability automatically triggers. The ability doesn't do anything at this point.
(copied from cr-excerpts-w1a-w1a.md)

603.3b If multiple abilities have triggered since the last time a player received priority, the abilities are placed on the stack in a two-part process. First, each player, in APNAP order, puts each triggered ability they control with a trigger condition that isn't another ability triggering on the stack in any order they choose. (See rule 101.4.) Second, each player, in APNAP order, puts all remaining triggered abilities they control on the stack in any order they choose. Then the game once again checks for and performs state-based actions until none are performed, then abilities that triggered during this process go on the stack. This process repeats until no new state-based actions are performed and no abilities trigger. Then the appropriate player gets priority.
(copied from cr-excerpts-w10-.md)

603.4 A triggered ability may read "When/Whenever/At [trigger event], if [condition], [effect]." When the trigger event occurs, the ability checks whether the stated condition is true. The ability triggers only if it is; otherwise it does nothing. If the ability triggers, it checks the stated condition again as it resolves. If the condition isn't true at that time, the ability is removed from the stack and does nothing. Note that this mirrors the check for legal targets.
(copied from cr-excerpts-w10-.md)

603.6a Enters-the-battlefield abilities trigger when a permanent enters the battlefield. These are written, "When [this object] enters, . . . " or "Whenever a [type] enters, . . ." Each time an event puts one or more permanents onto the battlefield, all permanents on the battlefield (including the newcomers) are checked for any enters-the-battlefield triggers that match the event.
(copied from cr-excerpts-w2a-w2a.md)

603.6c Leaves-the-battlefield abilities trigger when a permanent moves from the battlefield to another zone, or when a phased-in permanent leaves the game because its owner leaves the game. These are written as, but aren’t limited to, “When [this object] leaves the battlefield, . . .” or “Whenever [something] is put into a graveyard from the battlefield, . . . .” (See also rule 603.10.) An ability that attempts to do something to the card that left the battlefield checks for it only in the first zone that it went to. An ability that triggers when a card is put into a certain zone “from anywhere” is never treated as a leaves-the-battlefield ability, even if an object is put into that zone from the battlefield.
(copied from cr-excerpts-w10-w10.md)

603.7a Delayed triggered abilities are created during the resolution of spells or abilities, as the result of a replacement effect being applied, or as a result of a static ability that allows a player to take an action. A delayed triggered ability won’t trigger until it has actually been created, even if its trigger event occurred just beforehand. Other events that happen earlier may make the trigger event impossible.
(copied from cr-excerpts-w11-w11.md)

603.7b A delayed triggered ability will trigger only once—the next time its trigger event occurs—unless it has a stated duration, such as “this turn.” If its trigger event occurs more than once simultaneously and the ability doesn’t have a stated duration, the controller of the delayed triggered ability chooses which event causes the ability to trigger.
(copied from cr-excerpts-w11-w11.md)

603.7c A delayed triggered ability that refers to a particular object still affects it even if the object changes characteristics. However, if that object is no longer in the zone it’s expected to be in at the time the delayed triggered ability resolves, the ability won’t affect it. (Note that if that object left that zone and then returned, it’s a new object and thus won’t be affected. See rule 400.7.)
(copied from cr-excerpts-w11-w11.md)

603.10a Some zone-change triggers look back in time. These are leaves-the-battlefield abilities, abilities that trigger when a card leaves a graveyard, and abilities that trigger when an object that all players can see is put into a hand or library.
(copied from cr-excerpts-w10-w10.md)

605.1a An activated ability is a mana ability if it meets all of the following criteria: it doesn’t require a target (see rule 115.6), it could add mana to a player’s mana pool when it resolves, and it’s not a loyalty ability. (See rule 606, “Loyalty Abilities.”)
(copied from cr-excerpts-w12-w12.md)

605.3a A player may activate an activated mana ability whenever they have priority, whenever they are casting a spell or activating an ability that requires a mana payment, or whenever a rule or effect asks for a mana payment, even if it’s in the middle of casting or resolving a spell or activating or resolving an ability.
(copied from cr-excerpts-w12-w12.md)

608.2h If an effect requires information from the game (such as the number of creatures on the battlefield), the answer is determined only once, when the effect is applied. If the effect requires information from a specific object, including the source of the ability itself, the effect uses the current information of that object if it’s in the public zone it was expected to be in; if it’s no longer in that zone, or if the effect has moved it from a public zone to a hidden zone, the effect uses the object’s last known information. See rule 113.7a. If an ability states that an object does something, it’s the object as it exists—or as it most recently existed—that does it, not the ability.
(copied from cr-excerpts-w11-w11.md)

610.3 Some one-shot effects cause an object to change zones “until” a specified event occurs. A second one-shot effect is created immediately after the specified event. This second one-shot effect returns the object to its previous zone.
(copied from cr-excerpts-w9-w9.md)

610.3c An object returned to the battlefield this way returns under its owner’s control unless otherwise specified.
(copied from cr-excerpts-w11-w11.md)

614.1a Effects that use the word “instead” are replacement effects. Most replacement effects use the word “instead” to indicate what events will be replaced with other events.
(copied from cr-excerpts-w11-w11.md)

614.6 If an event is replaced, it never happens. A modified event occurs instead, which may in turn trigger abilities. Note that the modified event may contain instructions that can’t be carried out, in which case the impossible instruction is simply ignored.
(copied from cr-excerpts-w13-w13.md)

701.21a To sacrifice a permanent, its controller moves it from the battlefield directly to its owner’s graveyard. A player can’t sacrifice something that isn’t a permanent, or something that’s a permanent they don’t control. Sacrificing a permanent doesn’t destroy it, so regeneration or other effects that replace destruction can’t affect this action.
(copied from cr-excerpts-w11-w11.md)

702.7b If at least one attacking or blocking creature has first strike or double strike (see rule 702.4) as the combat damage step begins, the only creatures that assign combat damage in that step are those with first strike or double strike. After that step, instead of proceeding to the end of combat step, the phase gets a second combat damage step. The only creatures that assign combat damage in that step are the remaining attackers and blockers that had neither first strike nor double strike as the first combat damage step began, as well as the remaining attackers and blockers that currently have double strike. After that step, the phase proceeds to the end of combat step.
(copied from cr-excerpts-w8-w8.md)

702.8a Flash is a static ability that functions in any zone from which you could play the card it’s on. “Flash” means “You may play this card any time you could cast an instant.”
(copied from cr-excerpts-w10-w10.md)

702.74a Evoke represents two abilities: a static ability that functions in any zone from which the card with evoke can be cast and a triggered ability that functions on the battlefield. “Evoke [cost]” means “You may cast this card by paying [cost] rather than paying its mana cost” and “When this permanent enters, if its evoke cost was paid, its controller sacrifices it.” Casting a spell for its evoke cost follows the rules for paying alternative costs in rules 601.2b and 601.2f–h.
(copied from cr-excerpts-w11-w11.md)

702.131b Ascend on a permanent represents a static ability. It means "Any time you control ten or more permanents and you don't have the city's blessing, you get the city's blessing for the rest of the game."
(copied from cr-excerpts-w8-w8.md)

702.138a Escape represents a static ability that functions while the card with escape is in a player’s graveyard. “Escape [cost]” means “You may cast this card from your graveyard by paying [cost] rather than paying its mana cost.” Casting a spell using its escape ability follows the rules for paying alternative costs in rules 601.2b and 601.2f–h.
(copied from cr-excerpts-w10-w10.md)

702.138b A spell or permanent “escaped” if that spell or the spell that became that permanent as it resolved was cast from a graveyard with an escape ability.
(copied from cr-excerpts-w10-w10.md)

702.176a Impending is a keyword that represents four abilities. The first is a static ability that functions while the spell with impending is on the stack. The second is static ability that creates a replacement effect that may apply to the permanent with impending as it enters the battlefield from the stack. The third is a static ability that functions on the battlefield. The fourth is a triggered ability that functions on the battlefield. “Impending N—[cost]” means “You may choose to pay [cost] rather than pay this spell’s mana cost,” “If you chose to pay this permanent’s impending cost, it enters with N time counters on it,” “As long as this permanent’s impending cost was paid and it has a time counter on it, it’s not a creature,” and “At the beginning of your end step, if this permanent’s impending cost was paid and it has a time counter on it, remove a time counter from it.” Casting a spell for its impending cost follows the rules for paying alternative costs in rules 601.2b and 601.2f–h.
(copied from cr-excerpts-w9-w9.md)

702.181a Mobilize is a triggered ability. "Mobilize N" means "Whenever this creature attacks, create N 1/1 red Warrior creature tokens. Those tokens enter tapped and attacking. Sacrifice them at the beginning of the next end step."
(copied from cr-excerpts-w8-w8.md)

704.5d If a token is in a zone other than the battlefield, it ceases to exist.
(copied from cr-excerpts-w11-w11.md)

704.5g If a creature has toughness greater than 0, it has damage marked on it, and the total damage marked on it is greater than or equal to its toughness, that creature has been dealt lethal damage and is destroyed. Regeneration can replace this event.
(copied from cr-excerpts-w10-w10.md)

707.9b Some copy effects modify a characteristic as part of the copying process. The final set of values for that characteristic becomes part of the copiable values of the copy.
(copied from cr-excerpts-w8-w8.md)

117.3a The active player receives priority at the beginning of most steps and phases, after any turn-based actions (such as drawing a card during the draw step; see rule 703) have been dealt with and abilities that trigger at the beginning of that phase or step have been put on the stack. No player receives priority during the untap step. Players usually don't get priority during the cleanup step (see rule 514.3).
(copied from cr-excerpts-w10-.md)

118.3 A player can’t pay a cost without having the necessary resources to pay it fully. For example, a player with only 1 life can’t pay a cost of 2 life, and a permanent that’s already tapped can’t be tapped to pay a cost. See rule 202, “Mana Cost and Color,” and rule 602, “Activating Activated Abilities.”
(copied from cr-excerpts-w12-w12.md)

