

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpeciesType {
    Plant,
    Herbivore,
    Carnivore,
    Decomposer,
}

impl SpeciesType {
    pub fn name(&self) -> &str {
        match self {
            Self::Plant => "生产者",
            Self::Herbivore => "初级消费者",
            Self::Carnivore => "高级消费者",
            Self::Decomposer => "分解者",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Traits {
    pub cold_resistance: f64,
    pub drought_resistance: f64,
    pub speed: f64,
    pub size: f64,
    pub sense_range: f64,
}

#[derive(Clone, Debug)]
pub struct Species {
    pub id: u32,
    pub name: String,
    pub species_type: SpeciesType,
    pub color: (u8, u8, u8),
    pub repro_rate: f64,
    pub efficiency: f64,
    pub traits: Traits,
    pub diet: Vec<u32>,
    pub symbiosis: Vec<(u32, SymbiosisType)>,
    pub competition: Vec<u32>,
    pub generation_count: u32,
}

#[derive(Clone, Copy, Debug)]
pub enum SymbiosisType {
    Mutualism,
    Commensalism,
}

impl Species {
    pub fn new(
        name: &str,
        species_type: SpeciesType,
        color: (u8, u8, u8),
        repro_rate: f64,
        speed: f64,
        efficiency: f64,
        next_id: u32,
    ) -> Self {
        let id = next_id;

        let (sense, size) = match species_type {
            SpeciesType::Plant => (0.0, 8.0),
            SpeciesType::Herbivore => (80.0, 5.0),
            SpeciesType::Carnivore => (100.0, 5.0),
            SpeciesType::Decomposer => (40.0, 4.0),
        };

        Self {
            id,
            name: name.to_string(),
            species_type,
            color,
            repro_rate,
            efficiency,
            traits: Traits {
                cold_resistance: 0.3 + (js_sys::Math::random() * 0.5),
                drought_resistance: 0.2 + (js_sys::Math::random() * 0.5),
                speed,
                size,
                sense_range: sense,
            },
            diet: Vec::new(),
            symbiosis: Vec::new(),
            competition: Vec::new(),
            generation_count: 0,
        }
    }
}
