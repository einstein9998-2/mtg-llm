//! Expectation blocks (`expect` and `check`, SCHEMA.md section 4).

use crate::runner::{counter_kind, seat_of, Runner};
use crate::{fail, unsupported, Err, J, R};
use mtg_core::card::*;
use mtg_core::decision::*;
use mtg_core::ids::*;
use mtg_core::state::*;
use mtg_core::types::*;

pub fn pending_kind_name(k: &DecisionKind) -> &'static str {
    match k {
        DecisionKind::Priority => "priority",
        DecisionKind::Mulligan => "mulligan",
        DecisionKind::ChooseTarget { .. } => "choose_target",
        DecisionKind::DeclareAttacker { .. } => "declare_attackers",
        DecisionKind::DeclareBlocker { .. } => "declare_blockers",
        DecisionKind::ChooseCards { purpose: CardsPurpose::PutEach, .. } => "simultaneous_secret_choice",
        DecisionKind::ChooseCards { .. } => "choose_cards",
        DecisionKind::May | DecisionKind::PayUnless | DecisionKind::ChangeTargets => "yes_no",
        DecisionKind::OrderTriggers => "order_triggers",
        DecisionKind::ChooseMode { .. } => "choose_mode",
        DecisionKind::ChooseX | DecisionKind::ChooseReplicate | DecisionKind::PayEnergyAmount => "choose_number",
        DecisionKind::ChooseColor => "choose_color",
        DecisionKind::ChooseDungeon => "choose_dungeon",
        DecisionKind::ChooseRoom => "choose_room",
        DecisionKind::LegendRule => "legend_rule",
        DecisionKind::ChooseName { .. } | DecisionKind::ChooseType => "choose_name",
        DecisionKind::ChooseAddCost { .. } => "choose_additional_cost",
        DecisionKind::PayPhyrexian { .. } => "pay_phyrexian",
        DecisionKind::AssignDamage { .. } => "assign_combat_damage",
    }
}

fn step_name(s: Step) -> &'static str {
    match s {
        Step::Untap => "untap",
        Step::Upkeep => "upkeep",
        Step::Draw => "draw",
        Step::Main1 => "main1",
        Step::BeginCombat => "begin_combat",
        Step::DeclareAttackers => "declare_attackers",
        Step::DeclareBlockers => "declare_blockers",
        Step::FirstStrikeDamage => "first_strike_damage",
        Step::CombatDamage => "combat_damage",
        Step::EndCombat => "end_combat",
        Step::Main2 => "main2",
        Step::End => "end",
        Step::Cleanup => "cleanup",
    }
}

fn as_names(v: &J) -> Vec<String> {
    v.as_array()
        .map(|a| {
            a.iter()
                .map(|x| match x {
                    J::String(s) => s.split(" @").next().unwrap().to_string(),
                    J::Object(m) => m.get("card").or_else(|| m.get("token")).and_then(|c| c.as_str()).unwrap_or("?").to_string(),
                    o => o.to_string(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn sorted(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v
}

impl Runner {
    pub(crate) fn check_block(&self, e: &J) -> R<()> {
        let m = match e.as_object() {
            Some(m) => m,
            None => fail!("bad expectation block"),
        };
        for (k, v) in m {
            match k.as_str() {
                "at" | "note" => {}
                "game" => self.check_game(v)?,
                "turn" => {
                    let t = self.st().turn_data().turn as u64;
                    if v.as_u64() != Some(t) {
                        fail!("turn is {t}, expected {v}");
                    }
                }
                "phase" => {
                    let s = step_name(self.st().turn_data().step);
                    if v.as_str() != Some(s) {
                        fail!("phase is {s}, expected {v}");
                    }
                }
                "active" => {
                    let a = self.st().turn_data().active;
                    if seat_of(v.as_str().unwrap_or(""))? != a {
                        fail!("active player is {a:?}, expected {v}");
                    }
                }
                "priority" => {
                    let p = self.st().turn_data().priority;
                    if seat_of(v.as_str().unwrap_or(""))? != p {
                        fail!("priority is {p:?}, expected {v}");
                    }
                }
                "stack" => self.check_stack(v)?,
                "pending" => self.check_pending(v)?,
                "p0" => self.check_player(Seat(0), v)?,
                "p1" => self.check_player(Seat(1), v)?,
                "legal" => self.check_legal(v)?,
                "events_contain" => {}
                "observer" => {}
                "obs" => {
                    let seat = seat_of(m.get("observer").and_then(|x| x.as_str()).unwrap_or("p0"))?;
                    self.check_obs(seat, v)?
                }
                other => unsupported!("expect key {other}"),
            }
        }
        Ok(())
    }

    /// What `seat` can see (doc 02): counts, revealed cards and known library order, and names
    /// that must appear nowhere in its observation.
    fn check_obs(&self, seat: Seat, v: &J) -> R<()> {
        let ob = self.g.observe(seat);
        let names = |l: &[mtg_view::ViewCard]| l.iter().map(|c| c.name.clone()).collect::<Vec<_>>();
        let text = format!("{ob:?}");
        let who = format!("p{}", seat.idx());
        let m = match v.as_object() {
            Some(m) => m,
            None => return Ok(()),
        };
        for (k, x) in m {
            match k.as_str() {
                "me" | "opp" => {
                    let me = k == "me";
                    for (k2, y) in x.as_object().cloned().unwrap_or_default() {
                        let (have, want): (Vec<String>, Vec<String>) = match k2.as_str() {
                            "hand_count" | "library_count" => {
                                let n = match (me, k2.as_str()) {
                                    (true, "hand_count") => ob.me.hand.len(),
                                    (true, _) => ob.me.library_count,
                                    (false, "hand_count") => ob.opp.hand_count,
                                    (false, _) => ob.opp.library_count,
                                };
                                if y.as_u64() != Some(n as u64) {
                                    fail!("{who} sees {k} {k2} = {n}, expected {y}");
                                }
                                continue;
                            }
                            "revealed_hand" if !me => (sorted(names(&ob.opp.revealed_hand)), sorted(as_names(&y))),
                            "library_known_top" => (names(if me { &ob.me.library_known_top } else { &ob.opp.library_known_top }), as_names(&y)),
                            "library_known_bottom" => (names(if me { &ob.me.library_known_bottom } else { &ob.opp.library_known_bottom }), as_names(&y)),
                            other => unsupported!("obs key {k}.{other}"),
                        };
                        if have != want {
                            fail!("{who} sees {k} {k2} = {have:?}, expected {want:?}");
                        }
                    }
                }
                "option_labels_exclude_names" | "events_text_exclude" => {
                    for n in as_names(x) {
                        if text.contains(&format!("\"{n}\"")) {
                            fail!("{who}'s observation shows {n:?} but {k} says it must not");
                        }
                    }
                }
                other => unsupported!("obs key {other}"),
            }
        }
        Ok(())
    }

    fn check_game(&self, v: &J) -> R<()> {
        let res = self.g.result();
        let over = v.get("over").and_then(|x| x.as_bool());
        if let Some(o) = over {
            if res.is_some() != o {
                fail!("game over is {}, expected {o}", res.is_some());
            }
        }
        if let Some(w) = v.get("winner").and_then(|x| x.as_str()) {
            if res != Some(GameResult::Win(seat_of(w)?)) {
                fail!("result is {res:?}, expected winner {w}");
            }
        }
        if v.get("draw").and_then(|x| x.as_bool()) == Some(true) || v.get("result").and_then(|x| x.as_str()) == Some("draw") {
            if res != Some(GameResult::Draw) {
                fail!("result is {res:?}, expected a draw");
            }
        }
        Ok(())
    }

    fn check_pending(&self, v: &J) -> R<()> {
        let p = self.pending();
        if v.is_null() {
            if p.is_some() {
                fail!("a decision is pending but none expected");
            }
            return Ok(());
        }
        let p = match p {
            Some(p) => p,
            None => fail!("no decision pending"),
        };
        if let Some(a) = v.get("actor").and_then(|x| x.as_str()) {
            if seat_of(a)? != p.seat {
                fail!("pending actor is {:?}, expected {a}", p.seat);
            }
        }
        if let Some(k) = v.get("kind").and_then(|x| x.as_str()) {
            if pending_kind_name(&p.kind) != k {
                fail!("pending kind is {} ({:?}), expected {k}", pending_kind_name(&p.kind), p.kind);
            }
        }
        Ok(())
    }

    fn check_stack(&self, v: &J) -> R<()> {
        let want = v.as_array().cloned().unwrap_or_default();
        let have = self.st().stack().to_vec();
        if want.len() != have.len() {
            let names: Vec<String> = have.iter().map(|e| self.describe_stack_entry(e)).collect();
            fail!("stack has {} entries {:?}, expected {}", have.len(), names, want.len());
        }
        for (i, (w, h)) in want.iter().zip(have.iter()).enumerate() {
            let kind = w.get("kind").and_then(|x| x.as_str()).unwrap_or("");
            match (kind, h.kind) {
                ("spell", StackKind::Spell) => {
                    let is_copy = self.st().obj_data(&self.db, h.obj).kind == ObjKind::SpellCopy;
                    if let Some(want_copy) = w.get("copy").and_then(|x| x.as_bool()) {
                        if want_copy != is_copy {
                            fail!("stack[{i}] is {}{}, expected copy: {want_copy}", if is_copy { "a copy of " } else { "" }, self.describe_stack_entry(h));
                        }
                    } else if is_copy {
                        fail!("stack[{i}] is a copy ({}), the script lists it without copy: true", self.describe_stack_entry(h));
                    }
                    if let Some(c) = w.get("card").and_then(|x| x.as_str()) {
                        if c.starts_with('@') || c.contains(':') {
                            let r = self.obj_alias(c)?;
                            if r.slot != h.obj.slot {
                                fail!("stack[{i}] is {} not {c}", self.describe_stack_entry(h));
                            }
                        } else if self.db.def(h.def).name != c {
                            fail!("stack[{i}] is {} not {c}", self.describe_stack_entry(h));
                        }
                    }
                }
                ("triggered", StackKind::Ability { source, ability }) | ("activated", StackKind::Ability { source, ability }) => {
                    let is_trig = matches!(self.db.def(h.def).abilities.get(ability as usize), Some(AbilityDef::Triggered(_)));
                    if (kind == "triggered") != is_trig {
                        fail!("stack[{i}] is {}, expected {kind}", self.describe_stack_entry(h));
                    }
                    if let Some(t) = w.get("text_contains") {
                        if !self.ability_matches(&serde_json::json!({ "text_contains": t }), source, h.def, ability as usize)? {
                            fail!("stack[{i}] ability text {:?} does not contain {t}", self.ability_text(h.def, ability as usize));
                        }
                    }
                    if let Some(s) = w.get("source").and_then(|x| x.as_str()) {
                        let r = if s.starts_with('@') || s.contains(':') { self.obj_alias(s)?.slot } else { u16::MAX };
                        if r != u16::MAX && r != source.slot {
                            fail!("stack[{i}] source is {} not {s}", self.def_name(source));
                        }
                        if r == u16::MAX && self.db.def(h.def).name != s {
                            fail!("stack[{i}] source is {} not {s}", self.db.def(h.def).name);
                        }
                    }
                }
                _ => fail!("stack[{i}] is {}, expected kind {kind}", self.describe_stack_entry(h)),
            }
            if let Some(c) = w.get("controller").and_then(|x| x.as_str()) {
                if seat_of(c)? != h.controller {
                    fail!("stack[{i}] controller is {:?}, expected {c}", h.controller);
                }
            }
            if let Some(ts) = w.get("targets").and_then(|x| x.as_array()) {
                // An "up to" slot left empty is not a target.
                let actual: Vec<Target> = h.targets.iter().copied().filter(|t| *t != Target::None).collect();
                if ts.len() != actual.len() {
                    fail!("stack[{i}] has {} targets, expected {}", actual.len(), ts.len());
                }
                for (t, ht) in ts.iter().zip(actual.iter()) {
                    let want = self.target_of(t)?;
                    let same = match (want, *ht) {
                        (Target::Obj(a), Target::Obj(b)) => a.slot == b.slot,
                        (a, b) => a == b,
                    };
                    if !same {
                        fail!("stack[{i}] target differs: {ht:?} vs {t}");
                    }
                }
            }
            if let Some(x) = w.get("x").and_then(|x| x.as_u64()) {
                if h.x as u64 != x {
                    fail!("stack[{i}] X is {}, expected {x}", h.x);
                }
            }
        }
        Ok(())
    }

    fn describe_stack_entry(&self, e: &StackEntry) -> String {
        match e.kind {
            StackKind::Spell => format!("spell {}", self.db.def(e.def).name),
            StackKind::Ability { ability, .. } => format!("ability {}#{ability}", self.db.def(e.def).name),
        }
    }

    // ---- players ----------------------------------------------------------------------------

    fn names_in(&self, objs: &[ObjRef]) -> Vec<String> {
        objs.iter().map(|&r| self.def_name(r)).collect()
    }

    fn check_player(&self, seat: Seat, v: &J) -> R<()> {
        let st = self.st();
        let m = match v.as_object() {
            Some(m) => m,
            None => return Ok(()),
        };
        let who = format!("p{}", seat.idx());
        for (k, x) in m {
            match k.as_str() {
                "life" => {
                    if x.as_i64() != Some(st.life(seat) as i64) {
                        fail!("{who} life is {}, expected {x}", st.life(seat));
                    }
                }
                "command" => {
                    let have: Vec<(String, String)> = st
                        .dungeon_of(seat)
                        .map(|(d, room)| {
                            let def = self.db.def(d);
                            (def.name.clone(), def.dungeon.as_ref().map(|dd| dd.rooms[room as usize].name.clone()).unwrap_or_default())
                        })
                        .into_iter()
                        .collect();
                    let mut want = Vec::new();
                    let mut want_emblems: Vec<String> = Vec::new();
                    for e in x.as_array().cloned().unwrap_or_default() {
                        match (e.get("dungeon").and_then(|d| d.as_str()), e.get("room").and_then(|d| d.as_str()), e.get("emblem").and_then(|d| d.as_str())) {
                            (Some(d), Some(r), _) => want.push((d.to_string(), r.to_string())),
                            (_, _, Some(t)) => want_emblems.push(t.to_string()),
                            _ => unsupported!("command expectation {e}"),
                        }
                    }
                    let have_emblems: Vec<String> = st.emblems_of(seat).iter().map(|&d| self.db.def(d).name.clone()).collect();
                    if have_emblems.len() != want_emblems.len() || !want_emblems.iter().all(|w| have_emblems.iter().any(|h| h.contains(w.as_str()))) {
                        fail!("{who} emblems are {have_emblems:?}, expected {want_emblems:?}");
                    }
                    if have != want {
                        fail!("{who} command zone is {have:?}, expected {want:?}");
                    }
                }
                "completed_dungeons" => {
                    let have = sorted(st.completed_dungeons(seat).iter().map(|&d| self.db.def(d).name.clone()).collect());
                    let want = sorted(as_names(x));
                    if have != want {
                        fail!("{who} completed dungeons {have:?}, expected {want:?}");
                    }
                }
                "energy" => {
                    if x.as_u64() != Some(st.energy(seat) as u64) {
                        fail!("{who} energy is {}, expected {x}", st.energy(seat));
                    }
                }
                "hand" => {
                    let have = sorted(self.names_in(st.hand(seat)));
                    let want = sorted(as_names(x));
                    if have != want {
                        fail!("{who} hand is {have:?}, expected {want:?}");
                    }
                }
                "hand_count" => {
                    if x.as_u64() != Some(st.hand_count(seat) as u64) {
                        fail!("{who} hand has {} cards, expected {x}", st.hand_count(seat));
                    }
                }
                "library_count" => {
                    if x.as_u64() != Some(st.library_count(seat) as u64) {
                        fail!("{who} library has {} cards, expected {x}", st.library_count(seat));
                    }
                }
                "library_top" => {
                    let want = as_names(x);
                    let have: Vec<String> = st.library(seat).iter().rev().take(want.len()).map(|&r| self.def_name(r)).collect();
                    if have != want {
                        fail!("{who} library top is {have:?}, expected {want:?}");
                    }
                }
                "library_bottom" => {
                    let want = as_names(x);
                    let lib = st.library(seat);
                    let have: Vec<String> = lib.iter().take(want.len()).rev().map(|&r| self.def_name(r)).collect();
                    if have != want {
                        fail!("{who} library bottom is {have:?}, expected {want:?}");
                    }
                }
                "library_bottom_multiset" => {
                    let want = sorted(as_names(x));
                    let lib = st.library(seat);
                    let have = sorted(lib.iter().take(want.len()).map(|&r| self.def_name(r)).collect());
                    if have != want {
                        fail!("{who} library bottom multiset is {have:?}, expected {want:?}");
                    }
                }
                "graveyard" => {
                    let have = sorted(self.names_in(st.graveyard(seat)));
                    let want = sorted(as_names(x));
                    if have != want {
                        fail!("{who} graveyard is {have:?}, expected {want:?}");
                    }
                }
                "graveyard_ordered" => {
                    let have = self.names_in(st.graveyard(seat));
                    let want = as_names(x);
                    if have != want {
                        fail!("{who} graveyard (bottom to top) is {have:?}, expected {want:?}");
                    }
                }
                "exile" => {
                    let mine: Vec<ObjRef> = st.exile().iter().copied().filter(|&r| st.obj_data(&self.db, r).owner == seat).collect();
                    let have = sorted(self.names_in(&mine));
                    let want = sorted(as_names(x));
                    if have != want {
                        fail!("{who} exile is {have:?}, expected {want:?}");
                    }
                }
                "battlefield" => self.check_battlefield(seat, x, true)?,
                "battlefield_contains" => self.check_battlefield(seat, x, false)?,
                "mana_pool" => {
                    let pool = st.pool(seat);
                    let mut want = [0u8; 6];
                    if let Some(mm) = x.as_object() {
                        for (c, n) in mm {
                            let i = ["W", "U", "B", "R", "G", "C"].iter().position(|z| z == c).unwrap_or(5);
                            want[i] = n.as_u64().unwrap_or(0) as u8;
                        }
                    }
                    if pool != want {
                        fail!("{who} mana pool is {pool:?}, expected {want:?}");
                    }
                }
                "lands_played" => {
                    if x.as_u64() != Some(st.lands_played(seat) as u64) {
                        fail!("{who} lands played is {}, expected {x}", st.lands_played(seat));
                    }
                }
                "spells_cast_this_turn" => {
                    if x.as_u64() != Some(st.spells_cast(seat) as u64) {
                        fail!("{who} spells cast this turn is {}, expected {x}", st.spells_cast(seat));
                    }
                }
                "designations" => {
                    if x.as_array().map(|a| a.is_empty()).unwrap_or(true) {
                        continue;
                    }
                    unsupported!("{k}");
                }
                "known_to_me" => unsupported!("known_to_me"),
                other => unsupported!("player expect key {other}"),
            }
        }
        Ok(())
    }

    fn check_battlefield(&self, seat: Seat, descs: &J, exact: bool) -> R<()> {
        let st = self.st();
        let mine: Vec<ObjRef> = st.battlefield().iter().copied().filter(|&r| st.obj_data(&self.db, r).controller == seat).collect();
        let want: Vec<J> = descs.as_array().cloned().unwrap_or_default();
        // Assign descriptors to permanents (bipartite matching by backtracking; lists are tiny).
        let n = want.len();
        let mut ok = vec![vec![false; mine.len()]; n];
        for (i, d) in want.iter().enumerate() {
            for (j, &r) in mine.iter().enumerate() {
                ok[i][j] = self.perm_matches(d, r)?;
            }
        }
        let mut used = vec![false; mine.len()];
        fn assign(i: usize, n: usize, ok: &[Vec<bool>], used: &mut Vec<bool>) -> bool {
            if i == n {
                return true;
            }
            for j in 0..used.len() {
                if ok[i][j] && !used[j] {
                    used[j] = true;
                    if assign(i + 1, n, ok, used) {
                        return true;
                    }
                    used[j] = false;
                }
            }
            false
        }
        if !assign(0, n, &ok, &mut used) {
            let have: Vec<String> = mine.iter().map(|&r| self.describe_perm(r)).collect();
            fail!("p{} battlefield {have:?} does not match {descs}", seat.idx());
        }
        if exact && mine.len() != n {
            let have: Vec<String> = mine.iter().map(|&r| self.describe_perm(r)).collect();
            fail!("p{} controls {} permanents {have:?}, expected exactly {n}", seat.idx(), mine.len());
        }
        Ok(())
    }

    fn describe_perm(&self, r: ObjRef) -> String {
        let d = self.st().obj_data(&self.db, r);
        let mut s = self.db.def(d.def).name.clone();
        if d.tapped {
            s.push_str(" (tapped)");
        }
        if d.chars.types.contains(Types::CREATURE) {
            s.push_str(&format!(" {}/{}", d.chars.power, d.chars.toughness));
        }
        s
    }

    fn perm_matches(&self, d: &J, r: ObjRef) -> R<bool> {
        let od = self.st().obj_data(&self.db, r);
        let def = self.db.def(od.def);
        let m = match d {
            J::String(s) => return Ok(def.name == s.split(" @").next().unwrap()),
            J::Object(m) => m,
            _ => return Ok(false),
        };
        for (k, v) in m {
            let ok = match k.as_str() {
                "card" => v.as_str() == Some(def.name.as_str()),
                "token" => v.as_str() == Some(def.name.as_str()) || v.as_bool() == Some(od.kind == ObjKind::Token),
                "controller" => seat_of(v.as_str().unwrap_or(""))? == od.controller,
                "tapped" => v.as_bool() == Some(od.tapped),
                "sick" => v.as_bool() == Some(od.summoning_sick),
                "damage" => v.as_u64() == Some(od.damage as u64),
                "counters" => {
                    let mut all = true;
                    if let Some(c) = v.as_object() {
                        for (kn, n) in c {
                            let kind = counter_kind(kn)?;
                            let have = od.counters.iter().filter(|(k2, _)| *k2 == kind).map(|(_, n)| *n as u64).sum::<u64>();
                            if Some(have) != n.as_u64() {
                                all = false;
                            }
                        }
                    }
                    all
                }
                "pt" => {
                    let a = v.as_array().cloned().unwrap_or_default();
                    a.len() == 2 && a[0].as_i64() == Some(od.chars.power as i64) && a[1].as_i64() == Some(od.chars.toughness as i64)
                }
                "types" => {
                    let mut want = Types::empty();
                    for t in v.as_array().cloned().unwrap_or_default() {
                        match Types::from_name(&t.as_str().unwrap_or("").to_uppercase()) {
                            Some(t) => want |= t,
                            None => return Ok(false),
                        }
                    }
                    od.chars.types == want
                }
                "supertypes" => v.as_array().cloned().unwrap_or_default().iter().all(|t| Supertypes::from_name(&t.as_str().unwrap_or("").to_uppercase()).map(|t| od.chars.supertypes.contains(t)).unwrap_or(false)),
                "colors" => v.as_array().cloned().unwrap_or_default().iter().all(|t| Colors::from_name(t.as_str().unwrap_or("")).map(|t| od.chars.colors.contains(t)).unwrap_or(false)),
                "attacking" => self.st().combat().map(|c| c.attackers.iter().any(|a| a.obj == r)).unwrap_or(false) == v.as_bool().unwrap_or(true),
                "keywords" => v.as_array().cloned().unwrap_or_default().iter().all(|t| Keywords::from_name(&t.as_str().unwrap_or("").to_uppercase().replace(' ', "_")).map(|t| od.chars.keywords.contains(t)).unwrap_or(false)),
                "keywords_exclude" => v.as_array().cloned().unwrap_or_default().iter().all(|t| Keywords::from_name(&t.as_str().unwrap_or("").to_uppercase().replace(' ', "_")).map(|t| !od.chars.keywords.contains(t)).unwrap_or(true)),
                "subtypes" => v.as_array().cloned().unwrap_or_default().iter().all(|t| subtype_index(t.as_str().unwrap_or("")).map(|i| od.chars.subtypes.has(i)).unwrap_or(false)),
                "attached_to" => match v {
                    J::Null => od.attached_to.is_none(),
                    J::String(s) => match od.attached_to {
                        Some(a) => a.slot == self.obj_alias(s)?.slot,
                        None => false,
                    },
                    _ => false,
                },
                "alias" | "as" => match v.as_str() {
                    Some(s) => self.obj_alias(&format!("@{}", s.trim_start_matches('@')))?.slot == r.slot,
                    None => false,
                },
                "face_up" => true,
                other => unsupported!("permanent descriptor key {other}"),
            };
            if !ok {
                return Ok(false);
            }
        }
        Ok(true)
    }

    // ---- legal-action patterns -------------------------------------------------------------

    fn check_legal(&self, v: &J) -> R<()> {
        for (k, x) in v.as_object().cloned().unwrap_or_default() {
            let seat = seat_of(&k)?;
            let p = match self.pending() {
                Some(p) if p.seat == seat => p,
                Some(p) => fail!("legal check for {k} but {:?} must act", p.seat),
                None => fail!("legal check with no pending decision"),
            };
            for (key, want) in [("include", true), ("exclude", false)] {
                for pat in x.get(key).and_then(|y| y.as_array()).cloned().unwrap_or_default() {
                    let found = self.pattern_offered(&p, &pat)?;
                    if found != want {
                        fail!("legal {k}: pattern {pat} is {} but the scenario says {key}", if found { "offered" } else { "not offered" });
                    }
                }
            }
        }
        Ok(())
    }

    fn pattern_offered(&self, p: &Pending, pat: &J) -> R<bool> {
        let t = pat.get("t").and_then(|x| x.as_str()).unwrap_or("");
        Ok(match t {
            "pass" => p.options.contains(&Opt::Pass),
            "play_land" => {
                let want = self.obj_alias(pat["card"].as_str().unwrap_or(""))?;
                p.options.iter().any(|o| matches!(o, Opt::PlayLand(r) if self.same_obj(*r, want)))
            }
            "cast" if pat.as_object().map(|m| m.keys().any(|k| matches!(k.as_str(), "modes" | "targets" | "x" | "pay" | "phyrexian" | "delve" | "replicate")) || m.get("alt_cost").map(|a| a.get("exile_blue_card").is_some()).unwrap_or(false)).unwrap_or(false) => {
                // A pattern that names choices: try the whole action on a copy of the game.
                let mut sim = Runner { db: self.db.clone(), g: self.g.clone(), aliases: self.aliases.clone(), handles: self.handles.clone(), step_no: self.step_no, last_events: vec![], lenient: true };
                match sim.run_seat_step_pub(p.seat, pat) {
                    Ok(()) => true,
                    Err(Err::Fail(_)) => false,
                    Err(Err::Unsupported(m)) => return Err(Err::Unsupported(m)),
                }
            }
            "cast" => {
                let want = self.obj_alias(pat["card"].as_str().unwrap_or(""))?;
                let way: Option<u8> = match pat.get("alt_cost") {
                    None => None,
                    Some(J::Null) => Some(0),
                    Some(_) => Some(self.alt_way_for(want, pat)?),
                };
                p.options.iter().any(|o| matches!(o, Opt::Cast(r, w) if self.same_obj(*r, want) && way.map(|x| x == *w).unwrap_or(true)))
            }
            "activate" | "activate_from_hand" => {
                let want = self.obj_alias(pat["source"].as_str().unwrap_or(""))?;
                p.options.iter().any(|o| matches!(o, Opt::Activate { src, .. } | Opt::Mana { src, .. } if self.same_obj(*src, want)))
            }
            other => unsupported!("legal pattern {other}"),
        })
    }
}

#[allow(dead_code)]
fn _e(_: Err) {}
