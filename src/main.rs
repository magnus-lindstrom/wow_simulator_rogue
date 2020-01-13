

fn main() {

    let fight_length = 30.0;
    let dt = 0.1;
    let mut wep1 = Weapon {
        speed: 2.7,
        mean_dmg: 90,
        is_offhand: false,
        extra_hit_proc_chance: 3.79
    };
    let mut wep2 = Weapon {
        speed: 1.8,
        mean_dmg: 90,
        is_offhand: true,
        extra_hit_proc_chance: 0.0
    };

    let mut character = Rogue {
        energy: 100,
        agility: 100,
        strength: 100,
        crit: 20.0,
        hit: 9.0,
        weapon_skill: 300,
    };

    let mut time_struct = TimeTilEvents {
        glob_cd_refresh: 0.0,
        wep1_swing: 0.0,
        wep2_swing: 0.0,
        energy_refill: 0.0,
        fight_ends: fight_length
    };

    let mut tot_dmg = 0;
    while time_struct.fight_ends > 0.0 {
        if time_struct.glob_cd_refresh <= 0.0 {
            let dmg = yellow_attack(&mut character, &mut time_struct);
            if dmg > 0 { tot_dmg += dmg; }
        }
        if time_struct.wep1_swing <= 0.0 {
            let dmg = white_attack(character, &mut wep1, time_struct.fight_ends);
            time_struct.wep1_swing = wep1.speed;
            tot_dmg += dmg;
        }
        if time_struct.wep2_swing <= 0.0 {
            let dmg = white_attack(character, &mut wep2, time_struct.fight_ends);
            time_struct.wep2_swing = wep2.speed;
            tot_dmg += dmg;
        }
        subtract_times(&mut character, &mut time_struct, dt);
    }
}

fn announce_hit(dmg: u16, hit_type: String, time: f32) {
    if hit_type == "sin_strike" {
        println!("{:2.1}: Sinister strike for {:}", time, dmg);
    } else if hit_type == "evis" {
        println!("{:2.1}: Eviscerate for {:}", time, dmg);
    } else if hit_type == "mh_hit" {
        println!("{:2.1}: Main hand white hit for {:}", time, dmg);
    } else if hit_type == "oh_hit" {
        println!("{:2.1}: Off hand white hit for {:}", time, dmg);
    }
}

fn yellow_attack(mut character: &mut Rogue, 
                 mut time_struct: &mut TimeTilEvents) -> u16 {
    let mut dmg = 0;
    if character.energy >= 40 {
        character.energy -= 40;
        dmg = 200;
        time_struct.glob_cd_refresh = 1.0;
        announce_hit(dmg, "sin_strike".to_string(), time_struct.fight_ends);
    }
    return dmg;
}

fn white_attack(rogue: Rogue, mut weapon: &mut Weapon, time_left: f32) -> u16 {
    let mut dmg = weapon.mean_dmg;
    if weapon.is_offhand {
        dmg = (dmg as f32 * 0.8) as u16;
        announce_hit(dmg, "oh_hit".to_string(), time_left);
    } else {
        announce_hit(dmg, "mh_hit".to_string(), time_left);
    }
    return dmg;
}

struct Weapon {
    speed: f32,
    mean_dmg: u16,
    is_offhand: bool,
    extra_hit_proc_chance: f32
}

#[derive(Copy, Clone)]
struct Rogue {
    energy: i8,
    agility: u16,
    strength: u16,
    crit: f32,
    hit: f32,
    weapon_skill: u16
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
                  mut time_struct: &mut TimeTilEvents, dt: f32) {

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
        //todo: sometimes add 21 energy
        if character.energy < 81 {
            character.energy += 20;
        } else if character.energy >= 81 {
            character.energy = 100;
        }
    }
    time_struct.fight_ends -= dt;
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
*/
