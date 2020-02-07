/* TODO
 *   - Bring in weapons
 *   - Dynamic time steps
 *   - Display everything in terms of atp
 *   - 20% chance to apply a poison, base stat.
 *   - 20% chance for a boss to resist the poison
 *   - 130 dmg with current best instant poison
 */
extern crate rand;
extern crate clap;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::f32;
use rand::distributions::{Distribution, Uniform};
use clap::{Arg, App};


fn main() {

    let dt = 0.01;

    let args = get_arguments();

    let mut total_extra_hits: u32 = 0;
    let mut total_mh_hits: u32 = 0;
    let mut total_oh_hits: u32 = 0;

    let mut rogue: Rogue;
    let mut mh: Weapon;
    let mut oh: Weapon;

    // get the different characters that will be looped through
    let character_tuples = get_characters(&args);
    let characters = character_tuples.0;
    let descriptions = character_tuples.1;
    let mut chars_dps_vectors = Vec::new();

    for (description, character) in descriptions.iter().zip(characters.iter()) {

        rogue = character.0;
        mh = copy_wep(&character.1);
        oh = copy_wep(&character.2);

        if !args.no_buffs { add_raid_buffs(&mut rogue); }

        calculate_hit_numbers(&mut rogue, &mut mh, &mut oh, &args);

        // if not going for weights, print hit chances once
        if !args.weights { print_hit_chances(&rogue, &mh, &oh); }

        let mut stats = Stats::new();

        for _i_run in 0..args.iterations {

            let mut buffs = BuffsActive::new();

            let mut time_struct = TimeTilEvents::new(args.fight_length);

            let mut extra_attacks: i8 = 0;

            while time_struct.fight_ends > 0.0 {

                check_for_crusader(&mut rogue, &mh, &oh);

                if time_struct.glob_cd_refresh <= 0.0 {
                    let (dmg, dmg_poison, extra_swing) = 
                        yellow_attack(&mut rogue, &mut buffs, &mh, 
                                      &mut time_struct, &mut stats, args.verb);
                    if dmg > 0.0 { 
                        if mh.enchant == "crusader" { 
                            crusader_roll(&mut mh, args.verb); 
                        }
                        shadowcraft_roll(&mut rogue);
                        total_mh_hits += 1;
                    }
                    stats.add_poison_dmg(dmg_poison);
                    if extra_swing { extra_attacks += 1; }
                }
                // check if oh is ready for swing
                if time_struct.oh_swing <= 0.0 {
                    let (dmg, dmg_poison, extra_swing) = 
                        white_attack(&mut rogue, &mut oh, 
                                     time_struct.fight_ends, &mut stats, 
                                     args.verb);
                    if dmg > 0.0 { 
                        if oh.enchant == "crusader" { 
                            crusader_roll(&mut oh, args.verb); 
                        }
                        shadowcraft_roll(&mut rogue);
                        total_oh_hits += 1; 
                    }
                    stats.add_poison_dmg(dmg_poison);

                    // Reset swing timer
                    let mut haste_factor = 1.0;
                    if buffs.snd > 0.0 { haste_factor *= 1.3; } 
                    if buffs.blade_flurry > 0.0 { haste_factor *= 1.2; } 
                    if rogue.haste > 0.0 { haste_factor *= 1.0 + rogue.haste; }
                    time_struct.oh_swing = oh.speed / haste_factor;

                    if extra_swing { extra_attacks += 1; }
                }
                // check if extra swings are lined up
                while extra_attacks > 0 {
                    if args.verb { println!("Extra swing!"); }
                    total_extra_hits += 1;
                    let (dmg, dmg_poison, extra_swing) = 
                        white_attack(&mut rogue, &mut mh, time_struct.fight_ends,
                                     &mut stats, args.verb);
                    if dmg > 0.0 { 
                        if mh.enchant == "crusader" { 
                            crusader_roll(&mut mh, args.verb); 
                        }
                        shadowcraft_roll(&mut rogue);
                        total_mh_hits += 1;
                    }
                    stats.add_poison_dmg(dmg_poison);

                    // Reset swing timer
                    let mut haste_factor = 1.0;
                    if buffs.snd > 0.0 { haste_factor *= 1.3; } 
                    if buffs.blade_flurry > 0.0 { haste_factor *= 1.2; } 
                    if rogue.haste > 0.0 { haste_factor *= 1.0 + rogue.haste; }
                    time_struct.mh_swing = mh.speed / haste_factor;

                    if !extra_swing {
                        extra_attacks -= 1;
                    }
                }

                // check if mh is ready for swing
                if time_struct.mh_swing <= 0.0 {
                    let (dmg, dmg_poison, extra_swing) = 
                        white_attack(&mut rogue, &mut mh, time_struct.fight_ends,
                                     &mut stats, args.verb);
                    if dmg > 0.0 { 
                        if mh.enchant == "crusader" { 
                            crusader_roll(&mut mh, args.verb); 
                        }
                        shadowcraft_roll(&mut rogue);
                        total_mh_hits += 1; 
                    }
                    stats.add_poison_dmg(dmg_poison);
                    
                    // Reset swing timer
                    let mut haste_factor = 1.0;
                    if buffs.snd > 0.0 { haste_factor *= 1.3; } 
                    if buffs.blade_flurry > 0.0 { haste_factor *= 1.2; } 
                    if rogue.haste > 0.0 { haste_factor *= 1.0 + rogue.haste; }
                    time_struct.mh_swing = mh.speed / haste_factor;


                    if extra_swing { extra_attacks += 1; }
                }
                subtract_times(&mut rogue, &mut time_struct, &mut buffs,
                               &mut mh, &mut oh, dt);
            }
            stats.get_dps(args.fight_length);

        }

        chars_dps_vectors.push(mean(&stats.dps));

        if !args.weights {
            println!("Average dps for {} over {} runs was {:}.", 
                     args.param_file, args.iterations, 
                     chars_dps_vectors.last().unwrap());
            println!("White:      {:.3}", mean(&stats.ratios.white));
            println!("Backstab:   {:.3}", mean(&stats.ratios.backstab));
            println!("Poison:     {:.3}", mean(&stats.ratios.poison));
            println!("Eviscerate: {:.3}", mean(&stats.ratios.eviscerate));
            println!("Sin strike: {:.3}", mean(&stats.ratios.sinister));
        } else {
            let mean_dps_std = 1.96 * std_dev(&stats.dps) 
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
                    description, mean(&stats.dps), mean_dps_std, 
                    std_dev(&stats.dps));
            } else {
                let mut dps_diff = mean(&stats.dps) - chars_dps_vectors[0];
                dps_diff = 100.0 * dps_diff / chars_dps_vectors[0];
                println!("{:_<15}:{:>+9.3}% ±{:.3}%", description, 
                         dps_diff, 1.41 * 100.0 * mean_dps_std / chars_dps_vectors[0]);
            }
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

    let mut args: Args = default_args();
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
    if die < wep.speed / 40.0 {
        wep.crusader = 15.0;
        if verb { 
            if wep.is_offhand { println!("Crusader OH procc."); }
            else { println!("Crusader MH procc."); }
        }
    }
}

fn add_raid_buffs(rogue: &mut Rogue) {
    // motw
    rogue.agility += 12;
    rogue.strength += 12;
    // trueshot aura
    rogue.attack_power += 200;
    // DM buff
    rogue.attack_power += 200;
    // Ony buff
    rogue.attack_power += 140;
    rogue.crit += 0.05;
    // bom
    rogue.attack_power += 185;
    // battle shout
    rogue.attack_power += 241;
    // juju power
    rogue.strength += 30;
    // juju might
    rogue.attack_power += 40;
    // mongoose
    rogue.agility += 25;
    rogue.crit += 0.02;
    // grilled squid
    rogue.agility += 10;
    // bok
    rogue.agility = (rogue.agility as f32 * 1.1) as u16;
    rogue.strength = (rogue.strength as f32 * 1.1) as u16;
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

struct Args {
    no_buffs: bool,
    enemy_lvl: i16,
    fight_length: f32,
    iterations: u32,
    param_file: String,
    verb: bool,
    weight_mult: u16,
    weights: bool
}

fn default_args() -> Args {
    let args = Args {
        no_buffs: false,
        enemy_lvl: 0,
        fight_length: 0.0,
        iterations: 0,
        param_file: "".to_string(),
        verb: false,
        weight_mult: 0,
        weights: false
    };
    return args;
}

fn announce_hit(dmg: f32, attack_type: String, hit_type: String, time: f32) {
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
                 is_backstab: bool) -> String {

    let roll: f32 = roll_die();
    let mut percent_sum: f32;

    if color == "yellow" {

        if roll < wep.yellow_miss { return "miss".to_string(); }
        percent_sum = wep.yellow_miss + wep.dodge_chance;
        if roll < percent_sum { return "dodge".to_string(); }
        if is_backstab { percent_sum += wep.crit_backstab; } 
        else { percent_sum += wep.crit; }
        if roll < percent_sum { return "crit".to_string(); }

        return "hit".to_string();

    } else if color == "white" {

        if roll < wep.white_miss { return "miss".to_string(); }
        percent_sum = wep.white_miss + wep.dodge_chance;
        if roll < percent_sum { return "dodge".to_string(); }
        percent_sum += rogue.glancing_chance;
        if roll < percent_sum { return "glancing".to_string(); }
        percent_sum += wep.crit;
        if roll < percent_sum { return "crit".to_string(); }
        return "hit".to_string();

    } else { panic!("can only strike yellow or white hits"); }

}

fn backstab(rogue: &mut Rogue, wep: &Weapon, 
            time_struct: &TimeTilEvents, verb: bool) -> f32 {

    let hit_result = determine_hit(&rogue, wep, "yellow".to_string(),
        true);
    let mut dmg: f32;

    if hit_result == "miss" || hit_result == "dodge" {
        rogue.energy -= 12;
        dmg = 0.0;

    } else if hit_result == "crit" {
        rogue.energy -= 60;
        dmg = get_backstab_dmg(&wep, &rogue);
        dmg *= 2.0 + 0.06 * rogue.lethality as f32;
        rogue.combo_points += 1;

    } else if hit_result == "hit" {
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

fn sinister_strike(rogue: &mut Rogue, wep: &Weapon, 
                   time_struct: &TimeTilEvents, verb: bool) -> f32 {

    let hit_result = determine_hit(&rogue, wep, "yellow".to_string(),
        false);
    let mut dmg: f32 = 0.0;

    if hit_result == "miss" || hit_result == "dodge" {
        rogue.energy -= 8; //todo fix this
        dmg = 0.0;

    } else if hit_result == "glancing" {
        rogue.energy -= 40;
        dmg = get_sinister_strike_dmg(&wep, &rogue);
        dmg *= 1.0 - wep.glancing_red;
        rogue.combo_points += 1;

    } else if hit_result == "crit" {
        rogue.energy -= 40;
        dmg = get_sinister_strike_dmg(&wep, &rogue);
        dmg *= 2.0 + 0.06 * rogue.lethality as f32;
        rogue.combo_points += 1;

    } else if hit_result == "hit" {
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

fn eviscerate(rogue: &mut Rogue, wep: &Weapon, time_struct: &TimeTilEvents,
              verb: bool) -> f32 {

    let hit_result = determine_hit(&rogue, wep, 
                                   "yellow".to_string(), false);
    let mut dmg: f32 = 0.0;

    if hit_result == "miss" || hit_result == "dodge" {
        dmg = 0.0;

    } else if hit_result == "crit" {
        dmg = get_evis_dmg(rogue);
        dmg *= 2.0;
        let die = roll_die();
        if die < 0.2 * rogue.ruthlessness as f32 {
            rogue.combo_points = 1;
        } else { rogue.combo_points = 0; }

    } else if hit_result == "hit" {
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
                 wep: &Weapon, 
                 mut time_struct: &mut TimeTilEvents, stats: &mut Stats,
                 verb: bool) -> (f32, f32, bool) {
    // returns dmg and a true bool if an extra attack was triggered

    // use thistle tea if low on energy
    if rogue.energy < 10 && !buffs.used_thistle_tea {
        buffs.used_thistle_tea = true;
        rogue.energy = 100;
        if verb {
            announce_hit(0.0, "thistle_tea".to_string(), 
                         "thistle_tea".to_string(), time_struct.fight_ends);
        }
    }
    
    let mut dmg = 0.0;
    let mut dmg_poison = 0.0;
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
        if verb {
            announce_hit(0.0, "cds".to_string(), "cds".to_string(), 
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
        if verb {
            announce_hit(buffs.snd, "snd".to_string(), "snd".to_string(), 
                         time_struct.fight_ends);
        }
    // Sword weapon
    // Sinister strike if not yet at 5 combo points
    } else if wep.is_sword && rogue.combo_points < 5 && can_sinister {
        dmg = sinister_strike(rogue, wep, &time_struct, verb);
        if dmg > 0.0 {
            stats.sums.sinister_dmg += dmg;
            extra_hit = roll_for_extra_hit(rogue, wep);
            dmg_poison = poison_dmg(verb, time_struct.fight_ends);
        }
        time_struct.glob_cd_refresh = 1.0;
        if rogue.combo_points > 5 { rogue.combo_points = 5; }

    // Dagger weapon
    // Backstab if not yet at 5 combo points
    } else if wep.is_dagger && rogue.combo_points < 5 && can_backstab {
        dmg = backstab(rogue, wep, &time_struct, verb);
        if dmg > 0.0 {
            stats.sums.backstab_dmg += dmg;
            extra_hit = roll_for_extra_hit(rogue, wep);
            dmg_poison = poison_dmg(verb, time_struct.fight_ends);
        }
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
        if verb {
            announce_hit(buffs.snd, "snd".to_string(), "snd".to_string(),
                         time_struct.fight_ends);
        }
    // Full eviscerate at 5 combo points if snd is up
    } else if rogue.combo_points == 5 && snd_active && can_eviscerate { 
        dmg = eviscerate(rogue, wep, &time_struct, verb);
        if dmg > 0.0 {
            stats.sums.eviscerate_dmg += dmg;
            extra_hit = roll_for_extra_hit(rogue, wep);
            dmg_poison = poison_dmg(verb, time_struct.fight_ends);
        }
        time_struct.glob_cd_refresh = 1.0;
    }

    return (dmg, dmg_poison, extra_hit);
}

fn roll_for_extra_hit(rogue: &mut Rogue, wep: &Weapon) -> bool {
    let die = roll_die();
    if die < rogue.extra_hit_proc_chance + wep.extra_hit_proc_chance {
        return true;
    } else { return false; }
}

fn set_glancing_reduction(rogue: &mut Rogue, mh: &mut Weapon, 
                          oh: &mut Weapon, enemy_lvl: i16) {

    let mut delta_skill: i16;

    // MH
    if mh.is_dagger { delta_skill = 5 * enemy_lvl - rogue.daggers_skill; }
    else { delta_skill = 5 * enemy_lvl - rogue.swords_skill; }

    if      delta_skill == 15 { mh.glancing_red = 0.35; }
    else if delta_skill == 14 { mh.glancing_red = 0.31; }
    else if delta_skill == 13 { mh.glancing_red = 0.27; }
    else if delta_skill == 12 { mh.glancing_red = 0.23; }
    else if delta_skill == 11 { mh.glancing_red = 0.19; }
    else if delta_skill == 10 { mh.glancing_red = 0.15; }
    else if delta_skill == 09 { mh.glancing_red = 0.11; }
    else if delta_skill == 08 { mh.glancing_red = 0.07; }
    else if delta_skill <= 07 { mh.glancing_red = 0.05; }
    else { panic!("weapon skill-enemy defense not implemented"); }

    // OH
    if oh.is_dagger { delta_skill = 5 * enemy_lvl - rogue.daggers_skill; }
    else { delta_skill = 5 * enemy_lvl - rogue.swords_skill; }

    if      delta_skill == 15 { oh.glancing_red = 0.35; }
    else if delta_skill == 14 { oh.glancing_red = 0.31; }
    else if delta_skill == 13 { oh.glancing_red = 0.27; }
    else if delta_skill == 12 { oh.glancing_red = 0.23; }
    else if delta_skill == 11 { oh.glancing_red = 0.19; }
    else if delta_skill == 10 { oh.glancing_red = 0.15; }
    else if delta_skill == 09 { oh.glancing_red = 0.11; }
    else if delta_skill == 08 { oh.glancing_red = 0.07; }
    else if delta_skill <= 07 { oh.glancing_red = 0.05; }
    else { panic!("weapon skill-enemy defense not implemented"); }
}

fn set_glancing_chance(rogue: &mut Rogue, enemy_lvl: i16) {
    if enemy_lvl == 60 { rogue.glancing_chance = 0.1; }
    else if enemy_lvl == 61 { rogue.glancing_chance = 0.2; }
    else if enemy_lvl == 62 { rogue.glancing_chance = 0.3; }
    else if enemy_lvl == 63 { rogue.glancing_chance = 0.4; }
    else { panic!("No reliable glancing info on levels below 60"); }
}

fn set_dodge_chance(rogue: &Rogue, mh: &mut Weapon, oh: &mut Weapon, 
                    enemy_lvl: i16) {
    if enemy_lvl < 60 { 
        mh.dodge_chance = 0.05; 
        oh.dodge_chance = 0.05; 
    } else {
        // MH
        let mh_wep_skill: i16;
        if mh.is_dagger { mh_wep_skill = rogue.daggers_skill; }
        else { mh_wep_skill = rogue.swords_skill; }
        mh.dodge_chance = 0.05 + (5 * enemy_lvl - mh_wep_skill) as f32 * 0.001; 
        // OH
        let oh_wep_skill: i16;
        if oh.is_dagger { oh_wep_skill = rogue.daggers_skill; }
        else { oh_wep_skill = rogue.swords_skill; }
        oh.dodge_chance = 0.05 + (5 * enemy_lvl - oh_wep_skill) as f32 * 0.001; 
    }
}

fn set_yellow_miss_chance(rogue: &mut Rogue, mh: &mut Weapon,
                          oh: &mut Weapon, enemy_lvl: i16) {
    // MH 
    let mut delta_skill: i16;
    if mh.is_dagger {
        delta_skill = 5 * enemy_lvl - rogue.daggers_skill;
    } else {
        delta_skill = 5 * enemy_lvl - rogue.swords_skill;
    }
    if delta_skill < 0 { mh.yellow_miss = 0.05; }
    else if delta_skill <= 10 && delta_skill >= 0 { 
        mh.yellow_miss = 0.05 + 0.001 * delta_skill as f32; 
    } else if delta_skill == 11 { mh.yellow_miss = 0.072; }
    else if delta_skill == 12 { mh.yellow_miss = 0.074; }
    else if delta_skill == 13 { mh.yellow_miss = 0.076; }
    else if delta_skill == 14 { mh.yellow_miss = 0.078; }
    else if delta_skill == 15 { mh.yellow_miss = 0.080; }
    else { panic!("Weapon skill-enemy lvl combo not implemented"); }

    // OH 
    if oh.is_dagger {
        delta_skill = 5 * enemy_lvl - rogue.daggers_skill;
    } else {
        delta_skill = 5 * enemy_lvl - rogue.swords_skill;
    }
    if delta_skill < 0 { oh.yellow_miss = 0.05; }
    else if delta_skill <= 10 && delta_skill >= 0 { 
        oh.yellow_miss = 0.05 + 0.001 * delta_skill as f32; 
    } else if delta_skill == 11 { oh.yellow_miss = 0.072; }
    else if delta_skill == 12 { oh.yellow_miss = 0.074; }
    else if delta_skill == 13 { oh.yellow_miss = 0.076; }
    else if delta_skill == 14 { oh.yellow_miss = 0.078; }
    else if delta_skill == 15 { oh.yellow_miss = 0.080; }
    else { panic!("Weapon skill-enemy lvl combo not implemented"); }
}

fn set_white_miss_chance(mh: &mut Weapon, oh: &mut Weapon) {
    mh.white_miss = 0.8 * mh.yellow_miss + 0.2;
    oh.white_miss = 0.8 * oh.yellow_miss + 0.2;
}

fn get_total_attack_power(rogue: &Rogue) -> f32 {
    let attack_power = 100 + rogue.agility + rogue.strength 
        + rogue.attack_power;
    return attack_power as f32;
}

fn get_wep_dmg(wep: &Weapon, rogue: &Rogue) -> f32 {

    let mut attack_power = get_total_attack_power(&rogue);
    attack_power += rogue.nr_crusaders_active * 100.0;
    let dmg = wep.mean_dmg + attack_power * wep.speed / 14.0;
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

fn white_attack(rogue: &mut Rogue, wep: &mut Weapon, time_left: f32, 
                stats: &mut Stats, verb: bool) -> (f32, f32, bool) {
    // returns damage and a bool that is true if an extra swing procced

    let hit_result = determine_hit(&rogue, wep, "white".to_string(),
        false);
    let announce_string: String;
    if wep.is_offhand {
        announce_string = "oh_white".to_string();
    } else {
        announce_string = "mh_white".to_string();
    }

    if hit_result == "miss" || hit_result == "dodge" { 
        if verb {
            announce_hit(0.0, announce_string, hit_result, time_left);
        }
        return (0.0, 0.0, false);
    }

    let mut dmg = get_wep_dmg(&wep, &rogue);
    if wep.is_offhand {
        dmg = dmg * 0.5 * (1.0 + 0.1 * rogue.dw_specialization as f32) ;
    } 
    if hit_result == "glancing" { dmg *= 1.0 - wep.glancing_red; }
    else if hit_result == "crit" { dmg *= 2.0; }
    dmg = armor_reduction(dmg);
    stats.sums.white_dmg += dmg;

    if verb { announce_hit(dmg, announce_string, hit_result, time_left); }
    let extra_hit: bool = roll_for_extra_hit(rogue, wep);
    let dmg_poison = poison_dmg(verb, time_left);

    return (dmg, dmg_poison, extra_hit);
}

fn poison_dmg(verb: bool, time_left: f32) -> f32 {

    let die = roll_die();
    let mut dmg = 0.0;
    if die < 0.2 { 
        dmg = 130.0;
        if verb {
            announce_hit(dmg, "poison".to_string(), "poison".to_string(), 
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

struct Weapon {
    speed: f32,
    max_dmg: u16,
    min_dmg: u16,
    mean_dmg: f32,
    enchant: String,
    crusader: f32, // the time left on crusader
    is_offhand: bool,
    is_dagger: bool,
    is_sword: bool,
    crit: f32,
    crit_backstab: f32,
    dodge_chance: f32,
    white_miss: f32,
    yellow_miss: f32,
    glancing_red: f32,
    extra_hit_proc_chance: f32
}

impl Weapon {
    pub fn new() -> Weapon {
        Weapon {
            speed: 0.0,
            max_dmg: 0,
            min_dmg: 0,
            mean_dmg: 0.0,
            enchant: "none".to_string(),
            crusader: 0.0,
            is_offhand: false,
            is_dagger: false,
            is_sword: false,
            crit: 0.0,
            crit_backstab: 0.0,
            dodge_chance: 0.0,
            white_miss: 0.0,
            yellow_miss: 0.0,
            glancing_red: 0.0,
            extra_hit_proc_chance: 0.0
        }
    }
}

struct BuffsActive {
    blade_flurry: f32,
    snd: f32,
    adrenaline_rush: f32,
    used_cds: bool,
    used_thistle_tea: bool
}

impl BuffsActive {
    pub fn new() -> BuffsActive {
        BuffsActive {
            blade_flurry: 0.0,
            snd: 0.0,
            adrenaline_rush: 0.0,
            used_cds: false,
            used_thistle_tea: false
        }
    }
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
    white_mh: Vec<f32>
    white_oh: Vec<f32>
}

impl StatRatio {
    pub fn new() -> StatRatio {
        StatRatio {
            backstab: Vec::new(),
            eviscerate: Vec::new(),
            poison: Vec::new(),
            sinister: Vec::new(),
            white_mh: Vec::new()
            white_oh: Vec::new()
        }
    }
}

struct StatSum {
    backstab_dmg: f32,
    eviscerate_dmg: f32,
    poison_dmg: f32,
    sinister_dmg: f32,
    white_mh_dmg: f32
    white_oh_dmg: f32
}

impl StatSum {
    pub fn new() -> StatSum {
        StatSum {
            backstab_dmg: 0.0,
            eviscerate_dmg: 0.0,
            poison_dmg: 0.0,
            sinister_dmg: 0.0,
            white_mh_dmg: 0.0
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

struct HitType {
    hit,
    crit,
    glancing,
    miss,
    dodge
}

struct StatHit {
    : f32,
    eviscerate_dmg: f32,
    poison_dmg: f32,
    sinister_dmg: f32,
    white_mh_dmg: f32
    white_oh_dmg: f32
}

impl StatSum {
    pub fn new() -> StatSum {
        StatSum {
            backstab_dmg: 0.0,
            eviscerate_dmg: 0.0,
            poison_dmg: 0.0,
            sinister_dmg: 0.0,
            white_mh_dmg: 0.0
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

struct Stats {
    hits: StatHit,
    sums: StatSum,
    ratios: StatRatio,
    dps: Vec<f32>
}


impl Stats {
    pub fn new() -> Stats {
        Stats {
            sums: StatSum::new(),
            ratios: StatRatio::new(),
            dps: Vec::new()
        }
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
        self.sums.poison += dmg;
    }

    pub fn get_dps(&mut self, fight_length: f32) {
        let dmg_sum = self.sum();

        self.ratios.backstab.push(self.sums.backstab_dmg / dmg_sum);
        self.ratios.eviscerate.push(self.sums.eviscerate_dmg / dmg_sum);
        self.ratios.sinister.push(self.sums.sinister_dmg / dmg_sum);
        self.ratios.white.push(self.sums.white_dmg / dmg_sum);
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

#[derive(Copy, Clone)]
struct Rogue {
    energy: i8,
    agility: u16,
    strength: u16,
    attack_power: u16, // IMPORTANT: just attack power given directly by gear
    crit: f32,
    hit: f32,
    haste: f32,
    nr_crusaders_active: f32,
    swords_skill: i16,
    daggers_skill: i16,
    glancing_chance: f32,
    extra_hit_proc_chance: f32,
    shadowcraft_six_bonus: bool,
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

impl Rogue {
    pub fn new() -> Rogue { 
        Rogue{
            energy: 100,
            agility: 0,
            strength: 0,
            attack_power: 0, // attack power given directly by gear
            crit: 0.0,
            hit: 0.0,
            haste: 0.0,
            nr_crusaders_active: 0.0,
            glancing_chance: 0.0,
            swords_skill: 0,
            daggers_skill: 0,
            extra_hit_proc_chance: 0.0, // does not include thrash blade proc
            shadowcraft_six_bonus: false,
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


fn get_characters(args: &Args) -> (Vec<(Rogue, Weapon, Weapon)>, Vec<String>) {

    let mut characters = Vec::new();
    let mut descriptions = Vec::new();

    let mut char_tuple = read_params(&args.param_file);
    characters.push(char_tuple);
    descriptions.push("base".to_string());

    if args.weights {
        // one less agility
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.agility >= 8 * args.weight_mult {
            char_tuple.0.agility -= 8 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("-{} agi", 8 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more agility
        char_tuple = read_params(&args.param_file);
        char_tuple.0.agility += 8 * args.weight_mult;
        characters.push(char_tuple);
        let desc = format!("+{} agi", 8 * args.weight_mult);
        descriptions.push(desc);
        
        // ten less strength
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.strength >= 8 * args.weight_mult {
            char_tuple.0.strength -= 8 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("-{} str", 8 * args.weight_mult);
            descriptions.push(desc);
        }
        // ten more strength
        char_tuple = read_params(&args.param_file);
        char_tuple.0.strength += 8 * args.weight_mult;
        characters.push(char_tuple);
        let desc = format!("+{} str", 8 * args.weight_mult);
        descriptions.push(desc);
         
        // ten less attack power
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.attack_power >= 8 * args.weight_mult {
            char_tuple.0.attack_power -= 8 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("-{} atp", 8 * args.weight_mult);
            descriptions.push(desc);
        }
        // ten more attack power
        char_tuple = read_params(&args.param_file);
        char_tuple.0.attack_power += 8 * args.weight_mult;
        characters.push(char_tuple);
        let desc = format!("+{} atp", 8 * args.weight_mult);
        descriptions.push(desc);
        
        // one less hit
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.hit >= 0.01 * args.weight_mult as f32 {
            char_tuple.0.hit -= 0.01 * args.weight_mult as f32;
            characters.push(char_tuple);
            let desc = format!("-{} hit", 0.01 * args.weight_mult as f32);
            descriptions.push(desc);
        }
        // one more hit
        char_tuple = read_params(&args.param_file);
        char_tuple.0.hit += 0.01 * args.weight_mult as f32;
        characters.push(char_tuple);
        let desc = format!("+{} hit", 0.01 * args.weight_mult as f32);
        descriptions.push(desc);

        // one less crit
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.crit >= 0.01 * args.weight_mult as f32 {
            char_tuple.0.crit -= 0.01 * args.weight_mult as f32;
            characters.push(char_tuple);
            let desc = format!("-{} crit", 0.01 * args.weight_mult as f32);
            descriptions.push(desc);
        }
        // one more crit
        char_tuple = read_params(&args.param_file);
        char_tuple.0.crit += 0.01 * args.weight_mult as f32;
        characters.push(char_tuple);
        let desc = format!("+{} crit", 0.01 * args.weight_mult as f32);
        descriptions.push(desc);
         
        // one less haste
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.haste >= 0.01  * args.weight_mult as f32{
            char_tuple.0.haste -= 0.01 * args.weight_mult as f32;
            characters.push(char_tuple);
            let desc = format!("-{} haste", 0.01 * args.weight_mult as f32);
            descriptions.push(desc);
        }
        // one more haste
        char_tuple = read_params(&args.param_file);
        char_tuple.0.haste += 0.01 * args.weight_mult as f32;
        characters.push(char_tuple);
        let desc = format!("+{} haste", 0.01 * args.weight_mult as f32);
        descriptions.push(desc);

        // one less dagger skill
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.daggers_skill >= 300 + 1 * args.weight_mult as i16 {
            char_tuple.0.daggers_skill -= 1 * args.weight_mult as i16;
            characters.push(char_tuple);
            let desc = format!("-{} dgr skill", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more dagger skill
        char_tuple = read_params(&args.param_file);
        char_tuple.0.daggers_skill += 1 * args.weight_mult as i16;
        characters.push(char_tuple);
        let desc = format!("+{} dgr skill", 1 * args.weight_mult);
        descriptions.push(desc);
         
        // two less extra hit proc chance
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.extra_hit_proc_chance >= 0.02 * args.weight_mult as f32 {
            char_tuple.0.extra_hit_proc_chance -= 0.02 * args.weight_mult as f32;
            characters.push(char_tuple);
            let desc = format!("-{} hit proc", 0.02 * args.weight_mult as f32);
            descriptions.push(desc);
        }
        // two more extra hit proc chance
        char_tuple = read_params(&args.param_file);
        char_tuple.0.extra_hit_proc_chance += 0.02 * args.weight_mult as f32;
        characters.push(char_tuple);
        let desc = format!("+{} hit proc", 0.02 * args.weight_mult as f32);
        descriptions.push(desc);

        // TALENTS

        // one less improved eviscerate
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.improved_eviscerate >= 1 * args.weight_mult {
            char_tuple.0.improved_eviscerate -= 1 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("-{} imp evis", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more improved eviscerate
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.improved_eviscerate <= 3 - 1 * args.weight_mult {
            char_tuple.0.improved_eviscerate += 1 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("+{} imp evis", 1 * args.weight_mult);
            descriptions.push(desc);
        }

        // one less malice
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.malice >= 1 * args.weight_mult {
            char_tuple.0.malice -= 1 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("-{} malice", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more malice
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.malice <= 5 - 1 * args.weight_mult {
            char_tuple.0.malice += 1 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("+{} malice", 1 * args.weight_mult);
            descriptions.push(desc);
        }

        // one less ruthlessness
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.ruthlessness >= 1 * args.weight_mult {
            char_tuple.0.ruthlessness -= 1 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("-{} ruthlessness", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more ruthlessness
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.ruthlessness <= 3 - 1 * args.weight_mult {
            char_tuple.0.ruthlessness += 1 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("+{} ruthlessness", 1 * args.weight_mult);
            descriptions.push(desc);
        }

        // one less imp slice and dice
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.improved_slice_and_dice >= 1 * args.weight_mult {
            char_tuple.0.improved_slice_and_dice -= 1 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("-{} imp snd", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more imp slice and dice
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.improved_slice_and_dice <= 3 - 1 * args.weight_mult {
            char_tuple.0.improved_slice_and_dice += 1 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("+{} imp snd", 1 * args.weight_mult);
            descriptions.push(desc);
        }

        // one less relentless strikes
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.relentless_strikes >= 1 * args.weight_mult {
            char_tuple.0.relentless_strikes -= 1 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("-{} rel strikes", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more relentless strikes
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.relentless_strikes <= 1 - 1 * args.weight_mult {
            char_tuple.0.relentless_strikes += 1 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("+{} rel strikes", 1 * args.weight_mult);
            descriptions.push(desc);
        }

        // one less lethality
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.lethality >= 1 * args.weight_mult {
            char_tuple.0.lethality -= 1 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("-{} lethality", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more lethality
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.lethality <= 5 - 1 * args.weight_mult {
            char_tuple.0.lethality += 1 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("+{} lethality", 1 * args.weight_mult);
            descriptions.push(desc);
        }

        // one less improved backstab
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.imp_backstab >= 1 * args.weight_mult {
            char_tuple.0.imp_backstab -= 1 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("-{} imp bkstab", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more improved backstab
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.imp_backstab <= 3 - 1 * args.weight_mult {
            char_tuple.0.imp_backstab += 1 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("+{} imp bkstab", 1 * args.weight_mult);
            descriptions.push(desc);
        }

        // one less precision
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.precision >= 1 * args.weight_mult {
            char_tuple.0.precision -= 1 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("-{} precision", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more precision
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.precision <= 5 - 1 * args.weight_mult {
            char_tuple.0.precision += 1 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("+{} precision", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        
        // one less dw_specialization
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.dw_specialization >= 1 * args.weight_mult {
            char_tuple.0.dw_specialization -= 1 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("-{} dw spec", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more dw_specialization
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.dw_specialization <= 5 - 1 * args.weight_mult {
            char_tuple.0.dw_specialization += 1 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("+{} dw spec", 1 * args.weight_mult);
            descriptions.push(desc);
        }
         
        // one less sword_specialization
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.sword_specialization >= 1 * args.weight_mult {
            char_tuple.0.sword_specialization -= 1 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("-{} sword spec", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more sword_specialization
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.sword_specialization <= 5 - 1 * args.weight_mult {
            char_tuple.0.sword_specialization += 1 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("+{} sword spec", 1 * args.weight_mult);
            descriptions.push(desc);
        }
         
        // one less dagger_specialization
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.dagger_specialization >= 1 * args.weight_mult {
            char_tuple.0.dagger_specialization -= 1 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("-{} dagger spec", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more dagger_specialization
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.dagger_specialization <= 5 - 1 * args.weight_mult {
            char_tuple.0.dagger_specialization += 1 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("+{} dagger spec", 1 * args.weight_mult);
            descriptions.push(desc);
        }

        // one less weapon expertise
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.weapon_expertise >= 1 * args.weight_mult {
            char_tuple.0.weapon_expertise -= 1 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("-{} wep expert", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more weapon_expertise
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.weapon_expertise <= 2 - 1 * args.weight_mult {
            char_tuple.0.weapon_expertise += 1 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("+{} wep expert", 1 * args.weight_mult);
            descriptions.push(desc);
        }

        // one less aggression
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.aggression >= 1 * args.weight_mult {
            char_tuple.0.aggression -= 1 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("-{} aggression", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more aggression
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.aggression <= 3 - 1 * args.weight_mult {
            char_tuple.0.aggression += 1 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("+{} aggression", 1 * args.weight_mult);
            descriptions.push(desc);
        }

        // one less opportunity
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.opportunity >= 1 * args.weight_mult {
            char_tuple.0.opportunity -= 1 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("-{} opportunity", 1 * args.weight_mult);
            descriptions.push(desc);
        }
        // one more opportunity
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.opportunity <= 5 - 1 * args.weight_mult {
            char_tuple.0.opportunity += 1 * args.weight_mult;
            characters.push(char_tuple);
            let desc = format!("+{} opportunity", 1 * args.weight_mult);
            descriptions.push(desc);
        }
    } 
    return (characters, descriptions);
}

fn read_params(param_file: &String) -> (Rogue, Weapon, Weapon) {
    
    let mut param_field: u16 = 0; // to check what part the file is about
    let mut read_last = false;
    let mut rogue = Rogue::new();
    let mut mh = Weapon::new();
    let mut oh = Weapon::new();

    let f = File::open(param_file).expect("Couldn't open param_file");
    let file = BufReader::new(&f);
    for line in file.lines() {
        let l = line.unwrap();
        let first_char = l.chars().next().unwrap();
        if first_char != '#' && first_char != '@' {
            read_last = true;
            if param_field == 1 { weapon_adder(&l, &mut mh); }
            else if param_field == 2 { weapon_adder(&l, &mut oh); } 
            else { param_adder(&l, &mut rogue); }

            continue;
        }

        if read_last {
            param_field += 1;
        }
        read_last = false;
    }
    (rogue, mh, oh)
}
  
fn param_adder(text: &str, rogue: &mut Rogue) {

    let words = text.split_whitespace();
    let words_vec = words.collect::<Vec<&str>>();
    if words_vec[0] == "agility" {
        match words_vec[1].parse() {
            Ok(x) => rogue.agility = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "strength" { 
        match words_vec[1].parse() {
            Ok(x) => rogue.strength = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "crit" { 
        match words_vec[1].parse() {
            Ok(x) => rogue.crit = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "hit" { 
        match words_vec[1].parse() {
            Ok(x) => rogue.hit = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "haste" { 
        match words_vec[1].parse() {
            Ok(x) => rogue.haste = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "swords_skill" { 
        match words_vec[1].parse() {
            Ok(x) => rogue.swords_skill = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "daggers_skill" { 
        match words_vec[1].parse() {
            Ok(x) => rogue.daggers_skill = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "attack_power" { 
        match words_vec[1].parse() {
            Ok(x) => rogue.attack_power = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "extra_hit_proc_chance" { 
        match words_vec[1].parse() {
            Ok(x) => rogue.extra_hit_proc_chance = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "shadowcraft_six_bonus" { 
        rogue.shadowcraft_six_bonus = true;
    } 

    // now for talents
    else if words_vec[0] == "imp_backstab" { 
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
    } else if words_vec[0] == "nothing" {
    } else {
        panic!("Unrecognized keyword in params file: {}", words_vec[0]);
    }
}

fn weapon_adder(text: &str, wep: &mut Weapon) {

    let words = text.split_whitespace();
    let words_vec = words.collect::<Vec<&str>>();

    if words_vec[0] == "speed" {
        match words_vec[1].parse() {
            Ok(x) => wep.speed = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "min_dmg" { 
        match words_vec[1].parse() {
            Ok(x) => wep.min_dmg = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "max_dmg" { 
        match words_vec[1].parse() {
            Ok(x) => wep.max_dmg = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "enchant" { 
        match words_vec[1].parse() {
            Ok(x) => wep.enchant = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "is_offhand" { 
        match words_vec[1].parse() {
            Ok(x) => wep.is_offhand = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "is_dagger" { 
        match words_vec[1].parse() {
            Ok(x) => wep.is_dagger = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "is_sword" { 
        match words_vec[1].parse() {
            Ok(x) => wep.is_sword = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "extra_hit_proc_chance" { 
        match words_vec[1].parse() {
            Ok(x) => wep.extra_hit_proc_chance = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else {
        panic!("Unrecognized keyword in params file: {}", words_vec[0]);
    }

    if wep.enchant == "greater_striking" {
        wep.mean_dmg = 4.0 + (wep.min_dmg + wep.max_dmg) as f32 / 2.0;
    } else if wep.enchant == "superior_striking" {
        wep.mean_dmg = 5.0 + (wep.min_dmg + wep.max_dmg) as f32 / 2.0;
    } else { wep.mean_dmg = (wep.min_dmg + wep.max_dmg) as f32 / 2.0; }

}

fn calculate_hit_numbers(rogue: &mut Rogue, mh: &mut Weapon, 
                         oh: &mut Weapon, args: &Args) {

    if rogue.weapon_expertise == 1 { 
        rogue.swords_skill += 3; 
        rogue.daggers_skill += 3; 
    }
    else if rogue.weapon_expertise == 2 { 
        rogue.swords_skill += 5; 
        rogue.daggers_skill += 5; 
    }

    if mh.is_sword {
        mh.extra_hit_proc_chance += 
            0.01 * rogue.sword_specialization as f32;
    }
    if oh.is_sword {
        oh.extra_hit_proc_chance += 
            0.01 * rogue.sword_specialization as f32;
    }
    
    rogue.hit += rogue.precision as f32 * 0.01;

    set_crit_ratings(rogue, mh, oh, args.enemy_lvl);

    set_yellow_miss_chance(rogue, mh, oh, args.enemy_lvl);
    set_white_miss_chance(mh, oh);

    subtract_hit_from_miss(rogue, mh, oh, args);

    set_glancing_reduction(rogue, mh, oh, args.enemy_lvl);
    
    set_glancing_chance(rogue, args.enemy_lvl);

    set_dodge_chance(rogue, mh, oh, args.enemy_lvl);
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

fn set_crit_ratings(rogue: &mut Rogue, mh: &mut Weapon,
                    oh: &mut Weapon, enemy_lvl: i16) {

    let mut common_crit = rogue.crit; // crit directly from gear
    common_crit += 0.01 * rogue.agility as f32 / 29.0; // crit from agility
    common_crit += 0.01 * rogue.malice as f32; // talent

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
    mh.crit = common_crit;
    if mh.is_dagger { 
        mh.crit += 0.01 * rogue.dagger_specialization as f32;
        mh.crit_backstab = mh.crit;
        mh.crit_backstab += 0.1 * rogue.imp_backstab as f32; 
    }
    if mh.crit < 0.0 { mh.crit = 0.0; }
    if mh.crit_backstab < 0.0 { mh.crit = 0.0; }
    
    // give oh its crit
    oh.crit = common_crit;
    if oh.is_dagger { 
        oh.crit += 0.01 * rogue.dagger_specialization as f32;
    }
    if oh.crit < 0.0 { oh.crit = 0.0; }
    if oh.crit_backstab < 0.0 { oh.crit = 0.0; }

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
