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
extern crate serde;
extern crate serde_yaml;

use armory::{Character,ItemCollection};
use std::fmt;
use std::collections::HashMap;
use std::f32;
use rand::distributions::{Distribution, Uniform};
use std::fs;
use std::fs::File;
use std::io::Write;


fn main() {

    let args = utils::get_arguments();
    let character = Character::get_character(&args);
    let item_collection: ItemCollection = 
        ItemCollection::initialize_item_collection();

    println!("{:?}", item_collection.armor.get("bloodfang_hood"));

}
