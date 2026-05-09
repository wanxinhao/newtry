#[derive(Clone, Debug)]
pub struct Environment {
    pub temperature: f64,
    pub moisture: f64,
    pub sunlight: f64,
    pub wind_x: f64,
    pub wind_y: f64,
    pub season: u8, // 0=spring, 1=summer, 2=autumn, 3=winter
    pub day: u32,
    pub year: u32,
    pub resources: f64,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            temperature: 22.0,
            moisture: 0.7,
            sunlight: 0.8,
            wind_x: 0.0,
            wind_y: 0.0,
            season: 0,
            day: 0,
            year: 0,
            resources: 1.0,
        }
    }

    pub fn update(&mut self) {
        self.day += 1;
        if self.day % 200 == 0 {
            self.season = (self.season + 1) % 4;
            if self.season == 0 {
                self.year += 1;
            }
        }

        let phase = (self.day % 200) as f64 / 200.0;
        match self.season {
            0 => { // Spring
                self.temperature = lerp(10.0, 22.0, phase);
                self.moisture = lerp(0.6, 0.8, phase);
                self.sunlight = lerp(0.5, 0.8, phase);
            }
            1 => { // Summer
                self.temperature = lerp(22.0, 35.0, phase);
                self.moisture = lerp(0.8, 0.4, phase);
                self.sunlight = lerp(0.8, 1.0, phase);
            }
            2 => { // Autumn
                self.temperature = lerp(35.0, 15.0, phase);
                self.moisture = lerp(0.4, 0.6, phase);
                self.sunlight = lerp(1.0, 0.5, phase);
            }
            3 => { // Winter
                self.temperature = lerp(15.0, -5.0, phase);
                self.moisture = lerp(0.6, 0.8, phase);
                self.sunlight = lerp(0.5, 0.3, phase);
            }
            _ => {}
        }

        // Random wind
        self.wind_x = lerp(self.wind_x, (js_sys::Math::random() - 0.5) * 2.0, 0.05);
        self.wind_y = lerp(self.wind_y, (js_sys::Math::random() - 0.5) * 2.0, 0.05);

        // Resources regenerate
        self.resources = (self.resources + 0.001).clamp(0.1, 2.0);
    }

    pub fn season_name(&self) -> &str {
        match self.season {
            0 => "春",
            1 => "夏",
            2 => "秋",
            3 => "冬",
            _ => "?",
        }
    }
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}
