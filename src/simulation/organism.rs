use crate::simulation::species::Species;

#[derive(Clone, Debug)]
pub struct Organism {
    pub species_id: u32,
    pub x: f64,
    pub y: f64,
    pub energy: f64,
    pub age: u32,
    pub max_age: u32,
    pub size: f64,
    pub speed: f64,
    pub direction: f64,
    pub alive: bool,
    pub vx: f64,
    pub vy: f64,
    pub generation: u32,
    pub starving_ticks: u32,
    pub last_meal: u32,
}

impl Organism {
    pub fn new(species: &Species, x: f64, y: f64) -> Self {
        let max_age = match species.species_type {
            crate::simulation::species::SpeciesType::Plant => 500 + (js_sys::Math::random() * 1000.0) as u32,
            _ => 300 + (js_sys::Math::random() * 500.0) as u32,
        };

        Self {
            species_id: species.id,
            x,
            y,
            energy: 0.5 + js_sys::Math::random() * 0.5,
            age: 0,
            max_age,
            size: species.traits.size * (0.8 + js_sys::Math::random() * 0.4),
            speed: species.traits.speed * (0.8 + js_sys::Math::random() * 0.4),
            direction: js_sys::Math::random() * std::f64::consts::PI * 2.0,
            alive: true,
            vx: 0.0,
            vy: 0.0,
            generation: 0,
            starving_ticks: 0,
            last_meal: 0,
        }
    }

    pub fn update(&mut self, species: &Species, env_temp: f64, env_sunlight: f64, env_moisture: f64, wind_x: f64, wind_y: f64, world_w: f64, world_h: f64) {
        if !self.alive {
            return;
        }

        self.age += 1;
        self.energy -= 0.005 * (self.speed * 0.3 + 0.7);

        // Environmental effects on plants
        if species.species_type == crate::simulation::species::SpeciesType::Plant {
            self.energy += 0.01 * env_sunlight * env_moisture * species.traits.drought_resistance;
            if env_temp < 10.0 {
                self.energy -= 0.005 * (1.0 - species.traits.cold_resistance);
            }
        }

        // Movement for non-plants
        if species.species_type != crate::simulation::species::SpeciesType::Plant {
            self.move_creature(self.speed * self.energy, wind_x, wind_y);
        } else {
            self.size = (species.traits.size * 1.5).min(self.size + 0.002);
        }

        // Clamp to world
        self.x = self.x.clamp(5.0, world_w - 5.0);
        self.y = self.y.clamp(5.0, world_h - 5.0);

        // Death — starvation buffer instead of instant death
        if self.energy <= 0.0 {
            self.energy = 0.0;
            self.starving_ticks += 1;
            if self.starving_ticks > 20 {
                self.alive = false;
            }
        } else {
            self.starving_ticks = 0;
        }
        if self.age >= self.max_age {
            self.alive = false;
        }
    }

    fn move_creature(&mut self, speed: f64, wind_x: f64, wind_y: f64) {
        // Wander
        self.direction += (js_sys::Math::random() - 0.5) * 0.6;
        let target_vx = self.direction.cos() * speed * 0.5;
        let target_vy = self.direction.sin() * speed * 0.5;
        self.vx = lerp(self.vx, target_vx, 0.1);
        self.vy = lerp(self.vy, target_vy, 0.1);

        // Wind
        self.vx += wind_x * 0.02;
        self.vy += wind_y * 0.02;

        self.x += self.vx;
        self.y += self.vy;
    }

    pub fn chase(&mut self, target_x: f64, target_y: f64, speed: f64) {
        let dx = target_x - self.x;
        let dy = target_y - self.y;
        let d = (dx * dx + dy * dy).sqrt().max(1.0);
        let target_vx = (dx / d) * speed;
        let target_vy = (dy / d) * speed;
        self.vx = lerp(self.vx, target_vx, 0.15);
        self.vy = lerp(self.vy, target_vy, 0.15);
        self.x += self.vx;
        self.y += self.vy;
    }

    pub fn flee(&mut self, threat_x: f64, threat_y: f64, speed: f64) {
        let dx = self.x - threat_x;
        let dy = self.y - threat_y;
        let d = (dx * dx + dy * dy).sqrt().max(1.0);
        self.vx = (dx / d) * speed * 1.3;
        self.vy = (dy / d) * speed * 1.3;
        self.x += self.vx;
        self.y += self.vy;
    }

    pub fn can_reproduce(&self) -> bool {
        self.energy > 0.5 && self.age > 30
    }
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}
