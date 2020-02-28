use std::fmt::Display;
use crate::utils::{Args,deb,min_f32,max_f32,min_i32,max_i32,roll_die};
use crate::armory::{Character,Cooldown,HitProcc,Weapon,WeaponType};
use crate::armory::CooldownEffect::{EnergyRegenMultiplier,AttackSpeedMultiplier,
InstantEnergyRefill};
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
    cooldowns: Vec<Cooldown>,
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
            cooldowns: Vec::new(),
            stats: CurrentStats::new(),
            extra_attacks: 0,
            energy: 0,
            combo_points: 0,
            verb: 0
        }
    }

    pub fn apply_input_arguments(&mut self, args: &Args) {
        self.timekeep.dt = args.dt;
        self.timekeep.fight_length = args.fight_length;
        self.fight_length = args.fight_length;
        self.stats.set_fight_length(args.fight_length);
        self.verb = args.verb;
        self.timekeep.verb = args.verb;
        self.mh.enemy_lvl = args.enemy_lvl;
        self.oh.enemy_lvl = args.enemy_lvl;
    }

    pub fn get_stats(&self) -> CurrentStats {
        return self.stats.copy();
    }
    
    fn set_glancing_reduction(&mut self, character: &Character) {
        // Main hand
        let mut skill_delta: i32;
        if self.mh.get_weapon_type() == WeaponType::Dagger {
            skill_delta = 5 * self.mh.enemy_lvl 
                - character.prim_stats.dagger_skill;
        } else if character.mh.get_weapon_type() == WeaponType::Sword {
            skill_delta = 5 * self.mh.enemy_lvl 
                - character.prim_stats.sword_skill;
        } else { panic!("Weapon type not initialized or implemented yet!"); }

        // weapon expertise
        if self.mh.get_weapon_type() == WeaponType::Dagger 
            || self.mh.get_weapon_type() == WeaponType::Sword {
                skill_delta -= match character.talents.weapon_expertise {
                    0 => 0,
                    1 => 3,
                    2 => 5,
                    _ => panic!("Invalid value of weapon expertise.")
                };
        }

        self.modifiers.hit.glancing_mh = match skill_delta {
            15 => 1.0 - 0.35,
            14 => 1.0 - 0.31,
            13 => 1.0 - 0.27,
            12 => 1.0 - 0.23,
            11 => 1.0 - 0.19,
            10 => 1.0 - 0.15,
            9  => 1.0 - 0.11,
            8  => 1.0 - 0.07,
            -300..=7 =>  1.0 - 0.05,
            _ => panic!("Skill delta not implemented")
        };

        // Off hand
        let mut skill_delta: i32;
        if self.oh.get_weapon_type() == WeaponType::Dagger {
            skill_delta = 5 * self.oh.enemy_lvl 
                - character.prim_stats.dagger_skill;
        } else if character.oh.get_weapon_type() == WeaponType::Sword {
            skill_delta = 5 * self.oh.enemy_lvl 
                - character.prim_stats.sword_skill;
        } else { panic!("Weapon type not initialized yet!"); }

        // weapon expertise
        if self.oh.get_weapon_type() == WeaponType::Dagger 
            || self.oh.get_weapon_type() == WeaponType::Sword {
                skill_delta -= match character.talents.weapon_expertise {
                    0 => 0,
                    1 => 3,
                    2 => 5,
                    _ => panic!("Invalid value of weapon expertise.")
                };
        }

        self.modifiers.hit.glancing_oh = match skill_delta {
            15 => 1.0 - 0.35,
            14 => 1.0 - 0.31,
            13 => 1.0 - 0.27,
            12 => 1.0 - 0.23,
            11 => 1.0 - 0.19,
            10 => 1.0 - 0.15,
            9  => 1.0 - 0.11,
            8  => 1.0 - 0.07,
            -300..=7 =>  1.0 - 0.05,
            _ => panic!("Skill delta not implemented")
        };
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

        self.modifiers.set_modifiers(character);

        self.declare_proccs();
        self.set_cooldowns(character);
        self.set_glancing_reduction(character);
        self.incorporate_talents(character);
        self.set_rotation();
    }

    fn set_cooldowns(&mut self, character: &Character) {
        self.cooldowns = character.cooldowns.clone();
    }

    fn declare_proccs(&mut self) {
        self.stats.declare_proccs(&self.mh.hit_proccs);
        self.stats.declare_proccs(&self.oh.hit_proccs);
    }

    fn incorporate_talents(&mut self, character: &Character) {

        // assassination table
        // imp evis
        self.modifiers.hit.eviscerate += 0.05 * 
            character.talents.improved_eviscerate as f32;
       
        // malice
        self.mh.add_crit(0.01 * character.talents.malice as f32);
        self.oh.add_crit(0.01 * character.talents.malice as f32);

        // relentless strikes
        match character.talents.relentless_strikes {
            1 => self.modifiers.finisher.restore_energy_chance_per_combo_point = 
                0.2,
            0 => (),
            _ => panic!("Relentless strikes can only have one talent point")
        }

        // ruthlessness
        self.modifiers.finisher.add_combo_point_chance = 
            0.2 * character.talents.ruthlessness as f32;

        // improved slice and dice
        self.modifiers.general.slice_and_dice_duration_modifier += 
            0.15 * character.talents.improved_slice_and_dice as f32;
        
        // lethality
        self.modifiers.crit.backstab += 
            0.06 * character.talents.lethality as f32;
        self.modifiers.crit.sinister_strike += 
            0.06 * character.talents.lethality as f32;

        // combat table
        // imp sinister strike
        match character.talents.improved_sinister_strike {
            0 => (),
            1 => self.ability_costs.sinister_strike -= 3,
            2 => self.ability_costs.sinister_strike -= 5,
            _ => panic!("Illegal value of improved sinister strike")
        }

        // imp backstab
        self.mh.hit_table_backstab.add_crit(
            0.1 * character.talents.improved_backstab as f32);

        // precision
        self.mh.add_hit(0.01 * character.talents.precision as f32);
        self.oh.add_hit(0.01 * character.talents.precision as f32);

        // dagger specialization
        if self.mh.weapon_type == WeaponType::Dagger {
            self.mh.add_crit(
                0.01 * character.talents.dagger_specialization as f32);
        }
        if self.oh.weapon_type == WeaponType::Dagger {
            self.oh.add_crit(
                0.01 * character.talents.dagger_specialization as f32);
        }
        
        // dual wield specialization
        self.modifiers.hit.oh *= 
            1.0 + 0.1 * character.talents.dual_wield_specialization as f32;
        
        // sword specialization
        if character.talents.sword_specialization > 0 {
            panic!("Sword specialization not implemented yet.");
        }

        // aggression
        self.modifiers.hit.eviscerate *= 
            1.0 + 0.02 * character.talents.aggression as f32;
        self.modifiers.hit.sinister_strike *= 
            1.0 + 0.02 * character.talents.aggression as f32;
        
        // subtlety
        // opportunity
        self.modifiers.hit.backstab *= 
            1.0 + 0.04 * character.talents.opportunity as f32;
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
        if self.verb > 1 && energy > 0 {
            let msg = format!("{:.1}: Energy down to {}.", 
                              self.timekeep.timers.time_left,
                              self.energy);
            println!("{}", msg);
        }
    }

    fn add_energy(&mut self, energy_refill: i32) {
        self.energy = min_i32(self.modifiers.general.energy_max, 
                              self.energy + energy_refill);
    }

    fn eviscerate(&mut self) {
        let hit: Hit = self.mh.hit_table_yellow.roll_for_hit();
        let mut dmg;

        self.subtract_energy(self.ability_costs.eviscerate);
        self.start_global_cd();

        if self.combo_points == 1 { dmg = 247.0; }
        else if self.combo_points == 2 { dmg = 398.0; }
        else if self.combo_points == 3 { dmg = 549.0; }
        else if self.combo_points == 4 { dmg = 700.0; }
        else if self.combo_points == 5 { dmg = 851.0; }
        else { panic!("Can only eviscerate with 1-5 combo points."); }

        if hit == Hit::Hit || hit == Hit::Crit { 
            self.trigger_hit_procc_mh();
            self.clear_combo_points_and_roll_for_finisher_procs();

            dmg *= self.modifiers.hit.eviscerate;

            if hit == Hit::Crit {
                dmg += dmg * self.modifiers.crit.eviscerate;
            }
        }

        dmg = self.modifiers.armor_reduction(dmg);
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

        dur *= self.modifiers.general.slice_and_dice_duration_modifier;
        self.timekeep.timers.slice_and_dice = dur;
        self.start_global_cd();
        self.subtract_energy(self.ability_costs.slice_and_dice);
        self.clear_combo_points_and_roll_for_finisher_procs();
        if self.verb > 0 { self.show_slice_and_dice() } 
    }

    fn clear_combo_points_and_roll_for_finisher_procs(&mut self) {
        let mut new_combo_points = 0;
        if self.modifiers.finisher.gets_extra_combo_point() {
            new_combo_points = 1; 
            if self.verb > 0 { println!("Got extra combo point from finisher!"); }
        }
        if self.modifiers.finisher.gets_extra_energy(self.combo_points) {
            self.add_energy(25);
            if self.verb > 0 { println!("Got 25 energy from finisher!"); }
        }

        self.combo_points = new_combo_points;
    }

    fn add_combo_point(&mut self) {
        self.combo_points = min_i32(5, self.combo_points + 1);
    }

    fn print_procc(&mut self, procc: &HitProcc) {
        let sub_msg = match procc {
            HitProcc::Dmg(name,dmg,_,_) => 
                format!("{} procc for {:.0} dmg!", name, dmg),
            HitProcc::Strength(name,_,_,_) =>
                format!("Strength procc from {}!", name),
            HitProcc::ExtraAttack(name,_) =>
                format!("Extra swing procc from {}!", name),
            HitProcc::None => panic!("'None' proccs not allowed in simulation.")
        };
        let msg = format!("{:.1}: {}", 
                          self.timekeep.timers.time_left, sub_msg);
        println!("{}", msg);
    }

    fn extra_attack_procc(&mut self) {
        self.reset_mh_swing();
        self.add_extra_attack();
    }
    
    fn reset_mh_swing(&mut self) {
        self.timekeep.reset_mh_swing_timer(
            self.modifiers.general.attack_speed_modifier
            );
    }

    fn reset_oh_swing(&mut self) {
        self.timekeep.reset_oh_swing_timer(
            self.modifiers.general.attack_speed_modifier
            );
    }

    fn add_extra_attack(&mut self) {
        self.extra_attacks += 1;
    }

    fn roll_for_procc(&mut self, procc: &HitProcc) {
        let die = roll_die();
        let does_procc = match procc {
            HitProcc::Dmg(_,_,resist_chance,procc_chance) => {
                if die < *procc_chance { 
                    let die_resist = roll_die();
                    if die_resist > *resist_chance {true }
                    else { false }
                } else { false }
            },
            HitProcc::Strength(_,_,_,procc_chance) => {
                if die < *procc_chance { true }
                else { false }
            },
            HitProcc::ExtraAttack(_,procc_chance) => {
                if die < *procc_chance { 
                    self.extra_attack_procc();
                    true 
                }
                else { false }
            },
            HitProcc::None => panic!("'None' proccs not allowed in simulation.")
        };
        if does_procc {
            if self.verb > 0 { self.print_procc(procc); }
            self.stats.record_procc(procc); 
        }
    }

    fn trigger_hit_procc_mh(&mut self) {
        for i in 0..self.mh.hit_proccs.len() {
            let procc = self.mh.hit_proccs[i].clone();
            self.roll_for_procc(&procc);
        }
    }

    fn trigger_hit_procc_oh(&mut self) {
        for i in 0..self.oh.hit_proccs.len() {
            let procc = self.oh.hit_proccs[i].clone();
            self.roll_for_procc(&procc);
        }
    }

    fn backstab(&mut self) {
        let hit: Hit = self.mh.hit_table_backstab.roll_for_hit();
        let mut dmg = 0.0;
        if hit == Hit::Miss || hit == Hit::Dodge { 
            let energy_cost = (0.2 * self.ability_costs.backstab as f32) as i32;
            self.subtract_energy(energy_cost);
        }
        if hit == Hit::Hit || hit == Hit::Crit { 
            self.trigger_hit_procc_mh();
            self.subtract_energy(self.ability_costs.backstab);
            self.add_combo_point();
            dmg = 1.5 * self.mh.mean_dmg + 210.0;
            dmg *= self.modifiers.hit.backstab;

            if hit == Hit::Crit {
                dmg += dmg * self.modifiers.crit.backstab;
            }
        }
        dmg = self.modifiers.armor_reduction(dmg);
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

    fn perform_mh_strike(&mut self) {

        let hit: Hit = self.mh.hit_table_white.roll_for_hit();
        let mut dmg = 0.0;
        if hit == Hit::Hit || hit == Hit::Crit || hit == Hit::Glancing { 
            self.trigger_hit_procc_mh();
            dmg = self.mh.mean_dmg;

            if hit == Hit::Glancing {
                dmg *= self.modifiers.hit.glancing_mh;
            } else if hit == Hit::Crit {
                dmg *= 2.0;
            }
        }
        dmg = self.modifiers.armor_reduction(dmg);
        self.stats.record_mh_white_dmg_and_hit(dmg, &hit);
        self.print_mh_hit_and_dmg(hit, dmg);
    }

    fn print_mh_hit_and_dmg(&mut self, hit: Hit, dmg: f32) {
        if self.verb > 0 {
            let msg = format!("{:.1}: MH {} for {:.0} dmg.", 
                              self.timekeep.timers.time_left, hit, dmg);
            println!("{}", msg);
        }
    }

    fn print_oh_hit_and_dmg(&mut self, hit: Hit, dmg: f32) {
        if self.verb > 0 {
            let msg = format!("{:.1}: OH {} for {:.0} dmg.", 
                              self.timekeep.timers.time_left, hit, dmg);
            println!("{}", msg);
        }
    }

    fn perform_oh_strike(&mut self) {

        let hit: Hit = self.oh.hit_table_white.roll_for_hit();
        let mut dmg = 0.0;
        if hit == Hit::Hit || hit == Hit::Crit || hit == Hit::Glancing { 
            self.trigger_hit_procc_oh();
            dmg = self.oh.mean_dmg;
            dmg *= self.modifiers.hit.oh;

            if hit == Hit::Glancing {
                dmg *= self.modifiers.hit.glancing_oh;
            } else if hit == Hit::Crit {
                dmg *= 2.0;
            }
        }
        dmg = self.modifiers.armor_reduction(dmg);
        self.stats.record_oh_white_dmg_and_hit(dmg, &hit);
        self.print_oh_hit_and_dmg(hit, dmg);
    }

    fn check_mh_swing_timer_and_strike(&mut self) {
        if self.timekeep.timers.mh_swing > 0.0 { return; }
        self.perform_mh_strike();
        self.reset_mh_swing(); 
    }

    fn check_oh_swing_timer_and_strike(&mut self) {
        if self.timekeep.timers.oh_swing > 0.0 { return; }
        self.perform_oh_strike();
        self.reset_oh_swing();
    }

    pub fn print_stats(&mut self) {
        if self.verb > 1 { self.stats.print_stats(); }
    }

    fn cd_lacks_prerequisite(&mut self, cooldown: &Cooldown) -> bool {
        let mut lacks_req = false;
        if cooldown.cd_left > 0.0 { lacks_req = true; }
        else if cooldown.cost > self.energy { lacks_req = true; }
        else if self.energy > cooldown.use_below_energy { lacks_req = true; }
        else if cooldown.cost > 0 && self.timekeep.timers.global_cd > 0.0 {
            lacks_req = true;
        }
        return lacks_req;
    }

    fn use_ready_cooldowns(&mut self) {
        for i in 0..self.cooldowns.len() {
            let temp_cd = self.cooldowns[i].clone();
            if self.cd_lacks_prerequisite(&temp_cd) { continue }
            self.subtract_energy(self.cooldowns[i].cost);
            if self.cooldowns[i].cost > 0 { self.start_global_cd(); }
            match self.cooldowns[i].effect {
                EnergyRegenMultiplier(mult, duration) => {
                    self.modifiers.general.energy_regen_modifier *= mult;
                    self.cooldowns[i].cd_left = self.cooldowns[i].cd;
                    self.cooldowns[i].time_left = duration;
                },
                AttackSpeedMultiplier(mult, duration) => {
                    self.modifiers.general.attack_speed_modifier *= mult;
                    self.cooldowns[i].cd_left = self.cooldowns[i].cd;
                    self.cooldowns[i].time_left = duration;
                },
                InstantEnergyRefill(energy) => {
                    self.add_energy(energy);
                    self.cooldowns[i].cd_left = self.cooldowns[i].cd;
                }
            }
            if self.verb > 1 { self.print_cd_usage(temp_cd); }
        }

    }

    fn print_cd_usage(&self, cd: Cooldown) {
        let sub_msg = match cd.effect {
            EnergyRegenMultiplier(_,_) => {
                format!(", energy regen multiplied by {}!", 
                        self.modifiers.general.energy_regen_modifier)
            },
            AttackSpeedMultiplier(_,_) => {
                format!(", attack speed increased by {}!", 
                        self.modifiers.general.attack_speed_modifier)
            },
            InstantEnergyRefill(energy) => {
                format!(", gaining {} energy!", energy)
            }
        };
        let msg = format!("{:.1}: Used {}{}", self.timekeep.timers.time_left, 
                          cd.name, sub_msg);
        println!("{}", msg);
    }

    fn reset_char(&mut self) {
        self.energy = self.modifiers.general.energy_max;
    }

    pub fn simulate(&mut self) {
        self.stats.clear();
        self.timekeep.reset_timers();
        self.reset_char();

        while self.timekeep.timers.time_left > 0.0 {
            self.use_ready_cooldowns();
            self.perform_apt_yellow_ability();
            self.check_oh_swing_timer_and_strike();
            self.check_mh_swing_timer_and_strike();
            self.do_extra_attacks();

            self.check_energy_timer_and_refill_energy();

            self.take_time_step();

        }
        self.print_at_end_of_simulation();
    }

    fn do_extra_attacks(&mut self) {
        while self.extra_attacks > 0 {
            self.perform_mh_strike();
            self.extra_attacks -= 1;
        }
    }

    fn take_time_step(&mut self) {
        self.timekeep.take_time_step();
        self.check_cooldowns_wearing_off();
    }

    fn check_cooldowns_wearing_off(&mut self) {
        for i in 0..self.cooldowns.len() {
            if self.cooldowns[i].cd_left > 0.0 { 
                self.cooldowns[i].cd_left -= self.timekeep.dt; 
            }
            if self.cooldowns[i].time_left > 0.0 { 
                self.cooldowns[i].time_left -= self.timekeep.dt; 

                if self.cooldowns[i].time_left <= 0.0 {

                    match self.cooldowns[i].effect {
                        EnergyRegenMultiplier(mult,_) => {
                            self.modifiers.general.energy_regen_modifier /= mult;
                        }
                        AttackSpeedMultiplier(mult,_) => {
                            self.modifiers.general.attack_speed_modifier /= mult;
                        }
                        InstantEnergyRefill(_) => ( continue ),
                    }
                    self.print_cd_wearing_off(self.cooldowns[i].clone());
                }

            }
        }
    }


    fn print_cd_wearing_off(&mut self, cd: Cooldown) {
        if self.verb > 1 {
            println!("{:.1}: {} wore off, {}s cooldown.", 
                     self.timekeep.timers.time_left, cd.name, cd.cd);
        }
    }

    fn print_at_end_of_simulation(&mut self) {
        if self.verb > 2 {
            println!("\nSimulator object at the end of simulation:\n{:?}",
                     self);
            self.mh.print_hit_tables();
            self.oh.print_hit_tables();
        }
    }

    fn check_energy_timer_and_refill_energy(&mut self) {

        if self.timekeep.timers.energy_refill <= 0.0 {
            self.timekeep.reset_energy_timer();
            self.refill_energy();
        }
    }

    fn refill_energy(&mut self) {
        let mut refill: i32;
        let die = roll_die();
        if die < 0.25 { refill = 21; }
        else { refill = 20; }
        refill *= self.modifiers.general.energy_regen_modifier;
        self.add_energy(refill);
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
    energy_refill: f32,
    slice_and_dice: f32,
    time_left: f32,
    global_cd: f32,
    mh_swing: f32,
    oh_swing: f32
}

impl Timers {
    fn new() -> Timers {
        Timers {
            energy_refill: 1.0,
            slice_and_dice: 0.0,
            time_left: 0.0,
            global_cd: 0.0,
            mh_swing: 0.0,
            oh_swing: 0.0
        }
    }

    fn reset_with_fight_length(&mut self, fight_length: f32) {
        self.energy_refill = 1.0;
        self.slice_and_dice = 0.0;
        self.time_left = fight_length;
        self.global_cd = 0.0;
        self.mh_swing = 0.0;
        self.oh_swing = 0.0;
    }

}


#[derive(Debug)]
struct TimeKeeper {
    timers: Timers,
    fight_length: f32,
    dt: f32,
    mh_swing_interval: f32,
    oh_swing_interval: f32,
    verb: i32
}

impl TimeKeeper {
    fn new() -> TimeKeeper {
        TimeKeeper {
            timers: Timers::new(),
            fight_length: 0.0,
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

    fn reset_mh_swing_timer(&mut self, factor: f32) {
        self.timers.mh_swing = self.mh_swing_interval / factor;
        if self.verb > 1 {
            let msg = format!("{:.1}: Reset MH swing timer to {:.2}s.", 
                              self.timers.time_left, self.timers.mh_swing);
            println!("{}", msg);
        }
    }

    fn reset_oh_swing_timer(&mut self, factor: f32) {
        self.timers.oh_swing = self.oh_swing_interval / factor;
        if self.verb > 1 {
            let msg = format!("{:.1}: Reset OH swing timer to {:.2}s.", 
                              self.timers.time_left, self.timers.oh_swing);
            println!("{}", msg);
        }
    }

    fn reset_energy_timer(&mut self) { self.timers.energy_refill = 2.0; }

    fn reset_timers(&mut self) {
        self.timers.reset_with_fight_length(self.fight_length);
    }

    fn take_time_step(&mut self) {

        if self.timers.energy_refill > 0.0 { 
            self.timers.energy_refill -= self.dt; 
        }
        if self.timers.slice_and_dice > 0.0 { 
            self.timers.slice_and_dice -= self.dt; 
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

    fn get_weapon_type(&self) -> WeaponType { return self.weapon_type; }

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
        else { panic!("Weapon not initialized yet."); }
    }

    fn set_mechanics_from_character(&mut self, character: &Character) {
        self.set_wep_dmg(character);
        self.set_hit_tables(character);

        self.set_hit_proccs(&character);
        self.apply_enchant_dmg(&character); 
    }

    fn apply_enchant_dmg(&mut self, character: &Character) {
        if self.is_main_hand() {
            for i in 0..character.mh_enchants.len() {
                self.mean_dmg += character.mh_enchants[i].extra_damage;
            }
        } else if self.is_off_hand() {
            for i in 0..character.oh_enchants.len() {
                self.mean_dmg += character.oh_enchants[i].extra_damage;
            }
        } else { panic!("Uninitialized weapon"); }
    }

    fn set_hit_proccs(&mut self, character: &Character) {

        // armor hit proccs go on both weapons
        for i in 0..character.armor.len() {
            if character.armor[i].hit_procc != HitProcc::None {
                self.hit_proccs.push(character.armor[i].hit_procc.clone());
            }
        }

        // weapon enhants only for that weapon
        if self.is_main_hand() {
            if character.mh.get_hit_procc() != HitProcc::None {
                self.hit_proccs.push(character.mh.get_hit_procc());
            }
            for i in 0..character.mh_enchants.len() {
                if character.mh_enchants[i].hit_procc != HitProcc::None {
                    self.hit_proccs.push(
                        character.mh_enchants[i].hit_procc.clone());
                }
            }
        } else {
            if character.oh.get_hit_procc() != HitProcc::None {
                self.hit_proccs.push(character.oh.get_hit_procc());
            }
            for i in 0..character.oh_enchants.len() {
                if character.oh_enchants[i].hit_procc != HitProcc::None {
                    self.hit_proccs.push(
                        character.oh_enchants[i].hit_procc.clone());
                }
            }
        }
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
                self.set_backstab_hit_table(); 
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
        let hit_chance = self.get_effective_hit_chance_from_hit_and_skill_delta(
            character.sec_stats.hit, skill_delta);
        let mut miss_chance = get_miss_from_level_delta(skill_delta);
        miss_chance = miss_chance - hit_chance;
        self.hit_table_yellow.miss_value = miss_chance;

        // dodge chance
        let dodge_chance = 0.05 + 0.001 * (skill_delta) as f32;
        let dodge_value = miss_chance + dodge_chance;
        self.hit_table_yellow.dodge_value = dodge_value;

        // crit chance
        let mut crit_chance = character.sec_stats.crit;
        crit_chance = max_f32( 0.0, 
            crit_chance - 0.01 * (self.enemy_lvl - 60) as f32 );
        if self.enemy_lvl == 63 { 
            crit_chance = max_f32( 0.0, crit_chance - 0.018 );
        }
        let crit_value = dodge_value + crit_chance;
        self.hit_table_yellow.crit_value = crit_value;
    }
    
    fn set_backstab_hit_table(&mut self) {
        self.hit_table_backstab = self.hit_table_yellow.clone();
    }

    fn get_effective_hit_chance_from_hit_and_skill_delta(
        &self, hit: f32, skill_delta: i32) -> f32 {

        let mut hit_chance = hit;
        if skill_delta > 10 { 
            if hit_chance < 0.01 { 
                panic!("A hit application in two parts require that the hit \
                       from items alone is higher than 1% if the skill delta \
                       is greater than 10");
            }
            hit_chance -= 0.01;
        }
        return hit_chance;
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
        let hit_chance = self.get_effective_hit_chance_from_hit_and_skill_delta(
            character.sec_stats.hit, skill_delta);
        let mut miss_chance = get_miss_from_level_delta(skill_delta);
        miss_chance = 0.8 * miss_chance + 0.2;
        miss_chance = miss_chance - hit_chance;
        self.hit_table_white.miss_value = miss_chance;

        // dodge chance
        let dodge_chance = 0.05 + 0.001 * (skill_delta) as f32;
        let dodge_value = miss_chance + dodge_chance;
        self.hit_table_white.dodge_value = dodge_value;

        // glancing chance
        if self.enemy_lvl < 60 || self.enemy_lvl > 63 { 
            panic!("No reliable glancing numbers outside 60-63");
        }
        let glancing_chance = 0.1 + 0.1 * (self.enemy_lvl - 60) as f32; 
        let glancing_value = dodge_value + glancing_chance;
        self.hit_table_white.glancing_value = glancing_value;

        // crit chance
        let mut crit_chance = character.sec_stats.crit;
        crit_chance = max_f32( 0.0, 
            crit_chance - 0.01 * (self.enemy_lvl - 60) as f32 );
        if self.enemy_lvl == 63 { 
            crit_chance = max_f32( 0.0, crit_chance - 0.018 );
        }
        let crit_value = glancing_value + crit_chance;
        self.hit_table_white.crit_value = crit_value;
        
    }

    fn add_hit(&mut self, hit: f32) {
        self.hit_table_white.add_hit(hit);
        if self.is_main_hand() { 
            self.hit_table_yellow.add_hit(hit); 
            if self.weapon_type == WeaponType::Dagger {
                self.hit_table_backstab.add_hit(hit);
            }
        }
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

    fn print_hit_tables(&self) {
        println!("\nHit table for {} white attacks:", self.weapon_slot);
        self.hit_table_white.print_table();
        if self.is_main_hand() {
            println!("\nHit table for {} yellow attacks:", self.weapon_slot);
            self.hit_table_yellow.print_table();
            if self.weapon_type == WeaponType::Dagger {
            println!("\nHit table for {} backstab:", self.weapon_slot);
                self.hit_table_backstab.print_table();
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

    fn add_hit(&mut self, hit: f32) {
        let hit_to_subtract = min_f32(self.miss_value, hit);
        self.miss_value -= hit_to_subtract;
        self.dodge_value -= hit_to_subtract;
        self.crit_value -= hit_to_subtract;
    }

    fn print_table(&self) {
        println!("Miss chance:\t\t{:.1}%", 100.0 * self.miss_value);
        println!("Dodge chance:\t\t{:.1}%", 
                 100.0 * (self.dodge_value - self.miss_value));
        println!("Crit chance:\t\t{:.1}%", 
                 100.0 * (self.crit_value - self.dodge_value));
        println!("Hit chance:\t\t{:.1}%", 
                 100.0 * (1.0 - self.crit_value));
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

    fn add_hit(&mut self, hit: f32) {
        let hit_to_subtract = max_f32(0.0, self.miss_value - hit);
        self.miss_value -= hit_to_subtract;
        self.dodge_value -= hit_to_subtract;
        self.glancing_value -= hit_to_subtract;
        self.crit_value -= hit_to_subtract;
    }

    fn print_table(&self) {
        println!("Miss chance:\t\t{:.1}%", 100.0 * self.miss_value);
        println!("Dodge chance:\t\t{:.1}%", 
                 100.0 * (self.dodge_value - self.miss_value));
        println!("Glancing chance:\t{:.1}%", 
                 100.0 * (self.glancing_value - self.dodge_value));
        println!("Crit chance:\t\t{:.1}%", 
                 100.0 * (self.crit_value - self.glancing_value));
        println!("Hit chance:\t\t{:.1}%", 
                 100.0 * (1.0 - self.crit_value));
    }
}

#[derive(Debug)]
struct Modifiers {
    general: GeneralModifiers,
    hit: HitModifiers,
    crit: CritModifiers,
    finisher: FinisherModifiers,
    armor_factor: f32
}

impl Modifiers {
    fn new() -> Modifiers {
        Modifiers {
            general: GeneralModifiers::new(),
            hit: HitModifiers::new(),
            crit: CritModifiers::new(),
            finisher: FinisherModifiers::new(),
            armor_factor: 1.0
        }
    }

    fn armor_reduction(&self, dmg: f32) -> f32 {
        return dmg * self.armor_factor;
    }

    fn set_modifiers(&mut self, character: &Character) {
        self.general.set_modifiers(character);
        self.set_armor_factor();
    }

    fn set_armor_factor(&mut self) {
        let mut armor = 3731.0;
        // 5 sunder armor stacks
        armor -= 2250.0;
        // CoR
        armor -= 640.0;
        // Faerie fire
        armor -= 505.0;
        armor = max_f32(armor, 0.0);
        let x = armor / (85.0 * 60.0 + 40.0);
        let red = x / (1.0 + x);
        self.armor_factor = 1.0 - red;
    }

}

#[derive(Debug)]
struct GeneralModifiers {
    slice_and_dice_duration_modifier: f32,
    attack_speed_modifier: f32,
    energy_regen_modifier: i32,
    energy_max: i32
}

impl GeneralModifiers {
    fn new() -> GeneralModifiers {
        GeneralModifiers {
            slice_and_dice_duration_modifier: 1.0,
            attack_speed_modifier: 1.0,
            energy_regen_modifier: 1,
            energy_max: 100
        }
    }

    fn set_modifiers(&mut self, character: &Character) {
        self.attack_speed_modifier *= 1.0 + character.sec_stats.haste;
    }
}

#[derive(Debug)]
struct HitModifiers {
    glancing_mh: f32,
    glancing_oh: f32,
    sinister_strike: f32,
    backstab: f32,
    eviscerate: f32,
    oh: f32
}

impl HitModifiers {
    fn new() -> HitModifiers {
        HitModifiers {
            glancing_mh: 1.0,
            glancing_oh: 1.0,
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
    restore_energy_chance_per_combo_point: f32,
    add_combo_point_chance: f32
}

impl FinisherModifiers {
    fn new() -> FinisherModifiers {
        FinisherModifiers {
            restore_energy_chance_per_combo_point: 0.0,
            add_combo_point_chance: 0.0
        }
    }

    fn gets_extra_combo_point(&self) -> bool {
        let die = roll_die();
        if die < self.add_combo_point_chance { return true; }
        else { return false; }
    }

    fn gets_extra_energy(&self, combo_points: i32) -> bool {
        let die = roll_die();
        if die < combo_points as f32 
            * self.restore_energy_chance_per_combo_point { 
            return true; 
        }
        else { return false; }
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


#[derive(Clone,Copy,Debug,Display,PartialEq)]
pub enum WeaponSlot {
    Mh,
    Oh,
    None
}
