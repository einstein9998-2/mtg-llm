//! The driver: `advance` runs rules machinery until a decision is needed; `apply` feeds an answer
//! to the top frame (doc 01 section 3.2). Everything in progress lives in `State.frames`.

use crate::cx::Cx;
use crate::decision::*;
use crate::frame::*;
use crate::ids::*;

/// What a frame step did.
pub enum Next {
    /// Frame finished; pop it.
    Done,
    /// Keep the frame (it mutated itself); run it again.
    Stay,
    /// Keep the frame; a decision is pending.
    Await,
    /// Keep the frame and push a child on top.
    Push(Frame),
}

pub fn advance(cx: &mut Cx) -> Status {
    loop {
        if let Some(r) = cx.s.result {
            return Status::GameOver(r);
        }
        if let Some(p) = &cx.s.pending {
            return Status::NeedDecision(p.seat);
        }
        cx.s.steps += 1;
        run_top(cx);
    }
}

fn run_top(cx: &mut Cx) {
    let mut f = cx.s.frames.pop().expect("frame stack must never be empty");
    let next = match &mut f {
        Frame::Turn => crate::turn::run_turn(cx),
        Frame::Pregame(p) => crate::pregame::run(cx, p),
        Frame::Stabilize(st) => crate::stabilize::run(cx, st),
        Frame::Cast(c) => crate::cast::run(cx, c),
        Frame::Resolve(r) => crate::resolve::run(cx, r),
        Frame::Combat(c) => crate::combat::run(cx, c),
        Frame::Cleanup(c) => crate::turn::run_cleanup(cx, c),
    };
    settle(cx, f, next);
}

fn settle(cx: &mut Cx, f: Frame, next: Next) {
    match next {
        Next::Done => {}
        Next::Stay | Next::Await => cx.s.frames.push(f),
        Next::Push(g) => {
            cx.s.frames.push(f);
            cx.s.frames.push(g);
        }
    }
}

pub fn apply(cx: &mut Cx, id: DecisionId, idx: usize) -> Result<(), ApplyError> {
    if cx.s.result.is_some() {
        return Err(ApplyError::GameOver);
    }
    let p = match &cx.s.pending {
        None => return Err(ApplyError::NoDecision),
        Some(p) => p,
    };
    if p.id != id {
        return Err(ApplyError::StaleDecision);
    }
    if idx >= p.options.len() {
        return Err(ApplyError::BadIndex);
    }
    let p = cx.s.pending.take().unwrap();
    let opt = p.options[idx];
    let mut f = cx.s.frames.pop().expect("frame stack must never be empty");
    let next = match &mut f {
        Frame::Turn => crate::priority::feed(cx, &p, opt),
        Frame::Pregame(pg) => crate::pregame::feed(cx, pg, &p, opt),
        Frame::Stabilize(st) => crate::stabilize::feed(cx, st, &p, opt),
        Frame::Cast(c) => crate::cast::feed(cx, c, &p, opt),
        Frame::Resolve(r) => crate::resolve::feed(cx, r, &p, opt),
        Frame::Combat(c) => crate::combat::feed(cx, c, &p, opt),
        Frame::Cleanup(c) => crate::turn::feed_cleanup(cx, c, &p, opt),
    };
    settle(cx, f, next);
    Ok(())
}

/// Sets the pending decision. Options must already be in canonical order and non-empty.
pub fn ask(cx: &mut Cx, seat: Seat, kind: DecisionKind, options: Vec<Opt>) {
    debug_assert!(!options.is_empty(), "a decision with no options is an engine bug: {kind:?}");
    // Ids count per seat: a global counter would let one seat see how many decisions the other
    // made, which can depend on its hidden cards or choices (scry, Aether Vial).
    // A card offered as an option needs a view id in the chooser's id space (a scried, searched or
    // Doomsday card has none yet). Handed out in definition order, never library order, so the
    // numbers say nothing about where the cards sit.
    let mut fresh: Vec<ObjRef> = options
        .iter()
        .filter_map(|o| if let Opt::Card(r) = o { Some(*r) } else { None })
        .filter(|r| cx.s.is_live(*r) && cx.s.vids[r.slot as usize][seat.idx()] == ViewId::NONE)
        .collect();
    fresh.sort_by_key(|r| cx.s.objs[r.slot as usize].def);
    for r in fresh {
        cx.s.vids[r.slot as usize][seat.idx()] = ViewId(cx.s.next_vid[seat.idx()]);
        cx.s.next_vid[seat.idx()] += 1;
    }
    let id = DecisionId(cx.s.next_decision[seat.idx()]);
    cx.s.next_decision[seat.idx()] += 1;
    cx.s.pending = Some(Pending { id, seat, kind, options });
}
