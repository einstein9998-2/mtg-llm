//! Effect-level step differential, Rust side. Replays a position from its trace, plays the position's action, then drives the
//! engine to the next empty-stack priority window while answering decisions from the Forge log (follower) or by the shared
//! canonical rules (targets and costs chosen while casting). Compares the resulting canonical summary with Forge's.

use crate::pos::window_keys;
use crate::util::*;
use mtg_core::card::CardDb;
use mtg_core::decision::*;
use mtg_core::ids::*;
use mtg_core::state::{GameConfig, State};
use mtg_view::{DeckList, Game};
use std::collections::VecDeque;
use std::sync::Arc;

pub fn rebuild(db: Arc<CardDb>, decks: [&DeckList; 2], seed: u64, first: u8, trace: &[u32]) -> Result<Game, String> {
    let cfg = GameConfig { first_player: Seat(first), ..GameConfig::default() };
    let mut g = Game::new(db, decks, seed, cfg);
    g.set_track_view_events(false);
    g.set_keep_events(true);
    for (i, &idx) in trace.iter().enumerate() {
        match g.advance() {
            Status::NeedDecision(_) => {}
            Status::GameOver(_) => return Err(format!("replay ended early at {i}")),
        }
        let id = g.pending().ok_or("no pending")?.id;
        g.apply(id, idx as usize).map_err(|e| format!("replay apply {i}: {e:?}"))?;
    }
    g.advance();
    Ok(g)
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub player: usize,
    pub method: String,
    pub args: Vec<String>,
    pub ret: String,
    pub used: bool,
}

pub fn parse_entry(l: &str) -> Option<Entry> {
    let (left, ret) = l.rsplit_once(" => ")?;
    let mut it = left.splitn(3, ' ');
    let pl = it.next()?;
    let method = it.next()?.to_string();
    let rest = it.next().unwrap_or("");
    let player = if pl == "P1" { 0 } else { 1 };
    Some(Entry { player, method, args: rest.split(" | ").map(|s| s.to_string()).collect(), ret: ret.to_string(), used: false })
}

fn strip_id(n: &str) -> String {
    let n = n.trim();
    if n.ends_with(')') {
        if let Some(p) = n.rfind(" (") {
            if n[p + 2..n.len() - 1].chars().all(|c| c.is_ascii_digit()) {
                return n[..p].to_string();
            }
        }
    }
    n.to_string()
}

fn names_of(ret: &str) -> Vec<String> {
    let r = ret.trim();
    if r == "null" {
        return vec![];
    }
    if let Some(inner) = r.strip_prefix('[').and_then(|x| x.strip_suffix(']')) {
        return inner.split(';').filter(|x| !x.trim().is_empty()).map(strip_id).collect();
    }
    vec![strip_id(r)]
}

/// Splits "(left,right)" pairs of arrangeForScry/Surveil into the two name lists.
fn pair_names(ret: &str) -> Option<(Vec<String>, Vec<String>)> {
    let r = ret.trim().strip_prefix('(')?.strip_suffix(')')?;
    let (a, b) = r.split_once("],[")?;
    Some((names_of(&format!("{a}]")), names_of(&format!("[{b}"))))
}

const CARD_METHODS: &[&str] = &[
    "chooseCardsToDiscardFrom", "chooseSingleCardForZoneChange", "chooseCardsForZoneChange", "chooseSingleEntityForEffect", "chooseEntitiesForEffect",
    "chooseCardsForEffect", "chooseCardsForEffectMultiple", "choosePermanentsToSacrifice", "choosePermanentsToDestroy", "chooseCardsToDiscardUnlessType",
    "chooseCardsToRevealFromHand", "chooseCardsToDelve", "chooseCardsForCost",
];
const BIN_METHODS: &[&str] = &["confirmAction", "chooseBinary", "confirmTrigger", "confirmReplacementEffect", "willPutCardOnTop", "payManaOptionalCost"];

/// Identically named legendary permanents: Forge's log names the copy it kept without telling the copies apart, so the line driver tries the
/// picks in turn (see `linediff`): the m-th such decision of a line uses `picks[m]` (default 0).
pub struct LegendState {
    pub picks: Vec<usize>,
    pub counter: usize,
    /// number of options at each such decision met so far
    pub options: Vec<usize>,
}
pub static LEGEND: std::sync::Mutex<LegendState> = std::sync::Mutex::new(LegendState { picks: vec![], counter: 0, options: vec![] });

pub struct StepResult {
    pub shuffled_seats: [bool; 2],
    pub response_missing: bool,
    /// canonical action keys the engine offers at the response window
    pub window_actions: Vec<String>,
    pub unmatched: Vec<String>,
    pub shuffled: bool,
    pub over: Option<GameResult>,
    pub steps: u32,
    pub leftover: Vec<String>,
    pub trace: Vec<String>,
}

struct Follower<'a> {
    db: &'a CardDb,
    entries: Vec<Entry>,
    cur: Option<(CardsPurpose, VecDeque<String>)>,
    /// (purpose, remaining) of the previous card decision; a new series starts when the purpose changes or `remaining` does not drop
    last_card: Option<(CardsPurpose, u8)>,
    cur_targets_set: Option<VecDeque<String>>,
    pending_sets: VecDeque<VecDeque<String>>,
    x_value: Option<u32>,
    /// life the Forge AI paid for Phyrexian mana in the cast of this step (`PHY n` log line)
    phy_life: Option<i32>,
    unmatched: Vec<String>,
    shuffle_mirrored: bool,
    /// the kept-on-top cards of the last surveil/scry (Forge's first tuple member): feeds the OrderTop series the engine asks next
    top_order: Option<Vec<String>>,
    /// the cards the last scry sent to the bottom, in Forge's order: feeds the OrderBottom series
    bottom_order: Option<Vec<String>>,
}

fn pname(s: Seat) -> &'static str {
    if s.0 == 0 {
        "P1"
    } else {
        "P2"
    }
}

fn otext(db: &CardDb, st: &State, o: &Opt) -> String {
    match *o {
        Opt::Card(r) | Opt::Target(Target::Obj(r)) | Opt::Block(r) | Opt::PlayLand(r) | Opt::PlayLandBack(r, _) => def_name(db, st, r),
        Opt::Target(Target::Player(p)) => pname(p).to_string(),
        Opt::Target(Target::None) => "none".into(),
        Opt::Name(d) => db.def(CardDefId(d)).name.clone(),
        Opt::Type(t) => subtype_name(t).to_string(),
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
        Opt::Cast(r, w) => format!("{} {}", def_name(db, st, r), way_text(db, st, r, w)),
        Opt::Activate { src, ability } => format!("{} {}", def_name(db, st, src), ability_text(db, st.def_of(src), ability)),
        Opt::Mana { src, .. } => def_name(db, st, src),
        Opt::Target(_) => "target".into(),
    }
}

fn ctl_idx(db: &CardDb, st: &State, r: ObjRef) -> u8 {
    st.obj_data(db, r).controller.0
}

/// Canonical target key shared with the Forge probe: opponent player, own player, then objects by (name, controller).
fn target_key(db: &CardDb, st: &State, actor: Seat, o: &Opt) -> Option<String> {
    match *o {
        Opt::Target(Target::Player(p)) => Some(format!("0|{}", if p == actor { "1" } else { "0" })),
        Opt::Target(Target::Obj(r)) => Some(format!("1|{}|{}", def_name(db, st, r), ctl_idx(db, st, r))),
        _ => None,
    }
}

impl<'a> Follower<'a> {
    fn take(&mut self, methods: &[&str]) -> Option<usize> {
        let i = self.entries.iter().position(|e| !e.used && methods.contains(&e.method.as_str()))?;
        self.entries[i].used = true;
        Some(i)
    }

    fn pick_name(&self, g: &Game, p: &Pending, name: &str) -> Option<usize> {
        let st = g.raw_state();
        p.options.iter().position(|o| otext(self.db, st, o) == name)
    }

    fn follow_cards(&mut self, g: &Game, p: &Pending, purpose: CardsPurpose, remaining: u8) -> usize {
        let st = g.raw_state();
        let seat = p.seat.0 as usize;
        let done = p.options.iter().position(|o| matches!(o, Opt::Done));
        let same_series = matches!(self.last_card, Some((pp, r)) if pp == purpose && (remaining < r || purpose == CardsPurpose::Delve));
        self.last_card = Some((purpose, remaining));
        if !same_series {
            self.cur = None;
        }
        let opts: Vec<String> = p.options.iter().map(|o| otext(self.db, st, o)).collect();
        for _round in 0..4 {
            if self.cur.is_none() {
                // the next entry of the family this purpose reads, made by this seat
                let loaded: Option<Vec<String>> = match purpose {
                    CardsPurpose::SurveilGraveyard | CardsPurpose::ScryBottom => {
                        let m = if purpose == CardsPurpose::ScryBottom { "arrangeForScry" } else { "arrangeForSurveil" };
                        self.take_for(seat, &[m]).and_then(|i| pair_names(&self.entries[i].ret)).map(|(t, r)| {
                            self.top_order = Some(t);
                            if purpose == CardsPurpose::ScryBottom {
                                self.bottom_order = Some(r.clone());
                            }
                            r
                        })
                    }
                    // Forge moves the listed cards to the library one after another: the last one moved ends on top
                    CardsPurpose::OrderTop => self.take_order(seat).map(|i| {
                        let mut v = names_of(&self.entries[i].ret);
                        v.reverse();
                        v
                    }).or_else(|| self.top_order.take()),
                    // Forge delves in one call listing every exiled card (with repeats for duplicate names); no call: nothing delved
                    CardsPurpose::Delve => Some(self.take_for(seat, &["chooseCardsToDelve"]).map(|i| names_of(&self.entries[i].ret)).unwrap_or_default()),
                    CardsPurpose::OrderBottom => self.take_order(seat).map(|i| names_of(&self.entries[i].ret)).or_else(|| self.bottom_order.take()),
                    // Forge picks per zone (graveyard, hand, library) and ends each zone with a null pick; the engine asks one combined series
                    CardsPurpose::ExtractExile => {
                        let mut names = vec![];
                        for e in self.entries.iter_mut() {
                            if !e.used && e.player == seat && e.method == "chooseSingleCardForZoneChange" && e.args.first().map_or(false, |a| a.trim() == "Exile") {
                                e.used = true;
                                names.extend(names_of(&e.ret));
                            }
                        }
                        Some(names)
                    }
                    _ => {
                        let pos = self.entries.iter().position(|e| {
                            !e.used && e.player == seat && CARD_METHODS.contains(&e.method.as_str()) && {
                                let n = names_of(&e.ret);
                                n.is_empty() || n.iter().any(|x| opts.contains(x))
                            }
                        });
                        pos.map(|i| {
                            self.entries[i].used = true;
                            names_of(&self.entries[i].ret)
                        })
                    }
                };
                match loaded {
                    Some(v) => self.cur = Some((purpose, v.into())),
                    None => break,
                }
            }
            let cur = self.cur.as_mut().unwrap();
            while let Some(n) = cur.1.pop_front() {
                if let Some(i) = opts.iter().position(|o| *o == n) {
                    return i;
                }
                // Surveil/scry after a shuffle: the libraries differ in order, so mirror the decision (how many cards), not the card
                if matches!(purpose, CardsPurpose::SurveilGraveyard | CardsPurpose::ScryBottom) {
                    if let Some(i) = p.options.iter().position(|o| matches!(o, Opt::Card(_))) {
                        self.shuffle_mirrored = true;
                        return i;
                    }
                }
            }
            // this entry is exhausted: a Done ends the series, otherwise the next decision needs a fresh entry
            self.cur = None;
            if let Some(d) = done {
                return d;
            }
        }
        if matches!(purpose, CardsPurpose::OrderTop | CardsPurpose::OrderBottom) {
            // Forge logs no ordering when the cards are identical or there is one: any pick gives the same library
            let cards: Vec<usize> = (0..p.options.len()).filter(|&i| matches!(p.options[i], Opt::Card(_))).collect();
            if !cards.is_empty() && cards.iter().all(|&i| opts[i] == opts[cards[0]]) {
                return cards[0];
            }
        }
        if purpose == CardsPurpose::ExtractExile {
            // Forge's Surgical Extraction exiles every copy it finds, with no choice to log
            if let Some(i) = p.options.iter().position(|o| matches!(o, Opt::Card(_))) {
                return i;
            }
        }
        if let Some(d) = done {
            return d;
        }
        self.unmatched.push(format!("{:?}: no logged choice among {:?}", p.kind, opts));
        0
    }

    /// The next unused library ordering by this seat (orderings into the graveyard or exile are not decisions the engine asks about).
    fn take_order(&mut self, seat: usize) -> Option<usize> {
        let i = self.entries.iter().position(|e| !e.used && e.player == seat && e.method == "orderMoveToZoneList" && e.args.get(1).map_or(true, |a| a.trim() == "Library"))?;
        self.entries[i].used = true;
        Some(i)
    }

    fn take_for(&mut self, seat: usize, methods: &[&str]) -> Option<usize> {
        let i = self.entries.iter().position(|e| !e.used && e.player == seat && methods.contains(&e.method.as_str()))?;
        self.entries[i].used = true;
        Some(i)
    }

    fn follow_target(&mut self, g: &Game, p: &Pending, slot: u8) -> usize {
        let st = g.raw_state();
        // object targets are named like the Forge log does: name, counters, and `|T` when tapped (same-named permanents differ in these)
        let opts: Vec<String> = p
            .options
            .iter()
            .map(|o| match *o {
                Opt::Target(Target::Obj(r)) => format!("{}{}", creature_label(self.db, st, r), if st.obj_data(self.db, r).tapped { "|T" } else { "" }),
                _ => otext(self.db, st, o),
            })
            .collect();
        let none = p.options.iter().position(|o| matches!(o, Opt::Target(Target::None) | Opt::Done));
        if slot == 0 || self.cur_targets_set.is_none() {
            // a new target set: the next targeted trigger or ability Forge's AI put on the stack, in placement order
            let seat = p.seat.0 as usize;
            while self.pending_sets.is_empty() {
                let Some(i) = self.entries.iter().position(|e| !e.used && e.method == "orderAndPlaySimultaneousSa") else { break };
                self.entries[i].used = true;
                let _ = seat;
                let list = self.entries[i].args[0].clone();
                let inner = list.trim().trim_start_matches('[').trim_end_matches(']');
                // items are `Name` or `Name{t1;t2}`, separated by ';' outside braces
                let (mut depth, mut cur, mut items) = (0, String::new(), vec![]);
                for ch in inner.chars() {
                    match ch {
                        '{' => { depth += 1; cur.push(ch) }
                        '}' => { depth -= 1; cur.push(ch) }
                        ';' if depth == 0 => items.push(std::mem::take(&mut cur)),
                        _ => cur.push(ch),
                    }
                }
                items.push(cur);
                for it in items {
                    if let Some(bi) = it.find('{') {
                        // targets are `Name{counters}` or `Name{counters}|T`, so split on ';' outside nested braces
                        let body = &it[bi + 1..it.len() - 1];
                        let (mut d2, mut c2, mut ts) = (0, String::new(), VecDeque::new());
                        for ch in body.chars() {
                            match ch {
                                '{' => { d2 += 1; c2.push(ch) }
                                '}' => { d2 -= 1; c2.push(ch) }
                                ';' if d2 == 0 => ts.push_back(std::mem::take(&mut c2)),
                                _ => c2.push(ch),
                            }
                        }
                        ts.push_back(c2);
                        self.pending_sets.push_back(ts);
                    }
                }
            }
            match self.pending_sets.pop_front() {
                Some(ts) => self.cur_targets_set = Some(ts),
                None => {
                    self.unmatched.push(format!("{:?}: no logged trigger/ability targets among {:?}", p.kind, opts));
                    return none.unwrap_or(0);
                }
            }
        }
        let set = self.cur_targets_set.as_mut().unwrap();
        match set.pop_front() {
            Some(t) => {
                // "stack:Name" targets are spells on the stack: the option is the spell's card name
                let t = t.strip_prefix("stack:").map(|x| format!("{x}{{}}")).unwrap_or(t);
                match opts.iter().position(|o| *o == t) {
                    Some(i) => i,
                    None => {
                        self.unmatched.push(format!("{:?}: logged target {t:?} not among {:?}", p.kind, opts));
                        none.unwrap_or(0)
                    }
                }
            }
            // the AI chose fewer targets than the engine offers slots for ("up to")
            None => match none {
                Some(i) => i,
                None => {
                    self.unmatched.push(format!("{:?}: no more logged targets", p.kind));
                    0
                }
            },
        }
    }
}

/// Plays `action_idx` at the position's priority window and follows the Forge log to the next empty-stack priority.
pub fn run_step(g: &mut Game, action_idx: usize, log: &[String], combat: Option<&str>, response: Option<&str>) -> StepResult {
    run_step_ex(g, action_idx, log, combat, response, false, true)
}

/// `line`: stop at the next priority decision of either seat instead of passing to an empty stack (one step of a lockstep line);
/// `cast0`: the applied action puts a spell or ability on the stack, so the cast-time choices follow the canonical rules (false for a pass).
pub fn run_step_ex(g: &mut Game, action_idx: usize, log: &[String], combat: Option<&str>, response: Option<&str>, line: bool, cast0: bool) -> StepResult {
    let db = g.db().clone();
    let mut f = Follower { db: &db, entries: log.iter().filter_map(|l| if l.starts_with("X ") || l.starts_with("PHY ") || l.starts_with("PAID ") { None } else { parse_entry(l) }).collect(), cur: None, last_card: None, cur_targets_set: None, pending_sets: VecDeque::new(), x_value: None, phy_life: log.iter().find_map(|l| l.strip_prefix("PHY ").and_then(|x| x.trim().parse().ok())), unmatched: vec![], shuffle_mirrored: false, top_order: None, bottom_order: None };
    for l in log {
        if let Some(x) = l.strip_prefix("X ") {
            f.x_value = x.trim().parse().ok();
        }
    }
    let mut res = StepResult { shuffled_seats: [false; 2], unmatched: vec![], shuffled: false, over: None, steps: 0, leftover: vec![], trace: vec![], response_missing: false, window_actions: vec![] };
    let id = g.pending().unwrap().id;
    let actor = g.pending().unwrap().seat;
    let mut responded = response.is_none();
    // line mode: pay with the sources Forge's payer used (the engine's automatic payment prefers the hinted ones)
    let paid: Vec<String> = log.iter().find_map(|l| l.strip_prefix("PAID ")).map(|x| x.split(';').map(|n| n.to_string()).collect()).unwrap_or_default();
    if line && cast0 && !paid.is_empty() {
        let mut want = paid.clone();
        let mut hint = vec![];
        {
            let st = g.raw_state();
            for &r in st.battlefield() {
                let d = st.obj_data(&db, r);
                if d.controller != actor || d.tapped {
                    continue;
                }
                let n = def_name(&db, st, r);
                if let Some(i) = want.iter().position(|w| *w == n) {
                    want.remove(i);
                    hint.push(r);
                }
            }
        }
        g.raw_state_mut().scenario_pay_hint(hint);
    }
    g.apply(id, action_idx).expect("apply action");
    let mut cast_phase = combat.is_none() && cast0;
    // combat run: attackers still to declare, per name; the run ends at the first empty-stack priority of the second main phase
    let mut atk_left: std::collections::BTreeMap<String, u32> = Default::default();
    for n in combat.map(|c| c.trim_start_matches("combat|")).unwrap_or("").split(';').filter(|x| !x.is_empty()) {
        *atk_left.entry(n.to_string()).or_default() += 1;
    }
    let start_turn = g.raw_state().turn_data().turn;
    let debug_events = std::env::var("DIFF_EVENTS").is_ok();
    for _ in 0..600 {
        match g.advance() {
            Status::GameOver(r) => {
                res.over = Some(r);
                break;
            }
            Status::NeedDecision(_) => {}
        }
        for e in g.take_events() {
            if debug_events {
                eprintln!("  event {e:?}");
            }
            if let mtg_core::event::Event::Shuffled { player } = e {
                res.shuffled = true;
                res.shuffled_seats[player.0 as usize] = true;
            }
        }
        let p = g.pending().unwrap().clone();
        let st = g.raw_state();
        if debug_events && std::env::var("DIFF_PERMS").is_ok() {
            let ob = g.observe(p.seat);
            eprintln!("  stack: {:?}", ob.stack.iter().map(|x| format!("{} ability={}", x.name, x.is_ability)).collect::<Vec<_>>());
            eprintln!("  decision {:?} step {:?}: {}", p.kind, ob.step, ob.battlefield.iter().filter(|x| x.power > 0 || x.toughness > 0).map(|x| format!("{} {}/{} dmg{} {}", x.name, x.power, x.toughness, x.damage, if x.attacking { "ATK" } else { "" })).collect::<Vec<_>>().join(", "));
        }
        let idx = match p.kind {
            DecisionKind::Priority if line => break,
            DecisionKind::Priority => {
                if st.stack().is_empty() {
                    if combat.is_none() || st.turn_data().step == mtg_core::types::Step::Main2 || st.turn_data().turn != start_turn {
                        break;
                    }
                }
                let pass = p.options.iter().position(|o| matches!(o, Opt::Pass)).unwrap_or(0);
                if !responded && p.seat != actor {
                    // the opponent's first window with the action on the stack: play the position's response, chosen by canonical key
                    responded = true;
                    if let Ok((acts, _)) = window_keys(g) {
                        res.window_actions = acts;
                    }
                    match window_keys(g).ok().and_then(|(_, keys)| keys.iter().position(|k| k.as_deref() == response)) {
                        Some(i) => {
                            cast_phase = true;
                            i
                        }
                        None => {
                            res.response_missing = true;
                            res.unmatched.push(format!("response {response:?} not offered by the engine at its window"));
                            pass
                        }
                    }
                } else {
                    cast_phase = false;
                    pass
                }
            }
            DecisionKind::ChooseTarget { .. } if cast_phase => {
                let mut best: Option<(String, usize)> = None;
                for (i, o) in p.options.iter().enumerate() {
                    if let Some(k) = target_key(&db, st, p.seat, o) {
                        if best.as_ref().map_or(true, |b| k < b.0) {
                            best = Some((k, i));
                        }
                    }
                }
                best.map(|b| b.1).unwrap_or(0)
            }
            DecisionKind::ChooseTarget { slot } => {
                if slot == 0 {
                    f.cur_targets_set = None;
                }
                f.follow_target(g, &p, slot)
            }
            DecisionKind::ChooseCards { purpose: CardsPurpose::PayCost, .. } if cast_phase => {
                let mut best: Option<(String, usize)> = None;
                for (i, o) in p.options.iter().enumerate() {
                    if let Opt::Card(r) = o {
                        let n = def_name(&db, st, *r);
                        if best.as_ref().map_or(true, |b| n < b.0) {
                            best = Some((n, i));
                        }
                    }
                }
                best.map(|b| b.1).unwrap_or(0)
            }
            DecisionKind::ChooseCards { purpose, remaining } => f.follow_cards(g, &p, purpose, remaining),
            DecisionKind::PayPhyrexian { .. } if cast_phase && f.phy_life.is_some() => {
                // Forge paid `n` life for the cast: 2 life per symbol, the first symbols in order
                let rem = f.phy_life.unwrap();
                let yes = rem >= 2;
                f.phy_life = Some(if yes { rem - 2 } else { 0 });
                p.options.iter().position(|o| if yes { matches!(o, Opt::Yes) } else { matches!(o, Opt::No) }).unwrap_or(0)
            }
            DecisionKind::May | DecisionKind::PayUnless | DecisionKind::ChangeTargets | DecisionKind::PayPhyrexian { .. } => match (if matches!(p.kind, DecisionKind::PayUnless) { f.take_for(p.seat.0 as usize, &["payCostToPreventEffect"]) } else { None }).or_else(|| f.take_for(p.seat.0 as usize, BIN_METHODS)) {
                Some(i) => {
                    let yes = f.entries[i].ret.trim() == "true";
                    p.options.iter().position(|o| if yes { matches!(o, Opt::Yes) } else { matches!(o, Opt::No) }).unwrap_or(0)
                }
                None => {
                    // "you may search": Forge's AI answers by choosing a library card, with no separate confirmation call
                    let seat = p.seat.0 as usize;
                    let search = matches!(p.kind, DecisionKind::May)
                        && f.entries.iter().any(|e| !e.used && e.player == seat && e.method == "chooseSingleCardForZoneChange" && e.args.iter().any(|a| a.contains("[Library]")));
                    if search {
                        p.options.iter().position(|o| matches!(o, Opt::Yes)).unwrap_or(0)
                    } else {
                        f.unmatched.push(format!("{:?}: no logged yes/no", p.kind));
                        0
                    }
                }
            },
            DecisionKind::ChooseX | DecisionKind::ChooseReplicate | DecisionKind::PayEnergyAmount => {
                let want = f.x_value.take().or_else(|| f.take(&["chooseNumber", "announceRequirements"]).and_then(|i| f.entries[i].ret.trim().parse().ok()));
                match want.and_then(|w| p.options.iter().position(|o| matches!(o, Opt::Number(n) if *n == w))) {
                    Some(i) => i,
                    None => {
                        f.unmatched.push(format!("{:?}: no logged number ({want:?})", p.kind));
                        p.options.len() - 1
                    }
                }
            }
            DecisionKind::ChooseDungeon => {
                let names: Vec<&str> = db.dungeons().iter().map(|&d| db.def(d).name.as_str()).collect();
                let want = f.take_for(p.seat.0 as usize, &["chooseSingleCardFace"]).map(|i| strip_id(&f.entries[i].ret));
                match want.clone().and_then(|w| names.iter().position(|n| *n == w)).and_then(|k| p.options.iter().position(|o| matches!(o, Opt::Choice(c) if *c as usize == k))) {
                    Some(i) => i,
                    None => {
                        f.unmatched.push(format!("ChooseDungeon: no logged dungeon ({want:?} among {names:?})"));
                        0
                    }
                }
            }
            DecisionKind::ChooseName { .. } | DecisionKind::ChooseType => {
                let methods: &[&str] = if matches!(p.kind, DecisionKind::ChooseType) { &["chooseSomeType"] } else { &["chooseCardName"] };
                let want = f.take_for(p.seat.0 as usize, methods).map(|i| strip_id(&f.entries[i].ret));
                match want.clone().and_then(|w| p.options.iter().position(|o| otext(&db, g.raw_state(), o) == w)) {
                    Some(i) => i,
                    None => {
                        f.unmatched.push(format!("{:?}: no logged name ({want:?})", p.kind));
                        0
                    }
                }
            }
            DecisionKind::LegendRule | DecisionKind::OrderTriggers if line => {
                // identical names: Forge's log cannot say which copy it kept; likewise the order Forge's AI puts simultaneous triggers in is not
                // logged. The line driver retries with the other picks (see `linediff`).
                let mut l = LEGEND.lock().unwrap();
                let c = l.counter;
                let pick = l.picks.get(c).copied().unwrap_or(0).min(p.options.len() - 1);
                if l.options.len() <= c {
                    l.options.push(p.options.len());
                } else {
                    l.options[c] = p.options.len();
                }
                l.counter += 1;
                pick
            }
            DecisionKind::LegendRule => {
                f.unmatched.push("LegendRule: indistinguishable copies".into());
                0
            }
            // modes: the first legal one only (Forge's probe takes the minimum count), so decline further escalated modes
            DecisionKind::ChooseMode { chosen, .. } => if chosen >= 1 { p.options.iter().position(|o| matches!(o, Opt::Done)).unwrap_or(0) } else { 0 },
            DecisionKind::OrderTriggers => 0,
            DecisionKind::DeclareAttacker { creature } => {
                let name = creature_label(&db, st, creature);
                if debug_events {
                    for pm in g.observe(p.seat).battlefield.iter() {
                        eprintln!("  perm {} {}/{} {:?}", pm.name, pm.power, pm.toughness, pm.keywords);
                    }
                }
                let atk = p.options.iter().position(|o| matches!(o, Opt::Attack(mtg_core::decision::AttackTarget::Player(_))));
                let no = p.options.iter().position(|o| matches!(o, Opt::NoAttack));
                match (atk_left.get_mut(&name), atk) {
                    (Some(k), Some(a)) if *k > 0 => {
                        *k -= 1;
                        a
                    }
                    _ => no.unwrap_or_else(|| {
                        f.unmatched.push(format!("DeclareAttacker {name}: must attack but not in the position's attackers"));
                        0
                    }),
                }
            }
            DecisionKind::DeclareBlocker { creature } => {
                let name = creature_label(&db, st, creature);
                let seat = p.seat.0 as usize;
                let no = p.options.iter().position(|o| matches!(o, Opt::NoBlock));
                match f.entries.iter().position(|e| !e.used && e.player == seat && e.method == "blockAssign" && e.args[0] == name) {
                    Some(i) => {
                        f.entries[i].used = true;
                        let att = f.entries[i].ret.clone();
                        // same-labelled attackers are interchangeable, but two blockers Forge sent at different ones must not pile onto one:
                        // take the matching attacker that has the fewest blockers so far
                        let ob = g.observe(p.seat);
                        let blockers_on = |a: ObjRef| {
                            let v = st.view_id(a, p.seat);
                            ob.battlefield.iter().filter(|x| x.blocking == Some(v)).count()
                        };
                        let pick = p
                            .options
                            .iter()
                            .enumerate()
                            .filter_map(|(i, o)| match o {
                                Opt::Block(a) if creature_label(&db, st, *a) == att => Some((blockers_on(*a), i)),
                                _ => None,
                            })
                            .min();
                        match pick.map(|x| x.1) {
                            Some(k) => k,
                            None => {
                                f.unmatched.push(format!("DeclareBlocker {name}: Forge blocked {att}, not offered"));
                                no.unwrap_or(0)
                            }
                        }
                    }
                    None => no.unwrap_or_else(|| {
                        f.unmatched.push(format!("DeclareBlocker {name}: must block"));
                        0
                    }),
                }
            }
            DecisionKind::AssignDamage { attacker, blocker, .. } => {
                let (an, bn) = (def_name(&db, st, attacker), def_name(&db, st, blocker));
                let ent = f.entries.iter().position(|e| !e.used && e.method == "assignCombatDamage" && e.args[0] == an);
                let want = ent.and_then(|i| {
                    // ret is `{Blocker (id)=n;Blocker (id)=n}`; take this blocker's share
                    let inner = f.entries[i].ret.trim().trim_start_matches('{').trim_end_matches('}').to_string();
                    let mut v: Vec<(String, u16)> = inner.split(';').filter_map(|kv| kv.rsplit_once('=').map(|(k, n)| (strip_id(k), n.trim().parse().unwrap_or(0)))).collect();
                    let pos = v.iter().position(|(k, _)| *k == bn)?;
                    let n = v.remove(pos).1;
                    if v.is_empty() {
                        f.entries[i].used = true;
                    } else {
                        f.entries[i].ret = format!("{{{}}}", v.iter().map(|(k, n)| format!("{k}={n}")).collect::<Vec<_>>().join(";"));
                    }
                    Some(n)
                });
                match want.and_then(|w| p.options.iter().position(|o| matches!(o, Opt::Number(n) if *n as u16 == w))) {
                    Some(i) => i,
                    None => {
                        f.unmatched.push(format!("AssignDamage {an}->{bn}: no logged assignment ({want:?})"));
                        p.options.len() - 1
                    }
                }
            }
            _ => {
                f.unmatched.push(format!("{:?}: defaulted", p.kind));
                0
            }
        };
        res.trace.push(format!("{:?} -> {}", p.kind, otext(&db, g.raw_state(), &p.options[idx])));
        g.apply(p.id, idx).expect("apply decision");
        res.steps += 1;
    }
    for e in g.take_events() {
        if let mtg_core::event::Event::Shuffled { player } = e {
            res.shuffled = true;
            res.shuffled_seats[player.0 as usize] = true;
        }
    }
    if line {
        g.raw_state_mut().scenario_pay_hint(vec![]);
    }
    // decisions Forge made that the engine never asked for: the follower is not aligned with this card's decision structure
    let entries_snapshot = f.entries.clone();
    for e in &f.entries {
        if line && ignorable_leftover(e, &entries_snapshot) {
            continue;
        }
        if !e.used && (e.method == "payCostToPreventEffect" || CARD_METHODS.contains(&e.method.as_str()) || BIN_METHODS.contains(&e.method.as_str()) || ["arrangeForScry", "arrangeForSurveil", "orderMoveToZoneList", "chooseSingleCardFace", "blockAssign"].contains(&e.method.as_str())) {
            res.leftover.push(format!("{} {}", e.method, e.ret.chars().take(60).collect::<String>()));
        }
    }
    res.unmatched = f.unmatched;
    res
}

/// Plays `action_idx` and follows the canonical cast-phase choices until the opponent first has priority with the action on the stack.
/// `None` if the action resolves without such a window, or needs a choice the canonical rules do not cover.
pub fn advance_to_response(g: &mut Game, action_idx: usize) -> Option<()> {
    let db = g.db().clone();
    let actor = g.pending()?.seat;
    let id = g.pending()?.id;
    g.apply(id, action_idx).ok()?;
    for _ in 0..60 {
        if matches!(g.advance(), Status::GameOver(_)) {
            return None;
        }
        g.take_events();
        let p = g.pending()?.clone();
        let st = g.raw_state();
        let idx = match p.kind {
            DecisionKind::Priority => {
                if st.stack().is_empty() {
                    return None;
                }
                if p.seat != actor {
                    return Some(());
                }
                p.options.iter().position(|o| matches!(o, Opt::Pass))?
            }
            DecisionKind::ChooseTarget { .. } => {
                let mut best: Option<(String, usize)> = None;
                for (i, o) in p.options.iter().enumerate() {
                    if let Some(k) = target_key(&db, st, p.seat, o) {
                        if best.as_ref().map_or(true, |b| k < b.0) {
                            best = Some((k, i));
                        }
                    }
                }
                best?.1
            }
            DecisionKind::ChooseCards { purpose: CardsPurpose::PayCost, .. } => {
                let mut best: Option<(String, usize)> = None;
                for (i, o) in p.options.iter().enumerate() {
                    if let Opt::Card(r) = o {
                        let n = def_name(&db, st, *r);
                        if best.as_ref().map_or(true, |b| n < b.0) {
                            best = Some((n, i));
                        }
                    }
                }
                best?.1
            }
            DecisionKind::ChooseMode { chosen, .. } => if chosen >= 1 { p.options.iter().position(|o| matches!(o, Opt::Done))? } else { 0 },
            _ => return None,
        };
        g.apply(p.id, idx).ok()?;
    }
    None
}


/// Splits a logged "[A;B;C]" list (names may carry `{..}` or `(id)` decorations) into its entries.
fn list_items(a: &str) -> Vec<String> {
    let t = a.trim();
    match t.strip_prefix('[').and_then(|x| x.strip_suffix(']')) {
        Some(inner) => {
            let mut v = vec![];
            let (mut depth, mut cur) = (0i32, String::new());
            for c in inner.chars() {
                match c {
                    '{' | '(' => { depth += 1; cur.push(c) }
                    '}' | ')' => { depth -= 1; cur.push(c) }
                    ';' if depth == 0 => { v.push(strip_id(&cur)); cur.clear() }
                    _ => cur.push(c),
                }
            }
            if !cur.trim().is_empty() {
                v.push(strip_id(&cur));
            }
            v
        }
        None => if t.is_empty() || t == "null" { vec![] } else { vec![strip_id(t)] },
    }
}

/// Which argument of a logged choice holds its candidate list.
fn cand_arg(method: &str) -> Option<usize> {
    match method {
        "chooseSingleEntityForEffect" | "chooseEntitiesForEffect" | "chooseCardsForEffect" | "chooseCardsToDiscardFrom" => Some(0),
        "chooseSingleCardForZoneChange" | "chooseCardsForZoneChange" => Some(3),
        _ => None,
    }
}

/// Forge decisions the engine has no reason to ask about, so an unused log entry is not a decision-structure difference (line mode only):
/// a forced choice (the candidates are all chosen, or there is a single one), an ordering of identical names or into the graveyard, a duplicate
/// call for a choice that was mirrored, and a series of picks that together take every candidate (Doomsday with too few cards).
fn ignorable_leftover(e: &Entry, all: &[Entry]) -> bool {
    if e.used {
        return false;
    }
    if e.method == "confirmAction" {
        // a "may" Forge asks about before the choice the engine asks directly (search, cast from exile); accepting is implied by the choice that follows
        return e.ret.trim() == "true";
    }
    if e.method == "payCostToPreventEffect" {
        // the engine asks only when the player can pay; Forge's AI logs its refusal even when it cannot
        return e.ret.trim() == "false";
    }
    if names_of(&e.ret).is_empty() && cand_arg(&e.method).is_some() || e.method == "choosePermanentsToSacrifice" && names_of(&e.ret).is_empty() {
        return true;
    }
    if e.method == "orderMoveToZoneList" {
        let items = list_items(&e.args[0]);
        let to_graveyard = e.args.get(1).map_or(false, |a| matches!(a.trim(), "Graveyard" | "Exile"));
        return items.len() <= 1 || to_graveyard || items.iter().all(|x| *x == items[0]);
    }
    let Some(ci) = cand_arg(&e.method) else { return false };
    if e.method == "chooseEntitiesForEffect" {
        // Forge's summary call of a choose-N effect (Stock Up) repeats the picks of the single-choice calls that came first
        let src = e.args.get(4).cloned().unwrap_or_default();
        let mut pool: Vec<String> = all.iter().filter(|u| u.used && u.player == e.player && u.method == "chooseSingleEntityForEffect" && u.args.get(2) == Some(&src)).flat_map(|u| names_of(&u.ret)).collect();
        let mut ok = !names_of(&e.ret).is_empty();
        for n in names_of(&e.ret) {
            match pool.iter().position(|x| *x == n) {
                Some(i) => {
                    pool.remove(i);
                }
                None => ok = false,
            }
        }
        if ok {
            return true;
        }
    }
    let cands = list_items(e.args.get(ci).map(|s| s.as_str()).unwrap_or(""));
    let mut chosen = names_of(&e.ret);
    chosen.sort();
    let mut cs = cands.clone();
    cs.sort();
    if cands.len() <= 1 || (!chosen.is_empty() && cs == chosen) || cands.iter().all(|c| *c == cands[0]) {
        return true;
    }
    // a duplicate of a used call: same player, same candidates, same choice
    if all.iter().any(|u| u.used && u.player == e.player && cand_arg(&u.method).map_or(false, |k| list_items(u.args.get(k).map(|s| s.as_str()).unwrap_or("")) == cands) && { let mut c2 = names_of(&u.ret); c2.sort(); c2 == chosen }) {
        return true;
    }
    // a series for one source whose picks together take every candidate of its first call
    let src = e.args.get(if e.method == "chooseSingleCardForZoneChange" { 2 } else { 1 }).cloned().unwrap_or_default();
    let series: Vec<&Entry> = all.iter().filter(|x| !x.used && x.player == e.player && x.method == e.method && x.args.get(if e.method == "chooseSingleCardForZoneChange" { 2 } else { 1 }) == Some(&src)).collect();
    if series.len() > 1 {
        let first = list_items(series[0].args.get(ci).map(|s| s.as_str()).unwrap_or(""));
        let mut taken: Vec<String> = series.iter().flat_map(|x| names_of(&x.ret)).collect();
        let mut want = first.clone();
        taken.sort();
        want.sort();
        return want == taken;
    }
    false
}
