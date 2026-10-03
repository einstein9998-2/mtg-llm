//! Game records (doc 01 section 12): a game is a pure function of (engine version, card DB,
//! decks, seed, first player, action list). Replays are bit-identical.

use crate::game::*;
use mtg_core::card::CardDb;
use mtg_core::decision::*;
use mtg_core::ids::*;
use mtg_core::state::GameConfig;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameRecord {
    pub engine_version: u32,
    pub hash_schema: u32,
    pub card_db_hash: u64,
    pub decks: [Vec<u16>; 2],
    pub sides: [Vec<u16>; 2],
    pub seed: u64,
    pub first: u8,
    /// Starting life, hand size and whether ordinary mana abilities are explicit actions.
    pub starting_life: i32,
    pub hand_size: u8,
    pub explicit_mana: bool,
    /// (decision id, chosen index)
    pub actions: Vec<(u32, u16)>,
    /// (number of actions applied, state hash) checkpoints.
    pub checkpoints: Vec<(u32, u64)>,
}

impl GameRecord {
    pub fn new(db: &CardDb, decks: [&DeckList; 2], seed: u64, first: u8) -> GameRecord {
        GameRecord {
            engine_version: mtg_core::ENGINE_CORE_VERSION,
            hash_schema: mtg_core::HASH_SCHEMA,
            starting_life: GameConfig::default().starting_life,
            hand_size: GameConfig::default().hand_size,
            explicit_mana: false,
            card_db_hash: db.content_hash(),
            decks: [decks[0].main.iter().map(|d| d.0).collect(), decks[1].main.iter().map(|d| d.0).collect()],
            sides: [decks[0].side.iter().map(|d| d.0).collect(), decks[1].side.iter().map(|d| d.0).collect()],
            seed,
            first,
            actions: Vec::new(),
            checkpoints: Vec::new(),
        }
    }

    pub fn to_text(&self) -> String {
        let j = |v: &[u16]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",");
        let mut s = String::new();
        s += &format!("engine {}\nhash {}\ndb {:016x}\nseed {}\nfirst {}\nconfig {} {} {}\n", self.engine_version, self.hash_schema, self.card_db_hash, self.seed, self.first, self.starting_life, self.hand_size, self.explicit_mana as u8);
        s += &format!("deck0 {}\ndeck1 {}\nside0 {}\nside1 {}\n", j(&self.decks[0]), j(&self.decks[1]), j(&self.sides[0]), j(&self.sides[1]));
        s += "actions ";
        s += &self.actions.iter().map(|(i, a)| format!("{i}:{a}")).collect::<Vec<_>>().join(",");
        s += "\ncheckpoints ";
        s += &self.checkpoints.iter().map(|(n, h)| format!("{n}:{h:016x}")).collect::<Vec<_>>().join(",");
        s += "\n";
        s
    }

    pub fn from_text(t: &str) -> Result<GameRecord, String> {
        let mut r = GameRecord { engine_version: 0, hash_schema: 0, starting_life: 20, hand_size: 7, explicit_mana: false, card_db_hash: 0, decks: [vec![], vec![]], sides: [vec![], vec![]], seed: 0, first: 0, actions: vec![], checkpoints: vec![] };
        let list = |v: &str| -> Result<Vec<u16>, String> {
            if v.is_empty() {
                return Ok(vec![]);
            }
            v.split(',').map(|x| x.parse::<u16>().map_err(|e| e.to_string())).collect()
        };
        for line in t.lines() {
            let (k, v) = line.split_once(' ').unwrap_or((line, ""));
            match k {
                "engine" => r.engine_version = v.parse().map_err(|e: std::num::ParseIntError| e.to_string())?,
                "hash" => r.hash_schema = v.parse().map_err(|e: std::num::ParseIntError| e.to_string())?,
                "config" => {
                    let f: Vec<&str> = v.split(' ').collect();
                    if f.len() != 3 {
                        return Err("bad config".into());
                    }
                    r.starting_life = f[0].parse().map_err(|_| "bad life")?;
                    r.hand_size = f[1].parse().map_err(|_| "bad hand size")?;
                    r.explicit_mana = f[2] == "1";
                }
                "db" => r.card_db_hash = u64::from_str_radix(v, 16).map_err(|e| e.to_string())?,
                "seed" => r.seed = v.parse().map_err(|e: std::num::ParseIntError| e.to_string())?,
                "first" => r.first = v.parse().map_err(|e: std::num::ParseIntError| e.to_string())?,
                "deck0" => r.decks[0] = list(v)?,
                "deck1" => r.decks[1] = list(v)?,
                "side0" => r.sides[0] = list(v)?,
                "side1" => r.sides[1] = list(v)?,
                "actions" => {
                    if !v.is_empty() {
                        for p in v.split(',') {
                            let (i, a) = p.split_once(':').ok_or("bad action")?;
                            r.actions.push((i.parse().map_err(|_| "bad id")?, a.parse().map_err(|_| "bad idx")?));
                        }
                    }
                }
                "checkpoints" => {
                    if !v.is_empty() {
                        for p in v.split(',') {
                            let (n, h) = p.split_once(':').ok_or("bad checkpoint")?;
                            r.checkpoints.push((n.parse().map_err(|_| "bad n")?, u64::from_str_radix(h, 16).map_err(|_| "bad hash")?));
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(r)
    }

    pub fn decklists(&self) -> [DeckList; 2] {
        let f = |v: &Vec<u16>| v.iter().map(|&x| CardDefId(x)).collect::<Vec<_>>();
        [DeckList { main: f(&self.decks[0]), side: f(&self.sides[0]) }, DeckList { main: f(&self.decks[1]), side: f(&self.sides[1]) }]
    }
}

#[derive(Debug)]
pub enum ReplayError {
    EngineVersion { record: u32, engine: u32 },
    HashSchema { record: u32, engine: u32 },
    /// Checkpoints must be in ascending order, none past the end, the last at the final action count.
    BadCheckpoints(String),
    CardDbMismatch { record: u64, db: u64 },
    Apply { at: usize, err: ApplyError },
    CheckpointMismatch { at: u32, expected: u64, got: u64 },
    EndedEarly { at: usize },
}

/// Replays a record, verifying every checkpoint hash. Returns the finished game. A record whose
/// checkpoints are out of order, past the end, or missing the final one is refused: a replay that
/// verifies nothing must not pass.
pub fn replay(db: Arc<CardDb>, rec: &GameRecord) -> Result<Game, ReplayError> {
    if rec.engine_version != mtg_core::ENGINE_CORE_VERSION {
        return Err(ReplayError::EngineVersion { record: rec.engine_version, engine: mtg_core::ENGINE_CORE_VERSION });
    }
    if rec.hash_schema != mtg_core::HASH_SCHEMA {
        return Err(ReplayError::HashSchema { record: rec.hash_schema, engine: mtg_core::HASH_SCHEMA });
    }
    let h = db.content_hash();
    if rec.card_db_hash != h {
        return Err(ReplayError::CardDbMismatch { record: rec.card_db_hash, db: h });
    }
    let total = rec.actions.len() as u32;
    let mut last: Option<u32> = None;
    for &(n, _) in &rec.checkpoints {
        if n > total || last.map_or(false, |l| n <= l) {
            return Err(ReplayError::BadCheckpoints(format!("checkpoint at {n} is out of order or past the {total} actions")));
        }
        last = Some(n);
    }
    if last != Some(total) {
        return Err(ReplayError::BadCheckpoints(format!("no checkpoint at the final action count {total}")));
    }
    let decks = rec.decklists();
    let cfg = GameConfig { first_player: Seat(rec.first), starting_life: rec.starting_life, hand_size: rec.hand_size, explicit_mana: rec.explicit_mana };
    let mut g = Game::new(db, [&decks[0], &decks[1]], rec.seed, cfg);
    let mut cp = rec.checkpoints.iter().peekable();
    for (i, &(id, idx)) in rec.actions.iter().enumerate() {
        g.advance();
        if let Some(&&(n, h)) = cp.peek() {
            if n as usize == i {
                let got = g.hash_raw();
                if got != h {
                    return Err(ReplayError::CheckpointMismatch { at: n, expected: h, got });
                }
                cp.next();
            }
        }
        g.apply(DecisionId(id), idx as usize).map_err(|err| ReplayError::Apply { at: i, err })?;
    }
    g.advance();
    for &(n, h) in cp {
        debug_assert_eq!(n, total);
        let got = g.hash_raw();
        if got != h {
            return Err(ReplayError::CheckpointMismatch { at: n, expected: h, got });
        }
    }
    Ok(g)
}
