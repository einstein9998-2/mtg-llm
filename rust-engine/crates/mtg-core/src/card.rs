//! Card definitions and the immutable, `Arc`-shareable card database (doc 01 section 3.3, doc 03).
//!
//! The IR vocabulary lives in `ir`; instruction lists in `compile`. Everything here derives
//! `Hash` so `CardDb::content_hash` pins the exact data a game record was produced against.

pub use crate::ir::*;

use crate::compile::{compile_effect, compile_spell, Code};
use crate::ids::*;
use crate::mana::ManaCost;
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

// ---------------------------------------------------------------------------------------------
// Cards
// ---------------------------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CardDef {
    pub name: String,
    pub cost: Option<ManaCost>,
    pub types: Types,
    pub supertypes: Supertypes,
    pub subtypes: SubtypeSet,
    /// Added to the colors of the cost (color indicators, tokens).
    pub colors: Colors,
    pub pt: Option<(i16, i16)>,
    pub loyalty: Option<u8>,
    pub keywords: Keywords,
    pub abilities: Vec<AbilityDef>,
    pub is_token: bool,
    /// The name of the second face (`back_kind` says how it works) and, on the second face itself,
    /// `is_back` (not a card of its own: never in a deck).
    pub back: Option<String>,
    pub back_kind: Option<FaceKind>,
    pub is_back: bool,
    /// Resolved by `CardDb::link_faces`.
    #[serde(skip)]
    pub back_id: Option<CardDefId>,
    #[serde(skip)]
    pub front_id: Option<CardDefId>,
    /// Dungeon cards: the rooms.
    pub dungeon: Option<DungeonDef>,
    /// Compiled instruction lists, parallel to `abilities` (empty for statics). Built by
    /// `CardDb::add`.
    #[serde(skip)]
    pub code: Vec<Code>,
    /// Indices of the triggered abilities (built by `finish`).
    #[serde(skip)]
    pub trig: Vec<u8>,
}

impl Default for CardDef {
    fn default() -> Self {
        CardDef {
            name: String::new(),
            cost: None,
            types: Types::empty(),
            supertypes: Supertypes::empty(),
            subtypes: SubtypeSet::EMPTY,
            colors: Colors::empty(),
            pt: None,
            loyalty: None,
            keywords: Keywords::empty(),
            abilities: Vec::new(),
            is_token: false,
            back: None,
            back_kind: None,
            is_back: false,
            back_id: None,
            front_id: None,
            dungeon: None,
            code: Vec::new(),
            trig: Vec::new(),
        }
    }
}

impl CardDef {
    fn base(name: &str, types: Types) -> CardDef {
        CardDef { name: name.to_string(), types, ..CardDef::default() }
    }

    pub fn basic_land(name: &str, subtype: u8) -> CardDef {
        let mut c = CardDef::base(name, Types::LAND);
        c.supertypes = Supertypes::BASIC;
        c.subtypes = SubtypeSet::single(subtype);
        c
    }

    pub fn creature(name: &str, cost: &str, p: i16, t: i16, kw: Keywords) -> CardDef {
        let cost = ManaCost::parse(cost);
        let mut c = CardDef::base(name, Types::CREATURE);
        c.cost = Some(cost);
        c.pt = Some((p, t));
        c.keywords = kw;
        c
    }

    pub fn instant(name: &str, cost: &str, targets: Vec<TargetSpec>, effect: Effect) -> CardDef {
        CardDef::spell(name, Types::INSTANT, cost, targets, effect)
    }

    pub fn sorcery(name: &str, cost: &str, targets: Vec<TargetSpec>, effect: Effect) -> CardDef {
        CardDef::spell(name, Types::SORCERY, cost, targets, effect)
    }

    fn spell(name: &str, ty: Types, cost: &str, targets: Vec<TargetSpec>, effect: Effect) -> CardDef {
        let mut c = CardDef::base(name, ty);
        c.cost = Some(ManaCost::parse(cost));
        c.abilities.push(AbilityDef::Spell(SpellDef::single(Body { label: String::new(), targets, effect: Some(effect) })));
        c
    }

    pub fn token(name: &str, p: i16, t: i16, colors: Colors, kw: Keywords) -> CardDef {
        let mut c = CardDef::base(name, Types::CREATURE);
        c.pt = Some((p, t));
        c.colors = colors;
        c.keywords = kw;
        c.is_token = true;
        c
    }

    pub fn mana_value(&self) -> u32 {
        self.cost.map(|c| c.mana_value()).unwrap_or(0)
    }

    pub fn is_permanent_type(&self) -> bool {
        self.types.intersects(Types::PERMANENT)
    }

    /// The spell ability (instants, sorceries, and the permanent-spell case has none).
    pub fn spell_def(&self) -> Option<(usize, &SpellDef)> {
        self.abilities.iter().enumerate().find_map(|(i, a)| if let AbilityDef::Spell(s) = a { Some((i, s)) } else { None })
    }

    pub fn activated(&self, idx: u8) -> Option<&ActivatedDef> {
        match self.abilities.get(idx as usize) {
            Some(AbilityDef::Activated(a)) => Some(a),
            _ => None,
        }
    }

    /// Instructions of ability `idx`.
    pub fn code_of(&self, idx: u8) -> &[crate::compile::Instr] {
        self.code.get(idx as usize).map(|c| c.as_slice()).unwrap_or(&[])
    }

    /// Validates and compiles; also derives colors from the mana cost.
    fn finish(&mut self) -> Result<(), String> {
        if let Some(c) = self.cost {
            self.colors |= c.colors();
        }
        self.code.clear();
        if self.keywords.contains(Keywords::PROWESS) {
            self.abilities.push(AbilityDef::Triggered(TriggeredDef {
                text: "Prowess (whenever you cast a noncreature spell, this creature gets +1/+1 until end of turn)".to_string(),
                on: EventPat::SpellCast { caster: Rel::You, filter: ObjFilter { types_not: Types::CREATURE, ..ObjFilter::default() }, nth: None },
                zone: ZoneKind::Battlefield,
                once_per_turn: false,
                batch: false,
                cond: None,
                event_cond: false,
                targets: Vec::new(),
                effect: Some(Effect::Continuous { objs: Objs::One(ORef::This), layer: Layer::L7c, effect: ContEffect::ModifyPT(1, 1), until: Until::EndOfTurn }),
            }));
        }
        self.trig = self.abilities.iter().enumerate().filter(|(_, a)| matches!(a, AbilityDef::Triggered(_))).map(|(i, _)| i as u8).collect();
        for a in self.abilities.iter_mut() {
            if let AbilityDef::Spell(s) = a {
                s.normalize();
            }
        }
        for a in &self.abilities {
            let code = match a {
                AbilityDef::Spell(s) => {
                    if s.modes.is_empty() || s.choose.0 > s.choose.1 || s.choose.1 as usize > s.modes.len() {
                        return Err(format!("{}: bad modal spell definition", self.name));
                    }
                    check_targets(s.modes.iter().map(|m| (m.effect.as_ref(), m.targets.len())), &self.name)?;
                    compile_spell(s)?
                }
                AbilityDef::Activated(ad) => {
                    check_targets(std::iter::once((ad.effect.as_ref(), ad.targets.len())), &self.name)?;
                    match &ad.effect {
                        Some(e) => compile_effect(e)?,
                        None => Vec::new(),
                    }
                }
                AbilityDef::Triggered(t) => {
                    check_targets(std::iter::once((t.effect.as_ref(), t.targets.len())), &self.name)?;
                    match &t.effect {
                        Some(e) => compile_effect(e)?,
                        None => Vec::new(),
                    }
                }
                AbilityDef::Static(_) | AbilityDef::Alt(_) | AbilityDef::AddCost(_) | AbilityDef::Escalate(_) | AbilityDef::CostMod(_) | AbilityDef::Enters(_) | AbilityDef::Delve | AbilityDef::Replicate(_) | AbilityDef::Restrict(_) | AbilityDef::Permit(_) | AbilityDef::EntersChoice(_) | AbilityDef::Ascend | AbilityDef::DrawPlusOne | AbilityDef::EmptyDrawWins | AbilityDef::NoHandLimit => Vec::new(),
            };
            self.code.push(code);
        }
        Ok(())
    }
}

/// Lint: every `Target(n)` mentioned by an effect must be declared by its mode/ability.
fn check_targets<'a>(bodies: impl Iterator<Item = (Option<&'a Effect>, usize)>, name: &str) -> Result<(), String> {
    for (e, declared) in bodies {
        if let Some(e) = e {
            let used = crate::lint::max_target_used(e);
            if used >= declared as i32 {
                return Err(format!("{name}: effect uses target slot {used} but only {declared} declared"));
            }
        }
    }
    Ok(())
}

/// The card database: immutable, shared by all games and forks.
#[derive(Clone, Debug, Default)]
pub struct CardDb {
    pub defs: Vec<CardDef>,
    names: BTreeMap<String, CardDefId>,
    /// Bit z set if some triggered ability listens from zone z (`ZoneKind as u8`).
    pub trig_zones: u8,
    /// Keys of `Permit` abilities in the order first seen (virtual way = 128 + index).
    pub permit_keys: Vec<String>,
    /// Some card has a `Restrict` ability (lets hot paths skip the scan).
    pub has_restrict: bool,
    /// Some card has Ascend.
    pub has_ascend: bool,
    /// The dungeon cards, in definition order (the choices of a fresh venture).
    pub dungeons: Vec<CardDefId>,
}

impl CardDb {
    pub fn new() -> CardDb {
        CardDb::default()
    }

    pub fn try_add(&mut self, mut def: CardDef) -> Result<CardDefId, String> {
        if self.names.contains_key(&def.name) {
            return Err(format!("duplicate card name {}", def.name));
        }
        def.finish()?;
        for a in &def.abilities {
            match a {
                AbilityDef::Triggered(t) => self.trig_zones |= 1 << (t.zone as u8),
                AbilityDef::Permit(p) => {
                    if !self.permit_keys.contains(&p.key) {
                        self.permit_keys.push(p.key.clone());
                    }
                }
                AbilityDef::Restrict(_) => self.has_restrict = true,
                AbilityDef::Ascend => self.has_ascend = true,
                _ => {}
            }
        }
        let id = CardDefId(self.defs.len() as u16);
        if def.dungeon.is_some() {
            self.dungeons.push(id);
        }
        self.names.insert(def.name.clone(), id);
        self.defs.push(def);
        Ok(id)
    }

    pub fn add(&mut self, def: CardDef) -> CardDefId {
        self.try_add(def).unwrap_or_else(|e| panic!("{e}"))
    }

    /// The virtual cast way of a permission key.
    pub fn permit_way(&self, key: &str) -> Option<u8> {
        self.permit_keys.iter().position(|k| k == key).map(|i| 128 + i as u8)
    }

    pub fn dungeons(&self) -> &[CardDefId] {
        &self.dungeons
    }

    pub fn id(&self, name: &str) -> Option<CardDefId> {
        self.names.get(name).copied()
    }

    #[inline]
    pub fn def(&self, id: CardDefId) -> &CardDef {
        &self.defs[id.0 as usize]
    }

    /// Resolves the names of second faces to definitions (run once after every card is added).
    pub fn link_faces(&mut self) -> Result<(), String> {
        for i in 0..self.defs.len() {
            if let Some(n) = self.defs[i].back.clone() {
                let b = self.id(&n).ok_or_else(|| format!("{}: unknown back face {n:?}", self.defs[i].name))?;
                if !self.defs[b.0 as usize].is_back {
                    return Err(format!("{}: back face {n:?} is not marked is_back", self.defs[i].name));
                }
                self.defs[i].back_id = Some(b);
                self.defs[b.0 as usize].front_id = Some(CardDefId(i as u16));
            }
        }
        Ok(())
    }

    /// Cross-card checks run once after loading: referenced tokens exist.
    pub fn validate(&self) -> Result<(), String> {
        for d in &self.defs {
            for a in &d.abilities {
                let mut toks: Vec<&str> = Vec::new();
                match a {
                    AbilityDef::Spell(s) => s.modes.iter().filter_map(|m| m.effect.as_ref()).for_each(|e| crate::lint::tokens_used(e, &mut toks)),
                    AbilityDef::Activated(ad) => ad.effect.iter().for_each(|e| crate::lint::tokens_used(e, &mut toks)),
                    AbilityDef::Triggered(t) => t.effect.iter().for_each(|e| crate::lint::tokens_used(e, &mut toks)),
                    AbilityDef::Static(_) | AbilityDef::Alt(_) | AbilityDef::AddCost(_) | AbilityDef::Escalate(_) | AbilityDef::CostMod(_) | AbilityDef::Enters(_) | AbilityDef::Delve | AbilityDef::Replicate(_) | AbilityDef::Restrict(_) | AbilityDef::Permit(_) | AbilityDef::EntersChoice(_) | AbilityDef::Ascend | AbilityDef::DrawPlusOne | AbilityDef::EmptyDrawWins | AbilityDef::NoHandLimit => {}
                }
                for t in toks {
                    match self.id(t) {
                        Some(id) if self.def(id).is_token => {}
                        _ => return Err(format!("{}: unknown token {t:?}", d.name)),
                    }
                }
            }
        }
        Ok(())
    }

    /// Stable content hash, stored in every game record (doc 01 section 12).
    pub fn content_hash(&self) -> u64 {
        let mut h = crate::hash::Fx64::default();
        self.defs.hash(&mut h);
        h.finish()
    }
}
