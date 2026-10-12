use mtg_core::types::Keywords;

#[test]
fn emrakul_loads_last_with_keywords() {
    let db = mtg_cards::legacy::build();
    let id = db.id("Emrakul, the Aeons Torn").expect("Emrakul in the pool");
    // Registered last, so no existing CardDefId shifts.
    assert_eq!(id.0 as usize + 1, db.defs.len());
    let d = db.def(id);
    let kw = d.keywords;
    assert!(kw.contains(Keywords::FLYING | Keywords::UNCOUNTERABLE | Keywords::PROT_COLORED_SPELLS));
}
