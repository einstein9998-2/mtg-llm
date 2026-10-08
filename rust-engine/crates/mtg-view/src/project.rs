//! Projection from engine state to observations (doc 02 sections 2-4, 7).

use crate::game::Game;
use crate::view::*;
use mtg_core::card::CardDb;
use mtg_core::decision::*;
use mtg_core::event::Event;
use mtg_core::hash::Fx64;
use mtg_core::ids::*;
use mtg_core::state::{ObjKind, StackKind, State};
use mtg_core::types::*;
use std::hash::{Hash, Hasher};

fn name(db: &CardDb, d: CardDefId) -> String {
    db.def(d).name.clone()
}

fn card(s: &State, db: &CardDb, r: ObjRef, seat: Seat) -> ViewCard {
    let d = s.def_of(r);
    ViewCard { vid: s.view_id(r, seat), def: d, name: name(db, d) }
}

fn view_target(s: &State, db: &CardDb, t: Target, seat: Seat) -> ViewTarget {
    match t {
        Target::None => ViewTarget::None,
        Target::Player(p) => ViewTarget::Player { me: p == seat },
        Target::Obj(r) => {
            if s.is_live_ref(r) {
                ViewTarget::Object { vid: s.view_id(r, seat), name: name(db, s.def_of(r)) }
            } else {
                ViewTarget::Gone
            }
        }
    }
}

/// Exhaustive redaction of one event for one seat (doc 02 section 7). Adding an `Event` variant
/// is a compile error here until someone decides who may see it.
pub fn redact(db: &CardDb, e: &Event, seat: Seat) -> Option<ViewEvent> {
    let v = seat.idx();
    Some(match e {
        Event::TurnBegan { turn, active } => ViewEvent::TurnBegan { turn: *turn, active_is_me: *active == seat },
        Event::StepBegan { step } => ViewEvent::StepBegan { step: *step },
        Event::ZoneChange { def, from, to, owner, vid_old, vid_new, .. } => {
            if vid_new[v] != ViewId::NONE || vid_old[v] != ViewId::NONE {
                ViewEvent::Moved { name: name(db, *def), vid_old: vid_old[v], vid_new: vid_new[v], from: *from, to: *to, owner_is_me: *owner == seat }
            } else {
                ViewEvent::HiddenMove { owner_is_me: *owner == seat, from: *from, to: *to }
            }
        }
        Event::Drew { player, def, vid, .. } => {
            if *player == seat {
                ViewEvent::DrewCard { vid: vid[v], name: name(db, *def) }
            } else {
                ViewEvent::OppDrewCard
            }
        }
        Event::DrewFromEmpty { player } => ViewEvent::DrewFromEmptyLibrary { me: *player == seat },
        // Damage targets/sources are public objects or players; ids here are resolved by the Game
        // layer at observation time (see `Game::push_events`), which has the state.
        Event::Damage { .. } => return None,
        Event::LifeChange { player, delta } => ViewEvent::LifeChange { me: *player == seat, delta: *delta },
        Event::SpellCast { def, controller, vid, .. } => ViewEvent::SpellCast { vid: vid[v], name: name(db, *def), by_me: *controller == seat },
        Event::AbilityActivated { .. } | Event::BecameTarget { .. } | Event::Blocked { .. } => return None,
        Event::LandPlayed { .. } => return None,
        Event::Tapped { .. } => return None,
        Event::Untapped { .. } => return None,
        Event::CountersChanged { .. } => return None,
        Event::Shuffled { player } => ViewEvent::Shuffled { me: *player == seat },
        Event::Attacks { .. } => return None,
        Event::AttackersDeclared { count } => ViewEvent::AttackersDeclared { count: *count },
        Event::BlockersDeclared { count } => ViewEvent::BlockersDeclared { count: *count },
        Event::SpellCountered { .. } => return None,
        Event::SpellFizzled { .. } => return None,
        Event::ManaAdded { .. } => return None,
        Event::MulliganTaken { player } => ViewEvent::MulliganTaken { me: *player == seat },
        Event::HandKept { player, bottomed } => ViewEvent::HandKept { me: *player == seat, bottomed: *bottomed },
        Event::TokenCreated { .. } => return None,
        Event::GameEnded => ViewEvent::GameEnded,
    })
}

/// Events that need state lookups (object view ids) are redacted with access to the state at the
/// time they are drained. Their referenced objects are public (battlefield/stack), so the lookup
/// is by current `ObjRef` generation; stale references degrade to `None`/`Gone`.
pub fn redact_with_state(s: &State, db: &CardDb, e: &Event, seat: Seat) -> Option<ViewEvent> {
    let vid_of = |r: ObjRef| if s.is_live_ref(r) { s.view_id(r, seat) } else { ViewId::NONE };
    match e {
        Event::Damage { source, to, amount, combat } => Some(ViewEvent::Damage {
            source: source.map(vid_of).filter(|v| *v != ViewId::NONE),
            to: view_target(s, db, *to, seat),
            amount: *amount,
            combat: *combat,
        }),
        Event::AbilityActivated { source, controller } => Some(ViewEvent::AbilityActivated { source: Some(vid_of(*source)).filter(|v| *v != ViewId::NONE), by_me: *controller == seat }),
        Event::LandPlayed { obj, controller } => Some(ViewEvent::LandPlayed { vid: vid_of(*obj), name: name(db, s.def_of(*obj)), by_me: *controller == seat }),
        Event::Tapped { obj } => Some(ViewEvent::Tapped { vid: vid_of(*obj) }),
        Event::Untapped { obj } => Some(ViewEvent::Untapped { vid: vid_of(*obj) }),
        Event::CountersChanged { obj, kind, delta } => Some(ViewEvent::CountersChanged { vid: vid_of(*obj), kind: *kind, delta: *delta }),
        Event::SpellCountered { obj } => Some(ViewEvent::SpellCountered { vid: vid_of(*obj) }),
        Event::SpellFizzled { obj } => Some(ViewEvent::SpellFizzled { vid: vid_of(*obj) }),
        Event::TokenCreated { obj, def, controller } => Some(ViewEvent::TokenCreated { vid: vid_of(*obj), name: name(db, *def), by_me: *controller == seat }),
        other => redact(db, other, seat),
    }
}

fn label_for(s: &State, db: &CardDb, seat: Seat, o: &Opt) -> (ActionKind, Option<ViewId>, Option<u32>, String) {
    let nm = |r: ObjRef| format!("{} [v{}]", name(db, s.def_of(r)), s.view_id(r, seat).0);
    match *o {
        Opt::Pass => (ActionKind::Pass, None, None, "Pass priority".into()),
        Opt::PlayLandBack(r, pay) => (ActionKind::PlayLand, Some(s.view_id(r, seat)), Some(pay as u32), format!("Play {} as a land{}", name(db, s.def_of(r)), if pay { " (pay 3 life)" } else { " (tapped)" })),
        Opt::PlayLand(r) => (ActionKind::PlayLand, Some(s.view_id(r, seat)), None, format!("Play {}", name(db, s.def_of(r)))),
        Opt::Cast(r, way) => {
            let mut def = db.def(s.def_of(r));
            if way == mtg_core::restrict::ADVENTURE_WAY {
                def = def.back_id.map(|b| db.def(b)).unwrap_or(def);
            }
            let extra = mtg_core::cost::alt_of(def, way).map(|a| format!(" ({})", a.label)).unwrap_or_default();
            (ActionKind::CastSpell, Some(s.view_id(r, seat)), None, format!("Cast {}{extra}", def.name))
        }
        Opt::Mode(i) => (ActionKind::Mode, None, Some(i as u32), format!("Mode {}", i + 1)),
        Opt::Choice(i) => (ActionKind::Choice, None, Some(i as u32), format!("Option {}", i + 1)),
        Opt::Activate { src, ability } => {
            // The Oracle sentence tells apart a Class level-up from a fetch land's search ("ability 2" alone says nothing).
            let txt = match db.def(s.def_of(src)).abilities.get(ability as usize) {
                Some(mtg_core::ir::AbilityDef::Activated(ad)) if !ad.text.is_empty() => {
                    let t: String = ad.text.chars().take(80).collect();
                    format!(" — {t}")
                }
                _ => String::new(),
            };
            (ActionKind::ActivateAbility, Some(s.view_id(src, seat)), Some(ability as u32), format!("Activate {} ability {}{}", nm(src), ability, txt))
        }
        Opt::Mana { src, ability, color, pick } => {
            let c = ['W', 'U', 'B', 'R', 'G', 'C'][color as usize];
            (ActionKind::ActivateMana, Some(s.view_id(src, seat)), Some(ability as u32 * 8 + color as u32), format!("Tap {} for {c}{}", nm(src), pick.map(|p| format!(", sacrificing {}", nm(p))).unwrap_or_default()))
        }
        Opt::Target(Target::None) => (ActionKind::Done, None, None, "No target".into()),
        Opt::Target(Target::Player(p)) => (ActionKind::ChooseTarget, None, None, if p == seat { "Target: you".into() } else { "Target: opponent".into() }),
        Opt::Target(Target::Obj(r)) => (ActionKind::ChooseTarget, Some(s.view_id(r, seat)), None, format!("Target: {}", nm(r))),
        Opt::Card(r) => (ActionKind::ChooseCard, Some(s.view_id(r, seat)), None, format!("Choose {}", name(db, s.def_of(r)))),
        Opt::Yes => (ActionKind::Yes, None, None, "Yes".into()),
        Opt::No => (ActionKind::No, None, None, "No".into()),
        Opt::Keep => (ActionKind::Keep, None, None, "Keep hand".into()),
        Opt::Mulligan => (ActionKind::Mulligan, None, None, "Mulligan".into()),
        Opt::Attack(AttackTarget::Walker(w)) => (ActionKind::Attack, Some(s.view_id(w, seat)), None, format!("Attack planeswalker {}", nm(w))),
        Opt::Attack(AttackTarget::Player(p)) => (ActionKind::Attack, None, None, if p == seat { "Attack you".into() } else { "Attack opponent".into() }),
        Opt::NoAttack => (ActionKind::NoAttack, None, None, "Do not attack".into()),
        Opt::Block(a) => (ActionKind::Block, Some(s.view_id(a, seat)), None, format!("Block {}", nm(a))),
        Opt::NoBlock => (ActionKind::NoBlock, None, None, "Do not block".into()),
        Opt::Number(n) => (ActionKind::Number, None, Some(n), format!("Assign {n}")),
        Opt::Done => (ActionKind::Done, None, None, "Done".into()),
        Opt::Name(d) => (ActionKind::Name, None, Some(d as u32), name(db, mtg_core::ids::CardDefId(d)).to_string()),
        Opt::Type(t) => (ActionKind::Name, None, Some(t as u32), mtg_core::types::SUBTYPE_NAMES[t as usize].to_string()),
    }
}

fn decision_view(s: &State, db: &CardDb, p: &Pending, seat: Seat) -> Decision {
    let vid = |r: ObjRef| s.view_id(r, seat);
    let (kind, context) = match p.kind {
        DecisionKind::Priority => (ViewDecisionKind::Priority, "You have priority.".to_string()),
        DecisionKind::Mulligan => (ViewDecisionKind::Mulligan, "Keep this hand or take a mulligan?".to_string()),
        DecisionKind::ChooseTarget { slot } => (ViewDecisionKind::ChooseTarget { slot }, format!("Choose target {}.", slot + 1)),
        DecisionKind::DeclareAttacker { creature } => (ViewDecisionKind::DeclareAttacker { creature: vid(creature) }, format!("Attack with {}?", name(db, s.def_of(creature)))),
        DecisionKind::DeclareBlocker { creature } => (ViewDecisionKind::DeclareBlocker { creature: vid(creature) }, format!("Block with {}?", name(db, s.def_of(creature)))),
        DecisionKind::ChooseCards { purpose: purpose @ mtg_core::decision::CardsPurpose::Delve, remaining } => (
            ViewDecisionKind::ChooseCards { purpose, remaining },
            "Delve: exile graveyard cards one at a time, each one pays for one generic mana of the spell. Done is offered once the rest of the cost can be paid.".to_string(),
        ),
        DecisionKind::ChooseCards { purpose, remaining } => (ViewDecisionKind::ChooseCards { purpose, remaining }, format!("Choose a card ({remaining} left): {purpose:?}.")),
        DecisionKind::May => (ViewDecisionKind::May, "Do it?".to_string()),
        DecisionKind::ChooseMode { chosen, .. } => (ViewDecisionKind::ChooseMode { chosen }, format!("Choose a mode ({chosen} chosen).")),
        DecisionKind::OrderTriggers => (ViewDecisionKind::OrderTriggers, "Choose which triggered ability to put on the stack next (the last one put on resolves first).".to_string()),
        DecisionKind::ChooseColor => (ViewDecisionKind::ChooseColor, "Choose a color.".to_string()),
        DecisionKind::ChooseDungeon => (ViewDecisionKind::ChooseDungeon, "Choose a dungeon to venture into.".to_string()),
        DecisionKind::ChooseRoom => (ViewDecisionKind::ChooseRoom, "Choose the next room.".to_string()),
        DecisionKind::ChooseName { lands } => (ViewDecisionKind::ChooseName, if lands { "Choose a card name.".to_string() } else { "Choose a nonland card name.".to_string() }),
        DecisionKind::ChooseType => (ViewDecisionKind::ChooseName, "Choose a creature type.".to_string()),
        DecisionKind::LegendRule => (ViewDecisionKind::LegendRule, "Choose the legendary permanent to keep.".to_string()),
        DecisionKind::PayUnless => (ViewDecisionKind::May, "Pay to avoid the effect?".to_string()),
        DecisionKind::ChangeTargets => (ViewDecisionKind::May, "Choose new targets for the copy?".to_string()),
        DecisionKind::ChooseReplicate => (ViewDecisionKind::ChooseX, "How many times do you pay the replicate cost?".to_string()),
        DecisionKind::PayEnergyAmount => (ViewDecisionKind::ChooseX, "How much energy do you pay?".to_string()),
        DecisionKind::ChooseX => (ViewDecisionKind::ChooseX, "Choose the value of X.".to_string()),
        DecisionKind::ChooseAddCost { .. } => (ViewDecisionKind::ChooseAddCost, "Choose an additional cost.".to_string()),
        DecisionKind::PayPhyrexian { color } => (ViewDecisionKind::PayPhyrexian, format!("Pay the Phyrexian {} symbol with life (yes) or mana (no)?", ['W', 'U', 'B', 'R', 'G', 'C'][color as usize])),
        DecisionKind::AssignDamage { attacker, blocker, remaining } => (
            ViewDecisionKind::AssignDamage { attacker: vid(attacker), blocker: vid(blocker), remaining },
            format!("Assign combat damage from {} to {} ({remaining} left).", name(db, s.def_of(attacker)), name(db, s.def_of(blocker))),
        ),
    };
    let options: Vec<ActionDesc> = p
        .options
        .iter()
        .enumerate()
        .map(|(i, o)| {
            let (k, subj, value, mut label) = label_for(s, db, seat, o);
            match (p.kind, o) {
                (DecisionKind::ChooseMode { spell, .. }, Opt::Mode(i)) => {
                    if let Some((_, sd)) = db.def(s.def_of(spell)).spell_def() {
                        let l = &sd.modes[*i as usize].label;
                        label = if l.is_empty() { format!("Mode {}", i + 1) } else { l.clone() };
                    }
                }
                (DecisionKind::ChooseAddCost { spell }, Opt::Choice(i)) => {
                    if let Some(ac) = mtg_core::cost::add_cost_of(db.def(s.def_of(spell))) {
                        label = mtg_core::cost::describe_cost(&ac.options[*i as usize]);
                    }
                }
                (DecisionKind::ChooseAddCost { .. }, Opt::No) => label = "No additional cost".into(),
                (DecisionKind::PayPhyrexian { .. }, Opt::Yes) => label = "Pay 2 life".into(),
                (DecisionKind::PayPhyrexian { .. }, Opt::No) => label = "Pay mana".into(),
                (DecisionKind::ChooseMode { .. }, Opt::Done) => label = "No more modes".into(),
                (DecisionKind::OrderTriggers, Opt::Choice(i)) => {
                    if let Some(t) = s.pending_trigger(*i as usize) {
                        let v = s.view_id(t.source, seat);
                        let who = if v == ViewId::NONE { name(db, t.def).to_string() } else { format!("{} [v{}]", name(db, t.def), v.0) };
                        label = format!("Put the trigger of {who} (ability {}) on the stack next", t.ability);
                    }
                }
                (DecisionKind::ChooseX, Opt::Number(n)) => label = format!("X = {n}"),
                (DecisionKind::PayEnergyAmount, Opt::Number(n)) => label = format!("Pay {n} energy"),
                (DecisionKind::ChooseReplicate, Opt::Number(n)) => label = format!("Replicate {n} time(s)"),
                (DecisionKind::ChooseDungeon, Opt::Choice(i)) => label = name(db, db.dungeons()[*i as usize]).to_string(),
                (DecisionKind::ChooseRoom, Opt::Choice(i)) => {
                    if let Some((d, room)) = s.dungeon_of(seat) {
                        if let Some(dd) = &db.def(d).dungeon {
                            label = dd.rooms[dd.rooms[room as usize].exits[*i as usize] as usize].name.clone();
                        }
                    }
                }
                (DecisionKind::ChooseColor, Opt::Choice(i)) => label = ["White", "Blue", "Black", "Red", "Green"][*i as usize].to_string(),
                _ => {}
            }
            let subject_def = match o {
                Opt::Card(r) | Opt::Cast(r, _) | Opt::PlayLand(r) | Opt::PlayLandBack(r, _) => Some(s.def_of(*r)),
                Opt::Target(Target::Obj(r)) | Opt::Block(r) | Opt::Activate { src: r, .. } | Opt::Mana { src: r, .. } | Opt::Attack(AttackTarget::Walker(r)) => {
                    if s.view_id(*r, seat) != ViewId::NONE { Some(s.def_of(*r)) } else { None }
                }
                _ => None,
            };
            ActionDesc { idx: i as u16, kind: k, subject: subj.filter(|v| *v != ViewId::NONE), subject_def, value, label }
        })
        .collect();
    Decision { id: p.id, seat: p.seat, trivial: options.len() == 1, low_frequency: p.kind.low_frequency(), kind, options, context }
}

fn exile_of(s: &State, db: &CardDb, owner: Seat, seat: Seat) -> Vec<ViewCard> {
    let mut v: Vec<ViewCard> = s.exile().iter().filter(|&&r| s.owner_of(r) == owner && s.view_id(r, seat) != ViewId::NONE).map(|&r| card(s, db, r, seat)).collect();
    v.sort_by_key(|c| (c.def, c.vid));
    v
}

fn designations(s: &State, db: &CardDb, p: Seat, seat: Seat) -> ViewDesignations {
    ViewDesignations {
        dungeon: s.dungeon_of(p).map(|(d, room)| (name(db, d), room)),
        completed_dungeons: s.completed_dungeons(p).iter().map(|&d| name(db, d)).collect(),
        emblems: s.emblems_of(p).iter().map(|&d| name(db, d)).collect(),
        ring_level: s.ring_level(p),
        ring_bearer: s.ring_bearer(p).filter(|&r| s.is_live_ref(r)).map(|r| s.view_id(r, seat)),
        blessing: s.has_blessing(p),
    }
}

pub fn observe(g: &Game, seat: Seat) -> Observation {
    let s = g.core_state();
    let db = g.db();
    let opp = seat.other();
    let td = s.turn_data();
    let mut hand: Vec<ViewCard> = s.hand(seat).iter().map(|&r| card(s, db, r, seat)).collect();
    hand.sort_by_key(|c| (c.def, c.vid));
    let gy = |p: Seat| s.graveyard(p).iter().map(|&r| card(s, db, r, seat)).collect::<Vec<_>>();
    let combat = s.combat();
    let battlefield: Vec<ViewPermanent> = s
        .battlefield()
        .iter()
        .map(|&r| {
            let d = s.obj_data(db, r);
            let attacking = combat.map(|c| c.attackers.iter().any(|a| a.obj == r)).unwrap_or(false);
            let blocking = combat.and_then(|c| c.attackers.iter().find(|a| a.blockers.contains(&r)).map(|a| s.view_id(a.obj, seat)).filter(|v| *v != ViewId::NONE));
            ViewPermanent {
                vid: d.vid[seat.idx()],
                def: d.def,
                name: name(db, d.def),
                controlled_by_me: d.controller == seat,
                owned_by_me: d.owner == seat,
                tapped: d.tapped,
                summoning_sick: d.summoning_sick && !d.chars.keywords.contains(mtg_core::types::Keywords::HASTE),
                damage: d.damage,
                counters: d.counters.clone(),
                types: d.chars.types,
                power: d.chars.power,
                toughness: d.chars.toughness,
                keywords: d.chars.keywords,
                attacking,
                attack_target: if attacking { s.attack_target_of(r).map(|t| match t {
                    mtg_core::decision::AttackTarget::Player(p) => ViewAttackTarget::Player { me: p == seat },
                    mtg_core::decision::AttackTarget::Walker(w) => ViewAttackTarget::Planeswalker(s.view_id(w, seat)),
                }) } else { None },
                blocking,
            }
        })
        .collect();
    let stack: Vec<ViewStackItem> = s
        .stack()
        .iter()
        .map(|e| ViewStackItem {
            vid: s.view_id(e.obj, seat),
            def: e.def,
            name: name(db, e.def),
            controlled_by_me: e.controller == seat,
            is_ability: matches!(e.kind, StackKind::Ability { .. }),
            targets: e.targets.iter().map(|t| view_target(s, db, *t, seat)).collect(),
        })
        .collect();
    let decision = s.pending().filter(|p| p.seat == seat).map(|p| decision_view(s, db, p, seat));
    let mut ob = Observation {
        seat,
        turn: td.turn,
        step: td.step,
        active_is_me: td.active == seat,
        priority_is_me: td.priority == seat,
        me: SelfView {
            life: s.life(seat),
            energy: s.energy(seat),
            pool: s.pool(seat),
            hand,
            graveyard: gy(seat),
            library_count: s.library_count(seat),
            lands_played: s.lands_played(seat),
            library_known_top: s.known_library(seat, seat).0.iter().map(|&r| card(s, db, r, seat)).collect(),
            library_known_bottom: s.known_library(seat, seat).1.iter().map(|&r| card(s, db, r, seat)).collect(),
            exile: exile_of(s, db, seat, seat),
            designations: designations(s, db, seat, seat),
        },
        opp: OppView {
            life: s.life(opp),
            energy: s.energy(opp),
            hand_count: s.hand_count(opp),
            graveyard: gy(opp),
            library_count: s.library_count(opp),
            lands_played: s.lands_played(opp),
            revealed_hand: {
                let mut v: Vec<ViewCard> = s.known_hand(opp, seat).iter().map(|&r| card(s, db, r, seat)).collect();
                v.sort_by_key(|c| (c.def, c.vid));
                v
            },
            library_known_top: s.known_library(opp, seat).0.iter().map(|&r| card(s, db, r, seat)).collect(),
            library_known_bottom: s.known_library(opp, seat).1.iter().map(|&r| card(s, db, r, seat)).collect(),
            exile: exile_of(s, db, opp, seat),
            designations: designations(s, db, opp, seat),
        },
        battlefield,
        stack,
        events: g.view_events_for(seat),
        decision,
        view_hash: 0,
    };
    let mut h = Fx64::default();
    ob.hash(&mut h);
    if mtg_core::canary::on(mtg_core::canary::HASH_OVER_STATE) {
        s.hash_full().hash(&mut h);
    }
    ob.view_hash = h.finish();
    let _ = ObjKind::Card;
    ob
}
