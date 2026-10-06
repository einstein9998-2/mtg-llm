# Storm scenarios, group C: notes for the implementer

42 scenarios, one file each, `scenarios/storm-C-<topic>.yaml`: Hexing Squelcher 17, Gaea's Will 14, Song of Creation 11. Written from the Oracle text in the brief and the Comprehensive Rules (2025-11-14, Savecraft `rules_search`). No engine source, patch or Forge output was read. Ids are `storm-c-...`.

Files in this group: `scenarios/storm-C-*.yaml`, `SCENARIO-NOTES-C.md`, `oracle-additions-C.json` (Hexing Squelcher, Gaea's Will, Song of Creation, plus Beseech the Mirror and Tendrils of Agony, which are group A's cards used by one Song scenario each; entries are identical to A's), `cr-excerpts-storm-C.md` (verbatim CR text; every `verified: true` CR source is in it; rules marked "(existing)" were copied from the Orim's Chant excerpts).

## Validation

Scratch copy of `reference/` with `oracle.json` + `oracle-additions-C.json` merged and `cr-excerpts-storm-C.md` added. Command (same shape as the Chant package):

`python3 validate_c.py --oracle <scratch>/refs/oracle.json --refs <scratch>/refs /mnt/project-files/rust-engine-storm/scenarios/storm-C-*.yaml` -> `42 scenario(s), 0 error(s), 0 warning(s)`.

`validate_c.py` is the stock `tools/validate.py` with ONE edit: `"suspend"` added to `ACTION_T` (line 19). The stock validator rejects `{t: suspend, ...}` as "unknown action" (three script steps in three files); with that one-word change to the real `tools/validate.py` the plain command works. Everything else it checks (aliases, card names, verified oracle and CR quotes) passes unmodified. `expect_illegal` steps and `legal:` patterns are not checked for `t`, so they would pass either way.

## New action and decision spellings (the runner adapter must learn these)

| spelling | where | meaning |
|---|---|---|
| `{t: suspend, card: "@will", pay: {tap: [...]}}` (`pay` optional, `auto` default) | Gaea's Will scenarios | the suspend special action from hand: pay {G}, exile face up with 4 time counters. Not a cast, no stack. In `legal:` patterns `{t: suspend, card: "@will"}`; `expect_illegal` is used where it must not be offered |
| exile descriptor `{card: "Gaea's Will", face_up: true, counters: {time: N}}` | setup `exile:` and `expect`/`check` `exile:` | time counters on an exiled card (setup and assertion). Count 0 is asserted by omitting `counters` |
| `bargained: true`, `sacrifice: ["@petal"]` on `{t: cast, ...}` | Song + Beseech | bargain announced as the spell is cast, with the sacrificed permanent named inline (same shape as `kicked:` in the Chant package and `sacrifice:` on activations) |
| `{decision: yes_no, purpose: cast_exiled_free, answer: true}` then `{decision: choose_target, slot: 0, answer: {player: p1}}` | Song + Beseech | the "may cast the exiled card without paying" choice and its target. Shapes are the adapter's; the outcome is what is fixed. `choose_target` with a player answer uses `{player: pN}` (SCHEMA only shows an alias answer) |
| `{decision: yes_no, purpose: pay_ward, answer: true|false}` | all ward scenarios | ward cost payment on the trigger's resolution (same as the existing Koma scenarios) |
| `{decision: yes_no, purpose: pay_for_daze, ...}`, `{decision: yes_no, purpose: change_targets, answer: false}` | Daze, storm copies | existing spellings |
| `{decision: order_triggers, answer: [{source: "@song", text_contains: "draw"}, {source: "@tend", text_contains: "copy"}]}` | two Song scenarios | existing spelling (first listed goes on the stack first); two triggers from different sources controlled by one player |
| `{decision: simultaneous_secret_choice, ...}`, `{decision: search, found: "@x"}` | Show and Tell, Beseech | existing |

Cast with the free alternative cost from Omniscience / Aluren is the existing `alt_cost: {free: omniscience|aluren}`; Gaea's Will cast with it is the only legal cast of the card from hand.

## What each scenario covers

### Hexing Squelcher (17)

| file | what it pins |
|---|---|
| cast-force-of-will-hard-cast | the Squelcher SPELL can't be countered: a hard-cast Force of Will may still be cast at it (offered), resolves, does nothing; the creature enters |
| cast-force-of-will-pitch-and-force-of-negation | pitch Force of Will is also wasted (1 life and the exiled card are not refunded); Force of Negation cannot target a creature spell at all |
| spells-you-control-force-of-negation | static "spells you control can't be countered" on the battlefield: pitched Force of Negation on Brainstorm does nothing; Brainstorm goes to the graveyard, NOT exile (the exile clause applies only if countered) |
| show-and-tell-uncounterable | Show and Tell with the Squelcher in play is not countered by a hard-cast Force of Will; p1 with no eligible card gets no choice |
| abilities-still-counterable | negative: Stifle still counters p0's Goblin Bombardment activation (spells only) |
| ward-self-pay-life | ward on the Squelcher: Bolt targets it, trigger on stack, p1 pays 2 life (18) on resolution, Bolt resolves, Squelcher dies |
| ward-self-decline | decline: Bolt countered, Squelcher undamaged, no life lost |
| ward-exactly-two-life | p1 at 2 life may pay (life >= 2, CR 119.4), goes to 0, loses (`game: over, winner p0`) |
| ward-one-life-cannot-pay | p1 at 1 life cannot pay: NO decision raised, Bolt countered, game continues |
| ward-other-creature-spell-pay | another creature (Ocelot Pride) is warded: pay 2 life, Bolt kills it |
| ward-other-creature-ability-decline | the ward also counters abilities: Goblin Bombardment ability of p1 countered on decline; sacrificed token gone anyway |
| ward-no-trigger-for-noncreatures-or-players | Abrade on Lotus Petal and Bolt at p0's face: no ward, no prompt |
| ward-not-for-own-spells | p0's own Bolt/Bombardment on its own creatures: no ward, no life paid |
| prismatic-ending-targets-and-exiles | uncounterable does not stop targeting: Prismatic Ending X=1 (W+U) targets the Squelcher, ward paid, Squelcher exiled |
| daze-prompt-offered-spell-resolves | Daze on a protected spell: the {1} payment is still offered; declined, the spell resolves anyway |
| veil-of-summer-unchanged | with Squelcher in play Veil still draws (opponent cast blue Bilbo) and still gives hexproof from blue (Hydroblast cannot target the Squelcher); red Bolt still targets it and meets ward |
| leaves-battlefield-spells-counterable-again | after the Squelcher dies (Bolt, ward paid) Force of Will counters p0's Bolt |

### Gaea's Will (14)

| file | what it pins |
|---|---|
| suspend-from-hand | `{t: suspend}`: pay {G}, exiled face up with 4 time counters, no stack, no cast (spells_cast 0) |
| cannot-be-hard-cast | no mana cost = unpayable (CR 118.6): no cast from hand with any mana; suspend offered; a non-green source cannot pay |
| suspend-sorcery-timing | suspend only in own main phase with an empty stack: illegal in upkeep, illegal with a spell on the stack, legal after |
| suspend-not-in-opponents-turn | illegal in the opponent's main phase even with priority and an empty stack |
| cast-from-exile-at-last-counter | one counter: upkeep trigger, then cast trigger, then Will cast free (a cast: spells_cast 1), resolves, exiled (not graveyard) |
| time-counters-count-down-only-on-own-upkeeps | 4 counters: nothing in p1's upkeep, 4 -> 3 -> 2 -> 1 -> 0 over p0's upkeeps of turns 5, 7, 9, 11; cast on the fourth |
| omniscience-free-cast | Omniscience makes the cast legal (CR 118.6a); a cast; Will exiles itself |
| graveyard-plays-and-exile-instead | after the Will: upkeep = instants only (no sorcery, no land); main phase = one land from graveyard (land drop limit), a sorcery; Bolt/Pyroclasm exiled; opponent's dying creature goes to p1's graveyard; cards already in graveyard are usable |
| countered-spell-is-exiled | a spell Force of Will counters goes to exile, Force of Will to p1's graveyard |
| lotus-petal-and-led-exiled | Petal sacrifice, LED sacrifice and the discarded hand are exiled; LED mana casts a graveyard Bolt; exiled Ponder cannot be cast |
| until-end-of-turn-only | next turn the graveyard Bolt cannot be cast, and a Bolt cast from hand goes to the graveyard |
| opponents-graveyard-unaffected | p1's graveyard Bolt not castable by p0; p1's Bolt goes to p1's graveyard |
| orims-chant-blocks-suspend | Chant on p0 in its upkeep: p0 cannot suspend (CR 702.62c) in main phase |
| orims-chant-blocks-last-counter-cast | Chant in response to the cast trigger: Will cannot be cast, stays exiled with no counters, never cast two turns later |

### Song of Creation (11)

| file | what it pins |
|---|---|
| cast-and-additional-land | cast {1}{G}{U}{R}, no draw for its own cast, then two land drops but not a third |
| additional-land-only-for-controller-on-own-turn | negative: opponent still has one drop, p0 cannot play a land in p1's turn |
| draw-two-on-cast-even-if-countered | trigger above the spell; Force of Will counters the Petal; the draw still happens |
| storm-copies-are-not-casts | Flusterstorm after p1's Bolt: one copy, one Song trigger (draw 2, not 4); p1's cast draws nothing; original fizzles after the copy counters Bolt |
| tendrils-storm-copy-not-cast | Lotus Petal then Tendrils: Song draws for both casts (4 cards), not for the copy; both resolutions drain 2 (p0 24, p1 16) |
| end-step-discard-own-end-step-only | nothing in p1's end step; in p0's end step the trigger discards the hand |
| orims-chant-after-draw-trigger | Chant after the cast: the draw trigger on the stack still resolves, no further cast possible, end-step discard still happens |
| aluren-free-cast-draws | a free cast with Aluren is a cast: draw two |
| gaeas-will-suspend-cast-then-end-step-exile | Will cast from suspend in upkeep triggers Song; hand discarded at end step is EXILED (Will's replacement) |
| beseech-free-cast-draws-twice | bargained Beseech and the free cast of the found Bolt are two casts: draw 2 each |
| no-draw-for-abilities-lands-or-opponents-spells | land play, Petal ability and p1's Bolt draw nothing |

## Assumptions about the runner (not rulings)

- `advance: {to: upkeep, of: p0}` stops when p0 has priority in the upkeep WITH the upkeep trigger already on the stack (several scenarios check the stack there). Likewise `advance: {to: end, of: p0}` stops with Song's end-step trigger on the stack. If the runner stops before the trigger is put on the stack, those checks need an extra pass step; the spec thread should say so.
- Setups start in p1's end step (`phase: end`, `active: p1`) and advance into p0's turn, to avoid assuming whether a setup at `phase: upkeep` has already put the upkeep trigger on the stack.
- `exile:` setup/assertion entries with `counters: {time: N}` (see above). The exile `counters` key is already in the validator's descriptor vocabulary.
- Each `resolve_top` after a trigger or spell where p0 is the active player: p0 holds priority and passes first.
- A decision with no real choice is not raised: ward when the payer has fewer than 2 life (`ward-one-life-cannot-pay`), p1's Show and Tell with no eligible card, Flusterstorm/Daze payment when the payer has no untapped mana.

## Rulings and details I was unsure about

1. **Daze on an uncounterable spell is still offered the payment** (`daze-prompt-offered-spell-resolves`). From CR 118.12a ("unless" = the player may pay; if they don't, counter) plus 101.2 (the counter can't happen). I did not retrieve an official ruling. If the engine decides to skip the prompt when the counter can't happen, the `yes_no` step must be dropped (the spec thread decides); the outcome (spell resolves, no mana spent) is identical.
2. **Gaea's Will exiles itself** (all Will scenarios). Basis: its replacement effect starts to apply during resolution; as the final part of resolution the spell is put into the graveyard (CR 608.2n), so the replacement applies to it. Not backed by a retrieved ruling.
3. **Ward payment timing**: the life is paid when the ward trigger RESOLVES (CR 702.21a + 118.12a), not on cast; the scenarios assert life 20 before and 18 after. A payment prompt raised for a player who cannot pay (1 life) would not match.
4. **Paying life down to 0 is legal** (CR 119.4: life total >= payment) and the payer then loses to the state-based action (704.5a): `ward-exactly-two-life`.
5. **Hexproof from blue vs ward**: Veil of Summer's hexproof stops a blue spell targeting the Squelcher but not a red spell; the ward trigger of a red spell still happens. Not tested: a blue spell targeting another creature while Veil is active.
6. **Storm count and copies**: copies are not cast (CR 707.10), so they neither draw with Song nor count toward storm.
7. **Free cast during resolution (Beseech)**: the Bolt is put on the stack during Beseech's resolution, Beseech then finishes resolving, and Song's trigger is put above the Bolt; the scenario asserts the stack only after that, so it does not depend on how the engine orders those internal steps.
8. `suspend-sorcery-timing` / `suspend-not-in-opponents-turn` rest on CR 702.62a "if you could begin to cast this card by putting it onto the stack from your hand" and 307.1 for sorceries.
9. **Hexing Squelcher engine simplification**: scenarios only look at outcomes (pay 2 life or the spell/ability is countered) and never at which object is the source of a ward trigger on another creature: the stack assertions for those use `kind: triggered` + `text_contains: "counter"` without `source`, except where the Squelcher itself is the target.

## What I did not write / could not express

- Gaea's Will "nothing to cast if no legal targets": the card has no targets, so this suspend case does not exist. The "unable to cast" case is covered through Orim's Chant (`orims-chant-blocks-last-counter-cast`). Declining the free cast is not expressible (engine simplification, brief).
- The existing-schema `legal:` pattern for a suspend with a payment of a specific source (`pay`) is only used in `expect_illegal` steps.
- Gaea's Will + Beseech the Mirror found-card casting is group A's.
- Stifle on the ward trigger, and the Squelcher leaving at the instant of targeting: excluded by the brief.
- No `symmetric: true` scenarios; every Squelcher/Will/Song line has p0 as the Storm player and p1 as the Alurentell player by construction (some scenarios have p1 as the active player so that p1's instants/sorceries are cast in its own turn).
