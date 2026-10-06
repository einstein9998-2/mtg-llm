# Colorless Tron scenarios, group B: notes for the implementer

51 scenarios, one file each, `scenarios/tron-B-<topic>.yaml`. Written from the Oracle text in SCENARIO-BRIEF.md and the Comprehensive Rules (2025-11-14, Savecraft `rules_search`, or text already pinned in the spec project's `reference/cr-excerpts*.md`). No engine source, patch, `tron.cards.ron` or gap file was read. Cards: Portable Hole, Mishra's Research Desk, Torpor Orb, Ensnaring Bridge, Pithing Needle, Tormod's Crypt, Liquimetal Coating, Expedition Map, Extinguisher Battleship (enters trigger only).

## Validation

Command (scratch copy of `rust-engine-spec/reference/` with `oracle-additions-B.json` merged into `oracle.json["cards"]` and `cr-excerpts-tron-B.md` added):

`python3 /mnt/project-files/rust-engine-spec/tools/validate.py --oracle <scratch>/refs/oracle.json --refs <scratch>/refs scenarios/tron-B-*.yaml` -> `51 scenario(s), 0 error(s), 0 warning(s)`.

Deliverables: `scenarios/tron-B-*.yaml`, `oracle-additions-B.json` (11 cards: the 9 above plus support cards Voltaic Key and Ugin, Eye of the Storms), `cr-excerpts-tron-B.md` (every rule cited with `verified: true`; verbatim from `rules_search` or from the pinned spec excerpts).

## Scenarios

| card | file topic | what it fixes |
|---|---|---|
| Portable Hole | exile-until-leaves-returns-new-object | exile, then Abrade destroys Hole: card returns as a new object (counter lost, summoning sick) |
| | legal-targets-mana-value-two-or-less-nonland | target mask: opponent's only, nonland, MV <= 2; tokens and Lotus Petal (MV 0) legal; own Carpet, Flickerwisp, Aluren, land illegal |
| | no-legal-target-trigger-removed | no target: no decision, Hole stays, nothing exiled (603.3d) |
| | leaves-before-trigger-resolves-nothing-exiled | Hole destroyed in response: target is not exiled (610.3b) |
| | exiled-token-does-not-return | token ceases to exist in exile, nothing returns |
| | target-sacrificed-in-response-trigger-fizzles | Lotus Petal sacrificed in response: trigger fizzles (608.2b) |
| Mishra's Research Desk | cast-chosen-spell-from-exile-other-stays-exiled | sacrifice is a cost (ability on stack with Desk in graveyard), choose one, cast Ponder from exile, unchosen Island not playable |
| | chosen-land-playable-unchosen-spell-not | play chosen land from exile (land drop used), unchosen Bolt not castable though mana is available |
| | land-drop-already-used-chosen-land-not-playable | chosen land cannot be played when the land drop is spent |
| | instant-speed-activation-chosen-instant-castable-now | activation in opponent's end step, chosen Brainstorm castable at once |
| | chosen-sorcery-not-castable-in-opponents-turn | chosen sorcery keeps sorcery timing |
| | countered-ability-sacrifice-still-paid | Stifle: Desk gone, nothing exiled |
| Torpor Orb | acererak-enters-no-bounce-no-venture | Acererak stays, no dungeon (Orb controlled by the opponent) |
| | atraxa-via-show-and-tell-no-reveal | Show and Tell put-in: no reveal, hand and library unchanged (own Orb) |
| | cloak-and-dagger-no-hand-reveal | no target prompt, nothing exiled |
| | acererak-attack-trigger-still-works | non-enters triggers unaffected, Zombie created |
| | bowmasters-enters-half-only | one ability with two trigger events: enters half suppressed, "opponent draws" half works (Consider) |
| | portable-hole-still-triggers | non-creature entering is unaffected; Orb itself is a legal Hole target |
| | spider-woman-enters-tapped-still-applies | static/replacement effects are not suppressed |
| Ensnaring Bridge | power-vs-hand-boundary | power == hand legal, > hand illegal, attacker's own hand irrelevant; whole declaration illegal |
| | empty-hand-power-zero-attacks-blocking-unaffected | empty hand: power 0 (Nethergoyf) may attack, power 1 may not; blockers unaffected |
| | atraxa-seven-cards-hand-shrinks-mid-combat | Atraxa (7) attacks with 7 cards; Dark Ritual afterwards does not remove her from combat |
| | atraxa-six-cards-cannot-attack | 6 cards: Atraxa barred, another attacker legal |
| | stock-up-raises-hand-atraxa-legal | hand 6 -> 7 via Stock Up before combat makes Atraxa legal |
| | opponent-hand-counts-for-controller-only | Bridge controller's hand (3) lets powers 1 and 2 attack; attacker's hand 0 irrelevant |
| Pithing Needle | wasteland-destroy-blocked-mana-allowed | both players, mana ability exempt, name chosen as it enters (no stack object) |
| | griselbrand-response-before-resolution | activation legal in response to the Needle spell, illegal after it names Griselbrand (cost still payable) |
| | boseiju-channel-from-hand-blocked | Channel from hand barred (control before), land play and mana ability fine |
| | street-wraith-cycling-blocked | cycling from hand barred (control before) |
| | ugin-loyalty-mana-ability-blocked | 0: Add {C}{C}{C} is a loyalty ability, not a mana ability, so barred; +2 barred; loyalty unchanged |
| | lotus-petal-mana-ability-allowed | sacrifice-for-mana still works |
| | tormods-crypt-blocked | Crypt cannot be activated, either target |
| Tormod's Crypt | in-response-to-reanimate-fizzles | exile Griselbrand in response, Reanimate fizzles, no life loss, Reanimate lands in graveyard |
| | exile-shrinks-nethergoyf-lethal-damage | graveyard exile re-evaluates CDA P/T (3/4 -> 0/1) and a damaged Nethergoyf dies |
| | free-cast-and-immediate-use | {0} cost, no summoning sickness, only targeted graveyard exiled |
| | countered-by-stifle | cost already paid, graveyard stays |
| | own-graveyard-also-exiles-itself | sacrificed Crypt is in the graveyard being exiled |
| Liquimetal Coating | land-becomes-artifact-land-keeps-mana | types exactly Artifact+Land, Island subtype, still taps for mana; only {T} cost |
| | effect-ends-at-cleanup | present in end step, gone in opponent's upkeep |
| | creature-becomes-artifact-abrade-destroys | target legality uses derived types (Abrade mode 2 illegal before, legal after) |
| | voltaic-key-untaps-coated-creature | Key target legality, untap works |
| | null-rod-shuts-coated-land-mana | coated land (artifact) cannot tap for mana under Null Rod; uncoated land can |
| Expedition Map | finds-nonbasic-land-to-hand-and-shuffles | any land card, mask excludes spells, to hand, shuffle (scripted) |
| | no-land-in-library-still-shuffles | search raised, found null, shuffle still happens |
| | may-decline-to-find-with-lands-present | 701.23b |
| | countered-ability-no-search-no-shuffle | `random: []`, no search decision |
| | cast-and-activate-same-turn | no summoning sickness; fetched land does not use the land drop |
| Extinguisher Battleship | etb-destroys-noncreature-then-four-damage-to-all-creatures | mask (noncreature only, itself included), destroy then 4 damage to both sides, survivors keep damage, Spacecraft takes none |
| | show-and-tell-enters-trigger-kills-simultaneous-creature | put-in (not cast) still triggers; opposing creature entering at the same time dies |
| | under-torpor-orb-still-triggers-and-destroys-orb | Spacecraft is not a creature, Orb does not stop it |
| | targets-itself-damage-still-dealt | self-destroy, damage still dealt |

## Rulings doubts (please read)

1. **Pithing Needle on cards in hand (Boseiju channel, Street Wraith cycling).** The expectation (the abilities are barred) follows from the Oracle text ("sources with the chosen name": a card in hand is the source of its cycling/channel ability) and CR 602.2a/702.29a, but I only remember the official ruling; both scenarios carry a `kind: ruling, verified: false` source. If the implementer reads Needle as "permanents only", these two fail; I believe that reading is wrong.
2. **Battleship targeting itself (damage still dealt).** Relies on 113.7a (ability independent of its source) and last-known information for "this Spacecraft deals 4 damage". No ruling cited, not retrieved. Marked `derived_from: cr`.
3. **Ugin scenario** uses the brief's Oracle text and starting loyalty 7 (set in setup, per brief). Ability indices: +2 = 0, 0 = 1 (the two triggers are not activated). The -11 text is not given in the brief and not used.
4. **Torpor Orb + Bowmasters** assumes a two-event trigger is suppressed only for the enters event. That is the plain reading of "creatures entering don't cause abilities to trigger" (603.2g-style: the ability triggers only on events that occur). No ruling cited.
5. **Spacecraft not a creature**: scenarios treat Extinguisher Battleship as a plain noncreature artifact (it only becomes a creature when stationed, out of scope). The `types: [Artifact]` assertions rely on that.

## Schema gaps and conventions I had to rely on

- **Play from exile** (Research Desk): `{t: play_land, card: "@x"}` / `{t: cast, card: "@x"}` on a card that is in exile with a play permission; `expect_illegal` of the same for the unchosen card. The "choose one of them" step is `{decision: choose_cards, purpose: choose_one_to_play, answer: ["@x"], expect_options: {include: [both], count: 2}}`; the adapter owns the exact shape. I assert no expiry (out of scope) and never use unearth.
- **Choose a card name** (Needle): `choose_name` raised while the Needle is already on the battlefield with an empty stack; `check` with `pending: {actor: p0, kind: choose_name}` assumes that DecisionKind spelling.
- **Ability on a stack whose source is in the graveyard** (sacrificed as cost): `{kind: activated, source: "@desk"}` in `stack:` and `targets: [{ability_of: "@crypt", nth: 0}]` for Stifle; the alias still refers to the same card after it moved zones.
- `expect_illegal` with `decision: declare_attackers` (Bridge): a declaration with a forbidden creature is rejected as a whole; a decision with at least one legal attacker is still raised (each Bridge scenario keeps one legal attacker so `advance` is not needed to pass an empty declaration).
- Several scripts use explicit passes (two per step) to reach declare attackers (Stock Up/Bridge) and `advance` for combat/turn transitions (Nethergoyf block, Coating cleanup). They assume no other decision arises on the way (padded libraries, empty or small hands, no triggers). The Coating cleanup scenario advances through p0's end step into p1's upkeep (`advance: {to: upkeep, of: p1}`).
- **Mana pools** are not asserted after Hole/Lotus Petal responses (the fizzle scenario), only after clearly stable points (Needle/Petal, Coating).
- Pool-card scenario assumptions: Nethergoyf's P/T (*/1+*) is set by graveyard card types (Lightning Bolt instant, Island land, Faithless Looting sorcery gives 3/4); `Phelia` and `Archon of Cruelty` etc. are pool cards used only as bodies.
- Mishra's Research Desk ability index 0 = the {1},{T},Sacrifice ability (unearth would be index 1); Expedition Map, Crypt, Coating, Voltaic Key, Pithing Needle: ability index 0 is their only activated ability.

## Not covered (by design)

Desk expiry and unearth; Karn/Trinisphere interactions (group A); Workshop/Nexus/Tower/Kozilek's Command (group C); Battleship station; Bridge with Raph & Mikey or haste-in-combat creatures; Torpor Orb leaving the battlefield mid-turn.
