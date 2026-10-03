//! Pre-game: shuffle, opening hands, London mulligan (CR 103.5). Low-frequency decisions.

use crate::cx::Cx;
use crate::decision::*;
use crate::engine::{ask, Next};
use crate::event::Event;
use crate::frame::*;
use crate::ids::*;
use crate::ops::MoveOpts;
use crate::types::*;

fn seat_at(cx: &Cx, who: u8) -> Seat {
    if who == 0 {
        cx.s.turn.first
    } else {
        cx.s.turn.first.other()
    }
}

fn draw_opening(cx: &mut Cx, seat: Seat) {
    let n = cx.s.cfg.hand_size;
    for _ in 0..n {
        cx.draw(seat);
    }
}

pub fn run(cx: &mut Cx, f: &mut PregameFrame) -> Next {
    match f.stage {
        PregameStage::Setup => {
            for s in [cx.s.turn.first, cx.s.turn.first.other()] {
                cx.shuffle_library(s);
            }
            for s in [cx.s.turn.first, cx.s.turn.first.other()] {
                draw_opening(cx, s);
            }
            f.stage = PregameStage::AskKeep;
            f.who = 0;
            Next::Stay
        }
        PregameStage::AskKeep => {
            let seat = seat_at(cx, f.who);
            let mulls = cx.s.players[seat.idx()].mulligans;
            let opts = if mulls < cx.s.cfg.hand_size { vec![Opt::Keep, Opt::Mulligan] } else { vec![Opt::Keep] };
            ask(cx, seat, DecisionKind::Mulligan, opts);
            Next::Await
        }
        PregameStage::Bottom { left } => {
            let seat = seat_at(cx, f.who);
            if left == 0 {
                finish_player(cx, f, seat);
                return Next::Stay;
            }
            let hand = cx.s.players[seat.idx()].hand.clone();
            let opts = crate::resolve::card_options(cx, seat, &hand);
            ask(cx, seat, DecisionKind::ChooseCards { purpose: CardsPurpose::BottomAfterMulligan, remaining: left }, opts);
            Next::Await
        }
        PregameStage::Done => Next::Done,
    }
}

fn finish_player(cx: &mut Cx, f: &mut PregameFrame, seat: Seat) {
    cx.s.players[seat.idx()].kept = true;
    let b = cx.s.players[seat.idx()].mulligans;
    cx.emit(Event::HandKept { player: seat, bottomed: b });
    if f.who == 0 {
        f.who = 1;
        f.stage = PregameStage::AskKeep;
    } else {
        f.stage = PregameStage::Done;
    }
}

pub fn feed(cx: &mut Cx, f: &mut PregameFrame, p: &Pending, opt: Opt) -> Next {
    let seat = p.seat;
    match (p.kind, opt) {
        (DecisionKind::Mulligan, Opt::Keep) => {
            let m = cx.s.players[seat.idx()].mulligans;
            f.stage = PregameStage::Bottom { left: m };
            Next::Stay
        }
        (DecisionKind::Mulligan, Opt::Mulligan) => {
            cx.s.players[seat.idx()].mulligans += 1;
            cx.emit(Event::MulliganTaken { player: seat });
            let hand = cx.s.players[seat.idx()].hand.clone();
            for r in hand {
                cx.move_zone(r, ZoneKind::Library, MoveOpts::default());
            }
            cx.shuffle_library(seat);
            draw_opening(cx, seat);
            Next::Stay
        }
        (DecisionKind::ChooseCards { purpose: CardsPurpose::BottomAfterMulligan, .. }, Opt::Card(r)) => {
            cx.move_zone(r, ZoneKind::Library, MoveOpts { to_bottom: true, ..Default::default() });
            if let PregameStage::Bottom { left } = &mut f.stage {
                *left -= 1;
            }
            Next::Stay
        }
        _ => unreachable!(),
    }
}
