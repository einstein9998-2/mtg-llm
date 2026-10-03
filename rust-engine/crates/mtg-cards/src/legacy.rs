//! The real Legacy card pool (RON), built up card by card against the spec scenarios.

use mtg_core::card::CardDb;
use std::sync::Arc;

pub const LEGACY_CARDS: &str = include_str!("../cards/legacy.cards.ron");

pub fn build() -> Arc<CardDb> {
    let db = mtg_dsl::build_db(&[("legacy.cards.ron", LEGACY_CARDS)]).unwrap_or_else(|e| panic!("legacy pool failed to load: {e}"));
    Arc::new(db)
}
