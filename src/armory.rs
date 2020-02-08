mod main


fn get_item(name: String) -> Item {

    if name == "bloodfang" {
        item = Item::new("bloodfang_hood")
    } else { panic!("Item not implemented {}.", name); }
}

enum ItemSet {
    Bloodfang,
    Nightslayer,
    Shadowcraft,
    Devilsaur,
    none
}

enum ItemSlot {
    Head,
    Neck,
    Shoulder,
    Back,
    Chest,
    Bracers,
    Hands,
    Waist,
    Legs,
    Feet,
    Ring,
    Trinket,
    Ranged,
    none,
}

enum WpnType {
    Sword,
    Dagger,
    Axe,
    Mace,
    none,
}

struct WpnSkill {
    weapon: WpnType,
    value: i16
}

impl WpnSkill {
    pub fn new(weapon_type: WpnType, value: i16) -> WpnSkill {
        WpnSkill {
            weapon_type: weapon_type,
            value: 0
        }
    }
}

pub struct Item {
    slot: ItemSlot,
    agility: u16,
    strength: u16,
    attack_power: u16,
    hit: f32,
    crit: f32,
    haste: f32,
    atk_proc: f32,
    wpn_skill: WpnSkill,
    set: ItemSet
}

impl Item {
    pub fn new(name: String) -> Item {

        if name == "bloodfang_hood" {
            Item {
                slot: ItemSlot::Head,
                agility: 27,
                strength: 19,
                attack_power: 0,
                hit: 0.0,
                crit: 0.01,
                haste: 0.0,
                atk_proc: 0.0,
                wpn_skill: WpnSkill::new(WpnType::none, 0),
            }
        }
    }
}

struct Talents {
    imp_backstab: u16,
    precision: u16,
    dw_specialization: u16,
    sword_specialization: u16,
    dagger_specialization: u16,
    weapon_expertise: u16,
    aggression: u16,
    opportunity: u16,
    improved_eviscerate: u16,
    malice: u16,
    ruthlessness: u16,
    improved_slice_and_dice: u16,
    relentless_strikes: u16,
    lethality: u16,
    combo_points: u16
}

impl Talents {
    pub fn new() -> Talents { 

        Talents{
            imp_backstab: 0,
            precision: 0,
            dw_specialization: 0,
            sword_specialization: 0,
            dagger_specialization: 0,
            weapon_expertise: 0,
            aggression: 0,
            opportunity: 0,
            improved_eviscerate: 0,
            malice: 0,
            ruthlessness: 0,
            improved_slice_and_dice: 0,
            relentless_strikes: 0,
            lethality: 0,
            combo_points: 0
        }
    }
}

enum Race {
    Human,
    NightElf,
    Dwarf,
    Gnome,
}

#[derive(Copy, Clone)]
struct Rogue {
    mh: Weapon,
    oh: Weapon,
    item_set: Vec<armory::Item>,
    stats: Stats,
    talents: Talents,
    energy: i8,
    combo_points: i8
}

impl Rogue {
    pub fn new(race: Race) -> Rogue { 

        Rogue{
            mh: Weapon::new(),
            oh: Weapon::new(),
            item_set: Vec::new(),
            stats: Stats::new_base(race),
            talents: Talents::new(),
            energy: 100,
            combo_points: 0,
        }
    }

    fn calculate_stats(&mut self) {
        for item in self.item_set {
            rogue.stats.agility += item.agility;
            rogue.stats.strength += item.strength;
            rogue.stats.attack_power += item.attack_power;
            rogue.stats.crit += item.crit;
            rogue.stats.hit += item.hit;
            rogue.stats.haste += item.haste;
            rogue.stats.dagger_skill += item.dagger_skill;
            rogue.stats.sword_skill += item.sword_skill;
            rogue.stats.extra_hit_proc_chance += item.extra_hit_proc_chance;
        }
        rogue.stats.agility += mh.agility + oh.agility;
        rogue.stats.strength += mh.strength + oh.strength;
        rogue.stats.attack_power += mh.attack_power + oh.attack_power;
        rogue.stats.crit += mh.crit + oh.crit;
        rogue.stats.hit += mh.hit + oh.hit;
        rogue.stats.haste += mh.haste + oh.haste;
        rogue.stats.dagger_skill += mh.dagger_skill + oh.dagger_skill;
        rogue.stats.sword_skill += mh.sword_skill + oh.sword_skill;
        rogue.stats.extra_hit_proc_chance += (mh.extra_hit_proc_chance 
                                              + oh.extra_hit_proc_chance);
    }
    
    fn calculate_hit_numbers(&mut self, args: &Args) {

        if rogue.weapon_expertise == 1 { 
            self.sword_skill += 3; 
            self.dagger_skill += 3; 
        }
        else if rogue.weapon_expertise == 2 { 
            self.sword_skill += 5; 
            self.dagger_skill += 5; 
        }

        if self.mh.wpn_type == WpnType::Sword {
            mh.extra_hit_proc_chance += 
                0.01 * rogue.sword_specialization as f32;
        }
        if self.oh.wpn_type == WpnType::Sword {
            oh.extra_hit_proc_chance += 
                0.01 * rogue.sword_specialization as f32;
        }
        
        rogue.stats.hit += rogue.precision as f32 * 0.01;

        self.set_crit_ratings(args.enemy_lvl);

        set_yellow_miss_chance(args.enemy_lvl);
        set_white_miss_chance();
        subtract_hit_from_miss(args);
        set_glancing_reduction(args.enemy_lvl);
        set_glancing_chance(args.enemy_lvl);
        set_dodge_chance(args.enemy_lvl);
    }

    fn set_dodge_chance(&mut self, enemy_lvl: i16) {
        if enemy_lvl < 60 { 
            self.mh.dodge_chance = 0.05; 
            self.oh.dodge_chance = 0.05; 
        } else {
            // MH
            let mh_wep_skill: i16;
            if self.mh.wpn_type == WpnType::Dagger { 
                mh_wep_skill = rogue.dagger_skill; 
            }
            else { mh_wep_skill = rogue.sword_skill; }
            self.mh.dodge_chance = (0.05 + (5 * enemy_lvl - mh_wep_skill) as f32 
                                    * 0.001); 
            // OH
            let oh_wep_skill: i16;
            if self.oh.wpn_type == WpnType::Dagger { 
                oh_wep_skill = rogue.dagger_skill; 
            }
            else { oh_wep_skill = rogue.sword_skill; }
            self.oh.dodge_chance = (0.05 + (5 * enemy_lvl - oh_wep_skill) as f32 
                                    * 0.001); 
        }
    }

    fn set_glancing_chance(&mut self, enemy_lvl: i16) {
        if enemy_lvl == 60 { self.glancing_chance = 0.1; }
        else if enemy_lvl == 61 { self.glancing_chance = 0.2; }
        else if enemy_lvl == 62 { self.glancing_chance = 0.3; }
        else if enemy_lvl == 63 { self.glancing_chance = 0.4; }
        else { panic!("No reliable glancing info on levels below 60"); }
    }

    fn set_glancing_reduction(&mut self, enemy_lvl: i16) {

        let mut delta_skill: i16;

        // MH
        if self.mh.wpn_type == WpnType::Dagger { 
            delta_skill = 5 * enemy_lvl - self.dagger_skill; 
        } else if self.mh.wpn_type == WpnType::Sword { 
            delta_skill = 5 * enemy_lvl - self.swords_skill; 
        } else { panic!("Weapon type not implemented."); }

        if      delta_skill == 15 { self.mh.glancing_red = 0.35; }
        else if delta_skill == 14 { self.mh.glancing_red = 0.31; }
        else if delta_skill == 13 { self.mh.glancing_red = 0.27; }
        else if delta_skill == 12 { self.mh.glancing_red = 0.23; }
        else if delta_skill == 11 { self.mh.glancing_red = 0.19; }
        else if delta_skill == 10 { self.mh.glancing_red = 0.15; }
        else if delta_skill == 09 { self.mh.glancing_red = 0.11; }
        else if delta_skill == 08 { self.mh.glancing_red = 0.07; }
        else if delta_skill <= 07 { self.mh.glancing_red = 0.05; }
        else { panic!("weapon skill-enemy defense not implemented"); }

        // OH
        if self.oh.wpn_type == WpnType::Dagger { 
            delta_skill = 5 * enemy_lvl - self.dagger_skill; 
        } else if self.oh.wpn_type == WpnType::Sword { 
            delta_skill = 5 * enemy_lvl - self.swords_skill; 
        } else { panic!("Weapon type not implemented."); }

        if      delta_skill == 15 { self.oh.glancing_red = 0.35; }
        else if delta_skill == 14 { self.oh.glancing_red = 0.31; }
        else if delta_skill == 13 { self.oh.glancing_red = 0.27; }
        else if delta_skill == 12 { self.oh.glancing_red = 0.23; }
        else if delta_skill == 11 { self.oh.glancing_red = 0.19; }
        else if delta_skill == 10 { self.oh.glancing_red = 0.15; }
        else if delta_skill == 09 { self.oh.glancing_red = 0.11; }
        else if delta_skill == 08 { self.oh.glancing_red = 0.07; }
        else if delta_skill <= 07 { self.oh.glancing_red = 0.05; }
        else { panic!("weapon skill-enemy defense not implemented"); }
    }

    fn subtract_hit_from_miss(&mut self, args: &Args) {

        // MH
        let wep_skill: i16;
        if self.mh.wpn_type == WpnType::Dagger { 
            wep_skill = rogue.daggers_skill; 
        }
        else { wep_skill = rogue.swords_skill; }

        // if target defense minus wep skill is 11 or more, one percent 
        // hit is negated
        if 5 * args.enemy_lvl - wep_skill > 10 {
            mh.yellow_miss = max_f32( 0.0, mh.yellow_miss - (rogue.hit - 0.01) );
            mh.white_miss = max_f32( 0.0, mh.white_miss - (rogue.hit - 0.01) );
        } else {
            mh.yellow_miss = max_f32(0.0, mh.yellow_miss - rogue.hit);
            mh.white_miss = max_f32(0.0, mh.white_miss - rogue.hit);
        }

        // OH
        let wep_skill: i16;
        if oh.is_dagger { wep_skill = rogue.daggers_skill; }
        else { wep_skill = rogue.swords_skill; }

        // if target defense minus wep skill is 11 or more, one percent 
        // hit is negated
        if 5 * args.enemy_lvl - wep_skill > 10 {
            oh.yellow_miss = main::max_f32(
                0.0, self.oh.yellow_miss - (self.stats.hit - 0.01)
                );
            oh.white_miss = main::max_f32(
                0.0, self.oh.white_miss - (self.stats.hit - 0.01) 
                );
        } else {
            oh.yellow_miss = main::max_f32(0.0, self.oh.yellow_miss - self.hit);
            oh.white_miss = main::max_f32(0.0, self.oh.white_miss - self.hit);
        }

    }

    fn set_white_miss_chance(&mut self) {
        self.mh.white_miss = 0.8 * self.mh.yellow_miss + 0.2;
        self.oh.white_miss = 0.8 * self.oh.yellow_miss + 0.2;
    }

    fn set_crit_ratings(&mut self, enemy_lvl: i16) {

        let mut common_crit = self.crit; // crit directly from gear
        common_crit += 0.01 * self.agility as f32 / 29.0; // crit from agility
        common_crit += 0.01 * self.malice as f32; // talent

        // when facing a lvl 63, 1.8 crit is removed from non-agi crit,
        // assuming that the rogue has
        // at least 2% crit gained directly from gear
        //
        // + 1% crit reduction per lvl above player level (or .2% per difference
        // in attack skill and defense skill) brings the crit down 
        // a maximum of 4.8. Source: attack table
        if enemy_lvl == 63 { common_crit -= 0.018; }
        common_crit -= (enemy_lvl - 60) as f32 * 0.01;

        // give mh its crit
        self.mh.crit = common_crit;
        if self.mh.wpn_type == WpnType::Dagger {
            self.mh.crit += 0.01 * self.dagger_specialization as f32;
            self.mh.crit_backstab = self.mh.crit;
            self.mh.crit_backstab += 0.1 * self.imp_backstab as f32; 
        }
        if self.mh.crit < 0.0 { self.mh.crit = 0.0; }
        if self.mh.crit_backstab < 0.0 { self.mh.crit = 0.0; }
        
        // give oh its crit
        self.oh.crit = common_crit;
        if self.oh.wpn_type == WpnType::Dagger {
            self.oh.crit += 0.01 * self.dagger_specialization as f32;
        }
        if self.oh.crit < 0.0 { self.oh.crit = 0.0; }
        if self.oh.crit_backstab < 0.0 { self.oh.crit = 0.0; }

    }

    fn set_yellow_miss_chance(&mut self, enemy_lvl: i16) {
        // MH 
        let mut delta_skill: i16;
        if self.oh.wpn_type == WpnType::Dagger {
            delta_skill = 5 * enemy_lvl - rogue.daggers_skill;
        } else {
            delta_skill = 5 * enemy_lvl - rogue.swords_skill;
        }
        if delta_skill < 0 { self.mh.yellow_miss = 0.05; }
        else if delta_skill <= 10 && delta_skill >= 0 { 
            self.mh.yellow_miss = 0.05 + 0.001 * delta_skill as f32; 
        } else if delta_skill == 11 { self.mh.yellow_miss = 0.072; }
        else if delta_skill == 12 { self.mh.yellow_miss = 0.074; }
        else if delta_skill == 13 { self.mh.yellow_miss = 0.076; }
        else if delta_skill == 14 { self.mh.yellow_miss = 0.078; }
        else if delta_skill == 15 { self.mh.yellow_miss = 0.080; }
        else { panic!("Weapon skill-enemy lvl combo not implemented"); }
    }
}

struct Stats {
    agility: u16,
    strength: u16,
    attack_power: u16,
    crit: f32,
    hit: f32,
    haste: f32,
    dagger_skill: u16,
    sword_skill: u16,
    extra_hit_proc_chance: f32
}

impl Stats {
    fn new_base(race: Race) -> Stats {
        if race == Race::Human {
            Stats {
                agility: 130,
                strength: 80,
                attack_power: 100,
                crit: 0.0,
                hit: 0.0,
                haste: 0.0,
                dagger_skill: 300,
                sword_skill: 305,
                extra_hit_proc_chance: 0.0
            }
        } else { panic!("Race not implemented yet!"); }
    }

    fn new() -> Stats {
        Stats {
            agility: 0,
            strength: 0,
            attack_power: 0,
            crit: 0.0,
            hit: 0.0,
            haste: 0.0,
            dagger_skill: 0,
            sword_skill: 0,
            extra_hit_proc_chance: 0.0
        }
    }
}

struct Weapon {
    speed: f32,
    max_dmg: u16,
    min_dmg: u16,
    mean_dmg: f32,
    stats: Stats,
    enchant: String,
    crusader: f32, // the time left on crusader
    is_offhand: bool,
    wpn_type: WpnType,
    wpn_skill: WpnSkill,
    crit: f32,
    crit_backstab: f32,
    dodge_chance: f32,
    white_miss: f32,
    yellow_miss: f32,
    glancing_red: f32,
    extra_hit_proc_chance: f32
}

impl Weapon {
    pub fn clean_wep() -> Weapon {
        Weapon {
            speed: 0.0,
            max_dmg: 0,
            min_dmg: 0,
            mean_dmg: 0.0,
            stats: Stats::new(),
            enchant: "none".to_string(),
            crusader: 0.0,
            is_offhand: false,
            wpn_type: WpnType::none,
            wpn_skill: WpnSkill::new(WpnType::none, 0),
            crit: 0.0,
            crit_backstab: 0.0,
            dodge_chance: 0.0,
            white_miss: 0.0,
            yellow_miss: 0.0,
            glancing_red: 0.0,
            extra_hit_proc_chance: 0.0
        }
    }

    pub fn new(name: String) -> Weapon {
        let mut wep = clean_wep();
        if name == "gutgore_ripper" {
            wep.speed: 1.8;
            wep.min_dmg = 63;
            wep.max_dmg = 119;
            wep.wpn_type = WpnType::Dagger;
                
        } else if name == "distracting_dagger" {
            wep.speed: 1.3;
            wep.min_dmg = 42;
            wep.max_dmg = 64;
            wep.wpn_type = WpnType::Dagger;
            wep.wpn_skill = WpnSkill::new(WpnType::Dagger, 6);

        } else { panic!("Weapon name not recognized {}", name); }

        wep.mean_dmg = (wep.min_dmg + wep.max_dmg) as f32 / 2.0;
        return wep
    }
}
