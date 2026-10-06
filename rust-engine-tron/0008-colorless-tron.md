# RFC 0008: Colorless Tron (cost floor, no-untap, protection from everything, dynamic mana, filter lands, and smaller effects)

Status: draft, ready for review (patch tested in a scratch copy; live engine untouched; independent reviewers and Brady's sign-off pending)
Author: Colorless Tron thread  Reviewers (two independent, one a rules reviewer): pending

## Problem
Brady's own Colorless Tron 75 (`decks/colorless-tron.txt`, deck 10) is an Alurentell opponent. 27 of its 32 distinct cards are missing from the engine (`decks/tron-engine-gap.md`). 9 are plain card data. The rest need core support, listed below. Shared Storm items (Urza's Saga, Wish for Karn -2, free cast of exiled cards for Ugin -11, activation condition on `ActivatedDef`) belong to RFC 0007 (Storm) and are NOT repeated here; Tron's main deck has 4 Urza's Saga, so Tron is not fully playable until 0007 lands.

## Proposed change
`mtg-core` (all enum cases are appended; existing discriminants do not move):
1. **Cost floor** (Trinisphere): `AbilityDef::CostFloor(CostFloorDef { min, cond })`. `finish_plan` (`cost.rs`), the one place every cast goes through, raises the mana cost to `min` after all other cost changes and before delve (CR 601.2f), whatever the way (hard cast, Force of Will pitch, Daze, Aluren/Omniscience free casts, Lotus Petal). Extra mana is generic. `CardDb::has_cost_floor` (like `has_restrict`) lets `cost_floor` skip the battlefield scan when no card in the pool has a floor.
2. **No untap** (Grim Monolith): `Keywords::NO_UNTAP`, read in the untap step (`turn.rs`).
3. **Unblockable** (Manifold Key): `Keywords::UNBLOCKABLE`, read in `can_block_pair` (`combat.rs`).
4. **Protection from everything for a player** (The One Ring): `PlayerFx::ProtectionFromEverything`. Damage to the player is prevented (`ops.rs`) and the player is not a legal target (`legal.rs`, `can_target_from`). Lasts until the controller's next turn (`Until::YourNextTurn`).
5. **Static restrictions**: `Restriction::NoCreatureEnterTriggers` (Torpor Orb; `trigger.rs` skips enter triggers for creatures) and `Restriction::AttackPowerOverHand` (Ensnaring Bridge; `attack_candidates` in `combat.rs`, hand size of the Bridge's controller).
6. **Effects**: `Effect::ExileTargetUntilLeaves { target }` (Portable Hole, uses the existing `until_links`), `Effect::ExileChoosePlay { n }` plus `CardsPurpose::PlayChosen` (Mishra's Research Desk, uses existing `exile_plays`; the end-of-next-turn expiry is NOT modeled), `Effect::AddManaExpr { color, n: Expr }` (mana ability whose amount is evaluated at use: Urza's Tower, Urza's Workshop).
7. **ObjFilter.cmc_x_max** ("mana value X or less", Kozilek's Command).
8. **Filter land payment** (Planar Nexus, "{1}, {T}: Add one mana of any color"): automatic payment (`plan_cost_with`) retries, when the plain plan fails, with k filter sources tapped as the "{1}" of their ability, each turning one other source into any color (two permanents make one colored mana). Explicit-mana mode does not offer mana abilities that have a mana cost (documented limit: that option is only reachable through automatic payment).
9. **17 subtypes** appended to the subtype table: Karn, Ugin, Tezzeret, Saga, Tower, Mine, Power-Plant, Spawn, Scion, Masticore, Spacecraft, Robot, Cave, Gate, Lair, Locus, Sphere (the last five are Planar Nexus's land types). 114 of 128 are used; Storm appends Lesson and Sorcerer after these.
10. **Restriction `who` is now honored** (`restrict.rs`, `activation_blocked`): Karn's static ("opponents can't activate...") previously ignored `Rel::Opp`/`Rel::You`; the binding is checked against the source's controller. Only Karn's static uses `CantActivate` with a `who`; the older `who: Opp` restrictions are `CantCast` (a different code path), so no existing card changes.
11. **Command zone scanned for triggers** (`trigger.rs`, `ZoneKind::Command`): needed for Tezzeret's emblem; only emblems live there.
12. **Mode and X feasibility** (`cast.rs`): a mode whose target is "mana value X (or less)" is offered only if some affordable X leaves a target; "up to X" targets (optional) no longer require X distinct targets in the X stage. Both found by the fuzz with Kozilek's Command (empty target list). `priority.rs` already had the activation-side equivalent.
13. **Mana-payment details**: payment candidates now sort objects named in the scenario's `pay` hint last when choosing which permanent to give up (`cost.rs`, `pick_candidates`; only affects which of several equal choices is offered first, so it changes no legal set); a dynamic mana ability that evaluates to 0 is not offered (`legal.rs`).
14. **Card database hash**: `ObjFilter.cmc_x_max` is a `HashFalseFlag` (hashes like an absent field when false), and every new enum case is appended, so the goldens replay bit-identically even though the card definitions of the 8 older decks are unchanged.
15. **Spec adapter** (`mtg-spec`, test-only): honors the `ability` index in legal-action patterns (card-order among activated abilities), matches emblems by name and ability text, maps `burden` counters, and checks "extra modes" and "X not offered" in illegal-action scenarios. Visible spec stays 838/838.

Card data: `crates/mtg-cards/cards/tron.cards.ron` (second source file in `legacy.rs`). Kozilek's Command is a Kindred Instant — Eldrazi (`KINDRED | INSTANT`, subtype Eldrazi), as in its Oracle text.

## Documented limits (not modeled; `known-divergences` entries to be added on approval)
Urza's Saga and Summon: Bahamut (RFC 0007), Karn -2, Ugin -11, Argentum Masticore, Mycosynth Lattice and Eldrazi Confluence (not defined at all), Extinguisher Battleship station (it is never a creature), Planar Nexus's filter only via automatic payment. Also: Kozilek's Command's two "target player" modes only target you (so while The One Ring's protection is up they are unavailable, where real Magic lets you target the opponent); Mishra's Research Desk's chosen card stays playable indefinitely instead of until the end of your next turn, and its unearth is missing; the Construct token's "+1/+1 for each artifact" (layer 7c in the rules) is a layer 7a set-P/T here, the same value unless another P/T effect applies, and the token is orphaned until Urza's Saga (RFC 0007) lands.

## Rules basis
Quotes are from the scenario writers' retrieved excerpts (`cr-excerpts-tron-{A,B,C}.md`, Savecraft rules module); Oracle text from Savecraft card data.
- **Cost floor** (Trinisphere). 601.2f: the total cost is the mana cost or alternative cost plus cost increases, minus reductions. 118.9d: "any additional costs, cost increases, and cost reductions that affect that spell are applied to that alternative cost", so Force of Will's pitch and Daze's return still pay {3}. Trinisphere's floor applies after all other changes, hence the single place in `finish_plan`. "You may cast [this object] without paying its mana cost" is itself an alternative cost (118.9), so Aluren's and Omniscience's free casts still pay {3}; scenarios `tron-a-trinisphere-*` pin each (Force of Will pitch, Daze, Aluren, Omniscience).
- **No untap** (Grim Monolith). 502.3: the active player untaps all permanents "but effects can keep one or more of a player's permanents from untapping".
- **Protection of a player** (The One Ring). 702.16j: "protection from everything" is protection from each object regardless of its characteristics; 702.16b: can't be targeted by spells or abilities from such a source; 702.16e: damage from such sources is prevented.
- **Static restrictions**. 508.1c: attack restrictions are checked when attackers are declared (Ensnaring Bridge counts the Bridge controller's hand). 603.2 / 603.6a: enter-the-battlefield triggers; Torpor Orb makes creatures entering not cause abilities to trigger (a static effect that stops the trigger event matching).
- **Exile until leaves** (Portable Hole). 610.3: a one-shot "until" effect, the return happens when the source leaves; 610.3b covers the leave-before-resolve case (nothing is exiled).
- **Mana abilities with dynamic amounts and filter costs** (Tower, Workshop, Planar Nexus). 605.1a: an activated ability that could add mana, has no target and is not a loyalty ability is a mana ability; 605.3b: resolves immediately, no stack. Amounts are counted on resolution (Tower: 7 only with Mine and Power-Plant).
- **Unblockable / X-or-less targets**: ordinary keyword and target restrictions, 509.1b, 601.2c.
- **Loyalty abilities and emblems** (Karn, Tezzeret, Ugin). 606.1-606.6 for loyalty abilities. Tezzeret's -7 emblem lives in the command zone and its triggers are scanned there (the engine now scans `ZoneKind::Command`).
- **Eldrazi Spawn / Scion tokens**: 605.3b sacrifice-for-mana is a mana ability; 111.7 token.

## Impact
- State layout and `hash_rules` / `hash_full`: no new state; `player_fx`, `until_links` and `exile_plays` are already hashed. Keywords gain two bits (derived), subtype table gains 17 entries (u128, 114 of 128 used).
- Determinism and golden replays: expected unchanged (results below).
- Hidden information: none new (Desk's exiled cards are public once exiled; the chosen one is a public choice).
- Effect VM / IR: 3 new `Effect` cases, 1 `AbilityDef`, 1 `PlayerFx`, 2 `Restriction`, 1 `CardsPurpose`, 1 `ObjFilter` field, 2 keyword bits. `deps-reviewed.txt` gains one reviewed pair (Tezzeret's emblem and Magus of the Moon, layer 4).
- `ENGINE_CORE_VERSION` bump: no if goldens replay bit-identically (to be confirmed below). The card database hash and `n_defs` change because cards were added; nets trained before need the new pool.

## Test plan and results
Scenarios were written by three separate agents from Oracle text and the CR only (the implementer did not write or see them first); defects found in them were sent back to their writers.
- 156 Tron scenarios: 156 pass. Visible spec: 838 of 838. Workspace tests: 81 passed, 0 failed here (the independent review's run, without the sideboarding `mtg-match` tests that live in PR #4, got 73 passed, 0 failed, and 994/994 spec with the 156 added). Goldens (test pool and real pool) bit-identical.
- 9-deck fuzz, 3000 games, 2.04 M decisions, 0 violations. It found two Kozilek's Command bugs (items 12), fixed.
- Matchup-critical checks pinned by scenarios: Trinisphere against Force of Will, Daze, Aluren and Omniscience (14 `tron-a-trinisphere-*` files); Torpor Orb (`tron-B-*orb*`); Planar Nexus payment and land types (`tron-C-nexus-*`); Pithing Needle against Boseiju's channel.
- Not run: the sealed holdout and Forge differential testing (this change adds no holdout cards).
Details and the exact commands are in `README.md`.
