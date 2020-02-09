use crate::Args;
use crate::max_f32;


enum ItemSet {
    Bloodfang,
    Nightslayer,
    Shadowcraft,
    Devilsaur,
    None
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
    None,
}

enum WpnType {
    Sword,
    Dagger,
    Axe,
    Mace,
    None,
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
                wpn_skill: WpnSkill::new(WpnType::None, 0),
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

struct CDs {
    blade_flurry: f32,
    snd: f32,
    adrenaline_rush: f32,
    used_cds: bool,
    used_thistle_tea: bool
}
    
impl CDs {
    pub fn new() -> CDs {
        CDs {
            blade_flurry: 0.0,
            snd: 0.0,
            adrenaline_rush: 0.0,
            used_cds: false,
            used_thistle_tea: false
        }
    }
}

struct Crusader {
    mh_crus: f32,
    oh_crus: f32
}

impl Crusader {
    pub fn new() -> Crusader {
        Crusader {
            mh_crus: 0.0,
            oh_crus: 0.0
        }
    }
}

struct RaidBuffs {
    motw: bool,
    trueshot_aura: bool,
    dire_maul_buffs: bool,
    ony_buff: bool,
    bom: bool,
    battle_shout: bool,
    juju_power: bool,
    juju_might: bool,
    mongoose: bool,
    grilled_squid: bool,
    bok: bool
}

impl RaidBuffs {
    pub fn all_buffs() -> RaidBuffs {
        RaidBuffs {
            motw: true,
            trueshot_aura: true,
            dire_maul_buffs: true,
            ony_buff: true,
            bom: true,
            battle_shout: true,
            juju_power: true,
            juju_might: true,
            mongoose: true,
            grilled_squid: true,
            bok: true
        }
    }
}

struct Buffs {
    cds: CDs,
    raid_buffs: RaidBuffs,
    crusader: Crusader
}

impl Buffs {
    pub fn new() -> Buffs {
        Buffs {
            cds: CDs::new(),
            raid_buffs: RaidBuffs::get_all(),
            crusader: Crusader::new()
        }
    }
}

pub struct Rogue {
    mh: Weapon,
    oh: Weapon,
    item_set: Vec<Item>,
    stats: Stats,
    buffs: Buffs,
    pub talents: Talents,
    energy: i8,
    combo_points: i8,
}

impl Rogue {
    pub fn new(race: Race) -> Rogue { 

        Rogue{
            mh: Weapon::new(),
            oh: Weapon::new(),
            item_set: Vec::new(),
            stats: Stats::new_base(race),
            buffs: Buffs::new(),
            talents: Talents::new(),
            energy: 100,
            combo_points: 0,
        }
    }
    
    pub fn get_agility(self) -> u16 { self.stats.agility }
    pub fn get_strength(self) -> u16 { self.stats.strength }
    pub fn get_attack_power(self) -> u16 { self.stats.attack_power }
    pub fn get_crit(self) -> f32 { self.stats.crit }
    pub fn get_hit(self) -> f32 { self.stats.hit }
    pub fn get_haste(self) -> f32 { self.stats.haste }
    pub fn get_dagger_skill(self) -> u16 { self.stats.dagger_skill }
    pub fn get_sword_skill(self) -> u16 { self.stats.sword_skill }
    pub fn get_extra_hit_proc_chance(self) -> f32 { 
        self.stats.extra_hit_proc_chance 
    }

    fn print_hit_chances(self) {

        println!("*** MH: White hits summary ***");
        println!("miss chance: {:}", self.mh.white_miss);
        println!("dodge chance: {:}", self.mh.dodge_chance);
        println!("glancing chance: {:}", rogue.glancing_chance);
        println!("crit chance: {:}", self.mh.crit);
        let mut tmp = self.mh.white_miss;
        let mut tmp1 = tmp + self.mh.dodge_chance;
        let mut tmp2 = tmp1 + self.glancing_chance;
        let mut tmp3 = tmp2 + self.mh.crit;
        println!("hit chance: {:}", 1.0 - tmp3);
        println!("{:}-{:}-{:}-{:}\n", tmp, tmp1, tmp2, tmp3);
        
        println!("*** MH: Yellow hits summary ***");
        println!("miss chance: {:}", self.mh.yellow_miss);
        println!("dodge chance: {:}", self.mh.dodge_chance);
        println!("crit chance: {:}", self.mh.crit);
        tmp = self.mh.yellow_miss;
        tmp1 = tmp + self.mh.dodge_chance;
        tmp2 = tmp1 + self.mh.crit;
        println!("hit chance: {:}", 1.0 - tmp2);
        println!("{:}-{:}-{:}\n", tmp, tmp1, tmp2);

        if self.mh.is_dagger {
            println!("*** MH: Backstab summary ***");
            println!("miss chance: {:}", self.mh.yellow_miss);
            println!("dodge chance: {:}", self.mh.dodge_chance);
            println!("crit chance: {:}", self.mh.crit_backstab);
            tmp = self.mh.yellow_miss;
            tmp1 = tmp + self.mh.dodge_chance;
            tmp2 = tmp1 + self.mh.crit_backstab;
            println!("hit chance: {:}", 1.0 - tmp2);
            println!("{:}-{:}-{:}\n", tmp, tmp1, tmp2);
        }

        println!("*** OH: White hits summary ***");
        println!("miss chance: {:}", self.oh.white_miss);
        println!("dodge chance: {:}", self.oh.dodge_chance);
        println!("glancing chance: {:}", rogue.glancing_chance);
        println!("crit chance: {:}", self.oh.crit);
        tmp = self.oh.white_miss;
        tmp1 = tmp + self.oh.dodge_chance;
        tmp2 = tmp1 + rogue.glancing_chance;
        tmp3 = tmp2 + self.oh.crit;
        println!("hit chance: {:}", 1.0 - tmp3);
        println!("{:}-{:}-{:}-{:}\n", tmp, tmp1, tmp2, tmp3);
        
    }

    fn add_raid_buffs(&mut self) {
        if self.buffs.raid_buffs.motw { 
            self.stats.agility += 12; 
            self.stats.strength += 12; 
        }
        if self.buffs.raid_buffs.trueshot_aura { self.stats.attack_power += 200; }
        if self.buffs.raid_buffs.dm_buffs { self.stats.attack_power += 200; }
        if self.buffs.raid_buffs.ony_buff { 
            self.stats.attack_power += 140; 
            self.stats.crit += 0.05; 
        }
        if self.buffs.raid_buffs.bom { self.stats.attack_power += 185; }
        if self.buffs.raid_buffs.battle_shout { self.stats.attack_power += 241; }
        if self.buffs.raid_buffs.juju_power { self.stats.strength += 30; }
        if self.buffs.raid_buffs.juju_might { self.stats.attack_power += 40; }
        if self.buffs.raid_buffs.mongoose { 
            self.stats.agility += 25; 
            self.stats.crit += 0.02; 
        }
        if self.buffs.raid_buffs.grilled_squid { self.stats.agility += 10; }
        if self.buffs.raid_buffs.bok { 
            self.stats.agility = (self.stats.agility as f32 * 1.1) as u16; 
            self.stats.strength = (self.stats.strength as f32 * 1.1) as u16; 
        }
    }

    fn calculate_stats(&mut self) {
        for item in self.item_set {
            self.stats.agility += item.agility;
            self.stats.strength += item.strength;
            self.stats.attack_power += item.attack_power;
            self.stats.crit += item.crit;
            self.stats.hit += item.hit;
            self.stats.haste += item.haste;
            self.stats.dagger_skill += item.dagger_skill;
            self.stats.sword_skill += item.sword_skill;
            self.stats.extra_hit_proc_chance += item.extra_hit_proc_chance;
        }
        self.stats.agility += self.mh.agility + self.oh.agility;
        self.stats.strength += self.mh.strength + self.oh.strength;
        self.stats.attack_power += self.mh.attack_power + self.oh.attack_power;
        self.stats.crit += self.mh.crit + self.oh.crit;
        self.stats.hit += self.mh.hit + self.oh.hit;
        self.stats.haste += self.mh.haste + self.oh.haste;
        self.stats.dagger_skill += self.mh.dagger_skill + self.oh.dagger_skill;
        self.stats.sword_skill += self.mh.sword_skill + self.oh.sword_skill;
        self.stats.extra_hit_proc_chance += self.mh.extra_hit_proc_chance 
                                            + self.oh.extra_hit_proc_chance;
    }
    
    fn calculate_hit_numbers(&mut self, args: &Args) {

        if self.weapon_expertise == 1 { 
            self.sword_skill += 3; 
            self.dagger_skill += 3; 
        }
        else if self.weapon_expertise == 2 { 
            self.sword_skill += 5; 
            self.dagger_skill += 5; 
        }

        if self.mh.wpn_type == WpnType::Sword {
            self.mh.extra_hit_proc_chance += 
                0.01 * self.talents.sword_specialization as f32;
        }
        if self.oh.wpn_type == WpnType::Sword {
            self.oh.extra_hit_proc_chance += 
                0.01 * self.talents.sword_specialization as f32;
        }
        
        self.stats.hit += self.precision as f32 * 0.01;

        self.set_crit_ratings(args.enemy_lvl);

        self.set_yellow_miss_chance(args.enemy_lvl);
        self.set_white_miss_chance();
        self.subtract_hit_from_miss(args);
        self.set_glancing_reduction(args.enemy_lvl);
        self.set_glancing_chance(args.enemy_lvl);
        self.set_dodge_chance(args.enemy_lvl);
    }

    fn set_dodge_chance(&mut self, enemy_lvl: i16) {
        if enemy_lvl < 60 { 
            self.mh.dodge_chance = 0.05; 
            self.oh.dodge_chance = 0.05; 
        } else {
            // MH
            let mh_wep_skill: i16;
            if self.mh.wpn_type == WpnType::Dagger { 
                mh_wep_skill = self.dagger_skill; 
            }
            else { mh_wep_skill = self.sword_skill; }
            self.mh.dodge_chance = 0.05 + (5 * enemy_lvl - mh_wep_skill) as f32 
                                   * 0.001; 
            // OH
            let oh_wep_skill: i16;
            if self.oh.wpn_type == WpnType::Dagger { 
                oh_wep_skill = self.dagger_skill; 
            }
            else { oh_wep_skill = self.sword_skill; }
            self.oh.dodge_chance = 0.05 + (5 * enemy_lvl - oh_wep_skill) as f32 
                                   * 0.001; 
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
            delta_skill = 5 * enemy_lvl - self.sword_skill; 
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
            delta_skill = 5 * enemy_lvl - self.sword_skill; 
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
            wep_skill = self.dagger_skill; 
        }
        else { wep_skill = self.sword_skill; }

        // if target defense minus wep skill is 11 or more, one percent 
        // hit is negated
        if 5 * args.enemy_lvl - wep_skill > 10 {
           self.mh.yellow_miss = max_f32(
               0.0, self.mh.yellow_miss - (self.hit - 0.01) );
           self.mh.white_miss = max_f32(
               0.0, self.mh.white_miss - (self.hit - 0.01) );
        } else {
           self.mh.yellow_miss = max_f32(0.0, self.mh.yellow_miss - self.hit);
           self.mh.white_miss = max_f32(0.0, self.mh.white_miss - self.hit);
        }

        // OH
        let wep_skill: i16;
        if self.oh.is_dagger { wep_skill = self.dagger_skill; }
        else { wep_skill = self.sword_skill; }

        // if target defense minus wep skill is 11 or more, one percent 
        // hit is negated
        if 5 * args.enemy_lvl - wep_skill > 10 {
            self.oh.yellow_miss = max_f32(
                0.0, self.oh.yellow_miss - (self.stats.hit - 0.01)
                );
            self.oh.white_miss = max_f32(
                0.0, self.oh.white_miss - (self.stats.hit - 0.01) 
                );
        } else {
            self.oh.yellow_miss = max_f32(0.0, self.oh.yellow_miss - self.hit);
            self.oh.white_miss = max_f32(0.0, self.oh.white_miss - self.hit);
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
            delta_skill = 5 * enemy_lvl - self.dagger_skill;
        } else {
            delta_skill = 5 * enemy_lvl - self.sword_skill;
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

pub struct Stats {
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

pub struct Weapon {
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
            wpn_type: WpnType::None,
            wpn_skill: WpnSkill::new(WpnType::None, 0),
            crit: 0.0,
            crit_backstab: 0.0,
            dodge_chance: 0.0,
            white_miss: 0.0,
            yellow_miss: 0.0,
            glancing_red: 0.0,
            extra_hit_proc_chance: 0.0
        }
    }

    pub fn get_speed(self) -> f32 { self.speed }
    pub fn crusader_active(self) -> bool {self.crusader > 0.0 }

    pub fn new(name: String) -> Weapon {
        let mut wep = Weapon::clean_wep();
        if name == "gutgore_ripper" {
            wep.speed = 1.8;
            wep.min_dmg = 63;
            wep.max_dmg = 119;
            wep.wpn_type = WpnType::Dagger;
                
        } else if name == "distracting_dagger" {
            wep.speed = 1.3;
            wep.min_dmg = 42;
            wep.max_dmg = 64;
            wep.wpn_type = WpnType::Dagger;
            wep.wpn_skill = WpnSkill::new(WpnType::Dagger, 6);

        } else { panic!("Weapon name not recognized {}", name); }

        wep.mean_dmg = (wep.min_dmg + wep.max_dmg) as f32 / 2.0;
        return wep
    }
}
