//! Differential testing harness for the Rust engine against Forge (doc 04).
//!
//! `scn`: runs Forge-interaction-test scenarios (`.scn`, written against Forge) on the Rust engine.
//! `util`: engine-side naming helpers shared by the scenario runner and the lockstep driver.

pub mod util;
pub mod scn;
pub mod pos;
pub mod step;
pub mod line;
