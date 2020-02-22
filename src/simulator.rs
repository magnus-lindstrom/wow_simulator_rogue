use std::fmt::Display;
use crate::utils::{Args,deb,max_f32,min_i32,max_i32,roll_die};
use crate::armory::{Character,Weapon,WeaponType};
use crate::stats::CurrentStats;


#[derive(Debug)]
pub struct Simulator {
    timekeep: TimeKeeper,
    fight_length: f32,
    mh: WepSimulator, 
    oh: WepSimulator,
    rotation: Rotation,
    ability_costs: AbilityCosts,
    modifiers: Modifiers,
    stats: CurrentStats,
    extra_attacks: i32,
    energy: i32,
    combo_points: i32,
    verb: i32
}

impl Simulator {
    pub fn new() -> Simulator {
        Simulator {
            timekeep: TimeKeeper::new(),
            fight_length: 0.0,
            mh: WepSimulator::new(),
            oh: WepSimulator::new(),
            rotation: Rotation::None,
            ability_costs: AbilityCosts::new(),
            modifiers: Modifiers::new(),
            stats: CurrentStats::new(),
            extra_attacks: 0,
            energy: 0,
            combo_points: 0,
            verb: 0
        }
    }

    pub fn apply_input_arguments(&mut self, args: &Args) {
        self.timekeep.dt = args.dt;
        self.timekeep.timers.time_left = args.fight_length;
        self.fight_length = args.fight_length;
        self.stats.set_fight_length(args.fight_length);
        self.verb = args.verb;
        self.timekeep.verb = args.verb;
        self.mh.enemy_lvl = args.enemy_lvl;
        self.oh.enemy_lvl = args.enemy_lvl;
    }

    pub fn configure_with_character(&mut self, character: &Character) {
        self.timekeep.set_mh_swing_interval(&character.mh);
        self.timekeep.set_oh_swing_interval(&character.oh);

        self.mh.set_weapon_type_and_normalized_speed(&character.mh);
        self.mh.set_main_hand();
        self.mh.set_mechanics_from_character(character);

        self.oh.set_weapon_type_and_normalized_speed(&character.oh);
        self.oh.set_off_hand();
        self.oh.set_mechanics_from_character(character);

        self.incorporate_talents(character);

        self.set_rotation();
    }

    fn incorporate_talents(&mut self, character: &Character) {

        // assassination table
        self.modifiers.hit.eviscerate += 0.05 * 
            character.talents.improved_eviscerate as f32;
       
        self.mh.add_crit(0.01 * character.talents.malice as f32);
        self.oh.add_crit(0.01 * character.talents.malice as f32);

        // combat table
        self.mh.hit_table_backstab.add_crit(
            0.1 * character.talents.improved_backstab as f32);
    }


    fn set_rotation(&mut self) {
        if self.mh.weapon_type == WeaponType::Dagger {
            self.rotation = Rotation::BackstabEvis;
        } else {
            self.rotation = Rotation::SinStrikeEvis;
        }
    }

    fn perform_apt_yellow_ability(&mut self) {
        if self.timekeep.timers.global_cd > 0.0 { return; }
        if self.rotation == Rotation::BackstabEvis {
            self.backstab_evis_rotation();
        } else if self.rotation == Rotation::SinStrikeEvis {
            self.sin_strike_evis_rotation();
        }
    }

    fn sin_strike_evis_rotation(&mut self) {
    }

    fn show_slice_and_dice(&self) {
        let msg = format!("{:.1}: Slice and dice applied for {:.1}s", 
                          self.timekeep.timers.time_left,
                          self.timekeep.timers.slice_and_dice);
        println!("{}", msg);
    }

    fn subtract_energy(&mut self, energy: i32) {
        self.energy = max_i32(0, self.energy - energy);
    }

    fn eviscerate(&mut self) {
        let hit: Hit = self.mh.hit_table_yellow.roll_for_hit();
        let mut dmg = 0.0;

        self.energy -= self.ability_costs.eviscerate;
        self.start_global_cd();

        if self.combo_points == 1 { dmg = 247.0; }
        else if self.combo_points == 2 { dmg = 398.0; }
        else if self.combo_points == 3 { dmg = 549.0; }
        else if self.combo_points == 4 { dmg = 700.0; }
        else if self.combo_points == 5 { dmg = 851.0; }
        else { panic!("Can only eviscerate with 1-5 combo points."); }

        if hit == Hit::Hit || hit == Hit::Crit { 
            self.clear_combo_points();

            dmg *= self.modifiers.hit.eviscerate;

            if hit == Hit::Crit {
                dmg += dmg * self.modifiers.crit.eviscerate;
            }
        }

        self.stats.record_eviscerate_dmg_and_hit(dmg, &hit);

        if self.verb > 0 {
            let msg = format!("{:.1}: Eviscerate {} for {:.0} dmg.", 
                              self.timekeep.timers.time_left, hit, dmg);
            println!("{}", msg);
        }
    }

    fn slice_and_dice(&mut self) {
        let mut dur: f32;
        if self.combo_points == 1 { dur = 9.0; }
        else if self.combo_points == 2 { dur = 12.0; }
        else if self.combo_points == 3 { dur = 15.0; }
        else if self.combo_points == 4 { dur = 18.0; }
        else if self.combo_points == 5 { dur = 21.0; }
        else if self.combo_points == 0 { dur = 0.0; }
        else { panic!("Can only have 0-5 combo points."); }

        dur *= self.modifiers.general.slice_and_dice_duration;
        self.timekeep.timers.slice_and_dice = dur;
        self.start_global_cd();
        self.subtract_energy(self.ability_costs.slice_and_dice);
        self.combo_points = 0;
        if self.verb > 0 { self.show_slice_and_dice() } 
    }

    fn clear_combo_points(&mut self) {
        self.combo_points = 0;
    }

    fn add_combo_point(&mut self) {
        self.combo_points = min_i32(5, self.combo_points + 1);
    }

    fn backstab(&mut self) {
        let hit: Hit = self.mh.hit_table_backstab.roll_for_hit();
        let mut dmg = 0.0;
        if hit == Hit::Miss || hit == Hit::Dodge { 
            self.energy -= (0.2 * self.ability_costs.backstab as f32) as i32;
        }
        if hit == Hit::Hit || hit == Hit::Crit { 
            self.energy -= self.ability_costs.backstab;
            self.add_combo_point();
            dmg = 1.5 * self.mh.mean_dmg + 210.0;
            dmg *= self.modifiers.hit.backstab;

            if hit == Hit::Crit {
                dmg += dmg * self.modifiers.crit.backstab;
            }
        }
        self.stats.record_backstab_dmg_and_hit(dmg, &hit);
        self.start_global_cd();

        if self.verb > 0 {
            let msg = format!("{:.1}: Backstab {} for {:.0} dmg.", 
                              self.timekeep.timers.time_left, hit, dmg);
            println!("{}", msg);
        }

    }

    fn start_global_cd(&mut self) {
        self.timekeep.timers.global_cd = 1.0;
    }

    fn backstab_evis_rotation(&mut self) {
        let can_backstab = self.energy >= self.ability_costs.backstab;
        let can_eviscerate = self.energy >= self.ability_costs.eviscerate;
        let can_slice_and_dice = 
            self.energy >= self.ability_costs.slice_and_dice;
        let active_slice_and_dice = self.timekeep.timers.slice_and_dice > 0.0;

        if self.combo_points == 2 && ! active_slice_and_dice 
            && can_slice_and_dice { self.slice_and_dice() }
        else if self.combo_points < 5 && can_backstab { self.backstab(); }
        else if self.combo_points == 5 && ! active_slice_and_dice 
            && can_slice_and_dice { self.slice_and_dice(); }
        else if self.combo_points == 5 && active_slice_and_dice 
            && can_eviscerate { self.eviscerate(); }
    }

    fn check_mh_swing_timer_and_strike(&mut self) {

        if self.timekeep.timers.mh_swing > 0.0 { return; }
        else { self.timekeep.reset_mh_swing_timer(); }

        let hit: Hit = self.mh.hit_table_white.roll_for_hit();
        let mut dmg = 0.0;
        if hit == Hit::Hit || hit == Hit::Crit || hit == Hit::Glancing { 
            dmg = self.mh.mean_dmg;

            if hit == Hit::Glancing {
                dmg *= self.modifiers.hit.glancing;
            } else if hit == Hit::Crit {
                dmg *= 2.0;
            }
        }

        self.stats.record_mh_white_dmg_and_hit(dmg, &hit);

        let msg = format!("{:.1}: MH {} for {:.0} dmg.", 
                          self.timekeep.timers.time_left, hit, dmg);
        println!("{}", msg);
    }

    fn check_oh_swing_timer_and_strike(&mut self) {

        if self.timekeep.timers.oh_swing > 0.0 { return; }
        else { self.timekeep.reset_oh_swing_timer(); }

        let hit: Hit = self.oh.hit_table_white.roll_for_hit();
        let mut dmg = 0.0;
        if hit == Hit::Hit || hit == Hit::Crit || hit == Hit::Glancing { 
            dmg = self.oh.mean_dmg;
            dmg *= self.modifiers.hit.oh;

            if hit == Hit::Glancing {
                dmg *= self.modifiers.hit.glancing;
            } else if hit == Hit::Crit {
                dmg *= 2.0;
            }
        }

        self.stats.record_oh_white_dmg_and_hit(dmg, &hit);

        let msg = format!("{:.1}: OH {} for {:.0} dmg.", 
                          self.timekeep.timers.time_left, hit, dmg);
        println!("{}", msg);
    }

    pub fn print_stats(&mut self) {
        self.stats.print_stats();
    }

    pub fn simulate(&mut self) {
        self.stats.clear();

        while self.timekeep.timers.time_left > 0.0 {
            self.perform_apt_yellow_ability();
            self.check_oh_swing_timer_and_strike();
            self.check_mh_swing_timer_and_strike();

            self.timekeep.take_time_step();
            self.check_energy_timer_and_refill_energy();
        }
    }

    fn check_energy_timer_and_refill_energy(&mut self) {

        if self.timekeep.timers.energy_refill <= 0.0 {
            self.timekeep.reset_energy_timer();
            self.refill_energy();
        }
    }

    fn refill_energy(&mut self) {
        let refill: i32;
        let die = roll_die();
        if die < 0.25 { refill = 21; }
        else { refill = 20; }
        self.energy = min_i32(self.modifiers.general.energy_max, 
                              self.energy + refill);
        if self.verb > 1 { self.show_energy_refill(); }
    }

    fn show_energy_refill(&mut self) {
        let msg = format!("{:.1}: Energy refilled to {}.", 
                          self.timekeep.timers.time_left,
                          self.energy);
        println!("{}", msg);
    }
}

#[derive(Debug)]
struct AbilityCosts {
    sinister_strike: i32,
    backstab: i32,
    eviscerate: i32,
    slice_and_dice: i32,
    blade_flurry: i32
}

impl AbilityCosts {
    fn new() -> AbilityCosts {
        AbilityCosts {
            sinister_strike: 45,
            backstab: 60,
            eviscerate: 35,
            slice_and_dice: 25,
            blade_flurry: 25
        }
    }
}

#[derive(Debug)]
struct Timers {
    adrenaline_rush: f32,
    adrenaline_rush_cd: f32,
    energy_refill: f32,
    blade_flurry: f32,
    blade_flurry_cd: f32,
    slice_and_dice: f32,
    thistle_tea_cd: f32,
    time_left: f32,
    global_cd: f32,
    mh_swing: f32,
    oh_swing: f32
}

impl Timers {
    fn new() -> Timers {
        Timers {
            adrenaline_rush: 0.0,
            adrenaline_rush_cd: 0.0,
            energy_refill: 0.0,
            blade_flurry: 0.0,
            blade_flurry_cd: 0.0,
            slice_and_dice: 0.0,
            thistle_tea_cd: 0.0,
            time_left: 0.0,
            global_cd: 0.0,
            mh_swing: 0.0,
            oh_swing: 0.0
        }
    }
}


#[derive(Debug)]
struct TimeKeeper {
    timers: Timers,
    dt: f32,
    mh_swing_interval: f32,
    oh_swing_interval: f32,
    verb: i32
}

impl TimeKeeper {
    fn new() -> TimeKeeper {
        TimeKeeper {
            timers: Timers::new(),
            dt: 0.0,
            mh_swing_interval: 0.0,
            oh_swing_interval: 0.0,
            verb: 0
        }
    }

    fn set_mh_swing_interval(&mut self, weapon: &Weapon) {
        self.mh_swing_interval = weapon.get_swing_interval();
    }

    fn set_oh_swing_interval(&mut self, weapon: &Weapon) {
        self.oh_swing_interval = weapon.get_swing_interval();
    }

    fn reset_mh_swing_timer(&mut self) {
        self.timers.mh_swing = self.mh_swing_interval;
        if self.verb > 1 {
            let msg = format!("{:.1}: Reset MH swing timer to {:.0}s.", 
                              self.timers.time_left, self.timers.mh_swing);
            println!("{}", msg);
        }
    }

    fn reset_oh_swing_timer(&mut self) {
        self.timers.oh_swing = self.oh_swing_interval;
        if self.verb > 1 {
            let msg = format!("{:.1}: Reset OH swing timer to {:.0}s.", 
                              self.timers.time_left, self.timers.oh_swing);
            println!("{}", msg);
        }
    }

    fn reset_energy_timer(&mut self) { self.timers.energy_refill = 2.0; }

    fn take_time_step(&mut self) {

        if self.timers.adrenaline_rush > 0.0 { 
            self.timers.adrenaline_rush -= self.dt; 
        }
        if self.timers.adrenaline_rush_cd > 0.0 { 
            self.timers.adrenaline_rush_cd -= self.dt; 
        }
        if self.timers.energy_refill > 0.0 { 
            self.timers.energy_refill -= self.dt; 
        }
        if self.timers.blade_flurry > 0.0 { 
            self.timers.blade_flurry -= self.dt; 
        }
        if self.timers.blade_flurry_cd > 0.0 { 
            self.timers.blade_flurry_cd -= self.dt; 
        }
        if self.timers.slice_and_dice > 0.0 { 
            self.timers.slice_and_dice -= self.dt; 
        }
        if self.timers.thistle_tea_cd > 0.0 { 
            self.timers.thistle_tea_cd -= self.dt; 
        }
        if self.timers.time_left > 0.0 { 
            self.timers.time_left -= self.dt; 
        }
        if self.timers.global_cd > 0.0 { 
            self.timers.global_cd -= self.dt; 
        }
        if self.timers.mh_swing > 0.0 { 
            self.timers.mh_swing -= self.dt; 
        }
        if self.timers.oh_swing > 0.0 { 
            self.timers.oh_swing -= self.dt; 
        }
    }
}

#[derive(Debug)]
struct WepSimulator {
    weapon_type: WeaponType,
    mean_dmg: f32,
    normalized_speed: f32,
    hit_table_yellow: YellowHitTable,
    hit_table_backstab: YellowHitTable,
    hit_table_white: WhiteHitTable,
    hit_proccs: Vec<HitProcc>,
    enemy_lvl: i32,
    weapon_slot: WeaponSlot
}

impl WepSimulator {
    fn new() -> WepSimulator {
        WepSimulator {
            weapon_type: WeaponType::None,
            mean_dmg: 0.0,
            normalized_speed: 0.0,
            hit_table_yellow: YellowHitTable::new(),
            hit_table_backstab: YellowHitTable::new(),
            hit_table_white: WhiteHitTable::new(),
            hit_proccs: Vec::new(),
            enemy_lvl: 0,
            weapon_slot: WeaponSlot::None
        }
    }

    fn set_weapon_type_and_normalized_speed(&mut self, weapon: &Weapon) {
        self.weapon_type = weapon.get_weapon_type();
        self.set_normalized_speed();
    }

    fn set_normalized_speed(&mut self) {
        if self.weapon_type == WeaponType::Dagger { self.normalized_speed = 1.7; }
        else if self.weapon_type == WeaponType::Sword { 
            self.normalized_speed = 2.4; 
        }
        else { panic!("Weapon type not yet implemented."); }
    }

    fn set_main_hand(&mut self) { self.weapon_slot = WeaponSlot::Mh; }

    fn set_off_hand(&mut self) { self.weapon_slot = WeaponSlot::Oh; }

    fn is_main_hand(&self) -> bool { 
        if self.weapon_slot == WeaponSlot::Mh { return true; }
        else if self.weapon_slot == WeaponSlot::Oh { return false; }
        else { panic!("Weapon type not initialized yet."); }
    }

    fn is_off_hand(&self) -> bool { 
        if self.weapon_slot == WeaponSlot::Oh { return true; }
        else if self.weapon_slot == WeaponSlot::Mh { return false; }
        else { panic!("Weapon type not initialized yet."); }
    }

    fn set_mechanics_from_character(&mut self, character: &Character) {
        self.set_wep_dmg(character);
        self.set_hit_tables(character);
    }

    fn set_wep_dmg(&mut self, character: &Character) {

        if self.is_off_hand() {
            self.mean_dmg = character.oh.get_mean_dmg();
        } else {
            self.mean_dmg = character.mh.get_mean_dmg();
        }
        self.mean_dmg += self.normalized_speed 
                       * character.sec_stats.attack_power as f32
                       / 14.0;

    }

    fn set_hit_tables(&mut self, character: &Character) {
        if self.is_main_hand() { 
            self.set_yellow_hit_table(character);
            if self.weapon_type == WeaponType::Dagger {
                self.set_backstab_hit_table(character); 
            }
        }
        self.set_white_hit_table(character);
    }

    fn set_yellow_hit_table(&mut self, character: &Character) {
        if self.enemy_lvl == 0 {
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

        // dodge chance
        let dodge_value = miss_chance + 0.05 + 0.001 * (skill_delta) as f32;
        self.hit_table_yellow.dodge_value = dodge_value;

        // crit chance
        let mut crit_value = dodge_value + character.sec_stats.crit;
        crit_value -= 0.01 * (self.enemy_lvl - 60) as f32;
        if self.enemy_lvl == 63 { crit_value -= 0.018; }
        self.hit_table_yellow.crit_value = crit_value;
    }
    
    fn set_backstab_hit_table(&mut self, character: &Character) {
        self.hit_table_backstab = self.hit_table_yellow.clone();
    }

    fn set_white_hit_table(&mut self, character: &Character) {
        if self.enemy_lvl == 0 {
            panic!("Simulator object must have enemy lvl before \
                   creating hit tables.");
        }

        let skill_delta: i32;
        if self.is_off_hand() {
            if character.oh.get_weapon_type() == WeaponType::Dagger {
                skill_delta = 5 * self.enemy_lvl 
                    - character.prim_stats.dagger_skill;
            } else if character.oh.get_weapon_type() == WeaponType::Sword {
                skill_delta = 5 * self.enemy_lvl 
                    - character.prim_stats.sword_skill;
            } else { panic!("Weapon type not implemented!"); }
        } else {
            if character.mh.get_weapon_type() == WeaponType::Dagger {
                skill_delta = 5 * self.enemy_lvl 
                    - character.prim_stats.dagger_skill;
            } else if character.mh.get_weapon_type() == WeaponType::Sword {
                skill_delta = 5 * self.enemy_lvl 
                    - character.prim_stats.sword_skill;
            } else { panic!("Weapon type not implemented!"); }
        }

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

    fn add_crit(&mut self, crit: f32) {
        self.hit_table_white.add_crit(crit);
        if self.is_main_hand() { 
            self.hit_table_yellow.add_crit(crit); 
            if self.weapon_type == WeaponType::Dagger {
                self.hit_table_backstab.add_crit(crit);
            }
        }
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

#[derive(Debug,Clone)]
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

    fn roll_for_hit(&self) -> Hit {
        let die = roll_die();
        if die < self.miss_value { return Hit::Miss; }
        else if die < self.dodge_value { return Hit::Dodge; }
        else if die < self.crit_value { return Hit::Crit; }
        else { return Hit::Hit; }
    }

    fn add_crit(&mut self, crit: f32) {
        self.crit_value += crit;
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

    fn roll_for_hit(&self) -> Hit {
        let die = roll_die();
        if die < self.miss_value { return Hit::Miss; }
        else if die < self.dodge_value { return Hit::Dodge; }
        else if die < self.glancing_value { return Hit::Glancing; }
        else if die < self.crit_value { return Hit::Crit; }
        else { return Hit::Hit; }
    }

    fn add_crit(&mut self, crit: f32) {
        self.crit_value += crit;
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
struct Modifiers {
    general: GeneralModifiers,
    hit: HitModifiers,
    crit: CritModifiers,
    finisher: FinisherModifiers
}

impl Modifiers {
    fn new() -> Modifiers {
        Modifiers {
            general: GeneralModifiers::new(),
            hit: HitModifiers::new(),
            crit: CritModifiers::new(),
            finisher: FinisherModifiers::new()
        }
    }
}

#[derive(Debug)]
struct GeneralModifiers {
    slice_and_dice_duration: f32,
    energy_max: i32
}

impl GeneralModifiers {
    fn new() -> GeneralModifiers {
        GeneralModifiers {
            slice_and_dice_duration: 1.0,
            energy_max: 100
        }
    }
}

#[derive(Debug)]
struct HitModifiers {
    glancing: f32,
    sinister_strike: f32,
    backstab: f32,
    eviscerate: f32,
    oh: f32
}

impl HitModifiers {
    fn new() -> HitModifiers {
        HitModifiers {
            glancing: 1.0,
            sinister_strike: 1.0,
            backstab: 1.0,
            eviscerate: 1.0,
            oh: 0.5
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
            sinister_strike: 1.0,
            backstab: 1.0,
            eviscerate: 1.0,
        }
    }
}

#[derive(Debug)]
struct FinisherModifiers {
    restore_energy_chance: f32,
    add_combo_point_chance: f32
}

impl FinisherModifiers {
    fn new() -> FinisherModifiers {
        FinisherModifiers {
            restore_energy_chance: 0.0,
            add_combo_point_chance: 0.0
        }
    }
}

#[derive(Debug,PartialEq)]
enum Rotation {
    SinStrikeEvis,
    BackstabEvis,
    None,
}

#[derive(Display,PartialEq)]
pub enum Hit {
    Hit, Crit, Miss, Glancing, Dodge
}


#[derive(Clone,Copy,Debug,PartialEq)]
pub enum WeaponSlot {
    Mh,
    Oh,
    None
}
