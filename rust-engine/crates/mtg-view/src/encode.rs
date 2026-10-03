//! Fixed-shape numeric encoding of an `Observation` for a policy/value network (Phase 3 handoff).
//!
//! A state is a dense float vector (`state_len(n_defs)` entries, layout below); the options of the
//! pending decision are rows of small integers (`OptionFeat`) meant for an embedding lookup: the
//! network scores each option from the state vector and the option's embedding, then a softmax
//! over the rows is the policy. Everything comes from the `Observation`, so it carries no hidden
//! information. `n_defs` is the size of the card database (`CardDb::defs.len()`), which is the
//! closed pool plus tokens and faces.
//!
//! State layout (blocks of `n_defs` card counts, then scalars):
//!   0 my hand, 1 my graveyard, 2 my battlefield, 3 my tapped battlefield, 4 opp battlefield,
//!   5 opp tapped battlefield, 6 opp graveyard, 7 stack, 8 my known library top, 9 my exile,
//!   10 opponent's exile.
//! Scalars (`N_SCALARS`): see `SCALAR_NAMES`.

use crate::view::*;
use mtg_core::ids::ViewId;
use mtg_core::types::{Step, Types};

pub const N_BLOCKS: usize = 11;

pub const SCALAR_NAMES: [&str; 46] = [
    "my_life/20", "opp_life/20", "turn/20", "active_is_me", "priority_is_me", "my_library/60", "opp_library/60", "opp_hand/7", "my_hand/7",
    "pool_w", "pool_u", "pool_b", "pool_r", "pool_g", "pool_c", "my_energy/10", "opp_energy/10", "my_lands_played", "opp_lands_played",
    "my_creatures/10", "opp_creatures/10", "my_power/20", "opp_power/20", "my_toughness/20", "opp_toughness/20", "my_attackers/10", "opp_attackers/10",
    "stack_size/5",
    "step_untap_upkeep", "step_draw", "step_main1", "step_begin_combat", "step_declare_attackers", "step_declare_blockers", "step_damage",
    "step_end_combat", "step_main2", "step_end_cleanup", "decision_is_mine", "reserved",
    "my_ring_level/4", "opp_ring_level/4", "my_blessing", "opp_blessing", "my_dungeon_room+1/8", "opp_dungeon_room+1/8",
];
pub const N_SCALARS: usize = SCALAR_NAMES.len();

pub fn state_len(n_defs: usize) -> usize {
    N_BLOCKS * n_defs + N_SCALARS
}

/// Encodes the observation into `out` (cleared and resized to `state_len(n_defs)`).
pub fn encode_state(o: &Observation, n_defs: usize, out: &mut Vec<f32>) {
    out.clear();
    out.resize(state_len(n_defs), 0.0);
    let mut add = |block: usize, def: usize| {
        if def < n_defs {
            out[block * n_defs + def] += 1.0;
        }
    };
    for c in &o.me.hand {
        add(0, c.def.0 as usize);
    }
    for c in &o.me.graveyard {
        add(1, c.def.0 as usize);
    }
    for c in &o.me.library_known_top {
        add(8, c.def.0 as usize);
    }
    for c in &o.opp.graveyard {
        add(6, c.def.0 as usize);
    }
    for s in &o.stack {
        add(7, s.def.0 as usize);
    }
    for c in &o.me.exile {
        add(9, c.def.0 as usize);
    }
    for c in &o.opp.exile {
        add(10, c.def.0 as usize);
    }
    let (mut mc, mut oc, mut mp, mut op, mut mt, mut ot, mut ma, mut oa) = (0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32);
    for p in &o.battlefield {
        let d = p.def.0 as usize;
        if p.controlled_by_me {
            add(2, d);
            if p.tapped {
                add(3, d);
            }
        } else {
            add(4, d);
            if p.tapped {
                add(5, d);
            }
        }
        if p.types.contains(Types::CREATURE) {
            let (c, pw, tg, at) = if p.controlled_by_me { (&mut mc, &mut mp, &mut mt, &mut ma) } else { (&mut oc, &mut op, &mut ot, &mut oa) };
            *c += 1.0;
            *pw += p.power.max(0) as f32;
            *tg += p.toughness.max(0) as f32;
            if p.attacking {
                *at += 1.0;
            }
        }
    }
    let base = N_BLOCKS * n_defs;
    let step = |s: Step| -> usize {
        match s {
            Step::Untap | Step::Upkeep => 28,
            Step::Draw => 29,
            Step::Main1 => 30,
            Step::BeginCombat => 31,
            Step::DeclareAttackers => 32,
            Step::DeclareBlockers => 33,
            Step::FirstStrikeDamage | Step::CombatDamage => 34,
            Step::EndCombat => 35,
            Step::Main2 => 36,
            Step::End | Step::Cleanup => 37,
        }
    };
    let sc: [f32; 28] = [
        o.me.life as f32 / 20.0, o.opp.life as f32 / 20.0, o.turn as f32 / 20.0, o.active_is_me as u8 as f32, o.priority_is_me as u8 as f32,
        o.me.library_count as f32 / 60.0, o.opp.library_count as f32 / 60.0, o.opp.hand_count as f32 / 7.0, o.me.hand.len() as f32 / 7.0,
        o.me.pool[0] as f32, o.me.pool[1] as f32, o.me.pool[2] as f32, o.me.pool[3] as f32, o.me.pool[4] as f32, o.me.pool[5] as f32,
        o.me.energy as f32 / 10.0, o.opp.energy as f32 / 10.0, o.me.lands_played as f32, o.opp.lands_played as f32,
        mc / 10.0, oc / 10.0, mp / 20.0, op / 20.0, mt / 20.0, ot / 20.0, ma / 10.0, oa / 10.0, o.stack.len() as f32 / 5.0,
    ];
    out[base..base + 28].copy_from_slice(&sc);
    let ex: [f32; 6] = [
        o.me.designations.ring_level as f32 / 4.0, o.opp.designations.ring_level as f32 / 4.0,
        o.me.designations.blessing as u8 as f32, o.opp.designations.blessing as u8 as f32,
        o.me.designations.dungeon.as_ref().map_or(0.0, |d| (d.1 as f32 + 1.0) / 8.0), o.opp.designations.dungeon.as_ref().map_or(0.0, |d| (d.1 as f32 + 1.0) / 8.0),
    ];
    out[base + 40..base + 46].copy_from_slice(&ex);
    out[base + step(o.step)] = 1.0;
    if o.decision.as_ref().map_or(false, |d| d.seat == o.seat) {
        out[base + 38] = 1.0;
    }
}

/// One option of the pending decision as small integers for embedding lookups.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OptionFeat {
    /// `ActionKind` as an integer.
    pub kind: u8,
    /// `ViewDecisionKind` as an integer (same for every option of a decision).
    pub decision: u8,
    /// Card definition of the subject plus one; 0 when the option has no card subject.
    pub subject_def: u16,
    /// Where the subject is: 0 none/unknown, 1 my hand, 2 my graveyard, 3 my battlefield, 4 opp
    /// battlefield, 5 stack, 6 opp graveyard, 7 other (library cards), 8 exile.
    pub subject_zone: u8,
    /// Number payload (X, damage), else 0.
    pub value: u32,
}

fn decision_code(k: &ViewDecisionKind) -> u8 {
    use ViewDecisionKind::*;
    match k {
        Priority => 0,
        Mulligan => 1,
        ChooseTarget { .. } => 2,
        DeclareAttacker { .. } => 3,
        DeclareBlocker { .. } => 4,
        ChooseCards { .. } => 5,
        AssignDamage { .. } => 6,
        May => 7,
        OrderTriggers => 8,
        ChooseMode { .. } => 9,
        ChooseX => 10,
        ChooseAddCost => 11,
        PayPhyrexian => 12,
        ChooseColor => 13,
        ChooseDungeon => 14,
        ChooseRoom => 15,
        ChooseName => 16,
        LegendRule => 17,
    }
}

fn find(o: &Observation, v: ViewId) -> (u16, u8) {
    for p in &o.battlefield {
        if p.vid == v {
            return (p.def.0 + 1, if p.controlled_by_me { 3 } else { 4 });
        }
    }
    for s in &o.stack {
        if s.vid == v {
            return (s.def.0 + 1, 5);
        }
    }
    for c in &o.me.hand {
        if c.vid == v {
            return (c.def.0 + 1, 1);
        }
    }
    for c in &o.me.graveyard {
        if c.vid == v {
            return (c.def.0 + 1, 2);
        }
    }
    for c in &o.opp.graveyard {
        if c.vid == v {
            return (c.def.0 + 1, 6);
        }
    }
    for c in o.me.exile.iter().chain(&o.opp.exile) {
        if c.vid == v {
            return (c.def.0 + 1, 8);
        }
    }
    for c in o.me.library_known_top.iter().chain(&o.me.library_known_bottom).chain(&o.opp.library_known_top).chain(&o.opp.library_known_bottom).chain(&o.opp.revealed_hand) {
        if c.vid == v {
            return (c.def.0 + 1, 7);
        }
    }
    (0, 0)
}

/// Encodes the options of the pending decision (empty when `o.decision` is `None`).
pub fn encode_options(o: &Observation) -> Vec<OptionFeat> {
    let Some(d) = &o.decision else { return Vec::new() };
    let dc = decision_code(&d.kind);
    d.options
        .iter()
        .map(|a| {
            let (found_def, found_zone) = a.subject.map_or((0, 0), |v| find(o, v));
            // A card offered from a zone the observation does not list (library, exile) still has
            // its definition on the option.
            let subject_def = if found_def != 0 { found_def } else { a.subject_def.map_or(0, |d| d.0 + 1) };
            let subject_zone = if found_zone != 0 { found_zone } else if a.subject_def.is_some() { 7 } else { 0 };
            OptionFeat { kind: a.kind as u8, decision: dc, subject_def, subject_zone, value: a.value.unwrap_or(0) }
        })
        .collect()
}
