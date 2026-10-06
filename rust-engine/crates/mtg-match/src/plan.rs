//! Sideboard plans: per (own deck, opposing deck) a list of swaps from the 75-card base list.
//!
//! File format, one file per deck named `<deck>.txt` (same stem as the deck list):
//!
//! ```text
//! # comment
//! vs ur-cutter          # opposing deck stem; `vs *` is the fallback
//! vs ur-cutter on-draw  # optional: only when this seat is on the draw (`on-play` likewise);
//!                       # the more specific section wins over the plain one
//! -4 Stock Up           # N cards of the main deck go to the sideboard
//! +3 Carpet of Flowers  # N cards of the sideboard come in
//! ```
//!
//! Plans are always applied to the base 75, never stacked on a previous game's deck, so game 3
//! uses the same list as game 2 (a player may change plans between games; that is for later).

use mtg_core::card::CardDb;
use mtg_core::types::Supertypes;
use mtg_core::ids::CardDefId;
use mtg_view::DeckList;
use std::collections::BTreeMap;
use std::path::Path;

/// Cards swapped for one matchup.
#[derive(Clone, Debug, Default)]
pub struct Plan {
    pub out: Vec<CardDefId>,
    pub inn: Vec<CardDefId>,
}

/// All plans of a directory: own deck stem -> (opposing deck stem or `*`) -> plan.
#[derive(Clone, Debug, Default)]
pub struct PlanBook {
    plans: BTreeMap<String, BTreeMap<(String, Side), Plan>>,
}

/// Which games a plan section applies to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Side {
    Any,
    OnPlay,
    OnDraw,
}

impl PlanBook {
    pub fn is_empty(&self) -> bool {
        self.plans.is_empty()
    }

    /// The plan that applies whichever side the seat is on (play/draw sections are ignored).
    pub fn plan(&self, own: &str, opp: &str) -> Option<&Plan> {
        self.plan_for(own, opp, None)
    }

    /// The plan for a game in which this seat is on the play (`Some(true)`), on the draw
    /// (`Some(false)`) or unknown (`None`): the matching play/draw section, else the plain one,
    /// else `vs *`.
    pub fn plan_for(&self, own: &str, opp: &str, on_play: Option<bool>) -> Option<&Plan> {
        let m = self.plans.get(own)?;
        let side = on_play.map(|p| if p { Side::OnPlay } else { Side::OnDraw });
        side.and_then(|s| m.get(&(opp.to_string(), s)))
            .or_else(|| m.get(&(opp.to_string(), Side::Any)))
            .or_else(|| side.and_then(|s| m.get(&("*".to_string(), s))))
            .or_else(|| m.get(&("*".to_string(), Side::Any)))
    }

    /// Every (own, opposing) pair that has an explicit plan (play/draw sections count once).
    pub fn pairs(&self) -> Vec<(String, String)> {
        let mut v: Vec<(String, String)> = self.plans.iter().flat_map(|(a, m)| m.keys().map(move |(b, _)| (a.clone(), b.clone()))).collect();
        v.dedup();
        v
    }

    /// Every explicit section: (own, opposing, side, plan).
    pub fn entries(&self) -> Vec<(String, String, Side, &Plan)> {
        self.plans.iter().flat_map(|(a, m)| m.iter().map(move |((b, side), p)| (a.clone(), b.clone(), *side, p))).collect()
    }

    /// Reads every `*.txt` in `dir`. Unknown card names and malformed lines are errors.
    pub fn load_dir(db: &CardDb, dir: &Path) -> Result<PlanBook, String> {
        let mut book = PlanBook::default();
        let mut paths: Vec<_> = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().map_or(false, |x| x == "txt")).collect();
        paths.sort();
        for p in paths {
            let stem = p.file_stem().unwrap().to_string_lossy().to_string();
            let text = std::fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
            let plans = parse_plans(db, &text).map_err(|e| format!("{}: {e}", p.display()))?;
            book.plans.insert(stem, plans);
        }
        Ok(book)
    }
}

fn parse_plans(db: &CardDb, text: &str) -> Result<BTreeMap<(String, Side), Plan>, String> {
    let mut out: BTreeMap<(String, Side), Plan> = BTreeMap::new();
    let mut cur: Option<(String, Side)> = None;
    for (ln, raw) in text.lines().enumerate() {
        let l = raw.split('#').next().unwrap().trim();
        if l.is_empty() {
            continue;
        }
        let err = |m: &str| format!("line {}: {m}: `{raw}`", ln + 1);
        if let Some(rest) = l.strip_prefix("vs ") {
            let mut words: Vec<&str> = rest.split_whitespace().collect();
            let side = match words.last().copied() {
                Some("on-play") => Side::OnPlay,
                Some("on-draw") => Side::OnDraw,
                _ => Side::Any,
            };
            if side != Side::Any {
                words.pop();
            }
            if words.len() != 1 {
                return Err(err("expected `vs <deck> [on-play|on-draw]`"));
            }
            let key = (words[0].to_string(), side);
            if out.contains_key(&key) {
                return Err(err("repeated `vs` section"));
            }
            out.insert(key.clone(), Plan::default());
            cur = Some(key);
            continue;
        }
        let (sign, rest) = l.split_at(1);
        let (n, name) = rest.trim().split_once(' ').ok_or_else(|| err("expected `+N Card` or `-N Card`"))?;
        let n: usize = n.parse().map_err(|_| err("bad count"))?;
        let id = db.id(name.trim()).ok_or_else(|| err("unknown card"))?;
        let plan = out.get_mut(cur.as_ref().ok_or_else(|| err("swap before any `vs` line"))?).unwrap();
        match sign {
            "-" => plan.out.extend(std::iter::repeat(id).take(n)),
            "+" => plan.inn.extend(std::iter::repeat(id).take(n)),
            _ => return Err(err("line must start with + or -")),
        }
    }
    Ok(out)
}

fn take(list: &mut Vec<CardDefId>, id: CardDefId) -> bool {
    match list.iter().position(|&c| c == id) {
        Some(i) => {
            list.remove(i);
            true
        }
        None => false,
    }
}

/// Applies `plan` to the base 75. Errors if a card to take out is not in the main deck, a card to
/// bring in is not in the sideboard, the swap is not one for one, or a deck would break the
/// constructed rules (60 main cards, at most 15 sideboard cards, at most 4 copies of a non-basic).
pub fn board(db: &CardDb, base: &DeckList, plan: &Plan) -> Result<DeckList, String> {
    let name = |id: CardDefId| db.def(id).name.clone();
    if plan.out.len() != plan.inn.len() {
        return Err(format!("plan takes out {} cards and brings in {}", plan.out.len(), plan.inn.len()));
    }
    let mut d = base.clone();
    for &c in &plan.out {
        if !take(&mut d.main, c) {
            return Err(format!("cannot take out {}: not (enough copies) in the main deck", name(c)));
        }
        d.side.push(c);
    }
    for &c in &plan.inn {
        if !take(&mut d.side, c) {
            return Err(format!("cannot bring in {}: not (enough copies) in the sideboard", name(c)));
        }
        d.main.push(c);
    }
    check_legal(db, &d)?;
    Ok(d)
}

/// 60+ main cards (the base deck's size is kept, so this only checks it did not change), at most
/// 15 sideboard cards, at most four copies of a card across main and side unless it is basic.
pub fn check_legal(db: &CardDb, d: &DeckList) -> Result<(), String> {
    if d.main.len() < 60 {
        return Err(format!("main deck has {} cards", d.main.len()));
    }
    if d.side.len() > 15 {
        return Err(format!("sideboard has {} cards", d.side.len()));
    }
    let mut n: BTreeMap<CardDefId, usize> = BTreeMap::new();
    for &c in d.main.iter().chain(d.side.iter()) {
        *n.entry(c).or_default() += 1;
    }
    for (c, k) in n {
        let def = db.def(c);
        if k > 4 && !def.supertypes.contains(Supertypes::BASIC) {
            return Err(format!("{k} copies of {}", def.name));
        }
    }
    Ok(())
}
