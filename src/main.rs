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
mod stats;
mod weights;

extern crate rand;
extern crate clap;
extern crate serde;
extern crate serde_yaml;

#[macro_use]
extern crate enum_display_derive;

use armory::Character;
use simulator::Simulator;
use stats::OverallStats;
use weights::StatShift;
use utils::Args;


fn get_stat_weights(args: &Args) {

    let stat_shifts = StatShift::new(args);
    let mut default_dps: f32 = 0.0;
    for (i, stat_shift) in stat_shifts.iter().enumerate() {

        let mut character = Character::create_character(args);
        character.apply_stat_shift(&stat_shift);
        character.convert_stats_and_set_cooldowns();

        let mut simulator: Simulator = Simulator::new();
        simulator.apply_input_arguments(args);
        simulator.configure_with_character(&character);

        let mut stats = OverallStats::new_from_args(args);
        stats.add_weights_text(&stat_shift.text);

        for _iter in 0..args.iterations {
            simulator.simulate();
            simulator.print_stats();
            stats.import_current_data(simulator.get_stats());
        }
        if i == 0 { 
            stats.print_stat_weight_default_run(); 
            default_dps = stats.get_mean_dps();
        }
        else { 
            stats.print_stat_weight_minus_default_dps(default_dps); 
        }
    }
}

fn normal_simulation(args: &Args) {

    let mut character = Character::create_character(args);
    character.convert_stats_and_set_cooldowns();

    let mut simulator: Simulator = Simulator::new();
    simulator.apply_input_arguments(args);
    simulator.configure_with_character(&character);

    let mut stats = OverallStats::new_from_args(args);

    for _iter in 0..args.iterations {
        simulator.simulate();
        simulator.print_stats();
        stats.import_current_data(simulator.get_stats());
    }
    stats.print();

    character.print_all_stats(args);
}

fn main() {

    let args = utils::get_arguments();
    if args.weights { get_stat_weights(&args); }
    else { normal_simulation(&args); }

}
