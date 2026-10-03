//! A tiny scripting helper for scenario tests: drive a `Game` by matching option labels.

use mtg_core::decision::Status;
use mtg_core::ids::Seat;
use mtg_view::{Decision, Game};

pub struct Driver {
    pub g: Game,
}

impl Driver {
    pub fn new(g: Game) -> Driver {
        let mut d = Driver { g };
        d.g.advance();
        d
    }

    pub fn decision(&self) -> Decision {
        let seat = self.g.pending().expect("a pending decision").seat;
        self.g.observe(seat).decision.expect("decision for decider")
    }

    /// Chooses the unique option of the current decision whose label starts with `prefix`.
    /// Panics (listing the options) if there is no match or more than one.
    pub fn act(&mut self, prefix: &str) -> &mut Driver {
        let d = self.decision();
        let m: Vec<_> = d.options.iter().filter(|o| o.label.starts_with(prefix)).collect();
        assert!(m.len() == 1, "act({prefix:?}): {} matches among {:#?} ({:?} for {:?})", m.len(), d.options.iter().map(|o| &o.label).collect::<Vec<_>>(), d.kind, d.seat);
        let idx = m[0].idx as usize;
        self.g.apply(d.id, idx).unwrap();
        self.g.advance();
        self
    }

    /// Chooses the option whose label is exactly `label`.
    pub fn act_exact(&mut self, label: &str) -> &mut Driver {
        let d = self.decision();
        let m: Vec<_> = d.options.iter().filter(|o| o.label == label).collect();
        assert!(m.len() == 1, "act_exact({label:?}): {} matches among {:?} ({:?} for {:?})", m.len(), d.options.iter().map(|o| &o.label).collect::<Vec<_>>(), d.kind, d.seat);
        let idx = m[0].idx as usize;
        self.g.apply(d.id, idx).unwrap();
        self.g.advance();
        self
    }

    /// Chooses the option at index `idx` of the current decision.
    pub fn act_idx(&mut self, idx: usize) -> &mut Driver {
        let d = self.decision();
        self.g.apply(d.id, idx).unwrap();
        self.g.advance();
        self
    }

    pub fn has(&self, prefix: &str) -> bool {
        self.decision().options.iter().any(|o| o.label.starts_with(prefix))
    }

    pub fn seat(&self) -> Seat {
        self.g.pending().unwrap().seat
    }

    pub fn labels(&self) -> Vec<String> {
        self.decision().options.into_iter().map(|o| o.label).collect()
    }

    /// Both players pass until the stack is empty and the step ends or a non-pass-only decision
    /// is reached; stops as soon as the decider has an option other than passing.
    pub fn pass_until(&mut self, pred: impl Fn(&Driver) -> bool) -> &mut Driver {
        for _ in 0..2000 {
            if pred(self) {
                return self;
            }
            if self.g.result().is_some() {
                return self;
            }
            if self.has("Pass priority") {
                self.act("Pass priority");
            } else {
                let d = self.decision();
                assert!(d.options.len() == 1, "pass_until: non-trivial decision {:?} {:?}", d.kind, d.options.iter().map(|o| &o.label).collect::<Vec<_>>());
                self.g.apply(d.id, d.options[0].idx as usize).unwrap();
                self.g.advance();
            }
        }
        panic!("pass_until: predicate never satisfied");
    }

    pub fn over(&self) -> bool {
        matches!(self.g.result(), Some(_))
    }
}

pub fn status_is_over(s: Status) -> bool {
    matches!(s, Status::GameOver(_))
}
