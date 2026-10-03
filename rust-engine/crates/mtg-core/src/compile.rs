//! Compiles effect trees to flat instruction lists (doc 03 section 5). The instruction pointer
//! plus a few locals is the whole resumption state, so resolution can pause at any decision.

use crate::ir::*;
use crate::mana::ManaCost;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Instr {
    Leaf(Effect),
    Jump(u16),
    JumpUnless(Cond, u16),
    /// Asks `who` yes/no; "no" jumps to `skip`.
    May { who: PRef, skip: u16 },
    /// Asks `who` whether to pay `pays`; "no" (or being unable to) jumps to `skip`, "yes" pays.
    MayPay { who: PRef, pays: ManaCost, skip: u16 },
    LoopInit { filter: ObjFilter, depth: u8 },
    /// Loop over the players, active player first.
    PlayerLoopInit { depth: u8 },
    PlayerLoopNext { depth: u8, end: u16 },
    LoopNext { depth: u8, end: u16 },
    RepeatInit { n: Expr, depth: u8 },
    RepeatNext { depth: u8, end: u16 },
    /// Continues at `skip` unless mode `i` was chosen when the spell was cast; otherwise sets the
    /// base offset of this mode's target slots.
    Mode { i: u8, skip: u16 },
}

pub const MAX_LOOP_DEPTH: u8 = 2;

pub type Code = Vec<Instr>;

struct Compiler {
    code: Code,
    depth: u8,
}

impl Compiler {
    fn pc(&self) -> u16 {
        self.code.len() as u16
    }

    fn emit(&mut self, i: Instr) -> usize {
        self.code.push(i);
        self.code.len() - 1
    }

    fn patch(&mut self, at: usize, target: u16) {
        match &mut self.code[at] {
            Instr::Jump(t) | Instr::JumpUnless(_, t) => *t = target,
            Instr::May { skip, .. } | Instr::MayPay { skip, .. } | Instr::Mode { skip, .. } => *skip = target,
            Instr::LoopNext { end, .. } | Instr::RepeatNext { end, .. } | Instr::PlayerLoopNext { end, .. } => *end = target,
            other => panic!("cannot patch {other:?}"),
        }
    }

    fn effect(&mut self, e: &Effect) -> Result<(), String> {
        match e {
            Effect::Seq(v) => {
                for x in v {
                    self.effect(x)?;
                }
            }
            Effect::If { cond, then, els } => {
                let j = self.emit(Instr::JumpUnless(cond.clone(), 0));
                self.effect(then)?;
                match els {
                    Some(els) => {
                        let j2 = self.emit(Instr::Jump(0));
                        let pc = self.pc();
                        self.patch(j, pc);
                        self.effect(els)?;
                        let pc = self.pc();
                        self.patch(j2, pc);
                    }
                    None => {
                        let pc = self.pc();
                        self.patch(j, pc);
                    }
                }
            }
            Effect::May { who, then } => {
                let m = self.emit(Instr::May { who: *who, skip: 0 });
                self.effect(then)?;
                let pc = self.pc();
                self.patch(m, pc);
            }
            Effect::MayPay { who, pays, then } => {
                let m = self.emit(Instr::MayPay { who: *who, pays: pays.clone(), skip: 0 });
                self.effect(then)?;
                let pc = self.pc();
                self.patch(m, pc);
            }
            Effect::ForEach { filter, body } => {
                let depth = self.depth;
                if depth >= MAX_LOOP_DEPTH {
                    return Err("loops nested too deeply".into());
                }
                self.depth += 1;
                self.emit(Instr::LoopInit { filter: filter.clone(), depth });
                let top = self.pc();
                let n = self.emit(Instr::LoopNext { depth, end: 0 });
                self.effect(body)?;
                self.emit(Instr::Jump(top));
                let pc = self.pc();
                self.patch(n, pc);
                self.depth -= 1;
            }
            Effect::MayElse { who, can, then, els } => {
                // `can` false: the choice is not offered and the player takes the else branch.
                let c = self.emit(Instr::JumpUnless(can.clone(), 0));
                let m = self.emit(Instr::May { who: *who, skip: 0 });
                self.effect(then)?;
                let j = self.emit(Instr::Jump(0));
                let pc = self.pc();
                self.patch(m, pc);
                self.patch(c, pc);
                self.effect(els)?;
                let pc = self.pc();
                self.patch(j, pc);
            }
            Effect::ForEachPlayer { body } => {
                let depth = self.depth;
                if depth >= MAX_LOOP_DEPTH {
                    return Err("loops nested too deeply".into());
                }
                self.depth += 1;
                self.emit(Instr::PlayerLoopInit { depth });
                let top = self.pc();
                let n = self.emit(Instr::PlayerLoopNext { depth, end: 0 });
                self.effect(body)?;
                self.emit(Instr::Jump(top));
                let pc = self.pc();
                self.patch(n, pc);
                self.depth -= 1;
            }
            Effect::Repeat { n, body } => {
                let depth = self.depth;
                if depth >= MAX_LOOP_DEPTH {
                    return Err("loops nested too deeply".into());
                }
                self.depth += 1;
                self.emit(Instr::RepeatInit { n: n.clone(), depth });
                let top = self.pc();
                let nx = self.emit(Instr::RepeatNext { depth, end: 0 });
                self.effect(body)?;
                self.emit(Instr::Jump(top));
                let pc = self.pc();
                self.patch(nx, pc);
                self.depth -= 1;
            }
            leaf => {
                self.emit(Instr::Leaf(leaf.clone()));
            }
        }
        Ok(())
    }
}

pub fn compile_effect(e: &Effect) -> Result<Code, String> {
    let mut c = Compiler { code: Vec::new(), depth: 0 };
    c.effect(e)?;
    Ok(c.code)
}

/// Compiles a spell: one block per mode, each guarded by a `Mode` instruction.
pub fn compile_spell(s: &SpellDef) -> Result<Code, String> {
    let mut c = Compiler { code: Vec::new(), depth: 0 };
    for (i, m) in s.modes.iter().enumerate() {
        let guard = c.emit(Instr::Mode { i: i as u8, skip: 0 });
        if let Some(e) = &m.effect {
            c.effect(e)?;
        }
        let pc = c.pc();
        c.patch(guard, pc);
    }
    Ok(c.code)
}
