# Open questions and assumptions

Collected 2026-10-01 during spec writing and review. None of these changes a visible scenario's expectation; items about decision shapes are ASSUMED shapes the runner adapter must decide (see the SCHEMA addenda).

- Show and Tell: design doc 02 section 9.1 says the second chooser learns nothing about the first pick; CR 101.4a/b says each player must clearly indicate they chose a card (hidden zone card may stay face down). Doc 02 is stricter than the CR (rests on an unconfirmed Gatherer ruling). Decide what the observer may see.
- Defense Grid "its controller's turn": read as the spell's controller (English reading, 608.2c/109.5); confirm with the official ruling when reachable.
- Does declining a "may search ... then shuffle" still shuffle? Not asserted anywhere.
- When does "until your next turn" end? CR silent for a player still in the game (800.4m multiplayer only). Scenarios assert only mid-opponent-turn and next main1.
- Completed dungeon re-choice; Acererak ETB return after Acererak left the battlefield; Runestone Caverns permission duration; sorcery in Mad Wizard's Lair.
- Doc 05 corrections: Oubliette is not random (discard a card and sacrifice a creature, an artifact, and a land); Tomb of Annihilation completes in 3 ventures via Oubliette or 4 via Veils.
- Decision shapes assumed (SCHEMA addenda): Acererak yes_no then choose_cards; replicate yes_no then choose_target; Atraxa single choose_cards; no decision raised for empty attack/block declarations, trivial damage assignment, or order_replacements with identical outcome.
- Holdout key handling: the key lives only in the spec-writer session sandbox; any chat message is readable by every project Claude, so it cannot be handed over there. Decide with Brady (for example he generates a new key locally and re-seals).
- Show and Tell: can the opponent tell "chose nothing" from "had no eligible card"? Skipping the decision when a player has no eligible card could reveal whether they hold an artifact, creature, enchantment or land. Recommendation: a public "chose nothing" event in both cases (would need a schema change so paired worlds can differ in the hidden player's own steps). Not tested.
- Design doc 02 section 9.1 and doc 04 section 7.2 item 6 overstate Show and Tell secrecy (CR 101.4a/b: other player may know a card was chosen); doc 02 section 6 table row for random-order bottoming is garbled (update cell missing).
- Does a mana ability activation between two passes reset pass succession (117.4)? Natural reading: yes, so the other player gets priority again. Scenarios written to be insensitive where possible.
- Design doc 02 knowledge table has no row for searching a hand (Surgical Extraction).
- Aluren/Forge: seat-swap symmetric scenarios exist so an engine cannot favor seat 0.
- Daze/Flusterstorm aimed at a spell that can't be countered: is the "pay {1}" prompt still raised? CR 101.2/113.6g do not say. Not asserted.
- Amass with exactly one Army creature: is a one-candidate choice raised? Scenarios assume no (about six scenarios depend on it).
- Optional "may pay {B}" with no way to pay (Nihil Spellbomb): no prompt assumed (matches the Daze-style convention).
- Cloak and Dagger bounced before its ETB resolves: nothing is exiled (CR 610.3b); whether the "may exile" choice is still raised is not fixed.
- Mobilize delayed sacrifice: one trigger for all tokens or one per token (603.7c silent).
- Lion's Eye Diamond activation while casting a spell (605.3a vs "Activate only as an instant"): not asserted.
- Triumph of Saint Katherine when it is no longer in the graveyard at resolution ("If you do"): not asserted.
- Source of the Quantum Riddler warp delayed trigger: not asserted.
- Containment Priest's "exile it instead" on a card already in exile (Flickerwisp's return): does it count as a zone change? Only the final zone is asserted.
- Lazotep Quarry's "4/4 black Zombie" copy: does it replace or add to the copied creature types? Token subtypes not asserted.
- Riddler plus Bowmasters in the draw step: exactly one trigger asserted (121.2, 614.6); under review.
- Test gap: no way to probe that a printed ability was removed by an effect (Magus of the Moon fetchland: the scenario would also pass if the fetch ability were kept).
- Kaito, Bane of Nightmares: on its controller's turn, does "he's a 3/4 Ninja creature" replace the planeswalker type (CR 205.1a literal reading: type-setting replaces unless "in addition"/"still a") or keep it (recalled, unverified ruling that he stays a planeswalker)? The engine makes him a creature only. Scenarios no longer assert the type line on his turn; add one once confirmed.
- Runner conventions found by the holdout run: legal-action patterns cannot name which Island a Daze alternative cost returns (use expect_illegal); mana-ability `activate` patterns for lands need the runner to match mana options; token setup entries must honor counters, damage and attached_to.
