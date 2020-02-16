use crate::utils::{Args,max_f32};
use crate::armory::{Character,SecStats,WeaponType};


#[derive(Debug)]
pub struct Simulator {
    timers: Timers,
    mh: MhSimulator, 
    oh: OhSimulator,
    dmg_modifiers: DmgModifiers,
    extra_attacks: i32,
    rotation: Rotation,
    dt: f32,
    fight_length: f32,
    iterations: i32,
    verb: bool
}

impl Simulator {
    pub fn new() -> Simulator {
        Simulator {
            timers: Timers::new(),
            mh: MhSimulator::new(),
            oh: OhSimulator::new(),
            dmg_modifiers: DmgModifiers::new(),
            extra_attacks: 0,
            rotation: Rotation::None,
            dt: 0.0,
            fight_length: 0.0,
            iterations: 0,
            verb: false
        }
    }

    pub fn apply_input_arguments(&mut self, args: &Args) {
        self.dt = args.dt;
        self.fight_length = args.fight_length;
        self.iterations = args.iterations;
        self.verb = args.verb;
        self.mh.enemy_lvl = args.enemy_lvl;
        self.oh.enemy_lvl = args.enemy_lvl;
    }

    pub fn configure_with_character(&mut self, character: &Character) {
        self.mh.set_mechanics_from_stats(character);
    }
    
}

#[derive(Debug)]
struct Timers {
    adrenaline_rush: f32,
    adrenaline_rush_cd: f32,
    blade_flurry: f32,
    blade_flurry_cd: f32,
    snd: f32,
    thistle_tea_cd: f32
}

impl Timers {
    fn new() -> Timers {
        Timers {
            adrenaline_rush: 0.0,
            adrenaline_rush_cd: 0.0,
            blade_flurry: 0.0,
            blade_flurry_cd: 0.0,
            snd: 0.0,
            thistle_tea_cd: 0.0
        }
    }
}

#[derive(Debug)]
struct MhSimulator {
    mean_dmg: f32,
    swing_speed: f32,
    hit_table_yellow: YellowHitTable,
    hit_table_backstab: YellowHitTable,
    hit_table_white: WhiteHitTable,
    hit_proccs: Vec<HitProcc>,
    enemy_lvl: i32
}

impl MhSimulator {
    fn new() -> MhSimulator {
        MhSimulator {
            mean_dmg: 0.0,
            swing_speed: 0.0,
            hit_table_yellow: YellowHitTable::new(),
            hit_table_backstab: YellowHitTable::new(),
            hit_table_white: WhiteHitTable::new(),
            hit_proccs: Vec::new(),
            enemy_lvl: 0
        }
    }

    fn set_mechanics_from_stats(&mut self, character: &Character) {
        self.mean_dmg = character.mh.get_mean_dmg();
        self.swing_speed = character.mh.get_swing_speed();
        self.set_hit_tables(character);
    }

    fn set_hit_tables(&mut self, character: &Character) {
        self.set_yellow_hit_tables(character);
        self.set_white_hit_table(character);
    }

    fn set_yellow_hit_tables(&mut self, character: &Character) {
        if self.enemy_lvl == 60 {
            panic!("Simulator object must have enemy lvl before \
                   creating hit tables.");
        }

        let skill_delta: i32;
        if character.mh.get_weapon_type() == WeaponType::Dagger {
            skill_delta = 5 * self.enemy_lvl - character.prim_stats.dagger_skill;
        } else if character.mh.get_weapon_type() == WeaponType::Sword {
            skill_delta = 5 * self.enemy_lvl - character.prim_stats.sword_skill;
        } else { panic!("Weapon type not implemented!"); }

        // miss chance
        let mut miss_chance = get_miss_from_level_delta(skill_delta);
        miss_chance = subtract_hit_from_miss(character.sec_stats.hit, 
                                                  miss_chance);
        self.hit_table_yellow.miss_value = miss_chance;
        self.hit_table_backstab.miss_value = miss_chance;

        // dodge chance
        let dodge_value = miss_chance + 0.05 + 0.001 * (skill_delta) as f32;
        self.hit_table_yellow.dodge_value = dodge_value;
        self.hit_table_backstab.dodge_value = dodge_value;

        // crit chance
        let mut crit_value = dodge_value + character.sec_stats.crit;
        crit_value -= 0.01 * (self.enemy_lvl - 60) as f32;
        if self.enemy_lvl == 63 { crit_value -= 0.018; }
        self.hit_table_yellow.crit_value = crit_value;
        crit_value += 0.1 * character.talents.improved_backstab as f32;
        self.hit_table_backstab.crit_value = crit_value;
    }

    fn set_white_hit_table(&mut self, character: &Character) {
        if self.enemy_lvl == 60 {
            panic!("Simulator object must have enemy lvl before \
                   creating hit tables.");
        }

        let skill_delta: i32;
        if character.oh.get_weapon_type() == WeaponType::Dagger {
            skill_delta = 5 * self.enemy_lvl - character.prim_stats.dagger_skill;
        } else if character.oh.get_weapon_type() == WeaponType::Sword {
            skill_delta = 5 * self.enemy_lvl - character.prim_stats.sword_skill;
        } else { panic!("Weapon type not implemented!"); }

        // miss chance
        let mut miss_chance = get_miss_from_level_delta(skill_delta);
        miss_chance = 0.8 * miss_chance + 0.2;
        miss_chance = subtract_hit_from_miss(character.sec_stats.hit, 
                                                  miss_chance);
        self.hit_table_white.miss_value = miss_chance;

        // dodge chance
        let dodge_value = miss_chance + 0.05 + 0.001 * (skill_delta) as f32;
        self.hit_table_white.dodge_value = dodge_value;

        // crit chance
        let mut crit_value = dodge_value + character.sec_stats.crit;
        crit_value -= 0.01 * (self.enemy_lvl - 60) as f32;
        if self.enemy_lvl == 63 { crit_value -= 0.018; }
        self.hit_table_white.crit_value = crit_value;
        
        // glancing chance
        if self.enemy_lvl >= 60 && self.enemy_lvl <= 63 { 
            self.hit_table_white.glancing_value = 
                0.1 + 0.1 * (self.enemy_lvl - 60) as f32; 
        } else { panic!("No reliable glancing numbers outside 60-63"); }
    }
}

fn get_miss_from_level_delta(delta: i32) -> f32 {
    if delta < 0 { return 0.05; }
    else if delta <= 10 && delta >= 0 { return 0.05 + 0.001 * delta as f32; }
    else if delta <= 15 { return 0.07 + 0.002 * ((delta - 10) as f32); }
    else { panic!("Level difference not implemented"); }
}

fn subtract_hit_from_miss(hit: f32, miss: f32) -> f32 {
    if miss > 0.06 { return max_f32(0.0, miss - hit + 0.01); }
    else { return max_f32(0.0, miss - hit); }
}

#[derive(Debug)]
struct YellowHitTable {
    // a random number is rolled, the first of the below entries that exceeds 
    // that number determines the hit type
    miss_value: f32,
    dodge_value: f32,
    crit_value: f32
}

impl YellowHitTable {
    fn new() -> YellowHitTable {
        YellowHitTable {
            miss_value: 0.0,
            dodge_value: 0.0,
            crit_value: 0.0
        }
    }
}

#[derive(Debug)]
struct WhiteHitTable {
    miss_value: f32,
    dodge_value: f32,
    glancing_value: f32,
    crit_value: f32
}

impl WhiteHitTable {
    fn new() -> WhiteHitTable {
        WhiteHitTable {
            miss_value: 0.0,
            dodge_value: 0.0,
            glancing_value: 0.0,
            crit_value: 0.0
        }
    }
}

#[derive(Debug)]
enum HitProcc {
    Dmg(DmgProcc), // dmg caused todo: how to deal with dmg reductions? (armor,
                    // resistance)
    Dot(DotProcc),
    Strength(StrProcc),
    ExtraAttackProcc(ExtraAttackProcc),
    None,
}

#[derive(Debug)]
struct DmgProcc {
    procc_chance: f32
}

#[derive(Debug)]
struct DotProcc {
    procc_chance: f32,
    time_between_ticks: f32,
    ticks: f32,
    dmg: f32
}

#[derive(Debug)]
struct StrProcc {
    procc_chance: f32,
    duration: f32,
    strength: i32
}

#[derive(Debug)]
struct ExtraAttackProcc {
    procc_chance: f32
}

#[derive(Debug)]
struct OhSimulator {
    mean_dmg: f32,
    swing_speed: f32,
    hit_table: WhiteHitTable,
    hit_proccs: Vec<HitProcc>,
    enemy_lvl: i32
}

impl OhSimulator {
    fn new() -> OhSimulator {
        OhSimulator {
            mean_dmg: 0.0,
            swing_speed: 0.0,
            hit_table: WhiteHitTable::new(),
            hit_proccs: Vec::new(),
            enemy_lvl: 0
        }
    }
}

#[derive(Debug)]
struct DmgModifiers {
    general_modifiers: GeneralModifiers,
    crit_modifiers: CritModifiers
}

impl DmgModifiers {
    fn new() -> DmgModifiers {
        DmgModifiers {
            general_modifiers: GeneralModifiers::new(),
            crit_modifiers: CritModifiers::new()
        }
    }
}

#[derive(Debug)]
struct GeneralModifiers {
    glancing: f32,
    sinister_strike: f32,
    backstab: f32,
    eviscerate: f32,
    oh: f32
}

impl GeneralModifiers {
    fn new() -> GeneralModifiers {
        GeneralModifiers {
            glancing: 0.0,
            sinister_strike: 0.0,
            backstab: 0.0,
            eviscerate: 0.0,
            oh: 0.0
        }
    }
}

#[derive(Debug)]
struct CritModifiers {
    sinister_strike: f32,
    backstab: f32,
    eviscerate: f32
}

impl CritModifiers {
    fn new() -> CritModifiers {
        CritModifiers {
            sinister_strike: 0.0,
            backstab: 0.0,
            eviscerate: 0.0,
        }
    }
}

#[derive(Debug)]
enum Rotation {
    SinStrikeEvis,
    BackstabEvis,
    None,
}
