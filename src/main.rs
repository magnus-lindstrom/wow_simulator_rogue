/* TODO
 * - things taken out that will be reimplemented
 *   - weapon enchants
 *   - crusader proccs
 *
 * - Dynamic time steps
 * - Display everything in terms of atp
 * - 20% chance to apply a poison, base stat.
 * - 20% chance for a boss to resist the poison
 * - 130 dmg with current best instant poison
 */
mod armory;

extern crate rand;
extern crate clap;

use crate::armory::{Rogue, Weapon, Item, Stats};
use std::fs::File;
use std::fmt;
use std::io::{BufRead, BufReader};
use std::f32;
use rand::distributions::{Distribution, Uniform};
use clap::{Arg, App};


fn main() {

    let dt = 0.01;

    let args = get_arguments();

    let mut rogue: Rogue;
    let mut mh: Weapon;
    let mut oh: Weapon;

    // get the different rogues that will be looped through
    let characters = get_characters(&args);
    let rogues = characters.0;
    let descriptions = characters.1;
    let mut chars_dps_vectors = Vec::new();

    for (description, rogue) in descriptions.iter().zip(rogues.iter()) {

        if !args.no_buffs { rogue.add_raid_buffs(); }

        // if not going for weights, print hit chances once
        if !args.weights { print_hit_chances(&rogue, &mh, &oh); }

        let mut statistics = Statistics::new();

        for _i_run in 0..args.iterations {

            let mut time_struct = TimeTilEvents::new(args.fight_length);

            let mut extra_attacks: i8 = 0;

            while time_struct.fight_ends > 0.0 {

                check_for_crusader(&mut rogue, &mh, &oh);

                if time_struct.glob_cd_refresh <= 0.0 {
                    let extra_swing =
                        yellow_attack(&mut rogue, &mut buffs, &mut mh, 
                                      &mut time_struct, &mut statistics, &args);
                    if extra_swing { extra_attacks += 1; }
                }
                // check if oh is ready for swing
                if time_struct.oh_swing <= 0.0 {
                    let extra_swing =
                        white_attack(&mut rogue, &mut oh, &mut statistics,
                                     time_struct.fight_ends, 
                                     &args);

                    // Reset swing timer
                    let mut haste_factor = 1.0;
                    if buffs.snd > 0.0 { haste_factor *= 1.3; } 
                    if buffs.blade_flurry > 0.0 { haste_factor *= 1.2; } 
                    if rogue.get_haste() > 0.0 { haste_factor *= 1.0 + rogue.get_haste(); }
                    time_struct.oh_swing = oh.get_speed() / haste_factor;

                    if extra_swing { extra_attacks += 1; }
                }
                // check if extra swings are lined up
                while extra_attacks > 0 {
                    if args.verb { println!("Extra swing!"); }
                    let extra_swing =
                        white_attack(&mut rogue, &mut mh, &mut statistics, 
                                     time_struct.fight_ends, &args);

                    // Reset swing timer
                    let mut haste_factor = 1.0;
                    if buffs.snd > 0.0 { haste_factor *= 1.3; } 
                    if buffs.blade_flurry > 0.0 { haste_factor *= 1.2; } 
                    if rogue.get_haste() > 0.0 { haste_factor *= 1.0 + rogue.get_haste(); }
                    time_struct.mh_swing = mh.get_speed() / haste_factor;

                    if !extra_swing {
                        extra_attacks -= 1;
                    }
                }

                // check if mh is ready for swing
                if time_struct.mh_swing <= 0.0 {
                    let extra_swing = 
                        white_attack(&mut rogue, &mut mh, &mut statistics, 
                                     time_struct.fight_ends, &args);
                    
                    // Reset swing timer
                    let mut haste_factor = 1.0;
                    if buffs.snd > 0.0 { haste_factor *= 1.3; } 
                    if buffs.blade_flurry > 0.0 { haste_factor *= 1.2; } 
                    if rogue.get_haste() > 0.0 { haste_factor *= 1.0 + rogue.get_haste(); }
                    time_struct.mh_swing = mh.get_speed() / haste_factor;


                    if extra_swing { extra_attacks += 1; }
                }
                subtract_times(&mut rogue, &mut time_struct, &mut buffs,
                               &mut mh, &mut oh, dt);
            }
            statistics.get_dps(args.fight_length);

        }

        chars_dps_vectors.push(mean(&statistics.dps));

        print_statistics(&statistics, &args, &chars_dps_vectors, 
                         description.to_string());
    }
}

fn print_statistics(statistics: &Statistics, args: &Args, 
                    chars_dps_vectors: &Vec<f32>, description: String) {

    if !args.weights {
        println!("Average dps for {} over {} runs was {:}.", 
                 args.param_file, args.iterations, 
                 chars_dps_vectors.last().unwrap());
        println!("White:      {:.3} (MH/OH:{:.2}/{:.2})", 
                 mean(&statistics.ratios.white), 
                 mean(&statistics.ratios.white_mh),
                 mean(&statistics.ratios.white_oh));
        println!("Backstab:   {:.3}", mean(&statistics.ratios.backstab));
        println!("Poison:     {:.3}", mean(&statistics.ratios.poison));
        println!("Eviscerate: {:.3}", mean(&statistics.ratios.eviscerate));
        println!("Sin strike: {:.3}", mean(&statistics.ratios.sinister));

    } else {
        let mean_dps_std = 1.96 * std_dev(&statistics.dps) 
            / (args.iterations as f32).sqrt();
        if description == "base" {
            println!("\nAnalysis of stat weights based on char in file: {}", 
                     args.param_file);
            println!("Enemy level: {}", args.enemy_lvl);
            println!("Length of each fight: {}", 
                     args.fight_length);
            println!("Nr of iterations per variation: {}\n", 
                     args.iterations);
            println!(
                "{:_<15}:{:>9.3}  ±{:.3} dps  (±{:.3}dps in distribution)", 
                description, mean(&statistics.dps), mean_dps_std, 
                std_dev(&statistics.dps));
        } else {
            let mut dps_diff = mean(&statistics.dps) - chars_dps_vectors[0];
            dps_diff = 100.0 * dps_diff / chars_dps_vectors[0];
            println!("{:_<15}:{:>+9.3}% ±{:.3}%", description, dps_diff, 
                     1.41 * 100.0 * mean_dps_std / chars_dps_vectors[0]);
        }
    }
}

fn check_for_crusader(rogue: &mut Rogue, mh: &Weapon, oh: &Weapon) {

    if mh.crusader > 0.0 && oh.crusader > 0.0 {
        rogue.nr_crusaders_active = 2.0;
    } else if mh.crusader > 0.0 || oh.crusader > 0.0 {
        rogue.nr_crusaders_active = 1.0;
    } else {
        rogue.nr_crusaders_active = 0.0;
    }
}

fn get_arguments() -> Args {

    let matches = App::new("WoW rogue simulator") 
        .version("0.1.0") 
        .author("Magnus Lindström <magnus.lindstrom@tuta.io>")
        .about("Compares items/specs for PvE raiding purposes. Combat Rogues.") 
        .arg(Arg::with_name("Parameter file") 
             .required(true)
             .short("f") 
             .long("file").takes_value(true) 
             .help("Parameter file that contains all rogue traits."))
        .arg(Arg::with_name("Nr of iterations") 
             .short("i") 
             .long("iterations").takes_value(true) 
             .help("Number of iterations to average over. Default is 10 000"))
        .arg(Arg::with_name("Fight length") 
             .short("t") 
             .long("time").takes_value(true) 
             .help("Seconds of duration per fight. Default is 60s."))
        .arg(Arg::with_name("Enemy level") 
             .short("e") 
             .long("enemy_lvl").takes_value(true) 
             .help("Lvl of the enemy. Default is 63."))
        .arg(Arg::with_name("Weights") 
            .short("w") 
            .long("weights") 
            .takes_value(false) 
            .help("Permute stats/talents slightly to get delta dps values."))
        .arg(Arg::with_name("Weight multiplier") 
            .short("m") 
            .long("weight_mult") 
            .takes_value(true) 
            .help("Change degree of permutation by a factor."))
        .arg(Arg::with_name("No buffs") 
            .short("n") 
            .long("no_buffs") 
            .takes_value(false) 
            .help("Run without buffs to rogue."))
        .arg(Arg::with_name("Verbose") 
            .short("v") 
            .long("verbose") 
            .takes_value(false) 
            .help("Be verbose, print details about fights."))
        .get_matches();

    let file = matches.value_of("Parameter file").unwrap();
    let iterations = matches.value_of("Nr of iterations").unwrap_or("10_000");
    let fight_length = matches.value_of("Fight length").unwrap_or("60");
    let enemy_lvl = matches.value_of("Enemy level").unwrap_or("63");
    let weights = matches.is_present("Weights");
    let weight_mult = matches.value_of("Weight multiplier").unwrap_or("1");
    let no_buffs = matches.is_present("No buffs");
    let verb = matches.is_present("Verbose");

    let mut args = Args::default_args();
    args.param_file = file.to_string();
    args.verb = verb;
    args.no_buffs = no_buffs;
    args.weights = weights;
    args.weight_mult = weight_mult.parse().unwrap();
    args.enemy_lvl = enemy_lvl.parse().unwrap();
    args.iterations = iterations.parse().unwrap();
    let fl: u16 = fight_length.parse().unwrap();
    args.fight_length = fl as f32;

    if args.fight_length > 120.0 {
        println!("WARNING: fight longer than 2min, no cooldown for");
        println!("blade flurry implemented.");
    }

    return args;

}

fn crusader_roll(wep: &mut Weapon, verb: bool) {
    let die = roll_die();
    if die < wep.get_speed() / 40.0 {
        wep.crusader = 15.0;
        if verb { 
            if wep.is_offhand { println!("Crusader OH procc."); }
            else { println!("Crusader MH procc."); }
        }
    }
}

fn shadowcraft_roll(rogue: &mut Rogue) {
    if rogue.shadowcraft_six_bonus {
        let die = roll_die();
        if die < 0.03 { 
            if rogue.energy < 66 { rogue.energy += 35; } 
            else { rogue.energy = 100; }
        }
    }
}

fn print_hit_chances(rogue: &Rogue, mh: &Weapon, oh: &Weapon) {

    println!("*** MH: White hits summary ***");
    println!("miss chance: {:}", mh.white_miss);
    println!("dodge chance: {:}", mh.dodge_chance);
    println!("glancing chance: {:}", rogue.glancing_chance);
    println!("crit chance: {:}", mh.crit);
    let mut tmp = mh.white_miss;
    let mut tmp1 = tmp + mh.dodge_chance;
    let mut tmp2 = tmp1 + rogue.glancing_chance;
    let mut tmp3 = tmp2 + mh.crit;
    println!("hit chance: {:}", 1.0 - tmp3);
    println!("{:}-{:}-{:}-{:}\n", tmp, tmp1, tmp2, tmp3);
    
    println!("*** MH: Yellow hits summary ***");
    println!("miss chance: {:}", mh.yellow_miss);
    println!("dodge chance: {:}", mh.dodge_chance);
    println!("crit chance: {:}", mh.crit);
    tmp = mh.yellow_miss;
    tmp1 = tmp + mh.dodge_chance;
    tmp2 = tmp1 + mh.crit;
    println!("hit chance: {:}", 1.0 - tmp2);
    println!("{:}-{:}-{:}\n", tmp, tmp1, tmp2);

    if mh.is_dagger {
        println!("*** MH: Backstab summary ***");
        println!("miss chance: {:}", mh.yellow_miss);
        println!("dodge chance: {:}", mh.dodge_chance);
        println!("crit chance: {:}", mh.crit_backstab);
        tmp = mh.yellow_miss;
        tmp1 = tmp + mh.dodge_chance;
        tmp2 = tmp1 + mh.crit_backstab;
        println!("hit chance: {:}", 1.0 - tmp2);
        println!("{:}-{:}-{:}\n", tmp, tmp1, tmp2);
    }

    println!("*** OH: White hits summary ***");
    println!("miss chance: {:}", oh.white_miss);
    println!("dodge chance: {:}", oh.dodge_chance);
    println!("glancing chance: {:}", rogue.glancing_chance);
    println!("crit chance: {:}", oh.crit);
    tmp = oh.white_miss;
    tmp1 = tmp + oh.dodge_chance;
    tmp2 = tmp1 + rogue.glancing_chance;
    tmp3 = tmp2 + oh.crit;
    println!("hit chance: {:}", 1.0 - tmp3);
    println!("{:}-{:}-{:}-{:}\n", tmp, tmp1, tmp2, tmp3);
    
}

pub struct Args {
    no_buffs: bool,
    enemy_lvl: i16,
    fight_length: f32,
    iterations: u32,
    param_file: String,
    verb: bool,
    weight_mult: u16,
    weights: bool
}

impl Args {
    fn default_args() -> Args {
        Args {
            no_buffs: false,
            enemy_lvl: 0,
            fight_length: 0.0,
            iterations: 0,
            param_file: "".to_string(),
            verb: false,
            weight_mult: 0,
            weights: false
        }
    }
}

fn announce_hit(dmg: f32, attack_type: String, hit_type: HitType, time: f32) {
    if attack_type == "sin_strike" {
        println!("{:2.1}: Sinister strike {} for {:.0}", time, hit_type, dmg);
    } else if attack_type == "backstab" {
        println!("{:2.1}: Backstab {} for {:.0}", time, hit_type, dmg);
    } else if attack_type == "evis" {
        println!("{:2.1}: Eviscerate {} for {:.0}", time, hit_type, dmg);
    } else if attack_type == "mh_white" {
        println!("{:2.1}: MH white {} for {:.0}", time, hit_type, dmg);
    } else if attack_type == "oh_white" {
        println!("{:2.1}: OH white {} for {:.0}", time, hit_type, dmg);
    } else if attack_type == "snd" {
        println!("{:2.1}: Slice and dice applied for {:.2}s", time, dmg);
    } else if attack_type == "cds" {
        println!("{:2.1}: Blade flurry + Adrenaline Rush for 15s", time);
    } else if attack_type == "thistle_tea" {
        println!("{:2.1}: Thistle tea used.", time);
    } else if attack_type == "poison" {
        println!("{:2.1}: Instant poison for {:.0}", time, dmg);
    }
}

fn roll_die() -> f32 {
    // rolls a die between [0, 1)
    
    let mut rng = rand::thread_rng();
    let roll_range = Uniform::from(100..10_000); // not including upper bound
    let roll = roll_range.sample(&mut rng);
    let roll: f32 = (roll as f32) / 10_000.0;
    return roll;
}

fn determine_hit(rogue: &Rogue, wep: &Weapon, color: String, 
                 is_backstab: bool) -> HitType {

    let roll: f32 = roll_die();
    let mut percent_sum: f32;

    if color == "yellow" {

        if roll < wep.yellow_miss { return HitType::Miss; }
        percent_sum = wep.yellow_miss + wep.dodge_chance;
        if roll < percent_sum { return HitType::Dodge; }
        if is_backstab { percent_sum += wep.crit_backstab; } 
        else { percent_sum += wep.crit; }
        if roll < percent_sum { return HitType::Crit; }
        return HitType::Hit;

    } else if color == "white" {

        if roll < wep.white_miss { return HitType::Miss; }
        percent_sum = wep.white_miss + wep.dodge_chance;
        if roll < percent_sum { return HitType::Dodge; }
        percent_sum += rogue.glancing_chance;
        if roll < percent_sum { return HitType::Glancing; }
        percent_sum += wep.crit;
        if roll < percent_sum { return HitType::Crit; }
        return HitType::Hit;

    } else { panic!("can only strike yellow or white hits"); }

}

fn backstab(rogue: &mut Rogue, wep: &Weapon, statistics: &mut Statistics,
            time_struct: &TimeTilEvents, verb: bool) -> f32 {

    let hit_result = determine_hit(&rogue, wep, "yellow".to_string(),
        true);
    let mut dmg: f32;
    statistics.add_yellow_hit(&hit_result);

    if hit_result == HitType::Miss || hit_result == HitType::Dodge {
        rogue.energy -= 12;
        dmg = 0.0;

    } else if hit_result == HitType::Crit {
        rogue.energy -= 60;
        dmg = get_backstab_dmg(&wep, &rogue);
        dmg *= 2.0 + 0.06 * rogue.lethality as f32;
        rogue.combo_points += 1;

    } else if hit_result == HitType::Hit {
        rogue.energy -= 60;
        dmg = get_backstab_dmg(&wep, &rogue);
        rogue.combo_points += 1;
    } else { panic!("Backstab can't be glancing hit."); }
    dmg *= 1.0 + 0.04 * rogue.opportunity as f32;
    dmg = armor_reduction(dmg);

    if verb {
        announce_hit(dmg, "backstab".to_string(), hit_result, 
                     time_struct.fight_ends);
    }

    return dmg;
}

fn sinister_strike(rogue: &mut Rogue, wep: &Weapon, statistics: &mut Statistics,
                   time_struct: &TimeTilEvents, verb: bool) -> f32 {

    let hit_result = determine_hit(&rogue, wep, "yellow".to_string(),
        false);
    let mut dmg: f32 = 0.0;
    statistics.add_yellow_hit(&hit_result);

    if hit_result == HitType::Miss || hit_result == HitType::Dodge {
        rogue.energy -= 8; //todo fix this
        dmg = 0.0;

    } else if hit_result == HitType::Glancing {
        rogue.energy -= 40;
        dmg = get_sinister_strike_dmg(&wep, &rogue);
        dmg *= 1.0 - wep.glancing_red;
        rogue.combo_points += 1;

    } else if hit_result == HitType::Crit {
        rogue.energy -= 40;
        dmg = get_sinister_strike_dmg(&wep, &rogue);
        dmg *= 2.0 + 0.06 * rogue.lethality as f32;
        rogue.combo_points += 1;

    } else if hit_result == HitType::Hit {
        rogue.energy -= 40;
        dmg = get_sinister_strike_dmg(&wep, &rogue);
        rogue.combo_points += 1;
    }
    if verb {
        announce_hit(dmg, "sin_strike".to_string(), hit_result, 
                     time_struct.fight_ends);
    }

    dmg *= 1.0 + (0.02 * rogue.aggression as f32);
    dmg = armor_reduction(dmg);

    return dmg;
}

fn eviscerate(rogue: &mut Rogue, wep: &Weapon, statistics: &mut Statistics, 
              time_struct: &TimeTilEvents, verb: bool) -> f32 {

    let hit_result = determine_hit(&rogue, wep, "yellow".to_string(), false);
    let mut dmg: f32 = 0.0;
    statistics.add_yellow_hit(&hit_result);

    if hit_result == HitType::Miss || hit_result == HitType::Dodge {
        dmg = 0.0;

    } else if hit_result == HitType::Crit {
        dmg = get_evis_dmg(rogue);
        dmg *= 2.0;
        let die = roll_die();
        if die < 0.2 * rogue.ruthlessness as f32 {
            rogue.combo_points = 1;
        } else { rogue.combo_points = 0; }

    } else if hit_result == HitType::Hit {
        dmg = get_evis_dmg(rogue);
        let die = roll_die();
        if die < 0.2 * rogue.ruthlessness as f32 {
            rogue.combo_points = 1;
        } else { rogue.combo_points = 0; }
    }
    rogue.energy -= 35;
    dmg *= 1.0 + (0.02 * rogue.aggression as f32);
    dmg *= 1.0 + (0.05 * rogue.improved_eviscerate as f32);
    dmg = armor_reduction(dmg);
    if verb {
        announce_hit(dmg, "evis".to_string(), hit_result, 
                     time_struct.fight_ends);
    }
    return dmg;
}

fn yellow_attack(rogue: &mut Rogue, mut buffs: &mut BuffsActive,
                 wep: &mut Weapon, 
                 mut time_struct: &mut TimeTilEvents, statistics: &mut Statistics,
                 args: &Args) -> bool {
    // returns dmg and a true bool if an extra attack was triggered

    // use thistle tea if low on energy
    if rogue.energy < 10 && !buffs.used_thistle_tea {
        buffs.used_thistle_tea = true;
        rogue.energy = 100;
        if args.verb {
            announce_hit(0.0, "thistle_tea".to_string(), 
                         HitType::None, time_struct.fight_ends);
        }
    }
    
    let mut dmg = 0.0;
    let mut extra_hit: bool = false;
    
    let can_blade_flurry = rogue.energy >= 20 && !buffs.used_cds;
    let can_sinister = rogue.energy >= 40;
    let can_backstab = rogue.energy >= 60;
    let can_eviscerate = rogue.energy >= 35;
    let can_snd = rogue.energy >= 25;
    let snd_active = buffs.snd > 0.0;


    // Use cooldowns if good situation
    if can_blade_flurry && buffs.snd > 15.0 {
        rogue.energy -= 20;
        buffs.blade_flurry = 15.0;
        buffs.adrenaline_rush = 15.0;
        time_struct.glob_cd_refresh = 1.0;
        buffs.used_cds = true;
        if args.verb {
            announce_hit(0.0, "cds".to_string(), HitType::None, 
                         time_struct.fight_ends);
        }
    // Short snd if no snd up at 2 combo points
    } else if rogue.combo_points == 2 && can_snd && !snd_active {
        rogue.energy -= 25;
        buffs.snd = snd_duration(rogue);
        time_struct.glob_cd_refresh = 1.0;
        let die = roll_die();
        if die < 0.2 * rogue.ruthlessness as f32 {
            rogue.combo_points = 1;
        } else { rogue.combo_points = 0; }
        if args.verb {
            announce_hit(buffs.snd, "snd".to_string(), HitType::None, 
                         time_struct.fight_ends);
        }
    // Sword weapon
    // Sinister strike if not yet at 5 combo points
    } else if wep.is_sword && rogue.combo_points < 5 && can_sinister {
        dmg = sinister_strike(rogue, wep, statistics, &time_struct, args.verb);
        statistics.add_sinister_dmg(dmg);
        time_struct.glob_cd_refresh = 1.0;
        if rogue.combo_points > 5 { rogue.combo_points = 5; }

    // Dagger weapon
    // Backstab if not yet at 5 combo points
    } else if wep.is_dagger && rogue.combo_points < 5 && can_backstab {
        dmg = backstab(rogue, wep, statistics, &time_struct, args.verb);
        statistics.add_backstab_dmg(dmg);
        time_struct.glob_cd_refresh = 1.0;
        if rogue.combo_points > 5 { rogue.combo_points = 5; }

    // Long snd if no snd up at 5 combo points
    } else if rogue.combo_points == 5 && can_snd && !snd_active {
        rogue.energy -= 25;
        buffs.snd = snd_duration(rogue);
        time_struct.glob_cd_refresh = 1.0;
        let die = roll_die();
        if die < 0.2 * rogue.ruthlessness as f32 {
            rogue.combo_points = 1;
        } else { rogue.combo_points = 0; }
        if args.verb {
            announce_hit(buffs.snd, "snd".to_string(), HitType::None,
                         time_struct.fight_ends);
        }
    // Full eviscerate at 5 combo points if snd is up
    } else if rogue.combo_points == 5 && snd_active && can_eviscerate { 
        dmg = eviscerate(rogue, wep, statistics, &time_struct, args.verb);
        statistics.add_eviscerate_dmg(dmg);
        time_struct.glob_cd_refresh = 1.0;
    }

    if dmg > 0.0 {
        if wep.enchant == "crusader" { 
            crusader_roll(wep, args.verb); 
        }
        shadowcraft_roll(rogue);
        extra_hit = roll_for_extra_hit(rogue, wep);
        let dmg_poison = poison_dmg(args.verb, time_struct.fight_ends);
        statistics.add_poison_dmg(dmg_poison);
    }

    return extra_hit;
}

fn roll_for_extra_hit(rogue: &mut Rogue, wep: &Weapon) -> bool {
    let die = roll_die();
    if die < rogue.extra_hit_proc_chance + wep.extra_hit_proc_chance {
        return true;
    } else { return false; }
}

fn get_total_attack_power(rogue: &Rogue) -> f32 {
    let attack_power = 100 + rogue.agility + rogue.strength 
        + rogue.attack_power;
    return attack_power as f32;
}

fn get_wep_dmg(wep: &Weapon, rogue: &Rogue) -> f32 {

    let mut attack_power = get_total_attack_power(&rogue);
    attack_power += rogue.nr_crusaders_active * 100.0;
    let dmg = wep.mean_dmg + attack_power * wep.get_speed() / 14.0;
    return dmg;
}

fn get_sinister_strike_dmg(wep: &Weapon, rogue: &Rogue) -> f32 {
    let normal_wep_dmg = get_wep_dmg(&wep, &rogue);
    let dmg = normal_wep_dmg + 68.0;
    return dmg;
}

fn get_backstab_dmg(wep: &Weapon, rogue: &Rogue) -> f32 {
    let normal_wep_dmg = get_wep_dmg(&wep, &rogue);
    let dmg = 1.5 * normal_wep_dmg + 210.0;
    return dmg;
}

fn snd_duration(rogue: &mut Rogue) -> f32 {

    let mut dur: f32 = 0.0;
    if rogue.combo_points == 1 { dur = 9.0; }
    if rogue.combo_points == 2 { dur = 12.0; }
    if rogue.combo_points == 3 { dur = 15.0; }
    if rogue.combo_points == 4 { dur = 18.0; }
    if rogue.combo_points == 5 { dur = 21.0; }
    dur *= 1.0 + (0.15 * rogue.improved_slice_and_dice as f32);

    return dur;
}

fn get_evis_dmg(rogue: &mut Rogue) -> f32 {
    let mut dmg: f32 ;
    if rogue.combo_points == 1 { dmg = 247.0; }
    else if rogue.combo_points == 2 { dmg = 398.0; }
    else if rogue.combo_points == 3 { dmg = 549.0; }
    else if rogue.combo_points == 4 { dmg = 700.0; }
    else if rogue.combo_points == 5 { dmg = 851.0; }
    else { panic!("Invalid nr of combo points in get_evis_dmg"); }

    let attack_power = get_total_attack_power(&rogue);
    dmg += (attack_power * (rogue.combo_points as f32)) * 0.05;
    return dmg
}

fn white_attack(rogue: &mut Rogue, wep: &mut Weapon, statistics: &mut Statistics, 
                time_left: f32, args: &Args) -> bool {
    // returns damage and a bool that is true if an extra swing procced

    let hit_result = determine_hit(&rogue, wep, "white".to_string(),
        false);
    statistics.add_white_hit(&hit_result);
    let announce_string: String;
    if wep.is_offhand {
        announce_string = "oh_white".to_string();
    } else {
        announce_string = "mh_white".to_string();
    }

    if hit_result == HitType::Miss || hit_result == HitType::Dodge {
        if args.verb {
            announce_hit(0.0, announce_string, hit_result, time_left);
        }
        return false;
    }

    // below only happens on damage dealt
    
    if wep.enchant == "crusader" { 
        crusader_roll(wep, args.verb); 
    }
    shadowcraft_roll(rogue);

    let mut dmg = get_wep_dmg(&wep, &rogue);
    if wep.is_offhand {
        dmg = dmg * 0.5 * (1.0 + 0.1 * rogue.dw_specialization as f32) ;
    } 
    if hit_result == HitType::Glancing { dmg *= 1.0 - wep.glancing_red; }
    else if hit_result == HitType::Crit { dmg *= 2.0; }
    dmg = armor_reduction(dmg);

    // add dmg to dmg counter
    if wep.is_offhand { statistics.add_white_oh_dmg(dmg); }
    else { statistics.add_white_mh_dmg(dmg); }

    if args.verb { announce_hit(dmg, announce_string, hit_result, time_left); }
    let extra_hit: bool = roll_for_extra_hit(rogue, wep);
    let dmg_poison = poison_dmg(args.verb, time_left);
    statistics.add_poison_dmg(dmg_poison);

    return extra_hit;
}

fn poison_dmg(verb: bool, time_left: f32) -> f32 {

    let die = roll_die();
    let mut dmg = 0.0;
    if die < 0.2 { 
        dmg = 130.0;
        if verb {
            announce_hit(dmg, "poison".to_string(), HitType::None, 
                         time_left);
        }
    }
    return dmg;
}

fn armor_reduction(dmg: f32) -> f32 {
    let mut armor = 3731.0;
    // 5 sunder armor stacks
    armor -= 2250.0;
    // CoR
    armor -= 640.0;
    // Faerie Fire
    armor -= 505.0;
    armor = max_f32(armor, 0.0);
    let x = 0.1 * armor / (8.5 * 60.0 + 40.0);
    let red = x / (1.0 + x);
    return dmg * (1.0 - red);
}

fn copy_wep(wep: &Weapon) -> Weapon {
    let copy = Weapon {
        speed: wep.speed,
        max_dmg: wep.max_dmg,
        min_dmg: wep.min_dmg,
        mean_dmg: wep.mean_dmg,
        enchant: wep.enchant.clone(),
        crusader: wep.crusader,
        is_offhand: wep.is_offhand,
        is_dagger: wep.is_dagger,
        is_sword: wep.is_sword,
        crit: wep.crit,
        crit_backstab: wep.crit_backstab,
        dodge_chance: wep.dodge_chance,
        white_miss: wep.white_miss,
        yellow_miss: wep.yellow_miss,
        glancing_red: wep.glancing_red,
        extra_hit_proc_chance: wep.extra_hit_proc_chance
    };
    return copy;
}

struct TimeTilEvents {
    glob_cd_refresh: f32,
    mh_swing: f32,
    oh_swing: f32,
    energy_refill: f32,
    fight_ends: f32
}

impl TimeTilEvents {
    pub fn new(fight_length: f32) -> TimeTilEvents {
        TimeTilEvents {
            glob_cd_refresh: 0.0,
            mh_swing: 0.0,
            oh_swing: 0.0,
            energy_refill: 1.0,
            fight_ends: fight_length
        }
    }
}

struct StatRatio {
    backstab: Vec<f32>,
    eviscerate: Vec<f32>,
    poison: Vec<f32>,
    sinister: Vec<f32>,
    white: Vec<f32>,
    white_mh: Vec<f32>,
    white_oh: Vec<f32>
}

impl StatRatio {
    pub fn new() -> StatRatio {
        StatRatio {
            backstab: Vec::new(),
            eviscerate: Vec::new(),
            poison: Vec::new(),
            sinister: Vec::new(),
            white: Vec::new(),
            white_mh: Vec::new(),
            white_oh: Vec::new()
        }
    }
}

struct StatSum {
    backstab_dmg: f32,
    eviscerate_dmg: f32,
    poison_dmg: f32,
    sinister_dmg: f32,
    white_mh_dmg: f32,
    white_oh_dmg: f32
}

impl StatSum {
    pub fn new() -> StatSum {
        StatSum {
            backstab_dmg: 0.0,
            eviscerate_dmg: 0.0,
            poison_dmg: 0.0,
            sinister_dmg: 0.0,
            white_mh_dmg: 0.0,
            white_oh_dmg: 0.0
        }
    }

    pub fn reset_counters(&mut self) {
        self.backstab_dmg = 0.0;
        self.eviscerate_dmg = 0.0;
        self.poison_dmg = 0.0;
        self.sinister_dmg = 0.0;
        self.white_mh_dmg = 0.0;
        self.white_oh_dmg = 0.0;
    }
}

#[derive(Debug, PartialEq, Eq)]
enum HitType {
    Hit,
    Crit,
    Glancing,
    Miss,
    Dodge,
    None
}

impl fmt::Display for HitType { 
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { 
        let printable = match *self { 
            HitType::Hit => "hit", 
            HitType::Crit => "crit", 
            HitType::Glancing => "glancing", 
            HitType::Miss => "miss", 
            HitType::Dodge => "dodge",
            HitType::None => "none" 
        }; 
        write!(f, "{}", printable)
    }
}

#[derive(Debug)]
struct Hits {
    hit: u32,
    crit: u32,
    glancing: u32,
    miss: u32,
    dodge: u32
}

impl Hits {
    pub fn new() -> Hits {
        Hits {
            hit: 0,
            crit: 0,
            glancing: 0,
            miss: 0,
            dodge: 0
        }
    }
}

struct Statistics {
    sums: StatSum,
    yellow_hits: Hits,
    white_hits: Hits,
    ratios: StatRatio,
    dps: Vec<f32>
}

impl Statistics {
    pub fn new() -> Statistics {
        Statistics {
            sums: StatSum::new(),
            yellow_hits: Hits::new(),
            white_hits: Hits::new(),
            ratios: StatRatio::new(),
            dps: Vec::new()
        }
    }

    pub fn add_yellow_hit(&mut self, hit_type: &HitType) {
        if *hit_type == HitType::Hit { self.yellow_hits.hit += 1; }
        else if *hit_type == HitType::Crit { self.yellow_hits.crit += 1; }
        else if *hit_type == HitType::Miss { self.yellow_hits.miss += 1; }
        else if *hit_type == HitType::Dodge { self.yellow_hits.dodge += 1; }
        else { panic!("Hit type {:?} not allowed for yellow hits!", hit_type); }
    }

    pub fn add_white_hit(&mut self, hit_type: &HitType) {
        if *hit_type == HitType::Hit { self.white_hits.hit += 1; }
        else if *hit_type == HitType::Crit { self.white_hits.crit += 1; }
        else if *hit_type == HitType::Glancing { self.white_hits.glancing += 1; }
        else if *hit_type == HitType::Miss { self.white_hits.miss += 1; }
        else if *hit_type == HitType::Dodge { self.white_hits.dodge += 1; }
        else { panic!("Hit type {:?} not allowed for white hits!", hit_type); }
    }


    pub fn sum(&self) -> f32 {
        let dmg_sum = 
            self.sums.backstab_dmg
            + self.sums.eviscerate_dmg
            + self.sums.sinister_dmg
            + self.sums.white_mh_dmg
            + self.sums.white_oh_dmg
            + self.sums.poison_dmg
            ;
        return dmg_sum;
    }

    pub fn add_white_mh_dmg(&mut self, dmg: f32) {
        self.sums.white_mh_dmg += dmg;
    }
    pub fn add_white_oh_dmg(&mut self, dmg: f32) {
        self.sums.white_oh_dmg += dmg;
    }
    pub fn add_eviscerate_dmg(&mut self, dmg: f32) {
        self.sums.eviscerate_dmg += dmg;
    }
    pub fn add_backstab_dmg(&mut self, dmg: f32) {
        self.sums.backstab_dmg += dmg;
    }
    pub fn add_sinister_dmg(&mut self, dmg: f32) {
        self.sums.sinister_dmg += dmg;
    }
    pub fn add_poison_dmg(&mut self, dmg: f32) {
        self.sums.poison_dmg += dmg; 
    }

    pub fn get_dps(&mut self, fight_length: f32) {
        let dmg_sum = self.sum();

        self.ratios.backstab.push(self.sums.backstab_dmg / dmg_sum);
        self.ratios.eviscerate.push(self.sums.eviscerate_dmg / dmg_sum);
        self.ratios.sinister.push(self.sums.sinister_dmg / dmg_sum);
        self.ratios.white.push(
            (self.sums.white_mh_dmg + self.sums.white_oh_dmg)
            / dmg_sum);
        self.ratios.white_mh.push(self.sums.white_mh_dmg / dmg_sum);
        self.ratios.white_oh.push(self.sums.white_oh_dmg / dmg_sum);
        self.ratios.poison.push(self.sums.poison_dmg / dmg_sum);

        self.dps.push(dmg_sum / fight_length);

        self.sums.reset_counters();
    }
}

fn deb<T: std::fmt::Debug>(x: T) {
    println!("{:?}", x);
}

fn subtract_times(mut rogue: &mut Rogue, 
                  mut time_struct: &mut TimeTilEvents, 
                  mut buffs: &mut BuffsActive, mh: &mut Weapon, 
                  oh: &mut Weapon, dt: f32) {

    if time_struct.glob_cd_refresh > 0.0 {
        time_struct.glob_cd_refresh -= dt;
    } 
    if time_struct.mh_swing > 0.0 {
        time_struct.mh_swing -= dt;
    } 
    if time_struct.oh_swing > 0.0 { 
        time_struct.oh_swing -= dt; 
    }

    if mh.crusader > 0.0 { 
        mh.crusader -= dt; 
    }
    if oh.crusader > 0.0 { 
        oh.crusader -= dt; 
    }

    time_struct.energy_refill -= dt;
    if time_struct.energy_refill <= 0.0 { 
        let mut energy_increase: i8;
        let die = roll_die();
        if die < 0.25 { energy_increase = 21; }
        else { energy_increase = 20; }
        if buffs.adrenaline_rush > 0.0 { energy_increase *= 2; }

        if rogue.energy < (100 - energy_increase) {
            rogue.energy += energy_increase;
        } else { rogue.energy = 100; }

        time_struct.energy_refill = 2.0; 
    }

    time_struct.fight_ends -= dt;

    if buffs.blade_flurry > 0.0 {
        buffs.blade_flurry -= dt;
    }
    if buffs.adrenaline_rush > 0.0 {
        buffs.adrenaline_rush -= dt;
    }
    // want to see for how long we've been without slice and dice
    buffs.snd -= dt;
}

fn get_characters(args: &Args) -> (Vec<Rogue>, Vec<String>) {

    let mut rogues = Vec::new();
    let mut descriptions = Vec::new();

    let mut rogue = read_params(&args.param_file);
    rogue.calculate_stats();
    rogue.calculate_hit_numbers(&args);
    rogues.push(rogues);
    descriptions.push("base".to_string());

    if args.weights {
        // one less agility
        rogues = read_params(&args.param_file);
        if rogues.stats.agility >= 8 * args.weight_mult {
            rogues.stats.agility -= 8 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("-{} agi", 8 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more agility
        rogues = read_params(&args.param_file);
        rogues.stats.agility += 8 * args.weight_mult;
        rogues.push(rogues);
        let desc = format!("+{} agi", 8 * args.weight_mult);
        descriptions.push(desc);
        
        // ten less strength
        rogues = read_params(&args.param_file);
        if rogues.stats.strength >= 8 * args.weight_mult {
            rogues.stats.strength -= 8 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("-{} str", 8 * args.weight_mult);
            descriptions.push(desc);
        }
        // ten more strength
        rogues = read_params(&args.param_file);
        rogues.stats.strength += 8 * args.weight_mult;
        rogues.push(rogues);
        let desc = format!("+{} str", 8 * args.weight_mult);
        descriptions.push(desc);
         
        // ten less attack power
        rogues = read_params(&args.param_file);
        if rogues.stats.attack_power >= 8 * args.weight_mult {
            rogues.stats.attack_power -= 8 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("-{} atp", 8 * args.weight_mult);
            descriptions.push(desc);
        }
        // ten more attack power
        rogues = read_params(&args.param_file);
        rogues.stats.attack_power += 8 * args.weight_mult;
        rogues.push(rogues);
        let desc = format!("+{} atp", 8 * args.weight_mult);
        descriptions.push(desc);
        
        // one less hit
        rogues = read_params(&args.param_file);
        if rogues.stats.hit >= 0.01 * args.weight_mult as f32 {
            rogues.stats.hit -= 0.01 * args.weight_mult as f32;
            rogues.push(rogues);
            let desc = format!("-{} hit", 0.01 * args.weight_mult as f32);
            descriptions.push(desc);
        }
        // one more hit
        rogues = read_params(&args.param_file);
        rogues.stats.hit += 0.01 * args.weight_mult as f32;
        rogues.push(rogues);
        let desc = format!("+{} hit", 0.01 * args.weight_mult as f32);
        descriptions.push(desc);

        // one less crit
        rogues = read_params(&args.param_file);
        if rogues.stats.crit >= 0.01 * args.weight_mult as f32 {
            rogues.stats.crit -= 0.01 * args.weight_mult as f32;
            rogues.push(rogues);
            let desc = format!("-{} crit", 0.01 * args.weight_mult as f32);
            descriptions.push(desc);
        }
        // one more crit
        rogues = read_params(&args.param_file);
        rogues.stats.crit += 0.01 * args.weight_mult as f32;
        rogues.push(rogues);
        let desc = format!("+{} crit", 0.01 * args.weight_mult as f32);
        descriptions.push(desc);
         
        // one less haste
        rogues = read_params(&args.param_file);
        if rogues.stats.haste >= 0.01  * args.weight_mult as f32{
            rogues.stats.haste -= 0.01 * args.weight_mult as f32;
            rogues.push(rogues);
            let desc = format!("-{} haste", 0.01 * args.weight_mult as f32);
            descriptions.push(desc);
        }
        // one more haste
        rogues = read_params(&args.param_file);
        rogues.stats.haste += 0.01 * args.weight_mult as f32;
        rogues.push(rogues);
        let desc = format!("+{} haste", 0.01 * args.weight_mult as f32);
        descriptions.push(desc);

        // one less dagger skill
        rogues = read_params(&args.param_file);
        if rogues.stats.daggers_skill >= 300 + 1 * args.weight_mult as i16 {
            rogues.stats.daggers_skill -= 1 * args.weight_mult as i16;
            rogues.push(rogues);
            let desc = format!("-{} dgr skill", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more dagger skill
        rogues = read_params(&args.param_file);
        rogues.stats.daggers_skill += 1 * args.weight_mult as i16;
        rogues.push(rogues);
        let desc = format!("+{} dgr skill", 1 * args.weight_mult);
        descriptions.push(desc);
         
        // two less extra hit proc chance
        rogues = read_params(&args.param_file);
        if rogues.stats.extra_hit_proc_chance >= 0.02 * args.weight_mult as f32 {
            rogues.stats.extra_hit_proc_chance -= 0.02 * args.weight_mult as f32;
            rogues.push(rogues);
            let desc = format!("-{} hit proc", 0.02 * args.weight_mult as f32);
            descriptions.push(desc);
        }
        // two more extra hit proc chance
        rogues = read_params(&args.param_file);
        rogues.stats.extra_hit_proc_chance += 0.02 * args.weight_mult as f32;
        rogues.push(rogues);
        let desc = format!("+{} hit proc", 0.02 * args.weight_mult as f32);
        descriptions.push(desc);

        // TALENTS

        // one less improved eviscerate
        rogues = read_params(&args.param_file);
        if rogues.stats.stats.proved_eviscerate >= 1 * args.weight_mult {
            rogues.stats.improved_eviscerate -= 1 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("-{} imp evis", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more improved eviscerate
        rogues = read_params(&args.param_file);
        if rogues.stats.improved_eviscerate <= 3 - 1 * args.weight_mult {
            rogues.stats.improved_eviscerate += 1 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("+{} imp evis", 1 * args.weight_mult);
            descriptions.push(desc);
        }

        // one less malice
        rogues = read_params(&args.param_file);
        if rogues.stats.malice >= 1 * args.weight_mult {
            rogues.stats.malice -= 1 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("-{} malice", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more malice
        rogues = read_params(&args.param_file);
        if rogues.stats.malice <= 5 - 1 * args.weight_mult {
            rogues.stats.malice += 1 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("+{} malice", 1 * args.weight_mult);
            descriptions.push(desc);
        }

        // one less ruthlessness
        rogues = read_params(&args.param_file);
        if rogues.stats.ruthlessness >= 1 * args.weight_mult {
            rogues.stats.ruthlessness -= 1 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("-{} ruthlessness", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more ruthlessness
        rogues = read_params(&args.param_file);
        if rogues.stats.ruthlessness <= 3 - 1 * args.weight_mult {
            rogues.stats.ruthlessness += 1 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("+{} ruthlessness", 1 * args.weight_mult);
            descriptions.push(desc);
        }

        // one less imp slice and dice
        rogues = read_params(&args.param_file);
        if rogues.stats.improved_slice_and_dice >= 1 * args.weight_mult {
            rogues.stats.improved_slice_and_dice -= 1 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("-{} imp snd", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more imp slice and dice
        rogues = read_params(&args.param_file);
        if rogues.stats.improved_slice_and_dice <= 3 - 1 * args.weight_mult {
            rogues.stats.improved_slice_and_dice += 1 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("+{} imp snd", 1 * args.weight_mult);
            descriptions.push(desc);
        }

        // one less relentless strikes
        rogues = read_params(&args.param_file);
        if rogues.stats.relentless_strikes >= 1 * args.weight_mult {
            rogues.stats.relentless_strikes -= 1 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("-{} rel strikes", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more relentless strikes
        rogues = read_params(&args.param_file);
        if rogues.stats.relentless_strikes <= 1 - 1 * args.weight_mult {
            rogues.stats.relentless_strikes += 1 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("+{} rel strikes", 1 * args.weight_mult);
            descriptions.push(desc);
        }

        // one less lethality
        rogues = read_params(&args.param_file);
        if rogues.stats.lethality >= 1 * args.weight_mult {
            rogues.stats.lethality -= 1 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("-{} lethality", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more lethality
        rogues = read_params(&args.param_file);
        if rogues.stats.lethality <= 5 - 1 * args.weight_mult {
            rogues.stats.lethality += 1 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("+{} lethality", 1 * args.weight_mult);
            descriptions.push(desc);
        }

        // one less improved backstab
        rogues = read_params(&args.param_file);
        if rogues.stats.imp_backstab >= 1 * args.weight_mult {
            rogues.stats.imp_backstab -= 1 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("-{} imp bkstab", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more improved backstab
        rogues = read_params(&args.param_file);
        if rogues.stats.imp_backstab <= 3 - 1 * args.weight_mult {
            rogues.stats.imp_backstab += 1 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("+{} imp bkstab", 1 * args.weight_mult);
            descriptions.push(desc);
        }

        // one less precision
        rogues = read_params(&args.param_file);
        if rogues.stats.precision >= 1 * args.weight_mult {
            rogues.stats.precision -= 1 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("-{} precision", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more precision
        rogues = read_params(&args.param_file);
        if rogues.stats.precision <= 5 - 1 * args.weight_mult {
            rogues.stats.precision += 1 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("+{} precision", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        
        // one less dw_specialization
        rogues = read_params(&args.param_file);
        if rogues.stats.dw_specialization >= 1 * args.weight_mult {
            rogues.stats.dw_specialization -= 1 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("-{} dw spec", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more dw_specialization
        rogues = read_params(&args.param_file);
        if rogues.stats.dw_specialization <= 5 - 1 * args.weight_mult {
            rogues.stats.dw_specialization += 1 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("+{} dw spec", 1 * args.weight_mult);
            descriptions.push(desc);
        }
         
        // one less sword_specialization
        rogues = read_params(&args.param_file);
        if rogues.stats.sword_specialization >= 1 * args.weight_mult {
            rogues.stats.sword_specialization -= 1 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("-{} sword spec", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more sword_specialization
        rogues = read_params(&args.param_file);
        if rogues.stats.sword_specialization <= 5 - 1 * args.weight_mult {
            rogues.stats.sword_specialization += 1 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("+{} sword spec", 1 * args.weight_mult);
            descriptions.push(desc);
        }
         
        // one less dagger_specialization
        rogues = read_params(&args.param_file);
        if rogues.stats.dagger_specialization >= 1 * args.weight_mult {
            rogues.stats.dagger_specialization -= 1 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("-{} dagger spec", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more dagger_specialization
        rogues = read_params(&args.param_file);
        if rogues.stats.dagger_specialization <= 5 - 1 * args.weight_mult {
            rogues.stats.dagger_specialization += 1 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("+{} dagger spec", 1 * args.weight_mult);
            descriptions.push(desc);
        }

        // one less weapon expertise
        rogues = read_params(&args.param_file);
        if rogues.stats.weapon_expertise >= 1 * args.weight_mult {
            rogues.stats.weapon_expertise -= 1 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("-{} wep expert", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more weapon_expertise
        rogues = read_params(&args.param_file);
        if rogues.stats.weapon_expertise <= 2 - 1 * args.weight_mult {
            rogues.stats.weapon_expertise += 1 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("+{} wep expert", 1 * args.weight_mult);
            descriptions.push(desc);
        }

        // one less aggression
        rogues = read_params(&args.param_file);
        if rogues.stats.aggression >= 1 * args.weight_mult {
            rogues.stats.aggression -= 1 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("-{} aggression", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more aggression
        rogues = read_params(&args.param_file);
        if rogues.stats.aggression <= 3 - 1 * args.weight_mult {
            rogues.stats.aggression += 1 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("+{} aggression", 1 * args.weight_mult);
            descriptions.push(desc);
        }

        // one less opportunity
        rogues = read_params(&args.param_file);
        if rogues.stats.opportunity >= 1 * args.weight_mult {
            rogues.stats.opportunity -= 1 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("-{} opportunity", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more opportunity
        rogues = read_params(&args.param_file);
        if rogues.stats.opportunity <= 5 - 1 * args.weight_mult {
            rogues.stats.opportunity += 1 * args.weight_mult;
            rogues.push(rogues);
            let desc = format!("+{} opportunity", 1 * args.weight_mult);
            descriptions.push(desc);
        }
    } 
    return (rogues, descriptions);
}

fn read_params(param_file: &String) -> Rogue {
    
    let mut param_field: u16 = 0; // to check what part the file is about
    let mut read_last = false;
    let mut rogue = Rogue::new();

    let f = File::open(param_file).expect("Couldn't open param_file");
    let file = BufReader::new(&f);
    for line in file.lines() {
        let l = line.unwrap();
        let first_char = l.chars().next().unwrap();
        if first_char != '#' && first_char != '@' {
            read_last = true;
            if param_field == 0 { 
                let item = Item::new(&l); 
                rogue.item_set.push(item);
            }
            else if param_field == 1 { rogue.mh = Weapon::new(&l); }
            else if param_field == 2 { rogue.oh = Weapon::new(&l); }
        } else { continue; }

        if read_last {
            param_field += 1;
        }
        read_last = false;
    }
    rogue
}
  
fn param_adder(text: &str, rogue: &mut Rogue) {

    let words = text.split_whitespace();
    let words_vec = words.collect::<Vec<&str>>();

    if words_vec[0] == "imp_backstab" { 
        match words_vec[1].parse() {
            Ok(x) => rogue.imp_backstab = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "precision" { 
        match words_vec[1].parse() {
            Ok(x) => rogue.precision = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "dw_specialization" { 
        match words_vec[1].parse() {
            Ok(x) => rogue.dw_specialization = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "sword_specialization" { 
        match words_vec[1].parse() {
            Ok(x) => rogue.sword_specialization = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "dagger_specialization" { 
        match words_vec[1].parse() {
            Ok(x) => rogue.dagger_specialization = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "weapon_expertise" { 
        match words_vec[1].parse() {
            Ok(x) => rogue.weapon_expertise = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "aggression" { 
        match words_vec[1].parse() {
            Ok(x) => rogue.aggression = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "opportunity" { 
        match words_vec[1].parse() {
            Ok(x) => rogue.opportunity = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "improved_eviscerate" { 
        match words_vec[1].parse() {
            Ok(x) => rogue.improved_eviscerate = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "malice" { 
        match words_vec[1].parse() {
            Ok(x) => rogue.malice = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "ruthlessness" { 
        match words_vec[1].parse() {
            Ok(x) => rogue.ruthlessness = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "improved_slice_and_dice" { 
        match words_vec[1].parse() {
            Ok(x) => rogue.improved_slice_and_dice = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "relentless_strikes" { 
        match words_vec[1].parse() {
            Ok(x) => rogue.relentless_strikes = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "lethality" { 
        match words_vec[1].parse() {
            Ok(x) => rogue.lethality = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else {
        panic!("Unrecognized keyword in params file: {}", words_vec[0]);
    }
}

fn subtract_hit_from_miss(rogue: &Rogue, mh: &mut Weapon, oh: &mut Weapon, 
                          args: &Args) {

    // MH
    let wep_skill: i16;
    if mh.is_dagger { wep_skill = rogue.daggers_skill; }
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
        oh.yellow_miss = max_f32( 0.0, oh.yellow_miss - (rogue.hit - 0.01) );
        oh.white_miss = max_f32( 0.0, oh.white_miss - (rogue.hit - 0.01) );
    } else {
        oh.yellow_miss = max_f32(0.0, oh.yellow_miss - rogue.hit);
        oh.white_miss = max_f32(0.0, oh.white_miss - rogue.hit);
    }

}

fn mean(numbers: &Vec<f32>) -> f32 {

    let mut sum: f64 = 0.0;
    for n in numbers.iter() { sum += *n as f64; }

    let avg = sum / numbers.len() as f64;
    return avg as f32;
}

fn std_dev(numbers: &Vec<f32>) -> f32 {

    let mean = mean(numbers);
    let mut tmp = 0.0;
    for nr in numbers {
        tmp += (nr - mean).powf(2.0);
    }
    let std_dev = (tmp / (numbers.len() - 1) as f32).sqrt();
    return std_dev;
}

fn max_f32(x: f32, y: f32) -> f32 {
    if x >= y { return x; }
    else { return y; }
}
