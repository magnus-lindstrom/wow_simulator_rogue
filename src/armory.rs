extern crate serde;
extern crate serde_yaml;

use crate::utils::Args;
use crate::weights::StatShift;
use std::fs;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

const ARMOR_COLLECTION_PATH: &str = "db/armor.yaml";
const ENCHANT_COLLECTION_PATH: &str = "db/enchants.yaml";
const SET_BONUSES_COLLECTION_PATH: &str = "db/set_bonuses.yaml";
const WEAPON_COLLECTION_PATH: &str = "db/weapons.yaml";


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
    pub agility: i32,
    pub strength: i32,
    stamina: i32,
    pub sword_skill: i32,
    pub dagger_skill: i32
}

impl PrimStats {
    pub fn new_from_race(race: Race) -> PrimStats {
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

    fn print_stats(&self, talents: &Talents) {
        let mut dagger_skill = self.dagger_skill;
        let mut sword_skill = self.sword_skill;

        if talents.weapon_expertise == 1 { 
            dagger_skill += 3; 
            sword_skill += 3; 
        }
        else if talents.weapon_expertise == 2 { 
            dagger_skill += 5; 
            sword_skill += 5; 
        }
        let msg = format!("\n*** Primary stats ***\n\
        Strength: {}\n\
        Agility: {}\n\
        Stamina: {}\n\
        Sword skill: {}\n\
        Dagger_skill: {}", self.strength, self.agility, self.stamina, 
        sword_skill, dagger_skill);
        println!("{}", msg);
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
    pub fn new_from_race(race: Race) -> SecStats {
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

    fn print_stats(&self, prim_stats: &PrimStats, talents: &Talents) {
        let mut weapon_skill = prim_stats.dagger_skill;
        if talents.weapon_expertise == 1 { weapon_skill += 3; }
        else if talents.weapon_expertise == 2 { weapon_skill += 5; }

        let mut crit = self.crit + 0.01 * talents.malice as f32;
        crit += 0.01 * talents.dagger_specialization as f32;
        crit += 0.0004 * (weapon_skill - 300) as f32;
        
        let hit = self.hit + 0.01 * talents.precision as f32;

        let msg = format!("\n*** Secondary stats vs lvl 60 ***\n\
        Crit: {}\n\
        Hit: {}\n\
        Haste: {}\n\
        Attack power: {}", crit, hit, self.haste, 
        self.attack_power);
        println!("{}", msg);
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

#[derive(Debug,Clone,Serialize,Deserialize,PartialEq)]
pub enum HitProcc {
    Dmg(String, f32, f32, f32), // name, damage, resist chance, procc chance
    Strength(String, i32, f32, f32), // name, strength, duration, procc chance
    ExtraAttack(String, f32), // name, procc chance
    None,
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Weapon {
    name: String,
    weapon_type: WeaponType,
    set_tag: String,
    prim_stats: PrimStats,
    sec_stats: SecStats,
    swing_interval: f32,
    min_dmg: f32,
    max_dmg: f32,
    mean_dmg: f32,
    hit_procc: HitProcc
}

impl Weapon {
    fn new() -> Weapon {
        Weapon {
            name: "".to_string(),
            weapon_type: WeaponType::None,
            set_tag: "".to_string(),
            prim_stats: PrimStats::new_from_race(Race::None),
            sec_stats: SecStats::new_from_race(Race::None),
            swing_interval: 0.0,
            min_dmg: 0.0,
            max_dmg: 0.0,
            mean_dmg: 0.0,
            hit_procc: HitProcc::None
        }
    }

    fn copy(&self) -> Weapon {
        Weapon {
            name: self.name.to_string(),
            weapon_type: self.weapon_type.clone(),
            set_tag: self.set_tag.to_string(),
            prim_stats: self.prim_stats.clone(),
            sec_stats: self.sec_stats.clone(),
            swing_interval: self.swing_interval,
            min_dmg: self.min_dmg,
            max_dmg: self.max_dmg,
            mean_dmg: self.mean_dmg,
            hit_procc: self.hit_procc.clone()
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

    pub fn get_hit_procc(&self) -> HitProcc {
        return self.hit_procc.clone();
    }
}

#[derive(Debug,Clone,Serialize,Deserialize)]
enum Slot {
    Head,
    Neck,
    Shoulders,
    Back,
    Chest,
    Wrists,
    Hands,
    Waist,
    Legs,
    Feet,
    Ring,
    Trinket,
    Ranged,
}
 
#[derive(Debug,Serialize,Deserialize)]
pub struct Armor {
    name: String,
    set_tag: String,
    slot: Slot,
    prim_stats: PrimStats,
    sec_stats: SecStats,
    pub hit_procc: HitProcc
}

impl Armor {
    fn copy(&self) -> Armor {
        Armor {
            name: self.name.to_string(),
            set_tag: self.set_tag.to_string(),
            slot: self.slot.clone(),
            prim_stats: self.prim_stats.clone(),
            sec_stats: self.sec_stats.clone(),
            hit_procc: self.hit_procc.clone()
        }
    }
}

#[derive(Debug,Serialize,Deserialize)]
struct CharacterSpecification {
    items: ItemSpecification,
    enchants: EnchantSpecification,
    buffs: Buffs,
    talents: Talents
}

impl CharacterSpecification {
    pub fn get_char_spec(args: &Args) -> CharacterSpecification {

        let character_spec_string = fs::read_to_string(&args.spec_file)
            .expect(&format!("Something went wrong items from {}.",
                    args.spec_file));
        let character_spec: CharacterSpecification = serde_yaml::from_str(
            &character_spec_string).unwrap();
        return character_spec;
    }
}

#[derive(Debug,Serialize,Deserialize)]
struct ItemSpecification {
    mh_name: String,
    oh_name: String,
    armor_names: Vec<String>
}

#[derive(Debug,Serialize,Deserialize)]
struct EnchantSpecification {
    armor_enchant_names: Vec<String>,
    mh_enchant_names: Vec<String>,
    oh_enchant_names: Vec<String>
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Enchant {
    pub name: String,
    prim_stats: PrimStats,
    sec_stats: SecStats,
    pub hit_procc: HitProcc,
    pub extra_damage: f32
}

impl Enchant {
    fn copy(&self) -> Enchant {
        Enchant {
            name: self.name.to_string(),
            prim_stats: self.prim_stats.clone(),
            sec_stats: self.sec_stats.clone(),
            hit_procc: self.hit_procc.clone(),
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

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum SpecialBonus {
    NewEnergyCap(i32),
    None
}

#[derive(Debug,Serialize,Deserialize)]
pub struct SetBonus {
    pub set_tag: String,
    pub pieces_needed: i32,
    pub prim_stats: PrimStats,
    pub sec_stats: SecStats,
    pub special_bonus: SpecialBonus
}

impl SetBonus {
    fn copy(&self) -> SetBonus {
        SetBonus {
            set_tag: self.set_tag.to_string(),
            pieces_needed: self.pieces_needed,
            prim_stats: self.prim_stats.clone(),
            sec_stats: self.sec_stats.clone(),
            special_bonus: self.special_bonus.clone()
        }
    }
}

#[derive(Clone,Debug,Serialize,Deserialize)]
pub enum CooldownEffect {
    EnergyRegenMultiplier(i32, f32), // multiplier, duration
    AttackSpeedMultiplier(f32, f32), // multiplier, duration
    InstantEnergyRefill(i32) // energy
}

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct Cooldown {
    pub name: String,
    pub effect: CooldownEffect,
    pub is_active: bool,
    pub time_left: f32,
    pub cd: f32,
    pub cd_left: f32,
    pub cost: i32,
    pub use_below_energy: i32
}

impl Cooldown {
    pub fn get_common_cooldowns() -> Vec<Cooldown> {
        let mut cd_vector = Vec::new();
        cd_vector.push(
            Cooldown {
                name: "Adrenaline rush".to_string(),
                effect: CooldownEffect::EnergyRegenMultiplier(2, 15.0),
                is_active: false,
                time_left: 0.0,
                cd: 5.0 * 60.0,
                cd_left: 0.0,
                cost: 0,
                use_below_energy: 50
            });
        cd_vector.push(
            Cooldown {
                name: "Blade flurry".to_string(),
                effect: CooldownEffect::AttackSpeedMultiplier(1.2, 15.0),
                is_active: false,
                time_left: 0.0,
                cd: 2.0 * 60.0,
                cd_left: 0.0,
                cost: 20,
                use_below_energy: 100
            });
        cd_vector.push(
            Cooldown {
                name: "Thistle tea".to_string(),
                effect: CooldownEffect::InstantEnergyRefill(100),
                is_active: false,
                time_left: 0.0,
                cd: 5.0 * 60.0,
                cd_left: 0.0,
                cost: 0,
                use_below_energy: 10
            });
        return cd_vector;
    }
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Character {
    race: Race,
    pub prim_stats: PrimStats,
    pub sec_stats: SecStats,
    buffs: Buffs,
    pub armor_enchants: Vec<Enchant>,
    pub mh_enchants: Vec<Enchant>, // both poisons and actual enchants
    pub oh_enchants: Vec<Enchant>, // both poisons and actual enchants
    pub mh: Weapon,
    pub oh: Weapon,
    pub armor: Vec<Armor>,
    pub set_bonuses: Vec<SetBonus>,
    pub talents: Talents,
    pub cooldowns: Vec<Cooldown>
}

impl Character {
    pub fn create_character(args: &Args) -> Character {
        let mut character = Character::new(Race::Human);

        let char_spec = CharacterSpecification::get_char_spec(args);
        character.set_armor_and_weapons(char_spec.items);
        character.apply_set_bonuses();
        character.set_enchants(char_spec.enchants);
        character.set_buffs(char_spec.buffs);
        character.set_talents(char_spec.talents);
        character.apply_stats_from_armor_and_weapons();
        character.apply_stats_from_buffs();
        character.apply_stats_from_enchants();
        return character;
    }

    pub fn apply_stat_shift(&mut self, stat_shift: &StatShift) {
        self.apply_prim_stats(stat_shift.prim_stats);
        self.apply_sec_stats(stat_shift.sec_stats);
    }

    pub fn convert_stats_and_set_cooldowns(&mut self) {
        self.convert_primary_stats_to_secondary();
        self.set_common_cooldowns();
    }

    pub fn print_all_stats(&self, args: &Args) {
        if args.verb > 2 {
            self.prim_stats.print_stats(&self.talents);
            self.sec_stats.print_stats(&self.prim_stats, &self.talents);
        }
    }

    fn set_common_cooldowns(&mut self) {
        self.cooldowns = Cooldown::get_common_cooldowns();
    }

    fn new(race: Race) -> Character {
        Character {
            race: race,
            prim_stats: PrimStats::new_from_race(race),
            sec_stats: SecStats::new_from_race(race),
            buffs: Buffs::new(),
            armor_enchants: Vec::new(),
            mh_enchants: Vec::new(),
            oh_enchants: Vec::new(),
            mh: Weapon::new(),
            oh: Weapon::new(),
            armor: Vec::new(),
            set_bonuses: Vec::new(),
            talents: Talents::new(),
            cooldowns: Vec::new()
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
            let enchant = enchant_collection
                .get(&enchant_name.to_string())
                .expect(&format!("Could not find {} in enchants file.", 
                                 enchant_name));
            self.armor_enchants.push(enchant.copy());
        }

        // mh enchants
        for enchant_name in &enchant_spec.mh_enchant_names {
            let enchant = enchant_collection
                .get(&enchant_name.to_string())
                .expect(&format!("Could not find {} in enchants file.", 
                                enchant_name));
            self.mh_enchants.push(enchant.copy());
        }

        // oh enchants
        for enchant_name in &enchant_spec.oh_enchant_names {
            let enchant = enchant_collection
                .get(&enchant_name.to_string())
                .expect(&format!("Could not find {} in enchants file.", 
                                enchant_name));
            self.oh_enchants.push(enchant.copy());
        }

    }

    fn set_talents(&mut self, talents: Talents) {
        self.talents = talents;
    }

    fn set_buffs(&mut self, buffs: Buffs) {
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

    fn get_armor_by_name(&self, name: String) -> Armor {
        let item_collection: ItemCollection = 
            ItemCollection::initialize_item_collection();
        let armor = item_collection.armor.get(&name).
            expect(&format!("Could not find {} in item file.", name));
        return armor.copy();
    }

    fn get_weapon_by_name(&self, name: String) -> Weapon {
        let item_collection: ItemCollection = 
            ItemCollection::initialize_item_collection();
        let weapon = item_collection.weapons.get(&name).
            expect(&format!("Could not find {} in item file.", name));
        return weapon.copy();
    }

    fn set_armor_and_weapons(&mut self, item_spec: ItemSpecification) {
        for armor_name in &item_spec.armor_names {
            let armor = self.get_armor_by_name(armor_name.to_string());
            self.armor.push(armor.copy());
        }
        let mh = self.get_weapon_by_name(item_spec.mh_name.to_string());
        self.mh = mh.copy();
        self.mh.set_mean_dmg();
        let oh = self.get_weapon_by_name(item_spec.oh_name.to_string());
        self.oh = oh.copy();
        self.oh.set_mean_dmg();
    }

    fn get_set_bonus_db(&self) -> Vec<SetBonus> {
        let set_bonuses_string = fs::read_to_string(SET_BONUSES_COLLECTION_PATH)
                .expect("Something went wrong reading items from file.");
        let set_bonuses: Vec<SetBonus> = serde_yaml::from_str(
            &set_bonuses_string).expect("Could not parse set bonus db");
        return set_bonuses;
    }

    fn count_set_pieces_worn(&self) -> HashMap<String,i32> {
        let mut equipped_sets: HashMap<String,i32> = HashMap::new();
        for i in 0..self.armor.len() {
            if self.armor[i].set_tag == "" { continue; }
            let count = equipped_sets.entry(self.armor[i].set_tag.to_string())
                .or_insert(0);
            *count += 1;
        }
        return equipped_sets;
    }

    fn apply_set_bonuses(&mut self) {
        let pieces_worn = self.count_set_pieces_worn();

        let db: Vec<SetBonus> = self.get_set_bonus_db();
        for i in 0..db.len() {
            match pieces_worn.get(&db[i].set_tag) {
                Some(nr) => {
                    if nr >= &db[i].pieces_needed {
                        self.set_bonuses.push(db[i].copy());
                    }
                },
                None => continue
            }
        }
    }

    fn apply_stats_from_enchants(&mut self) {
        for i in 0..self.armor_enchants.len() {
            self.apply_prim_stats(self.armor_enchants[i].prim_stats);
            self.apply_sec_stats(self.armor_enchants[i].sec_stats);
        }
        for i in 0..self.mh_enchants.len() {
            self.apply_prim_stats(self.mh_enchants[i].prim_stats);
            self.apply_sec_stats(self.mh_enchants[i].sec_stats);
        }
        for i in 0..self.oh_enchants.len() {
            self.apply_prim_stats(self.oh_enchants[i].prim_stats);
            self.apply_sec_stats(self.oh_enchants[i].sec_stats);
        }
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
        self.prim_stats.sword_skill += prim_stats.sword_skill;
        self.prim_stats.dagger_skill += prim_stats.dagger_skill;
    }

    fn apply_sec_stats(&mut self, sec_stats: SecStats) {
        self.sec_stats.crit += sec_stats.crit;
        self.sec_stats.hit += sec_stats.hit;
        self.sec_stats.haste += sec_stats.haste;
        self.sec_stats.attack_power += sec_stats.attack_power;
    }
}




            





