use crate::simulation::environment::Environment;
use crate::simulation::organism::Organism;
use crate::simulation::species::{Species, SpeciesType};
use crate::simulation::config::SimulationConfig;

pub const WORLD_W: f64 = 1200.0;
pub const WORLD_H: f64 = 800.0;
pub const MAX_ORGANISMS: usize = 2000;
pub const MUTATION_RATE: f64 = 0.05;

#[derive(Clone, Debug)]
pub struct EcosystemMaturity {
    pub metabolic_rate: f64,
    pub production_respiration_ratio: f64,
    pub cycling_index: f64,
    pub total_biomass: f64,
}

impl Default for EcosystemMaturity {
    fn default() -> Self {
        Self {
            metabolic_rate: 0.0,
            production_respiration_ratio: 1.0,
            cycling_index: 0.0,
            total_biomass: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GameEvent {
    pub event_type: EventType,
    pub message: String,
    pub tick: u32,
}

#[derive(Clone, Debug)]
pub enum EventType {
    Birth,
    Death,
    Evolution,
    Symbiosis,
}

#[derive(Clone)]
pub struct World {
    pub species: Vec<Species>,
    pub organisms: Vec<Organism>,
    pub environment: Environment,
    pub events: Vec<GameEvent>,
    pub pop_history: Vec<Vec<u32>>,
    pub diversity_history: Vec<f64>,
    pub tick: u32,
    pub running: bool,
    pub speed: u32,
    pub species_counter: u32,
    pub config: SimulationConfig,
    pub maturity: EcosystemMaturity,
    tick_primary_production: f64,
    tick_respiration: f64,
    tick_decomposer_return: f64,
    rolling_tpp: Vec<f64>,
    rolling_tr: Vec<f64>,
}

// Helper: organism snapshot for read-only queries
struct OrgInfo {
    idx: usize,
    species_id: u32,
    x: f64,
    y: f64,
    energy: f64,
    alive: bool,
}

impl World {
    pub fn new() -> Self {
        Self {
            species: Vec::new(),
            organisms: Vec::new(),
            environment: Environment::new(),
            events: Vec::new(),
            pop_history: Vec::new(),
            diversity_history: Vec::new(),
            tick: 0,
            running: false,
            speed: 3,
            species_counter: 0,
            config: SimulationConfig::default(),
            maturity: EcosystemMaturity::default(),
            tick_primary_production: 0.0,
            tick_respiration: 0.0,
            tick_decomposer_return: 0.0,
            rolling_tpp: Vec::new(),
            rolling_tr: Vec::new(),
        }
    }

    pub fn add_species(&mut self, name: &str, species_type: SpeciesType, color: (u8, u8, u8), count: usize, repro_rate: f64, speed: f64, efficiency: f64) {
        self.species_counter += 1;
        let mut sp = Species::new(name, species_type, color, repro_rate, speed, efficiency, self.species_counter);

        // Setup diet based on existing species
        for existing in &self.species {
            match species_type {
                SpeciesType::Herbivore if existing.species_type == SpeciesType::Plant => {
                    sp.diet.push(existing.id);
                }
                SpeciesType::Carnivore if existing.species_type == SpeciesType::Herbivore => {
                    sp.diet.push(existing.id);
                }
                _ => {}
            }
            // Add symbiosis
            if species_type == SpeciesType::Herbivore && existing.species_type == SpeciesType::Plant {
                if sp.symbiosis.is_empty() {
                    sp.symbiosis.push((existing.id, crate::simulation::species::SymbiosisType::Mutualism));
                }
            }
        }

        // Also update existing species to eat this new one
        let new_id = sp.id;
        for existing in &mut self.species {
            match existing.species_type {
                SpeciesType::Herbivore if species_type == SpeciesType::Plant => {
                    existing.diet.push(new_id);
                }
                SpeciesType::Carnivore if species_type == SpeciesType::Herbivore => {
                    existing.diet.push(new_id);
                }
                _ => {}
            }
        }

        self.pop_history.push(Vec::new());

        for _ in 0..count {
            let x = 50.0 + js_sys::Math::random() * (WORLD_W - 100.0);
            let y = 50.0 + js_sys::Math::random() * (WORLD_H - 100.0);
            let o = Organism::new(&sp, x, y);
            self.organisms.push(o);
        }

        self.push_event(EventType::Birth, format!("新物种 \"{}\" 加入生态系统 (×{})", name, count));
        self.species.push(sp);
    }

    fn push_event(&mut self, event_type: EventType, message: String) {
        self.events.insert(0, GameEvent {
            event_type,
            message,
            tick: self.tick,
        });
        if self.events.len() > 50 {
            self.events.truncate(50);
        }
    }

    pub fn add_event(&mut self, event_type: EventType, message: &str) {
        self.push_event(event_type, message.to_string());
    }

    pub fn update(&mut self) {
        self.tick += 1;
        self.environment.update();

        let env_temp = self.environment.temperature;
        let env_sun = self.environment.sunlight;
        let env_moist = self.environment.moisture;
        let wind_x = self.environment.wind_x;
        let wind_y = self.environment.wind_y;

        // Build species lookup (clone to avoid borrow issues)
        let species_vec: Vec<Species> = self.species.clone();
        let species_map: std::collections::HashMap<u32, Species> = species_vec.into_iter().map(|s| (s.id, s)).collect();

        // Snapshot organism positions for decision-making
        let snapshots: Vec<OrgInfo> = self.organisms.iter().enumerate().map(|(i, o)| OrgInfo {
            idx: i,
            species_id: o.species_id,
            x: o.x,
            y: o.y,
            energy: o.energy,
            alive: o.alive,
        }).collect();

        // Reset tick-level energy flow accumulators
        self.tick_primary_production = 0.0;
        self.tick_respiration = 0.0;
        self.tick_decomposer_return = 0.0;

        // Phase 1: Update movement, energy, eating
        for i in 0..self.organisms.len() {
            if !self.organisms[i].alive {
                continue;
            }

            let sp_id = self.organisms[i].species_id;
            let species = match species_map.get(&sp_id) {
                Some(s) => s,
                None => continue,
            };

            // Update movement/energy, tracking energy flows
            let energy_before = self.organisms[i].energy;
            self.organisms[i].update(species, env_temp, env_sun, env_moist, wind_x, wind_y, WORLD_W, WORLD_H);
            let energy_delta = self.organisms[i].energy - energy_before;

            // Track energy flows for maturity metrics
            match species.species_type {
                SpeciesType::Plant => {
                    if energy_delta > 0.0 {
                        self.tick_primary_production += energy_delta;
                    }
                    self.tick_respiration += energy_delta.abs().min(0.005 * (self.organisms[i].speed * 0.3 + 0.7));
                }
                _ => {
                    self.tick_respiration += 0.005 * (self.organisms[i].speed * 0.3 + 0.7);
                }
            }

            // For non-plants, seek food and flee
            if species.species_type != SpeciesType::Plant && !species.diet.is_empty() {
                let sense = species.traits.sense_range;
                let ox = self.organisms[i].x;
                let oy = self.organisms[i].y;
                let spd = self.organisms[i].speed * self.organisms[i].energy;

                // Find closest prey
                let mut closest_dist = sense;
                let mut closest_idx: Option<usize> = None;
                let mut closest_pred_dist = sense * 0.7;
                let mut closest_pred_idx: Option<usize> = None;

                for snap in &snapshots {
                    if snap.idx == i || !snap.alive {
                        continue;
                    }
                    let dx = ox - snap.x;
                    let dy = oy - snap.y;
                    let d = (dx * dx + dy * dy).sqrt();

                    // Check if prey
                    if species.diet.contains(&snap.species_id) && d < closest_dist {
                        closest_dist = d;
                        closest_idx = Some(snap.idx);
                    }

                    // Check if predator
                    if let Some(pred_sp) = species_map.get(&snap.species_id) {
                        if pred_sp.diet.contains(&sp_id) && d < closest_pred_dist {
                            closest_pred_dist = d;
                            closest_pred_idx = Some(snap.idx);
                        }
                    }
                }

                // Flee has priority
                if let Some(pred_idx) = closest_pred_idx {
                    let px = self.organisms[pred_idx].x;
                    let py = self.organisms[pred_idx].y;
                    self.organisms[i].flee(px, py, spd);
                } else if let Some(prey_idx) = closest_idx {
                    let tx = self.organisms[prey_idx].x;
                    let ty = self.organisms[prey_idx].y;
                    self.organisms[i].chase(tx, ty, spd);

                    // Try to eat
                    let dx = ox - tx;
                    let dy = oy - ty;
                    let d = (dx * dx + dy * dy).sqrt();
                    if d < self.organisms[i].size + self.organisms[prey_idx].size
                        && self.organisms[i].age - self.organisms[i].last_meal > 30
                    {
                        let prey_energy = self.organisms[prey_idx].energy;
                        let gain = match species.species_type {
                            SpeciesType::Herbivore => prey_energy * self.config.herbivore_conversion,
                            SpeciesType::Carnivore => prey_energy * self.config.carnivore_conversion,
                            _ => prey_energy * self.config.decomposer_conversion,
                        };
                        self.organisms[i].energy = (1.0_f64).min(self.organisms[i].energy + gain);
                        self.organisms[i].last_meal = self.organisms[i].age;
                        self.organisms[prey_idx].alive = false;
                    }
                }
            }

            // Decomposers feed on dead
            if species.species_type == SpeciesType::Decomposer {
                for snap in &snapshots {
                    if snap.idx != i && !snap.alive {
                        let dx = self.organisms[i].x - snap.x;
                        let dy = self.organisms[i].y - snap.y;
                        let d = (dx * dx + dy * dy).sqrt();
                        if d < 30.0 {
                            self.organisms[i].energy = (1.0_f64).min(self.organisms[i].energy + 0.1);
                            self.tick_decomposer_return += 0.1;
                            break;
                        }
                    }
                }
            }
        }

        // Phase 2: Reproduction with carrying capacity & ecological regulation
        let mut newborns: Vec<Organism> = Vec::new();

        // Pre-compute per-species alive counts for density calculations
        let mut species_counts: std::collections::HashMap<u32, f64> = std::collections::HashMap::new();
        for sp in &self.species {
            let count = self.organisms.iter().filter(|o| o.species_id == sp.id && o.alive).count() as f64;
            species_counts.insert(sp.id, count);
        }

        for i in 0..self.organisms.len() {
            if !self.organisms[i].alive {
                continue;
            }
            let sp_id = self.organisms[i].species_id;
            let species = match species_map.get(&sp_id) {
                Some(s) => s,
                None => continue,
            };

            let sp_count = species_counts.get(&sp_id).copied().unwrap_or(0.0);

            // Carrying capacity per species type (from config)
            let carrying_capacity = match species.species_type {
                SpeciesType::Plant => self.config.plant_carrying_capacity,
                SpeciesType::Herbivore => self.config.herbivore_carrying_capacity,
                SpeciesType::Carnivore => self.config.carnivore_carrying_capacity,
                SpeciesType::Decomposer => self.config.decomposer_carrying_capacity,
            };

            // Soft density-dependent factor with configurable theta
            let ratio = (sp_count / carrying_capacity).min(1.0);
            let density_factor = (1.0 - ratio.powf(self.config.density_theta)).max(0.01);

            // Allee effect: small populations have trouble reproducing, but with a floor
            let allee_factor = if sp_count < 5.0 && sp_count > 0.0 {
                (sp_count / 5.0).powi(2).max(0.15)
            } else {
                1.0
            };

            // Season modifier
            let season_mod = match (self.environment.season, species.species_type) {
                (0, _) => 1.5,
                (1, SpeciesType::Plant) => 1.2,
                (1, _) => 1.0,
                (2, _) => 0.7,
                (3, _) => 0.15,
                _ => 1.0,
            };

            // Rescue: endangered species get a reproduction boost
            let rescue = if sp_count > 0.0 && sp_count <= 3.0 {
                2.5
            } else {
                1.0
            };

            let effective_repro = species.repro_rate * density_factor * allee_factor * season_mod * rescue;

            if self.organisms[i].can_reproduce()
                && js_sys::Math::random() < effective_repro
                && self.organisms.len() + newborns.len() < MAX_ORGANISMS
            {
                let ox = self.organisms[i].x + (js_sys::Math::random() - 0.5) * 40.0;
                let oy = self.organisms[i].y + (js_sys::Math::random() - 0.5) * 40.0;
                let mut baby = Organism::new(species, ox, oy);
                baby.generation = self.organisms[i].generation + 1;

                if js_sys::Math::random() < self.config.mutation_rate {
                    let delta = (js_sys::Math::random() - 0.5) * 0.2;
                    baby.speed = (baby.speed + delta).max(0.1);
                    self.push_event(EventType::Evolution, format!("{} 发生突变: 速度 {}", species.name, if delta > 0.0 { "↑" } else { "↓" }));
                }

                baby.energy = 0.5;
                self.organisms[i].energy -= 0.35;
                newborns.push(baby);
            }
        }

        // Plant seed bank: plants can spontaneously appear when population is very low
        for sp in &self.species {
            if sp.species_type == SpeciesType::Plant {
                let sp_count = species_counts.get(&sp.id).copied().unwrap_or(0.0);
                if sp_count < 40.0 && sp_count > 0.0 && self.organisms.len() + newborns.len() < MAX_ORGANISMS {
                    if js_sys::Math::random() < 0.005 {
                        let x = 50.0 + js_sys::Math::random() * (WORLD_W - 100.0);
                        let y = 50.0 + js_sys::Math::random() * (WORLD_H - 100.0);
                        newborns.push(Organism::new(sp, x, y));
                    }
                }
            }
        }

        // Phase 3: Clean dead, check extinction
        let dead_species: Vec<u32> = self.organisms.iter()
            .filter(|o| !o.alive)
            .map(|o| o.species_id)
            .collect();

        self.organisms.retain(|o| o.alive);

        // Check for species extinction
        for sp_id in dead_species.iter().collect::<std::collections::HashSet<_>>() {
            let remaining = self.organisms.iter().filter(|o| o.species_id == *sp_id).count();
            if remaining == 0 {
                if let Some(sp) = species_map.get(sp_id) {
                    self.push_event(EventType::Death, format!("物种 \"{}\" 已灭绝！", sp.name));
                }
            }
        }

        // Update species population history
        for (idx, sp) in self.species.iter().enumerate() {
            let count = self.organisms.iter().filter(|o| o.species_id == sp.id).count() as u32;
            if let Some(history) = self.pop_history.get_mut(idx) {
                history.push(count);
                if history.len() > 300 {
                    history.remove(0);
                }
            }
        }

        // Add newborns
        self.organisms.extend(newborns);

        // Diversity index
        if self.tick % 10 == 0 {
            let total = self.organisms.len() as f64;
            if total > 0.0 {
                let mut h = 0.0;
                for sp in &self.species {
                    let count = self.organisms.iter().filter(|o| o.species_id == sp.id).count() as f64;
                    let p = count / total;
                    if p > 0.0 {
                        h -= p * p.ln();
                    }
                }
                self.diversity_history.push(h);
                if self.diversity_history.len() > 300 {
                    self.diversity_history.remove(0);
                }
            }
        }

        // Compute ecosystem maturity metrics (rolling window of 100 ticks)
        self.rolling_tpp.push(self.tick_primary_production);
        self.rolling_tr.push(self.tick_respiration.max(0.0001));
        if self.rolling_tpp.len() > 100 {
            self.rolling_tpp.remove(0);
            self.rolling_tr.remove(0);
        }

        let total_tpp: f64 = self.rolling_tpp.iter().sum();
        let total_tr: f64 = self.rolling_tr.iter().sum();

        // Total biomass = sum of all organism energy
        self.maturity.total_biomass = self.organisms.iter().map(|o| o.energy).sum();

        // TST/TB: metabolic rate (total throughput / total biomass)
        let tst = total_tpp + total_tr;
        self.maturity.metabolic_rate = if self.maturity.total_biomass > 0.0 {
            tst / self.maturity.total_biomass
        } else {
            0.0
        };

        // TPP/TR: production-respiration ratio
        self.maturity.production_respiration_ratio = if total_tr > 0.0 {
            total_tpp / total_tr
        } else {
            1.0
        };

        // Cycling index: decomposer nutrient return / total primary input
        let total_decomp: f64 = self.tick_decomposer_return; // simplified: current tick
        self.maturity.cycling_index = if total_tpp + total_decomp > 0.0 {
            total_decomp / (total_tpp + total_decomp) * 100.0
        } else {
            0.0
        };

        // Random events
        if self.tick % 500 == 0 {
            self.random_event();
        }
    }

    fn random_event(&mut self) {
        let r = js_sys::Math::random();
        if r < 0.33 {
            self.push_event(EventType::Death, "一场暴风雨席卷了生态系统".to_string());
            for o in &mut self.organisms {
                if o.alive && js_sys::Math::random() < 0.1 {
                    o.energy -= 0.3;
                }
            }
            self.environment.moisture = (self.environment.moisture + 0.2).min(1.0);
        } else if r < 0.66 {
            self.push_event(EventType::Birth, "阳光充足，植物疯长".to_string());
            for sp in &mut self.species {
                if sp.species_type == SpeciesType::Plant {
                    sp.repro_rate *= 1.3;
                }
            }
        } else {
            self.push_event(EventType::Death, "食物短缺，竞争加剧".to_string());
            self.environment.resources *= 0.7;
        }
    }

    pub fn trigger_event(&mut self, event_type: &str) {
        match event_type {
            "drought" => {
                self.push_event(EventType::Death, "严重干旱！水分急剧下降".to_string());
                self.environment.moisture = 0.1;
                let plant_ids: Vec<u32> = self.species.iter()
                    .filter(|s| s.species_type == SpeciesType::Plant)
                    .map(|s| s.id)
                    .collect();
                for o in &mut self.organisms {
                    if o.alive && plant_ids.contains(&o.species_id) {
                        o.energy -= 0.5;
                    }
                }
            }
            "flood" => {
                self.push_event(EventType::Death, "洪水泛滥！低洼区域被淹没".to_string());
                self.environment.moisture = 1.0;
                for o in &mut self.organisms {
                    if o.alive && o.y > WORLD_H * 0.7 {
                        o.energy -= 0.5;
                    }
                }
            }
            "plague" => {
                let targets: Vec<u32> = self.species.iter()
                    .filter(|s| s.species_type != SpeciesType::Plant)
                    .map(|s| s.id)
                    .collect();
                if !targets.is_empty() {
                    let idx = (js_sys::Math::random() * targets.len() as f64) as usize % targets.len();
                    let victim_id = targets[idx];
                    let victim_name = self.species.iter().find(|s| s.id == victim_id).map(|s| s.name.clone()).unwrap_or_default();
                    self.push_event(EventType::Death, format!("瘟疫袭击了 {}！", victim_name));
                    for o in &mut self.organisms {
                        if o.alive && o.species_id == victim_id && js_sys::Math::random() < 0.5 {
                            o.energy = 0.0;
                        }
                    }
                }
            }
            "boom" => {
                self.push_event(EventType::Birth, "资源大爆发！所有物种受益".to_string());
                self.environment.resources = 2.0;
                for o in &mut self.organisms {
                    if o.alive {
                        o.energy = (1.0_f64).min(o.energy + 0.3);
                    }
                }
            }
            "meteor" => {
                let cx = 200.0 + js_sys::Math::random() * (WORLD_W - 400.0);
                let cy = 200.0 + js_sys::Math::random() * (WORLD_H - 400.0);
                self.push_event(EventType::Death, "陨石撞击！附近生物大量死亡".to_string());
                for o in &mut self.organisms {
                    if o.alive {
                        let dx = o.x - cx;
                        let dy = o.y - cy;
                        if (dx * dx + dy * dy).sqrt() < 150.0 {
                            o.energy = 0.0;
                        }
                    }
                }
            }
            "iceage" => {
                self.push_event(EventType::Death, "冰期来临！温度骤降".to_string());
                self.environment.temperature = -15.0;
                let cold_weak: Vec<u32> = self.species.iter()
                    .filter(|s| s.traits.cold_resistance < 0.5)
                    .map(|s| s.id)
                    .collect();
                for o in &mut self.organisms {
                    if o.alive && cold_weak.contains(&o.species_id) {
                        o.energy -= 0.6;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn reset(&mut self) {
        self.species.clear();
        self.organisms.clear();
        self.events.clear();
        self.pop_history.clear();
        self.diversity_history.clear();
        self.tick = 0;
        self.species_counter = 0;
        self.maturity = EcosystemMaturity::default();
        self.tick_primary_production = 0.0;
        self.tick_respiration = 0.0;
        self.tick_decomposer_return = 0.0;
        self.rolling_tpp.clear();
        self.rolling_tr.clear();
        self.environment = Environment::new();
        self.init_defaults();
    }

    pub fn init_defaults(&mut self) {
        self.add_species("青草", SpeciesType::Plant, (39, 174, 96), 120, 0.04, 0.0, 0.8);
        self.add_species("野花", SpeciesType::Plant, (243, 156, 18), 60, 0.03, 0.0, 0.6);
        self.add_species("蘑菇", SpeciesType::Decomposer, (142, 68, 173), 20, 0.015, 0.3, 0.5);
        self.add_species("兔子", SpeciesType::Herbivore, (52, 152, 219), 25, 0.025, 2.0, 0.6);
        self.add_species("鹿", SpeciesType::Herbivore, (26, 188, 156), 15, 0.015, 1.5, 0.5);
        self.add_species("狐狸", SpeciesType::Carnivore, (231, 76, 60), 6, 0.01, 2.2, 0.7);
        self.add_species("狼", SpeciesType::Carnivore, (192, 57, 43), 3, 0.008, 2.5, 0.8);
    }

    pub fn total_population(&self) -> usize {
        self.organisms.len()
    }

    pub fn species_population(&self, species_id: u32) -> usize {
        self.organisms.iter().filter(|o| o.species_id == species_id).count()
    }
}
