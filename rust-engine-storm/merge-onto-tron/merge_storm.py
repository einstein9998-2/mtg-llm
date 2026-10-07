import sys,os
root=sys.argv[1]
def P(p): return os.path.join(root,p)
def sub(p,a,b,cnt=1):
    s=open(P(p)).read(); assert a in s,(p,a[:70]); open(P(p),'w').write(s.replace(a,b,cnt))
def has(p,a): return a in open(P(p)).read()
sub('crates/mtg-cards/src/legacy.rs','pub const TRON_CARDS: &str = include_str!("../cards/tron.cards.ron");\n','pub const TRON_CARDS: &str = include_str!("../cards/tron.cards.ron");\n\npub const STORM_CARDS: &str = include_str!("../cards/storm.cards.ron");\n')
sub('crates/mtg-cards/src/legacy.rs','("tron.cards.ron", TRON_CARDS)]','("tron.cards.ron", TRON_CARDS), ("storm.cards.ron", STORM_CARDS)]')
sub('crates/mtg-core/src/types.rs','"Cave", "Gate", "Lair", "Locus", "Sphere",','"Cave", "Gate", "Lair", "Locus", "Sphere",\n    // RFC 0007 (Storm): after Tron\'s names ("Saga" is already above).\n    "Lesson", "Sorcerer",')
sub('crates/mtg-core/src/card.rs','AbilityDef::NoHandLimit | AbilityDef::CostFloor(_) => Vec::new(),','AbilityDef::NoHandLimit | AbilityDef::CostFloor(_) | AbilityDef::Saga { .. } | AbilityDef::ExtraLandDrop | AbilityDef::SpellsUncounterable => Vec::new(),')
sub('crates/mtg-core/src/card.rs','AbilityDef::NoHandLimit | AbilityDef::CostFloor(_) => {}','AbilityDef::NoHandLimit | AbilityDef::CostFloor(_) | AbilityDef::Saga { .. } | AbilityDef::ExtraLandDrop | AbilityDef::SpellsUncounterable => {}')
sub('crates/mtg-core/src/card.rs','    pub has_cost_floor: bool,\n','    pub has_cost_floor: bool,\n    /// Some card has an extra land drop (RFC 0007).\n    pub has_extra_land: bool,\n    /// Some card has the static "spells you control can\'t be countered" (RFC 0007).\n    pub has_uncounterable_static: bool,\n    /// Some card is a Saga (RFC 0007).\n    pub has_saga: bool,\n')
sub('crates/mtg-core/src/card.rs','                AbilityDef::CostFloor(_) => self.has_cost_floor = true,\n','                AbilityDef::CostFloor(_) => self.has_cost_floor = true,\n                AbilityDef::ExtraLandDrop => self.has_extra_land = true,\n                AbilityDef::SpellsUncounterable => self.has_uncounterable_static = true,\n                AbilityDef::Saga { .. } => self.has_saga = true,\n')
effects=open(os.path.join(os.path.dirname(os.path.abspath(__file__)),'merge_effects.txt')).read()
sub('crates/mtg-core/src/ir.rs','    AddManaExpr { color: ManaColor, n: Expr },\n','    AddManaExpr { color: ManaColor, n: Expr },\n'+effects)
sub('crates/mtg-core/src/ir.rs','    ProtectionFromEverything,\n}','''    ProtectionFromEverything,
    /// The player may play lands and cast spells from their graveyard (Gaea's Will, RFC 0007).
    PlayFromGraveyard,
    /// A card that would be put into the player's graveyard from anywhere is exiled instead (RFC 0007).
    ExileInsteadOfGraveyard,
}''')
sub('crates/mtg-core/src/ir.rs','    CostFloor(CostFloorDef),\n}','''    CostFloor(CostFloorDef),
    // ---- RFC 0007 (Storm) ----
    /// A Saga (CR 714): enters with a lore counter, gets another after its controller's draw step,
    /// and is sacrificed once the chapter `last` has resolved. Chapters are `Triggered` abilities
    /// on `EventPat::Chapter`.
    Saga { last: u8 },
    /// "You may play an additional land on each of your turns." (Song of Creation)
    ExtraLandDrop,
    /// "Spells you control can't be countered." (Hexing Squelcher, while it is on the battlefield)
    SpellsUncounterable,
}''')
sub('crates/mtg-core/src/lint.rs','        Effect::SetPTX { objs: o, x, .. } => tmax(objs(o), expr(x)),\n','''        Effect::SetPTX { objs: o, x, .. } => tmax(objs(o), expr(x)),
        Effect::Wish { .. } | Effect::CastMovedFree { .. } | Effect::Imprint { .. } | Effect::AddManaImprinted | Effect::CounterEventUnlessLife { .. } => -1,
        Effect::DiscardRandom { who, n } => tmax(p(who), expr(n)),
        Effect::CastFree(o) => o_(o),
''')
sub('crates/mtg-core/src/decision.rs','    PlayChosen,\n}','    PlayChosen,\n    // RFC 0007 (Storm)\n    /// Imprint (Chrome Mox): the card exiled from hand; `Done` declines.\n    Imprint,\n}')
sub('crates/mtg-spec/src/runner.rs','        "burden" => CounterKind::Other(0),\n','        "burden" => CounterKind::Other(0),\n        "lore" => CounterKind::Lore,\n')
# legal.rs (Tron's filter-mana mechanism already covers Giant's Boulder's "{1},{T}: any color")
sub('crates/mtg-core/src/legal.rs','                    if ad.is_mana {\n                        if let Some(m) = simple_mana_ability(ad) {\n                            mask |= m.colors;\n                            let mn = match','''                    if ad.is_mana {
                        if let Some(c) = &ad.cond {
                            if !self.activation_cond_holds(seat, r, c) {
                                continue;
                            }
                        }
                        if is_imprint_mana(ad) {
                            let m = self.imprint_colors(r);
                            if m != 0 {
                                mask |= m;
                                if ability == 255 {
                                    ability = i as u8;
                                }
                            }
                        } else if let Some(m) = simple_mana_ability(ad) {
                            mask |= m.colors;
                            let mn = match''')
sub('crates/mtg-core/src/legal.rs','                let simple = simple_mana_ability(ad).is_some();\n                if simple && !explicit {\n                    continue;\n                }\n','''                let imprint = is_imprint_mana(ad);
                let simple = simple_mana_ability(ad).is_some() || imprint;
                if simple && !explicit {
                    continue;
                }
                if let Some(c) = &ad.cond {
                    if !self.activation_cond_holds(seat, r, c) {
                        continue;
                    }
                }
''')
# drop the pool-paid Boulder arm (a duplicate of Tron's CostItem::Mana arm)
p='crates/mtg-core/src/legal.rs'
s2=open(P(p)).read()
a=s2.index("                // Giant's Boulder's {1}: spent from the mana pool")
b=s2.index('                _ => debug_assert!(false, "unsupported mana ability cost")')
open(P(p),'w').write(s2[:a]+s2[b:])
# expect.rs
p='crates/mtg-spec/src/expect.rs'
sub(p,'''            "activate" | "activate_from_hand" => {
                let want = self.obj_alias(pat["source"].as_str().unwrap_or(""))?;
                // `ability` (when given)''','''            "suspend" => {
                let want = self.obj_alias(pat["card"].as_str().unwrap_or(""))?;
                let ab = self.suspend_ability(want);
                p.options.iter().any(|o| matches!(o, Opt::Activate { src, ability } if self.same_obj(*src, want) && Some(*ability) == ab))
            }
            "activate" | "activate_from_hand" => {
                let want = self.obj_alias(pat["source"].as_str().unwrap_or(""))?;
                // `choose_color` names the mana color of a mana ability.
                let color: Option<u8> = pat.get("choose_color").and_then(|x| x.as_str()).and_then(|c| ["W", "U", "B", "R", "G", "C"].iter().position(|x| *x == c)).map(|i| i as u8);
                // `ability` (when given)''')
sub(p,'''                    Opt::Activate { src, ability } | Opt::Mana { src, ability, .. } => self.same_obj(*src, want) && want_ab.map_or(true, |w| w == *ability),
                    _ => false,''','''                    Opt::Activate { src, ability } => self.same_obj(*src, want) && want_ab.map_or(true, |w| w == *ability) && color.is_none(),
                    Opt::Mana { src, ability, color: c, .. } => self.same_obj(*src, want) && want_ab.map_or(true, |w| w == *ability) && color.map_or(true, |x| x == *c),
                    _ => false,''')
# Construct Token lives in tron.cards.ron
p='crates/mtg-cards/cards/storm.cards.ron'
s=open(P(p)).read()
a=s.index('  (name: "Construct Token"'); b=s.index('  (name: ',a+10)
s=s[:a]+s[b:]
s=s.replace('// Kept apart from legacy.cards.ron','// "Construct Token" is defined in tron.cards.ron (identical definition).\n// Kept apart from legacy.cards.ron',1)
open(P(p),'w').write(s)
print("merged")
