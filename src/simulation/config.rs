#[derive(Clone, Debug)]
pub struct SimulationConfig {
    pub world_width: f64,
    pub world_height: f64,
    pub max_organisms: usize,
    pub base_metabolism: f64,
    pub herbivore_conversion: f64,
    pub carnivore_conversion: f64,
    pub decomposer_conversion: f64,
    pub reproduction_cost: f64,
    pub season_length: u32,
    pub density_theta: f64,
    pub mutation_rate: f64,
    pub plant_carrying_capacity: f64,
    pub herbivore_carrying_capacity: f64,
    pub carnivore_carrying_capacity: f64,
    pub decomposer_carrying_capacity: f64,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            world_width: 1200.0,
            world_height: 800.0,
            max_organisms: 2000,
            base_metabolism: 0.005,
            herbivore_conversion: 0.15,
            carnivore_conversion: 0.60,
            decomposer_conversion: 0.30,
            reproduction_cost: 0.35,
            season_length: 200,
            density_theta: 1.0,
            mutation_rate: 0.05,
            plant_carrying_capacity: 400.0,
            herbivore_carrying_capacity: 80.0,
            carnivore_carrying_capacity: 20.0,
            decomposer_carrying_capacity: 60.0,
        }
    }
}
