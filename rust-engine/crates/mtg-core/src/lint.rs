//! Static checks over effect trees (doc 03 section 11).

use crate::ir::*;

fn tmax(a: i32, b: i32) -> i32 {
    a.max(b)
}

fn p(r: &PRef) -> i32 {
    match r {
        PRef::Target(i) => *i as i32,
        PRef::ControllerOf(o) | PRef::OwnerOf(o) => o_(o),
        _ => -1,
    }
}

fn o_(r: &ORef) -> i32 {
    match r {
        ORef::Target(i) => *i as i32,
        _ => -1,
    }
}

fn objs(o: &Objs) -> i32 {
    match o {
        Objs::One(r) => o_(r),
        Objs::TopOfGraveyard { who, .. } => p(who),
        Objs::All(_) | Objs::Captured => -1,
        Objs::TargetsFrom(n) => *n as i32,
    }
}

fn expr(e: &Expr) -> i32 {
    match e {
        Expr::Life(r) | Expr::CardsInHand(r) | Expr::CardsInLibrary(r) | Expr::CardsInGraveyard(r) | Expr::SpellsCastThisTurn(r) | Expr::LifeLostThisTurn(r) | Expr::LifeGainedThisTurn(r) => p(r),
        Expr::PowerOf(o) | Expr::ToughnessOf(o) | Expr::CmcOf(o) | Expr::Counters(o, _) | Expr::ManaSpent(o) | Expr::ColorsSpent(o) | Expr::Replicated(o) => o_(o),
        Expr::Energy(r) => p(r),
        Expr::Devotion(_, w) => p(w),
        Expr::GraveyardTypes(Some(r)) => p(r),
        Expr::Plus(a, b) | Expr::Minus(a, b) | Expr::Times(a, b) | Expr::Max(a, b) | Expr::Min(a, b) => tmax(expr(a), expr(b)),
        Expr::HalfDown(a) | Expr::HalfUp(a) => expr(a),
        _ => -1,
    }
}

fn cond(c: &Cond) -> i32 {
    match c {
        Cond::Not(c) => cond(c),
        Cond::And(v) | Cond::Or(v) => v.iter().map(cond).max().unwrap_or(-1),
        Cond::Cmp(a, _, b) => tmax(expr(a), expr(b)),
        Cond::IsActive(r) | Cond::Delirium(r) | Cond::Revolt(r) | Cond::Blessing(r) => p(r),
        Cond::Completed { who, .. } => p(who),
        Cond::UsedThisTurn(_) => -1,
        Cond::HasCounter(o, _) => o_(o),
        Cond::Controls { who, .. } => p(who),
        Cond::Matches(o, _) | Cond::ControlledBy(o, _) => o_(o),
        Cond::CastColor { who, .. } => p(who),
        Cond::True | Cond::WasCast { .. } => -1,
    }
}

/// Highest target slot index mentioned anywhere in the effect (-1 if none).
pub fn max_target_used(e: &Effect) -> i32 {
    match e {
        Effect::Seq(v) => v.iter().map(max_target_used).max().unwrap_or(-1),
        Effect::If { cond: c, then, els } => tmax(tmax(cond(c), max_target_used(then)), els.as_ref().map(|x| max_target_used(x)).unwrap_or(-1)),
        Effect::May { who, then } => tmax(p(who), max_target_used(then)),
        Effect::ForEach { body, .. } => max_target_used(body),
        Effect::Repeat { n, body } => tmax(expr(n), max_target_used(body)),
        Effect::Damage { amount, to } => tmax(
            expr(amount),
            match to {
                Rcpt::Target(i) => *i as i32,
                Rcpt::Player(r) => p(r),
                Rcpt::Obj(o) => o_(o),
            },
        ),
        Effect::Draw { who, n } | Effect::GainLife { who, n } | Effect::LoseLife { who, n } | Effect::Discard { who, n } | Effect::Mill { who, n } => tmax(p(who), expr(n)),
        Effect::Destroy(o) | Effect::Exile(o) | Effect::Bounce(o) | Effect::Tap(o) | Effect::Untap(o) => objs(o),
        Effect::CounterSpell(i) => *i as i32,
        Effect::CounterSpellOf(o) => o_(o),
        Effect::CounterUnless { target, .. } => *target as i32,
        Effect::CounterEventUnless { .. } => -1,
        Effect::CreateToken { n, who, .. } => tmax(expr(n), p(who)),
        Effect::AddCounters { objs: o, n, .. } => tmax(objs(o), expr(n)),
        Effect::AddMana { .. } => -1,
        Effect::AddManaAny { n } => expr(n),
        Effect::Scry(e) | Effect::Surveil(e) | Effect::PutBack(e) | Effect::Reorder(e) => expr(e),
        Effect::LookTake { look, .. } => expr(look),
        Effect::Search { .. } => -1,
        Effect::Shuffle(w) => p(w),
        Effect::Sacrifice(o) => objs(o),
        Effect::CounterSpellExile(i) => *i as i32,
        Effect::MoveTo { objs: o, ctrl, .. } => tmax(objs(o), ctrl.as_ref().map(|c| p(c)).unwrap_or(-1)),
        Effect::CounterAbility(i) => *i as i32,
        Effect::DiscardChoose { who, .. } | Effect::LookTop { who } | Effect::DiscardNamed { who } => p(who),
        Effect::ChooseName { .. } => -1,
        Effect::LookPutTop { look } => expr(look),
        Effect::Win(w) => p(w),
        Effect::CounterAny(i) => *i as i32,
        Effect::CopySpell { what, n } => tmax(o_(what), expr(n)),
        Effect::Delay { .. } | Effect::WatchAttacks { .. } => -1,
        Effect::Attach { what, to } => tmax(o_(what), o_(to)),
        Effect::ExileUntil { objs: o, .. } => objs(o),
        Effect::Continuous { objs: o, .. } => objs(o),
        Effect::PlayerFx { who, .. } => p(who),
        Effect::MayPay { who, then, .. } => tmax(p(who), max_target_used(then)),
        Effect::SacrificeChosen { who, .. } => p(who),
        Effect::PutChosen { who, cmc, .. } => tmax(p(who), cmc.as_ref().map(expr).unwrap_or(-1)),
        Effect::ExileGraveyard { who } => p(who),
        Effect::GainEnergy { who, n } | Effect::LoseEnergy { who, n } => tmax(p(who), expr(n)),
        Effect::Reflexive { .. } | Effect::DelayMoved { .. } | Effect::LinkedToken { .. } => -1,
        Effect::ExileLinked(o) | Effect::CopyToken(o) => objs(o),
        Effect::EachPutFromHand { .. } | Effect::Venture | Effect::MarkUsed(_) | Effect::AllowCastMoved | Effect::TapAttackMoved => -1,
        Effect::ExilePlay { n } | Effect::DrawRevealCast { n } => expr(n),
        Effect::MayElse { who, can, then, els } => tmax(tmax(p(who), cond(can)), tmax(max_target_used(then), max_target_used(els))),
        Effect::ForEachPlayer { body } => max_target_used(body),
        Effect::SacrificeOneEach { who, .. } => p(who),
        Effect::RevealPickTypes { look } => expr(look),
        Effect::Pile { .. } => -1,
        Effect::ExtractSame { target } => *target as i32,
        Effect::ReanimateAttach | Effect::AddManaRestricted | Effect::ExileCastEnergy => -1,
        Effect::CopyAs { objs: o, .. } => objs(o),
        Effect::EnergyWrath { .. } | Effect::CastFromGraveyard { .. } => -1,
        Effect::ExileUntilLeaves { creature } => *creature as i32,
        Effect::CreateEmblem(_) | Effect::NinjutsuEnter | Effect::RingTempts | Effect::DelayEventObj { .. } | Effect::MiracleCast(_) | Effect::PileBack | Effect::ExileReturnTransformed | Effect::KeepOneOfEach | Effect::DigCreatureAttacking | Effect::GambitReveal => -1,
        Effect::Amass { n, .. } => expr(n),
        Effect::TakeMoved { .. } => -1,
        Effect::SetPTX { objs: o, x, .. } => tmax(objs(o), expr(x)),
    }
}

/// Names of tokens created by an effect.
pub fn tokens_used<'a>(e: &'a Effect, out: &mut Vec<&'a str>) {
    match e {
        Effect::Seq(v) => v.iter().for_each(|x| tokens_used(x, out)),
        Effect::If { then, els, .. } => {
            tokens_used(then, out);
            if let Some(x) = els {
                tokens_used(x, out);
            }
        }
        Effect::May { then, .. } | Effect::MayPay { then, .. } => tokens_used(then, out),
        Effect::MayElse { then, els, .. } => {
            tokens_used(then, out);
            tokens_used(els, out);
        }
        Effect::ForEachPlayer { body } => tokens_used(body, out),
        Effect::LinkedToken { token } | Effect::Amass { token, .. } => out.push(token.as_str()),
        Effect::ForEach { body, .. } | Effect::Repeat { body, .. } => tokens_used(body, out),
        Effect::CreateToken { token, .. } | Effect::CreateEmblem(token) => out.push(token.as_str()),
        _ => {}
    }
}
