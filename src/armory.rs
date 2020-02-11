#[derive(Clone,Copy,Debug,PartialEq)]
pub enum Race {
    Human,
    NightElf,
    Gnome,
    Dwarf,
    None
}

#[derive(Debug)]
struct PrimStats {
    agility: i32,
    strength: i32,
    stamina: i32,
    sword_skill: i32,
    dagger_skill: i32
}

impl PrimStats {
    fn new(race: Race) -> PrimStats {
        if race == Race::Human {
            PrimStats {
                agility: 130,
                strength: 80,
                stamina: 75,
                sword_skill: 305,
                dagger_skill: 300
            }
        } else if race == Race::None {
            PrimStats {
                agility: 0,
                strength: 0,
                stamina: 0,
                sword_skill: 0,
                dagger_skill: 0
            }
        } else { panic!("Race not implemented"); }
    }
}

#[derive(Debug)]
struct SecStats {
    crit: f32,
    hit: f32,
    haste: f32,
    attack_power: i32
}

impl SecStats {
    fn new(race: Race) -> SecStats {
        if race == Race::Human {
            SecStats {
                crit: 0.0,
                hit: 0.0,
                haste: 0.0,
                attack_power: 100
            }
        } else if race == Race::None {
            SecStats {
                crit: 0.0,
                hit: 0.0,
                haste: 0.0,
                attack_power: 0
            }
        } else { panic!("Race not implemented"); }
    }
}

#[derive(Debug)]
pub struct Weapon {
    prim_stats: PrimStats,
    sec_stats: SecStats,
    swing_speed: f32,
    min_dmg: f32,
    max_dmg: f32,
    mean_dmg: f32
}

impl Weapon {
    fn new(name: String) -> Weapon {
        let mut wep: Weapon;
        if name == "gutgore_ripper" {
            let prim_stats = PrimStats::new(Race::None);
            let sec_stats = SecStats::new(Race::None);
            wep = Weapon {
                prim_stats: prim_stats,
                sec_stats: sec_stats,
                swing_speed: 1.8,
                min_dmg: 63.0,
                max_dmg: 119.0,
                mean_dmg: 0.0
            };
        } else if name == "" {
            let prim_stats = PrimStats::new(Race::None);
            let sec_stats = SecStats::new(Race::None);
            wep = Weapon {
                prim_stats: prim_stats,
                sec_stats: sec_stats,
                swing_speed: 0.0,
                min_dmg: 0.0,
                max_dmg: 0.0,
                mean_dmg: 0.0
            };

        }else { panic!("Weapon not implemented: {}"); }

        wep.mean_dmg = (wep.min_dmg + wep.max_dmg) / 2.0;
        return wep;
    }
}
        
#[derive(Debug)]
pub struct Armor {
    // todo: add item slots and sets
    prim_stats: PrimStats,
    sec_stats: SecStats
}

impl Armor {
    fn new(name: String) -> Armor {
        let mut armor: Armor;
        if name == "bloodfang_hood" {
            let prim_stats = PrimStats::new(Race::None);
            let sec_stats = SecStats::new(Race::None);
            armor = Armor {
                prim_stats: prim_stats,
                sec_stats: sec_stats
            };
        } else { panic!("Armor piece not implemented: {}", name); }

        return armor;
    }
}

#[derive(Debug)]
pub struct Character {
    race: Race,
    prim_stats: PrimStats,
    sec_stats: SecStats,
    mh: Weapon,
    oh: Weapon,
    armor: Vec<Armor>
}

impl Character {
    pub fn new(race: Race) -> Character {
        Character {
            race: race,
            prim_stats: PrimStats::new(race),
            sec_stats: SecStats::new(race),
            mh: Weapon::new("".to_string()),
            oh: Weapon::new("".to_string()),
            armor: Vec::new()
        }
    }

    pub fn add_armor(&mut self, armor_name: String) {
        self.armor.push(Armor::new(armor_name));
    }

    pub fn set_mh(&mut self, wep_name: String) {
        println!("{}", wep_name);
        self.mh = Weapon::new(wep_name)
    }
    pub fn set_oh(&mut self, wep_name: String) {
        println!("{}", wep_name);
        self.oh = Weapon::new(wep_name)
    }

}




            





