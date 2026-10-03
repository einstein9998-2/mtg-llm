//! Primitive state changes. Every change flows through `ProposedEvent` -> replacement pipeline ->
//! commit -> `Event` (doc 01 section 7.1). In M1 the pipeline has no registered replacement
//! effects, but the hook (`propose`) is the single place they will plug in (doc 01 section 9).

use crate::decision::Target;
use crate::cx::Cx;
use crate::event::*;
use crate::ids::*;
use crate::ir::AbilityDef;
use crate::state::*;
use crate::types::*;

#[derive(Copy, Clone, Debug, Default)]
pub struct MoveOpts {
    /// Controller on the battlefield (default: owner).
    pub controller: Option<Seat>,
    pub tapped: bool,
    /// Library destination: true = bottom.
    pub to_bottom: bool,
    /// Suppress the generic `ZoneChange` event (draws emit `Drew` instead).
    pub quiet: bool,
    /// The object was cast (it is a spell resolving), as opposed to put onto the battlefield.
    pub cast: bool,
    /// Enters with this many counters of a kind.
    pub counters: Option<(CounterKind, u16)>,
    /// Instant and sorcery cards exiled with delve to cast the moving spell.
    pub delved: u8,
    /// How the moving spell was cast (recorded on the permanent): zone and way.
    pub cast_from: Option<(ZoneKind, u8)>,
    /// The choice made as the permanent entered (see `AbilityDef::EntersChoice`).
    pub chosen: u16,
    /// Delayed (next end step) ability to register as the permanent enters.
    pub end_step_ability: Option<u8>,
    /// Enter the battlefield or the stack as the card's second face (transformed, a modal
    /// double-faced card's back, an Adventure). Any other move shows the front face again.
    pub face: bool,
}

/// What is about to happen, before replacement effects (doc 01 section 7.1).
#[derive(Clone, Debug)]
pub enum ProposedEvent {
    Move { obj: ObjRef, to: ZoneKind, opts: MoveOpts },
    Draw { player: Seat },
    Damage { source: Option<ObjRef>, to: Target, amount: u32, combat: bool },
    LifeChange { player: Seat, delta: i32 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Event happened; for moves, the new object reference (None if the object ceased to exist).
    Done(Option<ObjRef>),
    Prevented,
}

pub fn zone_visible_to(zone: ZoneKind, owner: Seat, seat: Seat) -> bool {
    match zone {
        ZoneKind::Battlefield | ZoneKind::Stack | ZoneKind::Graveyard | ZoneKind::Exile | ZoneKind::Command => true,
        ZoneKind::Hand => owner == seat,
        ZoneKind::Library | ZoneKind::Gone => false,
    }
}

impl<'a> Cx<'a> {
    /// `seat` gets to see card `r` (it is shown to them): they know what it is and can refer to it.
    /// Reveals the whole hand of `owner` to `seat`: view ids are handed out in definition order,
    /// not hand order, so their numbers say nothing about the order the cards were drawn in.
    pub(crate) fn reveal_hand_to(&mut self, seat: Seat, owner: Seat) {
        let mut hand = self.s.players[owner.idx()].hand.clone();
        hand.sort_by_key(|r| self.s.objs[r.slot as usize].def);
        for r in hand {
            self.reveal_to(seat, r);
        }
    }

    pub(crate) fn reveal_to(&mut self, seat: Seat, r: ObjRef) {
        self.s.knowledge[r.slot as usize].known_to |= 1 << seat.idx();
        let v = &mut self.s.vids[r.slot as usize][seat.idx()];
        if *v == ViewId::NONE {
            *v = ViewId(self.s.next_vid[seat.idx()]);
            self.s.next_vid[seat.idx()] += 1;
        }
    }

    // ---- pipeline ---------------------------------------------------------------------------

    /// Runs a proposed event through the replacement pipeline and commits it.
    pub fn propose(&mut self, pe: ProposedEvent) -> Outcome {
        if let ProposedEvent::Move { obj, to: ZoneKind::Battlefield, opts } = pe {
            let m = self.enters_replacements(obj, opts);
            if m.prevented {
                return Outcome::Prevented;
            }
            return self.commit(ProposedEvent::Move { obj, to: m.to, opts: m.opts });
        }
        self.commit(pe)
    }

    fn commit(&mut self, pe: ProposedEvent) -> Outcome {
        match pe {
            ProposedEvent::Move { obj, to, opts } => Outcome::Done(self.commit_move(obj, to, opts, None)),
            ProposedEvent::Draw { player } => self.commit_draw(player),
            ProposedEvent::Damage { source, to, amount, combat } => self.commit_damage(source, to, amount, combat),
            ProposedEvent::LifeChange { player, delta } => self.commit_life(player, delta),
        }
    }

    // ---- zone changes -----------------------------------------------------------------------

    pub fn move_zone(&mut self, obj: ObjRef, to: ZoneKind, opts: MoveOpts) -> Option<ObjRef> {
        match self.propose(ProposedEvent::Move { obj, to, opts }) {
            Outcome::Done(r) => r,
            Outcome::Prevented => Some(obj),
        }
    }

    /// Last-known information snapshot of a live object (uses the derived cache).
    pub fn lki(&mut self, r: ObjRef) -> Lki {
        self.refresh();
        let o = self.s.obj(r);
        let c = self.chars(r);
        Lki {
            def: o.def,
            owner: o.owner,
            controller: o.controller,
            types: c.types,
            supertypes: c.supertypes,
            subtypes: c.subtypes,
            colors: c.colors,
            power: c.power,
            toughness: c.toughness,
            keywords: c.keywords,
            was_token: o.kind == ObjKind::Token,
            counters: o.counters.to_vec(),
            attached: o.attached_to,
        }
    }

    fn remove_from_zone(&mut self, r: ObjRef, from: ZoneKind, owner: Seat) {
        fn rm(v: &mut Vec<ObjRef>, r: ObjRef) {
            if let Some(pos) = v.iter().rposition(|&x| x == r) {
                v.remove(pos);
            } else {
                debug_assert!(false, "object {r:?} missing from its zone list");
            }
        }
        match from {
            ZoneKind::Library => rm(&mut self.s.players[owner.idx()].library, r),
            ZoneKind::Hand => rm(&mut self.s.players[owner.idx()].hand, r),
            ZoneKind::Graveyard => rm(&mut self.s.players[owner.idx()].graveyard, r),
            ZoneKind::Battlefield => rm(&mut self.s.battlefield, r),
            ZoneKind::Exile => rm(&mut self.s.exile, r),
            ZoneKind::Stack => {
                if let Some(pos) = self.s.stack.iter().rposition(|e| e.obj == r) {
                    self.s.stack.remove(pos);
                }
            }
            ZoneKind::Command | ZoneKind::Gone => {}
        }
    }

    pub(crate) fn commit_move(&mut self, r: ObjRef, to: ZoneKind, opts: MoveOpts, lki: Option<Lki>) -> Option<ObjRef> {
        if !self.s.is_live(r) {
            return None;
        }
        let (from, kind, owner, def) = {
            let o = self.s.obj(r);
            (o.zone, o.kind, o.owner, o.def)
        };
        let lki = match lki {
            Some(l) => Some(l),
            None if from == ZoneKind::Battlefield && self.s.batch.depth > 0 && self.s.batch.lki.iter().any(|(x, _)| *x == r) => self.s.batch.lki.iter().find(|(x, _)| *x == r).map(|(_, l)| l.clone()),
            None if from == ZoneKind::Battlefield || from == ZoneKind::Stack => Some(self.lki(r)),
            None => None,
        };
        if from == ZoneKind::Battlefield && to != ZoneKind::Battlefield {
            // Abilities of this permanent already on the stack look back at it (CR 113.7a).
            if let Some(l) = &lki {
                for e in self.s.stack.iter_mut() {
                    if matches!(e.kind, StackKind::Ability { source, .. } if source == r) && e.cap.lki.is_none() {
                        e.cap.lki = Some(l.clone());
                    }
                }
            }
            let c = self.s.obj(r).controller;
            self.s.players[c.idx()].turn.perm_left = true;
        }
        self.remove_from_zone(r, from, owner);
        if from == ZoneKind::Battlefield || to == ZoneKind::Battlefield || from == ZoneKind::Graveyard || to == ZoneKind::Graveyard {
            // Graveyard contents feed delirium and the */1+* creatures, so they dirty the cache too.
            self.s.derived_dirty = true;
            // Anything attached to this object becomes unattached (SBA handles illegal auras).
        }
        let slot = r.slot as usize;
        let vid_old = self.s.vids[slot];
        let ceases = match kind {
            ObjKind::Card => false,
            ObjKind::Token => to != ZoneKind::Battlefield,
            ObjKind::SpellCopy | ObjKind::Ability => to != ZoneKind::Stack,
        };
        let controller = match to {
            ZoneKind::Battlefield => opts.controller.unwrap_or(owner),
            _ => owner,
        };
        let new_gen = r.gen.wrapping_add(1);
        let ts = self.s.next_ts();
        let turn = self.s.turn.number;
        {
            let o = &mut self.s.objs[slot];
            o.gen = new_gen;
            o.zone = to;
            o.controller = controller;
            o.tapped = to == ZoneKind::Battlefield && opts.tapped;
            o.phased = false;
            o.damage = 0;
            o.dt_damage = false;
            o.counters.clear();
            o.attached_to = None;
            o.timestamp = ts;
            o.entered_turn = turn;
            o.prev_zone = from;
            let cd = self.db.def(o.def);
            if opts.face && matches!(to, ZoneKind::Battlefield | ZoneKind::Stack) {
                if let Some(b) = cd.back_id {
                    o.def = b;
                }
            } else if let Some(f) = cd.front_id {
                o.def = f;
            }
            o.x_value = 0;
            let (cf, cw) = opts.cast_from.unwrap_or((ZoneKind::Gone, 0));
            o.cast_from = cf;
            o.cast_way = cw;
            o.chosen = opts.chosen;
        }
        let nr = ObjRef { slot: r.slot, gen: new_gen };
        if to == ZoneKind::Battlefield {
            if let Some((k, n)) = opts.counters {
                self.s.objs[slot].counters.push((k, n));
            }
        }
        if to == ZoneKind::Battlefield {
            if let Some(ability) = opts.end_step_ability {
                let def = self.s.objs[slot].def;
                self.s.delayed.push(DelayedTrig { when: crate::ir::DelayWhen::NextEndStep, source: nr, def, ability, controller, cap: Captured::default() });
            }
        }
        // Place in the new zone (objects that cease to exist are never listed).
        match if ceases { ZoneKind::Gone } else { to } {
            ZoneKind::Library => {
                let lib = &mut self.s.players[owner.idx()].library;
                if opts.to_bottom {
                    lib.insert(0, nr);
                } else {
                    lib.push(nr);
                }
            }
            ZoneKind::Hand => self.s.players[owner.idx()].hand.push(nr),
            ZoneKind::Graveyard => self.s.players[owner.idx()].graveyard.push(nr),
            ZoneKind::Battlefield => self.s.battlefield.push(nr),
            ZoneKind::Exile => self.s.exile.push(nr),
            ZoneKind::Stack | ZoneKind::Command | ZoneKind::Gone => {}
        }
        // View ids and knowledge (doc 02 sections 4.2, 6).
        let mut vid_new = [ViewId::NONE; 2];
        for v in 0..2usize {
            if !ceases && zone_visible_to(to, owner, Seat(v as u8)) {
                vid_new[v] = ViewId(self.s.next_vid[v]);
                self.s.next_vid[v] += 1;
                if crate::canary::on(crate::canary::VIEWID_FROM_CARDID) {
                    vid_new[v] = ViewId(100_000 + self.s.objs[slot].card.map(|c| c.0 as u32).unwrap_or(0));
                }
            }
        }
        self.s.vids[slot] = vid_new;
        if kind == ObjKind::Card {
            let k = &mut self.s.knowledge[slot];
            for v in 0..2usize {
                if vid_new[v] != ViewId::NONE {
                    k.known_to |= 1 << v;
                }
            }
            k.pos_known_to = 0;
        }
        if !opts.quiet {
            self.emit(Event::ZoneChange { obj: nr, def, from, to, owner, controller, vid_old, vid_new, lki });
        }
        if to == ZoneKind::Battlefield && !ceases && self.db.def(def).abilities.iter().any(|a| matches!(a, AbilityDef::EntersChoice(_))) {
            self.s.enter_choices.push(nr);
        }
        // Cards exiled "until this leaves the battlefield" come back at once (610.3).
        if from == ZoneKind::Battlefield && to != ZoneKind::Battlefield && !self.s.until_links.is_empty() {
            let (back, keep): (Vec<UntilLink>, Vec<UntilLink>) = std::mem::take(&mut self.s.until_links).into_iter().partition(|l| l.src == r);
            self.s.until_links = keep;
            for l in back {
                if self.s.is_live(l.card) && self.s.obj(l.card).zone == ZoneKind::Exile {
                    self.move_zone(l.card, l.from, MoveOpts::default());
                }
            }
        }
        if ceases {
            self.s.free_slot(nr);
            return None;
        }
        Some(nr)
    }

    /// Puts a copy of the spell on the stack entry `e` onto the stack, controlled by `ctrl`
    /// (CR 707.10). The copy is a spell but not a card; it ceases to exist when it leaves the stack.
    pub(crate) fn make_spell_copy(&mut self, e: &StackEntry, ctrl: Seat) -> ObjRef {
        let nr = self.s.alloc_slot(ObjKind::SpellCopy, e.def, ctrl);
        {
            let o = self.s.obj_mut_raw(nr);
            o.zone = ZoneKind::Stack;
            o.controller = ctrl;
        }
        let mut vid = [ViewId::NONE; 2];
        for v in 0..2 {
            vid[v] = ViewId(self.s.next_vid[v]);
            self.s.next_vid[v] += 1;
        }
        self.s.vids[nr.slot as usize] = vid;
        let mut cast = e.cast;
        cast.replicate = 0;
        self.s.stack.push(StackEntry { obj: nr, controller: ctrl, kind: StackKind::Spell, targets: e.targets.clone(), x: e.x, def: e.def, modes: e.modes, cap: Captured::default(), cast });
        nr
    }

    // ---- drawing, milling, shuffling --------------------------------------------------------

    /// Executes an instruction to draw `n` cards (CR 121.2a): replacement effects such as Quantum
    /// Riddler's change the number before any card is drawn.
    pub fn draw_n(&mut self, p: Seat, n: i32) {
        if n <= 0 {
            return;
        }
        let mut n = n;
        if self.s.players[p.idx()].hand.len() <= 1 {
            self.refresh();
            let plus = self
                .s
                .battlefield
                .iter()
                .filter(|&&r| {
                    let o = self.s.obj(r);
                    o.controller == p && !o.phased && self.db.def(o.def).abilities.iter().any(|a| matches!(a, AbilityDef::DrawPlusOne))
                })
                .count() as i32;
            n += plus;
        }
        for _ in 0..n {
            self.draw(p);
        }
    }

    /// Draws one card. Returns false if the library was empty (the player will lose at SBA time).
    pub fn draw(&mut self, p: Seat) -> bool {
        matches!(self.propose(ProposedEvent::Draw { player: p }), Outcome::Done(Some(_)))
    }

    fn commit_draw(&mut self, p: Seat) -> Outcome {
        let top = self.s.players[p.idx()].library.last().copied();
        match top {
            None => {
                let wins = self.s.battlefield.iter().any(|&r| {
                    let o = self.s.obj(r);
                    o.controller == p && !o.phased && self.db.def(o.def).abilities.iter().any(|a| matches!(a, AbilityDef::EmptyDrawWins))
                });
                if wins {
                    self.s.result = Some(crate::decision::GameResult::Win(p));
                    return Outcome::Done(None);
                }
                self.s.players[p.idx()].drew_from_empty = true;
                self.emit(Event::DrewFromEmpty { player: p });
                Outcome::Done(None)
            }
            Some(r) => {
                let def = self.s.obj(r).def;
                let nr = self.commit_move(r, ZoneKind::Hand, MoveOpts { quiet: true, ..Default::default() }, None).unwrap();
                let vid = self.s.vids[nr.slot as usize];
                self.s.players[p.idx()].turn.cards_drawn = self.s.players[p.idx()].turn.cards_drawn.saturating_add(1);
                if self.s.turn.step == Step::Draw && self.s.turn.active == p {
                    self.s.players[p.idx()].turn.draw_step_draws = self.s.players[p.idx()].turn.draw_step_draws.saturating_add(1);
                }
                // Owner now knows the card; position knowledge ends (doc 02 section 6).
                self.s.knowledge[nr.slot as usize].known_to |= 1 << p.idx();
                self.emit(Event::Drew { player: p, obj: nr, def, vid });
                Outcome::Done(Some(nr))
            }
        }
    }

    /// Mills the top card of `p`'s library into their graveyard. Returns false if empty.
    pub fn mill_one(&mut self, p: Seat) -> bool {
        self.mill_one_ref(p).is_some()
    }

    /// Mills the top card and returns its new reference (None if the library was empty or the
    /// card went elsewhere).
    pub fn mill_one_ref(&mut self, p: Seat) -> Option<ObjRef> {
        let r = self.s.players[p.idx()].library.last().copied()?;
        self.move_zone(r, ZoneKind::Graveyard, MoveOpts::default())
    }

    /// Puts `rest` into a random order (cards about to go to the bottom of `p`'s library); a
    /// scenario can script the outcome (the order, top first).
    pub(crate) fn random_bottom_order(&mut self, p: Seat, rest: &mut Vec<ObjRef>) {
        if let Some(sc) = self.s.scripted.as_mut() {
            match sc.queue.front() {
                Some((seat, RandKind::BottomOrder, _)) if *seat == p => {
                    let (_, _, order) = sc.queue.pop_front().unwrap();
                    let mut pool: Vec<ObjRef> = rest.clone();
                    let mut new: Vec<ObjRef> = Vec::new();
                    for d in &order {
                        match pool.iter().position(|r| self.s.objs[r.slot as usize].def == *d) {
                            Some(i) => new.push(pool.remove(i)),
                            None => break,
                        }
                    }
                    if new.len() == rest.len() {
                        *rest = new;
                        return;
                    }
                    self.s.scripted.as_mut().unwrap().violations += 1;
                }
                _ => sc.violations += 1,
            }
        }
        self.s.rng.shuffle(rest);
    }

    pub fn shuffle_library(&mut self, p: Seat) {
        if let Some(sc) = self.s.scripted.as_mut() {
            match sc.queue.pop_front() {
                Some((seat, RandKind::Shuffle, order)) if seat == p => {
                    let lib = &mut self.s.players[p.idx()].library;
                    let mut pool: Vec<ObjRef> = lib.clone();
                    let mut new: Vec<ObjRef> = Vec::new();
                    let objs = &self.s.objs;
                    let mut ok = order.len() == pool.len();
                    for d in &order {
                        match pool.iter().position(|r| objs[r.slot as usize].def == *d) {
                            Some(i) => new.push(pool.remove(i)),
                            None => {
                                ok = false;
                                break;
                            }
                        }
                    }
                    if ok {
                        new.reverse(); // the library's top is its last element
                        *lib = new;
                    } else {
                        sc.violations += 1;
                        self.s.rng.shuffle(lib);
                    }
                }
                _ => {
                    sc.violations += 1;
                    let lib = &mut self.s.players[p.idx()].library;
                    self.s.rng.shuffle(lib);
                }
            }
        } else {
            let lib = &mut self.s.players[p.idx()].library;
            self.s.rng.shuffle(lib);
        }
        // All position knowledge is lost (doc 02 section 6).
        let slots: Vec<u16> = self.s.players[p.idx()].library.iter().map(|r| r.slot).collect();
        for sl in slots {
            self.s.knowledge[sl as usize].pos_known_to = 0;
        }
        self.emit(Event::Shuffled { player: p });
    }

    // ---- life and damage --------------------------------------------------------------------

    pub fn gain_life(&mut self, p: Seat, n: u32) {
        if n > 0 {
            self.propose(ProposedEvent::LifeChange { player: p, delta: n as i32 });
        }
    }

    pub fn lose_life(&mut self, p: Seat, n: u32) {
        if n > 0 {
            self.propose(ProposedEvent::LifeChange { player: p, delta: -(n as i32) });
        }
    }

    fn commit_life(&mut self, p: Seat, delta: i32) -> Outcome {
        let pl = &mut self.s.players[p.idx()];
        pl.life += delta;
        if delta > 0 {
            pl.turn.life_gained = pl.turn.life_gained.saturating_add(delta as u16);
        } else {
            pl.turn.life_lost = pl.turn.life_lost.saturating_add((-delta) as u16);
        }
        self.emit(Event::LifeChange { player: p, delta });
        Outcome::Done(None)
    }

    pub fn deal_damage(&mut self, source: Option<ObjRef>, to: Target, amount: u32, combat: bool) {
        if amount > 0 {
            self.propose(ProposedEvent::Damage { source, to, amount, combat });
        }
    }

    fn commit_damage(&mut self, source: Option<ObjRef>, to: Target, amount: u32, combat: bool) -> Outcome {
        self.refresh();
        let (sk, sctrl) = match source {
            Some(src) if self.s.is_live(src) => (self.chars(src).keywords, Some(self.s.obj(src).controller)),
            _ => (Keywords::empty(), None),
        };
        match to {
            Target::None => return Outcome::Prevented,
            Target::Player(p) => {
                self.propose(ProposedEvent::LifeChange { player: p, delta: -(amount as i32) });
            }
            Target::Obj(t) => {
                if !self.s.is_live(t) {
                    return Outcome::Prevented;
                }
                let is_creature = self.chars(t).types.contains(Types::CREATURE);
                if self.chars(t).types.contains(Types::PLANESWALKER) {
                    // 120.3c: damage to a planeswalker removes that many loyalty counters.
                    self.add_counters(t, CounterKind::Loyalty, -(amount.min(u16::MAX as u32) as i32));
                }
                let o = self.s.obj_mut(t);
                if is_creature {
                    o.damage = o.damage.saturating_add(amount.min(u16::MAX as u32) as u16);
                    if sk.contains(Keywords::DEATHTOUCH) {
                        o.dt_damage = true;
                    }
                }
            }
        }
        if sk.contains(Keywords::LIFELINK) {
            if let Some(c) = sctrl {
                self.propose(ProposedEvent::LifeChange { player: c, delta: amount as i32 });
            }
        }
        self.emit(Event::Damage { source, to, amount, combat });
        Outcome::Done(None)
    }

    // ---- permanents -------------------------------------------------------------------------

    pub fn tap(&mut self, r: ObjRef) {
        if !self.s.obj(r).tapped {
            self.s.obj_mut(r).tapped = true;
            self.emit(Event::Tapped { obj: r });
        }
    }

    pub fn untap(&mut self, r: ObjRef) {
        if self.s.obj(r).tapped {
            if self.s.obj(r).counter(CounterKind::Stun) > 0 {
                // 122.1d: a stun counter replaces the untap.
                self.add_counters(r, CounterKind::Stun, -1);
                return;
            }
            self.s.obj_mut(r).tapped = false;
            self.emit(Event::Untapped { obj: r });
        }
    }

    pub fn add_counters(&mut self, r: ObjRef, kind: CounterKind, delta: i32) {
        if !self.s.is_live(r) || delta == 0 {
            return;
        }
        let o = self.s.obj_mut(r);
        if let Some(c) = o.counters.iter_mut().find(|c| c.0 == kind) {
            let v = (c.1 as i32 + delta).max(0);
            c.1 = v as u16;
        } else if delta > 0 {
            o.counters.push((kind, delta as u16));
        }
        o.counters.retain(|c| c.1 > 0);
        self.s.derived_dirty = true;
        self.emit(Event::CountersChanged { obj: r, kind, delta });
    }

    pub fn create_token(&mut self, def: CardDefId, owner: Seat) -> Option<ObjRef> {
        self.create_token_with(def, owner, true)
    }

    /// `replacements`: apply enters-the-battlefield replacements (false for scenario setup).
    pub fn create_token_with(&mut self, def: CardDefId, owner: Seat, replacements: bool) -> Option<ObjRef> {
        // A token engine that doubles forever (Ocelot Pride's copies under random play) would
        // exhaust the 16-bit object arena. Past `RUNAWAY_OBJECTS` live objects the game is declared
        // a draw instead of crashing: no real game gets near it (known deviation).
        if self.s.result.is_some() || self.s.live_objects() >= crate::state::RUNAWAY_OBJECTS {
            self.s.result.get_or_insert(crate::decision::GameResult::Draw);
            return None;
        }
        let r = self.s.alloc_slot(ObjKind::Token, def, owner);
        // alloc_slot leaves the token in `Gone`; it enters the battlefield directly.
        let m = if replacements {
            self.enters_replacements(r, MoveOpts::default())
        } else {
            crate::replace::EnterMods { to: ZoneKind::Battlefield, opts: MoveOpts::default(), prevented: false }
        };
        if m.prevented || m.to != ZoneKind::Battlefield {
            // Replaced by "exile it instead" or prevented: a token that would be exiled ceases to exist.
            self.s.free_slot(r);
            return None;
        }
        let nr = self.commit_move_token_enter(r, owner, m.opts);
        self.emit(Event::TokenCreated { obj: nr, def, controller: owner });
        Some(nr)
    }

    fn commit_move_token_enter(&mut self, r: ObjRef, owner: Seat, opts: MoveOpts) -> ObjRef {
        let slot = r.slot as usize;
        let ts = self.s.next_ts();
        let turn = self.s.turn.number;
        let new_gen = r.gen;
        {
            let o = &mut self.s.objs[slot];
            o.zone = ZoneKind::Battlefield;
            o.controller = owner;
            o.tapped = opts.tapped;
            if let Some((k, n)) = opts.counters {
                o.counters.push((k, n));
            }
            o.timestamp = ts;
            o.entered_turn = turn;
        }
        let nr = ObjRef { slot: r.slot, gen: new_gen };
        self.s.battlefield.push(nr);
        self.s.derived_dirty = true;
        let mut vid = [ViewId::NONE; 2];
        for v in 0..2usize {
            vid[v] = ViewId(self.s.next_vid[v]);
            self.s.next_vid[v] += 1;
        }
        self.s.vids[slot] = vid;
        let def = self.s.objs[slot].def;
        self.emit(Event::ZoneChange {
            obj: nr,
            def,
            from: ZoneKind::Gone,
            to: ZoneKind::Battlefield,
            owner,
            controller: owner,
            vid_old: [ViewId::NONE; 2],
            vid_new: vid,
            lki: None,
        });
        nr
    }
}
