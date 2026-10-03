# MTG Engine — Card DSL and Card Implementation

Status: DRAFT v0.1. Companion to `01-core-design.md` (references sections 3, 6, 8 from there) and `02-action-api-and-hidden-info.md`.

---

## 1. Goals

- Cards are **data** wherever possible. The brief's target: a DSL of composable primitives (triggers, costs, targets, effects, replacement hooks) covering roughly **70-80%** of the pool, with hand-written code only for the weird cards.
- Adding a card MUST NOT require changing the engine core. If it does, that is an RFC (doc 01 section 14), and a signal that a primitive is missing.
- Every mutation a card can cause goes through the same `ProposedEvent` pipeline as the rest of the engine (doc 01 section 9). Cards cannot bypass replacement effects, cannot touch hidden state they are not entitled to, and cannot do anything the effect VM does not offer.
- Cards are **resumable**: card behavior pauses for decisions and resumes (doc 01 section 6.2), so it compiles to bytecode, and hand-written cards are state machines, not host-language code that blocks.
- Cards are reviewable by someone who has the Oracle text and the rulings open: the data reads close to the card.

Non-goals: a general scripting language, Turing-complete card logic, support for cards outside the closed pool.

---

## 2. Pipeline and file format

```text
cards/<set-agnostic name>.card.ron   --parse/validate-->   CardIR   --lint-->   --compile-->   CardDb entry (bytecode + tables)
                    ^                                                                                ^
         Oracle snapshot (MTGJSON)                                                   oracle_hash, ruling_ids, impl_status, version
```

- **Surface syntax: RON** for v0. It maps one-to-one onto the Rust IR enums via `serde`, so there is no custom parser to write or get wrong, schema validation is free, and unknown variants fail loudly. A terser surface syntax can be layered on later if authoring volume justifies it; it would compile to the same IR.
- Card files carry: name, Oracle-text snapshot hash, mana cost, types, subtypes, supertypes, base P/T/loyalty, colors/color indicator, faces (split, adventure, transform where the pool needs them), abilities, and provenance (`implemented_by`, `oracle_snapshot_date`, `rulings_ids`).
- **Source of Oracle text and rulings:** a pinned MTGJSON or Scryfall bulk export (check terms and attribution requirements when we pick one; both are generally permissive but I have not verified current terms). Oracle text is used for the glossary (section 9) and for hashing; it is **never parsed by the engine**.
- `CardDb` build produces: compiled ability bytecode, keyword bitsets, trigger index (doc 01 section 7.2), static-ability registry, cost-modifier registry, and a `db_hash` stored in every `GameRecord`.
- The loader MUST reject: unknown primitives, cards that use rules features the mechanics inventory marks Out of scope (doc 01 section 16), abilities with `ChooseCards` from a hidden zone that lack a legal knowledge path (section 6), and missing glossary text.

---

## 3. Primitive vocabulary

All primitives are closed enums. Adding one is a reviewed change (doc 01 section 14.1, "core-adjacent").

### 3.1 Costs

Each cost primitive implements `can_pay(state, ctx)`, `enumerate_choices(state, ctx)` (equivalence-class collapsed), and `pay(...)` as events.

| Primitive | Notes |
|---|---|
| `Mana(cost)` | Including hybrid, Phyrexian, X, snow, colorless-specific symbols as pool needs |
| `Tap(selector)` / `TapSelf` / `Untap...` | Tap-cost summoning-sickness rule is enforced by the engine, not the card |
| `PayLife(n)` | Cannot pay more life than you have (except 0) |
| `Sacrifice(selector, count)` / `SacrificeSelf` | |
| `Discard(selector, count, mode: Choose|Random|Hand)` | `Hand` is Lion's Eye Diamond |
| `ExileFromZone(zone, selector, count)` | Force of Will's blue card; delve helpers; flashback exile |
| `ReturnToHand(selector, count)` | Daze |
| `RemoveCounters(kind, n)` | |
| `Reveal(selector)` | |
| `Composite([..])` | Ordered costs; payment order is per the CR |
| Cost modifiers | `Delve`, `Convoke`, `AdditionalCost(..)`, `ReduceBy(..)`, `IncreaseBy(..)` live in the rules-modifier registry (doc 01 section 10.5) |

### 3.2 Selectors (object filters)

Composable predicates over objects, used for targets, costs, "each", "all", and conditions:

`Zone(z)`, `Controller(You|Opp|Any|Source|ChosenPlayer)`, `Owner(..)`, `Types(..)`, `Subtypes(..)`, `Supertypes(..)`, `Color(..)`, `Colorless`, `Multicolor`, `Cmc(cmp, value)`, `Power(cmp, value)`, `Toughness(cmp, value)`, `Name(..)`, `NameInZone(..)`, `Tapped/Untapped`, `Attacking/Blocking`, `HasKeyword(..)`, `HasCounter(..)`, `Token/NonToken`, `Another`, `Self_`, `AttachedTo`, `EnchantedBy`, `WasCast`, `And/Or/Not`, `SharesTypeWith(..)`. Values inside selectors use section 3.4.

### 3.3 Targets

`TargetSpec { id, count: Range, filter: Selector, zone_scope, distinct: bool, divisible: Option<..>, optional: bool }`. Target legality at cast/activation/trigger time and at resolution is computed by one engine function; cards only declare specs. Protection, hexproof, shroud, and "can't be the target" are applied by the engine (doc 01 section 9.2).

### 3.4 Values

`Const(n)`, `X`, `Count(selector)`, `Life(player)`, `CmcOf(ref)`, `PowerOf(ref)`, `ToughnessOf(ref)`, `StormCount`, `CardsInZone(zone, player)`, `DistinctCardTypesInGraveyards`, `EventAmount` ("that much"), `Plus/Minus/Times/Max/Min/HalfRoundedUp/HalfRoundedDown`, `Chosen(local)`.

### 3.5 Effects (what resolves)

Grouped; each compiles to one or more VM ops, and each mutation is a `ProposedEvent`.

| Group | Primitives |
|---|---|
| Zone movement | `Move(selector, to: Zone, position, face, tapped, controller)`, `Destroy`, `Exile`, `Bounce`, `Sacrifice`, `Mill`, `Discard`, `PutOntoBattlefield`, `ReturnFromGraveyard` |
| Cards | `Draw(player, n)`, `Search(player, zone, selector, to, reveal, shuffle, may_fail)`, `Shuffle(player)`, `Scry(n)`, `Surveil(n)`, `Look(player, zone, n)`, `Reveal(...)`, `ReorderLibraryTop(...)`, `PutFromHandOnLibraryTop(n, order)` |
| Damage and life | `Damage(amount, to, source_override)`, `GainLife`, `LoseLife`, `SetLife`, `PreventDamage` |
| Counterspells | `CounterSpell(target)`, `CounterUnless(target, unless_pays)`, `CounterAbility(target)`, `Redirect(target)` (if the pool has Misdirection-style) |
| Objects | `CreateToken(def, count, controller)`, `Copy(target, new_targets_choice)`, `Attach(a, b)`, `AddCounters`, `RemoveCounters`, `Proliferate` (if needed), `Tap/Untap`, `Transform` (pool-gated), `Regenerate` (pool-gated) |
| Mana | `AddMana(mana_spec, restriction)`, `Choice(color)` |
| Continuous | `Until(duration, ModifyPT/GrantKeyword/GrantAbility/SetTypes/SetColors/ControlChange/CantAttack/CantBlock/CantCast/...)`, `LockedInSet` vs dynamic by construction |
| Replacement/prevention install | `InstallReplacement(duration, matcher, modifier)` |
| Delayed | `Delay(event_pattern, effect)`, `Reflexive(effect)` |
| Control flow | `Sequence([..])`, `If(cond, then, else)`, `Choose(modal options)`, `May(effect)`, `Unless(player, cost, effect)`, `ForEach(selector, effect)`, `Repeat(n, effect)` |
| Special | `ExtraTurn`, `ExtraCombat`, `WinGame`, `LoseGame`, `CastWithoutPaying(selector, from)`, `PlayFrom(zone, ...)`, `Flicker(..)`, `Custom(op_id)` (section 7) |

### 3.6 Events (for triggers and replacement matchers)

The `EventKind` enum from doc 01 section 7.1: ZoneChange (with from/to filters), Draw, Discard, Damage, LifeChange, SpellCast, AbilityActivated, SpellCountered, TriggerPutOnStack, CounterPlaced, TokenCreated, TurnBegins/StepBegins, AttackDeclared, BlockDeclared, PermanentTapped, ShuffleLibrary, and a few others as the pool requires. A trigger is `Trigger { on: EventPattern, condition: Option<Condition>, intervening_if: Option<Condition>, zone_of_interest: Zones, optional: bool, effect, targets }`.

### 3.7 Static and continuous abilities

`Static { layer, applies_to: Selector (dynamic) , effect: ContinuousSpec }` covering: P/T modifications, keyword grants, type changes (Blood Moon), color changes, ability removal, "can't" restrictions, cost modification, characteristic-defining abilities. Replacement effects as statics: `Replacement { matcher: EventMatcher, modifier: EventModifier, optional: bool, self_replacement: bool }`.

### 3.8 Keywords

Evergreen and Legacy-relevant keywords are **engine intrinsic** (flying, first strike, double strike, deathtouch, lifelink, haste, hexproof, shroud, indestructible, menace, reach, trample, vigilance, flash, defender, protection, etc.). A card lists them as `Keywords([..])` and the engine applies them. Complex keyword abilities (cycling, flashback, kicker, storm, cascade, delve, convoke, madness, suspend, evoke, equip, ninjutsu, annihilator, miracle, ...) are implemented as **macro expansions in the loader** into the primitives above, in one reviewed place, so each is written once and tested once. Pool-gated per section 16 of doc 01.

---

## 4. Worked examples (syntax illustrative, not final)

```ron
// Lightning Bolt
Card(
  name: "Lightning Bolt", cost: "{R}", types: [Instant],
  abilities: [
    Spell(
      targets: [TargetSpec(id: 0, count: Exactly(1), filter: AnyOf([Creature, Planeswalker, Player]))],
      effect: Damage(amount: Const(3), to: Target(0)),
    ),
  ],
)

// Brainstorm
Card(
  name: "Brainstorm", cost: "{U}", types: [Instant],
  abilities: [
    Spell(effect: Sequence([
      Draw(player: You, n: Const(3)),
      PutFromHandOnLibraryTop(player: You, n: Const(2), order: PlayerChoice),
    ])),
  ],
)

// Daze: alternative cost + counter unless its controller pays {1}
Card(
  name: "Daze", cost: "{1}{U}", types: [Instant],
  abilities: [
    AlternativeCost(cost: [ReturnToHand(selector: And([Subtype(Island), Controller(You), Zone(Battlefield)]), count: Const(1))]),
    Spell(
      targets: [TargetSpec(id: 0, count: Exactly(1), filter: Zone(Stack))],
      effect: CounterUnless(target: Target(0), unless_pays: Mana("{1}")),
    ),
  ],
)

// Polluted Delta
Card(
  name: "Polluted Delta", types: [Land],
  abilities: [
    Activated(
      cost: [TapSelf, PayLife(Const(1)), SacrificeSelf],
      effect: Search(player: You, zone: Library, selector: Or([Subtype(Island), Subtype(Swamp)]),
                     to: Battlefield, tapped: false, reveal: false, shuffle: true, may_fail: true),
    ),
  ],
)

// Young Pyromancer
Card(
  name: "Young Pyromancer", cost: "{1}{R}", types: [Creature], subtypes: [Human, Shaman], pt: (2, 1),
  abilities: [
    Triggered(
      on: SpellCast(caster: You, filter: Or([Instant, Sorcery])),
      effect: CreateToken(def: "Elemental 1/1 red", count: Const(1), controller: You),
    ),
  ],
)

// Nethergoyf: characteristic-defining ability in layer 7a (Barrowgoyf is the same with all graveyards)
Card(
  name: "Nethergoyf", cost: "{B}", types: [Creature], subtypes: [Lhurgoyf],
  abilities: [
    Static(layer: L7a, applies_to: Self_,
      effect: SetPT(power: DistinctCardTypesInGraveyard(You), toughness: Plus(DistinctCardTypesInGraveyard(You), Const(1)))),
  ],
)
```

The "Stifle-ability" targets are works of the engine, not the card: Polluted Delta's activation puts an ability object on the stack, which a Stifle effect can target (doc 01 section 6.1).

---

## 5. Compilation to bytecode (effect VM)

Effect trees compile to a flat instruction list per ability. Instruction set (closed; frozen with the core):

| Op | Meaning |
|---|---|
| `Emit(ProposedEventTemplate)` | Build a `ProposedEvent` from locals/values and run it through the pipeline (replacement effects included) |
| `Choose { who, kind, source, count, bind }` | Suspend and ask `who` (a `Decision` with the engine-computed legal mask); bind the answer to a local. This is the only way a card asks for input |
| `EvalValue(expr) -> local` | Compute a value from state/locals |
| `Jump`, `JumpIf(cond)`, `Loop(counter)` | Control flow |
| `ForEachBegin/Next(selector)` | Iterate a snapshot of a selector result (snapshot taken at loop start) |
| `PushDelayed(pattern, code)` | Register a delayed trigger |
| `InstallContinuous(spec)` / `InstallReplacement(spec)` | Create continuous or replacement effects |
| `RevealToSeats(zone_slice, seats)` | Knowledge update (doc 02 section 6) |
| `CustomOp(op_id)` | Dispatch to a registered state-machine op (section 7) |
| `End` | Finish; engine moves the spell/ability object to its final zone |

A `VmState { code_ref, pc, locals: [Value; N], iter_stack }` is plain data, stored in a `ResolveObject` frame. Resume after a decision is: write the answer into `locals`, `pc += 1`.

Values captured at resolution time (targets, modes, X, "that much") live in locals and in the stack object, so effects behave correctly after the source has left (last-known information).

---

## 6. Interaction with hidden information

1. **A card can choose from a zone only if the chooser may see it.** `Choose` with `source: Zone(z)` is legal for a chooser only if `z` is visible to them. Cards that look at a hidden zone (Thoughtseize, Duress, Gitaxian Probe, Jace's Brainstorm-like "look at top") must include `RevealToSeats` first. The loader lints this.
2. **Library search** presents a sorted multiset (doc 02 section 9.4).
3. **Random effects** (random discard, shuffle) draw from the game's RNG inside the VM, never from agent input.
4. **Knowledge updates are part of ops**, not left to card authors: `Draw`, `Look`, `Scry`, `ReorderLibraryTop`, `Shuffle`, `Search`, and `Reveal` update `Knowledge` automatically (doc 02 section 6). A card cannot reorder a library without declaring it.
5. **Simultaneous secret choices** use `Choose { who: BothPlayers, secret: true }`, which becomes the paired decision in doc 02 section 9.1.

---

## 7. Hand-written cards and custom ops

For cards the DSL cannot express (see the list in section 8), engineers register a `CustomOp`:

```rust
pub trait CustomOp: Send + Sync {
    const ID: CustomOpId;
    // A pure state machine: all progress lives in `locals`, which the VM stores in state.
    fn step(&self, cx: &mut OpCx, locals: &mut [Value; 4], answer: Option<Answer>) -> OpResult;
}
pub enum OpResult { Done, NeedDecision(DecisionSpec), Continue }
```

Rules:

- **No blocking and no host-stack state.** A custom op that needs a choice returns `NeedDecision` and is called again with the `Answer`. This keeps the game clonable and resumable (doc 01 R3, R5).
- **No direct state mutation.** `OpCx` offers only: read views the card is entitled to, emit `ProposedEvent`s, request decisions, update `Knowledge` via the sanctioned helpers, install effects. It has no `&mut State`.
- Each custom op requires: a written spec (what Oracle says, CR citations, rulings), at least the rulings scenarios authored by the spec agent (doc 04 section 4), review by an agent or human who did not write it, and a differential scenario set vs Forge (doc 04 section 5).
- Custom ops are enumerated in `custom_ops.md` with owner, rationale (why the DSL could not express it), and the proposed generalization if one exists. A recurring pattern (two or more cards needing the same op) becomes a new DSL primitive through the core-adjacent process.

---

## 8. Hard-card list (pool-specific)

The pool is the final eight decks in `/mnt/project-files/decks/` (UR Cutter, Alurentell, Boros Aggro, BW Death and Taxes, Dimir Tempo, UWx Control, Doomsday, Reanimator; 144 unique non-basic cards counting sideboards). The per-card feature inventory is `05-pool-mechanics-inventory.md`; this table lists the cards that stress specific machinery. "Plan" is a first guess at DSL, hybrid (DSL plus engine support) or custom op. (S) marks sideboard-only cards, gated after the main 60. Stifle is now a main-deck card (UWx Control), so the "abilities on the stack" scenario in doc 04 tests a real pool requirement.

| Card(s) | What it stresses | Plan |
|---|---|---|
| Force of Will, Force of Negation, Daze, Snuff Out, Massacre, Unmask, Solitude (evoke) | Alternative costs: exile-from-hand, return an Island (any land with the Island type), life or board conditions, turn-dependent availability, evoke sacrifice trigger | DSL |
| Aluren, Omniscience | Free-cast permission that grants flash, applies to **both** players, is an alternative cost (CR 118.9) so it does not stack with another one, and interacts with Lavinia (no mana spent) | Omniscience DSL; Aluren Custom or a rules-modifier primitive |
| Stifle, Consign to Memory, ward (Koma) | Targeting an ability object on the stack, "mana abilities can't be targeted", replicate copies with new targets, counter a colorless spell | Stifle and Consign hybrid; ward DSL macro |
| Hydroblast, Pyroblast, Flusterstorm (S), Veil of Summer | Color checks at cast and resolution, modal targeting, storm copies, "spells you control can't be countered", hexproof from two colors | DSL |
| Brainstorm, Ponder, Preordain, Consider, Flow State, Stock Up, Mishra's Bauble, Personal Tutor | Own-library knowledge, bottom placement "in any order", look at any player's top card, delayed draw at the next upkeep, search-to-top | DSL (+ `ReorderLibraryTop`, bottom-order choice) |
| Fetchlands, Prismatic Vista, surveil lands, MDFC lands, Wasteland, Karakas, Cavern of Souls, Ancient Tomb, City of Traitors, Boseiju, Carpet of Flowers | Cost vs effect split, land-type searches, enters tapped plus surveil trigger, "pay 3 life or enters tapped" on the MDFC back, nonbasic land destruction, legendary bounce, restricted mana with an uncounterable rider, channel from hand, triggered (stack) mana | DSL (Cavern hybrid) |
| Lion's Eye Diamond, Lotus Petal, Dark Ritual | Mana ability with discard-hand cost during cost payment, mana pool lifetime, sacrifice mana ability | DSL with CR 601.2g timing |
| Doomsday | Choose five from library and graveyard, order them, exile the rest, lose half life | Custom |
| Thassa's Oracle, Jace Wielder of Mysteries | Devotion count, top-X look, win on library size, draw-from-empty replacement | DSL / hybrid |
| Show and Tell, Stronghold Gambit | Simultaneous secret choices, simultaneous reveal and entry, "lowest mana value creature enters" | Custom (paired secret decision) |
| Reanimate, Animate Dead, Shallow Grave, Faithless Looting, Cabal Therapy, Collective Brutality | Graveyard targeting, life loss equal to mana value, Aura that rewrites its own enchant restriction, delayed exile, flashback, naming a card, escalate | Animate Dead Custom; the rest DSL |
| Griselbrand, Archon of Cruelty, Atraxa, Koma, Raph & Mikey | Pay-life draw ability (Bowmasters triggers), edict plus discard plus drain, reveal-ten per-type choice with random-order bottoming, uncounterable plus ward 4, reveal-until-creature put onto the battlefield tapped and attacking | DSL; Atraxa Custom; Raph & Mikey hybrid |
| Acererak the Archlich, the three dungeons | Conditional bounce-and-venture (the dungeon choice is strategic: avoid Tomb to loop with Aluren, finish Tomb when taxes stop the loop) on entry (loops with Aluren), choose-a-dungeon decision, command-zone dungeon with a venture marker, room choice, completion record per named dungeon, opponent "pay or lose" decisions in Tomb, state-based dungeon removal | Custom |
| Samwise the Stouthearted | The Ring emblem and Ring-bearer designation across four levels | Custom |
| Kaito, Bane of Nightmares | Ninjutsu from hand, planeswalker that is a creature on your turn (layers 4, 6, 7b), emblem, stun counters, "an opponent lost life this turn" | Custom |
| Ajani, Nacatl Pariah; Tamiyo, Inquisitive Student | Transform by exile-and-return, batch "one or more Cats die", third-draw-this-turn count, back-face planeswalkers | Hybrid |
| Orcish Bowmasters | Trigger on opponent's draws except the first in their draw step, amass, flash | DSL (needs per-turn draw history) |
| Containment Priest (S), Grafdigger's Cage, Spider-Woman | Replacement effects on entering without being cast, "can't enter" for creature cards in graveyards and libraries, enters-tapped for opposing artifacts and creatures | DSL |
| Gaddock Teeg, Lavinia, Deafening Silence, Voice of Victory, Defense Grid, Disruptor Flute, Null Rod | Cast restrictions and cost increases, per-turn spell counts, "no mana was spent" check, named-card choice as the permanent enters, activation restrictions | DSL (rules-modifier registry) |
| Quantum Riddler, Overlord of the Balemurk | Warp (alternative cost, delayed exile, recast from exile), draw replacement; impending counters and "not a creature" (layer 4) | DSL / hybrid |
| Nethergoyf, Phlage, Barrowgoyf | Escape with a "four or more card types among" subset, CDAs reading one or all graveyards (7a), mill-and-take trigger | DSL / hybrid |
| Dragon's Rage Channeler, Unholy Heat, Murktide Regent, Brazen Borrower | Delirium, delve exile record feeding counters, adventure | DSL (hybrid for Murktide) |
| Cori-Steel Cutter, Ocelot Pride, Amped Raptor, Guide of Souls, Mobilize (Voice of Victory) | Flurry, ascend and token copies, energy and cast-for-energy, reflexive "when you do" trigger, tapped-and-attacking tokens | DSL / hybrid |
| Skyclave Apparition, Cloak and Dagger, Phelia, Flickerwisp-style effects, Hide on the Ceiling | Exile-until-leaves links, last-known mana value on leave, flicker with delayed return, X targets | DSL |
| Sand Scout, Lazotep Quarry, Moonshadow, Bilbo | Desert search, "one or more land cards" once-per-turn trigger, token copy with exceptions, counters-gated trigger, cast from graveyard on attack | DSL / hybrid |
| Surgical Extraction, Dismember, Bitter Triumph, Prismatic Ending, Prismari Charm | Phyrexian mana, three-zone search, optional additional cost chosen at cast, converge, three modes with one or two targets | Hybrid / DSL |
| Magus of the Moon (S), Containment Priest (S) | Layer 4 and 6 on nonbasic lands (surveil, fetch and Wasteland abilities stripped), flash replacement | DSL |
| Triumph of Saint Katherine (S) | Miracle window, face-down exile pile shuffled back on top ("known to be in the top K") | Hybrid |
| Swords to Plowshares, Path to Exile, Erode, Fatal Push, Lightning Bolt, Pyroclasm (S), Meltdown (S), Abrade (S) | Opponent-chooses-to-search riders, revolt history, sweepers, modal destroy-or-damage | DSL |
| Planeswalkers: Kaito, Jace Wielder, Ajani and Tamiyo back faces | Loyalty costs, one activation per turn, attackable walkers, 0-loyalty SBA, emblems | DSL |

## 9. Oracle text and the cached prompt glossary

The brief puts Oracle text once in a cached prefix and card names only in per-decision prompts. The pipeline:

- The card DB stores each card's **Oracle text snapshot** (display only; never parsed) and a short **rules-notes** blurb for known engine behaviors where Oracle text alone misleads agents (for example, "Daze may be cast by returning an Island; the opponent may pay {1}").
- A `glossary(deck_a, deck_b) -> String` generator emits a stable, sorted text block for the union of cards in a matchup. It is deterministic so prompt caching works.
- `ActionDesc.label` and `ViewCard.name` use the exact card name from the glossary.

---

## 10. Provenance, versioning, status

Each card carries:

| Field | Purpose |
|---|---|
| `oracle_hash` | Hash of the Oracle text snapshot it was implemented against; an Oracle update flags the card for re-review |
| `impl: Dsl | Hybrid | Custom` | For the 70-80% coverage metric |
| `status` | `Draft` -> `Implemented` -> `Specified` (rulings scenarios present and reviewed) -> `DiffTested` -> `Gated` (part of a gated deck) |
| `implemented_by`, `reviewed_by` | Role-separation audit trail (doc 01 section 14.4) |
| `rulings_ids` | Gatherer/Scryfall ruling ids covered by scenarios |

A coverage report (`cards status`) lists per deck: counts by impl type, by status, cards missing scenarios, cards with open mismatches. This is the dashboard the deck gate (doc 04 section 8) reads.

---

## 11. Lints (run at DB build)

- Unknown or deprecated primitives; unreachable abilities.
- Hidden-zone `Choose` without a legal reveal path (section 6).
- Targets declared but unused; targets used but undeclared.
- Mana-cost parse and sanity (converted mana value matches Oracle).
- Abilities whose zone of interest is empty or wrong for the event (for example, a graveyard trigger declared with battlefield-only interest).
- Duplicate keyword grants, conflicting static abilities within a single card.
- Any card using a primitive flagged pool-gated that is not in the current mechanics inventory (doc 01 section 16).
