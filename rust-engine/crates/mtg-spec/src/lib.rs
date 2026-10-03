//! Scenario runner for the spec thread's rulings-as-spec scenarios (see
//! `/mnt/project-files/rust-engine-spec/SCHEMA.md`). Scenarios arrive as JSON (converted from YAML
//! by `tools/spec2json.py`); this crate never reads the YAML and never edits the spec folder.
//!
//! Scenario mode runs with auto-resolution off: the runner answers engine decisions only as the
//! script says. Macro actions (`cast`, `activate`, ...) are expanded here into the engine's
//! sequence of decisions; the sub-decisions of a cast are answered from the action's fields.

pub mod expect;
pub mod runner;

pub use runner::{run_scenario, Outcome};

pub(crate) type J = serde_json::Value;

/// A scenario failure: either the engine misbehaved (`Fail`) or the scenario needs something the
/// engine or runner does not support yet (`Unsupported`).
#[derive(Debug, Clone)]
pub enum Err {
    Fail(String),
    Unsupported(String),
}

pub(crate) type R<T> = Result<T, Err>;

#[macro_export]
macro_rules! fail {
    ($($a:tt)*) => { return Err($crate::Err::Fail(format!($($a)*))) };
}

#[macro_export]
macro_rules! unsupported {
    ($($a:tt)*) => { return Err($crate::Err::Unsupported(format!($($a)*))) };
}
