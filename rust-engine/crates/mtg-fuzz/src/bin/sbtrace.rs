//! Replays the games written by `bo3 --trace` and extracts, for seat 0 (deck A), what happened to
//! each card: when it was drawn, cast, entered the battlefield or was used up (Lotus Petal), and what
//! was still in hand at the end. One JSON line per game on stdout. Checks that every replay ends
//! with the recorded winner (the replay-determinism gate) and exits 1 otherwise.
//! Usage: sbtrace <games.jsonl> > features.jsonl
use mtg_core::event::Event as E;
use mtg_core::ids::{DecisionId, Seat};
use mtg_core::state::GameConfig;
use mtg_core::types::ZoneKind;
use mtg_view::{DeckList, Game};
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn deck(db: &mtg_core::card::CardDb, v: &Value) -> DeckList {
    let ids = |k: &str| v[k].as_array().unwrap().iter().map(|n| db.id(n.as_str().unwrap()).unwrap_or_else(|| panic!("card {n}"))).collect::<Vec<_>>();
    DeckList { main: ids("main"), side: ids("side") }
}

fn main() {
    let path = std::env::args().nth(1).expect("games.jsonl");
    let db = mtg_cards::legacy::build();
    let mut bad = 0;
    for line in std::fs::read_to_string(&path).unwrap().lines() {
        let v: Value = serde_json::from_str(line).unwrap();
        let decks = [deck(&db, &v["decks"][0]), deck(&db, &v["decks"][1])];
        let first = Seat(v["first"].as_u64().unwrap() as u8);
        let mut g = Game::new(db.clone(), [&decks[0], &decks[1]], v["seed"].as_u64().unwrap(), GameConfig { first_player: first, ..GameConfig::default() });
        g.set_keep_events(true);
        g.set_track_view_events(false);
        let actions: Vec<(u32, usize)> = v["actions"].as_array().unwrap().iter().map(|a| (a[0].as_u64().unwrap() as u32, a[1].as_u64().unwrap() as usize)).collect();
        let name = |d: mtg_core::ids::CardDefId| db.def(d).name.clone();
        // Seat 0 bookkeeping. `aturn` counts seat 0's own turns (0 = before its first turn).
        let (mut turn, mut aturn, mut total_turns) = (0u16, 0u32, 0u16);
        let mut drawn: BTreeMap<String, Vec<u32>> = BTreeMap::new();
        let mut cast: BTreeMap<String, Vec<u32>> = BTreeMap::new();
        let mut entered: BTreeMap<String, Vec<u32>> = BTreeMap::new();
        let mut used: BTreeMap<String, Vec<u32>> = BTreeMap::new();
        let mut hand: BTreeMap<String, i32> = BTreeMap::new();
        let mut timeline: Vec<String> = Vec::new();
        let mut opp_cast: Vec<String> = Vec::new();
        let mut result = None;
        let mut ai = 0usize;
        let mut winner: Option<u8> = None;
        loop {
            let st = g.advance();
            for e in g.take_events() {
                match e {
                    E::TurnBegan { turn: t, active } => {
                        turn = t;
                        total_turns = t;
                        if active.0 == 0 {
                            aturn += 1;
                        }
                    }
                    E::Drew { player, def, .. } if player.0 == 0 => {
                        drawn.entry(name(def)).or_default().push(aturn);
                    }
                    E::SpellCast { def, controller, .. } => {
                        if controller.0 == 0 {
                            cast.entry(name(def)).or_default().push(aturn);
                            timeline.push(format!("T{aturn} cast {}", name(def)));
                        } else if opp_cast.len() < 200 {
                            opp_cast.push(format!("t{turn} {}", name(def)));
                        }
                    }
                    E::ZoneChange { def, from, to, owner, controller, .. } => {
                        if owner.0 == 0 && to == ZoneKind::Hand && from != ZoneKind::Hand {
                            *hand.entry(name(def)).or_default() += 1;
                        }
                        if owner.0 == 0 && from == ZoneKind::Hand && to != ZoneKind::Stack {
                            *hand.entry(name(def)).or_default() -= 1;
                        }
                        if controller.0 == 0 && to == ZoneKind::Battlefield && from != ZoneKind::Stack {
                            entered.entry(name(def)).or_default().push(aturn);
                            timeline.push(format!("T{aturn} puts {} onto the battlefield", name(def)));
                        }
                        if controller.0 == 0 && from == ZoneKind::Battlefield && to == ZoneKind::Graveyard && name(def) == "Lotus Petal" {
                            used.entry(name(def)).or_default().push(aturn);
                            timeline.push(format!("T{aturn} uses Lotus Petal"));
                        }
                    }
                    _ => {}
                }
            }
            match st {
                mtg_core::decision::Status::GameOver(r) => {
                    result = Some(r);
                    break;
                }
                mtg_core::decision::Status::NeedDecision(seat) => {
                    let Some(&(id, idx)) = actions.get(ai) else { break };
                    ai += 1;
                    if g.apply(DecisionId(id), idx).is_err() {
                        eprintln!("replay: action {ai} rejected in match {} game {}", v["match"], v["game"]);
                        bad += 1;
                        break;
                    }
                    let _ = seat;
                }
            }
        }
        if let Some(mtg_core::decision::GameResult::Win(s)) = result {
            winner = Some(s.0);
        }
        let recorded = v["winner"].as_u64().map(|x| x as u8);
        if winner != recorded || (result.is_none() && recorded.is_some()) {
            eprintln!("replay mismatch in match {} game {}: recorded {:?}, replayed {:?}", v["match"], v["game"], recorded, winner);
            bad += 1;
        }
        let hand_end: BTreeMap<&String, i32> = hand.iter().filter(|(_, n)| **n > 0).map(|(k, n)| (k, *n)).collect();
        println!(
            "{}",
            json!({
                "match": v["match"], "game": v["game"], "first": v["first"], "winner": v["winner"], "boarded": v["boarded"],
                "turns": total_turns, "a_turns": aturn,
                "drawn": drawn, "cast": cast, "entered": entered, "used": used, "hand_end": hand_end,
                "timeline": timeline, "opp_cast": opp_cast,
            })
        );
    }
    if bad > 0 {
        eprintln!("{bad} replay problems");
        std::process::exit(1);
    }
}
