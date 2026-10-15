# Eureka scratch implementation: code excerpts (not applied to crates/, core is frozen)

Taken from a scratch copy of the payfix2 tree. Excerpts only; line numbers are from that tree.

## ir.rs (Effect enum)
```rust
    EachPutFromHand { filter: ObjFilter },
    /// Eureka: starting with the controller, each player in turn may put one card matching the filter
    /// from their hand onto the battlefield (it enters at once); repeat until a whole round passes
    /// with no card put onto the battlefield. Triggers wait until the spell has finished resolving.
    PutLoop { filter: ObjFilter },
    /// Exile the top `n` cards of your library; you may play them for as long as they stay exiled.
    ExilePlay { n: Expr },
```

## lint.rs
```rust
        Effect::EachPutFromHand { .. } | Effect::PutLoop { .. } | Effect::Venture | Effect::MarkUsed(_) | Effect::AllowCastMoved | Effect::TapAttackMoved => -1,
```

## resolve.rs: the effect
```rust
                }
            }
            cx.end_batch();
            Flow::Next
        }
        Effect::PutLoop { filter } => {
            // Eureka. aux = step (even: the controller, odd: the other player); sc[0] = cards put this round.
            let order = [env.controller, env.controller.other()];
            loop {
                let step = *aux as usize;
                if step >= 64 {
                    break;
                }
                if step > 0 && step % 2 == 0 {
                    if sc[0] == 0 {
                        break;
                    }
                    sc[0] = 0;
                }
                let seat = order[step % 2];
                let cands: Vec<ObjRef> = cx.s.players[seat.idx()].hand.clone().into_iter().filter(|&r| {
                    let c = cx.chars(r);
                    cx.filter_match(filter, r, &c, false, seat)
                }).collect();
                if cands.is_empty() {
                    *aux += 1;
                    continue;
                }
                let mut opts = card_options(cx, seat, &cands);
                opts.push(Opt::Done);
                ask(cx, seat, DecisionKind::ChooseCards { purpose: CardsPurpose::PutOnto, remaining: 1 }, opts);
                return Flow::Await;
            }
            Flow::Next
        }
        Effect::CopyToken(o) => {
            cx.s.moved.clear();
            for r in objs_of(cx, env, o) {
                if cx.s.is_live(r) && cx.s.obj(r).kind == ObjKind::Token {
                    let def = cx.s.obj(r).def;
                    if let Some(t) = cx.create_token(def, env.controller) {
                        cx.s.moved.push(t);
                    }
                }
            }
            Flow::Next
```

## resolve.rs: feed (decision routing)
```rust
    let (dest, put_to) = {
        let code = entry_code(cx.db, &f.entry);
        match code.get(f.pc as usize) {
            Some(Instr::Leaf(Effect::Search { dest, tapped, .. })) => (Some((*dest, *tapped, true)), None),
            Some(Instr::Leaf(Effect::SearchNoShuffle { dest, .. })) => (Some((*dest, false, false)), None),
            Some(Instr::Leaf(Effect::PutChosen { to, tapped, .. })) => (None, Some((*to, *tapped))),
            Some(Instr::Leaf(Effect::PutLoop { .. })) => (None, Some((ZoneKind::Battlefield, false))),
            Some(Instr::Leaf(Effect::TakeMoved { to, .. })) => (None, Some((*to, false))),
            _ => (None, None),
            let n = f.sc[0] as i32;
            cx.add_counters(r, CounterKind::PlusOne, n);
            f.aux = 1;
            Next::Stay
        }
        (DecisionKind::ChooseCards { purpose: CardsPurpose::PutOnto, .. }, o) => {
            let looping = matches!(entry_code(cx.db, &f.entry).get(f.pc as usize), Some(Instr::Leaf(Effect::PutLoop { .. })));
            if let (Opt::Card(r), Some((to, tapped))) = (o, put_to) {
                cx.move_zone(r, to, MoveOpts { tapped, controller: if to == ZoneKind::Battlefield { Some(p.seat) } else { None }, ..Default::default() });
                if looping {
                    f.sc[0] += 1;
                }
            }
            if looping {
                f.aux += 1;
            } else {
                f.aux = 1;
            }
            Next::Stay
        }
        (DecisionKind::ChooseCards { purpose: CardsPurpose::PlayChosen, .. }, o) => {
            f.aux = 1;
            if let Opt::Card(r) = o {
                cx.s.exile_plays.push((r, p.seat));
            }
            Next::Stay
        }
        (DecisionKind::ChooseCards { purpose: CardsPurpose::ExileUntilLeaves, .. }, o) => {
            f.aux = 1;
            if let (Opt::Card(r), StackKind::Ability { source, .. }) = (o, f.entry.kind) {
                // The source must still be on the battlefield (610.3b).
```

## legacy.rs registration
```rust
pub const EUREKA_CARDS: &str = include_str!("../cards/eureka.cards.ron");
```

## Scenarios (own-scenarios/scenarios/own.yaml)
```yaml
---
id: own-eureka-alternating-puts-repeat-until-a-round-is-empty
title: Eureka lets the caster and then the opponent put one permanent card each, round after round, until nobody puts a card
area: card
setup:
  turn: 3
  active: p0
  phase: main1
  p0:
    hand: ["Eureka @eu", "Island @i1", "Mountain @m1"]
    battlefield: [Forest, Forest, Forest, Forest]
    library_pad: 10
  p1:
    hand: ["Forest @f1", "Plains @p1"]
    library_pad: 10
script:
  - p0: {t: cast, card: "@eu", pay: auto}
  - resolve_top: true
  - p0: {decision: choose_cards, answer: ["@i1"]}
  - p1: {decision: choose_cards, answer: ["@f1"]}
  - p0: {decision: choose_cards, answer: ["@m1"]}
  - p1: {decision: choose_cards, answer: ["@p1"]}
expect:
  - at: after_script
    p0:
      hand: []
      battlefield_contains: ["Island", "Mountain"]
      graveyard: ["Eureka"]
    p1:
      hand: []
      battlefield_contains: ["Forest", "Plains"]
---
id: own-eureka-caster-declining-does-not-end-it-while-the-opponent-put-a-card
title: Eureka goes on after the caster declines in a round where the opponent put a card (the harness declines the later prompts itself, so a card left in hand stays there)
area: card
setup:
  turn: 3
  active: p0
  phase: main1
  p0:
    hand: ["Eureka @eu", "Island @i1", "Mountain @m1"]
    battlefield: [Forest, Forest, Forest, Forest]
    library_pad: 10
  p1:
    hand: ["Forest @f1"]
    library_pad: 10
script:
  - p0: {t: cast, card: "@eu", pay: auto}
  - resolve_top: true
  - p0: {decision: choose_cards, answer: []}
  - p1: {decision: choose_cards, answer: ["@f1"]}
  - p0: {decision: choose_cards, answer: ["@m1"]}
expect:
  - at: after_script
    p0:
      hand: ["Island"]
      battlefield_contains: ["Mountain"]
    p1:
      hand: []
      battlefield_contains: ["Forest"]
```
