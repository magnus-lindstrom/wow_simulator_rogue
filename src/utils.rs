use crate::armory;
use clap::{Arg, App};
use std::fs::File;
use std::io::{BufRead, BufReader};


pub fn max_f32(x: f32, y: f32) -> f32 {
    if x >= y { return x; }
    else { return y; }
}

pub struct Args {
    dt: f32,
    no_buffs: bool,
    enemy_lvl: i32,
    fight_length: f32,
    iterations: i32,
    param_file: String,
    verb: bool,
    weight_mult: i32,
    weights: bool
}

impl Args {
    fn default_args() -> Args {
        Args {
            dt: 0.0,
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

pub fn get_arguments() -> Args {

    let matches = App::new("WoW rogue simulator") 
        .version("0.1.0") 
        .author("Magnus Lindström <magnus.lindstrom@tuta.io>")
        .about("Compares items/specs for PvE raiding purposes. Combat Rogues.") 
        .arg(Arg::with_name("Step length") 
             .required(false)
             .short("s") 
             .long("step").takes_value(true) 
             .help("The time step. Default is 0.01s."))
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

    let dt = matches.value_of("Step length").unwrap_or("0.01");
    let file = matches.value_of("Parameter file").unwrap();
    let iterations = matches.value_of("Nr of iterations").unwrap_or("1");
    let fight_length = matches.value_of("Fight length").unwrap_or("60");
    let enemy_lvl = matches.value_of("Enemy level").unwrap_or("63");
    let weights = matches.is_present("Weights");
    let weight_mult = matches.value_of("Weight multiplier").unwrap_or("1");
    let no_buffs = matches.is_present("No buffs");
    let verb = matches.is_present("Verbose");

    let mut args = Args::default_args();
    args.dt = dt.parse().unwrap();
    args.param_file = file.to_string();
    args.verb = verb;
    args.no_buffs = no_buffs;
    args.weights = weights;
    args.weight_mult = weight_mult.parse().unwrap();
    args.enemy_lvl = enemy_lvl.parse().unwrap();
    args.iterations = iterations.parse().unwrap();
    let fl: u32 = fight_length.parse().unwrap();
    args.fight_length = fl as f32;

    if args.fight_length > 120.0 {
        println!("WARNING: fight longer than 2min, no cooldown for");
        println!("blade flurry implemented.");
    }

    return args;

}

fn read_params(param_file: &String) -> armory::Character {
    
    // todo: weapons not being added
    let mut param_field: u16 = 0; // to check what part the file is about
    let mut read_last = false;
    let mut character = armory::Character::new(armory::Race::Human);

    let f = File::open(param_file).expect("Couldn't open param_file");
    let file = BufReader::new(&f);
    for line in file.lines() {
        let l = line.unwrap();
        let first_char = l.chars().next().unwrap();
        if first_char != '#' && first_char != '@' {
            read_last = true;
            if param_field == 0 { character.add_armor(l); }
            else if param_field == 1 { 
                character.set_mh(l);
            } else if param_field == 2 { 
                character.set_oh(l); 
            } 
            continue;
        }

        if read_last {
            param_field += 1;
        }
        read_last = false;
    }
    character
}

