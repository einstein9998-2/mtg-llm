//! `mtg-core`: the rules engine core. Internal; agents use `mtg-view` only (doc 01 section 3.1).
#![forbid(unsafe_code)]

pub mod canary;
pub mod card;
pub mod compile;
pub mod ir;
pub mod lint;
pub mod cast;
pub mod combat;
pub mod cost;
pub mod cx;
pub mod decision;
pub mod derive;
pub mod dungeon;
pub mod engine;
pub mod eval;
pub mod event;
pub mod fork;
pub(crate) mod batch;
pub mod frame;
pub mod hash;
pub mod ids;
pub mod legal;
pub mod libfx;
pub mod replace;
pub mod restrict;
pub mod mana;
pub mod ops;
pub mod pregame;
pub mod priority;
pub mod read;
pub mod resolve;
pub mod scenario;
pub mod rng;
pub mod sba;
pub mod stabilize;
pub mod trigger;
pub mod state;
pub mod turn;
pub mod types;

/// Bumped by any reviewed core change (doc 01 section 14.2). Stored in every game record.
pub const ENGINE_CORE_VERSION: u32 = 6;

/// Bumped when the definition of the state hash changes (fields added, hashing order changed)
/// without a rules change, so a stale record can be told from a corrupt one.
pub const HASH_SCHEMA: u32 = 4;
