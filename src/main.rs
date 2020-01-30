/* TODO
 * - Add poisons, scale them vs armor reduction with proper resistances.
 *   - 20% chance to apply a poison, base stat.
 *   - 20% chance for a boss to resist the poison
 *   - 130 dmg with current best instant poison
 * - Implement the option to modify stats to gain insight on how important
 *   specific stats are at different levels
 *   - maybe print to file to plot with python?
 */
extern crate rand;
extern crate clap;

use std::fs::File;
use std::io::{BufRead, BufReader};
use rand::distributions::{Distribution, Uniform};
use clap::{Arg, App};


fn main() {

    let dt = 0.01;

    let args = get_arguments();

    let mut total_extra_hits: u32 = 0;
    let mut total_mh_hits: u32 = 0;
    let mut total_oh_hits: u32 = 0;

    let mut rogue: Rogue;
    let mut wep1: Weapon;
    let mut wep2: Weapon;

    // get the different characters that will be looped through
    let character_tuples = get_characters(&args);
    let characters = character_tuples.0;
    let descriptions = character_tuples.1;
    let mut chars_dps_vectors = Vec::new();

    for (description, character) in descriptions.iter().zip(characters.iter()) {

        rogue = character.0;
        wep1 = copy_wep(&character.1);
        wep2 = copy_wep(&character.2);

        add_raid_buffs(&mut rogue);

        calculate_hit_numbers(&mut rogue, &mut wep1, &mut wep2);

        // if not going for weights, print hit chances once
        if !args.weights { print_hit_chances(&rogue, wep1.is_dagger); }

        let mut dps_vec = Vec::new();

        for _i_run in 0..args.iterations {

            let mut buffs = BuffsActive {
                blade_flurry: 0.0,
                snd: 0.0,
                adrenaline_rush: 0.0,
            };

            let mut time_struct = TimeTilEvents {
                glob_cd_refresh: 0.0,
                wep1_swing: 0.0,
                wep2_swing: 0.0,
                energy_refill: 1.0,
                fight_ends: args.fight_length
            };

            let mut extra_attacks: i8 = 0;
            let mut tot_dmg = 0.0;
            let mut tot_poison_dmg = 0.0;

            while time_struct.fight_ends > 0.0 {

                check_for_crusader(&mut rogue, &wep1, &wep2);

                if time_struct.glob_cd_refresh <= 0.0 {
                    let (dmg, dmg_poison, extra_swing) = 
                        yellow_attack(&mut rogue, &mut buffs, &wep1, 
                                      &mut time_struct, args.verb);
                    if dmg > 0.0 { 
                        tot_dmg += dmg; 
                        if wep1.enchant == "crusader" { 
                            crusader_roll(&mut wep1, args.verb); 
                        }
                        shadowcraft_roll(&mut rogue);
                        total_mh_hits += 1;
                    }
                    if dmg_poison > 0.0 { tot_poison_dmg += dmg_poison; }
                    if extra_swing { extra_attacks += 1; }
                }
                // check if oh is ready for swing
                if time_struct.wep2_swing <= 0.0 {
                    let (dmg, dmg_poison, extra_swing) = 
                        white_attack(&mut rogue, &mut wep2, 
                                     time_struct.fight_ends, args.verb);
                    if dmg > 0.0 { 
                        if wep2.enchant == "crusader" { 
                            crusader_roll(&mut wep2, args.verb); 
                        }
                        shadowcraft_roll(&mut rogue);
                        total_oh_hits += 1; 
                    }
                    if dmg_poison > 0.0 { tot_poison_dmg += dmg_poison; }

                    // Reset swing timer
                    time_struct.wep2_swing = wep2.speed;
                    let mut haste_mult = 1.0;
                    if buffs.snd > 0.0 { haste_mult -= 0.3; } 
                    if rogue.haste > 0.0 { haste_mult -= rogue.haste; }
                    time_struct.wep2_swing *= haste_mult; 
                    

                    tot_dmg += dmg;
                    if extra_swing { extra_attacks += 1; }
                }
                // check if extra swings are lined up
                while extra_attacks > 0 {
                    if args.verb { println!("Extra swing!"); }
                    total_extra_hits += 1;
                    let (dmg, dmg_poison, extra_swing) = 
                        white_attack(&mut rogue, &mut wep1, time_struct.fight_ends,
                                     args.verb);
                    if dmg > 0.0 { 
                        if wep1.enchant == "crusader" { 
                            crusader_roll(&mut wep1, args.verb); 
                        }
                        shadowcraft_roll(&mut rogue);
                        total_mh_hits += 1;
                    }
                    if dmg_poison > 0.0 { tot_poison_dmg += dmg_poison; }

                    // Reset swing timer for MH
                    time_struct.wep1_swing = wep1.speed;
                    let mut haste_mult = 1.0;
                    if buffs.snd > 0.0 { haste_mult -= 0.3; } 
                    if rogue.haste > 0.0 { haste_mult -= rogue.haste; }
                    time_struct.wep1_swing *= haste_mult; 

                    tot_dmg += dmg;
                    if !extra_swing {
                        extra_attacks -= 1;
                    }
                }

                // check if mh is ready for swing
                if time_struct.wep1_swing <= 0.0 {
                    let (dmg, dmg_poison, extra_swing) = 
                        white_attack(&mut rogue, &mut wep1, time_struct.fight_ends,
                                     args.verb);
                    if dmg > 0.0 { 
                        if wep1.enchant == "crusader" { 
                            crusader_roll(&mut wep1, args.verb); 
                        }
                        shadowcraft_roll(&mut rogue);
                        total_mh_hits += 1; 
                    }
                    if dmg_poison > 0.0 { tot_poison_dmg += dmg_poison; }
                    
                    // Reset swing timer
                    time_struct.wep1_swing = wep1.speed;
                    let mut haste_mult = 1.0;
                    if buffs.snd > 0.0 { haste_mult -= 0.3; } 
                    if rogue.haste > 0.0 { haste_mult -= rogue.haste; }
                    time_struct.wep1_swing *= haste_mult; 


                    if extra_swing { extra_attacks += 1; }
                    tot_dmg += dmg;
                }
                subtract_times(&mut rogue, &mut time_struct, &mut buffs,
                               &mut wep1, &mut wep2, dt);
            }
            //
            // armor reduction
            tot_dmg = armor_reduction(tot_dmg);
            let all_dmg = tot_dmg + tot_poison_dmg;

            if args.verb {
                println!("\nDps during {:} seconds was {:}.", args.fight_length, 
                         all_dmg/args.fight_length);
                println!("Total number of mh/of hits: {}/{}.", total_mh_hits, 
                         total_oh_hits);
                println!("Total number of extra hits: {}.", total_extra_hits);
            }

            // store dps of run
            dps_vec.push(all_dmg/args.fight_length);
        }

        chars_dps_vectors.push(mean(&dps_vec));

        if !args.weights {
            println!("Average dps for {} over {} runs was {:}.", 
                     args.param_file, args.iterations, mean(&dps_vec));
        } else {
            if description == "base" {
                println!("\nAnalysis of stat weights based on char in file: {}", 
                         args.param_file);
                println!("Length of each fight: {}", 
                         args.fight_length);
                println!("Nr of iterations per variation: {}\n", 
                         args.iterations);
                println!("{:_<12}:  {:}", description, mean(&dps_vec));
            } else {
                println!("{:_<12}: {:+.5}", description, 
                         mean(&dps_vec) - chars_dps_vectors[0]);
            }
        }
    }
}

fn check_for_crusader(rogue: &mut Rogue, wep1: &Weapon, wep2: &Weapon) {

    if wep1.crusader > 0.0 && wep2.crusader > 0.0 {
        rogue.nr_crusaders_active = 2.0;
    } else if wep1.crusader > 0.0 || wep2.crusader > 0.0 {
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
        .arg(Arg::with_name("file") 
             .required(true)
             .short("f") 
             .long("file").takes_value(true) 
             .help("Parameter file that contains all rogue traits."))
        .arg(Arg::with_name("iterations") 
             .short("i") 
             .long("iterations").takes_value(true) 
             .help("Number of iterations to average over."))
        .arg(Arg::with_name("fight_length") 
             .short("t") 
             .long("time").takes_value(true) 
             .help("Seconds of duration per fight."))
        .arg(Arg::with_name("weights") 
            .short("w") 
            .long("weights") 
            .takes_value(false) 
            .help("Permute stats/talents slightly to get dps values."))
        .arg(Arg::with_name("verbose") 
            .short("v") 
            .long("verbose") 
            .takes_value(false) 
            .help("Be verbose, print details about fights."))
        .get_matches();

    let file = matches.value_of("file").unwrap();
    let iterations = matches.value_of("iterations").unwrap_or("10_000");
    let fight_length = matches.value_of("fight_length").unwrap_or("60");
    let weights = matches.is_present("weights");
    let verb = matches.is_present("verbose");


    let mut args: Args = default_args();
    args.param_file = file.to_string();
    args.verb = verb;
    args.weights = weights;
    args.iterations = iterations.parse().unwrap();
    let fl: u16 = fight_length.parse().unwrap();
    args.fight_length = fl as f32;

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

fn print_hit_chances(rogue: &Rogue, mh_is_dagger: bool) {

    let mut crit: f32;
    let white_miss: f32;
    let yellow_miss: f32;
    let dodge: f32;
    if mh_is_dagger { 
        crit = rogue.crit + 0.01 * rogue.dagger_specialization as f32;
        white_miss = rogue.white_miss_daggers;
        yellow_miss = rogue.yellow_miss_daggers;
        dodge = rogue.dodge_daggers;
    } else { 
        crit = rogue.crit; 
        white_miss = rogue.white_miss_swords;
        yellow_miss = rogue.yellow_miss_swords;
        dodge = rogue.dodge_swords;
    }
    println!("*** White hits summary ***");
    println!("miss chance: {:}", white_miss);
    println!("dodge chance: {:}", dodge);
    println!("glancing chance: {:}", rogue.glancing);
    println!("crit chance: {:}", crit);
    let mut tmp = white_miss;
    let mut tmp1 = tmp + dodge;
    let mut tmp2 = tmp1 + rogue.glancing;
    let     tmp3 = tmp2 + crit;
    println!("{:}-{:}-{:}-{:}\n", tmp, tmp1, tmp2, tmp3);
    
    println!("*** Yellow hits summary ***");
    println!("miss chance: {:}", yellow_miss);
    println!("dodge chance: {:}", dodge);
    println!("crit chance: {:}", crit);
    tmp = yellow_miss;
    tmp1 = tmp + dodge;
    tmp2 = tmp1 + crit;
    println!("{:}-{:}-{:}\n", tmp, tmp1, tmp2);

    if mh_is_dagger {
        crit += 0.1 * rogue.imp_backstab as f32; 
        println!("*** Backstab summary ***");
        println!("miss chance: {:}", yellow_miss);
        println!("dodge chance: {:}", dodge);
        println!("crit chance: {:}", crit);
        tmp = yellow_miss;
        tmp1 = tmp + dodge;
        tmp2 = tmp1 + crit;
        println!("{:}-{:}-{:}\n", tmp, tmp1, tmp2);
    }
}

struct Args {
    verb: bool,
    iterations: u32,
    param_file: String,
    weights: bool,
    fight_length: f32
}

fn default_args() -> Args {
    let args = Args {
        verb: false,
        iterations: 1000,
        param_file: "".to_string(),
        weights: false,
        fight_length: 60.0
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

fn determine_hit(rogue: &Rogue, is_dagger: bool, color: String, 
                 is_backstab: bool) -> String {

    let roll: f32 = roll_die();
    let mut percent_sum: f32;

    if color == "yellow" {

        if is_dagger {
            if roll < rogue.yellow_miss_daggers { return "miss".to_string(); }
            percent_sum = rogue.yellow_miss_daggers + rogue.dodge_daggers;
            if roll < percent_sum { return "dodge".to_string(); }
            percent_sum += rogue.crit 
                + 0.01 * rogue.dagger_specialization as f32; 
            if is_backstab {
                percent_sum += 0.1 * rogue.imp_backstab as f32; 
            }
            if roll < percent_sum { return "crit".to_string(); }

            return "hit".to_string();

        } else {
            if roll < rogue.yellow_miss_swords { return "miss".to_string(); }
            percent_sum = rogue.yellow_miss_swords + rogue.dodge_swords;
            if roll < percent_sum { return "dodge".to_string(); }
            percent_sum += rogue.crit;
            if roll < percent_sum { return "crit".to_string(); }

            return "hit".to_string();
        }


    } else if color == "white" {

        if is_dagger {
            if roll < rogue.white_miss_daggers { return "miss".to_string(); }
            let mut percent_sum = rogue.white_miss_daggers 
                + rogue.dodge_daggers;
            if roll < percent_sum { return "dodge".to_string(); }
            percent_sum += rogue.glancing;
            if roll < percent_sum { return "glancing".to_string(); }
            percent_sum += rogue.crit 
                + 0.01 * rogue.dagger_specialization as f32;
            if roll < percent_sum { return "crit".to_string(); }
            return "hit".to_string();
        } else {
            if roll < rogue.white_miss_swords { return "miss".to_string(); }
            let mut percent_sum = rogue.white_miss_swords 
                + rogue.dodge_swords;
            if roll < percent_sum { return "dodge".to_string(); }
            percent_sum += rogue.glancing;
            if roll < percent_sum { return "glancing".to_string(); }
            percent_sum += rogue.crit;
            if roll < percent_sum { return "crit".to_string(); }
            return "hit".to_string();
        }

    } else { panic!("can only strike yellow or white hits"); }

}

fn backstab(rogue: &mut Rogue, wep: &Weapon, 
            time_struct: &TimeTilEvents, verb: bool) -> f32 {

    let hit_result = determine_hit(&rogue, wep.is_dagger, "yellow".to_string(),
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

    if verb {
        announce_hit(dmg, "backstab".to_string(), hit_result, 
                     time_struct.fight_ends);
    }

    return dmg;
}

fn sinister_strike(rogue: &mut Rogue, wep: &Weapon, 
                   time_struct: &TimeTilEvents, verb: bool) -> f32 {

    let hit_result = determine_hit(&rogue, wep.is_dagger, "yellow".to_string(),
        false);
    let mut dmg: f32 = 0.0;

    if hit_result == "miss" || hit_result == "dodge" {
        rogue.energy -= 8; //todo fix this
        dmg = 0.0;

    } else if hit_result == "glancing" {
        rogue.energy -= 40;
        dmg = get_sinister_strike_dmg(&wep, &rogue);
        if wep.is_dagger { dmg *= 1.0 - rogue.glancing_red_daggers; }
        else { dmg *= 1.0 - rogue.glancing_red_swords; }
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

    return dmg;
}

fn eviscerate(rogue: &mut Rogue, wep: &Weapon, time_struct: &TimeTilEvents,
              verb: bool) -> f32 {

    let hit_result = determine_hit(&rogue, wep.is_dagger, 
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
    if verb {
        announce_hit(dmg, "evis".to_string(), hit_result, 
                     time_struct.fight_ends);
    }
    return dmg;
}

fn yellow_attack(rogue: &mut Rogue, mut buffs: &mut BuffsActive,
                 wep: &Weapon, 
                 mut time_struct: &mut TimeTilEvents,
                 verb: bool) -> (f32, f32, bool) {
    // returns dmg and a true bool if an extra attack was triggered
    
    let mut dmg = 0.0;
    let mut dmg_poison = 0.0;
    let mut extra_hit: bool = false;
    
    let can_sinister = rogue.energy >= 40;
    let can_backstab = rogue.energy >= 60;
    let can_eviscerate = rogue.energy >= 35;
    let can_snd = rogue.energy >= 25;
    let snd_active = buffs.snd > 0.0;

    // Short snd if no snd up at 2 combo points
    if rogue.combo_points == 2 && can_snd && !snd_active {
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

fn get_glancing_reduction(wep_skill: u16) -> f32 {
    if wep_skill == 305 { return 0.15; }
    else if wep_skill == 306 { return 0.11; }
    else if wep_skill == 307 { return 0.07; }
    else if wep_skill >= 308 && wep_skill <= 315 { return 0.05; }
    else { panic!("weapon skill not implemented"); }
}

fn get_dodge_chance(wep_skill: u16) -> f32 {
    let dodge_chance = 0.05 + (315 - wep_skill) as f32 * 0.001;
    return dodge_chance;
}

fn get_yellow_miss_chance(wep_skill: u16) -> f32 {
    let miss_chance = 0.05 + (315 - wep_skill) as f32 * 0.001;
    return miss_chance;
}

fn get_white_miss_chance(wep_skill: u16) -> f32 {
    let yellow_miss_chance = get_yellow_miss_chance(wep_skill);
    let miss_chance = 0.8 * yellow_miss_chance + 0.2;
    return miss_chance;
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

fn white_attack(rogue: &mut Rogue, wep: &mut Weapon, 
                time_left: f32, verb: bool) -> (f32, f32, bool) {
    // returns damage and a bool that is true if an extra swing procced

    let hit_result = determine_hit(&rogue, wep.is_dagger, "white".to_string(),
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
    if hit_result == "glancing" { 
        if wep.is_dagger { dmg *= 1.0 - rogue.glancing_red_daggers; }
        else { dmg *= 1.0 - rogue.glancing_red_swords; }
    } else if hit_result == "crit" { 
        dmg *= 2.0;
    }
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
    let x = 0.1 * 3731.0 / (8.5 * 60.0 + 40.0);
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
    extra_hit_proc_chance: f32
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
    swords_skill: u16,
    daggers_skill: u16,
    dodge_swords: f32,
    dodge_daggers: f32,
    white_miss_swords: f32,
    white_miss_daggers: f32,
    yellow_miss_swords: f32,
    yellow_miss_daggers: f32,
    glancing: f32,
    glancing_red_swords: f32,
    glancing_red_daggers: f32,
    extra_hit_proc_chance: f32,
    shadowcraft_six_bonus: bool,
    imp_sin_strike: u8,
    imp_backstab: u8,
    precision: u8,
    dw_specialization: u8,
    sword_specialization: u8,
    dagger_specialization: u8,
    weapon_expertise: u8,
    aggression: u8,
    opportunity: u8,
    improved_eviscerate: u8,
    malice: u8,
    ruthlessness: u8,
    improved_slice_and_dice: u8,
    relentless_strikes: u8,
    lethality: u8,
    combo_points: u16
}

struct BuffsActive {
    blade_flurry: f32,
    snd: f32,
    adrenaline_rush: f32
}

struct TimeTilEvents {
    glob_cd_refresh: f32,
    wep1_swing: f32,
    wep2_swing: f32,
    energy_refill: f32,
    fight_ends: f32
}

fn deb<T: std::fmt::Debug>(x: T) {
    println!("{:?}", x);
}

fn subtract_times(mut rogue: &mut Rogue, 
                  mut time_struct: &mut TimeTilEvents, 
                  mut buffs: &mut BuffsActive, wep1: &mut Weapon, 
                  wep2: &mut Weapon, dt: f32) {

    if time_struct.glob_cd_refresh > 0.0 {
        time_struct.glob_cd_refresh -= dt;
    } 
    if time_struct.wep1_swing > 0.0 {
        time_struct.wep1_swing -= dt;
    } 
    if time_struct.wep2_swing > 0.0 { 
        time_struct.wep2_swing -= dt; 
    }

    if wep1.crusader > 0.0 { 
        wep1.crusader -= dt; 
    }
    if wep2.crusader > 0.0 { 
        wep2.crusader -= dt; 
    }

    time_struct.energy_refill -= dt;
    if time_struct.energy_refill <= 0.0 { 
        let energy_increase: i8;
        let die = roll_die();
        if die < 0.25 { energy_increase = 21; }
        else { energy_increase = 20; }

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

fn init_rogue() -> Rogue {

    let rogue = Rogue {
        energy: 100,
        agility: 0,
        strength: 0,
        attack_power: 0, // IMPORTANT: just attack power given directly by gear
        crit: 0.0,
        hit: 0.0,
        haste: 0.0,
        nr_crusaders_active: 0.0,
        dodge_swords: 0.0,
        dodge_daggers: 0.0,
        white_miss_swords: 0.0,
        white_miss_daggers: 0.0,
        yellow_miss_swords: 0.0,
        yellow_miss_daggers: 0.0,
        glancing: 0.40,
        glancing_red_swords: 0.0,
        glancing_red_daggers: 0.0,
        swords_skill: 0,
        daggers_skill: 0,
        extra_hit_proc_chance: 0.0, // NOTE does not include thrash blade proc
        shadowcraft_six_bonus: false,
        imp_sin_strike: 0,
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
    };
    return rogue;
}

fn init_weapon() -> Weapon {

    let wep = Weapon {
        speed: 0.0,
        max_dmg: 0,
        min_dmg: 0,
        mean_dmg: 0.0,
        enchant: "none".to_string(),
        crusader: 0.0,
        is_offhand: false,
        is_dagger: false,
        is_sword: false,
        extra_hit_proc_chance: 0.0
    };
    return wep;
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
        char_tuple.0.agility -= 10;
        characters.push(char_tuple);
        descriptions.push("-10 agi".to_string());
        
        // one more agility
        char_tuple = read_params(&args.param_file);
        char_tuple.0.agility += 10;
        characters.push(char_tuple);
        descriptions.push("+10 agi".to_string());
        
        // ten less strength
        char_tuple = read_params(&args.param_file);
        char_tuple.0.strength -= 10;
        characters.push(char_tuple);
        descriptions.push("-10 str".to_string());
        // ten more strength
        char_tuple = read_params(&args.param_file);
        char_tuple.0.strength += 10;
        characters.push(char_tuple);
        descriptions.push("+10 str".to_string());
         
        // ten less attack power
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.attack_power >= 10 {
            char_tuple.0.attack_power -= 10;
            characters.push(char_tuple);
            descriptions.push("-10 atp".to_string());
        }
        // ten more attack power
        char_tuple = read_params(&args.param_file);
        char_tuple.0.attack_power += 10;
        characters.push(char_tuple);
        descriptions.push("+10 atp".to_string());
        
        // one less hit
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.hit >= 0.01 {
            char_tuple.0.hit -= 0.01;
            characters.push(char_tuple);
            descriptions.push("-1 hit".to_string());
        }
        // one more hit
        char_tuple = read_params(&args.param_file);
        char_tuple.0.hit += 0.01;
        characters.push(char_tuple);
        descriptions.push("+1 hit".to_string());

        // one less crit
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.crit >= 0.01 {
            char_tuple.0.crit -= 0.01;
            characters.push(char_tuple);
            descriptions.push("-1 crit".to_string());
        }
        // one more crit
        char_tuple = read_params(&args.param_file);
        char_tuple.0.crit += 0.01;
        characters.push(char_tuple);
        descriptions.push("+1 crit".to_string());
         
        // one less haste
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.haste >= 0.01 {
            char_tuple.0.haste -= 0.01;
            characters.push(char_tuple);
            descriptions.push("-1 haste".to_string());
        }
        // one more haste
        char_tuple = read_params(&args.param_file);
        char_tuple.0.haste += 0.01;
        characters.push(char_tuple);
        descriptions.push("+1 haste".to_string());

        // one less dagger skill
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.daggers_skill > 300 {
            char_tuple.0.daggers_skill -= 1;
            characters.push(char_tuple);
            descriptions.push("-1 dgr_skill".to_string());
        }
        // one more dagger skill
        char_tuple = read_params(&args.param_file);
        char_tuple.0.daggers_skill += 1;
        characters.push(char_tuple);
        descriptions.push("+1 dgr_skill".to_string());
         
        // two less extra hit proc chance
        char_tuple = read_params(&args.param_file);
        if char_tuple.0.extra_hit_proc_chance >= 0.02 {
            char_tuple.0.extra_hit_proc_chance -= 0.02;
            characters.push(char_tuple);
            descriptions.push("-2 hit proc".to_string());
        }
        // two more extra hit proc chance
        char_tuple = read_params(&args.param_file);
        char_tuple.0.extra_hit_proc_chance += 0.02;
        characters.push(char_tuple);
        descriptions.push("+2 hit proc".to_string());
    } 
    return (characters, descriptions);
}

fn read_params(param_file: &String) -> (Rogue, Weapon, Weapon) {
    
    let mut param_field: u8 = 0; // to check what part the file is about
    let mut read_last = false;
    let mut rogue: Rogue = init_rogue();
    let mut wep1: Weapon = init_weapon();
    let mut wep2: Weapon = init_weapon();

    let f = File::open(param_file).expect("Couldn't open param_file");
    let file = BufReader::new(&f);
    for line in file.lines() {
        let l = line.unwrap();
        let first_char = l.chars().next().unwrap();
        if first_char != '#' && first_char != '@' {
            read_last = true;
            if param_field == 1 { weapon_adder(&l, &mut wep1); }
            else if param_field == 2 { weapon_adder(&l, &mut wep2); } 
            else { param_adder(&l, &mut rogue); }

            continue;
        }

        if read_last {
            param_field += 1;
        }
        read_last = false;
    }
    (rogue, wep1, wep2)
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
    else if words_vec[0] == "imp_sin_strike" { 
        match words_vec[1].parse() {
            Ok(x) => rogue.imp_sin_strike = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "imp_backstab" { 
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

fn get_agi_crit_chance(agi: u16) -> f32 {
    // this function calculates the contribution to crit from agility alone
    // rogues get 1% crit per 29 agility, according to blizzard
    let extra_crit = 0.01 * agi as f32 / 29.0;
    return extra_crit;
}

fn calculate_hit_numbers(rogue: &mut Rogue, wep1: &mut Weapon, 
                         wep2: &mut Weapon) {

    if rogue.weapon_expertise == 1 { 
        rogue.swords_skill += 3; 
        rogue.daggers_skill += 3; 
    }
    else if rogue.weapon_expertise == 2 { 
        rogue.swords_skill += 5; 
        rogue.daggers_skill += 5; 
    }

    if wep1.is_sword {
        wep1.extra_hit_proc_chance += 
            0.01 * rogue.sword_specialization as f32;
    }
    if wep2.is_sword {
        wep2.extra_hit_proc_chance += 
            0.01 * rogue.sword_specialization as f32;
    }
    
    rogue.hit += rogue.precision as f32 * 0.01;
    rogue.dodge_daggers = get_dodge_chance(rogue.daggers_skill);
    rogue.dodge_swords = get_dodge_chance(rogue.swords_skill);

    rogue.crit += get_agi_crit_chance(rogue.agility);
    rogue.crit += 0.01 * rogue.malice as f32;
    // 1.8 crit is removed from non-agi crit. Assumed here that the rogue
    // has at least 2% crit gained directly from gear
    // + 3% crit reduction vs bosses brings the crit down a total of 4.8
    if rogue.crit < 0.048 { rogue.crit = 0.0; }
    else { rogue.crit -= 0.048; }

    rogue.white_miss_swords = get_white_miss_chance(rogue.swords_skill);
    rogue.white_miss_daggers = get_white_miss_chance(rogue.daggers_skill);
    if rogue.hit > rogue.white_miss_swords { rogue.white_miss_swords = 0.0; }
    else { rogue.white_miss_swords -= rogue.hit; }
    if rogue.hit > rogue.white_miss_daggers { rogue.white_miss_daggers = 0.0; }
    else { rogue.white_miss_daggers -= rogue.hit; }

    rogue.yellow_miss_swords = get_yellow_miss_chance(rogue.swords_skill);
    rogue.yellow_miss_daggers = get_yellow_miss_chance(rogue.swords_skill);
    if rogue.hit > rogue.yellow_miss_swords { rogue.yellow_miss_swords = 0.0; }
    else { rogue.yellow_miss_swords -= rogue.hit; }
    if rogue.hit > rogue.yellow_miss_daggers { 
        rogue.yellow_miss_daggers = 0.0;
    }
    else { rogue.yellow_miss_daggers -= rogue.hit; }

    rogue.glancing_red_swords = get_glancing_reduction(rogue.swords_skill);
    rogue.glancing_red_daggers = get_glancing_reduction(rogue.daggers_skill);
}
 
fn mean(numbers: &Vec<f32>) -> f32 {

    let mut sum: f64 = 0.0;
    for n in numbers.iter() { sum += *n as f64; }

    let avg = sum / numbers.len() as f64;
    return avg as f32;
}
