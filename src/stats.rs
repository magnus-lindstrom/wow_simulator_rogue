use crate::simulator::Hit;


#[derive(Debug)]
pub struct CurrentStats {
    dmg: f32,
    fight_length: f32,
    backstab: OneAttackStats,
    sinister_strike: OneAttackStats,
    eviscerate: OneAttackStats,
    mh_white: OneAttackStats,
    oh_white: OneAttackStats
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
            oh_white: OneAttackStats::new()
        }
    }

    pub fn set_fight_length(&mut self, fight_length: f32) {
        self.fight_length = fight_length;
    }

    pub fn record_mh_white_dmg_and_hit(&mut self, dmg: f32, hit_type: &Hit) {
        self.dmg += dmg;
        self.mh_white.add(hit_type);
    }

    pub fn record_oh_white_dmg_and_hit(&mut self, dmg: f32, hit_type: &Hit) {
        self.dmg += dmg;
        self.oh_white.add(hit_type);
    }

    pub fn record_sinister_strike_dmg_and_hit(&mut self, dmg: f32, hit_type: &Hit)
    {
        self.dmg += dmg;
        self.sinister_strike.add(hit_type);
    }
    
    pub fn record_backstab_dmg_and_hit(&mut self, dmg: f32, hit_type: &Hit) {
        self.dmg += dmg;
        self.backstab.add(hit_type);
    }

    pub fn record_eviscerate_dmg_and_hit(&mut self, dmg: f32, hit_type: &Hit) {
        self.dmg += dmg;
        self.eviscerate.add(hit_type);
    }

    pub fn print_stats(&mut self) {
        println!("\n*** Summary of hits over {:.0}s ***\n", self.fight_length);
        self.backstab.print_with_name("Backstab");
        self.sinister_strike.print_with_name("Sinister strike");
        self.eviscerate.print_with_name("Eviscerate");
        self.mh_white.print_with_name("MH white");
        self.oh_white.print_with_name("OH white");
    }

    pub fn clear(&mut self) {
        self.backstab.clear();
        self.sinister_strike.clear();
        self.eviscerate.clear();
        self.mh_white.clear();
        self.oh_white.clear();
    }
}

#[derive(Debug)]
struct OneAttackStats {
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
            tot_count: 0,
            crit: 0,
            dodge: 0,
            glancing: 0,
            hit: 0,
            miss: 0
        }
    }

    fn add(&mut self, hit_type: &Hit) {
        self.tot_count += 1;

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
        self.tot_count = 0;
        self.crit = 0;
        self.dodge = 0;
        self.glancing = 0;
        self.hit = 0;
        self.miss = 0;
    }
}

