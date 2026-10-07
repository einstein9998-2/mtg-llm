//! Bot-side guard for the Alurentell player, from Brady's tough-spot answers (playbook "Rules from
//! Brady's tough-spot review"). It only removes options the search is not allowed to pick; the engine,
//! the net and the search are untouched, and it reads nothing but the bot's own Observation.
//!
//! 1. Never cast a cantrip (Brainstorm, Ponder, Stock Up) in your own upkeep (Brady chose Brainstorm in the draw step).
//! 2. In your own main phase with the stack empty and a land drop still available: no passing and no
//!    committal spell (Show and Tell, Aluren, Atraxa, Acererak, Omniscience) before the land is played.
//!    Cantrips and Veil are still allowed first (cantrips find the land; the Veil line is land, Veil, Show and Tell).
use mtg_core::types::Step;
use mtg_view::*;

const CANTRIPS: [&str; 3] = ["Brainstorm", "Ponder", "Stock Up"];
const COMMITTAL: [&str; 5] = ["Show and Tell", "Aluren", "Atraxa", "Acererak", "Omniscience"];

fn casts(label: &str, names: &[&str]) -> bool {
    label.starts_with("Cast ") && names.iter().any(|n| label[5..].starts_with(n))
}

/// `true` for each option the guard forbids. Never forbids every option.
pub fn forbidden(obs: &Observation, d: &Decision) -> Vec<bool> {
    let mut bad = vec![false; d.options.len()];
    if !matches!(d.kind, ViewDecisionKind::Priority) || !obs.active_is_me || !obs.stack.is_empty() {
        return bad;
    }
    let can_land = d.options.iter().any(|o| o.kind == ActionKind::PlayLand);
    for (i, o) in d.options.iter().enumerate() {
        if obs.step == Step::Upkeep && casts(&o.label, &CANTRIPS) {
            bad[i] = true;
        }
        if obs.step.is_main() && can_land && (o.kind == ActionKind::Pass || casts(&o.label, &COMMITTAL)) {
            bad[i] = true;
        }
    }
    if bad.iter().all(|&b| b) {
        return vec![false; bad.len()];
    }
    bad
}
