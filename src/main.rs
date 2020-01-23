/* TODO
 * - implement 21 energy increase sometimes (every fourth tic, roughly)
 * - add buff support, BoK comes after all other buffs
 * - Insert correct values for mirah's song and thrash blade.
 * - Implement boss crit reduction 
 * - Implement full formula for calculating crit chance.
 * - Implement CORRECT hit calculation
 * - Implement the option to modify stats to gain insight on how important
 *   specific stats are at different levels
 *   - maybe print to file to plot with python?
 */
extern crate rand;

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use rand::distributions::{Distribution, Uniform};


fn main() {

    let n_runs = 1000;
    let fight_length = 60.0;
    let mut verb = false;
    let dt = 0.1;

    let args: Vec<String> = env::args().collect();
    if args.len() == 1 { 
        println!("Usage: supply one argument, the param file followed by a -v for verbose, if wanted.");
    }

    let param_file = &args[1];

    let mut total_extra_hits: u16 = 0;
    let mut total_mh_hits: u16 = 0;
    let mut total_oh_hits: u16 = 0;

    let mut character: Rogue;
    let mut wep1: Weapon;
    let mut wep2: Weapon;
    let char_tuple: (Rogue, Weapon, Weapon) = read_params(param_file);
    character = char_tuple.0;
    wep1 = char_tuple.1;
    wep2 = char_tuple.2;

    calculate_hit_numbers(&mut character);

    print_hit_chances(&character);

    let mut dps_vec = Vec::new();

    for i_run in 0..n_runs {

        let mut buffs = BuffsActive {
            blade_flurry: 0.0,
            snd: 0.0,
            adrenaline_rush: 0.0
        };

        let mut time_struct = TimeTilEvents {
            glob_cd_refresh: 0.0,
            wep1_swing: 0.0,
            wep2_swing: 0.0,
            energy_refill: 0.0,
            fight_ends: fight_length
        };

        let mut extra_attacks: i8 = 0;
        let mut tot_dmg = 0.0;

        while time_struct.fight_ends > 0.0 {
            if time_struct.glob_cd_refresh <= 0.0 {
                let (dmg, extra_swing) = yellow_attack(&mut character, &mut buffs,
                                                       &wep1, &mut time_struct,
                                                       verb);
                if dmg > 0.0 { 
                    tot_dmg += dmg; 
                    total_mh_hits += 1;
                }
                if extra_swing { extra_attacks += 1; }
            }
            // check if oh is ready for swing
            if time_struct.wep2_swing <= 0.0 {
                let (dmg, extra_swing) = white_attack(&mut character, &mut wep2, 
                                                      time_struct.fight_ends,
                                                      verb);
                if dmg > 0.0 { total_oh_hits += 1; }
                if buffs.snd > 0.0 {
                    time_struct.wep2_swing = wep2.speed * 0.7;
                } else {
                    time_struct.wep2_swing = wep2.speed;
                }
                tot_dmg += dmg;
                if extra_swing { extra_attacks += 1; }
            }
            // check if extra swings are lined up
            while extra_attacks > 0 {
                if verb { println!("Extra swing!"); }
                total_extra_hits += 1;
                total_mh_hits += 1;
                let (dmg, extra_swing) = white_attack(&mut character, &mut wep1, 
                                                      time_struct.fight_ends,
                                                      verb);
                // reset swing timer for MH
                if buffs.snd > 0.0 {
                    time_struct.wep1_swing = wep1.speed * 0.7;
                } else {
                    time_struct.wep1_swing = wep1.speed;
                }
                tot_dmg += dmg;
                if !extra_swing {
                    extra_attacks -= 1;
                }
            }

            // check if mh is ready for swing
            if time_struct.wep1_swing <= 0.0 {
                let (dmg, extra_swing) = white_attack(&mut character, 
                                                      &mut wep1, 
                                                      time_struct.fight_ends,
                                                      verb);
                if dmg > 0.0 { total_mh_hits += 1; }
                if buffs.snd > 0.0 {
                    time_struct.wep1_swing = wep1.speed * 0.7;
                } else {
                    time_struct.wep1_swing = wep1.speed;
                }

                if extra_swing { extra_attacks += 1; }
                tot_dmg += dmg;
            }
            subtract_times(&mut character, &mut time_struct, &mut buffs, dt);
        }
        if verb {
            println!("\nDps during {:} seconds was {:}.", fight_length, 
                     tot_dmg/fight_length);
            println!("Total number of mh/of hits: {}/{}.", total_mh_hits, 
                     total_oh_hits);
            println!("Total number of extra hits: {}.", total_extra_hits);
        }

        // store dps of run
        dps_vec.push(tot_dmg/fight_length);
    }

    println!("Average dps for {} was {:}.", param_file, mean(&dps_vec));
}

fn print_hit_chances(character: &Rogue) {

    println!("*** White hits summary ***");
    println!("miss chance: {:}", character.white_miss);
    println!("dodge chance: {:}", character.dodge);
    println!("glancing chance: {:}", character.glancing);
    println!("crit chance: {:}", character.crit);
    let mut tmp = character.white_miss;
    let mut tmp1 = tmp + character.dodge;
    let mut tmp2 = tmp1 + character.glancing;
    let mut tmp3 = tmp2 + character.crit;
    println!("{:}-{:}-{:}-{:}\n", tmp, tmp1, tmp2, tmp3);
    
    println!("*** Yellow hits summary ***");
    println!("miss chance: {:}", character.yellow_miss);
    println!("dodge chance: {:}", character.dodge);
    println!("glancing chance: {:}", character.glancing);
    println!("crit chance: {:}", character.crit);
    tmp = character.yellow_miss;
    tmp1 = tmp + character.dodge;
    tmp2 = tmp1 + character.glancing;
    tmp3 = tmp2 + character.crit;
    println!("{:}-{:}-{:}-{:}\n", tmp, tmp1, tmp2, tmp3);
}

fn announce_hit(dmg: f32, attack_type: String, hit_type: String, time: f32) {
    if attack_type == "sin_strike" {
        println!("{:2.1}: Sinister strike {} for {:.0}", time, hit_type, dmg);
    } else if attack_type == "evis" {
        println!("{:2.1}: Eviscerate {} for {:.0}", time, hit_type, dmg);
    } else if attack_type == "mh_white" {
        println!("{:2.1}: MH white {} for {:.0}", time, hit_type, dmg);
    } else if attack_type == "oh_white" {
        println!("{:2.1}: OH white {} for {:.0}", time, hit_type, dmg);
    } else if attack_type == "snd" {
        println!("{:2.1}: Slice and dice applied for {:.2}s", time, dmg);
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

fn determine_hit(character: &Rogue, color: String) -> String {

    let roll: f32 = roll_die();
    // println!("rolled {:}", roll);

    if color == "yellow" {

        if roll < character.yellow_miss { return "miss".to_string(); }
        let mut percent_sum = character.yellow_miss + character.dodge;
        if roll < percent_sum { return "dodge".to_string(); }
        percent_sum += character.glancing;
        if roll < percent_sum { return "glancing".to_string(); }
        percent_sum += character.crit;
        if roll < percent_sum { return "crit".to_string(); }
        return "hit".to_string();

    } else if color == "white" {

        if roll < character.white_miss { return "miss".to_string(); }
        let mut percent_sum = character.white_miss + character.dodge;
        if roll < percent_sum { return "dodge".to_string(); }
        percent_sum += character.glancing;
        if roll < percent_sum { return "glancing".to_string(); }
        percent_sum += character.crit;
        if roll < percent_sum { return "crit".to_string(); }
        return "hit".to_string();

    } else { panic!("can only strike yellow or white hits"); }

}

fn sinister_strike(character: &mut Rogue, wep: &Weapon, 
                   time_struct: &TimeTilEvents, verb: bool) -> f32 {

    let hit_result = determine_hit(&character, "yellow".to_string());
    let mut dmg: f32 = 0.0;

    if hit_result == "miss" {
        character.energy -= 5; //todo fix this
        dmg = 0.0;

    } else if hit_result == "dodge" {
        character.energy -= 5; //todo fix this
        dmg = 0.0;

    } else if hit_result == "glancing" {
        character.energy -= 40;
        dmg = get_sinister_strike_dmg(&wep, &character);
        dmg *= character.glancing_red;
        character.combo_points += 1;

    } else if hit_result == "crit" {
        character.energy -= 40;
        dmg = get_sinister_strike_dmg(&wep, &character);
        dmg *= 2.0;
        character.combo_points += 1;

    } else if hit_result == "hit" {
        character.energy -= 40;
        dmg = get_sinister_strike_dmg(&wep, &character);
        character.combo_points += 1;
    }
    if verb {
        announce_hit(dmg, "sin_strike".to_string(), hit_result, 
                     time_struct.fight_ends);
    }
    return dmg;
}

fn eviscerate(character: &mut Rogue, time_struct: &TimeTilEvents,
              verb: bool) -> f32 {

    let hit_result = determine_hit(&character, "yellow".to_string());
    let mut dmg: f32 = 0.0;

    if hit_result == "miss" || hit_result == "dodge" {
        dmg = 0.0;

    } else if hit_result == "glancing" {
        character.energy -= 35;
        dmg = get_evis_dmg(character);
        dmg *= character.glancing_red;
        character.combo_points = 0;

    } else if hit_result == "crit" {
        character.energy -= 35;
        dmg = get_evis_dmg(character);
        dmg *= 2.0;
        character.combo_points = 0;

    } else if hit_result == "hit" {
        dmg = get_evis_dmg(character);
        character.combo_points = 0;
    }
    character.energy -= 35;
    if verb {
        announce_hit(dmg, "evis".to_string(), hit_result, 
                     time_struct.fight_ends);
    }
    return dmg;
}

fn yellow_attack(character: &mut Rogue, mut buffs: &mut BuffsActive,
                 wep: &Weapon, 
                 mut time_struct: &mut TimeTilEvents,
                 verb: bool) -> (f32, bool) {
    // returns dmg and a true bool if an extra attack was triggered
    
    let mut dmg = 0.0;
    let mut extra_hit: bool = false;
    
    let can_sinister = character.energy >= 40;
    let can_eviscerate = character.energy >= 35;
    let can_snd = character.energy >= 25;
    let snd_active = buffs.snd > 0.0;

    // Short snd if no snd up at 2 combo points
    if character.combo_points == 2 && can_snd && !snd_active {
        character.energy -= 25;
        buffs.snd = snd_duration(character.combo_points);
        time_struct.glob_cd_refresh = 1.0;
        character.combo_points = 0;
        if verb {
            announce_hit(buffs.snd, "snd".to_string(), "snd".to_string(), 
                         time_struct.fight_ends);
        }
    // Sinister strike if not yet at 5 combo points
    } else if character.combo_points < 5 && can_sinister {
        dmg = sinister_strike(character, wep, &time_struct, verb);
        if dmg > 0.0 {
            extra_hit = roll_for_extra_hit(character, wep);
        }
        time_struct.glob_cd_refresh = 1.0;
        if character.combo_points > 5 { character.combo_points = 5; }

    // Long snd if no snd up at 5 combo points
    } else if character.combo_points == 5 && can_snd && !snd_active {
        character.energy -= 25;
        buffs.snd = snd_duration(character.combo_points);
        time_struct.glob_cd_refresh = 1.0;
        character.combo_points = 0;
        if verb {
            announce_hit(buffs.snd, "snd".to_string(), "snd".to_string(),
                         time_struct.fight_ends);
        }
    // Full eviscerate at 5 combo points if snd is up
    } else if character.combo_points == 5 && snd_active && can_eviscerate { 
        dmg = eviscerate(character, &time_struct, verb);
        if dmg > 0.0 {
            extra_hit = roll_for_extra_hit(character, wep);
        }
        time_struct.glob_cd_refresh = 1.0;
    }
    return (dmg, extra_hit);
}

fn roll_for_extra_hit(character: &mut Rogue, wep: &Weapon) -> bool {
    let die = roll_die();
    if die < character.extra_hit_proc_chance + wep.extra_hit_proc_chance {
        return true;
    } else { return false; }
}

fn get_glancing_reduction(wep_skill: u16) -> f32 {
    if wep_skill == 305 { return 0.15; }
    else if wep_skill == 306 { return 0.11; }
    else if wep_skill == 307 { return 0.07; }
    else if wep_skill == 308 { return 0.05; }
    else if wep_skill == 309 { return 0.05; }
    else if wep_skill == 310 { return 0.05; }
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

fn get_total_attack_power(character: &Rogue) -> f32 {
    let attack_power = 100 + character.agility + character.strength 
        + character.attack_power;
    return attack_power as f32;
}

fn get_wep_dmg(wep: &Weapon, character: &Rogue) -> f32 {

    let attack_power = get_total_attack_power(&character);
    let dmg = wep.mean_dmg + attack_power * wep.speed / 14.0;
    return dmg;
}

fn get_sinister_strike_dmg(wep: &Weapon, character: &Rogue) -> f32 {
    let normal_wep_dmg = get_wep_dmg(&wep, &character);
    let dmg = normal_wep_dmg + 68.0;
    return dmg;
}

fn snd_duration(combo_points: u16) -> f32 {

    let mut dur: f32 = 0.0;
    if combo_points == 1 { dur = 9.0; }
    if combo_points == 2 { dur = 12.0; }
    if combo_points == 3 { dur = 15.0; }
    if combo_points == 4 { dur = 18.0; }
    if combo_points == 5 { dur = 21.0; }
    dur *= 1.3;

    return dur;
}

fn get_evis_dmg(character: &mut Rogue) -> f32 {
    let mut dmg: f32 ;
    if character.combo_points == 1 { dmg = 247.0; }
    else if character.combo_points == 2 { dmg = 398.0; }
    else if character.combo_points == 3 { dmg = 549.0; }
    else if character.combo_points == 4 { dmg = 700.0; }
    else if character.combo_points == 5 { dmg = 851.0; }
    else { panic!("Invalid nr of combo points in get_evis_dmg"); }

    let attack_power = get_total_attack_power(&character);
    dmg += (attack_power * (character.combo_points as f32)) * 0.05;
    return dmg
}

fn white_attack(character: &mut Rogue, wep: &mut Weapon, 
                time_left: f32, verb: bool) -> (f32, bool) {
    // returns damage and a bool that is true if an extra swing procced

    let hit_result = determine_hit(&character, "white".to_string());
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
        return (0.0, false);
    }

    let mut dmg = get_wep_dmg(&wep, &character);
    if wep.is_offhand {
        dmg = dmg * 0.8;
    } 
    if hit_result == "glancing" { 
        dmg *= 1.0 - character.glancing_red;
    } else if hit_result == "crit" { 
        dmg *= 2.0;
    }
    if verb { announce_hit(dmg, announce_string, hit_result, time_left); }
    let extra_hit: bool = roll_for_extra_hit(character, wep);

    return (dmg, extra_hit);
}

fn armor_reduction(dmg: f32) -> f32 {
    let x = 0.1 * 3731.0 / (8.5 * 60.0 + 40.0);
    let red = x / (1.0 + x);
    return dmg * (1.0 - red);
}

struct Weapon {
    speed: f32,
    mean_dmg: f32,
    is_offhand: bool,
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
    weapon_skill: u16,
    dodge: f32,
    white_miss: f32,
    yellow_miss: f32,
    glancing: f32,
    glancing_red: f32,
    extra_hit_proc_chance: f32,
    shadowcraft_six_bonus: bool,
    imp_sin_strike: u8,
    precision: u8,
    dw_specialization: u8,
    sword_specialization: u8,
    weapon_expertise: u8,
    aggression: u8,
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

fn subtract_times(mut character: &mut Rogue, 
                  mut time_struct: &mut TimeTilEvents, 
                  mut buffs: &mut BuffsActive, dt: f32) {

    if time_struct.glob_cd_refresh > 0.0 {
        time_struct.glob_cd_refresh -= dt;
    } 
    if time_struct.wep1_swing > 0.0 {
        time_struct.wep1_swing -= dt;
    } 
    if time_struct.wep2_swing > 0.0 { 
        time_struct.wep2_swing -= dt; 
    }

    time_struct.energy_refill -= dt;
    if time_struct.energy_refill <= 0.0 { 

        time_struct.energy_refill = 2.0; 
        if character.energy < 81 {
            character.energy += 20;
        } else if character.energy >= 81 {
            character.energy = 100;
        }
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

    let character = Rogue {
        energy: 100,
        agility: 0,
        strength: 0,
        attack_power: 0, // IMPORTANT: just attack power given directly by gear
        crit: 0.0,
        hit: 0.0,
        dodge: 0.0,
        white_miss: 0.0,
        yellow_miss: 0.0,
        glancing: 0.0,
        glancing_red: 0.0,
        weapon_skill: 0,
        extra_hit_proc_chance: 0.0, // NOTE does not include thrash blade proc
        shadowcraft_six_bonus: false,
        imp_sin_strike: 0,
        precision: 0,
        dw_specialization: 0,
        sword_specialization: 0,
        weapon_expertise: 0,
        aggression: 0,
        improved_eviscerate: 0,
        malice: 0,
        ruthlessness: 0,
        improved_slice_and_dice: 0,
        relentless_strikes: 0,
        lethality: 0,
        combo_points: 0
    };
    return character;
}

fn init_weapon() -> Weapon {

    let wep = Weapon {
        speed: 0.0,
        mean_dmg: 0.0,
        is_offhand: false,
        extra_hit_proc_chance: 0.0
    };
    return wep;
}

fn read_params(param_file: &String) -> (Rogue, Weapon, Weapon) {
    
    let mut param_field: u8 = 0; // to check what part the file is about
    let mut read_last = false;
    let mut character: Rogue = init_rogue();
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
            else { param_adder(&l, &mut character); }

            continue;
        }

        if read_last {
            param_field += 1;
        }
        read_last = false;
    }
    (character, wep1, wep2)
}
  
fn param_adder(text: &str, character: &mut Rogue) {

    let words = text.split_whitespace();
    let words_vec = words.collect::<Vec<&str>>();
    if words_vec[0] == "agility" {
        match words_vec[1].parse() {
            Ok(x) => character.agility = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "strength" { 
        match words_vec[1].parse() {
            Ok(x) => character.strength = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "crit" { 
        match words_vec[1].parse() {
            Ok(x) => character.crit = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "hit" { 
        match words_vec[1].parse() {
            Ok(x) => character.hit = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "weapon_skill" { 
        match words_vec[1].parse() {
            Ok(x) => character.weapon_skill = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "attack_power" { 
        match words_vec[1].parse() {
            Ok(x) => character.attack_power = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "extra_hit_proc_chance" { 
        match words_vec[1].parse() {
            Ok(x) => character.extra_hit_proc_chance = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "shadowcraft_six_bonus" { 
        character.shadowcraft_six_bonus = true;
    } 
    // now for talents
    else if words_vec[0] == "imp_sin_strike" { 
        match words_vec[1].parse() {
            Ok(x) => character.imp_sin_strike = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "precision" { 
        match words_vec[1].parse() {
            Ok(x) => character.precision = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "dw_specialization" { 
        match words_vec[1].parse() {
            Ok(x) => character.dw_specialization = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "sword_specialization" { 
        match words_vec[1].parse() {
            Ok(x) => character.sword_specialization = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "weapon_expertise" { 
        match words_vec[1].parse() {
            Ok(x) => character.weapon_expertise = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "aggression" { 
        match words_vec[1].parse() {
            Ok(x) => character.aggression = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "improved_eviscerate" { 
        match words_vec[1].parse() {
            Ok(x) => character.improved_eviscerate = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "malice" { 
        match words_vec[1].parse() {
            Ok(x) => character.malice = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "ruthlessness" { 
        match words_vec[1].parse() {
            Ok(x) => character.ruthlessness = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "improved_slice_and_dice" { 
        match words_vec[1].parse() {
            Ok(x) => character.improved_slice_and_dice = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "relentless_strikes" { 
        match words_vec[1].parse() {
            Ok(x) => character.relentless_strikes = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "lethality" { 
        match words_vec[1].parse() {
            Ok(x) => character.lethality = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
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
    } else if words_vec[0] == "mean_dmg" { 
        match words_vec[1].parse() {
            Ok(x) => wep.mean_dmg = x,
            Err(x) => panic!("Can't translate word to number. {}", x)
        }
    } else if words_vec[0] == "is_offhand" { 
        match words_vec[1].parse() {
            Ok(x) => wep.is_offhand = x,
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
}

fn calculate_hit_numbers(character: &mut Rogue) {

    character.dodge = get_dodge_chance(character.weapon_skill);
    character.white_miss = get_white_miss_chance(character.weapon_skill);

    if character.hit > character.white_miss { character.white_miss = 0.0; }
    else { character.white_miss -= character.hit; }

    character.yellow_miss = get_yellow_miss_chance(character.weapon_skill);

    if character.hit > character.yellow_miss { character.yellow_miss = 0.0; }
    else { character.yellow_miss -= character.hit; }

    character.glancing_red = get_glancing_reduction(character.weapon_skill);
}
 
fn mean(numbers: &Vec<f32>) -> f32 {

    let sum: f32 = numbers.iter().sum();

    sum / numbers.len() as f32
}

/*
fn decide_event(mut time_struct: &mut TimeTilEvents, fight_time_left: f32, 
                character: Rogue) -> String {

    } else if time_struct.wep1_swing == 0 {
        return "white_1"
    } else if time_struct.wep2_swing == 0 {
        return "white_2"
    } else if time_struct.energy_refill == 0 {
        return "energy_refill"
    } 
}

Old rogue initialization
    let mut character = Rogue {
        energy: 100,
        agility: 371,
        strength: 140,
        attack_power: 80, // IMPORTANT: just attack power given directly by gear
        crit: 0.212,
        hit: 0.08,
        dodge: 0.0,
        white_miss: 0.0,
        yellow_miss: 0.0,
        glancing: 0.40,
        glancing_red: 0.0,
        weapon_skill: 310,
        extra_hit_proc_chance: 0.02, // NOTE does not include thrash blade proc
        combo_points: 0
    };
*/
