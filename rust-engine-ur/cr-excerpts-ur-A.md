# CR excerpts retrieved for the UR-alternate group A scenarios (Stormchaser's Talent, Otter token)

Rules 716.* were retrieved on 2026-10-08 from the Savecraft `rules_search` module ("MTG Comprehensive Rules (effective November 14, 2025)", rule 716 and rule 702.108). The other rules were retrieved earlier from the same module and are copied verbatim from the existing `rust-engine-spec/reference/cr-excerpts*.md` files. One rule per paragraph, examples omitted. Append this file to reference/ (as cr-excerpts-ur-A.md) so `verified: true` citations validate.

716.2 A class level bar is a keyword ability that represents both an activated ability and a static ability. A class level bar includes the activation cost of its activated ability and a level number. Any abilities printed within the same text box section as the class level bar are part of its static ability.

716.2a “[Cost]: Level N — [Abilities]” means “[Cost]: This Class’s level becomes N. Activate only if this Class is level N-1 and only as a sorcery” and “As long as this Class is level N or greater, it has [abilities].”

716.2b A level is a designation that any permanent can have. A Class retains its level even if it stops being a Class. Levels are not a copiable characteristic.

716.2d If a rule or effect refers to a permanent’s level and that permanent doesn’t have a level, it is treated as though its level is 1.

716.3 Any ability printed on a Class card that isn’t preceded by a class level bar is treated normally. In particular, the Class has the ability printed in its top text box section at all times. That ability may affect the game if it’s a static ability, it may trigger if it’s a triggered ability, and it can be activated if it’s an activated ability.

601.2a To propose the casting of a spell, a player first moves that card (or that copy of a card) from where it is to the stack. It becomes the topmost object on the stack. It has all the characteristics of the card (or the copy of a card) associated with it, and that player becomes its controller. Any continuous effects that modify the characteristics of the spell as you start casting it begin as it is put on the stack (see rule 611.2f). The spell remains on the stack until it resolves, it’s countered, or a rule or effect moves it elsewhere.

601.2i Once the steps described in 601.2a-h are completed, effects that modify the characteristics of the spell as it's cast are applied, then the spell becomes cast. Any abilities that trigger when a spell is cast or put onto the stack trigger at this time. If the spell's controller had priority before casting it, they get priority.

603.2 Whenever a game event or game state matches a triggered ability's trigger event, that ability automatically triggers. The ability doesn't do anything at this point.

603.3b If multiple abilities have triggered since the last time a player received priority, the abilities are placed on the stack in a two-part process. First, each player, in APNAP order, puts each triggered ability they control with a trigger condition that isn't another ability triggering on the stack in any order they choose. (See rule 101.4.) Second, each player, in APNAP order, puts all remaining triggered abilities they control on the stack in any order they choose. Then the game once again checks for and performs state-based actions until none are performed, then abilities that triggered during this process go on the stack. This process repeats until no new state-based actions are performed and no abilities trigger. Then the appropriate player gets priority.

603.3d The remainder of the process for putting a triggered ability on the stack is identical to the process for casting a spell listed in rules 601.2c-d. If a choice is required when the triggered ability goes on the stack but no legal choices can be made for it, or if a rule or a continuous effect otherwise makes the ability illegal, the ability is simply removed from the stack.

603.6a Enters-the-battlefield abilities trigger when a permanent enters the battlefield. These are written, "When [this object] enters, . . . " or "Whenever a [type] enters, . . ." Each time an event puts one or more permanents onto the battlefield, all permanents on the battlefield (including the newcomers) are checked for any enters-the-battlefield triggers that match the event.

602.5d Activated abilities that read “Activate only as a sorcery” mean the player must follow the timing rules for casting a sorcery spell, though the ability isn’t actually a sorcery. The player doesn’t actually need to have a sorcery card that they could cast.

608.2b If the spell or ability specifies targets, it checks whether the targets are still legal. A target that's no longer in the zone it was in when it was targeted is illegal. Other changes to the game state may cause a target to no longer be legal; for example, its characteristics may have changed or an effect may have changed the text of the spell. If the source of an ability has left the zone it was in, its last known information is used during this process. If all its targets, for every instance of the word "target," are now illegal, the spell or ability doesn't resolve. It's removed from the stack and, if it's a spell, put into its owner's graveyard. Otherwise, the spell or ability will resolve normally. Illegal targets, if any, won't be affected by parts of a resolving spell's effect for which they're illegal. Other parts of the effect for which those targets are not illegal may still affect them. If the spell or ability creates any continuous effects that affect game rules (see rule 613.11), those effects don't apply to illegal targets. If part of the effect requires information about an illegal target, it fails to determine any such information. Any part of the effect that requires that information won't happen.

400.7 . An object that moves from one zone to another becomes a new object with no memory of, or relation to, its previous existence. This rule has the following exceptions.

701.6a To counter a spell or ability means to cancel it, removing it from the stack. It doesn’t resolve and none of its effects occur. A countered spell is put into its owner’s graveyard.

701.6b The player who cast a countered spell or activated a countered ability doesn’t get a “refund” of any costs that were paid.

707.10 . To copy a spell, activated ability, or triggered ability means to put a copy of it onto the stack; a copy of a spell isn’t cast and a copy of an activated ability isn’t activated. A copy of a spell or ability copies both the characteristics of the spell or ability and all decisions made for it, including modes, targets, the value of X, and additional or alternative costs. (See rule 601, “Casting Spells.”) Choices that are normally made on resolution are not copied. If an effect of the copy refers to objects used to pay its costs, it uses the objects used to pay the costs of the original spell or ability. A copy of a spell is owned by the player under whose control it was put on the stack. A copy of a spell or ability is controlled by the player under whose control it was put on the stack. A copy of a spell is itself a spell, even though it has no spell card associated with it. A copy of an ability is itself an ability.

707.10c Some effects copy a spell or ability and state that its controller may choose new targets for the copy. The player may leave any number of the targets unchanged, even if those targets would be illegal. If the player chooses to change some or all of the targets, the new targets must be legal. Once the player has decided what the copy's targets will be, the copy is put onto the stack with those targets.

702.40a Storm is a triggered ability that functions on the stack. “Storm” means “When you cast this spell, copy it for each other spell that was cast before it this turn. If the spell has any targets, you may choose new targets for any of the copies.”

118.12a Some spells, activated abilities, and triggered abilities read, “[Do something] unless [a player does something else].” This means the same thing as “[A player may do something else]. If [that player doesn’t], [do something].”

117.1a A player may cast an instant spell any time they have priority. A player may cast a noninstant spell during their main phase any time they have priority and the stack is empty.

307.1 A player who has priority may cast a sorcery card from their hand during a main phase of their turn when the stack is empty. Casting a sorcery as a spell uses the stack. (See rule 601, “Casting Spells.”)

117.3b The active player receives priority after a spell or ability (other than a mana ability) resolves.

702.108a Prowess is a triggered ability. “Prowess” means “Whenever you cast a noncreature spell, this creature gets +1/+1 until end of turn.”

118.9 Some spells have alternative costs. An alternative cost is a cost listed in a spell's text, or applied to it from another effect, that its controller may pay rather than paying the spell's mana cost. Alternative costs are usually phrased, "You may [action] rather than pay [this object's] mana cost," or "You may cast [this object] without paying its mana cost." Note that some alternative costs are listed in keywords; see rule 702.
