use clap::{Arg, App};
use rand::distributions::{Distribution, Uniform};

extern crate serde;
extern crate serde_yaml;


pub fn deb<T: std::fmt::Debug>(x: T) {
    println!("{:?}", x);
}

pub fn min_f32(x: f32, y: f32) -> f32 {
    if x <= y { return x; }
    else { return y; }
}

pub fn max_f32(x: f32, y: f32) -> f32 {
    if x >= y { return x; }
    else { return y; }
}

pub fn min_i32(x: i32, y: i32) -> i32 {
    if x <= y { return x; }
    else { return y; }
}

pub fn max_i32(x: i32, y: i32) -> i32 {
    if x >= y { return x; }
    else { return y; }
}

#[derive(Debug)]
pub struct Args {
    pub dt: f32,
    pub enemy_lvl: i32,
    pub fight_length: f32,
    pub iterations: i32,
    pub items_file: String,
    pub talents_file: String,
    pub enchants_file: String,
    pub buffs_file: String,
    pub verb: i32,
    pub weight_mult: i32,
    pub weights: bool
}

impl Args {
    fn default_args() -> Args {
        Args {
            dt: 0.0,
            enemy_lvl: 0,
            fight_length: 0.0,
            iterations: 0,
            items_file: "".to_string(),
            talents_file: "".to_string(),
            enchants_file: "".to_string(),
            buffs_file: "".to_string(),
            verb: 0,
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
        .arg(Arg::with_name("Items file") 
             .required(true)
             .short("I") 
             .long("items").takes_value(true) 
             .help("Parameter file that contains items to equip."))
        .arg(Arg::with_name("Talents file") 
             .required(true)
             .short("T") 
             .long("talents").takes_value(true) 
             .help("Parameter file with specc of character."))
        .arg(Arg::with_name("Enchants file") 
             .required(true)
             .short("E") 
             .long("enchants").takes_value(true) 
             .help("Parameter file with buffs to use."))
        .arg(Arg::with_name("Buffs file") 
             .required(true)
             .short("B") 
             .long("buffs").takes_value(true) 
             .help("Parameter file with buffs to use."))
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
        .arg(Arg::with_name("Verbose") 
            .short("v") 
            .long("verbose") 
            .multiple(true)
            .takes_value(false) 
            .help("0: print a short summary of dps at the end of all iterations\n\
            1: prints a combat log with abilities used \n\
            and strikes made\n\
            2: prints minor events like energy refill and a detailed summary\n\
            of all attacks made.\n\
            3: dump entire simulator object at the end of a run."))
        .get_matches();

    let dt = matches.value_of("Step length").unwrap_or("0.01");
    let items = matches.value_of("Items file").unwrap();
    let talents = matches.value_of("Talents file").unwrap();
    let enchants = matches.value_of("Enchants file").unwrap();
    let buffs = matches.value_of("Buffs file").unwrap();
    let iterations = matches.value_of("Nr of iterations").unwrap_or("1");
    let fight_length = matches.value_of("Fight length").unwrap_or("60");
    let enemy_lvl = matches.value_of("Enemy level").unwrap_or("63");
    let weights = matches.is_present("Weights");
    let weight_mult = matches.value_of("Weight multiplier").unwrap_or("1");
    let verb = matches.occurrences_of("Verbose");

    let mut args = Args::default_args();
    args.dt = dt.parse().unwrap();
    args.items_file = items.to_string();
    args.talents_file = talents.to_string();
    args.enchants_file = enchants.to_string();
    args.buffs_file = buffs.to_string();
    args.verb = verb as i32;
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

pub fn roll_die() -> f32 {
    // rolls a die between [0, 1)
    
    let mut rng = rand::thread_rng();
    let roll_range = Uniform::from(100..10_000); // not including upper bound
    let roll = roll_range.sample(&mut rng);
    let roll: f32 = (roll as f32) / 10_000.0;
    return roll;
}

pub fn mean(numbers: &Vec<f32>) -> f32 {

    let mut sum: f64 = 0.0;
    for n in numbers.iter() { sum += *n as f64; }

    let avg = sum / numbers.len() as f64;
    return avg as f32;
}

pub fn std_dev(numbers: &Vec<f32>) -> f32 {

    let mean = mean(numbers);
    let mut tmp = 0.0;
    for nr in numbers {
        tmp += (nr - mean).powf(2.0);
    }
    let std_dev = (tmp / (numbers.len() - 1) as f32).sqrt();
    return std_dev;
}
