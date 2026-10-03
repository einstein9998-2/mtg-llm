//! The card DSL loader (doc 03 section 2): RON text -> `CardDef` -> `CardDb`. The surface syntax
//! is the serde form of the core IR, so unknown variants and fields fail loudly at load time.

use mtg_core::card::{CardDb, CardDef};

/// Parses one RON document holding a list of cards.
pub fn parse_cards(src: &str) -> Result<Vec<CardDef>, String> {
    let opts = ron::Options::default().with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME | ron::extensions::Extensions::UNWRAP_VARIANT_NEWTYPES);
    opts.from_str::<Vec<CardDef>>(src).map_err(|e| format!("{e}"))
}

/// Builds a database from named sources (the name appears in error messages).
pub fn build_db(sources: &[(&str, &str)]) -> Result<CardDb, String> {
    let mut db = CardDb::new();
    for (name, text) in sources {
        let cards = parse_cards(text).map_err(|e| format!("{name}: {e}"))?;
        for c in cards {
            let cname = c.name.clone();
            db.try_add(c).map_err(|e| format!("{name}: {cname}: {e}"))?;
        }
    }
    db.link_faces().map_err(|e| format!("faces: {e}"))?;
    db.validate().map_err(|e| format!("validation: {e}"))?;
    Ok(db)
}
