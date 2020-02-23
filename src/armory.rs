extern crate serde;
extern crate serde_yaml;

use crate::utils::{Args};
use std::fs;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

const ARMOR_COLLECTION_PATH: &str = "src/databases/armor.yaml";
const WEAPON_COLLECTION_PATH: &str = "src/databases/weapons.yaml";
const ENCHANT_COLLECTION_PATH: &str = "src/databases/enchants.yaml";


#[derive(Debug,Serialize,Deserialize)]
struct Buffs {
    motw: bool,
    bom: bool,
    battle_shout: bool,
    juju_power: bool,
    juju_might: bool,
    mongoose: bool,
    grilled_squid: bool,
    songflower_serenade: bool,
    bok: bool
}

impl Buffs {
    fn new() -> Buffs {
        Buffs {
            motw: false,
            bom: false,
            battle_shout: false,
            juju_power: false,
            juju_might: false,
            mongoose: false,
            grilled_squid: false,
            songflower_serenade: false,
            bok: false
        }
    }
}

#[derive(Clone,Copy,Debug,PartialEq,Serialize,Deserialize)]
pub enum Race {
    Human,
    NightElf,
    Gnome,
    Dwarf,
    None
}

#[derive(Clone,Copy,Debug,Serialize,Deserialize)]
pub struct PrimStats {
    agility: i32,
    strength: i32,
    stamina: i32,
    arcane_resistance: i32,
    fire_resistance: i32,
    frost_resistance: i32,
    shadow_resistance: i32,
    pub sword_skill: i32,
    pub dagger_skill: i32
}

impl PrimStats {
    fn new_from_race(race: Race) -> PrimStats {
        if race == Race::Human {
            PrimStats {
                agility: 130,
                strength: 80,
                stamina: 75,
                arcane_resistance: 0,
                fire_resistance: 0,
                frost_resistance: 0,
                shadow_resistance: 0,
                sword_skill: 305,
                dagger_skill: 300
            }
        } else if race == Race::None {
            PrimStats {
                agility: 0,
                strength: 0,
                stamina: 0,
                arcane_resistance: 0,
                fire_resistance: 0,
                frost_resistance: 0,
                shadow_resistance: 0,
                sword_skill: 0,
                dagger_skill: 0
            }
        } else { panic!("Race not implemented"); }
    }
}

#[derive(Clone,Copy,Debug,Serialize,Deserialize)]
pub struct SecStats {
    pub crit: f32,
    pub hit: f32,
    pub haste: f32,
    pub attack_power: i32
}

impl SecStats {
    fn new_from_race(race: Race) -> SecStats {
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

        let equipment_string = fs::read_to_string(ARMOR_COLLECTION_PATH)
                .expect("Something went wrong reading items from file.");
        let equipment: HashMap<String,Armor> = serde_yaml::from_str(
            &equipment_string).unwrap();

        let weapons_string = fs::read_to_string(WEAPON_COLLECTION_PATH)
                .expect("Something went wrong reading items from file.");
        let weapons: HashMap<String,Weapon> = serde_yaml::from_str(
            &weapons_string).expect("Error trying to deserialize weapons");

        return ItemCollection { armor: equipment, weapons: weapons };
    }
}

#[derive(Clone,Copy,Debug,PartialEq,Serialize,Deserialize)]
pub enum WeaponType {
    Dagger,
    Sword,
    None,
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Weapon {
    name: String,
    weapon_type: WeaponType,
    prim_stats: PrimStats,
    sec_stats: SecStats,
    enchant: Enchant,
    swing_interval: f32,
    min_dmg: f32,
    max_dmg: f32,
    mean_dmg: f32
}

impl Weapon {
    fn new() -> Weapon {
        Weapon {
            name: "".to_string(),
            weapon_type: WeaponType::None,
            prim_stats: PrimStats::new_from_race(Race::None),
            sec_stats: SecStats::new_from_race(Race::None),
            enchant: Enchant::new(),
            swing_interval: 0.0,
            min_dmg: 0.0,
            max_dmg: 0.0,
            mean_dmg: 0.0
        }
    }

    fn copy(&self) -> Weapon {
        Weapon {
            name: self.name.to_string(),
            weapon_type: self.weapon_type.clone(),
            prim_stats: self.prim_stats.clone(),
            sec_stats: self.sec_stats.clone(),
            enchant: self.enchant.copy(),
            swing_interval: self.swing_interval,
            min_dmg: self.min_dmg,
            max_dmg: self.max_dmg,
            mean_dmg: self.mean_dmg
        }
    }

    fn set_mean_dmg(&mut self) {
        self.mean_dmg = (self.min_dmg + self.max_dmg) / 2.0;
    }

    pub fn get_mean_dmg(&self) -> f32 {
        return self.mean_dmg;
    }

    pub fn get_swing_interval(&self) -> f32 {
        return self.swing_interval;
    }

    pub fn get_weapon_type(&self) -> WeaponType {
        return self.weapon_type;
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
struct ItemSpecification {
    mh_name: String,
    oh_name: String,
    armor_names: Vec<String>
}

impl ItemSpecification {
    pub fn get_item_specification(args: &Args) -> ItemSpecification {

        let character_spec_string = fs::read_to_string(&args.items_file)
            .expect(&format!("Something went wrong items from {}.",
                    args.items_file));
        let character_spec: ItemSpecification = serde_yaml::from_str(
            &character_spec_string).unwrap();
        return character_spec;
    }
}

#[derive(Debug,Serialize,Deserialize)]
struct EnchantSpecification {
    armor_enchant_names: Vec<String>,
    mh_enchant_name: String,
    oh_enchant_name: String
}

impl EnchantSpecification {
    fn get_enchant_spec(args: &Args) -> EnchantSpecification {
        let enchant_spec_string = fs::read_to_string(&args.enchants_file)
            .expect(&format!("Something went wrong reading enchants from {}.",
                    args.enchants_file));
        let enchant_spec: EnchantSpecification = serde_yaml::from_str(
            &enchant_spec_string).unwrap();
        return enchant_spec;
    }
}

#[derive(Debug,Serialize,Deserialize)]
struct Enchant {
    name: String,
    prim_stats: PrimStats,
    sec_stats: SecStats,
    extra_damage: f32
}

impl Enchant {
    pub fn new() -> Enchant {
        Enchant {
            name: "".to_string(),
            prim_stats: PrimStats::new_from_race(Race::None),
            sec_stats: SecStats::new_from_race(Race::None),
            extra_damage: 0.0
        }
    }

    fn copy(&self) -> Enchant {
        Enchant {
            name: self.name.to_string(),
            prim_stats: self.prim_stats.clone(),
            sec_stats: self.sec_stats.clone(),
            extra_damage: self.extra_damage
        }
    }
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Talents {
    // assassination
    pub improved_eviscerate: i32,
    pub malice: i32,
    pub ruthlessness: i32,
    pub improved_slice_and_dice: i32,
    pub relentless_strikes: i32,
    pub lethality: i32,
    // combat
    pub improved_sinister_strike: i32,
    pub improved_backstab: i32,
    pub precision: i32,
    pub dagger_specialization: i32,
    pub dual_wield_specialization: i32,
    pub sword_specialization: i32,
    pub weapon_expertise: i32,
    pub aggression: i32,
    // subtlety
    pub opportunity: i32
}

impl Talents {
    fn new() -> Talents {
        Talents {
            // assassination
            improved_eviscerate: 0,
            malice: 0,
            ruthlessness: 0,
            improved_slice_and_dice: 0,
            relentless_strikes: 0,
            lethality: 0,
            // combat
            improved_sinister_strike: 0,
            improved_backstab: 0,
            precision: 0,
            dagger_specialization: 0,
            dual_wield_specialization: 0,
            sword_specialization: 0,
            weapon_expertise: 0,
            aggression: 0,
            // subtlety
            opportunity: 0
        }
    }
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Character {
    race: Race,
    pub prim_stats: PrimStats,
    pub sec_stats: SecStats,
    buffs: Buffs,
    armor_enchants: Vec<Enchant>,
    pub mh: Weapon,
    pub oh: Weapon,
    armor: Vec<Armor>,
    pub talents: Talents
}

impl Character {
    pub fn create_character(args: &Args) -> Character {
        let mut character = Character::new(Race::Human);

        let item_spec = ItemSpecification::get_item_specification(args);
        println!("hej");
        character.set_armor_and_weapons(item_spec);
        let enchant_spec = EnchantSpecification::get_enchant_spec(args);
        character.set_enchants(enchant_spec);

        character.set_buffs(args);
        character.set_talents(args);
        character.apply_stats_from_armor_and_weapons();
        character.apply_stats_from_buffs();
        // character.apply_stats_from_enchants();
        character.convert_primary_stats_to_secondary();
        return character;
    }


    fn new(race: Race) -> Character {
        Character {
            race: race,
            prim_stats: PrimStats::new_from_race(race),
            sec_stats: SecStats::new_from_race(race),
            buffs: Buffs::new(),
            armor_enchants: Vec::new(),
            mh: Weapon::new(),
            oh: Weapon::new(),
            armor: Vec::new(),
            talents: Talents::new()
        }
    }

    fn convert_primary_stats_to_secondary(&mut self) {
        // attack power
        self.sec_stats.attack_power += self.prim_stats.agility;
        self.sec_stats.attack_power += self.prim_stats.strength;
        // crit
        self.sec_stats.crit += 0.01 * self.prim_stats.agility as f32 / 29.0;
    }

    fn initialize_enchant_collection(&self) -> HashMap<String,Enchant> {

        let enchant_string = fs::read_to_string(ENCHANT_COLLECTION_PATH)
                .expect("Something went wrong reading enchants from file.");
        let enchants: HashMap<String,Enchant> = serde_yaml::from_str(
            &enchant_string).unwrap();

        return enchants;
    }

    fn set_enchants(&mut self, enchant_spec: EnchantSpecification) {
        let enchant_collection = self.initialize_enchant_collection();

        // armor enchants
        for enchant_name in &enchant_spec.armor_enchant_names {
            let enchant = enchant_collection.get(&enchant_name.to_string())
                .expect(&format!("Could not find {} in enchants file.", 
                                 enchant_name));
            self.armor_enchants.push(enchant.copy());
        }

        // mh enchant
        let mh_enchant = enchant_collection
            .get(&enchant_spec.mh_enchant_name.to_string())
            .expect(&format!("Could not find {} in enchants file.", 
                            &enchant_spec.mh_enchant_name.to_string()));
        self.mh.enchant = mh_enchant.copy();

        // oh enchant
        let oh_enchant = enchant_collection
            .get(&enchant_spec.oh_enchant_name.to_string())
            .expect(&format!("Could not find {} in enchants file.", 
                            &enchant_spec.oh_enchant_name.to_string()));
        self.oh.enchant = oh_enchant.copy();

    }

    fn set_talents(&mut self, args: &Args) {
        let talents_string = fs::read_to_string(&args.talents_file)
                .expect(&format!("Something went wrong reading talents from {}.",
                        args.talents_file));
        let talents: Talents = serde_yaml::from_str(&talents_string)
                .expect("Something went wrong deserializing talents");

        self.talents = talents;
    }

    fn set_buffs(&mut self, args: &Args) {
        let buffs_string = fs::read_to_string(&args.buffs_file)
                .expect(&format!("Something went wrong reading buffs from {}.", 
                        args.buffs_file));
        let buffs: Buffs = serde_yaml::from_str(&buffs_string)
                .expect("Something went wrong deserializing buffs");
        self.buffs = buffs;
    }

    fn apply_stats_from_buffs(&mut self) {
        if self.buffs.motw {
            self.prim_stats.agility += 12;
            self.prim_stats.strength += 12;
        } if self.buffs.bom {
            self.sec_stats.attack_power += 185;
        } if self.buffs.battle_shout {
            self.sec_stats.attack_power += 241;
        } if self.buffs.juju_power {
            self.prim_stats.strength += 30;
        } if self.buffs.juju_might {
            self.sec_stats.attack_power += 40;
        } if self.buffs.mongoose {
            self.prim_stats.agility += 25;
            self.sec_stats.crit += 0.02;
        } if self.buffs.grilled_squid {
            self.prim_stats.agility += 10;
        } if self.buffs.songflower_serenade {
            self.prim_stats.agility += 15;
            self.prim_stats.strength += 15;
            self.prim_stats.stamina += 15;
            self.sec_stats.crit += 0.05;
        } if self.buffs.bok {
            self.prim_stats.agility = 
                (self.prim_stats.agility as f32 * 1.1) as i32;
            self.prim_stats.strength = 
                (self.prim_stats.strength as f32 * 1.1) as i32;
            self.prim_stats.stamina = 
                (self.prim_stats.stamina as f32 * 1.1) as i32;
        }
    }

    fn set_armor_and_weapons(&mut self, item_spec: ItemSpecification) {
        let item_collection: ItemCollection = 
            ItemCollection::initialize_item_collection();

        for armor_name in &item_spec.armor_names {
            let armor = item_collection.armor.get(&armor_name.to_string()).
                expect(&format!("Could not find {} in item file.", armor_name));
            self.armor.push(armor.copy());
        }
        let mh = item_collection.weapons.get(&item_spec.mh_name.to_string()).
            expect(&format!("Could not find {} in item file.", 
                            &item_spec.mh_name.to_string()));
        self.mh = mh.copy();
        self.mh.set_mean_dmg();
        let oh = item_collection.weapons.get(&item_spec.oh_name.to_string()).
            expect(&format!("Could not find {} in item file.", 
                            &item_spec.oh_name.to_string()));
        self.oh = oh.copy();
        self.oh.set_mean_dmg();
    }

    fn apply_stats_from_armor_and_weapons(&mut self) {
        for i in 0..self.armor.len() {
            self.apply_prim_stats(self.armor[i].prim_stats);
            self.apply_sec_stats(self.armor[i].sec_stats);
        }
        self.apply_prim_stats(self.mh.prim_stats);
        self.apply_prim_stats(self.oh.prim_stats);

        self.apply_sec_stats(self.mh.sec_stats);
        self.apply_sec_stats(self.oh.sec_stats);
    }

    fn apply_prim_stats(&mut self, prim_stats: PrimStats) {
        self.prim_stats.agility += prim_stats.agility;
        self.prim_stats.strength += prim_stats.strength;
        self.prim_stats.stamina += prim_stats.stamina;
        self.prim_stats.arcane_resistance += prim_stats.arcane_resistance;
        self.prim_stats.fire_resistance += prim_stats.fire_resistance;
        self.prim_stats.frost_resistance += prim_stats.frost_resistance;
        self.prim_stats.shadow_resistance += prim_stats.shadow_resistance;
        self.prim_stats.sword_skill += prim_stats.sword_skill;
        self.prim_stats.dagger_skill += prim_stats.dagger_skill;
    }

    fn apply_sec_stats(&mut self, sec_stats: SecStats) {
        self.sec_stats.crit += sec_stats.crit;
        self.sec_stats.hit += sec_stats.hit;
        self.sec_stats.haste += sec_stats.haste;
        self.sec_stats.attack_power += sec_stats.attack_power;
    }

    pub fn get_sec_stats(&self) -> SecStats {
        return self.sec_stats.clone();
    }
}




            





