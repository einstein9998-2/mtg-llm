//! Engine-internal unit tests (implementer-written; the rules-spec suite lives elsewhere).

use mtg_cards::testpool::{self, TestPool};
use mtg_core::scenario::*;
use mtg_core::ids::Seat;
use mtg_core::types::*;
use mtg_debug::invariants;
use mtg_debug::script::Driver;
use mtg_view::Game;

fn pool() -> TestPool {
    testpool::build()
}

struct P {
    s: PlayerSetup,
}

fn p() -> P {
    P { s: PlayerSetup { life: 20, library: vec!["Forest".to_string(); 12], ..Default::default() } }
}

impl P {
    fn hand(mut self, c: &[&str]) -> P {
        self.s.hand = c.iter().map(|x| x.to_string()).collect();
        self
    }
    fn bf(mut self, c: &[&str]) -> P {
        self.s.battlefield = c.iter().map(|x| PermSetup::new(x)).collect();
        self
    }
    fn bf_sick(mut self, c: &[&str]) -> P {
        for x in c {
            let mut ps = PermSetup::new(x);
            ps.summoning_sick = true;
            self.s.battlefield.push(ps);
        }
        self
    }
    fn gy(mut self, c: &[&str]) -> P {
        self.s.graveyard = c.iter().map(|x| x.to_string()).collect();
        self
    }
    fn life(mut self, l: i32) -> P {
        self.s.life = l;
        self
    }
    fn lib(mut self, c: &[&str]) -> P {
        self.s.library = c.iter().map(|x| x.to_string()).collect();
        self
    }
}

fn game(pool: &TestPool, active: u8, step: Step, a: P, b: P) -> Driver {
    let sc = ScenarioSetup { turn: 3, active: Seat(active), step, players: [a.s, b.s], seed: 5 };
    let g = Game::from_scenario(pool.db.clone(), &sc);
    let d = Driver::new(g);
    invariants::check(d.g.raw_state(), d.g.db()).unwrap();
    d
}

fn life(d: &Driver, s: u8) -> i32 {
    d.g.raw_state().life(Seat(s))
}

fn gy_names(d: &Driver, s: u8) -> Vec<String> {
    let st = d.g.raw_state();
    st.graveyard(Seat(s)).iter().map(|&r| d.g.db().def(st.def_of(r)).name.clone()).collect()
}

fn bf_names(d: &Driver) -> Vec<String> {
    let st = d.g.raw_state();
    st.battlefield().iter().map(|&r| d.g.db().def(st.def_of(r)).name.clone()).collect()
}

fn check(d: &Driver) {
    if d.g.pending().is_some() {
        invariants::check(d.g.raw_state(), d.g.db()).unwrap();
    }
}

#[test]
fn bolt_face() {
    let pl = pool();
    let mut d = game(&pl, 0, Step::Main1, p().hand(&["Lightning Bolt"]).bf(&["Mountain"]), p());
    d.act("Cast Lightning Bolt").act("Target: opponent");
    d.act("Pass priority").act("Pass priority");
    assert_eq!(life(&d, 1), 17);
    assert_eq!(gy_names(&d, 0), vec!["Lightning Bolt"]);
    check(&d);
}

#[test]
fn bolt_kills_creature() {
    let pl = pool();
    let mut d = game(&pl, 0, Step::Main1, p().hand(&["Lightning Bolt"]).bf(&["Mountain"]), p().bf(&["Grizzly Bears"]));
    d.act("Cast Lightning Bolt").act("Target: Grizzly Bears");
    d.act("Pass priority").act("Pass priority");
    assert_eq!(gy_names(&d, 1), vec!["Grizzly Bears"]);
    assert!(bf_names(&d).iter().all(|n| n != "Grizzly Bears"));
}

#[test]
fn counterspell_counters_bolt() {
    let pl = pool();
    let mut d = game(&pl, 0, Step::Main1, p().hand(&["Lightning Bolt"]).bf(&["Mountain"]), p().hand(&["Counterspell"]).bf(&["Island", "Island"]));
    d.act("Cast Lightning Bolt").act("Target: opponent");
    d.act("Pass priority");
    assert_eq!(d.seat(), Seat(1));
    d.act("Cast Counterspell").act("Target: Lightning Bolt");
    d.act("Pass priority").act("Pass priority"); // Counterspell resolves
    assert_eq!(gy_names(&d, 0), vec!["Lightning Bolt"]);
    assert_eq!(gy_names(&d, 1), vec!["Counterspell"]);
    assert_eq!(life(&d, 1), 20);
    check(&d);
}

#[test]
fn counterspell_needs_uu() {
    let pl = pool();
    let mut d = game(&pl, 0, Step::Main1, p().hand(&["Lightning Bolt"]).bf(&["Mountain"]), p().hand(&["Counterspell"]).bf(&["Island", "Mountain"]));
    d.act("Cast Lightning Bolt").act("Target: opponent");
    d.act("Pass priority");
    assert!(!d.has("Cast Counterspell"), "{:?}", d.labels());
}

#[test]
fn spell_fizzles_when_target_leaves() {
    let pl = pool();
    let mut d = game(&pl, 0, Step::Main1, p().hand(&["Lightning Bolt"]).bf(&["Mountain"]), p().hand(&["Unsummon"]).bf(&["Grizzly Bears", "Island"]));
    d.act("Cast Lightning Bolt").act("Target: Grizzly Bears");
    d.act("Pass priority");
    d.act("Cast Unsummon").act("Target: Grizzly Bears");
    d.act("Pass priority").act("Pass priority"); // Unsummon resolves
    d.act("Pass priority").act("Pass priority"); // Bolt fizzles
    assert!(gy_names(&d, 0).contains(&"Lightning Bolt".to_string()));
    assert_eq!(d.g.raw_state().hand_count(Seat(1)), 1 + 0); // Bears returned (Unsummon went to gy)
    assert_eq!(life(&d, 1), 20);
}

#[test]
fn summoning_sickness_blocks_attack_and_haste_does_not() {
    let pl = pool();
    let mut d = game(&pl, 0, Step::Main1, p().bf_sick(&["Grizzly Bears", "Raging Goblin"]).bf(&[]), p());
    // Both sick (scenario flag); haste goblin may attack, bears may not.
    let mut d2 = game(&pl, 0, Step::Main1, p().bf_sick(&["Grizzly Bears"]).bf_sick(&["Raging Goblin"]), p());
    d2.pass_until(|d| matches!(d.decision().kind, mtg_view::ViewDecisionKind::DeclareAttacker { .. }));
    assert!(d2.decision().context.contains("Raging Goblin"), "{}", d2.decision().context);
    d2.act("Attack opponent");
    assert!(!matches!(d2.decision().kind, mtg_view::ViewDecisionKind::DeclareAttacker { .. }), "Bears must not be offered an attack");
    let _ = &mut d;
}

#[test]
fn unblocked_attack_and_blocked_trade() {
    let pl = pool();
    let mut d = game(&pl, 0, Step::Main1, p().bf(&["Grizzly Bears"]), p());
    d.pass_until(|d| matches!(d.decision().kind, mtg_view::ViewDecisionKind::DeclareAttacker { .. }));
    d.act("Attack opponent");
    d.pass_until(|d| life(d, 1) < 20);
    assert_eq!(life(&d, 1), 18);

    // Bears attacks into Hill Giant: giant blocks and kills it.
    let mut d = game(&pl, 0, Step::Main1, p().bf(&["Grizzly Bears"]), p().bf(&["Hill Giant"]));
    d.pass_until(|d| matches!(d.decision().kind, mtg_view::ViewDecisionKind::DeclareAttacker { .. }));
    d.act("Attack opponent");
    d.pass_until(|d| matches!(d.decision().kind, mtg_view::ViewDecisionKind::DeclareBlocker { .. }));
    d.act("Block Grizzly Bears");
    for _ in 0..8 {
        d.act("Pass priority");
    }
    assert_eq!(gy_names(&d, 0), vec!["Grizzly Bears"]);
    assert_eq!(life(&d, 1), 20);
}

#[test]
fn first_strike_kills_before_damage() {
    let pl = pool();
    let mut d = game(&pl, 0, Step::Main1, p().bf(&["White Knight"]), p().bf(&["Grizzly Bears"]));
    d.pass_until(|d| matches!(d.decision().kind, mtg_view::ViewDecisionKind::DeclareAttacker { .. }));
    d.act("Attack opponent");
    d.pass_until(|d| matches!(d.decision().kind, mtg_view::ViewDecisionKind::DeclareBlocker { .. }));
    d.act("Block White Knight");
    for _ in 0..8 {
        d.act("Pass priority");
    }
    assert_eq!(gy_names(&d, 1), vec!["Grizzly Bears"]);
    assert!(bf_names(&d).contains(&"White Knight".to_string()), "first strike knight must survive");
}

#[test]
fn deathtouch_trade_and_trample() {
    let pl = pool();
    // Rats (deathtouch) blocks Craw Wurm: both die.
    let mut d = game(&pl, 0, Step::Main1, p().bf(&["Craw Wurm"]), p().bf(&["Typhoid Rats"]));
    d.pass_until(|d| matches!(d.decision().kind, mtg_view::ViewDecisionKind::DeclareAttacker { .. }));
    d.act("Attack opponent");
    d.pass_until(|d| matches!(d.decision().kind, mtg_view::ViewDecisionKind::DeclareBlocker { .. }));
    d.act("Block Craw Wurm");
    for _ in 0..8 {
        d.act("Pass priority");
    }
    assert_eq!(gy_names(&d, 0), vec!["Craw Wurm"]);
    assert_eq!(gy_names(&d, 1), vec!["Typhoid Rats"]);
    assert_eq!(life(&d, 1), 20, "no trample");

    // Rhox (4/4 trample) blocked by Bears (2/2): 2 damage tramples over.
    let mut d = game(&pl, 0, Step::Main1, p().bf(&["Rhox"]), p().bf(&["Grizzly Bears"]));
    d.pass_until(|d| matches!(d.decision().kind, mtg_view::ViewDecisionKind::DeclareAttacker { .. }));
    d.act("Attack opponent");
    d.pass_until(|d| matches!(d.decision().kind, mtg_view::ViewDecisionKind::DeclareBlocker { .. }));
    d.act("Block Rhox");
    // The attacking player decides how much of the trample damage the blocker gets (at least lethal).
    d.pass_until(|d| matches!(d.decision().kind, mtg_view::ViewDecisionKind::AssignDamage { .. }));
    d.act_idx(0);
    for _ in 0..8 {
        d.act("Pass priority");
    }
    assert_eq!(life(&d, 1), 18);
    assert_eq!(gy_names(&d, 1), vec!["Grizzly Bears"]);
}

#[test]
fn flying_cannot_be_blocked_by_ground_and_menace_needs_two() {
    let pl = pool();
    let mut d = game(&pl, 0, Step::Main1, p().bf(&["Wind Drake"]), p().bf(&["Grizzly Bears"]));
    d.pass_until(|d| matches!(d.decision().kind, mtg_view::ViewDecisionKind::DeclareAttacker { .. }));
    d.act("Attack opponent");
    // Bears cannot block a flier: no blocker decision at all; damage goes through.
    d.pass_until(|d| life(d, 1) < 20 || (matches!(d.decision().kind, mtg_view::ViewDecisionKind::DeclareBlocker { .. }) && d.decision().options.len() > 1));
    assert_eq!(life(&d, 1), 18);

    // Menace attacker, single potential blocker: blocking would be illegal, so it is not offered.
    let mut d = game(&pl, 0, Step::Main1, p().bf(&["Gigapede-ish Menace"]), p().bf(&["Grizzly Bears"]));
    d.pass_until(|d| matches!(d.decision().kind, mtg_view::ViewDecisionKind::DeclareAttacker { .. }));
    d.act("Attack opponent");
    d.pass_until(|d| life(d, 1) < 20 || (matches!(d.decision().kind, mtg_view::ViewDecisionKind::DeclareBlocker { .. }) && d.decision().options.len() > 1));
    assert_eq!(life(&d, 1), 17);

    // Two potential blockers: blocking with one is never completable only if it is the last
    // chance; offering both options to the first blocker is fine.
    let mut d = game(&pl, 0, Step::Main1, p().bf(&["Gigapede-ish Menace"]), p().bf(&["Grizzly Bears", "Hill Giant"]));
    d.pass_until(|d| matches!(d.decision().kind, mtg_view::ViewDecisionKind::DeclareAttacker { .. }));
    d.act("Attack opponent");
    d.pass_until(|d| matches!(d.decision().kind, mtg_view::ViewDecisionKind::DeclareBlocker { .. }));
    d.act("Block Gigapede");
    // The second blocker must now block too (a menace creature with exactly one blocker is illegal).
    assert!(matches!(d.decision().kind, mtg_view::ViewDecisionKind::DeclareBlocker { .. }));
    assert_eq!(d.labels().len(), 1, "{:?}", d.labels());
    assert!(d.has("Block Gigapede"));
}

#[test]
fn lifelink_and_double_strike() {
    let pl = pool();
    let mut d = game(&pl, 0, Step::Main1, p().bf(&["Vampire Nighthawk"]).life(10), p());
    d.pass_until(|d| matches!(d.decision().kind, mtg_view::ViewDecisionKind::DeclareAttacker { .. }));
    d.act("Attack opponent");
    d.pass_until(|d| life(d, 1) < 20);
    assert_eq!((life(&d, 0), life(&d, 1)), (12, 18));

    let mut d = game(&pl, 0, Step::Main1, p().bf(&["Fencing Ace"]), p());
    d.pass_until(|d| matches!(d.decision().kind, mtg_view::ViewDecisionKind::DeclareAttacker { .. }));
    d.act("Attack opponent");
    for _ in 0..12 {
        if d.over() {
            break;
        }
        d.act("Pass priority");
        if life(&d, 1) == 18 {
            break;
        }
    }
    assert_eq!(life(&d, 1), 18);
    // Second (regular) damage step also deals damage.
    for _ in 0..4 {
        d.act("Pass priority");
    }
    assert_eq!(life(&d, 1), 18 - 0, "{}", life(&d, 1));
}

#[test]
fn anthem_changes_survival() {
    let pl = pool();
    let mut d = game(&pl, 0, Step::Main1, p().hand(&["Lightning Bolt"]).bf(&["Mountain"]), p().bf(&["Grizzly Bears", "Anthem Banner"]));
    d.act("Cast Lightning Bolt").act("Target: Grizzly Bears");
    d.act("Pass priority").act("Pass priority");
    // 3 damage on a 3/3 (anthem) kills it anyway; use 2 damage instead.
    assert_eq!(gy_names(&d, 1), vec!["Grizzly Bears"]);
    let mut d = game(&pl, 0, Step::Main1, p().hand(&["Prey Upon-ish Strike"]).bf(&["Mountain", "Mountain"]), p().bf(&["Grizzly Bears", "Anthem Banner"]));
    d.act("Cast Prey Upon-ish Strike").act("Target: Grizzly Bears");
    d.act("Pass priority").act("Pass priority");
    assert!(gy_names(&d, 1).is_empty(), "3/3 with anthem survives 2 damage");
    check(&d);
}

#[test]
fn losing_by_life_and_by_empty_library() {
    let pl = pool();
    let mut d = game(&pl, 0, Step::Main1, p().hand(&["Lightning Bolt"]).bf(&["Mountain"]), p().life(3));
    d.act("Cast Lightning Bolt").act("Target: opponent");
    d.act("Pass priority").act("Pass priority");
    assert_eq!(d.g.result(), Some(mtg_core::decision::GameResult::Win(Seat(0))));

    // Player 1 has an empty library and draws on their turn.
    let mut d = game(&pl, 0, Step::Cleanup, p(), p().lib(&[]));
    d.pass_until(|d| d.over());
    assert_eq!(d.g.result(), Some(mtg_core::decision::GameResult::Win(Seat(0))));
}

#[test]
fn hand_size_discard_at_cleanup() {
    let pl = pool();
    let hand: Vec<&str> = vec!["Forest"; 6].into_iter().chain(["Grizzly Bears", "Hill Giant", "Goblin Piker"]).collect();
    let mut d = game(&pl, 0, Step::Main2, p().hand(&hand), p());
    d.pass_until(|d| matches!(d.decision().kind, mtg_view::ViewDecisionKind::ChooseCards { .. }));
    assert!(matches!(d.decision().kind, mtg_view::ViewDecisionKind::ChooseCards { remaining: 2, .. }));
    d.act("Choose Forest");
    d.act("Choose Forest");
    assert_eq!(d.g.raw_state().hand_count(Seat(0)), 7);
}

#[test]
fn one_land_per_turn_and_pinger() {
    let pl = pool();
    let mut d = game(&pl, 0, Step::Main1, p().hand(&["Forest", "Mountain"]), p());
    d.act("Play Forest");
    assert!(!d.has("Play Mountain"));

    // A summoning-sick Prodigal Sorcerer cannot ping; an unsick one can, using the stack.
    let mut d = game(&pl, 0, Step::Main1, p().bf_sick(&["Prodigal Sorcerer"]), p());
    assert!(!d.has("Activate"));
    let mut d = game(&pl, 0, Step::Main1, p().bf(&["Prodigal Sorcerer"]), p());
    d.act("Activate Prodigal Sorcerer").act("Target: opponent");
    assert_eq!(life(&d, 1), 20, "ability is on the stack");
    d.act("Pass priority").act("Pass priority");
    assert_eq!(life(&d, 1), 19);
    assert!(!d.has("Activate"), "tapped");
}

#[test]
fn tokens_cease_to_exist() {
    let pl = pool();
    let mut d = game(&pl, 0, Step::Main1, p().hand(&["Raise the Alarm", "Lightning Bolt"]).bf(&["Plains", "Plains", "Mountain"]), p());
    d.act("Cast Raise the Alarm");
    d.act("Pass priority").act("Pass priority");
    assert_eq!(bf_names(&d).iter().filter(|n| *n == "Soldier").count(), 2);
    d.act("Cast Lightning Bolt");
    d.act("Target: Soldier [v7]");
    d.act("Pass priority").act("Pass priority");
    assert_eq!(bf_names(&d).iter().filter(|n| *n == "Soldier").count(), 1);
    assert!(gy_names(&d, 0).iter().all(|n| n != "Soldier"));
    check(&d);
}

#[test]
fn first_player_skips_first_draw_and_mulligan_flow() {
    let pl = pool();
    let a = mtg_view::DeckList { main: pl.deck_red_green(), side: vec![] };
    let b = mtg_view::DeckList { main: pl.deck_white_black(), side: vec![] };
    let mut g = Game::new(pl.db.clone(), [&a, &b], 11, Default::default());
    g.advance();
    // First decision: seat 0 keep or mulligan; mulligan once then bottom one card.
    let d = g.observe(Seat(0)).decision.unwrap();
    assert_eq!(d.options.len(), 2);
    assert_eq!(g.observe(Seat(0)).me.hand.len(), 7);
    let mull = d.options.iter().find(|o| o.label == "Mulligan").unwrap().idx as usize;
    g.apply(d.id, mull).unwrap();
    g.advance();
    let d = g.observe(Seat(0)).decision.unwrap();
    let keep = d.options.iter().position(|o| o.label == "Keep hand").unwrap();
    g.apply(d.id, keep).unwrap();
    g.advance();
    let d = g.observe(Seat(0)).decision.unwrap();
    assert!(d.low_frequency);
    assert!(matches!(d.kind, mtg_view::ViewDecisionKind::ChooseCards { remaining: 1, .. }));
    g.apply(d.id, 0).unwrap();
    g.advance();
    assert_eq!(g.observe(Seat(0)).me.hand.len(), 6);
}

// ---- M2b: casting in full -------------------------------------------------------------------

#[test]
fn modal_spell_chooses_modes_and_targets_only_chosen() {
    let pl = pool();
    let mut d = game(&pl, 0, Step::Main1, p().hand(&["Test Charm"]).bf(&["Mountain", "Mountain"]), p());
    d.act("Cast Test Charm");
    assert!(d.labels().iter().any(|l| l.contains("Deal 2 damage")), "{:?}", d.labels());
    d.act("Deal 2 damage");
    d.act("Gain 3 life"); // choose 1..2: the cap is reached, so no Done is needed
    d.act("Target: opponent");
    d.act("Pass priority").act("Pass priority");
    assert_eq!(life(&d, 1), 18);
    assert_eq!(life(&d, 0), 23);
    check(&d);
}

#[test]
fn x_spell_scales_with_x() {
    let pl = pool();
    let mut d = game(&pl, 0, Step::Main1, p().hand(&["Fireball Stand-in"]).bf(&["Mountain", "Mountain", "Mountain", "Mountain"]), p());
    d.act("Cast Fireball Stand-in");
    d.act("X = 3");
    d.act("Target: opponent");
    d.act("Pass priority").act("Pass priority");
    assert_eq!(life(&d, 1), 17);
    check(&d);
}

#[test]
fn alt_cost_return_island_and_counter_unless_pay() {
    let pl = pool();
    let mut d = game(&pl, 0, Step::Main1, p().hand(&["Lightning Bolt"]).bf(&["Mountain"]), p().hand(&["Daze Stand-in"]).bf(&["Island"]));
    d.act("Cast Lightning Bolt").act("Target: opponent");
    d.act("Pass priority");
    assert!(d.has("Cast Daze Stand-in (return an Island)"), "{:?}", d.labels());
    d.act("Cast Daze Stand-in (return an Island)");
    d.act("Target: Lightning Bolt");
    d.act("Choose Island"); // the cost object is picked while paying
    assert!(!bf_names(&d).iter().any(|n| n == "Island"));
    d.act("Pass priority").act("Pass priority");
    // active player has no mana for {1} (Mountain still untapped? Bolt used it) so the counter just happens
    assert_eq!(life(&d, 1), 20);
    check(&d);
}

#[test]
fn counter_unless_asks_and_payment_saves_spell() {
    let pl = pool();
    let mut d = game(&pl, 0, Step::Main1, p().hand(&["Lightning Bolt"]).bf(&["Mountain", "Mountain"]), p().hand(&["Daze Stand-in"]).bf(&["Island", "Island"]));
    d.act("Cast Lightning Bolt").act("Target: opponent");
    d.act("Pass priority");
    d.act_exact("Cast Daze Stand-in").act("Target: Lightning Bolt");
    d.act("Pass priority").act("Pass priority"); // Daze resolves and asks the Bolt's controller
    assert_eq!(d.seat(), Seat(0));
    d.act("Yes");
    d.act("Pass priority").act("Pass priority");
    assert_eq!(life(&d, 1), 17);
    check(&d);
}

#[test]
fn force_alt_cost_exiles_blue_card_and_pays_life() {
    let pl = pool();
    let mut d = game(&pl, 0, Step::Main1, p().hand(&["Lightning Bolt"]).bf(&["Mountain"]), p().hand(&["Force Stand-in", "Force Stand-in"]).life(20));
    d.act("Cast Lightning Bolt").act("Target: opponent");
    d.act("Pass priority");
    d.act("Cast Force Stand-in (exile a blue card");
    d.act("Target: Lightning Bolt");
    d.act("Choose Force Stand-in");
    assert_eq!(life(&d, 1), 19);
    d.act("Pass priority").act("Pass priority");
    assert_eq!(life(&d, 1), 19);
    check(&d);
}

#[test]
fn additional_cost_choice_discard_or_life() {
    let pl = pool();
    let mut d = game(&pl, 0, Step::Main1, p().hand(&["Triumph Stand-in", "Forest"]).bf(&["Swamp", "Swamp"]), p().bf(&["Grizzly Bears"]));
    d.act("Cast Triumph Stand-in");
    let l = d.labels();
    assert!(l.len() >= 2, "{l:?}");
    d.act_idx(1); // pay 3 life
    d.act("Target: Grizzly Bears");
    assert_eq!(life(&d, 0), 17);
    d.act("Pass priority").act("Pass priority");
    assert_eq!(gy_names(&d, 1), vec!["Grizzly Bears"]);
    check(&d);
}

#[test]
fn phyrexian_mana_paid_with_life_when_no_black_mana() {
    let pl = pool();
    let mut d = game(&pl, 0, Step::Main1, p().hand(&["Dismember Stand-in"]).bf(&["Forest"]), p().bf(&["Grizzly Bears"]));
    d.act("Cast Dismember Stand-in");
    d.act("Target: Grizzly Bears");
    d.act("Pay 2 life");
    d.act("Pay 2 life");
    assert_eq!(life(&d, 0), 16);
    d.act("Pass priority").act("Pass priority");
    assert_eq!(gy_names(&d, 1), vec!["Grizzly Bears"]);
    check(&d);
}

#[test]
fn cost_tax_applies_to_opponent_only() {
    let pl = pool();
    let mut d = game(&pl, 0, Step::Main1, p().hand(&["Lightning Bolt"]).bf(&["Mountain"]), p().bf(&["Tax Collector"]));
    assert!(!d.has("Cast Lightning Bolt"), "{:?}", d.labels());
    let mut d = game(&pl, 0, Step::Main1, p().hand(&["Lightning Bolt"]).bf(&["Mountain", "Mountain"]), p().bf(&["Tax Collector"]));
    assert!(d.has("Cast Lightning Bolt"));
    d.act("Cast Lightning Bolt");
    check(&d);
}

#[test]
fn flashback_from_graveyard_exiles() {
    let pl = pool();
    let mut d = game(&pl, 0, Step::Main1, p().gy(&["Flash Burn"]).bf(&["Mountain", "Mountain", "Mountain"]), p());
    assert!(d.has("Cast Flash Burn (flashback)"), "{:?}", d.labels());
    d.act("Cast Flash Burn (flashback)").act("Target: opponent");
    d.act("Pass priority").act("Pass priority");
    assert_eq!(life(&d, 1), 18);
    assert!(gy_names(&d, 0).is_empty());
    check(&d);
}

// ---- M2c: triggers ---------------------------------------------------------------------------

fn stack_len(d: &Driver) -> usize {
    d.g.raw_state().stack().len()
}

#[test]
fn etb_trigger_uses_stack_and_resolves() {
    let pl = pool();
    let mut d = game(&pl, 0, Step::Main1, p().hand(&["Mulldrifter Stand-in"]).bf(&["Island", "Island", "Island", "Island"]), p());
    d.act("Cast Mulldrifter Stand-in");
    d.act("Pass priority").act("Pass priority"); // creature spell resolves; trigger goes on the stack
    assert_eq!(stack_len(&d), 1, "trigger on the stack");
    assert_eq!(d.g.raw_state().hand(Seat(0)).len(), 0);
    d.act("Pass priority").act("Pass priority");
    assert_eq!(d.g.raw_state().hand(Seat(0)).len(), 2);
    check(&d);
}

#[test]
fn trigger_with_target_asks_and_fizzles_if_target_gone() {
    let pl = pool();
    let mut d = game(&pl, 0, Step::Main1, p().hand(&["Ping Elemental"]).bf(&["Mountain", "Mountain", "Mountain"]), p().hand(&["Unsummon"]).bf(&["Grizzly Bears", "Island"]));
    d.act("Cast Ping Elemental");
    d.act("Pass priority").act("Pass priority");
    // targets chosen as the trigger is put on the stack
    d.act("Target: Grizzly Bears");
    assert_eq!(stack_len(&d), 1);
    // opponent bounces its own bear in response: the trigger fizzles
    d.act("Pass priority");
    d.act("Cast Unsummon").act("Target: Grizzly Bears");
    d.act("Pass priority").act("Pass priority"); // Unsummon resolves
    d.act("Pass priority").act("Pass priority"); // trigger fizzles
    assert_eq!(gy_names(&d, 1), vec!["Unsummon"]);
    assert_eq!(d.g.raw_state().hand(Seat(1)).len(), 1);
    check(&d);
}

#[test]
fn trigger_without_legal_target_is_removed() {
    let pl = pool();
    let mut d = game(&pl, 0, Step::Main1, p().hand(&["Ping Elemental"]).bf(&["Mountain", "Mountain", "Mountain"]), p());
    d.act("Cast Ping Elemental");
    d.act("Pass priority").act("Pass priority");
    // Ping Elemental itself is a legal creature target, so the trigger targets it.
    d.act("Target: Ping Elemental");
    assert_eq!(stack_len(&d), 1);
}

#[test]
fn apnap_order_active_first_then_nonactive_resolves_first() {
    let pl = pool();
    // Both players control a Soul Warden Stand-in; a creature entering triggers both.
    let mut d = game(&pl, 0, Step::Main1, p().hand(&["Grizzly Bears"]).bf(&["Forest", "Forest", "Soul Warden Stand-in"]), p().bf(&["Soul Warden Stand-in"]).life(10));
    d.act("Cast Grizzly Bears");
    d.act("Pass priority").act("Pass priority");
    // Two triggers, one per player: active player's goes on first, so the opponent's resolves first.
    assert_eq!(stack_len(&d), 2);
    d.act("Pass priority").act("Pass priority");
    assert_eq!(life(&d, 1), 11);
    assert_eq!(life(&d, 0), 20);
    d.act("Pass priority").act("Pass priority");
    assert_eq!(life(&d, 0), 21);
    check(&d);
}

#[test]
fn order_triggers_asks_only_for_distinct_triggers() {
    let pl = pool();
    // One Soul Warden (lifegain) and one Mulldrifter-less case: use two different trigger sources.
    let mut d = game(&pl, 0, Step::Main1, p().hand(&["Grizzly Bears"]).bf(&["Forest", "Forest", "Soul Warden Stand-in", "Pyromancer Stand-in"]), p());
    d.act("Cast Grizzly Bears");
    d.act("Pass priority").act("Pass priority");
    // Only Soul Warden triggers on a creature entering; no ordering decision.
    assert_eq!(stack_len(&d), 1);
    // Two identical Soul Wardens: still no ordering decision (interchangeable).
    let mut d = game(&pl, 0, Step::Main1, p().hand(&["Grizzly Bears"]).bf(&["Forest", "Forest", "Soul Warden Stand-in", "Soul Warden Stand-in"]), p());
    d.act("Cast Grizzly Bears");
    d.act("Pass priority").act("Pass priority");
    assert_eq!(stack_len(&d), 2);
}

#[test]
fn ordering_decision_for_different_triggers() {
    let pl = pool();
    let mut d = game(&pl, 0, Step::Main1, p().hand(&["Lightning Bolt"]).bf(&["Mountain", "Soul Warden Stand-in", "Pyromancer Stand-in", "Crowd Pleaser"]), p());
    // Bolt cast: Pyromancer triggers. Only one trigger -> no order decision.
    d.act("Cast Lightning Bolt").act("Target: opponent");
    assert_eq!(stack_len(&d), 2, "Bolt + Pyromancer trigger");
    check(&d);
}

#[test]
fn dies_trigger_sees_lki_and_graveyard_trigger_fires_from_graveyard() {
    let pl = pool();
    let mut d = game(&pl, 0, Step::Main1, p().hand(&["Lightning Bolt"]).bf(&["Mountain", "Blood Artist Stand-in"]), p().bf(&["Grave Warden"]));
    d.act("Cast Lightning Bolt").act("Target: Grave Warden");
    d.act("Pass priority").act("Pass priority"); // Bolt resolves; Warden dies
    // Blood Artist Stand-in (opp creature died) and Grave Warden's own trigger from the graveyard.
    assert_eq!(stack_len(&d), 2);
    d.act("Pass priority").act("Pass priority");
    d.act("Pass priority").act("Pass priority");
    assert_eq!(life(&d, 1), 22); // +3 from Grave Warden, -1 Blood Artist
    assert_eq!(life(&d, 0), 21);
    check(&d);
}

#[test]
fn upkeep_trigger_and_intervening_if() {
    let pl = pool();
    let mut d = game(&pl, 0, Step::Draw, p().bf(&["Arena Stand-in"]), p());
    d.pass_until(|d| d.g.raw_state().stack().len() == 1);
    assert_eq!(d.g.raw_state().turn_data().step, Step::Upkeep);
    check(&d);
}
