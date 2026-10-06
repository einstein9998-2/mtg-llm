# CR excerpts retrieved for the Storm group A scenarios

Retrieved 2026-10-06 from the Savecraft `rules_search` module, "MTG Comprehensive Rules (effective November 14, 2025)". Text copied verbatim (one rule per paragraph; examples omitted). Rules marked (existing) were copied unchanged from the existing `rust-engine-spec/reference/cr-excerpts*.md` files. Merge this file into `reference/` as `cr-excerpts-storm-A.md` so `verified: true` citations validate.

101.2 When a rule or effect allows or directs something to happen, and another effect states that it can’t happen, the “can’t” effect takes precedence.

101.3 Any part of an instruction that's impossible to perform is ignored. (In many cases the card will specify consequences for this; if it doesn't, there's no effect.)

101.4 If multiple players would make choices and/or take actions at the same time, the active player (the player whose turn it is) makes any choices required, then the next player in turn order (usually the player seated to the active player's left) makes any choices required, followed by the remaining nonactive players in turn order. Then the actions happen simultaneously. This rule is often referred to as the "Active Player, Nonactive Player (APNAP) order" rule.

107.1a You can’t choose a fractional number, deal fractional damage, gain fractional life, and so on. If a spell or ability could generate a fractional number, the spell or ability will tell you whether to round up or down.

107.3b If a player is casting a spell that has an {X} in its mana cost, the value of X isn’t defined by the text of that spell, and an effect lets that player cast that spell while paying neither its mana cost nor an alternative cost that includes X, then the only legal choice for X is 0. This doesn’t apply to effects that only reduce a cost, even if they reduce it to zero. See rule 601, “Casting Spells.”

111.7 A token that’s in a zone other than the battlefield ceases to exist. This is a state-based action; see rule 704. (Note that if a token changes zones, applicable triggered abilities will trigger before the token ceases to exist.)

115.1 . Some spells and abilities require their controller to choose one or more targets for them. The targets are object(s) and/or player(s) the spell or ability will affect. These targets are declared as part of the process of putting the spell or ability on the stack. The targets can’t be changed except by another spell or ability that explicitly says it can do so.

115.1a An instant or sorcery spell is targeted if its spell ability identifies something it will affect by using the phrase “target [something],” where the “something” is a phrase that describes an object and/or player. The target(s) are chosen as the spell is cast; see rule 601.2c. (If an activated or triggered ability of an instant or sorcery uses the word target, that ability is targeted, but the spell is not.)

118.6 Some objects have no mana cost. This represents an unpayable cost. An ability can also have an unpayable cost if its cost is based on the mana cost of an object with no mana cost. Attempting to cast a spell or activate an ability that has an unpayable cost is a legal action. However, attempting to pay an unpayable cost is an illegal action.

118.6a If an unpayable cost is increased by an effect or an additional cost is imposed, the cost is still unpayable. If an alternative cost is applied to an unpayable cost, including an effect that allows a player to cast a spell without paying its mana cost, the alternative cost may be paid.

118.9 Some spells have alternative costs. An alternative cost is a cost listed in a spell’s text, or applied to it from another effect, that its controller may pay rather than paying the spell’s mana cost. Alternative costs are usually phrased, “You may [action] rather than pay [this object’s] mana cost,” or “You may cast [this object] without paying its mana cost.” Note that some alternative costs are listed in keywords; see rule 702.

118.9a Only one alternative cost can be applied to any one spell as it’s being cast. The controller of the spell announces their intentions to pay that cost as described in rule 601.2b.

119.3 If an effect causes a player to gain life or lose life, that player’s life total is adjusted accordingly.

119.6 If a player has 0 or less life, that player loses the game as a state-based action. See rule 704.

202.2b Objects with no colored mana symbols in their mana costs are colorless.

202.3e When calculating the mana value of an object with an {X} in its mana cost, X is treated as 0 while the object is not on the stack, and X is treated as the number chosen for it while the object is on the stack.

305.1 A player who has priority may play a land card from their hand during a main phase of their turn when the stack is empty. Playing a land is a special action; it doesn’t use the stack (see rule 116). Rather, the player simply puts the land onto the battlefield. Since the land doesn’t go on the stack, it is never a spell, and players can’t respond to it with instants or activated abilities.

400.7 . An object that moves from one zone to another becomes a new object with no memory of, or relation to, its previous existence. This rule has the following exceptions.

400.11 An object is outside the game if it isn’t in any of the game’s zones. Outside the game is not a zone.

400.11a Cards in a player’s sideboard are outside the game. See rule 100.4.

400.11b Some effects bring cards into a game from outside the game. Those cards remain in the game until the game ends, their owner leaves the game, or a rule or effect removes them from the game, whichever comes first.

400.11c Cards outside the game can’t be affected by spells or abilities, except for characteristic-defining abilities printed on them (see rule 604.3) and spells and abilities that allow those cards to be brought into the game.

601.2a To propose the casting of a spell, a player first moves that card (or that copy of a card) from where it is to the stack. It becomes the topmost object on the stack. It has all the characteristics of the card (or the copy of a card) associated with it, and that player becomes its controller. Any continuous effects that modify the characteristics of the spell as you start casting it begin as it is put on the stack (see rule 611.2f). The spell remains on the stack until it resolves, it’s countered, or a rule or effect moves it elsewhere.

601.2b If the spell is modal, the player announces the mode choice (see rule 700.2). If the player wishes to splice any cards onto the spell (see rule 702.47), they reveal those cards in their hand. If the spell has alternative or additional costs that will be paid as it’s being cast such as buyback or kicker costs (see rules 118.8 and 118.9), the player announces their intentions to pay any or all of those costs (see rule 601.2f). A player can’t apply two alternative methods of casting or two alternative costs to a single spell.

601.2f The player determines the total cost of the spell. Usually this is just the mana cost. Some spells have additional or alternative costs. Some effects may increase or reduce the cost to pay, or may provide other alternative costs. Costs may include paying mana, tapping permanents, sacrificing permanents, discarding cards, and so on. The total cost is the mana cost or alternative cost (as determined in rule 601.2b), plus all additional costs and cost increases, and minus all cost reductions.

601.2i Once the steps described in 601.2a–h are completed, effects that modify the characteristics of the spell as it’s cast are applied, then the spell becomes cast. Any abilities that trigger when a spell is cast or put onto the stack trigger at this time. If the spell’s controller had priority before casting it, they get priority.

601.3 A player can begin to cast a spell only if a rule or effect allows that player to cast it and no rule or effect prohibits that player from casting it.

603.2 Whenever a game event or game state matches a triggered ability's trigger event, that ability automatically triggers. The ability doesn't do anything at this point.

603.3 Once an ability has triggered, its controller puts it on the stack as an object that’s not a card the next time a player would receive priority. See rule 117, “Timing and Priority.” The ability becomes the topmost object on the stack. It has the text of the ability that created it, and no other characteristics. It remains on the stack until it’s countered, it resolves, a rule causes it to be removed from the stack, or an effect moves it elsewhere.

605.1a An activated ability is a mana ability if it meets all of the following criteria: it doesn’t require a target (see rule 115.6), it could add mana to a player’s mana pool when it resolves, and it’s not a loyalty ability. (See rule 606, “Loyalty Abilities.”)

608.2b If the spell or ability specifies targets, it checks whether the targets are still legal. A target that’s no longer in the zone it was in when it was targeted is illegal. Other changes to the game state may cause a target to no longer be legal; for example, its characteristics may have changed or an effect may have changed the text of the spell. If the source of an ability has left the zone it was in, its last known information is used during this process. If all its targets, for every instance of the word “target,” are now illegal, the spell or ability doesn’t resolve. It’s removed from the stack and, if it’s a spell, put into its owner’s graveyard. Otherwise, the spell or ability will resolve normally. Illegal targets, if any, won’t be affected by parts of a resolving spell’s effect for which they’re illegal. Other parts of the effect for which those targets are not illegal may still affect them. If the spell or ability creates any continuous effects that affect game rules (see rule 613.11), those effects don’t apply to illegal targets. If part of the effect requires information about an illegal target, it fails to determine any such information. Any part of the effect that requires that information won’t happen.

608.2c The controller of the spell or ability follows its instructions in the order written. However, replacement effects may modify these actions. In some cases, later text on the card may modify the meaning of earlier text (for example, “Destroy target creature. It can’t be regenerated” or “Counter target spell. If that spell is countered this way, put it on top of its owner’s library instead of into its owner’s graveyard.”) Don’t just apply effects step by step without thinking in these cases—read the whole text and apply the rules of English to the text.

608.2g If an effect gives a player the option to pay mana, they may activate mana abilities before taking that action. If an effect specifically instructs or allows a player to cast a spell during resolution, they do so by following the steps in rules 601.2a–i, except no player receives priority after it’s cast. That spell becomes the topmost object on the stack, and the currently resolving spell or ability continues to resolve, which may include casting other spells this way. No other spells can normally be cast and no other abilities can normally be activated during resolution.

608.2h If an effect requires information from the game (such as the number of creatures on the battlefield), the answer is determined only once, when the effect is applied. If the effect requires information from a specific object, including the source of the ability itself, the effect uses the current information of that object if it’s in the public zone it was expected to be in; if it’s no longer in that zone, or if the effect has moved it from a public zone to a hidden zone, the effect uses the object’s last known information. See rule 113.7a. If an ability states that an object does something, it’s the object as it exists—or as it most recently existed—that does it, not the ability.

608.2m If an instant spell, sorcery spell, or ability that can legally resolve leaves the stack once it starts to resolve, it will continue to resolve fully.

614.1a Effects that use the word “instead” are replacement effects. Most replacement effects use the word “instead” to indicate what events will be replaced with other events.

614.6 If an event is replaced, it never happens. A modified event occurs instead, which may in turn trigger abilities. Note that the modified event may contain instructions that can’t be carried out, in which case the impossible instruction is simply ignored.

701.6a To counter a spell or ability means to cancel it, removing it from the stack. It doesn’t resolve and none of its effects occur. A countered spell is put into its owner’s graveyard.

701.9a To discard a card, move it from its owner’s hand to that player’s graveyard.

701.9b By default, effects that cause a player to discard a card allow the affected player to choose which card to discard. Some effects, however, require a random discard or allow another player to choose which card is discarded.

701.21a To sacrifice a permanent, its controller moves it from the battlefield directly to its owner’s graveyard. A player can’t sacrifice something that isn’t a permanent, or something that’s a permanent they don’t control. Sacrificing a permanent doesn’t destroy it, so regeneration or other effects that replace destruction can’t affect this action.

701.23a To search for a card in a zone, look at all cards in that zone (even if it’s a hidden zone) and find a card that matches the given description.

701.23b If a player is searching a hidden zone for cards with a stated quality, such as a card with a certain card type or color, that player isn’t required to find some or all of those cards even if they’re present in that zone.

701.23c If a player is instructed to search a hidden zone for cards that match an undefined quality, that player may still search that zone but can’t find any cards.

701.23d If a player is searching a hidden zone simply for a quantity of cards, such as “a card” or “three cards,” that player must find that many cards (or as many as possible, if the zone doesn’t contain enough cards).

701.23e If the effect that contains the search instruction doesn’t also contain instructions to reveal the found card(s), then they’re not revealed.

701.23j If an effect instructs a player to search outside the game for a card, that player may choose an appropriate card they own from outside the game.

701.24a To shuffle a library or a face-down pile of cards, randomize the cards within it so that no player knows their order.

701.24b Some effects cause a player to search a library for a card or cards, shuffle that library, then put some or all of the found cards into a different zone or in a certain position in that library. In such cases, the found cards aren’t included in the shuffle, even though they remain in the library at that time. Rather, all the cards in that library except those are shuffled. Abilities that trigger when a library is shuffled will still trigger. See also rule 401, “Library.”

702.34a Flashback appears on some instants and sorceries. It represents two static abilities: one that functions while the card is in a player’s graveyard and another that functions while the card is on the stack. “Flashback [cost]” means “You may cast this card from your graveyard if the resulting spell is an instant or sorcery spell by paying [cost] rather than paying its mana cost” and “If the flashback cost was paid, exile this card instead of putting it anywhere else any time it would leave the stack.” Casting a spell using its flashback ability follows the rules for paying alternative costs in rules 601.2b and 601.2f–h.

702.40a Storm is a triggered ability that functions on the stack. “Storm” means “When you cast this spell, copy it for each other spell that was cast before it this turn. If the spell has any targets, you may choose new targets for any of the copies.”

702.166a Bargain is a static ability that functions while the spell with bargain is on the stack. “Bargain” means “As an additional cost to cast this spell, you may sacrifice an artifact, enchantment, or token.” Paying a spell’s bargain cost follows the rules for paying additional costs in rules 601.2b and 601.2f–h.

702.166b If a spell’s controller declares the intention to pay that spell’s bargain cost, that spell has been “bargained.” See rule 601.2b.

702.166c Objects with bargain have additional abilities that specify what happens if they were bargained. These abilities are linked to the bargain ability printed on that object: they can refer only to that specific bargain ability. See rule 607, “Linked Abilities.”

704.5b If a player attempted to draw a card from a library with no cards in it since the last time state-based actions were checked, that player loses the game.

707.10 To copy a spell, activated ability, or triggered ability means to put a copy of it onto the stack; a copy of a spell isn’t cast and a copy of an activated ability isn’t activated. A copy of a spell or ability copies both the characteristics of the spell or ability and all decisions made for it, including modes, targets, the value of X, and additional or alternative costs. (See rule 601, “Casting Spells.”) Choices that are normally made on resolution are not copied. If an effect of the copy refers to objects used to pay its costs, it uses the objects used to pay the costs of the original spell or ability. A copy of a spell is owned by the player under whose control it was put on the stack. A copy of a spell or ability is controlled by the player under whose control it was put on the stack. A copy of a spell is itself a spell, even though it has no spell card associated with it. A copy of an ability is itself an ability.

707.10a If a copy of a spell is in a zone other than the stack, it ceases to exist. If a copy of a card is in any zone other than the stack or the battlefield, it ceases to exist. These are state-based actions. See rule 704.

707.10c Some effects copy a spell or ability and state that its controller may choose new targets for the copy. The player may leave any number of the targets unchanged, even if those targets would be illegal. If the player chooses to change some or all of the targets, the new targets must be legal. Once the player has decided what the copy’s targets will be, the copy is put onto the stack with those targets.
