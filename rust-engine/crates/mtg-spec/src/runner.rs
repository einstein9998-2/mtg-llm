//! Runs one scenario document against the engine.

use crate::{Err, R, J};
use crate::{fail, unsupported};
use mtg_core::card::*;
use mtg_core::decision::*;
use mtg_core::ids::*;
use mtg_core::scenario::*;
use mtg_core::state::*;
use mtg_core::types::*;
use mtg_view::Game;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Outcome {
    Pass,
    Fail(String),
    Unsupported(String),
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Alias {
    Slot(u16),
    Obj(ObjRef),
}

pub(crate) struct Runner {
    pub db: Arc<CardDb>,
    pub g: Game,
    pub aliases: HashMap<String, Alias>,
    /// Doc 04 handles (`p0:Island#1`) from the setup.
    pub handles: HashMap<String, Alias>,
    pub step_no: usize,
    pub last_events: Vec<String>,
    /// Probe mode (`legal` patterns): decisions the action does not answer take the first option.
    pub lenient: bool,
}

pub fn seat_of(s: &str) -> R<Seat> {
    match s {
        "p0" => Ok(Seat(0)),
        "p1" => Ok(Seat(1)),
        _ => Err(Err::Fail(format!("bad seat {s:?}"))),
    }
}

pub fn run_scenario(db: &Arc<CardDb>, doc: &J) -> Outcome {
    let mut variants: Vec<(bool, J)> = vec![(false, doc.clone())];
    if doc.get("symmetric").and_then(|v| v.as_bool()).unwrap_or(false) {
        variants.push((true, swap_seats(doc)));
    }
    for (swapped, d) in variants {
        match run_one(db, &d) {
            Ok(()) => {}
            Result::Err(Err::Fail(m)) => return Outcome::Fail(if swapped { format!("[seats swapped] {m}") } else { m }),
            Result::Err(Err::Unsupported(m)) => return Outcome::Unsupported(m),
        }
    }
    Outcome::Pass
}

/// Swaps p0 and p1 throughout (setup, script, expectations).
fn swap_seats(v: &J) -> J {
    match v {
        J::String(s) => {
            let t = s.replace("p0", "\u{1}").replace("p1", "p0").replace('\u{1}', "p1");
            J::String(t)
        }
        J::Array(a) => J::Array(a.iter().map(swap_seats).collect()),
        J::Object(m) => J::Object(m.iter().map(|(k, x)| (swap_seats(&J::String(k.clone())).as_str().unwrap().to_string(), swap_seats(x))).collect()),
        o => o.clone(),
    }
}

fn run_one(db: &Arc<CardDb>, doc: &J) -> R<()> {
    if let Some(wb) = doc.get("world_b") {
        // Paired worlds: the observer's whole stream must not depend on what it may not know.
        let observer = match doc.get("assert_streams_identical").and_then(|a| a.get("observer")).and_then(|o| o.as_str()) {
            Some(o) => seat_of(o)?,
            None => unsupported!("world_b without assert_streams_identical.observer"),
        };
        let mut b = doc.clone();
        b["setup"] = deep_merge(&doc["setup"], wb);
        let (mut sa, mut sb) = (Vec::new(), Vec::new());
        run_one_rec(db, doc, Some((observer, &mut sa)), true)?;
        run_one_rec(db, &b, Some((observer, &mut sb)), false)?;
        for (i, (x, y)) in sa.iter().zip(sb.iter()).enumerate() {
            if x != y {
                let (lx, ly): (Vec<&str>, Vec<&str>) = (x.lines().collect(), y.lines().collect());
                let k = lx.iter().zip(ly.iter()).position(|(a, b)| a != b).unwrap_or(0);
                fail!("the observer's stream differs between the worlds at point {i} (line {k}): {:?} vs {:?}", lx.get(k), ly.get(k));
            }
        }
        if sa.len() != sb.len() {
            fail!("the worlds' streams have different lengths ({} vs {})", sa.len(), sb.len());
        }
        return Ok(());
    }
    run_one_rec(db, doc, None, true)
}

/// Maps merge key by key, everything else is replaced (SCHEMA: `world_b`).
fn deep_merge(a: &J, b: &J) -> J {
    match (a, b) {
        (J::Object(x), J::Object(y)) => {
            let mut m = x.clone();
            for (k, v) in y {
                let merged = match x.get(k) {
                    Some(old) => deep_merge(old, v),
                    None => v.clone(),
                };
                m.insert(k.clone(), merged);
            }
            J::Object(m)
        }
        (_, b) => b.clone(),
    }
}

fn run_one_rec(db: &Arc<CardDb>, doc: &J, mut rec: Option<(Seat, &mut Vec<String>)>, check_expects: bool) -> R<()> {
    let mut r = Runner::setup(db, doc)?;
    let snap = |r: &Runner, rec: &mut Option<(Seat, &mut Vec<String>)>| {
        if let Some((seat, out)) = rec.as_mut() {
            out.push(format!("{:#?}", r.g.observe(*seat)));
        }
    };
    snap(&r, &mut rec);
    let mut scripted = false;
    if let Some(a) = doc.get("random").and_then(|v| v.as_array()) {
        let mut queue: Vec<(Seat, RandKind, Vec<CardDefId>)> = Vec::new();
        for e in a {
            let kind = match e.get("kind").and_then(|k| k.as_str()) {
                Some("shuffle") => RandKind::Shuffle,
                Some("bottom_order") => RandKind::BottomOrder,
                _ => unsupported!("scripted random event {e}"),
            };
            let seat = seat_of(e.get("player").and_then(|x| x.as_str()).unwrap_or("p0"))?;
            let mut order = Vec::new();
            for n in e.get("result").and_then(|x| x.as_array()).cloned().unwrap_or_default() {
                match r.db.id(n.as_str().unwrap_or("")) {
                    Some(d) => order.push(d),
                    None => unsupported!("scripted shuffle names unknown card {n}"),
                }
            }
            queue.push((seat, kind, order));
        }
        r.g.raw_state_mut().scenario_script_random(queue);
        scripted = true;
    }
    let script = doc["script"].as_array().cloned().unwrap_or_default();
    let expects = doc["expect"].as_array().cloned().unwrap_or_default();
    for (i, step) in script.iter().enumerate() {
        r.step_no = i;
        if !step_keeps_orders(step) {
            r.flush_orders()?;
        }
        r.run_step(step).map_err(|e| ctx(e, &format!("step {i} {}", brief(step))))?;
        snap(&r, &mut rec);
        if let Some(l) = step.get("label").and_then(|v| v.as_str()) {
            for e in expects.iter().filter(|_| check_expects) {
                if e.get("at").and_then(|v| v.as_str()) == Some(l) {
                    r.check_block(e).map_err(|e| ctx(e, &format!("expect at {l}")))?;
                }
            }
        }
    }
    r.flush_orders()?;
    for e in expects.iter().filter(|_| check_expects) {
        let at = e.get("at").and_then(|v| v.as_str()).unwrap_or("after_script");
        if at == "after_script" {
            r.check_block(e).map_err(|e| ctx(e, "expect after_script"))?;
        }
    }
    if scripted {
        let (viol, left) = r.st().scenario_random_report();
        if viol > 0 {
            fail!("the engine shuffled {viol} time(s) that the script's random queue does not cover");
        }
        if left > 0 {
            fail!("{left} scripted shuffle(s) never happened");
        }
    }
    Ok(())
}

fn brief(step: &J) -> String {
    let s = step.to_string();
    if s.len() > 160 {
        format!("{}...", &s[..160])
    } else {
        s
    }
}

fn ctx(e: Err, c: &str) -> Err {
    match e {
        Err::Fail(m) => Err::Fail(format!("{c}: {m}")),
        Err::Unsupported(m) => Err::Unsupported(m),
    }
}

// ---------------------------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------------------------

#[derive(Default, Clone, Debug)]
struct Entry {
    name: String,
    alias: Option<String>,
    tapped: bool,
    sick: bool,
    counters: Vec<(CounterKind, u16)>,
    damage: u16,
    attached_to: Option<String>,
    token: bool,
    controller: Option<Seat>,
    chosen_name: Option<String>,
    chosen_type: Option<String>,
}

pub(crate) fn counter_kind(name: &str) -> R<CounterKind> {
    Ok(match name {
        "+1/+1" => CounterKind::PlusOne,
        "-1/-1" => CounterKind::MinusOne,
        "loyalty" => CounterKind::Loyalty,
        "energy" => CounterKind::Energy,
        "time" => CounterKind::Time,
        "stun" => CounterKind::Stun,
        "charge" => CounterKind::Charge,
        "flying" => CounterKind::Flying,
        other => unsupported!("counter kind {other:?}"),
    })
}

fn parse_entry(v: &J) -> R<Entry> {
    let mut e = Entry::default();
    match v {
        J::String(s) => {
            if let Some((n, a)) = s.split_once(" @") {
                e.name = n.trim().to_string();
                e.alias = Some(a.trim().to_string());
            } else {
                e.name = s.trim().to_string();
            }
        }
        J::Object(m) => {
            if let Some(c) = m.get("card").and_then(|x| x.as_str()) {
                e.name = c.to_string();
            }
            if let Some(c) = m.get("token").and_then(|x| x.as_str()) {
                e.name = c.to_string();
                e.token = true;
            }
            e.alias = m.get("as").and_then(|x| x.as_str()).map(|s| s.to_string());
            e.tapped = m.get("tapped").and_then(|x| x.as_bool()).unwrap_or(false);
            e.sick = m.get("sick").and_then(|x| x.as_bool()).unwrap_or(false);
            e.damage = m.get("damage").and_then(|x| x.as_u64()).unwrap_or(0) as u16;
            e.attached_to = m.get("attached_to").and_then(|x| x.as_str()).map(|s| s.trim_start_matches('@').to_string());
            if let Some(c) = m.get("controller").and_then(|x| x.as_str()) {
                e.controller = Some(seat_of(c)?);
            }
            if let Some(c) = m.get("counters").and_then(|x| x.as_object()) {
                for (k, n) in c {
                    e.counters.push((counter_kind(k)?, n.as_u64().unwrap_or(0) as u16));
                }
            }
            if let Some(f) = m.get("flags").and_then(|x| x.as_object()) {
                for (k, v) in f {
                    match k.as_str() {
                        "chosen_name" => e.chosen_name = v.as_str().map(|s| s.to_string()),
                        "chosen_type" => e.chosen_type = v.as_str().map(|s| s.to_string()),
                        // Impending is tracked by the time counters alone (the "isn't a creature" static reads them).
                        "impending_paid" => {}
                        other => unsupported!("setup flag {other}"),
                    }
                }
            }
        }
        _ => fail!("bad setup entry {v}"),
    }
    Ok(e)
}

fn parse_list(v: Option<&J>) -> R<Vec<Entry>> {
    match v.and_then(|x| x.as_array()) {
        None => Ok(vec![]),
        Some(a) => a.iter().map(parse_entry).collect(),
    }
}

fn phase_of(s: &str) -> R<Step> {
    Ok(match s {
        "untap" => Step::Untap,
        "upkeep" => Step::Upkeep,
        "draw" => Step::Draw,
        "main1" => Step::Main1,
        "begin_combat" => Step::BeginCombat,
        "declare_attackers" => Step::DeclareAttackers,
        "declare_blockers" => Step::DeclareBlockers,
        "first_strike_damage" => Step::FirstStrikeDamage,
        "combat_damage" => Step::CombatDamage,
        "end_combat" => Step::EndCombat,
        "main2" => Step::Main2,
        "end" => Step::End,
        "cleanup" => Step::Cleanup,
        other => fail!("bad phase {other:?}"),
    })
}

impl Runner {
    pub(crate) fn setup(db: &Arc<CardDb>, doc: &J) -> R<Runner> {
        let s = &doc["setup"];
        let stack_setup: Vec<J> = s.get("stack").and_then(|x| x.as_array()).cloned().unwrap_or_default();
        let mut players: [PlayerSetup; 2] = Default::default();
        let mut ents: [[Vec<Entry>; 5]; 2] = Default::default(); // hand, battlefield, graveyard, exile, library
        let mut token_ents: Vec<(Seat, Entry)> = Vec::new();
        let mut pad_names = [0usize; 2];
        // (seat, dungeon, Some(room) = in the command zone, None = completed)
        let mut dungeons: Vec<(usize, String, Option<String>)> = Vec::new();
        for (si, key) in ["p0", "p1"].iter().enumerate() {
            let p = &s[*key];
            for c in p.get("command").and_then(|c| c.as_array()).cloned().unwrap_or_default() {
                match (c.get("dungeon").and_then(|x| x.as_str()), c.get("room").and_then(|x| x.as_str())) {
                    (Some(d), Some(r)) => dungeons.push((si, d.to_string(), Some(r.to_string()))),
                    _ => unsupported!("setup.command entry {c}"),
                }
            }
            for d in p.get("completed_dungeons").and_then(|c| c.as_array()).cloned().unwrap_or_default() {
                dungeons.push((si, d.as_str().unwrap_or("").to_string(), None));
            }
            let ps = &mut players[si];
            ps.life = p.get("life").and_then(|x| x.as_i64()).unwrap_or(20) as i32;
            ps.energy = p.get("energy").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
            ps.lands_played = p.get("lands_played").and_then(|x| x.as_u64()).unwrap_or(0) as u8;
            ps.spells_cast = p.get("spells_cast_this_turn").and_then(|x| x.as_u64()).unwrap_or(0) as u8;
            if let Some(m) = p.get("mana_pool").and_then(|x| x.as_object()) {
                for (k, n) in m {
                    let i = match k.as_str() {
                        "W" => 0,
                        "U" => 1,
                        "B" => 2,
                        "R" => 3,
                        "G" => 4,
                        "C" => 5,
                        o => fail!("bad mana color {o}"),
                    };
                    ps.pool[i] = n.as_u64().unwrap_or(0) as u8;
                }
            }
            ents[si][0] = parse_list(p.get("hand"))?;
            let bf = parse_list(p.get("battlefield"))?;
            for e in bf {
                if e.token {
                    token_ents.push((Seat(si as u8), e));
                } else {
                    ents[si][1].push(e);
                }
            }
            ents[si][2] = parse_list(p.get("graveyard"))?;
            ents[si][3] = parse_list(p.get("exile"))?;
            ents[si][4] = parse_list(p.get("library"))?;
            let pad = p.get("library_pad").and_then(|x| x.as_u64()).unwrap_or(0) as usize;
            pad_names[si] = pad;
            for _ in 0..pad {
                ents[si][4].push(Entry { name: "Island".to_string(), ..Default::default() });
            }
            ps.hand = ents[si][0].iter().map(|e| e.name.clone()).collect();
            ps.battlefield = ents[si][1]
                .iter()
                .map(|e| PermSetup { name: e.name.clone(), tapped: e.tapped, summoning_sick: e.sick, counters: e.counters.clone(), damage: e.damage, controller: e.controller, chosen: e.chosen_name.as_ref().and_then(|n| db.id(n)).map(|d| d.0 as u16 + 1).or_else(|| e.chosen_type.as_ref().and_then(|t| subtype_index(t)).map(|t| t as u16 + 1)).unwrap_or(0) })
                .collect();
            ps.graveyard = ents[si][2].iter().map(|e| e.name.clone()).collect();
            ps.exile = ents[si][3].iter().map(|e| e.name.clone()).collect();
            ps.library = ents[si][4].iter().map(|e| e.name.clone()).collect();
        }
        for pl in players.iter() {
            for n in pl.hand.iter().chain(pl.graveyard.iter()).chain(pl.exile.iter()).chain(pl.library.iter()).chain(pl.battlefield.iter().map(|b| &b.name)) {
                if db.id(n).is_none() {
                    unsupported!("card not in pool: {n}");
                }
            }
        }
        let active = seat_of(s["active"].as_str().unwrap_or("p0"))?;
        let step = phase_of(s["phase"].as_str().unwrap_or("main1"))?;
        let turn = s["turn"].as_u64().unwrap_or(3) as u16;
        let sc = ScenarioSetup { turn, active, step, players, seed: 5 };
        let order = creation_order(&sc);
        let mut game = Game::from_scenario(db.clone(), &sc);
        let mut aliases = HashMap::new();
        let mut handles: HashMap<String, Alias> = HashMap::new();
        let mut counts: HashMap<(usize, String), usize> = HashMap::new();
        // Doc 04 handle order: battlefield, hand, graveyard, exile, library.
        let zone_rank = |z: ZoneKind| match z {
            ZoneKind::Battlefield => 0,
            ZoneKind::Hand => 1,
            ZoneKind::Graveyard => 2,
            ZoneKind::Exile => 3,
            _ => 4,
        };
        let mut by_handle: Vec<(usize, u8, usize, u16, String)> = Vec::new();
        for (k, (seat, zone, i)) in order.iter().enumerate() {
            let zi = match zone {
                ZoneKind::Hand => 0,
                ZoneKind::Battlefield => 1,
                ZoneKind::Graveyard => 2,
                ZoneKind::Exile => 3,
                _ => 4,
            };
            let e = &ents[seat.idx()][zi][*i];
            if let Some(a) = &e.alias {
                aliases.insert(a.clone(), Alias::Slot(k as u16));
            }
            by_handle.push((seat.idx(), zone_rank(*zone), *i, k as u16, e.name.clone()));
        }
        by_handle.sort();
        for (seat, _, _, slot, name) in by_handle {
            let c = counts.entry((seat, name.clone())).or_insert(0);
            handles.insert(format!("p{seat}:{name}#{c}"), Alias::Slot(slot));
            *c += 1;
        }
        // Tokens.
        let mut token_attach: Vec<(String, String)> = Vec::new();
        for (seat, e) in token_ents {
            let def = match db.id(&e.name) {
                Some(d) => d,
                None => unsupported!("token not in pool: {}", e.name),
            };
            let r = game.raw_state_mut().scenario_token(db, def, seat, e.sick);
            if e.tapped {
                game.raw_state_mut().scenario_set_tapped(r, true);
            }
            if !e.counters.is_empty() || e.damage > 0 {
                game.raw_state_mut().scenario_set_counters_damage(r, &e.counters, e.damage);
            }
            if let Some(a) = &e.alias {
                aliases.insert(a.clone(), Alias::Obj(r));
                if let Some(to) = &e.attached_to {
                    token_attach.push((a.clone(), to.clone()));
                }
            }
        }
        for (si, name, room) in dungeons {
            let def = match db.id(&name) {
                Some(d) if db.def(d).dungeon.is_some() => d,
                _ => unsupported!("dungeon not in pool: {name}"),
            };
            match room {
                None => game.raw_state_mut().scenario_completed(Seat(si as u8), def),
                Some(room) => {
                    let idx = match db.def(def).dungeon.as_ref().and_then(|d| d.rooms.iter().position(|r| r.name == room)) {
                        Some(i) => i as u8,
                        None => fail!("dungeon {name} has no room {room}"),
                    };
                    let r = game.raw_state_mut().scenario_dungeon(db, def, Seat(si as u8), idx);
                    for k in 0..2 {
                        handles.insert(format!("p{si}:{name}#{k}"), Alias::Obj(r));
                    }
                }
            }
        }
        let mut run = Runner { db: db.clone(), g: game, aliases, handles, step_no: 0, last_events: vec![], lenient: false };
        // Attachments (Equipment names the creature).
        for (si, per) in ents.iter().enumerate() {
            let _ = si;
            for list in per.iter() {
                for e in list {
                    if let (Some(a), Some(to)) = (&e.alias, &e.attached_to) {
                        let (ra, rt) = (run.obj_alias(&format!("@{a}"))?, run.obj_alias(&format!("@{to}"))?);
                        run.g.raw_state_mut().scenario_attach(ra, rt);
                    }
                }
            }
        }
        for (a, to) in token_attach {
            let (ra, rt) = (run.obj_alias(&format!("@{a}"))?, run.obj_alias(&format!("@{}", to.trim_start_matches('@')))?);
            run.g.raw_state_mut().scenario_attach(ra, rt);
        }
        // Abilities already on the stack (bottom first).
        for e in stack_setup.iter() {
            if e.get("kind").and_then(|k| k.as_str()) != Some("triggered") {
                unsupported!("setup.stack entry {e}");
            }
            let src = run.obj_alias(e["source"].as_str().unwrap_or(""))?;
            let want = e.get("text_contains").and_then(|t| t.as_str()).unwrap_or("");
            let def = run.db.def(run.st().def_of(src));
            let idx = match def.abilities.iter().position(|a| matches!(a, AbilityDef::Triggered(t) if t.text.contains(want))) {
                Some(i) => i as u8,
                None => fail!("{} has no triggered ability containing {want:?}", def.name),
            };
            let ctrl = seat_of(e["controller"].as_str().unwrap_or("p0"))?;
            let mut targets = Vec::new();
            for t in e.get("targets").and_then(|t| t.as_array()).cloned().unwrap_or_default() {
                targets.push(run.target_of(&t)?);
            }
            let dbc = run.db.clone();
            run.g.raw_state_mut().scenario_push_ability(&dbc, src, idx, ctrl, targets);
        }
        if let Some(p) = s.get("priority").and_then(|x| x.as_str()) {
            run.g.raw_state_mut().scenario_set_priority(seat_of(p)?);
        }
        run.g.raw_state_mut().scenario_explicit_mana();
        run.advance();
        Ok(run)
    }

    // ---- low-level helpers ----------------------------------------------------------------

    pub(crate) fn advance(&mut self) {
        self.g.advance();
    }

    pub(crate) fn pending(&self) -> Option<Pending> {
        self.g.raw_state().pending().cloned()
    }

    pub(crate) fn st(&self) -> &State {
        self.g.raw_state()
    }

    pub(crate) fn resolve_alias(&self, a: Alias) -> ObjRef {
        match a {
            Alias::Slot(s) => self.st().cur_ref_of_slot(s),
            Alias::Obj(r) => r,
        }
    }

    /// `"@alias"` or a doc 04 handle to the current object.
    pub(crate) fn obj_alias(&self, s: &str) -> R<ObjRef> {
        let key = s.trim_start_matches('@');
        if let Some(a) = self.aliases.get(key) {
            return Ok(self.resolve_alias(*a));
        }
        if let Some(a) = self.handles.get(s) {
            return Ok(self.resolve_alias(*a));
        }
        fail!("unknown alias {s:?}")
    }

    pub(crate) fn def_name(&self, r: ObjRef) -> String {
        self.db.def(self.st().def_of(r)).name.clone()
    }

    pub(crate) fn apply_opt(&mut self, idx: usize) -> R<()> {
        let p = match self.pending() {
            Some(p) => p,
            None => fail!("no decision pending"),
        };
        if std::env::var("MTG_TRACE").is_ok() {
            eprintln!("  apply {:?} for {:?}: option {idx} of [{}]", p.kind, p.seat, self.describe_opts(&p));
        }
        if let Result::Err(e) = self.g.apply(p.id, idx) {
            fail!("engine rejected option {idx}: {e:?}");
        }
        self.advance();
        Ok(())
    }

    /// Applies the first option satisfying `pred`.
    pub(crate) fn choose(&mut self, what: &str, pred: impl Fn(&Runner, &Opt) -> bool) -> R<()> {
        let p = match self.pending() {
            Some(p) => p,
            None => fail!("{what}: no decision pending"),
        };
        match p.options.iter().position(|o| pred(self, o)) {
            Some(i) => self.apply_opt(i),
            None => fail!("{what}: not offered; {:?} for {:?} offers {}", p.kind, p.seat, self.describe_opts(&p)),
        }
    }

    pub(crate) fn describe_opts(&self, p: &Pending) -> String {
        p.options.iter().map(|o| self.describe_opt(o)).collect::<Vec<_>>().join(" | ")
    }

    pub(crate) fn describe_opt(&self, o: &Opt) -> String {
        match *o {
            Opt::Cast(r, w) => format!("cast {}#{w}", self.def_name(r)),
            Opt::PlayLand(r) => format!("play {}", self.def_name(r)),
            Opt::Activate { src, ability } => format!("activate {}#{ability}", self.def_name(src)),
            Opt::Target(Target::Obj(r)) => format!("target {}", self.def_name(r)),
            Opt::Card(r) => format!("card {}", self.def_name(r)),
            Opt::Block(r) => format!("block {}", self.def_name(r)),
            Opt::Choice(i) if matches!(self.pending().map(|p| p.kind), Some(DecisionKind::ChooseDungeon | DecisionKind::ChooseRoom)) => self.dungeon_choice_name(i),
            o => format!("{o:?}"),
        }
    }

    /// Option refers to the object `want` (or to an interchangeable copy: the engine collapses
    /// identical cards, so the representative stands for all of them).
    pub(crate) fn same_obj(&self, got: ObjRef, want: ObjRef) -> bool {
        if got == want {
            return true;
        }
        let st = self.st();
        if !st.is_live_ref(got) || !st.is_live_ref(want) {
            return false;
        }
        let (a, b) = (st.obj_data(&self.db, got), st.obj_data(&self.db, want));
        a.def == b.def && a.zone == b.zone && a.owner == b.owner && a.controller == b.controller && a.tapped == b.tapped && a.kind == b.kind
            && a.damage == b.damage && a.counters == b.counters && a.chars == b.chars && a.summoning_sick == b.summoning_sick && a.attached_to.is_some() == b.attached_to.is_some()
    }

    /// After the engine acted on `got` while the script named `want` (interchangeable), swap the
    /// aliases so later steps refer to the physical card that was actually used.
    pub(crate) fn swap_identity(&mut self, got_slot: u16, want_slot: u16) {
        if got_slot == want_slot {
            return;
        }
        for a in self.aliases.values_mut() {
            if let Alias::Slot(s) = a {
                if *s == got_slot {
                    *s = want_slot;
                } else if *s == want_slot {
                    *s = got_slot;
                }
            }
        }
        for a in self.handles.values_mut() {
            if let Alias::Slot(s) = a {
                if *s == got_slot {
                    *s = want_slot;
                } else if *s == want_slot {
                    *s = got_slot;
                }
            }
        }
    }

    pub(crate) fn expect_priority(&self, seat: Seat) -> R<Pending> {
        match self.pending() {
            Some(p) if p.seat == seat && p.kind == DecisionKind::Priority => Ok(p),
            Some(p) => fail!("{:?} must act at priority but the engine asks {:?} for {:?} ({})", seat, p.kind, p.seat, self.describe_opts(&p)),
            None => fail!("game over or no decision; {seat:?} cannot act"),
        }
    }

    // ---- steps -------------------------------------------------------------------------------

    pub(crate) fn run_step(&mut self, step: &J) -> R<()> {
        let m = match step.as_object() {
            Some(m) => m,
            None => fail!("bad step"),
        };
        if let Some(a) = m.get("p0") {
            return self.run_seat_step(Seat(0), a);
        }
        if let Some(a) = m.get("p1") {
            return self.run_seat_step(Seat(1), a);
        }
        if let Some(a) = m.get("advance") {
            return self.do_advance(a);
        }
        if m.get("resolve_top").is_some() {
            return self.do_resolve_top();
        }
        if let Some(a) = m.get("check") {
            return self.check_block(a);
        }
        if let Some(a) = m.get("bind") {
            return self.do_bind(a);
        }
        if let Some(a) = m.get("expect_illegal") {
            return self.do_expect_illegal(a);
        }
        if m.get("random").is_some() {
            unsupported!("scripted random step");
        }
        if m.keys().all(|k| k == "label" || k == "note") {
            return Ok(());
        }
        fail!("unknown step kind: {}", brief_keys(m))
    }

    pub(crate) fn run_seat_step_pub(&mut self, seat: Seat, a: &J) -> R<()> {
        self.run_seat_step(seat, a)
    }

    fn run_seat_step(&mut self, seat: Seat, a: &J) -> R<()> {
        if let Some(t) = a.get("t").and_then(|x| x.as_str()) {
            return self.do_macro(seat, t, a);
        }
        if a.get("decision").is_some() {
            return self.do_decision(seat, a);
        }
        fail!("bad action {a}")
    }

    fn do_resolve_top(&mut self) -> R<()> {
        if self.st().stack().is_empty() {
            fail!("resolve_top with an empty stack");
        }
        let first = match self.pending() {
            Some(p) if p.kind == DecisionKind::Priority => p.seat,
            _ => fail!("resolve_top: no priority decision pending"),
        };
        let other = first.other();
        self.expect_priority(first)?;
        self.choose("pass", |_, o| *o == Opt::Pass)?;
        self.expect_priority(other).map_err(|e| ctx(e, "resolve_top second pass"))?;
        self.choose("pass", |_, o| *o == Opt::Pass)
    }

    fn do_advance(&mut self, a: &J) -> R<()> {
        let to = phase_of(a["to"].as_str().unwrap_or(""))?;
        let of = seat_of(a["of"].as_str().unwrap_or("p0"))?;
        for n in 0..400 {
            let td = self.st().turn_data();
            if let Some(p) = self.pending() {
                if n > 0 && p.kind == DecisionKind::Priority && td.active == of && td.step == to && p.seat == of {
                    return Ok(());
                }
                if p.kind != DecisionKind::Priority {
                    if p.options.len() == 1 && !matches!(p.kind, DecisionKind::ChooseTarget { .. }) {
                        // A decision with no real choice (declaring no attackers or blockers).
                        self.apply_opt(0)?;
                        continue;
                    }
                    fail!("advance: engine raised {:?} for {:?} on the way ({})", p.kind, p.seat, self.describe_opts(&p));
                }
                if !self.st().stack().is_empty() {
                    fail!("advance: stack not empty at a pass");
                }
                self.apply_opt(self.pass_idx(&p)?)?;
            } else {
                fail!("advance: game ended or no decision");
            }
        }
        fail!("advance: did not reach {to:?} of {of:?}")
    }

    fn pass_idx(&self, p: &Pending) -> R<usize> {
        match p.options.iter().position(|o| *o == Opt::Pass) {
            Some(i) => Ok(i),
            None => fail!("no pass option"),
        }
    }

    fn do_bind(&mut self, a: &J) -> R<()> {
        let name = a["as"].as_str().unwrap_or("").to_string();
        let f = &a["find"];
        let want_name = f.get("name").and_then(|x| x.as_str());
        let want_ctrl = match f.get("controller").and_then(|x| x.as_str()) {
            Some(c) => Some(seat_of(c)?),
            None => None,
        };
        let mut found = Vec::new();
        let st = self.st();
        for &r in st.battlefield() {
            let d = st.obj_data(&self.db, r);
            if want_name.map(|n| self.db.def(d.def).name == n).unwrap_or(true) && want_ctrl.map(|c| c == d.controller).unwrap_or(true) {
                found.push(r);
            }
        }
        if found.len() != 1 {
            fail!("bind {name}: {} objects match", found.len());
        }
        self.aliases.insert(name, Alias::Obj(found[0]));
        Ok(())
    }

    fn do_expect_illegal(&mut self, a: &J) -> R<()> {
        let (seat, act) = if let Some(x) = a.get("p0") {
            (Seat(0), x)
        } else if let Some(x) = a.get("p1") {
            (Seat(1), x)
        } else {
            fail!("expect_illegal without a seat")
        };
        // Try on a clone: the action must be impossible and nothing may change.
        let saved = self.g.clone();
        let aliases = self.aliases.clone();
        let handles = self.handles.clone();
        let before = self.g.state_hash();
        let r = self.run_seat_step(seat, act);
        let ok = match r {
            Result::Err(Err::Fail(_)) => true,
            Result::Err(Err::Unsupported(m)) => {
                self.g = saved;
                self.aliases = aliases;
                self.handles = handles;
                return Err(Err::Unsupported(m));
            }
            Ok(()) => false,
        };
        self.g = saved;
        self.aliases = aliases;
        self.handles = handles;
        if self.g.state_hash() != before {
            fail!("expect_illegal changed state");
        }
        if !ok {
            fail!("action was legal but the scenario says it is illegal: {act}");
        }
        Ok(())
    }
}

pub(crate) fn brief_keys(m: &serde_json::Map<String, J>) -> String {
    m.keys().cloned().collect::<Vec<_>>().join(",")
}

// ---------------------------------------------------------------------------------------------
// Macro actions
// ---------------------------------------------------------------------------------------------

impl Runner {
    fn do_macro(&mut self, seat: Seat, t: &str, a: &J) -> R<()> {
        match t {
            "pass" => {
                let p = self.expect_priority(seat)?;
                let i = self.pass_idx(&p)?;
                self.apply_opt(i)
            }
            "play_land" => {
                self.expect_priority(seat)?;
                let want = self.obj_alias(a["card"].as_str().unwrap_or(""))?;
                if let Some(life) = a.get("back_face").map(|x| x.get("pay_life").and_then(|y| y.as_bool()).unwrap_or(false)) {
                    return self.choose_obj("play_land back face", want, |o| if let Opt::PlayLandBack(r, l) = o { if *l == life { Some(*r) } else { None } } else { None });
                }
                self.choose_obj("play_land", want, |o| if let Opt::PlayLand(r) = o { Some(*r) } else { None })
            }
            "cast" => self.do_cast(seat, a),
            "activate" | "activate_from_hand" => self.do_activate(seat, a, t == "activate_from_hand"),
            "legend_rule" => {
                let want = self.obj_alias(a["keep"].as_str().unwrap_or(""))?;
                self.choose_obj("legend_rule", want, |o| if let Opt::Card(r) = o { Some(*r) } else { None })
            }
            "simultaneous_secret_choice" => match a["answer"].as_str() {
                Some(al) => {
                    let want = self.obj_alias(al)?;
                    self.choose_obj("simultaneous_secret_choice", want, |o| if let Opt::Card(r) = o { Some(*r) } else { None })
                }
                None => self.choose("none", |_, o| *o == Opt::Done),
            },
            "declare_attackers" => self.do_declare_attackers(seat, a),
            "declare_blockers" => self.do_declare_blockers(seat, a),
            other => fail!("unknown action {other:?}"),
        }
    }

    /// Chooses the option naming `want` (or an interchangeable object).
    pub(crate) fn choose_obj(&mut self, what: &str, want: ObjRef, get: impl Fn(&Opt) -> Option<ObjRef>) -> R<()> {
        let p = match self.pending() {
            Some(p) => p,
            None => fail!("{what}: no decision"),
        };
        let mut hit = None;
        // The named object itself first: interchangeable copies may differ in what the scenario checks.
        for (i, o) in p.options.iter().enumerate() {
            if get(o) == Some(want) {
                hit = Some((i, want));
                break;
            }
        }
        if hit.is_none() {
            for (i, o) in p.options.iter().enumerate() {
                if let Some(r) = get(o) {
                    if self.same_obj(r, want) {
                        hit = Some((i, r));
                        break;
                    }
                }
            }
        }
        match hit {
            Some((i, got)) => {
                let (gs, ws) = (got.slot, want.slot);
                self.apply_opt(i)?;
                self.swap_identity(gs, ws);
                Ok(())
            }
            None => fail!("{what}: {} is not offered; {:?} for {:?} offers {}", self.def_name(want), p.kind, p.seat, self.describe_opts(&p)),
        }
    }

    pub(crate) fn alt_way_for(&self, card: ObjRef, a: &J) -> R<u8> {
        let alt = match a.get("alt_cost") {
            None | Some(J::Null) => return Ok(0),
            Some(x) => x,
        };
        if alt.get("adventure").is_some() {
            return Ok(mtg_core::restrict::ADVENTURE_WAY);
        }
        let keys: Vec<String> = alt.as_object().map(|m| m.keys().cloned().collect()).unwrap_or_default();
        let def = self.db.def(self.st().def_of(card));
        if alt.get("free").and_then(|x| x.as_str()).and_then(|k| self.db.permit_way(k)).is_some() && keys.iter().any(|k| k != "free" && def.abilities.iter().any(|x| matches!(x, AbilityDef::Alt(a) if a.key == *k))) {
            fail!("alternative costs are exclusive (118.9a): {keys:?}");
        }
        // The way is named by the first key that is not just a payment detail of another.
        for (i, ab) in def.abilities.iter().filter_map(|x| if let AbilityDef::Alt(a) = x { Some(a) } else { None }).enumerate() {
            let k = if ab.key.is_empty() { ab.label.clone() } else { ab.key.clone() };
            if keys.iter().any(|x| *x == k) || (keys.len() == 1 && keys[0] == "free" && k == format!("free:{}", alt["free"].as_str().unwrap_or(""))) {
                return Ok(i as u8 + 1);
            }
        }
        if keys.iter().any(|k| k == "free") {
            if let Some(w) = alt["free"].as_str().and_then(|k| self.db.permit_way(k)) {
                if keys.iter().any(|k| k != "free" && def.abilities.iter().any(|x| matches!(x, AbilityDef::Alt(a) if a.key == *k))) {
                    fail!("alternative costs are exclusive (118.9a): {keys:?}");
                }
                return Ok(w);
            }
            unsupported!("free cast permission ({})", alt["free"]);
        }
        unsupported!("alt cost {keys:?} on {}", def.name)
    }

    fn do_cast(&mut self, seat: Seat, a: &J) -> R<()> {
        self.expect_priority(seat)?;
        let want = self.obj_alias(a["card"].as_str().unwrap_or(""))?;
        let way = self.alt_way_for(want, a)?;
        self.set_pay_hint(a)?;
        let r = self.choose_obj("cast", want, |o| match o {
            Opt::Cast(r, w) if *w == way => Some(*r),
            _ => None,
        });
        let r = r.and_then(|_| self.drive_cast(seat, a, false));
        self.g.raw_state_mut().scenario_pay_hint(Vec::new());
        r
    }

    fn do_activate(&mut self, seat: Seat, a: &J, from_hand: bool) -> R<()> {
        self.expect_priority(seat)?;
        let src = self.obj_alias(a["source"].as_str().unwrap_or(""))?;
        if (self.st().zone_of(src) == ZoneKind::Hand) != from_hand {
            fail!("activate_from_hand/activate used for a source in {:?}", self.st().zone_of(src));
        }
        let idx = a["ability"].as_u64().unwrap_or(0) as usize;
        let def = self.db.def(self.st().def_of(src));
        let acts: Vec<(usize, &ActivatedDef)> = def.abilities.iter().enumerate().filter_map(|(i, x)| if let AbilityDef::Activated(d) = x { Some((i, d)) } else { None }).collect();
        let want_color: Option<u8> = a.get("choose_color").and_then(|x| x.as_str()).map(|c| match c {
            "W" => 0,
            "U" => 1,
            "B" => 2,
            "R" => 3,
            "G" => 4,
            _ => 5,
        });
        // Mana abilities are activated through `Opt::Mana`.
        let intrinsic = {
            let sub = self.st().obj_data(&self.db, src).chars.subtypes;
            acts.get(idx).is_none() && [SUB_PLAINS, SUB_ISLAND, SUB_SWAMP, SUB_MOUNTAIN, SUB_FOREST].iter().any(|s| sub.has(*s))
        };
        let mana_ab: Option<u8> = match acts.get(idx) {
            Some((i, d)) if d.is_mana => Some(*i as u8),
            Some(_) => None,
            None if intrinsic && idx == 0 => Some(255),
            None => None,
        };
        if let Some(mab) = mana_ab {
            let fixed: Option<u8> = if mab == 255 { None } else { def.activated(mab).and_then(|d| d.effect.as_ref()).and_then(mana_fixed_color) };
            let color = match (want_color, fixed) {
                (Some(c), _) => c,
                (None, Some(f)) => f,
                (None, None) => {
                    // An intrinsic ability of a single-type land.
                    let sub = self.st().obj_data(&self.db, src).chars.subtypes;
                    match [(SUB_PLAINS, 0u8), (SUB_ISLAND, 1), (SUB_SWAMP, 2), (SUB_MOUNTAIN, 3), (SUB_FOREST, 4)].iter().find(|(s, _)| sub.has(*s)) {
                        Some((_, c)) => *c,
                        None => unsupported!("mana ability needs choose_color"),
                    }
                }
            };
            let sac_pick: Option<Vec<u16>> = match a.get("sacrifice").and_then(|x| x.as_array()) {
                Some(l) => {
                    let mut v = Vec::new();
                    for e in l {
                        v.push(self.obj_alias(e.as_str().unwrap_or(""))?.slot);
                    }
                    Some(v)
                }
                None => None,
            };
            return self.choose_obj("activate mana", src, |o| match o {
                Opt::Mana { src, ability, color: c, pick } if *ability == mab && *c == color && pick.map(|p| sac_pick.as_ref().map(|w| w.contains(&p.slot)).unwrap_or(false)).unwrap_or(sac_pick.is_none()) => Some(*src),
                _ => None,
            });
        }
        let ab = acts.get(idx).map(|x| x.0);
        let ab = match ab {
            Some(i) => i as u8,
            None => unsupported!("activate ability {idx} of {}", def.name),
        };
        // Ninjutsu's returned attacker is a separate `choose_cards` step after the activation.
        let stop_pick = def.activated(ab).map(|d| d.cost.iter().any(|c| matches!(c, CostItem::ReturnToHand(f) if f.unblocked == Some(true)))).unwrap_or(false);
        self.choose_obj("activate", src, |o| match o {
            Opt::Activate { src, ability } if *ability == ab => Some(*src),
            _ => None,
        })?;
        self.drive_cast(seat, a, stop_pick)
    }

    /// Answers the sub-decisions of a cast/activation from the action's fields.
    fn drive_cast(&mut self, seat: Seat, a: &J, stop_pick: bool) -> R<()> {
        let mut targets: Vec<J> = a.get("targets").and_then(|x| x.as_array()).cloned().unwrap_or_default();
        targets.reverse();
        let mut modes: Vec<u64> = a.get("modes").and_then(|x| x.as_array()).map(|v| v.iter().filter_map(|x| x.as_u64()).collect()).unwrap_or_default();
        modes.reverse();
        let mut phy: Vec<String> = a.get("phyrexian").and_then(|x| x.as_array()).map(|v| v.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()).unwrap_or_default();
        phy.reverse();
        let mut pick_aliases: Vec<String> = Vec::new();
        fn collect_aliases(v: &J, out: &mut Vec<String>) {
            match v {
                J::String(s) if s.starts_with('@') => out.push(s.clone()),
                J::Array(l) => l.iter().for_each(|x| collect_aliases(x, out)),
                J::Object(m) => m.values().for_each(|x| collect_aliases(x, out)),
                _ => {}
            }
        }
        for key in ["alt_cost", "sacrifice", "discard", "exile", "return"] {
            if let Some(m) = a.get(key) {
                collect_aliases(m, &mut pick_aliases);
            }
        }
        let mut delve_aliases: Vec<String> = a.get("delve").and_then(|x| x.as_array()).map(|l| l.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()).unwrap_or_default();
        let delve_key: Vec<String> = delve_aliases.clone();
        if let Some(l) = a.get("alt_cost").and_then(|x| x.get("escape")).and_then(|x| x.get("exile")).and_then(|x| x.as_array()) {
            delve_aliases.extend(l.iter().filter_map(|x| x.as_str().map(|s| s.to_string())));
        }
        let x_val = a.get("x").and_then(|v| v.as_u64());
        for _ in 0..64 {
            let p = match self.pending() {
                Some(p) => p,
                None => break,
            };
            if self.st().top_frame_kind() != "cast" || p.seat != seat {
                break;
            }
            match p.kind {
                DecisionKind::ChooseMode { .. } => match modes.pop() {
                    Some(m) => self.choose("mode", |_, o| *o == Opt::Mode(m as u8 - 1))?,
                    None => self.choose("modes done", |_, o| *o == Opt::Done)?,
                },
                DecisionKind::ChooseReplicate => {
                    let n = a.get("replicate").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    self.choose("replicate", |_, o| *o == Opt::Number(n))?
                }
                DecisionKind::ChooseX => match x_val {
                    Some(x) => self.choose("x", |_, o| *o == Opt::Number(x as u32))?,
                    None if self.lenient => self.apply_opt(0)?,
                    None => fail!("cast needs X but the action gives none"),
                },
                DecisionKind::ChooseTarget { .. } => match targets.pop() {
                    Some(t) => self.choose_target_spec(&t)?,
                    None => {
                        if let Some(i) = p.options.iter().position(|o| *o == Opt::Done) {
                            // An optional slot the script leaves empty.
                            self.apply_opt(i)?;
                        } else if (p.options.len() == 1 && a.get("targets").is_none()) || self.lenient {
                            self.apply_opt(0)?;
                        } else {
                            fail!("cast raised a target choice the action does not specify ({})", self.describe_opts(&p));
                        }
                    }
                },
                DecisionKind::PayPhyrexian { .. } => match phy.pop() {
                    Some(m) => {
                        let want = if m == "life" { Opt::Yes } else { Opt::No };
                        self.choose("phyrexian", |_, o| *o == want)?
                    }
                    None if self.lenient => self.apply_opt(0)?,
                    None => fail!("cast raised a Phyrexian choice the action does not specify"),
                },
                DecisionKind::ChooseAddCost { .. } => {
                    if p.options.len() == 1 || self.lenient {
                        self.apply_opt(0)?;
                    } else {
                        unsupported!("additional cost choice in a scenario action");
                    }
                }
                DecisionKind::ChooseCards { purpose: CardsPurpose::Delve, .. } => {
                    let p2 = self.pending().unwrap();
                    let mut hit: Option<(usize, usize)> = None;
                    for (ai, al) in delve_aliases.iter().enumerate() {
                        let want = self.obj_alias(al)?;
                        if let Some(i) = p2.options.iter().position(|o| matches!(o, Opt::Card(r) if self.same_obj(*r, want))) {
                            hit = Some((ai, i));
                            break;
                        }
                    }
                    match hit {
                        Some((ai, i)) => {
                            delve_aliases.remove(ai);
                            self.apply_opt(i)?;
                        }
                        None if delve_aliases.is_empty() => match p2.options.iter().position(|o| *o == Opt::Done) {
                            Some(i) => self.apply_opt(i)?,
                            None => fail!("cast needs more delve but the action names no more cards ({})", self.describe_opts(&p2)),
                        },
                        None => fail!("none of the delve cards {delve_aliases:?} is a legal choice ({})", self.describe_opts(&p2)),
                    }
                }
                DecisionKind::ChooseCards { .. } => {
                    let mut done = false;
                    if stop_pick && pick_aliases.is_empty() {
                        break;
                    }
                    if !pick_aliases.is_empty() {
                        let p2 = self.pending().unwrap();
                        let mut hit: Option<(usize, usize)> = None;
                        for (ai, al) in pick_aliases.iter().enumerate() {
                            let want = self.obj_alias(al)?;
                            if let Some(i) = p2.options.iter().position(|o| matches!(o, Opt::Card(r) if self.same_obj(*r, want))) {
                                hit = Some((ai, i));
                                break;
                            }
                        }
                        match hit {
                            Some((ai, i)) => {
                                pick_aliases.remove(ai);
                                self.apply_opt(i)?;
                                done = true;
                            }
                            None => fail!("none of the named cost objects {pick_aliases:?} is a legal choice ({})", self.describe_opts(&p2)),
                        }
                    }
                    if !done {
                        let p2 = self.pending().unwrap();
                        if p2.options.len() == 1 || self.lenient {
                            self.apply_opt(0)?;
                        } else {
                            fail!("cast raised a card choice the action does not answer ({})", self.describe_opts(&p2));
                        }
                    }
                }
                _ => break,
            }
        }
        if !targets.is_empty() {
            fail!("action gave more targets than the engine asked for");
        }
        if delve_aliases.iter().any(|x| delve_key.contains(x)) {
            fail!("action names more delve cards than the generic cost can take (702.66a): {delve_aliases:?}");
        }
        Ok(())
    }

    /// Target given as an alias, `{player: pN}`, or `{trigger_of|ability_of: alias, nth}`.
    pub(crate) fn choose_target_spec(&mut self, t: &J) -> R<()> {
        let want: Target = self.target_of(t)?;
        let p = match self.pending() {
            Some(p) => p,
            None => fail!("no decision"),
        };
        match p.options.iter().position(|o| *o == Opt::Target(want)) {
            Some(i) => self.apply_opt(i),
            None => fail!("target {t} is not legal; options: {}", self.describe_opts(&p)),
        }
    }

    pub(crate) fn target_of(&self, t: &J) -> R<Target> {
        match t {
            J::String(s) => Ok(Target::Obj(self.obj_alias(s)?)),
            J::Object(m) => {
                if let Some(p) = m.get("player").and_then(|x| x.as_str()) {
                    return Ok(Target::Player(seat_of(p)?));
                }
                let key = if m.contains_key("trigger_of") { "trigger_of" } else { "ability_of" };
                if let Some(src) = m.get(key).and_then(|x| x.as_str()) {
                    let src = self.obj_alias(src)?;
                    let nth = m.get("nth").and_then(|x| x.as_u64()).unwrap_or(0) as usize;
                    let st = self.st();
                    let tc = m.get("text_contains").cloned();
                    let mut hits: Vec<ObjRef> = Vec::new();
                    for e in st.stack().iter().rev() {
                        if let StackKind::Ability { source, ability } = e.kind {
                            if source.slot != src.slot {
                                continue;
                            }
                            if let Some(t) = &tc {
                                if !self.ability_matches(&serde_json::json!({ "text_contains": t }), source, e.def, ability as usize)? {
                                    continue;
                                }
                            }
                            hits.push(e.obj);
                        }
                    }
                    if nth >= hits.len() {
                        fail!("no {nth}-th ability of that source on the stack");
                    }
                    return Ok(Target::Obj(hits.swap_remove(nth)));
                }
                fail!("bad target {t}")
            }
            _ => fail!("bad target {t}"),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Decision steps
// ---------------------------------------------------------------------------------------------

impl Runner {
    fn do_decision(&mut self, seat: Seat, a: &J) -> R<()> {
        let kind = a["decision"].as_str().unwrap_or("");
        // Decisions the rules give no choice on may be skipped by the script: take them silently.
        for _ in 0..8 {
            match self.pending() {
                Some(p) if p.seat == seat && self.kind_matches(kind, &p) => break,
                Some(p) if p.options.len() == 1 && p.kind != DecisionKind::Priority => {
                    self.apply_opt(0)?;
                }
                // A script that goes straight to the copy's new target means "yes, change them".
                Some(p) if p.seat == seat && p.kind == DecisionKind::ChangeTargets && kind == "choose_target" => {
                    self.apply_opt(0)?;
                }
                _ => break,
            }
        }
        let p = match self.pending() {
            Some(p) => p,
            None => fail!("decision {kind} but the game has no pending decision"),
        };
        if p.seat != seat || !self.kind_matches(kind, &p) {
            fail!("script answers {kind} for {seat:?} but the engine asks {:?} for {:?} ({})", p.kind, p.seat, self.describe_opts(&p));
        }
        if let Some(eo) = a.get("expect_options") {
            self.check_options(eo, &p)?;
        }
        match kind {
            "yes_no" => {
                let yes = a["answer"].as_bool().unwrap_or(false);
                let want = if yes { Opt::Yes } else { Opt::No };
                self.choose("yes_no", |_, o| *o == want)
            }
            "choose_target" => {
                let t = self.target_of(&a["answer"])?;
                match p.options.iter().position(|o| *o == Opt::Target(t)) {
                    Some(i) => self.apply_opt(i),
                    None => fail!("target not legal; options {}", self.describe_opts(&p)),
                }
            }
            "choose_cards" => {
                let ans: Vec<J> = a["answer"].as_array().cloned().unwrap_or_default();
                for al in ans {
                    let want = self.obj_alias(al.as_str().unwrap_or(""))?;
                    self.choose_obj("choose_cards", want, |o| if let Opt::Card(r) = o { Some(*r) } else { None })?;
                }
                // A selection that ends by itself needs no Done; otherwise finish it.
                if let Some(p2) = self.pending() {
                    if p2.seat == seat && matches!(p2.kind, DecisionKind::ChooseCards { .. }) && p2.options.contains(&Opt::Done) && same_kind(&p.kind, &p2.kind) {
                        self.choose("done", |_, o| *o == Opt::Done)?;
                    }
                }
                Ok(())
            }
            "declare_attackers" => self.do_declare_attackers(seat, a),
            "choose_attack_target" => {
                // The target of a creature that entered attacking (508.4): a player or a planeswalker.
                let want = match self.target_of(&a["answer"])? {
                    Target::Player(p) => AttackTarget::Player(p),
                    Target::Obj(r) => AttackTarget::Walker(r),
                    Target::None => fail!("choose_attack_target needs a player or a planeswalker"),
                };
                match p.options.iter().position(|o| *o == Opt::Attack(want)) {
                    Some(i) => self.apply_opt(i),
                    None => fail!("cannot attack that; options {}", self.describe_opts(&p)),
                }
            }
            "declare_blockers" => self.do_declare_blockers(seat, a),
            "assign_combat_damage" => self.do_assign_damage(seat, a),
            "order_cards" => self.apply_order(&a["answer"]),
            "legend_rule" => {
                let want = self.obj_alias(a["keep"].as_str().or_else(|| a["answer"].as_str()).unwrap_or(""))?;
                self.choose_obj("legend_rule", want, |o| if let Opt::Card(r) = o { Some(*r) } else { None })
            }
            "order_triggers" => self.do_order_triggers(seat, &a["answer"]),
            "simultaneous_secret_choice" => match a["answer"].as_str() {
                Some(al) => {
                    let want = self.obj_alias(al)?;
                    self.choose_obj("simultaneous_secret_choice", want, |o| if let Opt::Card(r) = o { Some(*r) } else { None })
                }
                None => self.choose("none", |_, o| *o == Opt::Done),
            },
            "choose_number" => {
                let n = a["answer"].as_u64().unwrap_or(0) as u32;
                self.choose("choose_number", |_, o| *o == Opt::Number(n))
            }
            "choose_color" => {
                let ans = a["answer"].as_str().unwrap_or("");
                match ["W", "U", "B", "R", "G"].iter().position(|c| *c == ans) {
                    Some(i) => self.choose("choose_color", |_, o| *o == Opt::Choice(i as u8)),
                    None => fail!("choose_color: bad color {ans:?}"),
                }
            }
            "choose_dungeon" | "choose_room" => {
                let ans = a["answer"].as_str().unwrap_or("");
                let hit = (0..p.options.len()).find(|&i| matches!(p.options[i], Opt::Choice(c) if self.dungeon_choice_name(c) == ans));
                match hit {
                    Some(i) => self.apply_opt(i),
                    None => fail!("{kind}: {ans:?} is not offered"),
                }
            }
            "choose_name" => {
                let ans = a["answer"].as_str().unwrap_or("");
                let hit = p.options.iter().position(|o| match o {
                    Opt::Name(d) => self.db.def(CardDefId(*d)).name == ans,
                    Opt::Type(t) => SUBTYPE_NAMES[*t as usize] == ans,
                    _ => false,
                });
                match hit {
                    Some(i) => self.apply_opt(i),
                    None => fail!("choose_name: {ans:?} is not offered"),
                }
            }
            "surveil" | "scry" => {
                let key = if kind == "surveil" { "to_graveyard" } else { "to_bottom" };
                for al in a.get(key).and_then(|x| x.as_array()).cloned().unwrap_or_default() {
                    let want = self.obj_alias(al.as_str().unwrap_or(""))?;
                    self.choose_obj(kind, want, |o| if let Opt::Card(r) = o { Some(*r) } else { None })?;
                }
                if let Some(p2) = self.pending() {
                    if p2.seat == seat && same_kind(&p.kind, &p2.kind) && is_purpose(&p2.kind, if kind == "surveil" { CardsPurpose::SurveilGraveyard } else { CardsPurpose::ScryBottom }) {
                        self.choose("done", |_, o| *o == Opt::Done)?;
                    }
                }
                // Scripts that do not mention the order of the bottomed cards accept any order.
                self.apply_order(&a["on_top_order"])
            }
            "search" => {
                let found: Vec<J> = match &a["found"] {
                    J::Null => vec![],
                    J::Array(l) => l.clone(),
                    x => vec![x.clone()],
                };
                for al in found {
                    let want = self.obj_alias(al.as_str().unwrap_or(""))?;
                    self.choose_obj("search", want, |o| if let Opt::Card(r) = o { Some(*r) } else { None })?;
                }
                if let Some(p2) = self.pending() {
                    if p2.seat == seat && (is_purpose(&p2.kind, CardsPurpose::Search) || is_purpose(&p2.kind, CardsPurpose::ExtractExile)) {
                        self.choose("search done", |_, o| *o == Opt::Done)?;
                    }
                }
                Ok(())
            }
            other => unsupported!("decision kind {other}"),
        }
    }

    /// The Oracle sentence of ability `ability` of `def`.
    pub(crate) fn ability_text(&self, def: CardDefId, ability: usize) -> String {
        match self.db.def(def).abilities.get(ability) {
            Some(AbilityDef::Triggered(t)) => t.text.clone(),
            Some(AbilityDef::Activated(a)) => a.text.clone(),
            _ => String::new(),
        }
    }

    /// Whether descriptor `d` (an alias, or `{source, text_contains}`) names the ability
    /// (`source`, `def`, `ability`).
    pub(crate) fn ability_matches(&self, d: &J, source: ObjRef, def: CardDefId, ability: usize) -> R<bool> {
        let (src, text) = match d {
            J::String(s) => (Some(s.as_str()), None),
            J::Object(m) => (m.get("source").and_then(|x| x.as_str()), m.get("text_contains").and_then(|x| x.as_str())),
            _ => fail!("bad ability descriptor {d}"),
        };
        if let Some(s) = src {
            if s.starts_with('@') || s.contains(':') {
                if !self.same_obj(self.obj_alias(s)?, source) {
                    return Ok(false);
                }
            } else if self.db.def(def).name != s {
                return Ok(false);
            }
        }
        if let Some(t) = text {
            if !self.ability_text(def, ability).to_lowercase().contains(&t.to_lowercase()) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Answers `order_triggers`: the first listed trigger is put on the stack first.
    fn do_order_triggers(&mut self, seat: Seat, answer: &J) -> R<()> {
        for d in answer.as_array().cloned().unwrap_or_default() {
            let p = match self.pending() {
                Some(p) if p.seat == seat && p.kind == DecisionKind::OrderTriggers => p,
                _ => break,
            };
            let mut hit = None;
            for (i, o) in p.options.iter().enumerate() {
                if let Opt::Choice(k) = o {
                    if let Some(pt) = self.st().pending_trigger(*k as usize) {
                        if self.ability_matches(&d, pt.source, pt.def, pt.ability as usize)? {
                            hit = Some(i);
                            break;
                        }
                    }
                }
            }
            match hit {
                Some(i) => self.apply_opt(i)?,
                None => fail!("order_triggers: no pending trigger matches {d}; options {}", self.describe_opts(&p)),
            }
        }
        Ok(())
    }

    /// Applies an ordering answer (top first) to the pending ordering decisions; decisions with
    /// one card left are forced and may already be gone.
    fn apply_order(&mut self, answer: &J) -> R<()> {
        for al in answer.as_array().cloned().unwrap_or_default() {
            let want = self.obj_alias(al.as_str().unwrap_or(""))?;
            match self.pending() {
                Some(p) if matches!(p.kind, DecisionKind::ChooseCards { purpose, .. } if purpose.is_order()) => {
                    self.choose_obj("order_cards", want, |o| if let Opt::Card(r) = o { Some(*r) } else { None })?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub(crate) fn kind_matches(&self, script_kind: &str, p: &Pending) -> bool {
        match (script_kind, p.kind) {
            ("yes_no", DecisionKind::May) | ("yes_no", DecisionKind::PayUnless) | ("yes_no", DecisionKind::ChangeTargets) => true,
            ("choose_target", DecisionKind::ChooseTarget { .. }) => true,
            ("choose_cards", DecisionKind::ChooseCards { purpose, .. }) => !purpose.is_order() && !matches!(purpose, CardsPurpose::SurveilGraveyard | CardsPurpose::ScryBottom | CardsPurpose::Search | CardsPurpose::ExtractExile),
            ("order_cards", DecisionKind::ChooseCards { purpose, .. }) => purpose.is_order(),
            ("simultaneous_secret_choice", DecisionKind::ChooseCards { purpose, .. }) => purpose == CardsPurpose::PutEach,
            ("surveil", DecisionKind::ChooseCards { purpose, .. }) => purpose == CardsPurpose::SurveilGraveyard,
            ("scry", DecisionKind::ChooseCards { purpose, .. }) => purpose == CardsPurpose::ScryBottom,
            ("search", DecisionKind::ChooseCards { purpose, .. }) => matches!(purpose, CardsPurpose::Search | CardsPurpose::ExtractExile),
            ("order_triggers", DecisionKind::OrderTriggers) => true,
            ("choose_mode", DecisionKind::ChooseMode { .. }) => true,
            ("choose_number", DecisionKind::ChooseX) | ("choose_number", DecisionKind::ChooseReplicate) | ("choose_number", DecisionKind::PayEnergyAmount) => true,
            ("declare_attackers", DecisionKind::DeclareAttacker { .. }) | ("choose_attack_target", DecisionKind::DeclareAttacker { .. }) => true,
            ("declare_blockers", DecisionKind::DeclareBlocker { .. }) => true,
            ("assign_combat_damage", DecisionKind::AssignDamage { .. }) => true,
            ("choose_color", DecisionKind::ChooseColor) => true,
            ("choose_dungeon", DecisionKind::ChooseDungeon) | ("choose_room", DecisionKind::ChooseRoom) => true,
            ("legend_rule", DecisionKind::LegendRule) => true,
            ("choose_name", DecisionKind::ChooseName { .. }) | ("choose_name", DecisionKind::ChooseType) => true,
            _ => false,
        }
    }

    /// The name of dungeon (or room) choice `i` of the pending decision.
    fn dungeon_choice_name(&self, i: u8) -> String {
        let p = self.pending().expect("pending decision");
        match p.kind {
            DecisionKind::ChooseDungeon => self.db.def(self.db.dungeons()[i as usize]).name.clone(),
            _ => match self.st().dungeon_of(p.seat) {
                Some((d, room)) => {
                    let dd = self.db.def(d).dungeon.as_ref().expect("dungeon def");
                    dd.rooms[dd.rooms[room as usize].exits[i as usize] as usize].name.clone()
                }
                None => String::new(),
            },
        }
    }

    /// Does the script entry (an alias, a card or option name, or a boolean) name option `o`?
    fn option_matches(&self, e: &J, o: &Opt) -> R<bool> {
        let obj_of = |o: &Opt| -> Option<ObjRef> {
            match o {
                Opt::Card(r) | Opt::Target(Target::Obj(r)) | Opt::Cast(r, _) | Opt::PlayLand(r) | Opt::Block(r) => Some(*r),
                Opt::Activate { src, .. } => Some(*src),
                Opt::Choice(i) if matches!(self.pending().map(|p| p.kind), Some(DecisionKind::OrderTriggers)) => self.st().pending_trigger(*i as usize).map(|t| t.source),
                _ => None,
            }
        };
        match e {
            J::Bool(b) => Ok(*o == if *b { Opt::Yes } else { Opt::No }),
            J::Null => Ok(*o == Opt::Done),
            J::String(s) if s.starts_with('@') || s.contains(':') => {
                let want = self.obj_alias(s)?;
                Ok(obj_of(o).map(|r| self.same_obj(r, want)).unwrap_or(false))
            }
            J::String(s) => Ok(match o {
                Opt::Name(d) => self.db.def(CardDefId(*d)).name == *s,
                Opt::Type(t) => SUBTYPE_NAMES[*t as usize] == *s,
                Opt::Choice(i) if matches!(self.pending().map(|p| p.kind), Some(DecisionKind::ChooseDungeon | DecisionKind::ChooseRoom)) => self.dungeon_choice_name(*i) == *s,
                Opt::Choice(i) if matches!(self.pending().map(|p| p.kind), Some(DecisionKind::ChooseColor)) => ["W", "U", "B", "R", "G"].get(*i as usize) == Some(&s.as_str()),
                _ => obj_of(o).map(|r| self.db.def(self.st().def_of(r)).name == *s).unwrap_or(false),
            }),
            J::Object(m) if m.contains_key("player") => {
                let seat = seat_of(m["player"].as_str().unwrap_or(""))?;
                Ok(*o == Opt::Target(Target::Player(seat)))
            }
            _ => Ok(false),
        }
    }

    fn check_options(&self, eo: &J, p: &Pending) -> R<()> {
        // Declarations are asked creature by creature; the script lists the creatures involved.
        let virtual_cards: Option<Vec<ObjRef>> = match p.kind {
            DecisionKind::DeclareAttacker { .. } | DecisionKind::DeclareBlocker { .. } => Some(self.st().declaration_candidates()),
            DecisionKind::AssignDamage { attacker, .. } => self.st().combat().and_then(|c| c.attackers.iter().find(|a| a.obj == attacker)).map(|a| a.blockers.to_vec()),
            _ => None,
        };
        for (key, want_in) in [("include", true), ("exclude", false), ("exact", true)] {
            if let Some(l) = eo.get(key).and_then(|x| x.as_array()) {
                for e in l {
                    let player_entry = e.get("player").is_some();
                    let hit = match &virtual_cards {
                        Some(_) if player_entry => match p.kind {
                            DecisionKind::AssignDamage { attacker, .. } => self.st().obj_data(&self.db, attacker).chars.keywords.contains(Keywords::TRAMPLE),
                            _ => false,
                        },
                        Some(v) => v.iter().any(|&r| self.option_matches(e, &Opt::Card(r)).unwrap_or(false)),
                        None => p.options.iter().any(|o| self.option_matches(e, o).unwrap_or(false)),
                    };
                    if hit != want_in {
                        fail!("expect_options {key} {e}: engine offers {}", self.describe_opts(p));
                    }
                }
                if key == "exact" && virtual_cards.is_none() {
                    let with_none = l.iter().any(|x| x.is_null());
                    let n = p.options.iter().filter(|o| with_none || **o != Opt::Done).count();
                    if n != l.len() {
                        fail!("expect_options exact {} entries but the engine offers {} ({})", l.len(), n, self.describe_opts(p));
                    }
                }
            }
        }
        if let Some(c) = eo.get("count").and_then(|x| x.as_u64()) {
            // `Done` is the end of a selection, not one of the options the script counts.
            let none_counts = matches!(p.kind, DecisionKind::ChooseCards { purpose: CardsPurpose::PutEach | CardsPurpose::PutOnto, .. });
            let n = p.options.iter().filter(|o| none_counts || **o != Opt::Done).count();
            if n as u64 != c {
                fail!("expect_options count {c} but the engine offers {} ({})", p.options.len(), self.describe_opts(p));
            }
        }
        Ok(())
    }
}

fn is_purpose(k: &DecisionKind, p: CardsPurpose) -> bool {
    matches!(k, DecisionKind::ChooseCards { purpose, .. } if *purpose == p)
}

fn same_kind(a: &DecisionKind, b: &DecisionKind) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

/// The color of a mana ability that always makes one color.
fn mana_fixed_color(e: &Effect) -> Option<u8> {
    match e {
        Effect::AddMana { color, .. } => Some(color.idx() as u8),
        Effect::Seq(v) => v.iter().find_map(mana_fixed_color),
        _ => None,
    }
}

// ---------------------------------------------------------------------------------------------
// Combat declarations
// ---------------------------------------------------------------------------------------------

impl Runner {
    /// Answers the engine's per-creature attack decisions from a whole declaration.
    fn do_declare_attackers(&mut self, seat: Seat, a: &J) -> R<()> {
        let mut want: Vec<(ObjRef, Seat)> = Vec::new();
        for at in a.get("attacks").and_then(|x| x.as_array()).cloned().unwrap_or_default() {
            let c = self.obj_alias(at["creature"].as_str().unwrap_or(""))?;
            let tgt = at["target"].get("player").and_then(|x| x.as_str());
            let tgt = match tgt {
                Some(p) => seat_of(p)?,
                None => unsupported!("attack target {}", at["target"]),
            };
            want.push((c, tgt));
        }
        let mut offered: Vec<ObjRef> = Vec::new();
        let mut required: Vec<ObjRef> = Vec::new();
        while let Some(p) = self.pending() {
            let creature = match p.kind {
                DecisionKind::DeclareAttacker { creature } if p.seat == seat => creature,
                _ => break,
            };
            offered.push(creature);
            if !p.options.contains(&Opt::NoAttack) {
                required.push(creature);
            }
            let hit = want.iter().position(|(c, _)| self.same_obj(creature, *c));
            match hit {
                Some(i) => {
                    let (c, tgt) = want.remove(i);
                    let o = Opt::Attack(AttackTarget::Player(tgt));
                    match p.options.iter().position(|x| *x == o) {
                        Some(k) => {
                            let (gs, ws) = (creature.slot, c.slot);
                            self.apply_opt(k)?;
                            self.swap_identity(gs, ws);
                        }
                        None => fail!("{} cannot attack {tgt:?}", self.def_name(c)),
                    }
                }
                None => match p.options.iter().position(|x| *x == Opt::NoAttack) {
                    Some(k) => self.apply_opt(k)?,
                    None => fail!("{} must attack but the script does not attack with it", self.def_name(creature)),
                },
            }
        }
        if let Some((c, _)) = want.first() {
            fail!("{} cannot attack (not offered as an attacker)", self.def_name(*c));
        }
        if let Some(eo) = a.get("expect_options") {
            self.check_subjects(eo, &offered, &required)?;
        }
        Ok(())
    }

    fn do_declare_blockers(&mut self, seat: Seat, a: &J) -> R<()> {
        let mut want: Vec<(ObjRef, ObjRef)> = Vec::new();
        for b in a.get("blocks").and_then(|x| x.as_array()).cloned().unwrap_or_default() {
            let blocker = self.obj_alias(b["blocker"].as_str().unwrap_or(""))?;
            let attacker = self.obj_alias(b["attacker"].as_str().unwrap_or(""))?;
            want.push((blocker, attacker));
        }
        let mut offered: Vec<ObjRef> = Vec::new();
        while let Some(p) = self.pending() {
            let creature = match p.kind {
                DecisionKind::DeclareBlocker { creature } if p.seat == seat => creature,
                _ => break,
            };
            offered.push(creature);
            let hit = want.iter().position(|(c, _)| self.same_obj(creature, *c));
            match hit {
                Some(i) => {
                    let (c, att) = want.remove(i);
                    let k = p.options.iter().position(|o| matches!(o, Opt::Block(x) if self.same_obj(*x, att)));
                    match k {
                        Some(k) => {
                            let (gs, ws) = (creature.slot, c.slot);
                            self.apply_opt(k)?;
                            self.swap_identity(gs, ws);
                        }
                        None => fail!("{} cannot block {}; options {}", self.def_name(c), self.def_name(att), self.describe_opts(&p)),
                    }
                }
                None => match p.options.iter().position(|x| *x == Opt::NoBlock) {
                    Some(k) => self.apply_opt(k)?,
                    None => fail!("{} must block (menace) but the script does not block with it", self.def_name(creature)),
                },
            }
        }
        if let Some((c, _)) = want.first() {
            fail!("{} cannot block (not offered as a blocker)", self.def_name(*c));
        }
        if let Some(eo) = a.get("expect_options") {
            self.check_subjects(eo, &offered, &[])?;
        }
        Ok(())
    }

    /// `include`/`exclude`/`required` of an options mask whose entries are creatures.
    fn check_subjects(&self, eo: &J, offered: &[ObjRef], required: &[ObjRef]) -> R<()> {
        for (key, must_be_in) in [("include", true), ("exclude", false)] {
            for al in eo.get(key).and_then(|x| x.as_array()).cloned().unwrap_or_default() {
                let c = self.obj_alias(al.as_str().unwrap_or(""))?;
                let found = offered.iter().any(|o| self.same_obj(*o, c));
                if found != must_be_in {
                    fail!("expect_options {key}: {} was {}offered", self.def_name(c), if found { "" } else { "not " });
                }
            }
        }
        for al in eo.get("required").and_then(|x| x.as_array()).cloned().unwrap_or_default() {
            let c = self.obj_alias(al.as_str().unwrap_or(""))?;
            if !required.iter().any(|o| self.same_obj(*o, c)) {
                fail!("expect_options required: {} is not forced to attack", self.def_name(c));
            }
        }
        Ok(())
    }

    fn do_assign_damage(&mut self, seat: Seat, a: &J) -> R<()> {
        let assignment = a.get("assignment").and_then(|x| x.as_object()).cloned().unwrap_or_default();
        let mut given: Vec<(ObjRef, u64)> = Vec::new();
        let mut to_player: Option<u64> = None;
        for (k, v) in assignment.iter() {
            if k == "player" {
                to_player = v.as_u64();
            } else {
                given.push((self.obj_alias(k)?, v.as_u64().unwrap_or(0)));
            }
        }
        let mut total_given = 0u64;
        let mut power0: Option<u64> = None;
        let mut n_asked = 0usize;
        while let Some(p) = self.pending() {
            let (blocker, remaining) = match p.kind {
                DecisionKind::AssignDamage { blocker, remaining, .. } if p.seat == seat => (blocker, remaining),
                _ => break,
            };
            let n = given.iter().find(|(c, _)| self.same_obj(blocker, *c)).map(|x| x.1);
            let n = match n {
                Some(n) => n,
                None => fail!("assignment names no damage for {}", self.def_name(blocker)),
            };
            total_given += n;
            n_asked += 1;
            power0.get_or_insert(remaining as u64);
            match p.options.iter().position(|o| *o == Opt::Number(n as u32)) {
                Some(k) => self.apply_opt(k)?,
                None => fail!("assigning {n} to {} is not legal; options {}", self.def_name(blocker), self.describe_opts(&p)),
            }
        }
        // The whole assignment is checked together (CR 510.1e): the full power is divided, and
        // without trample (the last blocker is not asked) nothing goes to the player.
        let sum: u64 = given.iter().map(|x| x.1).sum();
        let player = to_player.unwrap_or(0);
        if let Some(pw) = power0 {
            if sum + player != pw || (n_asked < given.len() && player != 0) {
                fail!("illegal damage assignment {assignment:?} (attacker power {pw})");
            }
        }
        let _ = total_given;
        Ok(())
    }
}

impl Runner {
    /// `pay: {tap: [aliases]}`: prefer those sources for the automatic payment.
    fn set_pay_hint(&mut self, a: &J) -> R<()> {
        let mut srcs = Vec::new();
        if let Some(l) = a.get("pay").and_then(|p| p.get("tap")).and_then(|t| t.as_array()) {
            for al in l {
                srcs.push(self.obj_alias(al.as_str().unwrap_or(""))?);
            }
        }
        self.g.raw_state_mut().scenario_pay_hint(srcs);
        Ok(())
    }
}

/// Steps that look at, or answer, a pending ordering decision.
fn step_keeps_orders(step: &J) -> bool {
    if step.get("check").is_some() || step.get("expect_illegal").is_some() {
        return true;
    }
    for k in ["p0", "p1"] {
        if let Some(a) = step.get(k) {
            if matches!(a.get("decision").and_then(|d| d.as_str()), Some("order_cards")) {
                return true;
            }
        }
    }
    false
}

impl Runner {
    /// Ordering decisions the script does not answer accept any order (the scenarios assert only
    /// what the rules fix): take the first option.
    fn flush_orders(&mut self) -> R<()> {
        for _ in 0..16 {
            match self.pending() {
                Some(p) if matches!(p.kind, DecisionKind::ChooseCards { purpose, .. } if purpose.is_order()) => self.apply_opt(0)?,
                _ => break,
            }
        }
        Ok(())
    }
}
