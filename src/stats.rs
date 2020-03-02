use crate::armory::HitProcc;
use crate::simulator::Hit;
use crate::utils::{Args,mean,std_dev};

use std::collections::HashMap;


#[derive(Debug)]
pub struct OverallStats {
    n_runs: i32,
    fight_length: f32,
    dps: Vec<f32>,
    backstab_ratio: Vec<f32>,
    sinister_strike_ratio: Vec<f32>,
    eviscerate_ratio: Vec<f32>,
    mh_white_ratio: Vec<f32>,
    oh_white_ratio: Vec<f32>,
    procc_dps_ratios: HashMap<String,Vec<f32>>
}

impl OverallStats {
    pub fn new_from_args(args: &Args) -> OverallStats {
        OverallStats {
            n_runs: args.iterations,
            fight_length: args.fight_length,
            dps: Vec::new(),
            backstab_ratio: Vec::new(),
            sinister_strike_ratio: Vec::new(),
            eviscerate_ratio: Vec::new(),
            mh_white_ratio: Vec::new(),
            oh_white_ratio: Vec::new(),
            procc_dps_ratios: HashMap::new()
        }
    }

    pub fn import_current_data(&mut self, stats: CurrentStats) {
        self.dps.push(stats.dmg / stats.fight_length);
        self.backstab_ratio.push(stats.backstab.dmg / stats.dmg);
        self.sinister_strike_ratio.push(stats.sinister_strike.dmg / stats.dmg);
        self.eviscerate_ratio.push(stats.eviscerate.dmg / stats.dmg);
        self.mh_white_ratio.push(stats.mh_white.dmg / stats.dmg);
        self.oh_white_ratio.push(stats.oh_white.dmg / stats.dmg);

        for (name, dmg_and_count) in &stats.proccs {
            let cur_vec = self.procc_dps_ratios.entry(name.to_string())
                .or_insert(Vec::new());
            cur_vec.push(dmg_and_count.dmg / stats.dmg);
        }
    }

    pub fn print_stat_weight(&self, stat_shift_text: &String) {

        let mean_dps = mean(&self.dps);
        let dps_within_std = std_dev(&self.dps);
        let mean_dps_std = 1.96 * dps_within_std / (self.n_runs as f32).sqrt();

        println!("{}{:.2}dps ±{:.2}",stat_shift_text, mean_dps, 
               mean_dps_std);
    }

    pub fn print(&self) {

        let mean_dps = mean(&self.dps);
        let dps_within_std = std_dev(&self.dps);
        let mean_dps_std = 1.96 * dps_within_std / (self.n_runs as f32).sqrt();

        let mean_backstab_ratio = mean(&self.backstab_ratio);
        let backstab_within_std = std_dev(&self.backstab_ratio);
        let mean_backstab_ratio_std = 1.96 * backstab_within_std
            / (self.n_runs as f32).sqrt();

        let mean_sinister_strike_ratio = mean(&self.sinister_strike_ratio);
        let sinister_strike_within_std = std_dev(&self.sinister_strike_ratio);
        let mean_sinister_strike_ratio_std = 1.96 * sinister_strike_within_std
            / (self.n_runs as f32).sqrt();

        let mean_eviscerate_ratio = mean(&self.eviscerate_ratio);
        let eviscerate_within_std = std_dev(&self.eviscerate_ratio);
        let mean_eviscerate_ratio_std = 1.96 * eviscerate_within_std
            / (self.n_runs as f32).sqrt();

        let mean_mh_white_ratio = mean(&self.mh_white_ratio);
        let mh_white_within_std = std_dev(&self.mh_white_ratio);
        let mean_mh_white_ratio_std = 1.96 * mh_white_within_std
            / (self.n_runs as f32).sqrt();

        let mean_oh_white_ratio = mean(&self.oh_white_ratio);
        let oh_white_within_std = std_dev(&self.oh_white_ratio);
        let mean_oh_white_ratio_std = 1.96 * oh_white_within_std
            / (self.n_runs as f32).sqrt();

        let mh_ratio = mean_mh_white_ratio 
            / (mean_mh_white_ratio + mean_oh_white_ratio);

        println!("\nStatistics over dps gathered from {} iterations of \
                  {}s each.", self.n_runs, self.fight_length);
        println!("Mean Dps:\t{:>8.2}  ±{:.2}", mean_dps, mean_dps_std);

        if mean_backstab_ratio > 0.0 {
            println!("Backstab:\t{:>8.2}% ±{:.2}%", 
                     100.0 * mean_backstab_ratio, 
                     100.0 * mean_backstab_ratio_std);
        }

        if mean_sinister_strike_ratio > 0.0 {
            println!("Sinister strike:\t{:>8.2}% ±{:.2}", 
                     100.0 * mean_sinister_strike_ratio, 
                     100.0 * mean_sinister_strike_ratio_std);
        }

        if mean_mh_white_ratio > 0.0 {
            println!("White hits:\t{:>8.2}% ±{:.2}%  (mh/oh: {:.0}/{:.0})", 
                     100.0 * (mean_mh_white_ratio + mean_oh_white_ratio), 
                     100.0 * (mean_mh_white_ratio_std + mean_oh_white_ratio_std),
                     100.0 * mh_ratio, 100.0 * (1.0 - mh_ratio)); 
        }

        if mean_eviscerate_ratio > 0.0 {
            println!("Eviscerate:\t{:>8.2}% ±{:.2}%", 
                     100.0 * mean_eviscerate_ratio, 
                     100.0 * mean_eviscerate_ratio_std);
        }

        for (name, dps_ratios) in &self.procc_dps_ratios {

            let mean_procc_dps_ratio = mean(&dps_ratios);
            let procc_dps_within_std = std_dev(dps_ratios);
            let mean_procc_dps_ratio_std = 1.96 * procc_dps_within_std
                / (self.n_runs as f32).sqrt();

            if mean_procc_dps_ratio > 0.0 {
                println!("{}:\t{:>8.2}% ±{:.2}%", name,
                         100.0 * mean_procc_dps_ratio, 
                         100.0 * mean_procc_dps_ratio_std);
            }
        }
    }
}

#[derive(Clone,Debug)]
pub struct CurrentStats {
    dmg: f32,
    fight_length: f32,
    backstab: OneAttackStats,
    sinister_strike: OneAttackStats,
    eviscerate: OneAttackStats,
    mh_white: OneAttackStats,
    oh_white: OneAttackStats,
    proccs: HashMap<String,DamageAndCount>
}

impl CurrentStats {
    pub fn new() -> CurrentStats {
        CurrentStats {
            dmg: 0.0,
            fight_length: 0.0,
            backstab: OneAttackStats::new(),
            sinister_strike: OneAttackStats::new(),
            eviscerate: OneAttackStats::new(),
            mh_white: OneAttackStats::new(),
            oh_white: OneAttackStats::new(),
            proccs: HashMap::new()
        }
    }

    pub fn copy(&self) -> CurrentStats {
        CurrentStats {
            dmg: self.dmg,
            fight_length: self.fight_length.clone(),
            backstab: self.backstab.clone(),
            sinister_strike: self.sinister_strike.clone(),
            eviscerate: self.eviscerate.clone(),
            mh_white: self.mh_white.clone(),
            oh_white: self.oh_white.clone(),
            proccs: self.proccs.clone()
        }
    }

    pub fn declare_proccs(&mut self, hit_proccs: &Vec<HitProcc>) {
        for i in 0..hit_proccs.len() {
            let name = match &hit_proccs[i] {
                HitProcc::Dmg(name,_,_,_) => name.clone(),
                HitProcc::Strength(name,_,_,_) => name.clone(),
                HitProcc::ExtraAttack(name,_) => name.clone(),
                HitProcc::None => panic!("Simulation does not run with \
                'None' proccs"),
            };

            self.proccs.insert(name.to_string(), DamageAndCount::new());
        }
    }

    pub fn set_fight_length(&mut self, fight_length: f32) {
        self.fight_length = fight_length;
    }

    pub fn record_mh_white_dmg_and_hit(&mut self, dmg: f32, hit_type: &Hit) {
        self.dmg += dmg;
        self.mh_white.add_dmg_and_hit(dmg, hit_type);
    }

    pub fn record_oh_white_dmg_and_hit(&mut self, dmg: f32, hit_type: &Hit) {
        self.dmg += dmg;
        self.oh_white.add_dmg_and_hit(dmg, hit_type);
    }

    pub fn record_sinister_strike_dmg_and_hit(&mut self, dmg: f32, hit_type: &Hit)
    {
        self.dmg += dmg;
        self.sinister_strike.add_dmg_and_hit(dmg, hit_type);
    }
    
    pub fn record_backstab_dmg_and_hit(&mut self, dmg: f32, hit_type: &Hit) {
        self.dmg += dmg;
        self.backstab.add_dmg_and_hit(dmg, hit_type);
    }

    pub fn record_eviscerate_dmg_and_hit(&mut self, dmg: f32, hit_type: &Hit) {
        self.dmg += dmg;
        self.eviscerate.add_dmg_and_hit(dmg, hit_type);
    }

    pub fn record_procc(&mut self, procc: &HitProcc) {
        match procc {
            HitProcc::Dmg(name,damage,_,_) => {
                let cur_val = self.proccs.entry(name.to_string())
                    .or_insert(DamageAndCount::new());
                cur_val.count += 1;
                cur_val.dmg += damage;
                self.dmg += damage
                },
            HitProcc::Strength(_,_,_,_) => (),

            HitProcc::ExtraAttack(_,_) => (),
            HitProcc::None => panic!("HitProcc::None cannot procc!"),
        }
    }

    fn print_dps(&self) {
        if self.backstab.dmg > 0.0 {
            println!("Backstab:\t\t{:.2}%", 100.0 * self.backstab.dmg / self.dmg);
        }
        if self.sinister_strike.dmg > 0.0 {
            println!("Sinister strike:\t{:.2}%", 
                     100.0 * self.sinister_strike.dmg / self.dmg);
        }
        if self.eviscerate.dmg > 0.0 {
            println!("Eviscerate:\t\t{:.2}%", 
                     100.0 * self.eviscerate.dmg / self.dmg);
        }
        let mh_white_ratio = 100.0 * self.mh_white.dmg 
            / (self.mh_white.dmg + self.oh_white.dmg);
        println!("White hits:\t\t{:.2}%\t(mh/oh: {:.0}/{:.0})", 
                 100.0 * (self.mh_white.dmg + self.oh_white.dmg) / self.dmg,
                 mh_white_ratio, 100.0 - mh_white_ratio);
        for (name, dmg_and_count) in &self.proccs {
            if dmg_and_count.dmg > 0.0 {
                println!("{}:\t\t{:.2}%", name,
                         100.0 * dmg_and_count.dmg / self.dmg);
            }
        }
        println!("\n");
    }

    pub fn print_stats(&mut self) {
        println!("\n*** Damage summary over {:.0}s ***\n", self.fight_length);
        self.print_dps();
        self.backstab.print_with_name("Backstab");
        self.sinister_strike.print_with_name("Sinister strike");
        self.eviscerate.print_with_name("Eviscerate");
        self.mh_white.print_with_name("MH white");
        self.oh_white.print_with_name("OH white");
    }
     
    pub fn clear(&mut self) {
        self.dmg = 0.0;
        self.backstab.clear();
        self.sinister_strike.clear();
        self.eviscerate.clear();
        self.mh_white.clear();
        self.oh_white.clear();
        self.clear_proccs();
    }

    fn clear_proccs(&mut self) {
        let mut new_map: HashMap<String,DamageAndCount> = HashMap::new();
        for (name, _) in &self.proccs {
            new_map.entry(name.to_string()).or_insert(DamageAndCount::new());
        }
        self.proccs = new_map;
    }
}

#[derive(Clone,Debug)]
struct OneAttackStats {
    dmg: f32,
    tot_count: i32,
    crit: i32,
    dodge: i32,
    glancing: i32,
    hit: i32,
    miss: i32
}
    
impl OneAttackStats {
    fn new() -> OneAttackStats {
        OneAttackStats {
            dmg: 0.0,
            tot_count: 0,
            crit: 0,
            dodge: 0,
            glancing: 0,
            hit: 0,
            miss: 0
        }
    }

    fn add_dmg_and_hit(&mut self, dmg: f32, hit_type: &Hit) {
        self.tot_count += 1;
        self.dmg += dmg;

        if *hit_type == Hit::Hit { self.hit += 1; }
        else if *hit_type == Hit::Crit { self.crit += 1; }
        else if *hit_type == Hit::Miss { self.miss += 1; }
        else if *hit_type == Hit::Glancing { self.glancing += 1; }
        else if *hit_type == Hit::Dodge { self.dodge += 1; }
        else if *hit_type == Hit::Hit { self.hit += 1; }
        else { panic!("Hit type not implemented: {}", *hit_type); }
    }

    fn print_with_name(&self, name: &str) {
        if self.tot_count > 0 { 
            println!("{}", name);
            println!("\tTotal hits: \t{}", self.tot_count); 
        }
        if self.hit > 0 { 
            println!("\tHit: \t\t{}\t{:.1}%", self.hit, 
                     100.0 * self.hit as f32 / self.tot_count as f32); 
        }
        if self.crit > 0 { 
            println!("\tCrit: \t\t{}\t{:.1}%", self.crit, 
                     100.0 * self.crit as f32 / self.tot_count as f32); 
        }
        if self.glancing > 0 { 
            println!("\tGlancing: \t{}\t{:.1}%", self.glancing, 
                     100.0 * self.glancing as f32 / self.tot_count as f32); 
        }
        if self.miss > 0 { 
            println!("\tMiss: \t\t{}\t{:.1}%", self.miss, 
                     100.0 * self.miss as f32 / self.tot_count as f32); 
        }
        if self.dodge > 0 { 
            println!("\tDodge: \t\t{}\t{:.1}%", self.dodge, 
                     100.0 * self.dodge as f32 / self.tot_count as f32); 
        }
    }

    fn clear(&mut self) {
        self.dmg = 0.0;
        self.tot_count = 0;
        self.crit = 0;
        self.dodge = 0;
        self.glancing = 0;
        self.hit = 0;
        self.miss = 0;
    }
}

#[derive(Clone,Debug)]
struct DamageAndCount {
    dmg: f32,
    count: i32
}

impl DamageAndCount {
    pub fn new() -> DamageAndCount {
        DamageAndCount {
            dmg: 0.0,
            count: 0
        }
    }
}
