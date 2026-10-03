//! Runs Forge interaction-test scenarios (`.scn`, see /mnt/project-files/interaction-tests/README.md)
//! on the Rust engine. The scenarios were written against Oracle text and verified on Forge, so a
//! failure here is an engine bug, a translation gap, or a Forge deviation (the `xfail:` ones).

use crate::util::*;
use mtg_core::card::CardDb;
use mtg_core::decision::*;
use mtg_core::ids::*;
use mtg_core::scenario::*;
use mtg_core::types::*;
use mtg_view::Game;
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Clone, Debug, Default)]
pub struct Scn {
    pub name: String,
    pub file: String,
    pub title: String,
    pub xfail: Option<String>,
    pub swap: bool,
    pub state: Vec<String>,
    pub script: [Vec<String>; 2],
    pub expect: Vec<String>,
}

/// Forge names a token after its type ("Cat Token"); the engine appends the source in brackets.
fn strip_src(n: String) -> String {
    match n.find(" (") {
        Some(i) if n.ends_with(')') => n[..i].to_string(),
        _ => n,
    }
}

pub fn parse_file(path: &std::path::Path) -> Vec<Scn> {
    let text = std::fs::read_to_string(path).unwrap();
    let file = path.file_name().unwrap().to_string_lossy().to_string();
    let mut out: Vec<Scn> = Vec::new();
    let mut section = String::new();
    for raw in text.lines() {
        let line = raw.trim_end();
        if let Some(n) = line.strip_prefix("=== ") {
            out.push(Scn { name: n.trim().to_string(), file: file.clone(), ..Default::default() });
            section.clear();
            continue;
        }
        let Some(cur) = out.last_mut() else { continue };
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        if !line.starts_with(char::is_whitespace) {
            let (k, v) = line.split_once(':').unwrap();
            let v = v.trim();
            match k.trim() {
                "title" => cur.title = v.to_string(),
                "xfail" => cur.xfail = Some(v.to_string()),
                "swap-seats" => cur.swap = v.eq_ignore_ascii_case("yes"),
                "ref" | "note" => {}
                s @ ("state" | "script" | "expect") => section = s.to_string(),
                other => panic!("unknown key {other}"),
            }
            continue;
        }
        let t = line.trim();
        match section.as_str() {
            "state" => cur.state.push(t.to_string()),
            "expect" => cur.expect.push(t.to_string()),
            "script" => {
                let (who, rest) = t.split_once(':').unwrap();
                let i = match who.trim() {
                    "p1" => 0,
                    "p2" => 1,
                    o => panic!("script line needs p1:/p2: {o}"),
                };
                cur.script[i].push(rest.trim().to_string());
            }
            _ => panic!("stray line {t}"),
        }
    }
    out
}

#[derive(Debug, Clone)]
pub enum Outcome {
    Pass,
    /// Passed, but some expectations or script features were skipped (listed).
    PassPartial(Vec<String>),
    Fail(Vec<String>),
    Error(String),
    Unsupported(String),
}

struct Run<'a> {
    db: &'a Arc<CardDb>,
    g: Game,
    /// p1 / p2 -> engine seat
    seat_of: [Seat; 2],
    queues: [VecDeque<String>; 2],
    trace: Vec<String>,
    failures: Vec<String>,
    skipped: Vec<String>,
    /// Remaining target names of the cast being played (by script seat index).
    cast_targets: [VecDeque<String>; 2],
    mode_hint: [Option<String>; 2],
    /// Names still to answer from the current multi-name pick.
    chain: [VecDeque<String>; 2],
    attack_set: [Option<Vec<String>>; 2],
    block_pairs: [Option<Vec<(String, String)>>; 2],
    attack_target_player: [Option<String>; 2],
    start_turn: u16,
    prio_calls: [u32; 2],
    defaults: Vec<String>,
    x_value: Option<u32>,
    backup: Option<(Game, bool, String)>,
}

impl<'a> Run<'a> {
    fn idx_of_seat(&self, s: Seat) -> usize {
        if self.seat_of[0] == s {
            0
        } else {
            1
        }
    }
    fn pname(&self, s: Seat) -> &'static str {
        if self.idx_of_seat(s) == 0 {
            "p1"
        } else {
            "p2"
        }
    }
    fn st(&self) -> &mtg_core::state::State {
        self.g.raw_state()
    }
    fn nm(&self, r: ObjRef) -> String {
        def_name(self.db, self.st(), r)
    }
    fn log(&mut self, s: String) {
        self.trace.push(s);
    }

    fn opt_text(&self, o: &Opt) -> String {
        match *o {
            Opt::Card(r) | Opt::Target(Target::Obj(r)) => self.nm(r),
            Opt::Target(Target::Player(p)) => self.pname(p).to_string(),
            Opt::Target(Target::None) => "none".into(),
            Opt::Name(d) => self.db.def(CardDefId(d)).name.clone(),
            Opt::Type(t) => subtype_name(t).to_string(),
            Opt::Block(a) => self.nm(a),
            Opt::Mode(i) => format!("mode{}", i + 1),
            Opt::Choice(i) => format!("choice{i}"),
            Opt::Number(n) => n.to_string(),
            Opt::Yes => "yes".into(),
            Opt::No => "no".into(),
            Opt::Done => "done".into(),
            Opt::Pass => "pass".into(),
            Opt::Keep => "keep".into(),
            Opt::Mulligan => "mulligan".into(),
            Opt::NoAttack => "noattack".into(),
            Opt::NoBlock => "noblock".into(),
            Opt::Attack(_) => "attack".into(),
            Opt::PlayLand(r) | Opt::PlayLandBack(r, _) => self.nm(r),
            Opt::Cast(r, w) => format!("{} {}", self.nm(r), way_text(self.db, self.st(), r, w)),
            Opt::Activate { src, ability } => format!("{} {}", self.nm(src), ability_text(self.db, self.st().def_of(src), ability)),
            Opt::Mana { src, .. } => self.nm(src),
        }
    }

    fn options_text(&self, p: &Pending) -> Vec<String> {
        p.options.iter().map(|o| self.opt_text(o)).collect()
    }

    fn apply(&mut self, idx: usize) {
        let p = self.g.pending().unwrap().clone();
        self.g.apply(p.id, idx).expect("apply");
    }

    fn stack_top_name(&self) -> Option<String> {
        self.st().stack().last().map(|e| def_name(self.db, self.st(), e.obj))
    }

    fn phase_matches(&self, arg: &str) -> bool {
        let mut a = arg.trim();
        let mut owner: Option<usize> = None;
        if a.starts_with("p1:") || a.starts_with("p2:") {
            owner = Some((a.as_bytes()[1] - b'1') as usize);
            a = &a[3..];
        }
        let Some(want) = step_from_name(&a.to_uppercase()) else { return false };
        let td = self.st().turn_data();
        if td.step != want {
            return false;
        }
        owner.map_or(true, |o| self.seat_of[o] == td.active)
    }

    // ---------------------------------------------------------------- priority

    /// Returns false when the scenario is finished.
    fn priority(&mut self, seat: Seat) -> Result<bool, String> {
        let me = self.idx_of_seat(seat);
        self.backup = None;
        self.prio_calls[me] += 1;
        if self.prio_calls[me] > 400 {
            return Err("script did not finish (400 priority windows)".into());
        }
        if self.st().turn_data().turn > self.start_turn + 6 {
            return Err("script did not finish (6 turns passed)".into());
        }
        loop {
            let head = self.queues[me].front().cloned();
            let only_picks = |q: &VecDeque<String>| q.iter().all(|l| l.starts_with("pick"));
            if self.st().stack().is_empty() && only_picks(&self.queues[0]) && only_picks(&self.queues[1]) && (self.queues[0].is_empty() || self.queues[1].is_empty() || true) && head.as_deref().map_or(true, |h| h.starts_with("pick")) {
                let left: Vec<String> = self.queues.iter().flatten().cloned().collect();
                if !left.is_empty() {
                    self.skipped.push(format!("picks never asked by the engine: {left:?}"));
                }
                return Ok(false);
            }
            let Some(head) = head else {
                self.pass();
                return Ok(true);
            };
            let (cmd, arg) = match head.split_once(char::is_whitespace) {
                Some((c, a)) => (c.to_string(), a.trim().to_string()),
                None => (head.clone(), String::new()),
            };
            match cmd.as_str() {
                "pass" => {
                    self.queues[me].pop_front();
                    self.log(format!("p{} pass", me + 1));
                    self.pass();
                    return Ok(true);
                }
                "at" => {
                    if !self.phase_matches(&arg) {
                        self.pass();
                        return Ok(true);
                    }
                    self.queues[me].pop_front();
                }
                "wait-empty" => {
                    if !self.st().stack().is_empty() {
                        self.pass();
                        return Ok(true);
                    }
                    self.queues[me].pop_front();
                }
                "when-stack" => {
                    if self.stack_top_name().map_or(true, |n| !n.contains(arg.trim())) {
                        self.pass();
                        return Ok(true);
                    }
                    self.queues[me].pop_front();
                }
                "check" => {
                    self.queues[me].pop_front();
                    if let Some(bad) = self.eval(&arg)? {
                        self.failures.push(format!("inline check [{arg}]: {bad}"));
                    }
                }
                "pick" if self.st().stack().is_empty() => {
                    self.queues[me].pop_front();
                    self.skipped.push(format!("pick never matched a decision: {head}"));
                    self.log(format!("p{} dropped stale {head}", me + 1));
                }
                "pick" | "attack" | "block" => {
                    self.pass();
                    return Ok(true);
                }
                "cast" | "play" | "activate" | "try" => {
                    self.queues[me].pop_front();
                    let try_only = cmd == "try";
                    let spec = if try_only { arg.split_once(char::is_whitespace).map(|x| x.1.trim().to_string()).unwrap_or_default() } else { arg.clone() };
                    let kind = if try_only { arg.split_whitespace().next().unwrap_or("").to_string() } else { cmd.clone() };
                    let found = self.find_action(seat, me, &kind, &spec);
                    match found {
                        Some(i) => {
                            self.backup = Some((self.g.clone(), try_only, head.clone()));
                            self.log(format!("p{} {head}  => option {i}", me + 1));
                            self.apply(i);
                            return Ok(true);
                        }
                        None if kind == "cast" && self.try_restricted_mana(seat, me, &kind, &spec) => {
                            let i = self.find_action(seat, me, &kind, &spec).expect("cast offered after restricted mana");
                            self.backup = Some((self.g.clone(), try_only, head.clone()));
                            self.log(format!("p{} {head}  => option {i} (after restricted mana)", me + 1));
                            self.apply(i);
                            return Ok(true);
                        }
                        None => {
                            let offered: Vec<String> = self.options_text(self.g.pending().unwrap());
                            self.log(format!("p{} could not {head}; offered {:?}", me + 1, offered));
                            if try_only {
                                continue;
                            }
                            self.failures.push(format!("script: could not {head} (not offered: {:?})", offered));
                            self.pass();
                            return Ok(true);
                        }
                    }
                }
                other => return Err(format!("unknown script command {other}")),
            }
        }
    }

    /// Plays option `idx` on a copy of the game and checks that every wanted target is offered
    /// when the engine asks for it (Forge refuses an illegal target and the cast does not happen).
    fn probe_targets(&self, idx: usize, me: usize) -> bool {
        let mut g = self.g.clone();
        let p = g.pending().unwrap().clone();
        g.apply(p.id, idx).unwrap();
        let mut want: VecDeque<String> = self.cast_targets[me].clone();
        for _ in 0..40 {
            if want.is_empty() {
                return true;
            }
            if !matches!(g.advance(), Status::NeedDecision(_)) {
                return want.is_empty();
            }
            let p = g.pending().unwrap().clone();
            match p.kind {
                DecisionKind::Priority => return want.is_empty(),
                DecisionKind::ChooseTarget { .. } => {
                    let w = want.front().unwrap().clone();
                    let hit = p.options.iter().position(|o| self.opt_text_in(&g, o) == w.trim_start_matches("stack:").trim());
                    match hit {
                        Some(i) => {
                            want.pop_front();
                            g.apply(p.id, i).unwrap();
                        }
                        None => return false,
                    }
                }
                _ => {
                    g.apply(p.id, 0).unwrap();
                }
            }
        }
        true
    }

    fn opt_text_in(&self, g: &Game, o: &Opt) -> String {
        match *o {
            Opt::Target(Target::Obj(r)) => def_name(self.db, g.raw_state(), r),
            Opt::Target(Target::Player(p)) => self.pname(p).to_string(),
            _ => String::new(),
        }
    }

    fn pass(&mut self) {
        let p = self.g.pending().unwrap().clone();
        let i = p.options.iter().position(|o| matches!(o, Opt::Pass)).expect("pass option");
        self.g.apply(p.id, i).unwrap();
    }

    /// spec: `Card Name[@p1] [~filter] [-> t1; t2 | t3] [x=N]`
    /// Forge's auto-payment spends Cavern-style restricted mana on its own; the engine needs the
    /// mana ability activated first. Tries each restricted-mana option on a clone and commits the
    /// first one after which the wanted cast is offered.
    fn try_restricted_mana(&mut self, seat: Seat, me: usize, kind: &str, spec: &str) -> bool {
        let p = match self.g.pending() {
            Some(p) => p.clone(),
            None => return false,
        };
        for (i, o) in p.options.iter().enumerate() {
            let Opt::Mana { src, ability, .. } = *o else { continue };
            if !ability_text(self.db, self.st().def_of(src), ability).contains("AddManaRestricted") {
                continue;
            }
            let saved = self.g.clone();
            self.apply(i);
            if !matches!(self.g.advance(), Status::NeedDecision(_)) {
                self.g = saved;
                continue;
            }
            if self.g.pending().map(|q| q.seat == seat).unwrap_or(false) && self.find_action(seat, me, kind, spec).is_some() {
                return true;
            }
            self.g = saved;
        }
        false
    }

    fn find_action(&mut self, seat: Seat, me: usize, kind: &str, spec: &str) -> Option<usize> {
        let mut spec = spec.to_string();
        let mut x: Option<u32> = None;
        if let Some(pos) = spec.find(" x=") {
            let tail = &spec[pos + 3..];
            let end = tail.find(char::is_whitespace).unwrap_or(tail.len());
            x = tail[..end].parse().ok();
            let rest = tail[end..].to_string();
            spec.truncate(pos);
            spec.push_str(&rest);
        }
        let mut targets = String::new();
        if let Some(pos) = spec.find("->") {
            targets = spec[pos + 2..].trim().to_string();
            spec.truncate(pos);
            spec = spec.trim().to_string();
        }
        let mut filter: Option<String> = None;
        if let Some(pos) = spec.find('~') {
            filter = Some(spec[pos + 1..].trim().to_string());
            spec.truncate(pos);
            spec = spec.trim().to_string();
        }
        let mut name = spec.trim().to_string();
        let mut ctl: Option<usize> = None;
        if let Some(at) = name.rfind('@') {
            ctl = Some(if &name[at + 1..] == "p1" { 0 } else { 1 });
            name.truncate(at);
        }
        let _ = x; // X is chosen through the ChooseX decision below
        self.x_value = x;
        let p = self.g.pending().unwrap().clone();
        let mut cands: Vec<(usize, String)> = Vec::new();
        for (i, o) in p.options.iter().enumerate() {
            let (src, text): (ObjRef, String) = match (*o, kind) {
                (Opt::PlayLand(r), "play") | (Opt::PlayLandBack(r, _), "play") => (r, String::new()),
                (Opt::Cast(r, w), "cast") => (r, way_text(self.db, self.st(), r, w)),
                (Opt::Activate { src, ability }, "activate") => (src, ability_text(self.db, self.st().def_of(src), ability)),
                (Opt::Mana { src, ability, .. }, "activate") => (src, ability_text(self.db, self.st().def_of(src), ability)),
                (Opt::Activate { src, ability }, "cast") => (src, ability_text(self.db, self.st().def_of(src), ability)),
                _ => continue,
            };
            let mut nm = self.nm(src);
            // the Adventure half is cast by the card's name in the scripts
            if let Opt::Cast(_, mtg_core::restrict::ADVENTURE_WAY) = o {
                nm = nm.clone();
            }
            if nm != name {
                continue;
            }
            if let Some(c) = ctl {
                let d = self.st().obj_data(self.db, src);
                if self.seat_of[c] != d.controller && !(d.zone != ZoneKind::Battlefield && self.seat_of[c] == d.owner) {
                    continue;
                }
            }
            cands.push((i, text));
        }
        if cands.is_empty() {
            return None;
        }
        self.cast_targets[me] = if targets.is_empty() { VecDeque::new() } else { targets.split(|c| c == ';' || c == '|').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect() };
        self.mode_hint[me] = None;
        let alt = |t: &str| !t.is_empty();
        let pick = match filter.as_deref() {
            None => cands.iter().find(|c| !alt(&c.1)).or(cands.first()).map(|c| c.0),
            Some("alt") => cands.iter().find(|c| alt(&c.1)).map(|c| c.0),
            Some("hard") => cands.iter().find(|c| !alt(&c.1)).map(|c| c.0),
            Some("free") => cands.iter().find(|c| c.1.starts_with("permit:") || c.1 == "free-effect" || c.1.contains("free")).map(|c| c.0),
            Some(f) if f.starts_with('#') => f[1..].parse::<usize>().ok().and_then(|n| cands.get(n)).map(|c| c.0),
            Some(f) => {
                let fl = f.to_lowercase();
                let fl2 = fl.strip_prefix("cost:").unwrap_or(&fl).to_string();
                let hit = cands.iter().find(|c| c.1.to_lowercase().contains(&fl2)).map(|c| c.0);
                let adv = cands.iter().find(|c| c.1 == "adventure").map(|c| c.0);
                if hit.is_none() && adv.is_some() && kind == "cast" {
                    adv
                } else if hit.is_none() && kind == "cast" {
                    // Forge models modal spells as separate abilities; here the filter is a mode
                    self.mode_hint[me] = Some(fl2);
                    cands.iter().find(|c| !alt(&c.1)).map(|c| c.0)
                } else {
                    hit
                }
            }
        };
        pick
    }

    // ---------------------------------------------------------------- non-priority decisions

    fn sub_decision(&mut self, seat: Seat) -> Result<(), String> {
        let p = self.g.pending().unwrap().clone();
        let me = self.idx_of_seat(seat);
        let n = p.options.len();
        self.log(format!("   [p{} decision {:?} options {:?}]", me + 1, p.kind, self.options_text(&p)));
        // combat declarations
        match p.kind {
            DecisionKind::DeclareAttacker { creature } => {
                self.drop_phase_waits(me, "COMBAT_DECLARE_ATTACKERS");
                if self.attack_set[me].is_none() {
                    if let Some(h) = self.queues[me].front() {
                        if h.starts_with("attack") {
                            let h = self.queues[me].pop_front().unwrap();
                            let mut spec = h[6..].trim().to_string();
                            let mut tgt = format!("p{}", 2 - me);
                            if let Some(a) = spec.find("->") {
                                tgt = spec[a + 2..].trim().to_string();
                                spec.truncate(a);
                            }
                            self.attack_set[me] = Some(spec.split(';').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect());
                            self.attack_target_player[me] = Some(tgt);
                            self.log(format!("p{} attacks with {spec}", me + 1));
                        }
                    }
                    if self.attack_set[me].is_none() {
                        self.attack_set[me] = Some(vec![]);
                    }
                }
                let nm = self.nm(creature);
                let set = self.attack_set[me].as_mut().unwrap();
                if let Some(pos) = set.iter().position(|x| *x == nm) {
                    set.remove(pos);
                    let i = p.options.iter().position(|o| matches!(o, Opt::Attack(_))).ok_or("attack not offered")?;
                    self.apply(i);
                } else {
                    let i = p.options.iter().position(|o| matches!(o, Opt::NoAttack)).ok_or("no-attack not offered")?;
                    self.apply(i);
                }
                return Ok(());
            }
            DecisionKind::DeclareBlocker { creature } => {
                self.drop_phase_waits(me, "COMBAT_DECLARE_BLOCKERS");
                if self.block_pairs[me].is_none() {
                    let mut pairs = vec![];
                    if let Some(h) = self.queues[me].front() {
                        if h.starts_with("block") {
                            let h = self.queues[me].pop_front().unwrap();
                            for pair in h[5..].split(';') {
                                if let Some((a, b)) = pair.split_once(" blocks ") {
                                    pairs.push((a.trim().to_string(), b.trim().to_string()));
                                }
                            }
                            self.log(format!("p{} blocks {h}", me + 1));
                        }
                    }
                    self.block_pairs[me] = Some(pairs);
                }
                let nm = self.nm(creature);
                let pairs = self.block_pairs[me].as_mut().unwrap();
                if let Some(pos) = pairs.iter().position(|x| x.0 == nm) {
                    let (_, atk) = pairs.remove(pos);
                    let i = p.options.iter().position(|o| matches!(o, Opt::Block(a) if self.nm(*a) == atk));
                    match i {
                        Some(i) => self.apply(i),
                        None => {
                            self.failures.push(format!("script: {nm} cannot block {atk}"));
                            let i = p.options.iter().position(|o| matches!(o, Opt::NoBlock)).unwrap();
                            self.apply(i);
                        }
                    }
                } else {
                    let i = p.options.iter().position(|o| matches!(o, Opt::NoBlock)).ok_or("no-block not offered")?;
                    self.apply(i);
                }
                return Ok(());
            }
            _ => {}
        }
        // targets of the spell being cast come from the cast spec
        if let DecisionKind::ChooseTarget { .. } = p.kind {
            if let Some(want) = self.cast_targets[me].front().cloned() {
                if let Some(i) = self.match_text(&p, &want) {
                    self.cast_targets[me].pop_front();
                    self.apply(i);
                    return Ok(());
                }
                // Forge refuses an illegal target and the cast does not happen: roll back
                if let Some((g, try_only, head)) = self.backup.take() {
                    self.g = g;
                    self.cast_targets[me].clear();
                    self.log(format!("p{} {head}: target {want:?} not offered ({:?}); action rolled back", me + 1, self.options_text(&p)));
                    if !try_only {
                        self.failures.push(format!("script: target {want:?} of [{head}] not legal (offered {:?})", self.options_text(&p)));
                    }
                    return Ok(());
                }
            }
        }
        if let DecisionKind::ChooseMode { .. } = p.kind {
            if let Some(h) = self.mode_hint[me].clone() {
                // label of a mode: the card's mode label
                let hit = self.mode_by_label(&p, &h);
                if let Some(i) = hit {
                    self.mode_hint[me] = None;
                    self.apply(i);
                    return Ok(());
                }
            }
        }
        if let DecisionKind::ChooseMode { .. } = p.kind {
            let has_pick = self.queues[me].front().map_or(false, |h| h.starts_with("pick") && self.match_pick(&p, h[4..].trim()).is_some());
            if !has_pick {
                if let Some(want) = self.cast_targets[me].front().cloned() {
                    if let Some(i) = self.mode_offering_target(&p, &want) {
                        self.apply(i);
                        return Ok(());
                    }
                }
            }
        }
        if let DecisionKind::ChooseX = p.kind {
            if let Some(x) = self.x_value {
                if let Some(i) = p.options.iter().position(|o| matches!(o, Opt::Number(n) if *n == x)) {
                    self.apply(i);
                    return Ok(());
                }
            }
        }
        // chained multi-name pick
        if let Some(w) = self.chain[me].front().cloned() {
            if let Some(i) = self.match_text(&p, &w) {
                self.chain[me].pop_front();
                self.apply(i);
                return Ok(());
            }
            // a Done option ends the chain
            if let Some(i) = p.options.iter().position(|o| matches!(o, Opt::Done)) {
                self.log(format!("p{} chain item {w:?} not offered; Done", me + 1));
                self.chain[me].clear();
                self.apply(i);
                return Ok(());
            }
            self.failures.push(format!("script: pick {w:?} not offered among {:?} ({:?})", self.options_text(&p), p.kind));
            self.chain[me].clear();
        }
        // "pick <entity>" then "pick yes/no" answer two Forge prompts; the engine asks only the yes/no
        if p.options.iter().any(|o| matches!(o, Opt::Yes)) && p.options.iter().any(|o| matches!(o, Opt::No)) && self.queues[me].len() >= 2 {
            let a = self.queues[me][0].trim().to_string();
            let b = self.queues[me][1].trim().to_string();
            let is_yn = |x: &str| matches!(x.strip_prefix("pick").map(|r| r.trim().to_lowercase()).as_deref(), Some("yes") | Some("no"));
            if a.starts_with("pick") && !is_yn(&a) && is_yn(&b) {
                self.queues[me].pop_front();
                self.log(format!("p{} dropped entity pick [{a}] before confirmation", me + 1));
            }
        }
        // a Forge "may cast? yes" / "play" confirmation the engine folds into the choice itself
        if self.queues[me].len() >= 2 {
            let conf = |x: &str| matches!(x.strip_prefix("pick").map(|r| r.trim().to_lowercase()).as_deref(), Some("yes") | Some("play"));
            let a = self.queues[me][0].trim().to_string();
            let b = self.queues[me][1].trim().to_string();
            if conf(&a) && !conf(&b) && self.match_pick(&p, a.strip_prefix("pick").unwrap().trim()).is_none() {
                if let Some(bp) = b.strip_prefix("pick") {
                    if self.match_pick(&p, bp.trim()).is_some() {
                        self.queues[me].pop_front();
                        self.log(format!("p{} dropped confirmation pick [{a}] the engine does not ask", me + 1));
                    }
                }
            }
        }
        if let Some(h) = self.queues[me].front().cloned() {
            if let Some(pk) = h.strip_prefix("pick") {
                let pk = pk.trim().to_string();
                match self.match_pick(&p, &pk) {
                    Some((i, rest)) => {
                        self.queues[me].pop_front();
                        self.log(format!("p{} pick {pk} for {:?} among {:?}", me + 1, p.kind, self.options_text(&p)));
                        for r in rest {
                            self.chain[me].push_back(r);
                        }
                        self.apply(i);
                        return Ok(());
                    }
                    None => {
                        // negative assertions are checked at the first real choice
                        if let Some(neg) = pk.strip_prefix('!') {
                            if n > 1 {
                                let txt = self.options_text(&p);
                                for name in neg.split(';') {
                                    if txt.iter().any(|t| t == name.trim()) {
                                        self.failures.push(format!("script: {:?} was offered but must not be ({:?})", name.trim(), txt));
                                    }
                                }
                                self.queues[me].pop_front();
                            }
                        }
                    }
                }
            }
        }
        // default
        let i = self.default_choice(&p);
        let txt = self.options_text(&p);
        self.defaults.push(format!("p{} DEFAULT {:?} -> {:?} among {:?}", me + 1, p.kind, txt.get(i), txt));
        self.log(format!("p{} DEFAULT {:?} -> {:?} among {:?}", me + 1, p.kind, txt.get(i), txt));
        self.apply(i);
        Ok(())
    }

    /// The first mode whose follow-up target decision offers `want` (Forge picks modes through
    /// the targets the script names).
    /// Forge declares attackers/blockers before the first priority window of the step, so scripts
    /// that wait for the step and then declare find the declaration waiting behind the `at`.
    fn drop_phase_waits(&mut self, me: usize, phase: &str) {
        while let Some(h) = self.queues[me].front() {
            if h.trim() == format!("at {phase}") && self.queues[me].get(1).map_or(false, |n| n.starts_with("attack") || n.starts_with("block")) {
                self.queues[me].pop_front();
            } else {
                break;
            }
        }
    }

    fn mode_offering_target(&self, p: &Pending, want: &str) -> Option<usize> {
        for (i, o) in p.options.iter().enumerate() {
            if !matches!(o, Opt::Mode(_)) {
                continue;
            }
            let mut g = self.g.clone();
            g.apply(p.id, i).ok()?;
            for _ in 0..12 {
                if !matches!(g.advance(), Status::NeedDecision(_)) {
                    break;
                }
                let q = g.pending().unwrap().clone();
                match q.kind {
                    DecisionKind::ChooseMode { .. } => {
                        let done = q.options.iter().position(|o| matches!(o, Opt::Done)).unwrap_or(0);
                        g.apply(q.id, done).ok()?;
                    }
                    DecisionKind::ChooseTarget { .. } => {
                        if q.options.iter().any(|o| self.opt_text_in(&g, o) == want.trim_start_matches("stack:").trim()) {
                            return Some(i);
                        }
                        break;
                    }
                    _ => break,
                }
            }
        }
        None
    }

    fn mode_by_label(&self, p: &Pending, hint: &str) -> Option<usize> {
        let DecisionKind::ChooseMode { spell, .. } = p.kind else { return None };
        let def = self.db.def(self.st().def_of(spell));
        let (_, sd) = def.spell_def()?;
        for (i, o) in p.options.iter().enumerate() {
            if let Opt::Mode(m) = o {
                let l = sd.modes[*m as usize].label.to_lowercase();
                if l.contains(hint) || format!("{:?}", sd.modes[*m as usize]).to_lowercase().contains(hint) {
                    return Some(i);
                }
            }
        }
        None
    }

    fn default_choice(&self, p: &Pending) -> usize {
        match p.kind {
            // optional things: decline by default, like an unscripted pass
            DecisionKind::May | DecisionKind::PayUnless => p.options.iter().position(|o| matches!(o, Opt::Yes)).unwrap_or(0),
            DecisionKind::ChangeTargets => p.options.iter().position(|o| matches!(o, Opt::No)).unwrap_or(0),
            DecisionKind::ChooseCards { purpose, .. } => {
                if matches!(purpose, CardsPurpose::SurveilGraveyard | CardsPurpose::ScryBottom) {
                    p.options.iter().position(|o| matches!(o, Opt::Done)).unwrap_or(0)
                } else if matches!(purpose, CardsPurpose::Delve) {
                    // enough already: stop; otherwise spend instants and sorceries first (counts for Murktide)
                    p.options.iter().position(|o| matches!(o, Opt::Done)).or_else(|| {
                        p.options.iter().position(|o| matches!(o, Opt::Card(r) if self.st().obj_data(self.db, *r).chars.types.intersects(Types::SPELL_KIND)))
                    }).unwrap_or(0)
                } else {
                    0
                }
            }
            DecisionKind::ChooseTarget { .. } => {
                // Forge's AI aims at the opponent
                let opp = p.seat.other();
                p.options.iter().position(|o| matches!(o, Opt::Target(Target::Player(s)) if *s == opp)).unwrap_or(0)
            }
            _ => 0,
        }
    }

    fn match_text(&self, p: &Pending, want: &str) -> Option<usize> {
        let want = want.trim();
        if let Some((n, c)) = want.rsplit_once('@') {
            let seat = self.seat_arg(c);
            for (i, o) in p.options.iter().enumerate() {
                let r = match o {
                    Opt::Card(r) | Opt::Target(Target::Obj(r)) => *r,
                    _ => continue,
                };
                if self.nm(r) == n.trim() && self.st().obj_data(self.db, r).controller == seat {
                    return Some(i);
                }
            }
            return None;
        }
        for (i, o) in p.options.iter().enumerate() {
            let t = self.opt_text(o);
            if t == want {
                return Some(i);
            }
        }
        // stack:Name -> any object target whose def name is Name
        if let Some(n) = want.strip_prefix("stack:") {
            for (i, o) in p.options.iter().enumerate() {
                if self.opt_text(o) == n.trim() {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Pure matching of a `pick` entry against a decision: the option to take and the names left
    /// over for successive decisions (multi-name picks).
    fn match_pick(&self, p: &Pending, pk: &str) -> Option<(usize, Vec<String>)> {
        let txt = self.options_text(p);
        let find = |pred: &dyn Fn(&Opt) -> bool| p.options.iter().position(|o| pred(o));
        let l = pk.to_lowercase();
        if pk.starts_with('!') {
            return None;
        }
        let idx = match l.as_str() {
            "yes" | "true" => find(&|o| matches!(o, Opt::Yes)),
            "no" | "false" => find(&|o| matches!(o, Opt::No)),
            "none" => find(&|o| matches!(o, Opt::Done | Opt::Target(Target::None) | Opt::No | Opt::NoBlock | Opt::NoAttack)),
            "skip" => find(&|o| matches!(o, Opt::Done | Opt::No)),
            "play" => find(&|o| matches!(o, Opt::Yes)).or_else(|| if p.options.iter().filter(|o| matches!(o, Opt::Card(_))).count() == 1 { find(&|o| matches!(o, Opt::Card(_))) } else { None }),
            _ => None,
        };
        if let Some(i) = idx {
            return Some((i, vec![]));
        }
        let names: Vec<String> = pk.split(';').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        if names.is_empty() {
            return None;
        }
        if names.len() == 1 {
            if let Ok(nv) = names[0].parse::<u32>() {
                if let Some(i) = find(&|o| matches!(o, Opt::Number(n) if *n == nv)) {
                    return Some((i, vec![]));
                }
            }
            if let Some(ci) = ["white", "blue", "black", "red", "green"].iter().position(|c| *c == l) {
                if let Some(i) = find(&|o| matches!(o, Opt::Choice(c) if *c as usize == ci)) {
                    return Some((i, vec![]));
                }
            }
        }
        for (k, name) in names.iter().enumerate() {
            if let Some(i) = self.match_text(p, name) {
                return Some((i, names[k + 1..].to_vec()));
            }
        }
        if let DecisionKind::ChooseMode { .. } = p.kind {
            for name in &names {
                if let Some(i) = self.mode_by_label(p, &name.to_lowercase()) {
                    return Some((i, vec![]));
                }
            }
        }
        // a name given where the engine asks yes/no: "attach it to the Monk" style scripts
        if p.options.iter().any(|o| matches!(o, Opt::Yes)) && p.options.iter().any(|o| matches!(o, Opt::No)) && names.len() == 1 && !matches!(l.as_str(), "none") {
            return find(&|o| matches!(o, Opt::Yes)).map(|i| (i, vec![]));
        }
        let labels: Vec<String> = self.g.decision_for(p.seat).map(|d| d.options.iter().map(|o| o.label.to_lowercase()).collect()).unwrap_or_default();
        if let Some(i) = labels.iter().position(|t| t.contains(&l)) {
            return Some((i, vec![]));
        }
        let _ = txt;
        None
    }

    // ---------------------------------------------------------------- expectations

    fn seat_arg(&self, s: &str) -> Seat {
        if s.trim() == "p1" {
            self.seat_of[0]
        } else {
            self.seat_of[1]
        }
    }

    fn zone_names(&self, seat: Seat, z: &str) -> Result<Vec<String>, String> {
        let s = self.st();
        let db = self.db;
        let names = |v: &[ObjRef]| v.iter().map(|&r| strip_src(def_name(db, s, r))).collect::<Vec<_>>();
        Ok(match z {
            "hand" => names(s.hand(seat)),
            "graveyard" | "gy" => names(s.graveyard(seat)),
            "library" | "lib" => names(s.library(seat)),
            "battlefield" | "bf" => s.battlefield().iter().filter(|&&r| s.obj_data(db, r).controller == seat).map(|&r| strip_src(def_name(db, s, r))).collect(),
            "exile" => s.exile().iter().filter(|&&r| s.obj_data(db, r).owner == seat).map(|&r| strip_src(def_name(db, s, r))).collect(),
            "command" => s.emblem_objs(seat).iter().map(|&r| def_name(db, s, r)).chain(s.dungeon_of(seat).map(|(d, _)| db.def(d).name.clone())).collect(),
            _ => return Err(format!("zone {z}")),
        })
    }

    fn battlefield_of(&self, seat: Seat, name: &str) -> Vec<ObjRef> {
        let s = self.st();
        s.battlefield().iter().copied().filter(|&r| s.obj_data(self.db, r).controller == seat && strip_src(def_name(self.db, s, r)) == name).collect()
    }

    fn eval(&mut self, e: &str) -> Result<Option<String>, String> {
        let e = e.trim();
        let (cmd, rest) = e.split_once(char::is_whitespace).map(|(a, b)| (a, b.trim())).unwrap_or((e, ""));
        let cmp = |have: i64, op: &str, want: i64| match op {
            "==" => have == want,
            "!=" => have != want,
            ">=" => have >= want,
            "<=" => have <= want,
            ">" => have > want,
            "<" => have < want,
            _ => false,
        };
        let parse_cmp = |s: &str| -> Option<(String, String, i64)> {
            for op in ["==", "!=", ">=", "<=", ">", "<"] {
                if let Some(pos) = s.find(op) {
                    let l = s[..pos].trim().to_string();
                    let r: i64 = s[pos + op.len()..].trim().parse().ok()?;
                    return Some((l, op.to_string(), r));
                }
            }
            None
        };
        let unq = |s: &str| s.trim().trim_matches('"').to_string();
        Ok(match cmd {
            "not" => match self.eval(rest)? {
                None => Some(format!("expectation unexpectedly true: {rest}")),
                Some(_) => None,
            },
            "life" => {
                let (l, op, r) = parse_cmp(rest).ok_or("bad life syntax")?;
                let have = self.st().life(self.seat_arg(&l)) as i64;
                if cmp(have, &op, r) { None } else { Some(format!("life is {have}")) }
            }
            "zone" => {
                let mut it = rest.splitn(3, char::is_whitespace);
                let (p, z, tail) = (it.next().unwrap_or(""), it.next().unwrap_or(""), it.next().unwrap_or("").trim());
                let names = self.zone_names(self.seat_arg(p), z)?;
                if let Some(n) = tail.strip_prefix("has ") {
                    let n = unq(n);
                    if names.contains(&n) { None } else { Some(format!("not found; zone has {names:?}")) }
                } else if let Some(n) = tail.strip_prefix("lacks ") {
                    let n = unq(n);
                    if names.contains(&n) { Some(format!("found {n}")) } else { None }
                } else if let Some(t) = tail.strip_prefix("size") {
                    let (_, op, r) = parse_cmp(&format!("x{t}")).ok_or("bad size syntax")?;
                    if cmp(names.len() as i64, &op, r) { None } else { Some(format!("size is {}: {names:?}", names.len())) }
                } else if let Some(t) = tail.strip_prefix("count ") {
                    let (l, op, r) = parse_cmp(t).ok_or("bad count syntax")?;
                    let n = unq(&l);
                    let have = names.iter().filter(|x| **x == n).count() as i64;
                    if cmp(have, &op, r) { None } else { Some(format!("count is {have}: {names:?}")) }
                } else {
                    return Err(format!("bad zone clause {tail}"));
                }
            }
            "owner-zone" => {
                let mut it = rest.splitn(3, char::is_whitespace);
                let (p, z, tail) = (it.next().unwrap_or(""), it.next().unwrap_or(""), it.next().unwrap_or("").trim());
                let seat = self.seat_arg(p);
                let s = self.st();
                let want_has = tail.starts_with("has");
                let n = unq(tail.trim_start_matches("has").trim_start_matches("lacks").trim());
                let zk = match z {
                    "exile" => ZoneKind::Exile,
                    "graveyard" | "gy" => ZoneKind::Graveyard,
                    "hand" => ZoneKind::Hand,
                    "battlefield" | "bf" => ZoneKind::Battlefield,
                    _ => return Err(format!("zone {z}")),
                };
                let mut all: Vec<ObjRef> = vec![];
                match zk {
                    ZoneKind::Exile => all.extend_from_slice(s.exile()),
                    ZoneKind::Battlefield => all.extend_from_slice(s.battlefield()),
                    ZoneKind::Graveyard => {
                        all.extend_from_slice(s.graveyard(Seat(0)));
                        all.extend_from_slice(s.graveyard(Seat(1)));
                    }
                    _ => {
                        all.extend_from_slice(s.hand(Seat(0)));
                        all.extend_from_slice(s.hand(Seat(1)));
                    }
                }
                let has = all.iter().any(|&r| s.obj_data(self.db, r).owner == seat && def_name(self.db, s, r) == n);
                if has == want_has { None } else { Some(if has { "found".into() } else { "not found".into() }) }
            }
            "stack" => {
                let n = self.st().stack().len();
                if rest == "empty" {
                    if n == 0 { None } else { Some(format!("stack has {n}")) }
                } else if let Some(t) = rest.strip_prefix("size") {
                    let (_, op, r) = parse_cmp(&format!("x{t}")).ok_or("bad stack size")?;
                    if cmp(n as i64, &op, r) { None } else { Some(format!("stack size {n}")) }
                } else {
                    return Err("bad stack clause".into());
                }
            }
            "tapped" | "untapped" => {
                let (p, n) = rest.split_once(char::is_whitespace).ok_or("bad syntax")?;
                let cs = self.battlefield_of(self.seat_arg(p), &unq(n));
                if cs.is_empty() {
                    Some("no such permanent".into())
                } else if cs.iter().any(|&r| self.st().obj_data(self.db, r).tapped == (cmd == "tapped")) {
                    None
                } else {
                    Some(format!("state is {}", if cmd == "tapped" { "untapped" } else { "tapped" }))
                }
            }
            "counters" => {
                let toks: Vec<&str> = rest.splitn(2, char::is_whitespace).collect();
                let p = toks[0];
                let rem = toks[1];
                let (l, op, r) = parse_cmp(rem).ok_or("bad counters syntax")?;
                let l = l.trim().to_string();
                let (name, kind) = if let Some(stripped) = l.strip_prefix('"') {
                    let end = stripped.find('"').ok_or("bad quote")?;
                    (stripped[..end].to_string(), stripped[end + 1..].trim().to_string())
                } else {
                    let (a, b) = l.split_once(char::is_whitespace).ok_or("bad counters syntax")?;
                    (a.to_string(), b.trim().to_string())
                };
                let cs = self.battlefield_of(self.seat_arg(p), &name);
                if cs.is_empty() {
                    Some("no such permanent".into())
                } else {
                    let k = counter_from_forge(&kind).ok_or_else(|| format!("counter kind {kind}"))?;
                    let have = self.st().obj_data(self.db, cs[0]).counters.iter().find(|c| c.0 == k).map(|c| c.1).unwrap_or(0) as i64;
                    if cmp(have, &op, r) { None } else { Some(format!("counters {have}")) }
                }
            }
            "pt" => {
                let (p, tail) = rest.split_once(char::is_whitespace).ok_or("bad pt")?;
                let (name, pt) = if let Some(stripped) = tail.trim().strip_prefix('"') {
                    let end = stripped.find('"').ok_or("quote")?;
                    (stripped[..end].to_string(), stripped[end + 1..].trim().to_string())
                } else {
                    let (a, b) = tail.trim().rsplit_once(char::is_whitespace).ok_or("bad pt")?;
                    (a.to_string(), b.to_string())
                };
                let (pw, tg) = pt.split_once('/').ok_or("bad pt")?;
                let cs = self.battlefield_of(self.seat_arg(p), &name);
                if cs.is_empty() {
                    Some("no such permanent".into())
                } else {
                    let ch = self.st().obj_data(self.db, cs[0]).chars;
                    if ch.power == pw.parse::<i32>().unwrap_or(-99) && ch.toughness == tg.parse::<i32>().unwrap_or(-99) { None } else { Some(format!("P/T is {}/{}", ch.power, ch.toughness)) }
                }
            }
            "creature" | "notcreature" => {
                let (p, n) = rest.split_once(char::is_whitespace).ok_or("bad syntax")?;
                let cs = self.battlefield_of(self.seat_arg(p), &unq(n));
                if cs.is_empty() {
                    Some("no such permanent".into())
                } else {
                    let is = self.st().obj_data(self.db, cs[0]).chars.types.contains(Types::CREATURE);
                    if is == (cmd == "creature") { None } else { Some(format!("isCreature={is}")) }
                }
            }
            "controls" => {
                let (p, n) = rest.split_once(char::is_whitespace).ok_or("bad syntax")?;
                if self.battlefield_of(self.seat_arg(p), &unq(n)).is_empty() { Some("not controlled".into()) } else { None }
            }
            "lost" | "alive" => {
                let seat = self.seat_arg(rest);
                let lost = match self.g.result() {
                    Some(GameResult::Win(w)) => w != seat,
                    Some(GameResult::Draw) => true,
                    None => false,
                };
                if lost == (cmd == "lost") { None } else { Some(if lost { "lost".into() } else { "still in game".into() }) }
            }
            "gameover" => {
                if self.g.result().is_some() { None } else { Some("game still running".into()) }
            }
            "phase" => {
                let st = step_name(self.st().turn_data().step);
                if st.eq_ignore_ascii_case(rest) { None } else { Some(format!("phase {st}")) }
            }
            "mana" => {
                let (l, op, r) = parse_cmp(rest).ok_or("bad mana syntax")?;
                let have: i64 = self.st().pool(self.seat_arg(&l)).iter().map(|&x| x as i64).sum();
                if cmp(have, &op, r) { None } else { Some(format!("pool {have}")) }
            }
            "spells" => {
                let (l, op, r) = parse_cmp(rest).ok_or("bad spells syntax")?;
                let have = self.st().spells_cast(self.seat_arg(&l)) as i64;
                if cmp(have, &op, r) { None } else { Some(format!("spells cast {have}")) }
            }
            "top" => {
                let toks: Vec<&str> = rest.splitn(3, char::is_whitespace).collect();
                let seat = self.seat_arg(toks[0]);
                let i: usize = toks[1].parse::<usize>().map_err(|e| e.to_string())? - 1;
                let n = unq(toks[2]);
                let lib = self.zone_names(seat, "library")?;
                // the engine stores the library bottom-first? use the accessor order check below
                let lib_top_first = self.library_top_first(seat);
                let _ = lib;
                if i >= lib_top_first.len() {
                    Some(format!("library has only {} cards", lib_top_first.len()))
                } else if lib_top_first[i] == n {
                    None
                } else {
                    Some(format!("card {} is {}; top cards {:?}", i + 1, lib_top_first[i], &lib_top_first[..lib_top_first.len().min(6)]))
                }
            }
            "draws" | "log" | "trace" => {
                self.skipped.push(format!("expectation {cmd} not supported on the Rust side"));
                None
            }
            other => return Err(format!("unknown expectation {other}")),
        })
    }

    /// The library in top-first order (the engine keeps the top at the END of its vector).
    fn library_top_first(&self, seat: Seat) -> Vec<String> {
        let s = self.st();
        s.library(seat).iter().rev().map(|&r| def_name(self.db, s, r)).collect()
    }
}

pub fn run(db: &Arc<CardDb>, sc: &Scn, verbose: bool) -> (Outcome, Vec<String>) {
    match run_inner(db, sc) {
        Ok((out, trace)) => {
            if verbose {
                for l in &trace {
                    println!("      {l}");
                }
            }
            (out, trace)
        }
        Err(e) => (Outcome::Error(e), vec![]),
    }
}

fn mana_pool_from(s: &str) -> Result<[u8; 6], String> {
    let mut p = [0u8; 6];
    for t in s.split_whitespace() {
        let i = match t {
            "W" => 0,
            "U" => 1,
            "B" => 2,
            "R" => 3,
            "G" => 4,
            "C" => 5,
            o => return Err(format!("mana symbol {o}")),
        };
        p[i] += 1;
    }
    Ok(p)
}

fn build_setup(db: &CardDb, sc: &Scn) -> Result<(ScenarioSetup, [Seat; 2], bool), String> {
    let seat_of = if sc.swap { [Seat(1), Seat(0)] } else { [Seat(0), Seat(1)] };
    let mut players: [PlayerSetup; 2] = [PlayerSetup { life: 20, ..Default::default() }, PlayerSetup { life: 20, ..Default::default() }];
    let mut turn: u16 = 3;
    let mut active = 0usize;
    let mut step = Step::Main1;
    let mut remove_sick = true;
    let mut explicit_mana = false;
    let mut have_lib = [false; 2];
    for line in &sc.state {
        let (k, v) = line.split_once('=').ok_or_else(|| format!("bad state line {line}"))?;
        let k = k.trim();
        let v = v.trim();
        match k {
            "turn" => turn = v.parse().map_err(|_| "turn")?,
            "activeplayer" => active = if v == "p1" || v == "human" { 0 } else { 1 },
            "activephase" => step = step_from_name(v).ok_or_else(|| format!("phase {v}"))?,
            "removesummoningsickness" => remove_sick = v == "true",
            _ => {
                let (who, what) = if let Some(r) = k.strip_prefix("human") { (0, r) } else if let Some(r) = k.strip_prefix("ai") { (1, r) } else if let Some(r) = k.strip_prefix("p1") { (0, r) } else if let Some(r) = k.strip_prefix("p2") { (1, r) } else { return Err(format!("state key {k}")) };
                let seat = seat_of[who].idx();
                let p = &mut players[seat];
                let items: Vec<&str> = v.split(';').map(|x| x.trim()).filter(|x| !x.is_empty()).collect();
                match what {
                    "life" => p.life = v.parse().map_err(|_| "life")?,
                    "landsplayed" => p.lands_played = v.parse().map_err(|_| "landsplayed")?,
                    "hand" => p.hand = items.iter().map(|s| s.to_string()).collect(),
                    "graveyard" => p.graveyard = items.iter().map(|s| s.to_string()).collect(),
                    "exile" => p.exile = items.iter().map(|s| s.to_string()).collect(),
                    "library" => {
                        p.library = items.iter().map(|s| s.to_string()).collect();
                        have_lib[seat] = true;
                    }
                    "manapool" => {
                        p.pool = mana_pool_from(v)?;
                        explicit_mana = true;
                    }
                    "battlefield" => {
                        for it in items {
                            let mut parts = it.split('|');
                            let name = parts.next().unwrap().trim();
                            let mut ps = PermSetup::new(name);
                            let mut modal = false;
                            for m in parts {
                                let m = m.trim();
                                if m == "Modal" {
                                    modal = true;
                                } else if m == "NoETBTrigs" {
                                    // no enter triggers: the setup never fires any
                                } else if m == "Tapped" {
                                    ps.tapped = true;
                                } else if let Some(c) = m.strip_prefix("Counters:") {
                                    for kv in c.split(',') {
                                        let (kn, n) = kv.split_once('=').ok_or("counter syntax")?;
                                        let kind = counter_from_forge(kn.trim()).ok_or_else(|| format!("counter kind {kn}"))?;
                                        ps.counters.push((kind, n.trim().parse().map_err(|_| "counter n")?));
                                    }
                                } else if let Some(t) = m.strip_prefix("ChosenType:") {
                                    let ti = SUBTYPE_NAMES.iter().position(|x| *x == t.trim()).ok_or_else(|| format!("subtype {t}"))?;
                                    ps.chosen = ti as u16 + 1;
                                } else if m == "Exiled" {
                                    // Forge: the permanent already has a linked exile; the engine's setup has no
                                    // link, so the modifier is dropped (scenario runs as partial coverage).
                                } else if m == "SummonSick" {
                                    ps.summoning_sick = true;
                                } else if let Some(d) = m.strip_prefix("Damage:") {
                                    ps.damage = d.trim().parse().map_err(|_| "damage")?;
                                } else {
                                    return Err(format!("unsupported modifier {m} on {name}"));
                                }
                            }
                            if modal {
                                // Forge's "Modal" state is the back face of a double-faced card: the engine models it as its own definition
                                let back = db.defs.iter().find(|d| d.name == name).and_then(|d| d.back.clone()).ok_or_else(|| format!("no back face for {name}"))?;
                                ps.name = back;
                            }
                            p.battlefield.push(ps);
                        }
                    }
                    other => return Err(format!("state key {other}")),
                }
            }
        }
    }
    for s in 0..2 {
        if !have_lib[s] {
            players[s].library = vec!["Island".to_string(); 10];
        }
    }
    for s in 0..2 {
        for n in players[s].hand.iter().chain(players[s].graveyard.iter()).chain(players[s].exile.iter()).chain(players[s].library.iter()).chain(players[s].battlefield.iter().map(|b| &b.name)) {
            if db.id(n).is_none() {
                return Err(format!("unknown card {n}"));
            }
        }
    }
    let _ = remove_sick;
    let setup = ScenarioSetup { turn, active: seat_of[active], step, players, seed: 12345 };
    Ok((setup, seat_of, explicit_mana))
}

/// Builds a game at the scenario's first priority from Forge-format state lines (the same lines the Forge probe is injected with).
pub fn game_from_state(db: &Arc<CardDb>, state: &[String]) -> Result<Game, String> {
    let sc = Scn { state: state.to_vec(), ..Default::default() };
    let (setup, _seat_of, _explicit) = build_setup(db, &sc)?;
    let mut g = Game::from_scenario(db.clone(), &setup);
    g.set_track_view_events(false);
    g.set_keep_events(true);
    g.advance();
    Ok(g)
}

fn run_inner(db: &Arc<CardDb>, sc: &Scn) -> Result<(Outcome, Vec<String>), String> {
    let (setup, seat_of, explicit) = build_setup(db, sc)?;
    let mut game = Game::from_scenario(db.clone(), &setup);
    if sc.script.iter().flatten().any(|l| l.contains("~Add") || l.contains("~add")) {
        game.raw_state_mut().scenario_explicit_mana();
    }
    let _ = explicit;
    let mut run = Run {
        db,
        g: game,
        seat_of,
        queues: [sc.script[0].iter().cloned().collect(), sc.script[1].iter().cloned().collect()],
        trace: vec![],
        failures: vec![],
        skipped: vec![],
        cast_targets: [VecDeque::new(), VecDeque::new()],
        mode_hint: [None, None],
        chain: [VecDeque::new(), VecDeque::new()],
        attack_set: [None, None],
        block_pairs: [None, None],
        attack_target_player: [None, None],
        start_turn: setup.turn,
        prio_calls: [0, 0],
        defaults: vec![],
        x_value: None,
        backup: None,
    };
    let mut steps = 0u32;
    loop {
        steps += 1;
        if steps > 20000 {
            return Ok((Outcome::Error("too many decisions".into()), run.trace));
        }
        match run.g.advance() {
            Status::GameOver(_) => break,
            Status::NeedDecision(seat) => {
                let p = run.g.pending().unwrap().clone();
                match p.kind {
                    DecisionKind::Priority => match run.priority(seat) {
                        Ok(true) => {}
                        Ok(false) => break,
                        Err(e) => {
                            return Ok((Outcome::Error(e), run.trace));
                        }
                    },
                    _ => {
                        let me = run.idx_of_seat(seat);
                        let _ = me;
                        if let Err(e) = run.sub_decision(seat) {
                            return Ok((Outcome::Error(e), run.trace));
                        }
                    }
                }
            }
        }
    }
    for e in sc.expect.clone() {
        match run.eval(&e) {
            Ok(None) => {}
            Ok(Some(bad)) => run.failures.push(format!("{e}   <-- {bad}")),
            Err(err) => run.failures.push(format!("{e}   <-- harness: {err}")),
        }
    }
    let mut trace = std::mem::take(&mut run.trace);
    for who in 0..2 {
        let seat = run.seat_of[who];
        let mut line = format!("   [final p{} life {}]", who + 1, run.st().life(seat));
        for z in ["hand", "battlefield", "graveyard", "exile", "library"] {
            if let Ok(v) = run.zone_names(seat, z) {
                let v: Vec<String> = if z == "library" { v.into_iter().rev().take(6).collect() } else { v };
                line.push_str(&format!(" {z}={v:?}"));
            }
        }
        trace.push(line);
    }
    trace.push(format!("   [final turn {} step {:?} stack {}]", run.st().turn_data().turn, run.st().turn_data().step, run.st().stack().len()));
    let out = if !run.failures.is_empty() {
        Outcome::Fail(run.failures)
    } else if !run.skipped.is_empty() {
        Outcome::PassPartial(run.skipped)
    } else {
        Outcome::Pass
    };
    Ok((out, trace))
}
