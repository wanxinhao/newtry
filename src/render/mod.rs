use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use crate::simulation::{World, SpeciesType};
use crate::simulation::world::{WORLD_W, WORLD_H};

pub struct Renderer {
    ctx: CanvasRenderingContext2d,
    scale: f64,
    offset_x: f64,
    offset_y: f64,
    w: f64,
    h: f64,
}

impl Renderer {
    pub fn new(canvas: &HtmlCanvasElement) -> Self {
        let ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        let w = canvas.width() as f64;
        let h = canvas.height() as f64;
        let scale = (w / WORLD_W).min(h / WORLD_H);
        let offset_x = (w - WORLD_W * scale) / 2.0;
        let offset_y = (h - WORLD_H * scale) / 2.0;

        Self { ctx, scale, offset_x, offset_y, w, h }
    }

    pub fn resize(&mut self, canvas: &HtmlCanvasElement) {
        let w = canvas.width() as f64;
        let h = canvas.height() as f64;
        self.w = w;
        self.h = h;
        self.scale = (w / WORLD_W).min(h / WORLD_H);
        self.offset_x = (w - WORLD_W * self.scale) / 2.0;
        self.offset_y = (h - WORLD_H * self.scale) / 2.0;
    }

    pub fn render(&self, world: &World) {
        let ctx = &self.ctx;

        ctx.clear_rect(0.0, 0.0, self.w, self.h);

        ctx.save();
        ctx.translate(self.offset_x, self.offset_y).unwrap();
        ctx.scale(self.scale, self.scale).unwrap();

        self.draw_background(ctx, world);
        self.draw_grid(ctx);
        self.draw_symbiosis_lines(ctx, world);
        self.draw_organisms(ctx, world);
        self.draw_effects(ctx, world);

        ctx.restore();
    }

    fn draw_background(&self, ctx: &CanvasRenderingContext2d, world: &World) {
        let (c1, c2) = match world.environment.season {
            0 => ("#0d1f0d", "#1a3a1a"), // spring
            1 => ("#1a1a0d", "#2a2a10"), // summer
            2 => ("#1a100d", "#2a1a10"), // autumn
            3 => ("#0d0d1a", "#101028"), // winter
            _ => ("#0a0e17", "#0a0e17"),
        };

        let grad = ctx.create_linear_gradient(0.0, 0.0, 0.0, WORLD_H);
        grad.add_color_stop(0.0, c1).unwrap();
        grad.add_color_stop(1.0, c2).unwrap();
        ctx.set_fill_style(&grad);
        ctx.fill_rect(0.0, 0.0, WORLD_W, WORLD_H);

        // Terrain dots
        ctx.set_fill_style_str("rgba(255,255,255,0.02)");
        for i in 0..8 {
            let x = ((i as f64 * 1.7 + world.tick as f64 * 0.001).sin() * 0.5 + 0.5) * WORLD_W;
            let y = ((i as f64 * 2.3 + world.tick as f64 * 0.0008).cos() * 0.5 + 0.5) * WORLD_H;
            let r = 80.0 + (i as f64 * 3.1).sin() * 40.0;
            ctx.begin_path();
            ctx.arc(x, y, r, 0.0, std::f64::consts::PI * 2.0).unwrap();
            ctx.fill();
        }
    }

    fn draw_grid(&self, ctx: &CanvasRenderingContext2d) {
        ctx.set_stroke_style_str("rgba(255,255,255,0.02)");
        ctx.set_line_width(0.5);
        let grid = 20.0;
        let mut x = 0.0;
        while x < WORLD_W {
            ctx.begin_path();
            ctx.move_to(x, 0.0);
            ctx.line_to(x, WORLD_H);
            ctx.stroke();
            x += grid;
        }
        let mut y = 0.0;
        while y < WORLD_H {
            ctx.begin_path();
            ctx.move_to(0.0, y);
            ctx.line_to(WORLD_W, y);
            ctx.stroke();
            y += grid;
        }
    }

    fn draw_symbiosis_lines(&self, ctx: &CanvasRenderingContext2d, world: &World) {
        for sp in &world.species {
            for &(partner_id, sym_type) in &sp.symbiosis {
                let partner = match world.species.iter().find(|s| s.id == partner_id) {
                    Some(s) => s,
                    None => continue,
                };

                let pop_a: Vec<_> = world.organisms.iter().filter(|o| o.alive && o.species_id == sp.id).collect();
                let pop_b: Vec<_> = world.organisms.iter().filter(|o| o.alive && o.species_id == partner.id).collect();

                if pop_a.is_empty() || pop_b.is_empty() {
                    continue;
                }

                let cxa: f64 = pop_a.iter().map(|o| o.x).sum::<f64>() / pop_a.len() as f64;
                let cya: f64 = pop_a.iter().map(|o| o.y).sum::<f64>() / pop_a.len() as f64;
                let cxb: f64 = pop_b.iter().map(|o| o.x).sum::<f64>() / pop_b.len() as f64;
                let cyb: f64 = pop_b.iter().map(|o| o.y).sum::<f64>() / pop_b.len() as f64;

                ctx.begin_path();
                let dash = js_sys::Array::new();
                dash.push(&4.0.into());
                dash.push(&4.0.into());
                ctx.set_line_dash(dash.as_ref()).unwrap();
                match sym_type {
                    crate::simulation::species::SymbiosisType::Mutualism => {
                        ctx.set_stroke_style_str("rgba(78,205,196,0.3)");
                    }
                    _ => {
                        ctx.set_stroke_style_str("rgba(255,230,109,0.2)");
                    }
                }
                ctx.set_line_width(1.0);
                ctx.move_to(cxa, cya);
                ctx.line_to(cxb, cyb);
                ctx.stroke();
                ctx.set_line_dash(js_sys::Array::new().as_ref()).unwrap();
            }
        }
    }

    fn draw_organisms(&self, ctx: &CanvasRenderingContext2d, world: &World) {
        let species_map: std::collections::HashMap<u32, &crate::simulation::Species> =
            world.species.iter().map(|s| (s.id, s)).collect();

        for o in &world.organisms {
            if !o.alive {
                continue;
            }

            let species = match species_map.get(&o.species_id) {
                Some(s) => s,
                None => continue,
            };

            let alpha = o.energy.clamp(0.3, 1.0);
            ctx.set_global_alpha(alpha);

            let (r, g, b) = species.color;
            let color = format!("rgb({},{},{})", r, g, b);

            match species.species_type {
                SpeciesType::Plant => {
                    self.draw_plant(ctx, o.x, o.y, o.size, &color, r, g, b, o.age);
                }
                SpeciesType::Herbivore => {
                    self.draw_herbivore(ctx, o.x, o.y, o.size, o.vx, o.vy, &color, r, g, b);
                }
                SpeciesType::Carnivore => {
                    self.draw_carnivore(ctx, o.x, o.y, o.size, o.vx, o.vy, &color, r, g, b);
                }
                SpeciesType::Decomposer => {
                    self.draw_decomposer(ctx, o.x, o.y, o.size, &color, r, g, b, o.age);
                }
            }

            ctx.set_global_alpha(1.0);
        }
    }

    fn draw_plant(&self, ctx: &CanvasRenderingContext2d, x: f64, y: f64, size: f64, _color: &str, r: u8, g: u8, b: u8, age: u32) {
        // Stem
        ctx.set_stroke_style_str(&format!("rgba({},{},{},0.5)", r, g, b));
        ctx.set_line_width(1.0);
        ctx.begin_path();
        ctx.move_to(x, y + size);
        ctx.line_to(x, y - size * 0.5);
        ctx.stroke();

        // Leaves
        ctx.set_fill_style_str(&format!("rgb({},{},{})", r, g, b));
        let leaf_count = 3 + size as usize;
        for i in 0..leaf_count {
            let angle = (i as f64 / leaf_count as f64) * std::f64::consts::PI * 2.0 + age as f64 * 0.01;
            let lx = x + angle.cos() * size * 0.7;
            let ly = y - size * 0.5 + angle.sin() * size * 0.5;
            ctx.begin_path();
            ctx.arc(lx, ly, size * 0.4, 0.0, std::f64::consts::PI * 2.0).unwrap();
            ctx.fill();
        }

        // Center
        let cr = (r as u16 + 40).min(255) as u8;
        let cg = (g as u16 + 40).min(255) as u8;
        let cb = (b as u16 + 20).min(255) as u8;
        ctx.set_fill_style_str(&format!("rgba({},{},{},0.8)", cr, cg, cb));
        ctx.begin_path();
        ctx.arc(x, y - size * 0.3, size * 0.3, 0.0, std::f64::consts::PI * 2.0).unwrap();
        ctx.fill();
    }

    fn draw_herbivore(&self, ctx: &CanvasRenderingContext2d, x: f64, y: f64, size: f64, vx: f64, vy: f64, _color: &str, r: u8, g: u8, b: u8) {
        let angle = vy.atan2(vx);

        ctx.save();
        ctx.translate(x, y).unwrap();
        ctx.rotate(angle).unwrap();

        // Body
        ctx.set_fill_style_str(&format!("rgb({},{},{})", r, g, b));
        ctx.begin_path();
        ctx.ellipse(0.0, 0.0, size * 1.3, size * 0.8, 0.0, 0.0, std::f64::consts::PI * 2.0).unwrap();
        ctx.fill();

        // Head
        let cr = (r as u16 + 30).min(255) as u8;
        let cg = (g as u16 + 30).min(255) as u8;
        let cb = (b as u16 + 30).min(255) as u8;
        ctx.set_fill_style_str(&format!("rgba({},{},{},0.9)", cr, cg, cb));
        ctx.begin_path();
        ctx.arc(size * 1.2, 0.0, size * 0.5, 0.0, std::f64::consts::PI * 2.0).unwrap();
        ctx.fill();

        // Eye
        ctx.set_fill_style_str("#fff");
        ctx.begin_path();
        ctx.arc(size * 1.4, -size * 0.15, size * 0.12, 0.0, std::f64::consts::PI * 2.0).unwrap();
        ctx.fill();

        ctx.restore();
    }

    fn draw_carnivore(&self, ctx: &CanvasRenderingContext2d, x: f64, y: f64, size: f64, vx: f64, vy: f64, _color: &str, r: u8, g: u8, b: u8) {
        let angle = vy.atan2(vx);

        ctx.save();
        ctx.translate(x, y).unwrap();
        ctx.rotate(angle).unwrap();

        // Body (sleek shape)
        ctx.set_fill_style_str(&format!("rgb({},{},{})", r, g, b));
        ctx.begin_path();
        ctx.move_to(size * 1.5, 0.0);
        ctx.line_to(-size, -size * 0.7);
        ctx.line_to(-size * 1.2, 0.0);
        ctx.line_to(-size, size * 0.7);
        ctx.close_path();
        ctx.fill();

        // Eyes (glowing)
        ctx.set_fill_style_str("#ff0");
        ctx.begin_path();
        ctx.arc(size * 0.8, -size * 0.2, size * 0.15, 0.0, std::f64::consts::PI * 2.0).unwrap();
        ctx.fill();
        ctx.begin_path();
        ctx.arc(size * 0.8, size * 0.2, size * 0.15, 0.0, std::f64::consts::PI * 2.0).unwrap();
        ctx.fill();

        ctx.restore();
    }

    fn draw_decomposer(&self, ctx: &CanvasRenderingContext2d, x: f64, y: f64, size: f64, _color: &str, r: u8, g: u8, b: u8, age: u32) {
        // Amoeba shape
        ctx.set_fill_style_str(&format!("rgba({},{},{},0.7)", r, g, b));
        ctx.begin_path();
        for i in 0..8 {
            let angle = (i as f64 / 8.0) * std::f64::consts::PI * 2.0;
            let wobble = (age as f64 * 0.05 + i as f64).sin() * size * 0.3;
            let px = x + angle.cos() * (size + wobble);
            let py = y + angle.sin() * (size + wobble);
            if i == 0 {
                ctx.move_to(px, py);
            } else {
                ctx.line_to(px, py);
            }
        }
        ctx.close_path();
        ctx.fill();

        // Nucleus
        let cr = (r as u16 + 60).min(255) as u8;
        let cg = (g as u16 + 60).min(255) as u8;
        let cb = (b as u16 + 60).min(255) as u8;
        ctx.set_fill_style_str(&format!("rgba({},{},{},0.6)", cr, cg, cb));
        ctx.begin_path();
        ctx.arc(x, y, size * 0.3, 0.0, std::f64::consts::PI * 2.0).unwrap();
        ctx.fill();
    }

    fn draw_effects(&self, ctx: &CanvasRenderingContext2d, world: &World) {
        let env = &world.environment;

        // Rain
        if env.moisture > 0.7 {
            ctx.set_stroke_style_str("rgba(100,150,255,0.1)");
            ctx.set_line_width(0.5);
            for i in 0..30 {
                let x = js_sys::Math::random() * WORLD_W;
                let y = js_sys::Math::random() * WORLD_H;
                ctx.begin_path();
                ctx.move_to(x, y);
                ctx.line_to(x + env.wind_x * 3.0, y + 8.0);
                ctx.stroke();
            }
        }

        // Snow
        if env.temperature < 5.0 {
            ctx.set_fill_style_str("rgba(255,255,255,0.15)");
            for i in 0..20 {
                let x = ((world.tick as f64 * 0.01 + i as f64 * 1.7).sin() * 0.5 + 0.5) * WORLD_W;
                let y = (world.tick as f64 * 0.3 + i as f64 * 47.0) % WORLD_H;
                ctx.begin_path();
                ctx.arc(x, y, 1.5, 0.0, std::f64::consts::PI * 2.0).unwrap();
                ctx.fill();
            }
        }

        // Sun glow
        if env.sunlight > 0.7 {
            let grad = ctx.create_radial_gradient(WORLD_W * 0.8, 20.0, 0.0, WORLD_W * 0.8, 20.0, 200.0).unwrap();
            let alpha = env.sunlight * 0.08;
            grad.add_color_stop(0.0, &format!("rgba(255,220,100,{})", alpha)).unwrap();
            grad.add_color_stop(1.0, "rgba(255,220,100,0)").unwrap();
            ctx.set_fill_style(&grad);
            ctx.fill_rect(0.0, 0.0, WORLD_W, WORLD_H);
        }
    }
}
