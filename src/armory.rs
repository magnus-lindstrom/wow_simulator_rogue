extern crate serde;
extern crate serde_yaml;

use crate::utils::{Args};
use std::fs;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

const ITEM_COLLECTION_PATH: &str = "src/items";


#[derive(Clone,Copy,Debug,PartialEq,Serialize,Deserialize)]
pub enum Race {
    Human,
    NightElf,
    Gnome,
    Dwarf,
    None
}

#[derive(Clone,Copy,Debug,Serialize,Deserialize)]
struct PrimStats {
    agility: i32,
    strength: i32,
    stamina: i32,
    sword_skill: i32,
    dagger_skill: i32
}

impl PrimStats {
    fn new(race: Race) -> PrimStats {
        if race == Race::Human {
            PrimStats {
                agility: 130,
                strength: 80,
                stamina: 75,
                sword_skill: 305,
                dagger_skill: 300
            }
        } else if race == Race::None {
            PrimStats {
                agility: 0,
                strength: 0,
                stamina: 0,
                sword_skill: 0,
                dagger_skill: 0
            }
        } else { panic!("Race not implemented"); }
    }
}

#[derive(Clone,Copy,Debug,Serialize,Deserialize)]
struct SecStats {
    crit: f32,
    hit: f32,
    haste: f32,
    attack_power: i32
}

impl SecStats {
    fn new(race: Race) -> SecStats {
        if race == Race::Human {
            SecStats {
                crit: 0.0,
                hit: 0.0,
                haste: 0.0,
                attack_power: 100
            }
        } else if race == Race::None {
            SecStats {
                crit: 0.0,
                hit: 0.0,
                haste: 0.0,
                attack_power: 0
            }
        } else { panic!("Race not implemented"); }
    }
}

#[derive(Debug,Serialize,Deserialize)]
pub struct ItemCollection {
    pub armor: HashMap<String,Armor>,
    pub weapons: HashMap<String,Weapon>
}

impl ItemCollection {
    pub fn initialize_item_collection() -> ItemCollection {

        let item_col_string = fs::read_to_string(ITEM_COLLECTION_PATH)
                .expect("Something went wrong reading items from file.");
        let item_col: ItemCollection = serde_yaml::from_str(
            &item_col_string).unwrap();
        /*
        let mut armor_map: HashMap<String,Armor> = HashMap::new();
        let mut weapon_map: HashMap<String,Weapon> = HashMap::new();
        
        let bf_hood = Armor::new("bloodfang_hood".to_string());
        let gutgore = Weapon::new("gutgore_ripper".to_string());

        armor_map.insert("bloodfang_hood".to_string(), bf_hood);
        weapon_map.insert("gutgore_ripper".to_string(), gutgore);

        let item_collection: ItemCollection = ItemCollection {
            armor: armor_map,
            weapons: weapon_map
        };
        */

        // let item_collection_string = fs::read_to_string(ITEM_COLLECTION_PATH)
            // .expect("Something went wrong reading item file.");
        // let item_collection: ItemCollection = serde_yaml::from_str(
            // &item_collection_string).unwrap();
        return item_col;
    }
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Weapon {
    name: String,
    prim_stats: PrimStats,
    sec_stats: SecStats,
    swing_speed: f32,
    min_dmg: f32,
    max_dmg: f32,
    mean_dmg: f32
}

impl Weapon {
    fn new() -> Weapon {
        Weapon {
            name: "".to_string(),
            prim_stats: PrimStats::new(Race::None),
            sec_stats: SecStats::new(Race::None),
            swing_speed: 0.0,
            min_dmg: 0.0,
            max_dmg: 0.0,
            mean_dmg: 0.0
        }
    }

    fn copy(&self) -> Weapon {
        Weapon {
            name: self.name.to_string(),
            prim_stats: self.prim_stats.clone(),
            sec_stats: self.sec_stats.clone(),
            swing_speed: self.swing_speed,
            min_dmg: self.min_dmg,
            max_dmg: self.max_dmg,
            mean_dmg: self.mean_dmg
        }
    }

    fn set_mean_dmg(&mut self) {
        self.mean_dmg = (self.min_dmg + self.max_dmg) / 2.0;
    }
}
        
#[derive(Debug,Serialize,Deserialize)]
pub struct Armor {
    // todo: add item slots and sets
    name: String,
    prim_stats: PrimStats,
    sec_stats: SecStats
}

impl Armor {
    fn copy(&self) -> Armor {
        Armor {
            name: self.name.to_string(),
            prim_stats: self.prim_stats.clone(),
            sec_stats: self.sec_stats.clone()
        }
    }
}

#[derive(Debug,Serialize,Deserialize)]
struct CharacterSpecification {
    mh_name: String,
    oh_name: String,
    armor_names: Vec<String>
}

impl CharacterSpecification {
    pub fn read_from_file(args: &Args) -> CharacterSpecification {

        let character_spec_string = fs::read_to_string(&args.param_file)
            .expect("Something went wrong reading file.");
        let character_spec: CharacterSpecification = serde_yaml::from_str(
            &character_spec_string).unwrap();
        return character_spec;
    }
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Character {
    race: Race,
    prim_stats: PrimStats,
    sec_stats: SecStats,
    mh: Weapon,
    oh: Weapon,
    armor: Vec<Armor>
}

impl Character {
    pub fn get_character(args: &Args) -> Character {
        let char_spec = CharacterSpecification::read_from_file(args);
        let character: Character = Character::assemble_character(char_spec);
        return character;
    }

    fn new(race: Race) -> Character {
        Character {
            race: race,
            prim_stats: PrimStats::new(race),
            sec_stats: SecStats::new(race),
            mh: Weapon::new(),
            oh: Weapon::new(),
            armor: Vec::new()
        }
    }

    fn assemble_character(char_spec: CharacterSpecification) -> Character {
        let mut character = Character::new(Race::Human);

        let item_collection: ItemCollection = 
            ItemCollection::initialize_item_collection();

        for armor_name in &char_spec.armor_names {
            let armor = item_collection.armor.get(&armor_name.to_string()).
                unwrap();
            character.armor.push(armor.copy());
        }
        let mh = item_collection.weapons.get(&char_spec.mh_name.to_string())
            .unwrap();
        character.mh = mh.copy();
        character.mh.set_mean_dmg();
        let oh = item_collection.weapons.get(&char_spec.oh_name.to_string())
            .unwrap();
        character.oh = oh.copy();
        character.oh.set_mean_dmg();

        return character;
    }

}




            





