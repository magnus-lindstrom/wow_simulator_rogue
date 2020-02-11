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

extern crate rand;
extern crate clap;

use std::fmt;
use std::f32;
use rand::distributions::{Distribution, Uniform};


fn main() {

    let args = utils::get_arguments();
    let character = armory::Character::new(armory::Race::Human);
    println!("character: {:?}", character);

}
