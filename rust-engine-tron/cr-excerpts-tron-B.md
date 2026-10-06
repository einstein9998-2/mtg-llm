# CR excerpts retrieved for the Colorless Tron group B scenarios

Retrieved 2026-10-06 from the Savecraft `rules_search` module (MTG Comprehensive Rules, effective November 14, 2025) in this task, or copied verbatim from the already pinned `reference/cr-excerpts*.md` files of the spec project (same source). One rule per paragraph; examples omitted. Copy into `rust-engine-spec/reference/` unchanged so `verified: true` citations validate.

111.7 A token that’s in a zone other than the battlefield ceases to exist. This is a state-based action; see rule 704. (Note that if a token changes zones, applicable triggered abilities will trigger before the token ceases to exist.)

113.7a Once activated or triggered, an ability exists on the stack independently of its source. Destruction or removal of the source after that time won’t affect the ability. Note that some abilities cause a source to do something (for example, “This creature deals 1 damage to any target”) rather than the ability doing anything directly. In these cases, any activated or triggered ability that references information about the source for use while announcing an activated ability or putting a triggered ability on the stack checks that information when the ability is put onto the stack. Otherwise, it will check that information when it resolves. In both instances, if the source is no longer in the zone it’s expected to be in at that time, its last known information is used. The source can still perform the action even though it no longer exists.

117.1a A player may cast an instant spell any time they have priority. A player may cast a noninstant spell during their main phase any time they have priority and the stack is empty.

117.3c If a player has priority when they cast a spell, activate an ability, or take a special action, that player receives priority afterward.

202.3a The mana value of an object with no mana cost is 0, unless that object is the back face of a nonmodal double-faced permanent or spell, or it is a melded permanent.

205.1b Some effects change an object’s card type, supertype, or subtype but specify that the object retains a prior card type, supertype, or subtype. In such cases, all the object’s prior card types, supertypes, and subtypes are retained. This rule applies to effects that use phrases such as “in addition to its other types” or that state that something is “still a [type, supertype, or subtype].” Some effects state that an object becomes an “artifact creature”; these effects also allow the object to retain all of its prior card types and subtypes. Some effects state that an object becomes a “[creature type or types] artifact creature”; these effects also allow the object to retain all of its prior card types and subtypes other than creature types, but replace any existing creature types.

302.6 A creature’s activated ability with the tap symbol or the untap symbol in its activation cost can’t be activated unless the creature has been under its controller’s control continuously since their most recent turn began. A creature can’t attack unless it has been under its controller’s control continuously since their most recent turn began. This rule is informally called the “summoning sickness” rule.

305.2 . A player can normally play one land during their turn; however, continuous effects may increase this number.

305.2a To determine whether a player can play a land, compare the number of lands the player can play this turn with the number of lands they have already played this turn (including lands played as special actions and lands played during the resolution of spells and abilities). If the number of lands the player can play is greater, the play is legal.

400.7 An object that moves from one zone to another becomes a new object with no memory of, or relation to, its previous existence. This rule has the following exceptions.

508.1c The active player checks each creature they control to see whether it’s affected by any restrictions (effects that say a creature can’t attack, or that it can’t attack unless some condition is met). If any restrictions are being disobeyed, the declaration of attackers is illegal.

509.1b The defending player checks each creature they control to see whether it’s affected by any restrictions (effects that say a creature can’t block, or that it can’t block unless some condition is met). If any restrictions are being disobeyed, the declaration of blockers is illegal.

514.2 . Second, the following actions happen simultaneously: all damage marked on permanents (including phased-out permanents) is removed and all “until end of turn” and “this turn” effects end. This turn-based action doesn’t use the stack.

602.2a The player announces that they are activating the ability. If an activated ability is being activated from a hidden zone, the card that has that ability is revealed (see rule 701.20a). That ability is created on the stack as an object that’s not a card. It becomes the topmost object on the stack. It has the text of the ability that created it, and no other characteristics. Its controller is the player who activated the ability. The ability remains on the stack until it’s countered, it resolves, or an effect moves it elsewhere.

603.2 . Whenever a game event or game state matches a triggered ability's trigger event, that ability automatically triggers. The ability doesn't do anything at this point.

603.3d The remainder of the process for putting a triggered ability on the stack is identical to the process for casting a spell listed in rules 601.2c–d. If a choice is required when the triggered ability goes on the stack but no legal choices can be made for it, or if a rule or a continuous effect otherwise makes the ability illegal, the ability is simply removed from the stack.

603.6a Enters-the-battlefield abilities trigger when a permanent enters the battlefield. These are written, “When [this object] enters, . . . “ or “Whenever a [type] enters, . . .” Each time an event puts one or more permanents onto the battlefield, all permanents on the battlefield (including the newcomers) are checked for any enters-the-battlefield triggers that match the event.

605.1a An activated ability is a mana ability if it meets all of the following criteria: it doesn’t require a target (see rule 115.6), it could add mana to a player’s mana pool when it resolves, and it’s not a loyalty ability. (See rule 606, “Loyalty Abilities.”)

606.1 Some activated abilities are loyalty abilities, which are subject to special rules.

608.2b If the spell or ability specifies targets, it checks whether the targets are still legal. A target that’s no longer in the zone it was in when it was targeted is illegal. Other changes to the game state may cause a target to no longer be legal; for example, its characteristics may have changed or an effect may have changed the text of the spell. If the source of an ability has left the zone it was in, its last known information is used during this process. If all its targets, for every instance of the word “target,” are now illegal, the spell or ability doesn’t resolve. It’s removed from the stack and, if it’s a spell, put into its owner’s graveyard. Otherwise, the spell or ability will resolve normally. Illegal targets, if any, won’t be affected by parts of a resolving spell’s effect for which they’re illegal. Other parts of the effect for which those targets are not illegal may still affect them. If the spell or ability creates any continuous effects that affect game rules (see rule 613.11), those effects don’t apply to illegal targets. If part of the effect requires information about an illegal target, it fails to determine any such information. Any part of the effect that requires that information won’t happen.

610.3 Some one-shot effects cause an object to change zones “until” a specified event occurs. A second one-shot effect is created immediately after the specified event. This second one-shot effect returns the object to its previous zone.

610.3b If a resolving triggered ability creates the initial one-shot effect that causes the object to change zones, and the specified event has already occurred before that one-shot effect would occur but after that ability triggered, the object doesn’t move.

610.3c An object returned to the battlefield this way returns under its owner’s control unless otherwise specified.

613.1d Layer 4: Type-changing effects are applied. These include effects that change an object's card type, subtype, and/or supertype.

701.6a To counter a spell or ability means to cancel it, removing it from the stack. It doesn’t resolve and none of its effects occur. A countered spell is put into its owner’s graveyard.

701.8a To destroy a permanent, move it from the battlefield to its owner’s graveyard.

701.21a To sacrifice a permanent, its controller moves it from the battlefield directly to its owner’s graveyard. A player can’t sacrifice something that isn’t a permanent, or something that’s a permanent they don’t control. Sacrificing a permanent doesn’t destroy it, so regeneration or other effects that replace destruction can’t affect this action.

701.23a To search for a card in a zone, look at all cards in that zone (even if it’s a hidden zone) and find a card that matches the given description.

701.23b If a player is searching a hidden zone for cards with a stated quality, such as a card with a certain card type or color, that player isn’t required to find some or all of those cards even if they’re present in that zone.

702.29a Cycling is an activated ability that functions only while the card with cycling is in a player’s hand. “Cycling [cost]” means “[Cost], Discard this card: Draw a card.”

704.5g If a creature has toughness greater than 0, it has damage marked on it, and the total damage marked on it is greater than or equal to its toughness, that creature has been dealt lethal damage and is destroyed. Regeneration can replace this event.
