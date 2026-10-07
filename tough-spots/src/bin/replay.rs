//! Replays a flagged position's action list and prints what the bot saw over the last few decisions
//! (hand, known library top, events). Agent-side view only.
//! Usage: replay <decks dir> <positions-private.jsonl> <id> [last N decisions]
use mtg_agent::*;
use mtg_core::decision::Status;
use mtg_core::ids::Seat;
use mtg_core::state::GameConfig;
use mtg_view::*;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let (dir, file, id) = (&a[1], &a[2], &a[3]);
    let last: usize = a.get(4).and_then(|s| s.parse().ok()).unwrap_or(12);
    let line = std::fs::read_to_string(file).unwrap().lines().find(|l| l.contains(&format!("\"id\": \"{id}\""))).expect("id").to_string();
    let v: serde_json::Value = serde_json::from_str(&line).unwrap();
    let (gseed, bot, first) = (v["game_seed"].as_u64().unwrap(), v["bot_seat"].as_u64().unwrap() as u8, v["first"].as_u64().unwrap() as u8);
    let hist: Vec<(u8, usize)> = v["replay"].as_array().unwrap().iter().map(|x| (x[0].as_u64().unwrap() as u8, x[1].as_u64().unwrap() as usize)).collect();
    let db = mtg_cards::legacy::build();
    let (names, decks) = load_deck_dir(&db, dir);
    let (me, opp) = (names.iter().position(|n| n == "alurentell").unwrap(), names.iter().position(|n| n == "ur-cutter").unwrap());
    let (d0, d1) = if bot == 0 { (&decks[me], &decks[opp]) } else { (&decks[opp], &decks[me]) };
    let mut g = Game::new(db.clone(), [d0, d1], gseed, GameConfig { first_player: Seat(first), ..GameConfig::default() });
    let names_of = |c: &[ViewCard]| c.iter().map(|x| x.name.clone()).collect::<Vec<_>>().join(", ");
    for (i, (seat, idx)) in hist.iter().enumerate() {
        let st = g.advance();
        let Status::NeedDecision(s) = st else { break };
        assert_eq!(s.0, *seat, "replay diverged at {i}");
        let sv = g.seat_view(s);
        let o = sv.observe();
        let d = o.decision.clone().unwrap();
        if s == Seat(bot) && i + last >= hist.len() {
            println!("--- decision {i} (turn {}, {:?}) options {}: picked [{}] {}", o.turn, o.step, d.options.len(), idx, d.options[*idx].label);
            println!("  hand: {}", names_of(&o.me.hand));
            println!("  known top: {} | known bottom: {}", names_of(&o.me.library_known_top), names_of(&o.me.library_known_bottom));
            println!("  library {}", o.me.library_count);
            for e in &o.events {
                println!("    {:?}", e);
            }
        }
        g.apply(d.id, *idx).unwrap();
    }
}
