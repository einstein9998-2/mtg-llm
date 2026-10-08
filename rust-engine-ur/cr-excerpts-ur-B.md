# CR excerpts retrieved for the UR alternate (group B) scenarios

Retrieved 2026-10-08 from the Savecraft `rules_search` module, "MTG Comprehensive Rules (effective November 14, 2025)". Text copied verbatim (one rule per paragraph; examples omitted). Rules 105.2, 105.2b, 120.3a, 120.3e, 205.4a, 205.4c, 400.3, 601.2d, 608.2b (first sentences used), 716.2b and 716.2d were retrieved directly by `rule` lookup on 2026-10-08; the others were already present, with the same wording, in the earlier excerpt files (rust-engine-spec/reference/cr-excerpts*.md and the Tron/Storm packages), which came from the same module. Append this file to reference/ (as cr-excerpts-ur-B.md) so `verified: true` citations validate.

104.4a If all the players remaining in a game lose simultaneously, the game is a draw.

105.2 An object can be one or more of the five colors, or it can be no color at all. An object is the color or colors of the mana symbols in its mana cost, regardless of the color of its frame. An object’s color or colors may also be defined by a color indicator or a characteristic-defining ability. See rule 202.2.

105.2b A multicolored object is two or more of the five colors.

105.2c A colorless object has no color.

111.7 A token that’s in a zone other than the battlefield ceases to exist. This is a state-based action; see rule 704. (Note that if a token changes zones, applicable triggered abilities will trigger before the token ceases to exist.)

111.8 A token that has left the battlefield can’t move to another zone or come back onto the battlefield. If such a token would change zones, it remains in its current zone instead. It ceases to exist the next time state-based actions are checked; see rule 704.

113.7a Once activated or triggered, an ability exists on the stack independently of its source. Destruction or removal of the source after that time won’t affect the ability. Note that some abilities cause a source to do something (for example, “This creature deals 1 damage to any target”) rather than the ability doing anything directly. In these cases, any activated or triggered ability that references information about the source for use while announcing an activated ability or putting a triggered ability on the stack checks that information when the ability is put onto the stack. Otherwise, it will check that information when it resolves. In both instances, if the source is no longer in the zone it’s expected to be in at that time, its last known information is used. The source can still perform the action even though it no longer exists.

117.3b The active player receives priority after a spell or ability (other than a mana ability) resolves.

118.12a Some spells, activated abilities, and triggered abilities read, “[Do something] unless [a player does something else].” This means the same thing as “[A player may do something else]. If [that player] doesn’t, [do something].”

118.9 Some spells have alternative costs. An alternative cost is a cost listed in a spell’s text, or applied to it from another effect, that its controller may pay rather than paying the spell’s mana cost. Alternative costs are usually phrased, “You may [action] rather than pay [this object’s] mana cost,” or “You may cast [this object] without paying its mana cost.” Note that some alternative costs are listed in keywords; see rule 702.

118.9a Only one alternative cost can be applied to any one spell as it’s being cast. The controller of the spell announces their intentions to pay that cost as described in rule 601.2b.

120.3a Damage dealt to a player by a source without infect causes that player to lose that much life.

120.3e Damage dealt to a creature by a source with neither wither nor infect causes that much damage to be marked on that creature.

121.1 A player draws a card by putting the top card of their library into their hand. This is done as a turn-based action during each player’s draw step. It may also be done as part of a cost or effect of a spell or ability.

205.4c Any land with the supertype “basic” is a basic land. Any land that doesn’t have this supertype is a nonbasic land, even if it has a basic land type.

400.3 If an object would go to any library, graveyard, or hand other than its owner’s, it goes to its owner’s corresponding zone.

400.7 An object that moves from one zone to another becomes a new object with no memory of, or relation to, its previous existence. This rule has the following exceptions.

601.2d If the spell requires the player to divide or distribute an effect (such as damage or counters) among one or more targets, the player announces the division. Each of these targets must receive at least one of whatever is being divided.

601.2h The player pays the total cost. First, they pay all costs that don’t involve random elements or moving objects from the library to a public zone, in any order. Then they pay all remaining costs in any order. Partial payments are not allowed. Unpayable costs can’t be paid.

603.3 Once an ability has triggered, its controller puts it on the stack as an object that's not a card the next time a player would receive priority. See rule 117, "Timing and Priority." The ability becomes the topmost object on the stack. It has the text of the ability that created it, and no other characteristics. It remains on the stack until it's countered, it resolves, a rule causes it to be removed from the stack, or an effect moves it elsewhere.

608.2b If the spell or ability specifies targets, it checks whether the targets are still legal. A target that’s no longer in the zone it was in when it was targeted is illegal. Other changes to the game state may cause a target to no longer be legal; for example, its characteristics may have changed or an effect may have changed the text of the spell. If the source of an ability has left the zone it was in, its last known information is used during this process. If all its targets, for every instance of the word “target,” are now illegal, the spell or ability doesn’t resolve. It’s removed from the stack and, if it’s a spell, put into its owner’s graveyard. Otherwise, the spell or ability will resolve normally. Illegal targets, if any, won’t be affected by parts of a resolving spell’s effect for which they’re illegal. Other parts of the effect for which those targets are not illegal may still affect them.

701.21a To sacrifice a permanent, its controller moves it from the battlefield directly to its owner’s graveyard. A player can’t sacrifice something that isn’t a permanent, or something that’s a permanent they don’t control. Sacrificing a permanent doesn’t destroy it, so regeneration or other effects that replace destruction can’t affect this action.

702.12b A permanent with indestructible can’t be destroyed. Such permanents aren’t destroyed by lethal damage, and they ignore the state-based action that checks for lethal damage (see rule 704.5g).

702.16b A permanent or player with protection can’t be targeted by spells with the stated quality and can’t be targeted by abilities from a source with the stated quality.

702.16e Any damage that would be dealt by sources that have the stated quality to a permanent or player with protection is prevented.

704.3 Whenever a player would get priority (see rule 117, "Timing and Priority"), the game checks for any of the listed conditions for state-based actions, then performs all applicable state-based actions simultaneously as a single event. If any state-based actions are performed as a result of a check, the check is repeated; otherwise all triggered abilities that are waiting to be put on the stack are put on the stack, then the check is repeated. Once no more state-based actions have been performed as the result of a check and no triggered abilities are waiting to be put on the stack, the appropriate player gets priority. This process also occurs during the cleanup step (see rule 514), except that if no state-based actions are performed as the result of the step's first check and no triggered abilities are waiting to be put on the stack, then no player gets priority and the step ends.

704.5a If a player has 0 or less life, that player loses the game.

704.5d If a token is in a zone other than the battlefield, it ceases to exist.

704.5g If a creature has toughness greater than 0, it has damage marked on it, and the total damage marked on it is greater than or equal to its toughness, that creature has been dealt lethal damage and is destroyed. Regeneration can replace this event.

716.2b A level is a designation that any permanent can have. A Class retains its level even if it stops being a Class. Levels are not a copiable characteristic.

716.2d If a rule or effect refers to a permanent’s level and that permanent doesn’t have a level, it is treated as though its level is 1.
