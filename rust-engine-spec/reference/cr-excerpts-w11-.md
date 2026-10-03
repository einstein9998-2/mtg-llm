# Comprehensive Rules excerpts used as spec sources

Pinned version: **Comprehensive Rules effective 2025-11-14**, as served by the Savecraft `rules_search` module on 2026-10-01. Text below is copied from that module. A scenario may mark a CR source `verified: true` only if the cited text is in this file (or in a `cr-excerpts-<batch>.md` file a writer added after retrieving it from `rules_search`). Anything cited from memory is `verified: false` and the reviewer must retrieve it.

## 117.3 (priority)

117.3a The active player receives priority at the beginning of most steps and phases, after any turn-based actions (such as drawing a card during the draw step; see rule 703) have been dealt with and abilities that trigger at the beginning of that phase or step have been put on the stack. No player receives priority during the untap step. Players usually don't get priority during the cleanup step (see rule 514.3).
117.3b The active player receives priority after a spell or ability (other than a mana ability) resolves.
117.3c If a player has priority when they cast a spell, activate an ability, or take a special action, that player receives priority afterward.
117.3d If a player has priority and chooses not to take any actions, that player passes. If any mana is in that player's mana pool, they announce what mana is there. Then the next player in turn order receives priority.

## 603.3 (putting triggered abilities on the stack)

603.3 Once an ability has triggered, its controller puts it on the stack as an object that's not a card the next time a player would receive priority. See rule 117, "Timing and Priority." The ability becomes the topmost object on the stack. It has the text of the ability that created it, and no other characteristics. It remains on the stack until it's countered, it resolves, a rule causes it to be removed from the stack, or an effect moves it elsewhere.
603.3a A triggered ability is controlled by the player who controlled its source at the time it triggered, unless it's a delayed triggered ability. To determine the controller of a delayed triggered ability, see rules 603.7d-f.
603.3b If multiple abilities have triggered since the last time a player received priority, the abilities are placed on the stack in a two-part process. First, each player, in APNAP order, puts each triggered ability they control with a trigger condition that isn't another ability triggering on the stack in any order they choose. (See rule 101.4.) Second, each player, in APNAP order, puts all remaining triggered abilities they control on the stack in any order they choose. Then the game once again checks for and performs state-based actions until none are performed, then abilities that triggered during this process go on the stack. This process repeats until no new state-based actions are performed and no abilities trigger. Then the appropriate player gets priority.
603.3c If a triggered ability is modal, its controller announces the mode choice when putting the ability on the stack. If one of the modes would be illegal (due to an inability to choose legal targets, for example), that mode can't be chosen. If no mode is chosen, the ability is removed from the stack. (See rule 700.2.)
603.3d The remainder of the process for putting a triggered ability on the stack is identical to the process for casting a spell listed in rules 601.2c-d. If a choice is required when the triggered ability goes on the stack but no legal choices can be made for it, or if a rule or a continuous effect otherwise makes the ability illegal, the ability is simply removed from the stack.

101.4 If multiple players would make choices and/or take actions at the same time, the active player (the player whose turn it is) makes any choices required, then the next player in turn order (usually the player seated to the active player's left) makes any choices required, followed by the remaining nonactive players in turn order. Then the actions happen simultaneously. This rule is often referred to as the "Active Player, Nonactive Player (APNAP) order" rule.

## 603.4 (intervening "if")

603.4 A triggered ability may read "When/Whenever/At [trigger event], if [condition], [effect]." When the trigger event occurs, the ability checks whether the stated condition is true. The ability triggers only if it is; otherwise it does nothing. If the ability triggers, it checks the stated condition again as it resolves. If the condition isn't true at that time, the ability is removed from the stack and does nothing. Note that this mirrors the check for legal targets.

## 704.3 (state-based actions timing)

704.3 Whenever a player would get priority (see rule 117, "Timing and Priority"), the game checks for any of the listed conditions for state-based actions, then performs all applicable state-based actions simultaneously as a single event. If any state-based actions are performed as a result of a check, the check is repeated; otherwise all triggered abilities that are waiting to be put on the stack are put on the stack, then the check is repeated. Once no more state-based actions have been performed as the result of a check and no triggered abilities are waiting to be put on the stack, the appropriate player gets priority. This process also occurs during the cleanup step (see rule 514), except that if no state-based actions are performed as the result of the step's first check and no triggered abilities are waiting to be put on the stack, then no player gets priority and the step ends.

(704.4, from the Savecraft reasoning guide, paraphrased there: state-based actions pay no attention to what happens during the resolution of a spell or ability. The rule text itself has not been retrieved verbatim; a writer citing 704.4 must fetch it.)

## 608.2b (resolution, target legality)

608.2b If the spell or ability specifies targets, it checks whether the targets are still legal. A target that's no longer in the zone it was in when it was targeted is illegal. Other changes to the game state may cause a target to no longer be legal; for example, its characteristics may have changed or an effect may have changed the text of the spell. If the source of an ability has left the zone it was in, its last known information is used during this process. If all its targets, for every instance of the word "target," are now illegal, the spell or ability doesn't resolve. It's removed from the stack and, if it's a spell, put into its owner's graveyard. Otherwise, the spell or ability will resolve normally. Illegal targets, if any, won't be affected by parts of a resolving spell's effect for which they're illegal. Other parts of the effect for which those targets are not illegal may still affect them. If the spell or ability creates any continuous effects that affect game rules (see rule 613.11), those effects don't apply to illegal targets. If part of the effect requires information about an illegal target, it fails to determine any such information. Any part of the effect that requires that information won't happen.

## 118.9 (alternative costs) and 601.2f (total cost)

118.9 Some spells have alternative costs. An alternative cost is a cost listed in a spell's text, or applied to it from another effect, that its controller may pay rather than paying the spell's mana cost. Alternative costs are usually phrased, "You may [action] rather than pay [this object's] mana cost," or "You may cast [this object] without paying its mana cost." Note that some alternative costs are listed in keywords; see rule 702.
118.9a Only one alternative cost can be applied to any one spell as it's being cast. The controller of the spell announces their intentions to pay that cost as described in rule 601.2b.
118.9b Alternative costs are generally optional. An effect that allows you to cast a spell may require a certain alternative cost to be paid.
118.9c An alternative cost doesn't change a spell's mana cost, only what its controller has to pay to cast it. Spells and abilities that ask for that spell's mana cost still see the original value.
118.9d If an alternative cost is being paid to cast a spell, any additional costs, cost increases, and cost reductions that affect that spell are applied to that alternative cost. (See rule 601.2f.)

601.2f The player determines the total cost of the spell. Usually this is just the mana cost. Some spells have additional or alternative costs. Some effects may increase or reduce the cost to pay, or may provide other alternative costs. Costs may include paying mana, tapping permanents, sacrificing permanents, discarding cards, and so on. The total cost is the mana cost or alternative cost (as determined in rule 601.2b), plus all additional costs and cost increases, and minus all cost reductions. If multiple cost reductions apply, the player may apply them in any order. If the mana component of the total cost is reduced to nothing by cost reduction effects, it is considered to be {0}. It can't be reduced to less than {0}. Once the total cost is determined, any effects that directly affect the total cost are applied. Then the resulting total cost becomes "locked in." If effects would change the total cost after this time, they have no effect.

## 701.49 (venture) and 309.5 (dungeon movement)

701.49a If a player is instructed to venture into the dungeon while they don't own a dungeon card in the command zone, they choose a dungeon card they own from outside the game and put it into the command zone. They put their venture marker on the topmost room. See rule 309, "Dungeons."
701.49b If a player is instructed to venture into the dungeon while their venture marker is in any room except a dungeon card's bottommost room, they choose an adjacent room, following the direction of an arrow pointing away from their current room. If there are multiple arrows pointing away from the room the player's venture marker is in, they choose one of them to follow. They move their venture marker to that adjacent room.
701.49c If a player is instructed to venture into the dungeon while their venture marker is in the bottommost room of a dungeon card, they remove that dungeon card from the game. Doing so causes the player to complete that dungeon (see rule 309.7). They then choose an appropriate dungeon card they own from outside the game, put it into the command zone, and put their venture marker on the topmost room of that dungeon.
701.49d Venture into [quality] is a variant of venture into the dungeon. (...)
309.5 The venture into the dungeon keyword action allows players to move their venture marker down the rooms of a dungeon card.
309.5a If a player ventures into the dungeon while they own a dungeon card in the command zone and their venture marker isn't on that dungeon's bottommost room, they move their venture marker from the room it is on to the next room, following the direction of an arrow pointing away from the room their venture marker is on. If there are multiple arrows pointing away from the room their venture marker is on, they choose one of them to follow.
309.5b If a player ventures into the dungeon while they own a dungeon card in the command zone and their venture marker is on that dungeon card's bottommost room, they remove that dungeon card from the game. They then choose a dungeon card they own from outside the game and put it into the command zone. They put their venture marker on the topmost room.
309.7 A player completes a dungeon as that dungeon card is removed from the game.

(309.2, 309.4a, 309.4c and 309.6 / 704.5t are cited by the design docs, which say they were checked against this CR version. They are not in this file because they were not retrieved in this thread; a writer citing them must fetch them first. Observation worth the reviewer's attention: 701.49a/309.5 as retrieved say "choose a dungeon card they own from outside the game" with no restriction on re-choosing a completed dungeon.)

## 702.56 (replicate)

702.56a Replicate is a keyword that represents two abilities. The first is a static ability that functions while the spell with replicate is on the stack. The second is a triggered ability that functions while the spell with replicate is on the stack. "Replicate [cost]" means "As an additional cost to cast this spell, you may pay [cost] any number of times" and "When you cast this spell, if a replicate cost was paid for it, copy it for each time its replicate cost was paid. If the spell has any targets, you may choose new targets for any of the copies." Paying a spell's replicate cost follows the rules for paying additional costs in rules 601.2b and 601.2f-h.
702.56b If a spell has multiple instances of replicate, each is paid separately and triggers based on the payments made for it, not any other instance of replicate.

## 702.66 (delve)

702.66a Delve is a static ability that functions while the spell with delve is on the stack. "Delve" means "For each generic mana in this spell's total cost, you may exile a card from your graveyard rather than pay that mana."
702.66b The delve ability isn't an additional or alternative cost and applies only after the total cost of the spell with delve is determined.
702.66c Multiple instances of delve on the same spell are redundant.

## 701.25 (surveil)

701.25a To "surveil N" means to look at the top N cards of your library, then put any number of them into your graveyard and the rest on top of your library in any order.
701.25b If an effect allows you to look at additional cards while you surveil, those cards are included among the cards you may put into your graveyard and on top of your library in any order.
701.25c If a player is instructed to surveil 0, no surveil event occurs. Abilities that trigger whenever a player surveils won't trigger.
701.25d An ability that triggers whenever a player surveils triggers after the process described in rule 701.25a is complete, even if some or all of those actions were impossible.

## Other rules text already vouched for by the design docs (not re-fetched)

Design doc 01 and 05 state they checked these against the same CR version: 510.1c (no damage assignment order), 616.1 and 616.1a-g (replacement ordering), 704.5 list incl. 704.5a/b/j/m/n/q/t, 613 layers, 309 and 701.49 dungeons, 701.54 The Ring, 702.185 warp, 702.181 mobilize, 702.176 impending. A scenario citing one of these may say `verified: false` with `note: vouched by doc 01`.

## Retrieval protocol for writers and reviewers

Query `mcp__Savecraft__query_reference` with `game_id: magic`, `module: rules_search`, `rule: "<number>"`. Each response carries a fixed ~9 KB reasoning guide after the rule text; ignore it. Batch several rule numbers in one call. Copy retrieved text into your own `reference/cr-excerpts-<batch>.md` (do not edit this file concurrently).
