# CR excerpts retrieved for the Tron-sideboard group B scenarios (Argentum Masticore, Mycosynth Lattice)

Rules 702.16, 702.7, 105.2, 105.4 and 603.12 were retrieved on 2026-10-09 from the Savecraft `rules_search` module ("MTG Comprehensive Rules (effective November 14, 2025)"). The remaining rules below were retrieved earlier from the same module and are copied verbatim from the existing `rust-engine-spec/reference/cr-excerpts*.md` files. One rule per paragraph, examples omitted. Append this file to reference/ (as cr-excerpts-tron2-B.md) so `verified: true` citations validate.

702.16b A permanent or player with protection can’t be targeted by spells with the stated quality and can’t be targeted by abilities from a source with the stated quality.

702.16e Any damage that would be dealt by sources that have the stated quality to a permanent or player with protection is prevented.

702.16f Attacking creatures with protection can’t be blocked by creatures that have the stated quality.

702.7b If at least one attacking or blocking creature has first strike or double strike (see rule 702.4) as the combat damage step begins, the only creatures that assign combat damage in that step are those with first strike or double strike. After that step, instead of proceeding to the end of combat step, the phase gets a second combat damage step. The only creatures that assign combat damage in that step are the remaining attackers and blockers that had neither first strike nor double strike as the first combat damage step began, as well as the remaining attackers and blockers that currently have double strike. After that step, the phase proceeds to the end of combat step.

105.2b A multicolored object is two or more of the five colors.

105.2c A colorless object has no color.

105.4 If a player is asked to choose a color, they must choose one of the five colors. “Multicolored” is not a color. Neither is “colorless.”

603.12 A resolving spell or ability may allow or instruct a player to take an action and create a triggered ability that triggers “when [a player] [does or doesn’t]” take that action or “when [something happens] this way.” These reflexive triggered abilities follow the rules for delayed triggered abilities (see rule 603.7), except that they’re checked immediately after being created and trigger based on whether the trigger event or events occurred earlier during the resolution of the spell or ability that created them.

113.7a Once activated or triggered, an ability exists on the stack independently of its source. Destruction or removal of the source after that time won’t affect the ability. Note that some abilities cause a source to do something (for example, “This creature deals 1 damage to any target”) rather than the ability doing anything directly. In these cases, any activated or triggered ability that references information about the source for use while announcing an activated ability or putting a triggered ability on the stack checks that information when the ability is put onto the stack. Otherwise, it will check that information when it resolves. In both instances, if the source is no longer in the zone it’s expected to be in at that time, its last known information is used. The source can still perform the action even though it no longer exists.

118.12a Some spells, activated abilities, and triggered abilities read, “[Do something] unless [a player does something else].” This means the same thing as “[A player may do something else]. If [that player doesn’t], [do something].”

202.2 . An object is the color or colors of the mana symbols in its mana cost, regardless of the color of its frame.

202.3 The mana value of an object is a number equal to the total amount of mana in its mana cost, regardless of color.

202.3e When calculating the mana value of an object with an {X} in its mana cost, X is treated as 0 while the object is not on the stack, and X is treated as the number chosen for it while the object is on the stack.

603.3d The remainder of the process for putting a triggered ability on the stack is identical to the process for casting a spell listed in rules 601.2c-d. If a choice is required when the triggered ability goes on the stack but no legal choices can be made for it, or if a rule or a continuous effect otherwise makes the ability illegal, the ability is simply removed from the stack.

605.1a An activated ability is a mana ability if it meets all of the following criteria: it doesn’t require a target (see rule 115.6), it could add mana to a player’s mana pool when it resolves, and it’s not a loyalty ability. (See rule 606, “Loyalty Abilities.”)

609.4b If an effect allows a player to spend mana “as though it were mana of any [type or color],” this affects only how the player may pay a cost. It doesn’t change that cost, and it doesn’t change what mana was actually spent to pay that cost. The same is true for effects that say “mana of any type can be spent.”

107.4c The colorless mana symbol {C} is used to represent one colorless mana, and also to represent a cost that can be paid only with one colorless mana.

613.1d Layer 4: Type-changing effects are applied. These include effects that change an object’s card type, subtype, and/or supertype.

701.21a To sacrifice a permanent, its controller moves it from the battlefield directly to its owner’s graveyard. A player can’t sacrifice something that isn’t a permanent, or something that’s a permanent they don’t control. Sacrificing a permanent doesn’t destroy it, so regeneration or other effects that replace destruction can’t affect this action.

701.8a To destroy a permanent, move it from the battlefield to its owner’s graveyard.

702.12b A permanent with indestructible can’t be destroyed. Such permanents aren’t destroyed by lethal damage, and they ignore the state-based action that checks for lethal damage (see rule 704.5g).

704.5f If a creature has toughness 0 or less, it’s put into its owner’s graveyard. Regeneration can’t replace this event.
