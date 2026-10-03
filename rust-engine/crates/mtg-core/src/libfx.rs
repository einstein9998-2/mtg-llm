//! Library-manipulating effects: scry, surveil, Brainstorm-style put back, reorder, look-and-take
//! and library search. Each is a small state machine over the VM scratch registers (`aux` = stage,
//! `sc` = counters) so it can pause for decisions and be cloned mid-resolution.
//!
//! Library orientation: the TOP of a library is the LAST element of its vector. "Position t" below
//! counts from the top (0 = top card).

use crate::cx::Cx;
use crate::decision::*;
use crate::engine::{ask, Next};
use crate::eval::Env;
use crate::frame::ResolveFrame;
use crate::ids::*;
use crate::ir::*;
use crate::ops::MoveOpts;
use crate::resolve::card_options;
use crate::types::*;

const S_START: u16 = 0;
const S_SELECT: u16 = 1;
const S_ORDER_BOTTOM: u16 = 2;
const S_ORDER_TOP: u16 = 3;

/// Whether the op wants to wait for a decision.
pub(crate) type Wait = bool;

impl<'a> Cx<'a> {
    /// `looker` looks at the top card of `owner`'s library and remembers where it is.
    pub(crate) fn look_top(&mut self, looker: Seat, owner: Seat) {
        if self.lib_len(owner) > 0 {
            let r = self.lib_at(owner, 0);
            self.know(looker, r, true);
        }
    }

    fn lib_len(&self, p: Seat) -> usize {
        self.s.players[p.idx()].library.len()
    }

    /// The card at position `t` from the top.
    fn lib_at(&self, p: Seat, t: usize) -> ObjRef {
        let l = &self.s.players[p.idx()].library;
        l[l.len() - 1 - t]
    }

    /// Top `k` cards, top first.
    fn lib_top(&self, p: Seat, k: usize) -> Vec<ObjRef> {
        (0..k.min(self.lib_len(p))).map(|t| self.lib_at(p, t)).collect()
    }

    fn lib_remove(&mut self, p: Seat, r: ObjRef) {
        let l = &mut self.s.players[p.idx()].library;
        let i = l.iter().position(|&x| x == r).expect("card is in the library");
        l.remove(i);
    }

    fn lib_to_bottom(&mut self, p: Seat, r: ObjRef) {
        self.lib_remove(p, r);
        self.s.players[p.idx()].library.insert(0, r);
    }

    /// Swaps `r` into position `t` from the top.
    fn lib_swap_to_top_pos(&mut self, p: Seat, r: ObjRef, t: usize) {
        let l = &mut self.s.players[p.idx()].library;
        let n = l.len();
        let i = l.iter().position(|&x| x == r).expect("card is in the library");
        l.swap(i, n - 1 - t);
    }

    /// Swaps `r` into position `b` from the bottom (0 = bottom-most).
    fn lib_swap_to_bottom_pos(&mut self, p: Seat, r: ObjRef, b: usize) {
        let l = &mut self.s.players[p.idx()].library;
        let i = l.iter().position(|&x| x == r).expect("card is in the library");
        l.swap(i, b);
    }

    fn know(&mut self, seat: Seat, r: ObjRef, pos: bool) {
        let k = &mut self.s.knowledge[r.slot as usize];
        k.known_to |= 1 << seat.idx();
        if pos {
            k.pos_known_to |= 1 << seat.idx();
        }
    }
}

fn ask_order(cx: &mut Cx, seat: Seat, purpose: CardsPurpose, cands: Vec<ObjRef>, remaining: usize) {
    let opts = card_options(cx, seat, &cands);
    ask(cx, seat, DecisionKind::ChooseCards { purpose, remaining: remaining as u8 }, opts);
}

/// Stage 3 / 2 helper shared by every effect that ends in an ordering of looked-at cards.
/// Returns true if a decision was asked.
fn order_stages(cx: &mut Cx, seat: Seat, f_aux: &mut u16, sc: &mut [u16; 3]) -> Wait {
    loop {
        match *f_aux {
            S_ORDER_BOTTOM => {
                let (b, j) = (sc[1] as usize, sc[2] as usize);
                if b >= 2 && j + 1 < b {
                    let len = cx.lib_len(seat);
                    // The cards not yet placed are the lowest b - j of the group: chosen cards are
                    // swapped up to the high end, so what remains sits in positions 0..b-j.
                    let cands: Vec<ObjRef> = (0..b - j).map(|i| cx.s.players[seat.idx()].library[i]).collect();
                    let _ = len;
                    ask_order(cx, seat, CardsPurpose::OrderBottom, cands, b - j);
                    return true;
                }
                for i in 0..b.min(cx.lib_len(seat)) {
                    let r = cx.s.players[seat.idx()].library[i];
                    cx.know(seat, r, true);
                }
                sc[2] = 0;
                *f_aux = S_ORDER_TOP;
            }
            S_ORDER_TOP => {
                let (k, j) = (sc[0] as usize, sc[2] as usize);
                if k >= 2 && j + 1 < k {
                    let cands: Vec<ObjRef> = (j..k).map(|t| cx.lib_at(seat, t)).collect();
                    ask_order(cx, seat, CardsPurpose::OrderTop, cands, k - j);
                    return true;
                }
                for t in 0..k.min(cx.lib_len(seat)) {
                    let r = cx.lib_at(seat, t);
                    cx.know(seat, r, true);
                }
                return false;
            }
            _ => return false,
        }
    }
}

impl<'a> Cx<'a> {
    /// Scry (`graveyard` false) or surveil (true).
    pub(crate) fn fx_scry(&mut self, env: &Env, n: &Expr, graveyard: bool, aux: &mut u16, sc: &mut [u16; 3]) -> Wait {
        let seat = env.controller;
        if *aux == S_START {
            let k = (self.eval_expr(env, n).max(0) as usize).min(self.lib_len(seat));
            if k == 0 {
                return false;
            }
            sc[0] = k as u16; // looked-at cards still on top
            sc[1] = 0; // cards put on the bottom
            sc[2] = 0;
            for r in self.lib_top(seat, k) {
                self.know(seat, r, false);
            }
            *aux = S_SELECT;
        }
        if *aux == S_SELECT {
            let k = sc[0] as usize;
            if k == 0 {
                *aux = S_ORDER_BOTTOM;
            } else {
                let cands = self.lib_top(seat, k);
                let mut opts = card_options(self, seat, &cands);
                opts.push(Opt::Done);
                let purpose = if graveyard { CardsPurpose::SurveilGraveyard } else { CardsPurpose::ScryBottom };
                ask(self, seat, DecisionKind::ChooseCards { purpose, remaining: k as u8 }, opts);
                return true;
            }
        }
        order_stages(self, seat, aux, sc)
    }

    pub(crate) fn fx_put_back(&mut self, env: &Env, n: &Expr, aux: &mut u16, sc: &mut [u16; 3]) -> Wait {
        let seat = env.controller;
        if *aux == S_START {
            let want = self.eval_expr(env, n).max(0) as usize;
            let hand = self.s.players[seat.idx()].hand.len();
            sc[1] = 0; // no bottom cards
            sc[2] = 0;
            sc[0] = 0; // cards put back so far
            // `aux` stays at START until the selection is over; `sc[1]` is reused below as the target count.
            sc[1] = want.min(hand) as u16;
            *aux = 10;
        }
        if *aux == 10 {
            let (done, target) = (sc[0] as usize, sc[1] as usize);
            if done < target {
                // A forced choice (every card in hand goes back) is still presented; the runner
                // decides whether to skip trivial decisions.
                let hand = self.s.players[seat.idx()].hand.clone();
                let opts = card_options(self, seat, &hand);
                ask(self, seat, DecisionKind::ChooseCards { purpose: CardsPurpose::PutBack, remaining: (target - done) as u8 }, opts);
                return true;
            }
            // Order the cards just put on top.
            sc[0] = sc[0].min(sc[1]);
            sc[1] = 0;
            sc[2] = 0;
            *aux = S_ORDER_TOP;
        }
        order_stages(self, seat, aux, sc)
    }

    pub(crate) fn fx_reorder(&mut self, env: &Env, n: &Expr, aux: &mut u16, sc: &mut [u16; 3]) -> Wait {
        let seat = env.controller;
        if *aux == S_START {
            let k = (self.eval_expr(env, n).max(0) as usize).min(self.lib_len(seat));
            if k == 0 {
                return false;
            }
            for r in self.lib_top(seat, k) {
                self.know(seat, r, false);
            }
            sc[0] = k as u16;
            sc[1] = 0;
            sc[2] = 0;
            *aux = S_ORDER_TOP;
        }
        order_stages(self, seat, aux, sc)
    }

    pub(crate) fn fx_look_take(&mut self, env: &Env, look: &Expr, take: u8, aux: &mut u16, sc: &mut [u16; 3]) -> Wait {
        let seat = env.controller;
        if *aux == S_START {
            let k = (self.eval_expr(env, look).max(0) as usize).min(self.lib_len(seat));
            if k == 0 {
                return false;
            }
            for r in self.lib_top(seat, k) {
                self.know(seat, r, false);
            }
            sc[0] = k as u16; // looked-at cards still on top
            sc[1] = take.min(k as u8) as u16; // still to take
            sc[2] = 0;
            *aux = 20;
        }
        if *aux == 20 {
            if sc[1] > 0 {
                let cands = self.lib_top(seat, sc[0] as usize);
                let opts = card_options(self, seat, &cands);
                ask(self, seat, DecisionKind::ChooseCards { purpose: CardsPurpose::LookTake, remaining: sc[1] as u8 }, opts);
                return true;
            }
            // The rest go to the bottom (kept in their current order until the owner orders them).
            let rest = self.lib_top(seat, sc[0] as usize);
            for r in rest.iter().rev() {
                self.lib_to_bottom(seat, *r);
            }
            sc[1] = rest.len() as u16;
            sc[0] = 0;
            sc[2] = 0;
            *aux = S_ORDER_BOTTOM;
        }
        order_stages(self, seat, aux, sc)
    }

    /// Atraxa: reveal the top `look` cards; put at most one card per card type into hand (each
    /// card serving one of its types), the rest on the bottom in a random order.
    pub(crate) fn fx_reveal_pick_types(&mut self, env: &Env, look: &Expr, aux: &mut u16, sc: &mut [u16; 3]) -> Wait {
        let seat = env.controller;
        if *aux == S_START {
            let k = (self.eval_expr(env, look).max(0) as usize).min(self.lib_len(seat));
            if k == 0 {
                return false;
            }
            for r in self.lib_top(seat, k) {
                self.know(Seat::P0, r, true);
                self.know(Seat::P1, r, true);
            }
            sc[0] = k as u16;
            self.s.moved.clear();
            *aux = 40;
        }
        if *aux == 40 {
            let chosen: Vec<Types> = self.s.moved.iter().map(|&r| self.pick_types(r)).collect();
            let cands: Vec<ObjRef> = self
                .lib_top(seat, sc[0] as usize)
                .into_iter()
                .filter(|&r| {
                    let mut v = chosen.clone();
                    v.push(self.pick_types(r));
                    distinct_types_possible(&v)
                })
                .collect();
            if cands.is_empty() {
                return self.reveal_pick_finish(seat, aux, sc);
            }
            let mut opts = card_options(self, seat, &cands);
            opts.push(Opt::Done);
            ask(self, seat, DecisionKind::ChooseCards { purpose: CardsPurpose::RevealPick, remaining: 1 }, opts);
            return true;
        }
        false
    }

    fn pick_types(&self, r: ObjRef) -> Types {
        self.types_of_cards(&[r])
    }

    pub(crate) fn reveal_pick_finish(&mut self, seat: Seat, aux: &mut u16, sc: &mut [u16; 3]) -> Wait {
        let mut rest = self.lib_top(seat, sc[0] as usize);
        self.random_bottom_order(seat, &mut rest);
        for r in rest {
            self.lib_to_bottom(seat, r);
            // Put on the bottom in a random order: everyone knows the card, nobody its place.
            self.s.knowledge[r.slot as usize].pos_known_to = 0;
        }
        sc[0] = 0;
        *aux = 41;
        false
    }

    /// Thassa's Oracle: look at the top `look` cards, keep up to one on top, the rest go to the
    /// bottom in a random order.
    pub(crate) fn fx_look_put_top(&mut self, env: &Env, look: &Expr, aux: &mut u16, sc: &mut [u16; 3]) -> Wait {
        let seat = env.controller;
        if *aux == S_START {
            let k = (self.eval_expr(env, look).max(0) as usize).min(self.lib_len(seat));
            if k == 0 {
                return false;
            }
            for r in self.lib_top(seat, k) {
                self.know(seat, r, false);
            }
            sc[0] = k as u16;
            let cands = self.lib_top(seat, k);
            let mut opts = card_options(self, seat, &cands);
            opts.push(Opt::Done);
            ask(self, seat, DecisionKind::ChooseCards { purpose: CardsPurpose::OracleTop, remaining: 1 }, opts);
            *aux = 30;
            return true;
        }
        false
    }

    /// Doomsday: pick `n` cards one at a time from library and graveyard (each goes on top of the
    /// library), exile everything else from both, then order the pile.
    pub(crate) fn fx_pile(&mut self, env: &Env, n: u8, aux: &mut u16, sc: &mut [u16; 3]) -> Wait {
        let seat = env.controller;
        if *aux == S_START {
            let total = self.lib_len(seat) + self.s.players[seat.idx()].graveyard.len();
            sc[0] = (n as usize).min(total) as u16; // cards still to pick
            sc[1] = 0; // cards picked (they sit on top of the library)
            sc[2] = 0;
            *aux = S_SELECT;
        }
        if *aux == S_SELECT {
            if sc[0] > 0 {
                let picked = sc[1] as usize;
                let lib = &self.s.players[seat.idx()].library;
                let mut cands: Vec<ObjRef> = lib[..lib.len() - picked].iter().rev().copied().collect();
                cands.extend(self.s.players[seat.idx()].graveyard.iter().copied());
                if cands.len() <= sc[0] as usize {
                    // Every candidate has to go in the pile: nothing to choose (the pile is ordered next).
                    for r in cands {
                        if self.s.players[seat.idx()].library.contains(&r) {
                            self.lib_remove(seat, r);
                            self.s.players[seat.idx()].library.push(r);
                        } else {
                            self.move_zone(r, ZoneKind::Library, MoveOpts::default());
                        }
                        sc[0] -= 1;
                        sc[1] += 1;
                    }
                } else {
                    for &r in &cands {
                        self.know(seat, r, false);
                    }
                    let opts = card_options(self, seat, &cands);
                    ask(self, seat, DecisionKind::ChooseCards { purpose: CardsPurpose::DoomsdayPile, remaining: sc[0] as u8 }, opts);
                    return true;
                }
            }
            // Pile complete: exile the rest of both zones.
            let k = sc[1] as usize;
            let rest: Vec<ObjRef> = {
                let p = &self.s.players[seat.idx()];
                let mut v: Vec<ObjRef> = p.library[..p.library.len() - k].to_vec();
                v.extend(p.graveyard.iter().copied());
                v
            };
            for r in rest {
                self.move_zone(r, ZoneKind::Exile, MoveOpts::default());
            }
            sc[0] = k as u16;
            sc[1] = 0;
            sc[2] = 0;
            *aux = S_ORDER_TOP;
        }
        order_stages(self, seat, aux, sc)
    }

    pub(crate) fn fx_search(&mut self, env: &Env, who: PRef, filter: &ObjFilter, aux: &mut u16) -> Wait {
        let seat = match self.eval_p(env, who) {
            Some(s) => s,
            None => return false,
        };
        match *aux {
            0 => {
                self.refresh();
                let cands: Vec<ObjRef> = self.s.players[seat.idx()].library.iter().rev().copied().filter(|&r| self.lib_card_matches(env, filter, r)).collect();
                for &r in &cands {
                    self.know(seat, r, false);
                }
                let mut opts = card_options(self, seat, &cands);
                opts.push(Opt::Done);
                ask(self, seat, DecisionKind::ChooseCards { purpose: CardsPurpose::Search, remaining: 1 }, opts);
                *aux = 1;
                true
            }
            _ => false,
        }
    }

    /// Does library card `r` match the filter (printed characteristics)?
    fn lib_card_matches(&self, env: &Env, f: &ObjFilter, r: ObjRef) -> bool {
        let c = self.chars(r);
        self.filter_match(f, r, &c, false, env.controller)
    }
}

/// Handles the answers of the library-effect decisions. Returns None if `p` is not one of ours.
pub(crate) fn feed(cx: &mut Cx, f: &mut ResolveFrame, p: &Pending, opt: Opt, dest: Option<(ZoneKind, bool)>) -> Option<Next> {
    let seat = p.seat;
    let purpose = match p.kind {
        DecisionKind::ChooseCards { purpose, .. } => purpose,
        _ => return None,
    };
    match (purpose, opt) {
        (CardsPurpose::ScryBottom, Opt::Card(r)) => {
            cx.lib_to_bottom(seat, r);
            f.sc[0] -= 1;
            f.sc[1] += 1;
        }
        (CardsPurpose::SurveilGraveyard, Opt::Card(r)) => {
            cx.move_zone(r, ZoneKind::Graveyard, MoveOpts::default());
            f.sc[0] -= 1;
        }
        (CardsPurpose::ScryBottom, Opt::Done) | (CardsPurpose::SurveilGraveyard, Opt::Done) => {
            // The remaining looked-at cards stay on top; selection is over.
            f.aux = S_ORDER_BOTTOM;
            f.sc[2] = 0;
        }
        (CardsPurpose::PutBack, Opt::Card(r)) => {
            cx.move_zone(r, ZoneKind::Library, MoveOpts::default());
            f.sc[0] += 1;
        }
        (CardsPurpose::DoomsdayPile, Opt::Card(r)) => {
            if cx.s.players[seat.idx()].library.contains(&r) {
                cx.lib_remove(seat, r);
                cx.s.players[seat.idx()].library.push(r);
            } else {
                cx.move_zone(r, ZoneKind::Library, MoveOpts::default());
            }
            f.sc[0] -= 1;
            f.sc[1] += 1;
        }
        (CardsPurpose::LookTake, Opt::Card(r)) => {
            cx.move_zone(r, ZoneKind::Hand, MoveOpts::default());
            f.sc[0] -= 1;
            f.sc[1] -= 1;
        }
        (CardsPurpose::OrderTop, Opt::Card(r)) => {
            let j = f.sc[2] as usize;
            cx.lib_swap_to_top_pos(seat, r, j);
            f.sc[2] += 1;
        }
        (CardsPurpose::OrderBottom, Opt::Card(r)) => {
            let j = f.sc[2] as usize;
            // Bottom cards are offered top-most first: the first chosen sits highest of them.
            let b = f.sc[1] as usize;
            cx.lib_swap_to_bottom_pos(seat, r, b - 1 - j);
            f.sc[2] += 1;
        }
        (CardsPurpose::RevealPick, Opt::Card(r)) => {
            if let Some(nr) = cx.move_zone(r, ZoneKind::Hand, MoveOpts::default()) {
                cx.s.moved.push(nr);
            }
            f.sc[0] -= 1;
        }
        (CardsPurpose::RevealPick, Opt::Done) => {
            cx.reveal_pick_finish(seat, &mut f.aux, &mut f.sc);
        }
        (CardsPurpose::OracleTop, o) => {
            let k = f.sc[0] as usize;
            let keep = if let Opt::Card(r) = o { Some(r) } else { None };
            let mut rest: Vec<ObjRef> = cx.lib_top(seat, k).into_iter().filter(|&x| Some(x) != keep).collect();
            cx.random_bottom_order(seat, &mut rest);
            for r in rest {
                cx.lib_to_bottom(seat, r);
                // Put on the bottom in a random order: nobody knows its place any more.
                cx.s.knowledge[r.slot as usize].pos_known_to = 0;
            }
            if let Some(r) = keep {
                cx.lib_remove(seat, r);
                cx.s.players[seat.idx()].library.push(r);
                cx.know(seat, r, true);
            }
            f.sc[0] = 0;
        }
        (CardsPurpose::Search, o) => {
            if let Opt::Card(r) = o {
                let (z, tapped) = dest.unwrap_or((ZoneKind::Hand, false));
                if z == ZoneKind::Library {
                    // Tutor to the top: shuffle first, then the revealed card goes on top.
                    cx.shuffle_library(seat);
                    cx.lib_remove(seat, r);
                    cx.s.players[seat.idx()].library.push(r);
                    cx.know(Seat::P0, r, true);
                    cx.know(Seat::P1, r, true);
                    return Some(Next::Stay);
                }
                cx.move_zone(r, z, MoveOpts { tapped, ..Default::default() });
            }
            cx.shuffle_library(seat);
        }
        _ => return None,
    }
    Some(Next::Stay)
}

/// Can each of the card-type sets be given its own distinct type from the eight types that
/// Atraxa counts?
fn distinct_types_possible(cards: &[Types]) -> bool {
    const KINDS: [Types; 7] = [Types::ARTIFACT, Types::CREATURE, Types::ENCHANTMENT, Types::INSTANT, Types::LAND, Types::PLANESWALKER, Types::SORCERY];
    fn go(cards: &[Types], i: usize, used: u8) -> bool {
        if i == cards.len() {
            return true;
        }
        for (k, t) in KINDS.iter().enumerate() {
            if cards[i].contains(*t) && used & (1 << k) == 0 && go(cards, i + 1, used | (1 << k)) {
                return true;
            }
        }
        false
    }
    go(cards, 0, 0)
}
