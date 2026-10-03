//! Hand-built test pool.

use mtg_core::card::*;
use mtg_core::ids::*;
use std::sync::Arc;

pub struct TestPool {
    pub db: Arc<CardDb>,
}

pub const TEST_CARDS: &str = include_str!("../cards/test.cards.ron");

pub fn build() -> TestPool {
    let db = mtg_dsl::build_db(&[("test.cards.ron", TEST_CARDS)]).unwrap_or_else(|e| panic!("test pool failed to load: {e}"));
    TestPool { db: Arc::new(db) }
}

impl TestPool {
    pub fn id(&self, name: &str) -> CardDefId {
        self.db.id(name).unwrap_or_else(|| panic!("no card {name}"))
    }

    /// Builds a 60-card list from (name, count) pairs, padded with basics of the first land.
    pub fn deck(&self, list: &[(&str, usize)]) -> Vec<CardDefId> {
        let mut v = Vec::new();
        for (n, c) in list {
            for _ in 0..*c {
                v.push(self.id(n));
            }
        }
        assert!(v.len() <= 60, "deck has {} cards", v.len());
        let pad = v[0];
        while v.len() < 60 {
            v.push(pad);
        }
        v
    }

    pub fn deck_red_green(&self) -> Vec<CardDefId> {
        self.deck(&[
            ("Mountain", 11),
            ("Forest", 11),
            ("Llanowar Elves", 3),
            ("Grizzly Bears", 4),
            ("Hill Giant", 3),
            ("Goblin Piker", 3),
            ("Raging Goblin", 3),
            ("Craw Wurm", 2),
            ("Rhox", 3),
            ("Giant Spider", 2),
            ("Wall of Stone", 1),
            ("Lightning Bolt", 4),
            ("Prey Upon-ish Strike", 3),
            ("Wasteland Stand-in", 3),
            ("Divination", 0),
        ])
    }

    pub fn deck_white_black(&self) -> Vec<CardDefId> {
        self.deck(&[
            ("Plains", 11),
            ("Swamp", 11),
            ("White Knight", 4),
            ("Fencing Ace", 3),
            ("Typhoid Rats", 4),
            ("Vampire Nighthawk", 3),
            ("Serra Angel", 3),
            ("Gigapede-ish Menace", 3),
            ("Anthem Banner", 2),
            ("Doom Blade", 4),
            ("Mind Rot", 3),
            ("Raise the Alarm", 3),
            ("Healing Salve", 3),
            ("Wall of Stone", 0),
        ])
    }

    pub fn deck_blue(&self) -> Vec<CardDefId> {
        self.deck(&[
            ("Island", 12),
            ("Mountain", 8),
            ("Wind Drake", 4),
            ("Invisible Stalker", 3),
            ("Looter Bear", 3),
            ("Prodigal Sorcerer", 4),
            ("Counterspell", 4),
            ("Unsummon", 4),
            ("Divination", 4),
            ("Lightning Bolt", 4),
            ("Hill Giant", 4),
            ("Craw Wurm", 2),
            ("Mind Rot", 4),
        ])
    }

    /// Exercises modes, X, alternative and additional costs, Phyrexian mana, cost taxes, flashback.
    pub fn deck_tricks(&self) -> Vec<CardDefId> {
        self.deck(&[
            ("Island", 7),
            ("Mountain", 6),
            ("Swamp", 5),
            ("Plains", 3),
            ("Test Charm", 4),
            ("Fireball Stand-in", 3),
            ("Daze Stand-in", 4),
            ("Force Stand-in", 4),
            ("Triumph Stand-in", 3),
            ("Dismember Stand-in", 3),
            ("Tax Collector", 3),
            ("Flash Burn", 4),
            ("Lightning Bolt", 3),
            ("Counterspell", 3),
            ("Grizzly Bears", 4),
            ("Looter Bear", 1),
        ])
    }

    /// Exercises triggered abilities: ETB, dies, from-graveyard, spell-cast, upkeep, intervening-if.
    pub fn deck_triggers(&self) -> Vec<CardDefId> {
        self.deck(&[
            ("Plains", 4),
            ("Island", 4),
            ("Swamp", 5),
            ("Mountain", 6),
            ("Soul Warden Stand-in", 4),
            ("Mulldrifter Stand-in", 3),
            ("Ping Elemental", 4),
            ("Blood Artist Stand-in", 4),
            ("Pyromancer Stand-in", 4),
            ("Arena Stand-in", 2),
            ("Crowd Pleaser", 3),
            ("Grave Warden", 4),
            ("Lightning Bolt", 4),
            ("Doom Blade", 3),
            ("Raise the Alarm", 3),
            ("Unsummon", 3),
        ])
    }
}
