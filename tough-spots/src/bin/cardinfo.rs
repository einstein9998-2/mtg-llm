//! Prints name -> {t: type letters, c: mana cost, pt} for every real card in the pool, as JSON.
//! The answer page embeds this so it can draw lands, creatures and spells differently.
//! Usage: cardinfo > cardinfo.json
use mtg_core::types::Types;

fn main() {
    let db = mtg_cards::legacy::build();
    let mut out = serde_json::Map::new();
    for d in db.defs.iter().filter(|d| !d.is_token && !d.is_back) {
        let t = if d.types.contains(Types::LAND) { "land" }
            else if d.types.contains(Types::CREATURE) { "creature" }
            else if d.types.contains(Types::PLANESWALKER) { "planeswalker" }
            else if d.types.contains(Types::ARTIFACT) { "artifact" }
            else if d.types.contains(Types::ENCHANTMENT) { "enchantment" }
            else if d.types.contains(Types::INSTANT) { "instant" }
            else if d.types.contains(Types::SORCERY) { "sorcery" }
            else { "other" };
        let cost = d.cost.map(|c| c.to_string()).unwrap_or_default();
        out.insert(d.name.clone(), serde_json::json!({ "t": t, "c": cost }));
    }
    println!("{}", serde_json::Value::Object(out));
}
