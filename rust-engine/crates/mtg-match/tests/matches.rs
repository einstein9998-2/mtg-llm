use mtg_core::decision::Status;
use mtg_core::fork::HiddenRequest;
use mtg_core::ids::{CardDefId, Seat};
use mtg_core::rng::Pcg64;
use mtg_match::*;
use mtg_view::{BeliefModel, DeckList, Policy, RandomPolicy};
use std::path::Path;

fn root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

fn load() -> (std::sync::Arc<mtg_core::card::CardDb>, Vec<String>, Vec<DeckList>, PlanBook) {
    let db = mtg_cards::legacy::build();
    let mut names = vec![];
    let mut decks = vec![];
    let mut paths: Vec<_> = std::fs::read_dir(root().join("decks")).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().map_or(false, |x| x == "txt")).collect();
    paths.sort();
    for p in paths {
        names.push(p.file_stem().unwrap().to_string_lossy().to_string());
        decks.push(load_deck(&db, &p));
    }
    let book = PlanBook::load_dir(&db, &root().join("sideboard-plans")).unwrap();
    (db, names, decks, book)
}

fn load_deck(db: &mtg_core::card::CardDb, p: &Path) -> DeckList {
    let (mut main, mut side, mut in_side, mut seen) = (vec![], vec![], false, false);
    for l in std::fs::read_to_string(p).unwrap().lines().map(str::trim) {
        if l.starts_with('#') {
            continue;
        }
        if l.is_empty() {
            in_side |= seen;
            continue;
        }
        let (n, name) = l.split_once(' ').unwrap();
        let id = db.id(name.trim()).unwrap_or_else(|| panic!("card {name}"));
        for _ in 0..n.parse::<usize>().unwrap() {
            if in_side { side.push(id) } else { main.push(id) }
        }
        seen |= !in_side;
    }
    DeckList { main, side }
}

#[test]
fn shipped_plans_are_legal_and_keep_60_15() {
    let (db, names, decks, book) = load();
    assert!(!book.is_empty());
    for (own, opp, _side, plan) in book.entries() {
        let i = names.iter().position(|n| *n == own).unwrap_or_else(|| panic!("plan for unknown deck {own}"));
        assert!(names.contains(&opp) || opp == "*", "{own}: unknown opponent {opp}");
        let d = board(&db, &decks[i], plan).unwrap_or_else(|e| panic!("{own} vs {opp}: {e}"));
        assert_eq!((d.main.len(), d.side.len()), (60, 15), "{own} vs {opp}");
        // The 75 cards are the same cards.
        let mut a: Vec<_> = d.main.iter().chain(d.side.iter()).copied().collect();
        let mut b: Vec<_> = decks[i].main.iter().chain(decks[i].side.iter()).copied().collect();
        a.sort();
        b.sort();
        assert_eq!(a, b);
    }
}

#[test]
fn board_rejects_bad_plans() {
    let (db, names, decks, _) = load();
    let a = &decks[names.iter().position(|n| n == "alurentell").unwrap()];
    let id = |n: &str| db.id(n).unwrap();
    let plan = |out: &[&str], inn: &[&str]| Plan { out: out.iter().map(|n| id(n)).collect(), inn: inn.iter().map(|n| id(n)).collect(), oversize: false };
    // Not one for one.
    assert!(board(&db, a, &plan(&["Stock Up"], &[])).is_err());
    // The card to take out is not in the main deck.
    assert!(board(&db, a, &plan(&["Lightning Bolt"], &["Prismatic Ending"])).is_err());
    // The card to bring in is not in the sideboard.
    assert!(board(&db, a, &plan(&["Stock Up"], &["Lightning Bolt"])).is_err());
    // Only one Force of Negation in the sideboard.
    assert!(board(&db, a, &plan(&["Stock Up", "Stock Up"], &["Force of Negation", "Force of Negation"])).is_err());
    // Both sideboard Veils in: four in the main deck, still four copies in all.
    assert!(board(&db, a, &plan(&["Stock Up", "Stock Up"], &["Veil of Summer", "Veil of Summer"])).is_ok());
    // A plan file with an unknown card is rejected at load time.
    let dir = std::env::temp_dir().join(format!("mtg-match-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("x.txt"), "vs y\n-1 No Such Card\n").unwrap();
    assert!(PlanBook::load_dir(&db, &dir).is_err());
    std::fs::write(dir.join("x.txt"), "-1 Stock Up\n").unwrap();
    assert!(PlanBook::load_dir(&db, &dir).is_err(), "a swap before any vs line");
    std::fs::remove_dir_all(&dir).unwrap();
}

fn req(known: [Vec<CardDefId>; 2], unknown: [usize; 2]) -> HiddenRequest {
    let mut known = known;
    known[0].sort();
    known[1].sort();
    HiddenRequest { observer: Seat(0), unknown, known }
}

#[test]
fn expected_model_surprise_displaces_an_unseen_card() {
    let (db, _, _, _) = load();
    let id = |n: &str| db.id(n).unwrap();
    let (bolt, heat, pyro) = (id("Lightning Bolt"), id("Unholy Heat"), id("Pyroblast"));
    let m = ExpectedModel { decks: [vec![bolt, bolt, heat], vec![bolt, bolt, heat]], exact: [true, false] };
    let mut rng = Pcg64::from_seed(3);
    // Seat 1: a Pyroblast was seen but is not in the expected list: one of the three unseen
    // expected cards makes room for it, so two stay hidden.
    let r = req([vec![], vec![pyro]], [3, 2]);
    for _ in 0..50 {
        let a = m.sample(&r, &mut rng).unwrap();
        assert_eq!(a.defs[0].len(), 3);
        assert_eq!(a.defs[1].len(), 2);
        assert!(a.defs[1].iter().all(|c| *c == bolt || *c == heat));
    }
    // An exact seat must not accept a surprise.
    let r = req([vec![pyro], vec![]], [2, 3]);
    assert!(m.sample(&r, &mut rng).is_err());
    // With no surprises it is the plain consistent model: the unseen cards are exactly the rest.
    let r = req([vec![], vec![heat]], [3, 2]);
    let a = m.sample(&r, &mut rng).unwrap();
    let mut got = a.defs[1].clone();
    got.sort();
    assert_eq!(got, vec![bolt, bolt]);
    // Every sample over many draws sometimes drops the Heat and sometimes a Bolt.
    let r = req([vec![], vec![pyro]], [3, 2]);
    let (mut dropped_heat, mut dropped_bolt) = (false, false);
    for _ in 0..200 {
        let a = m.sample(&r, &mut rng).unwrap();
        if !a.defs[1].contains(&heat) { dropped_heat = true } else { dropped_bolt = true }
    }
    assert!(dropped_heat && dropped_bolt);
}

struct RandFac;

impl PlayerFactory for RandFac {
    fn make<'m>(&mut self, seat: Seat, game_no: u32, _model: &'m dyn BeliefModel, seed: u64) -> Box<dyn Policy + 'm> {
        Box::new(RandomPolicy::new(seed ^ (seat.0 as u64 + 1) * 977 ^ game_no as u64))
    }
}

#[test]
fn match_structure_first_player_and_boarding() {
    let (db, names, decks, book) = load();
    let (a, b) = (names.iter().position(|n| n == "alurentell").unwrap(), names.iter().position(|n| n == "ur-cutter").unwrap());
    let mut seen_three = false;
    for seed in 0..40u64 {
        let cfg = MatchConfig { seed, first: Seat((seed % 2) as u8), max_decisions: 20000, ..MatchConfig::default() };
        let r = play_match(&db, ["alurentell", "ur-cutter"], [&decks[a], &decks[b]], [Some(&book), Some(&book)], [Some(&book), Some(&book)], &cfg, &mut RandFac).unwrap();
        assert!(r.games.len() >= 2 && r.games.len() <= 5);
        assert_eq!(r.wins[0] as usize, r.games.iter().filter(|g| g.winner == Some(Seat(0))).count());
        assert!(r.wins[0] <= 2 && r.wins[1] <= 2);
        seen_three |= r.games.len() == 3;
        assert_eq!(r.games[0].first, cfg.first);
        assert_eq!(r.games[0].boarded, [false, false], "game one is never boarded");
        for w in r.games.windows(2) {
            if let Some(win) = w[0].winner {
                assert_eq!(w[1].first, win.other(), "the loser plays first");
            }
            assert_eq!(w[1].boarded, [true, true]);
        }
        if r.wins[0] < 2 && r.wins[1] < 2 {
            assert_eq!(r.games.len() as u32, cfg.max_games);
        }
    }
    assert!(seen_three, "no match went to three games in 40 tries");
    // --board a: only seat 0 boards; a pair without a plan never boards.
    let cfg = MatchConfig { board: [true, false], seed: 5, max_decisions: 20000, ..MatchConfig::default() };
    let r = play_match(&db, ["alurentell", "ur-cutter"], [&decks[a], &decks[b]], [Some(&book), Some(&book)], [Some(&book), Some(&book)], &cfg, &mut RandFac).unwrap();
    assert!(r.games.iter().skip(1).all(|g| g.boarded == [true, false]));
    let (d, e) = (names.iter().position(|n| n == "dimir-tempo").unwrap(), names.iter().position(|n| n == "boros-aggro").unwrap());
    let r = play_match(&db, ["dimir-tempo", "boros-aggro"], [&decks[d], &decks[e]], [Some(&book), Some(&book)], [Some(&book), Some(&book)], &MatchConfig { seed: 9, max_decisions: 20000, ..MatchConfig::default() }, &mut RandFac).unwrap();
    assert!(r.games.iter().all(|g| g.boarded == [false, false]), "no plan for this pair");
}

/// The belief model must be able to fork at every decision of a game in which a seat has
/// sideboarded, whatever the observer was told about the opponent's list.
#[test]
fn forks_work_in_boarded_games_under_both_opponent_views() {
    let (db, names, decks, book) = load();
    for (pair, view) in [("ur-cutter", OppView::Main), ("ur-cutter", OppView::Plan), ("uwx-control", OppView::Main), ("reanimator", OppView::Plan)] {
        let a = names.iter().position(|n| n == "alurentell").unwrap();
        let b = names.iter().position(|n| n == pair).unwrap();
        let cfg = MatchConfig::default();
        let (boarded, flags) = game_decks(&db, ["alurentell", pair], [&decks[a], &decks[b]], [Some(&book), Some(&book)], &cfg, 1, Seat(0)).unwrap();
        assert_eq!(flags, [true, true]);
        for seed in 0..6u64 {
            let mut g = mtg_view::Game::new(db.clone(), [&boarded[0], &boarded[1]], seed, mtg_core::state::GameConfig::default());
            let mut rng = RandomPolicy::new(seed + 1);
            let mut forks = 0;
            for _ in 0..4000 {
                match g.advance() {
                    Status::GameOver(_) => break,
                    Status::NeedDecision(seat) => {
                        let mine = &boarded[seat.idx()];
                        let opp = &boarded[seat.other().idx()];
                        let believed = match view {
                            OppView::Plan => opp.main.clone(),
                            OppView::Main => decks[if seat.idx() == 0 { b } else { a }].main.clone(),
                        };
                        let mut dd = [Vec::new(), Vec::new()];
                        dd[seat.idx()] = mine.main.clone();
                        dd[seat.other().idx()] = believed;
                        let model = ExpectedModel { decks: dd, exact: [seat.idx() == 0, seat.idx() == 1] };
                        let sv = g.seat_view(seat);
                        let w = sv.fork(seed * 31 + forks, &model);
                        assert!(w.is_ok(), "fork failed ({pair}, {view:?}, seed {seed}): {:?}", w.err());
                        forks += 1;
                        let d = sv.decision().unwrap();
                        let i = rng.choose(&sv, d.options.len());
                        g.apply(d.id, i).unwrap();
                    }
                }
            }
            assert!(forks > 20);
        }
    }
}

#[test]
fn play_draw_sections_win_over_plain_and_star() {
    let (db, _, decks, _) = load();
    let dir = std::env::temp_dir().join(format!("mtg-match-pd-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("alurentell.txt"), "vs ur-cutter\n-1 Stock Up\n+1 Prismatic Ending\nvs ur-cutter on-draw\n-1 Ponder\n+1 Dismember\nvs *\n-1 Brainstorm\n+1 Dismember\n").unwrap();
    let book = PlanBook::load_dir(&db, &dir).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    let out = |p: Option<&Plan>| db.def(p.unwrap().out[0]).name.clone();
    assert_eq!(out(book.plan_for("alurentell", "ur-cutter", Some(true))), "Stock Up");
    assert_eq!(out(book.plan_for("alurentell", "ur-cutter", Some(false))), "Ponder");
    assert_eq!(out(book.plan_for("alurentell", "ur-cutter", None)), "Stock Up");
    assert_eq!(out(book.plan_for("alurentell", "doomsday", Some(false))), "Brainstorm");
    let _ = decks;
}

#[test]
fn oversize_plans_and_separate_books() {
    let (db, names, decks, _) = load();
    let a = &decks[names.iter().position(|n| n == "alurentell").unwrap()];
    let id = |n: &str| db.id(n).unwrap();
    let mut p = Plan { out: vec![id("Stock Up")], inn: vec![id("Prismatic Ending"), id("Faerie Macabre")], oversize: false };
    assert!(board(&db, a, &p).is_err(), "61 cards needs the directive");
    p.oversize = true;
    let d = board(&db, a, &p).unwrap();
    assert_eq!((d.main.len(), d.side.len()), (61, 14));
    // Two books: seat 0 boards, seat 1 has none.
    let dir = std::env::temp_dir().join(format!("mtg-match-books-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("alurentell.txt"), "vs alurentell\n-1 Stock Up\n+1 Prismatic Ending\n").unwrap();
    let book = PlanBook::load_dir(&db, &dir).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    let (decks2, flags) = game_decks(&db, ["alurentell", "alurentell"], [a, a], [Some(&book), None], &MatchConfig::default(), 1, Seat(0)).unwrap();
    assert_eq!(flags, [true, false]);
    assert_eq!(decks2[0].main.len(), 60);
    assert_ne!(decks2[0].main, decks2[1].main);
}

/// A plan under test must not reach the opponent's belief: the searching seat expects the public
/// plan's list, not the list the variant actually plays.
#[test]
fn belief_comes_from_the_public_plan_not_the_played_list() {
    let (db, names, decks, public) = load();
    let ai = names.iter().position(|n| n == "alurentell").unwrap();
    let bi = names.iter().position(|n| n == "ur-cutter").unwrap();
    let base = [&decks[ai], &decks[bi]];
    // A variant for Alurentell that differs from the public plan: one more Prismatic Ending, one fewer Stock Up.
    let dir = std::env::temp_dir().join(format!("mtg-match-variant-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("alurentell.txt"), "vs ur-cutter\n-1 Stock Up\n+1 Prismatic Ending\n").unwrap();
    let variant = PlanBook::load_dir(&db, &dir).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    let cfg = MatchConfig::default();
    let names2 = ["alurentell", "ur-cutter"];
    let (played, _) = game_decks(&db, names2, base, [Some(&variant), Some(&public)], &cfg, 1, Seat(0)).unwrap();
    let (pub_decks, _) = game_decks(&db, names2, base, [Some(&public), Some(&public)], &cfg, 1, Seat(0)).unwrap();
    assert_ne!(played[0].main, pub_decks[0].main, "the variant must differ from the public plan");
    let m = beliefs(base, &played, &pub_decks, OppView::Plan);
    let sorted = |v: &Vec<_>| { let mut v = v.clone(); v.sort(); v };
    // Seat 1 (UR) expects the public Alurentell list, not the played variant; seat 0 knows its own list exactly.
    assert_eq!(sorted(&m[1].decks[0]), sorted(&pub_decks[0].main));
    assert_ne!(sorted(&m[1].decks[0]), sorted(&played[0].main));
    assert_eq!(sorted(&m[0].decks[0]), sorted(&played[0].main));
    assert!(m[1].exact[1] && !m[1].exact[0]);
    // Main view: the opponent's game-one list.
    let mm = beliefs(base, &played, &pub_decks, OppView::Main);
    assert_eq!(sorted(&mm[1].decks[0]), sorted(&base[0].main));
}
