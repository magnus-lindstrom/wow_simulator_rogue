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
mod utils;
mod simulator;

extern crate rand;
extern crate clap;
extern crate serde;
extern crate serde_yaml;

#[macro_use]
extern crate enum_display_derive;

use armory::{Character};
use simulator::{Simulator};


fn main() {

    let args = utils::get_arguments();
    let character = Character::create_character(&args);
    let mut simulator: Simulator = Simulator::new();
    simulator.apply_input_arguments(&args);
    simulator.configure_with_character(&character);

    for i_iter in 0..args.iterations {
        simulator.simulate();
    }

    // println!("args: {:?}\n", args);
    // println!("character: {:?}\n", character);
    println!("simulator: {:?}\n", simulator);

}
