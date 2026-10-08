# Group B scenarios (Pyrokinesis, Price of Progress, Boomerang Basics): notes for the implementer

29 scenarios, one file each, `scenarios/ur-B-<topic>.yaml` (12 Pyrokinesis, 9 Price of Progress, 8 Boomerang Basics). Written from the Oracle text in SCENARIO-BRIEF.md (re-checked with Savecraft `card_search`, 2026-10-08) and the Comprehensive Rules (2025-11-14, Savecraft `rules_search`). No engine source, patch, `ur.cards.ron` or gap document was read, and no `mcp__hearthbot__*` tool was called. Every scenario is `derived_from: oracle` or `cr`, with no `ruling` sources and no `verified: false` entries.

Revision after the first engine run (2026-10-08): the card pool is closed, so every scenario that used Auriok Salvagers, Wirewood Savage, Cavern Harpy, Dryad Arbor, Imperial Recruiter, Raven Familiar, Paladin en-Vec or Darksteel Myr was rewritten with pool creatures (Street Wraith, Guide of Souls, Containment Priest, Ocelot Pride, Thassa's Oracle, Orcish Bowmasters) or dropped. Dropped because no pool card has the property: protection from red, an indestructible target, Dryad Arbor as a legal Pyrokinesis target / illegal Boomerang Basics target (no land creature in the pool). Also fixed: the Otter's prowess trigger in the token scenario, and the `legal` exile-red-card check in the "only red cards qualify" scenario (removed; the three `expect_illegal` steps and the Charm cast remain).

## Delivery and validation

- `oracle-additions-B.json`: `cards` (Pyrokinesis, Price of Progress, Boomerang Basics, Stormchaser's Talent, plus Urza's Tower / Mine / Power Plant copied unchanged from the Tron package so these scenarios validate stand-alone) and `tokens` (`Otter Token`). The Talent and Otter entries use the same keys as group A. No creature card is added; all creatures used are in `reference/oracle.json`.
- `cr-excerpts-ur-B.md`: every CR rule a `verified: true` source cites (it may hold a few extra rules from dropped scenarios; harmless). Rules 105.2, 105.2b, 120.3a, 120.3e, 205.4c, 400.3, 601.2d, 608.2b (the sentences quoted), 716.2b and 716.2d were fetched with a `rule` lookup on 2026-10-08; the rest were already present with identical wording in the earlier excerpt files (same module).
- Validation (scratch copy of `reference/oracle.json` with the additions merged; my excerpts file as the only cr-excerpts in the refs dir):
  `python3 /mnt/project-files/rust-engine-spec/tools/validate.py --oracle <scratch>/refs/oracle.json --refs <scratch>/refs /mnt/project-files/rust-engine-ur/scenarios/ur-B-*.yaml` -> `29 scenario(s), 0 error(s), 0 warning(s)`.

## Scenario list (what each asserts that an engine could get wrong)

| file (ur-B-...) | asserts |
|---|---|
| pyrokinesis-alt-cost-exile-red-card-one-target-takes-four | no lands: only the alt cost works; red card exiled at cast (visible while the spell is on the stack); one target (Street Wraith 3/4) takes all 4 |
| pyrokinesis-hard-cast-six-mana-two-plus-two | `{4}{R}{R}` paid from 2 Mountains + 4 Islands; 2+2 on a 1/2 and a 2/2; red card in hand not exiled |
| pyrokinesis-alt-cost-only-red-cards-qualify | blue card, colorless artifact, Pyrokinesis itself are illegal exile choices (`expect_illegal`); a blue-red card (Prismari Charm) is accepted |
| pyrokinesis-three-plus-one-damage-stays-marked | 3 on a 3/4 (survives, damage 3) and 1 on my own 1/1 (dies); a later Bolt kills the damaged 3/4 |
| pyrokinesis-one-one-one-one-four-tokens | four targets at 1 each; tokens die and vanish; a non-target creature is untouched |
| pyrokinesis-one-one-two-lethal-and-nonlethal-shares | 1/1 dies, 1/2 takes 1 and lives, 2/2 takes exactly lethal 2 |
| pyrokinesis-own-creature-bolted-in-response-share-lost | p1 Bolts the Oracle (my 1/3, share 1) in response; its share is lost, Street Wraith takes exactly its 3 |
| pyrokinesis-all-targets-illegal-fizzles | both targets sacrificed to Goblin Bombardment in response; spell does not resolve, goes to the graveyard |
| pyrokinesis-targets-creatures-only | Aluren, Island, Lotus Petal illegal alone or in a split; Ocelot Pride legal |
| pyrokinesis-force-of-will-exile-cost-stays-paid | countered by Force of Will; the exiled red card stays exiled; target undamaged |
| pyrokinesis-daze-tapped-out-countered | alternative cost is not "free" for Daze: with no mana to pay {1} the spell is countered |
| pyrokinesis-aluren-bowmasters-trigger-survives-pyrokinesis-on-its-source | free Orcish Bowmasters (Aluren); Pyrokinesis in response kills it, the enters trigger still pings the Pride and amasses |
| price-of-progress-counts-only-nonbasic-lands-for-each-player | duals, Ancient Tomb, City of Traitors, uncracked fetchland, Hedge Maze, Tron lands count; basics do not; each player's own count (14 / 10) |
| price-of-progress-zero-nonbasic-lands-deals-zero | 0 damage to both, spell still resolves |
| price-of-progress-fetchlands-cracked-in-response-basic-versus-nonbasic | sacrificed fetchlands stop counting; one fetch finds a basic (0), the other a dual (counts); fetch life paid |
| price-of-progress-wasteland-in-response-lowers-both-counts | Wasteland sacrificed and destroys the caster's Volcanic Island: counts taken on resolution |
| price-of-progress-lethal-for-one-player | opponent at 6 with 3 nonbasics dies, caster survives |
| price-of-progress-both-players-at-zero-is-a-draw | both players at 0 in one event: draw (CR 104.4a) |
| price-of-progress-city-of-traitors-trigger-response-still-counts | land play, City's sacrifice trigger on the stack, PoP in response still counts City |
| price-of-progress-force-of-will-counters-no-damage | countered by Force of Will, no damage |
| price-of-progress-daze-paid-with-last-land-resolves | Daze, real `yes_no` pay prompt, pays {1}, spell resolves |
| boomerang-basics-own-artifact-returns-and-draws | own noncreature permanent (Cori-Steel Cutter): returns, draw |
| boomerang-basics-opponents-aluren-no-draw-and-free-casts-end | opponent's Aluren: returns to their hand, no draw; Aluren works while BB is on the stack (free Ocelot Pride offered), not after |
| boomerang-basics-cannot-target-lands | Island and Volcanic Island illegal; opponent's creature legal, no draw |
| boomerang-basics-token-ceases-to-exist-and-still-draws | own Otter token (prowess trigger resolves first, Otter 2/2): no card in hand, still draws |
| boomerang-basics-class-returns-and-recast-starts-at-level-one | own level 2 Talent: bounce, draw, recast, level counter 1, new Otter |
| boomerang-basics-target-removed-in-response-no-draw | target exiled by Swords to Plowshares in response: spell does not resolve, no draw |
| boomerang-basics-owner-and-controller-differ-returns-to-owner | returns to the OWNER; draw depends on control, not ownership |
| boomerang-basics-bounces-own-omniscience-free-cast-draws | BB cast free via Omniscience bounces Omniscience, draws; second BB no longer castable |

## Rulings I was less sure of (all derived from CR plus Oracle)

1. **Zero targets for Pyrokinesis / the brief's "can't be cast with no creature".** The brief's group B list asks for it, but the brief's own "do NOT write" list says zero targets is a documented engine divergence. By the Oracle text ("any number of target creatures") the spell can be cast with zero targets. I wrote NO scenario that asserts it is castable or uncastable with zero targets.
2. **Tapped-out Daze victim gets no prompt.** In `pyrokinesis-daze-tapped-out-countered` p0 controls no lands, so "unless its controller pays {1}" gives no real choice and no `yes_no` is scripted. The paid case with a real prompt is `price-of-progress-daze-paid-with-last-land-resolves`.
3. **Both players at 0 life from one Price of Progress**: the spell is one effect, and I treat its damage as simultaneous (704.3 checks all state-based actions as one event, 104.4a draw). I found no CR sentence saying outright that one spell's damage to several players is simultaneous; marked `derived_from: cr`.
4. **Orcish Bowmasters scenario**: the trigger target is chosen when the trigger goes on the stack (`choose_target, slot: 0`), the damage uses last known information of the dead Bowmasters, and amass creates a 0/0 Orc Army token with one +1/+1 counter (asserted as `pt: [1, 1]`). The "trigger survives its source" part rests on 113.7a; the amass behavior comes from the Oracle text and the existing `Orc Army Token` entry.
5. **Setup with different owner and controller** (`boomerang-basics-owner-and-controller-differ-returns-to-owner`): a card is put in its owner's battlefield list with `controller: <other seat>` (as expectations do in `animate-dead` and `aluren-seat-symmetry`), and the other seat's `battlefield` is asserted as control. If the runner reads setup lists as controller lists, that one setup needs a different spelling.
6. **Class level representation.** The Talent is placed with `counters: {level: 2}` (brief). After the bounce and recast I assert `counters: {level: 1}` (716.2d, 716.2b + 400.7). The setup has no Otter on purpose (an Otter would add prowess triggers for the Boomerang Basics and Talent casts).
7. **Otter prowess in the token scenario**: casting Boomerang Basics (a noncreature spell) triggers the Otter's prowess; the trigger sits above the spell, resolves first (Otter 2/2), then the spell bounces the token. Two `resolve_top` steps with checks in between.
8. **Pool-only Aluren**: `opponents-aluren` uses Ocelot Pride (mana value 1) as the free cast; the Bowmasters scenario uses a free Orcish Bowmasters (mana value 2).

## Schema gaps and things I did not express

- `divide: {"@a": n, ...}` and `alt_cost: {exile_red_card: "@x"}` are used as the brief specifies; the validator accepted both (also inside `expect_illegal`). `exile_red_card: "@pyro"` (the spell itself) is used once as an illegal probe. A `legal` pattern naming which card is pitched cannot be evaluated by the harness, so none is used.
- No assertion for damage that was prevented; only `damage: N` on a permanent descriptor.
- Not covered on purpose: counter-removal / proliferate on a Class, "becomes the target" triggers, a zero-target Pyrokinesis (engine divergences from the brief); protection from red and indestructible targets (no pool creature has them); land creatures (none in the pool).
- The open priority question from SCHEMA (priority after a mana ability) is avoided.
- Group A overlap: `ur-A-boomerang-own-talent-draws-and-recast-resets-to-level-1` covers the same bounce-the-Talent line from the Class side.
