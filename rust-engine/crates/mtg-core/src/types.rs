//! Basic rules vocabulary: card types, colors, keywords, zones, steps.

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct Types: u16 {
        const LAND         = 1 << 0;
        const CREATURE     = 1 << 1;
        const ARTIFACT     = 1 << 2;
        const ENCHANTMENT  = 1 << 3;
        const PLANESWALKER = 1 << 4;
        const INSTANT      = 1 << 5;
        const SORCERY      = 1 << 6;
        const KINDRED      = 1 << 7;
        const DUNGEON      = 1 << 8;
        const EMBLEM       = 1 << 9;
    }
}

impl Types {
    pub const PERMANENT: Types = Types::from_bits_truncate(
        Types::LAND.bits() | Types::CREATURE.bits() | Types::ARTIFACT.bits() | Types::ENCHANTMENT.bits() | Types::PLANESWALKER.bits(),
    );
    pub const SPELL_KIND: Types = Types::from_bits_truncate(Types::INSTANT.bits() | Types::SORCERY.bits());
}

bitflags! {
    #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct Supertypes: u8 {
        const BASIC     = 1 << 0;
        const LEGENDARY = 1 << 1;
        const SNOW      = 1 << 2;
        const WORLD     = 1 << 3;
    }
}

bitflags! {
    #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct Colors: u8 {
        const W = 1 << 0;
        const U = 1 << 1;
        const B = 1 << 2;
        const R = 1 << 3;
        const G = 1 << 4;
    }
}

impl Colors {
    pub const ALL: [Colors; 5] = [Colors::W, Colors::U, Colors::B, Colors::R, Colors::G];
    pub fn count(self) -> u32 {
        self.bits().count_ones()
    }
}

/// Mana "colors" including colorless. Index order: W U B R G C.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ManaColor {
    W = 0,
    U = 1,
    B = 2,
    R = 3,
    G = 4,
    C = 5,
}

impl ManaColor {
    pub const ALL: [ManaColor; 6] = [ManaColor::W, ManaColor::U, ManaColor::B, ManaColor::R, ManaColor::G, ManaColor::C];
    pub fn to_color(self) -> Colors {
        match self {
            ManaColor::W => Colors::W,
            ManaColor::U => Colors::U,
            ManaColor::B => Colors::B,
            ManaColor::R => Colors::R,
            ManaColor::G => Colors::G,
            ManaColor::C => Colors::empty(),
        }
    }
    pub fn idx(self) -> usize {
        self as usize
    }
    pub fn symbol(self) -> char {
        ['W', 'U', 'B', 'R', 'G', 'C'][self as usize]
    }
}

bitflags! {
    /// Engine-intrinsic keywords (doc 03 section 3.8).
    #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct Keywords: u32 {
        const FLYING        = 1 << 0;
        const FIRST_STRIKE  = 1 << 1;
        const DOUBLE_STRIKE = 1 << 2;
        const DEATHTOUCH    = 1 << 3;
        const LIFELINK      = 1 << 4;
        const HASTE         = 1 << 5;
        const HEXPROOF      = 1 << 6;
        const SHROUD        = 1 << 7;
        const INDESTRUCTIBLE= 1 << 8;
        const MENACE        = 1 << 9;
        const REACH         = 1 << 10;
        const TRAMPLE       = 1 << 11;
        const VIGILANCE     = 1 << 12;
        const FLASH         = 1 << 13;
        const DEFENDER      = 1 << 14;
        /// "This spell can't be countered."
        const UNCOUNTERABLE = 1 << 15;
        /// "Attacks each combat if able."
        const MUST_ATTACK   = 1 << 16;
        /// Prowess: `CardDef::finish` adds the triggered ability.
        const PROWESS       = 1 << 17;
        const SWAMPWALK     = 1 << 18;
        /// "Can block only creatures with flying" (Brazen Borrower).
        const BLOCKS_ONLY_FLYERS = 1 << 19;
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ZoneKind {
    Library,
    Hand,
    Graveyard,
    Battlefield,
    Stack,
    Exile,
    Command,
    /// Tokens and copies that have ceased to exist; ability objects that left the stack.
    Gone,
}

impl ZoneKind {
    /// Zones that belong to a specific player (cards go to their owner's).
    pub fn is_owned_zone(self) -> bool {
        matches!(self, ZoneKind::Library | ZoneKind::Hand | ZoneKind::Graveyard)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Step {
    Untap,
    Upkeep,
    Draw,
    Main1,
    BeginCombat,
    DeclareAttackers,
    DeclareBlockers,
    FirstStrikeDamage,
    CombatDamage,
    EndCombat,
    Main2,
    End,
    Cleanup,
}

impl Step {
    pub fn is_main(self) -> bool {
        matches!(self, Step::Main1 | Step::Main2)
    }
    pub fn is_combat(self) -> bool {
        matches!(
            self,
            Step::BeginCombat | Step::DeclareAttackers | Step::DeclareBlockers | Step::FirstStrikeDamage | Step::CombatDamage | Step::EndCombat
        )
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CounterKind {
    PlusOne,
    MinusOne,
    Loyalty,
    Energy,
    Time,
    Stun,
    Charge,
    /// A flying counter (grants flying).
    Flying,
    Other(u8),
}

/// The closed table of subtypes in the pool. Index = bit position in `SubtypeSet`. The five basic
/// land types and Desert have fixed positions because the engine reads them directly.
pub const SUBTYPE_NAMES: &[&str] = &[
    "Plains", "Island", "Swamp", "Mountain", "Forest", "Desert", "Urza's", "Equipment", "Aura", "Clue", "Treasure", "Human", "Soldier", "Cleric", "Wizard",
    "Monk", "Elemental", "Goblin", "Cat", "Warrior", "Zombie", "Orc", "Army", "Illusion", "Bird", "Spirit", "Dragon", "Shaman", "Rogue", "Knight", "Scout",
    "Phyrexian", "Elf", "Beast", "Angel", "Demon", "Archon", "Sphinx", "Noble", "Peasant", "Dwarf", "Artificer", "Advisor", "Samurai", "Ninja", "Faerie", "Merfolk",
    "Ally", "Hound", "Saproling", "Insect", "Lhurgoyf", "Horror", "Bear", "Skeleton", "Atropal", "Hobbit", "Rat", "Fish", "Avatar", "Giant", "Construct", "Ooze",
    "Vampire", "Spider", "Hero", "Mutant", "Turtle", "Eldrazi", "Wall", "Sliver", "Pirate", "Citizen", "Cyberman", "Coil", "Kor", "Gnome", "Detective", "Assassin",
    "God", "Archer", "Bard", "Halfling", "Kithkin", "Wraith", "Serpent", "Dog", "Elder", "Incarnation", "Dinosaur", "Jace", "Kaito", "Sand",
    "Ajani", "Tamiyo", "Moonfolk", "Warlock",
];

/// Subtype set; bit `i` = `SUBTYPE_NAMES[i]`.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct SubtypeSet(pub u128);

/// First creature type in `SUBTYPE_NAMES` (everything before is a land, artifact, enchantment or spell type).
pub const SUB_FIRST_CREATURE_TYPE: u8 = 11;

pub const SUB_PLAINS: u8 = 0;
pub const SUB_ISLAND: u8 = 1;
pub const SUB_SWAMP: u8 = 2;
pub const SUB_ARMY: u8 = 22;
pub const SUB_MOUNTAIN: u8 = 3;
pub const SUB_FOREST: u8 = 4;
pub const SUB_DESERT: u8 = 5;
pub const SUB_AURA: u8 = 8;

pub fn subtype_index(name: &str) -> Option<u8> {
    SUBTYPE_NAMES.iter().position(|n| *n == name).map(|i| i as u8)
}

impl Serialize for SubtypeSet {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        let names: Vec<&str> = (0..SUBTYPE_NAMES.len()).filter(|&i| self.has(i as u8)).map(|i| SUBTYPE_NAMES[i]).collect();
        names.serialize(ser)
    }
}

impl<'de> Deserialize<'de> for SubtypeSet {
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let names: Vec<String> = Vec::deserialize(de)?;
        let mut set = SubtypeSet::EMPTY;
        for n in names {
            match subtype_index(&n) {
                Some(i) => set.insert(i),
                None => return Err(serde::de::Error::custom(format!("unknown subtype {n:?}; add it to SUBTYPE_NAMES"))),
            }
        }
        Ok(set)
    }
}

impl SubtypeSet {
    pub const EMPTY: SubtypeSet = SubtypeSet(0);
    pub fn single(i: u8) -> SubtypeSet {
        SubtypeSet(1u128 << i)
    }
    pub fn has(self, i: u8) -> bool {
        self.0 & (1u128 << i) != 0
    }
    pub fn insert(&mut self, i: u8) {
        self.0 |= 1u128 << i;
    }
    pub fn intersects(self, o: SubtypeSet) -> bool {
        self.0 & o.0 != 0
    }
    pub fn union(self, o: SubtypeSet) -> SubtypeSet {
        SubtypeSet(self.0 | o.0)
    }
}
