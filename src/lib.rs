use leptos::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

mod simulation;
mod render;

use simulation::{World, SpeciesType};
use simulation::world::EventType;
use render::Renderer;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let world = Rc::new(RefCell::new(World::new()));
    world.borrow_mut().init_defaults();

    let (tick, set_tick) = signal(0u32);
    let (population, set_population) = signal(0usize);
    let (season, set_season) = signal("春".to_string());
    let (year, set_year) = signal(0u32);
    let (temperature, set_temperature) = signal(22.0f64);
    let (moisture, set_moisture) = signal(70.0f64);
    let (sunlight, set_sunlight) = signal(80.0f64);
    let (events_text, set_events_text) = signal(String::new());
    let (species_html, set_species_html) = signal(String::new());
    let (detail_html, set_detail_html) = signal("点击物种查看详情".to_string());
    let (metabolic_rate, set_metabolic_rate) = signal(0.0f64);
    let (tpp_tr_ratio, set_tpp_tr_ratio) = signal(1.0f64);
    let (cycling_index, set_cycling_index) = signal(0.0f64);
    let (settings_open, set_settings_open) = signal(false);

    // Setup canvas rendering — deferred to after DOM mount
    let renderer: Rc<RefCell<Option<Renderer>>> = Rc::new(RefCell::new(None));

    let renderer_init = renderer.clone();
    let world_loop = world.clone();
    let renderer_loop = renderer.clone();

    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::new(move || {
        // Initialize renderer on first frame (canvas must be in DOM)
        if renderer_loop.borrow().is_none() {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            if let Some(el) = document.get_element_by_id("mainCanvas") {
                let canvas: web_sys::HtmlCanvasElement = el.dyn_into().unwrap();
                let parent = canvas.parent_element().unwrap();
                let rect = parent.get_bounding_client_rect();
                let dpr = window.device_pixel_ratio();
                canvas.set_width((rect.width() * dpr) as u32);
                canvas.set_height((rect.height() * dpr) as u32);
                *renderer_init.borrow_mut() = Some(Renderer::new(&canvas));
            } else {
                // Canvas not ready yet, retry next frame
                web_sys::window().unwrap()
                    .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
                    .unwrap();
                return;
            }
        }

        let mut w = world_loop.borrow_mut();
        if w.running {
            let spd = w.speed;
            for _ in 0..spd {
                w.update();
            }
            set_tick.set(w.tick);
            set_population.set(w.total_population());
            set_season.set(w.environment.season_name().to_string());
            set_year.set(w.environment.year);
            set_temperature.set(w.environment.temperature);
            set_moisture.set(w.environment.moisture * 100.0);
            set_sunlight.set(w.environment.sunlight * 100.0);

            // Update maturity metrics
            set_metabolic_rate.set(w.maturity.metabolic_rate);
            set_tpp_tr_ratio.set(w.maturity.production_respiration_ratio);
            set_cycling_index.set(w.maturity.cycling_index);

            // Update events
            let evt_text = w.events.iter().take(15).map(|e| {
                let cls = match e.event_type {
                    EventType::Birth => "birth",
                    EventType::Death => "death",
                    EventType::Evolution => "evolution",
                    EventType::Symbiosis => "symbiosis",
                };
                format!("<div class=\"event {}\"><span class=\"time\">{}年</span><span class=\"msg\">{}</span></div>",
                    cls, e.tick / 200, e.message)
            }).collect::<Vec<_>>().join("");
            set_events_text.set(evt_text);

            // Update species list
            let sp_html = w.species.iter().map(|sp| {
                let count = w.species_population(sp.id);
                let type_name = match sp.species_type {
                    SpeciesType::Plant => "生产者",
                    SpeciesType::Herbivore => "消费者",
                    SpeciesType::Carnivore => "捕食者",
                    SpeciesType::Decomposer => "分解者",
                };
                let (r, g, b) = sp.color;
                format!(r#"<div class="species-card" onclick="selectSpecies({})">
                    <div><span class="dot" style="background:rgb({},{},{})"></span><span class="name">{}</span></div>
                    <div class="info">{} · {} · 第{}代</div>
                    <div class="pop-bar"><div class="fill" style="width:{}%;background:rgb({},{},{})"></div></div>
                </div>"#,
                    sp.id, r, g, b, sp.name, type_name, count, sp.generation_count,
                    (count as f64 / 100.0 * 100.0).min(100.0), r, g, b)
            }).collect::<Vec<_>>().join("");
            set_species_html.set(sp_html);
        }

        // Render
        if let Some(ref r) = *renderer_loop.borrow() {
            r.render(&w);
        }

        // Request next frame
        web_sys::window().unwrap()
            .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();
    }));

    // Start the loop after a short delay to let the view mount
    let g_start = g.clone();
    let init_closure = Closure::once(Box::new(move || {
        web_sys::window().unwrap()
            .request_animation_frame(g_start.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();
    }) as Box<dyn FnOnce()>);
    web_sys::window().unwrap()
        .set_timeout_with_callback_and_timeout_and_arguments_0(init_closure.as_ref().unchecked_ref(), 0)
        .unwrap();
    init_closure.forget();

    // Select species handler (JS callback)
    let world_select = world.clone();
    let select_callback = Closure::wrap(Box::new(move |id: u32| {
        let w = world_select.borrow();
        if let Some(sp) = w.species.iter().find(|s| s.id == id) {
            let count = w.species_population(sp.id);
            let type_name = match sp.species_type {
                SpeciesType::Plant => "生产者",
                SpeciesType::Herbivore => "初级消费者",
                SpeciesType::Carnivore => "高级消费者",
                SpeciesType::Decomposer => "分解者",
            };
            let (r, g, b) = sp.color;
            let diet_names: Vec<String> = sp.diet.iter().filter_map(|did| {
                w.species.iter().find(|s| s.id == *did).map(|s| s.name.clone())
            }).collect();
            let detail = format!(
                r#"<div style="margin-bottom:10px;display:flex;align-items:center;gap:8px;">
                    <span style="display:inline-block;width:8px;height:8px;border-radius:50%;background:rgb({},{},{});box-shadow:0 0 8px rgba({},{},{},0.4);"></span>
                    <strong style="color:rgb({},{});font-size:13px;">{}</strong>
                    <span style="color:#71717a;font-size:11px;">{}</span>
                </div>
                <div class="env-stat"><span class="label">种群数量</span><span class="value">{}</span></div>
                <div class="env-stat"><span class="label">繁殖率</span><span class="value">{:.1}%</span></div>
                <div class="env-stat"><span class="label">速度</span><span class="value">{:.2}</span></div>
                <div class="env-stat"><span class="label">抗寒性</span><span class="value">{:.0}%</span></div>
                <div class="env-stat"><span class="label">抗旱性</span><span class="value">{:.0}%</span></div>
                <div class="env-stat"><span class="label">突变代数</span><span class="value">{}</span></div>
                <div class="env-stat"><span class="label">食物来源</span><span class="value">{}</span></div>"#,
                r, g, b, r, g, b, r, g, sp.name, type_name,
                count, sp.repro_rate * 100.0, sp.traits.speed,
                sp.traits.cold_resistance * 100.0, sp.traits.drought_resistance * 100.0,
                sp.generation_count,
                if diet_names.is_empty() { "无".to_string() } else { diet_names.join(", ") }
            );
            // Update DOM directly
            if let Some(el) = web_sys::window().unwrap().document().unwrap().get_element_by_id("speciesDetail") {
                el.set_inner_html(&detail);
            }
        }
    }) as Box<dyn FnMut(u32)>);

    let js_fn: &js_sys::Function = select_callback.as_ref().unchecked_ref();
    let global = js_sys::global();
    js_sys::Reflect::set(&global, &"selectSpecies".into(), js_fn).unwrap();
    select_callback.forget();

    // Clone Rc for settings panel closures
    let w_cfg1 = world.clone(); let w_cfg2 = world.clone();
    let w_cfg3 = world.clone(); let w_cfg4 = world.clone();
    let w_cfg5 = world.clone(); let w_cfg6 = world.clone();
    let w_cfg7 = world.clone(); let w_cfg8 = world.clone();
    let w_cfg9 = world.clone(); let w_cfg10 = world.clone();
    let w_cfg11 = world.clone(); let w_cfg12 = world.clone();
    let w_cfg13 = world.clone(); let w_cfg14 = world.clone();
    let w_cfg15 = world.clone(); let w_cfg16 = world.clone();
    let w_cfg17 = world.clone(); let w_cfg18 = world.clone();
    let w_cfg19 = world.clone(); let w_cfg20 = world.clone();
    let w_cfg21 = world.clone(); let w_cfg22 = world.clone();
    let w_cfg23 = world.clone(); let w_cfg24 = world.clone();

    view! {
        <div id="app">
            <div class="header">
                <h1>"万 物 共 生"</h1>
                <div class="header-controls">
                    <button id="btnPlay" class="active" on:click={
                        let w = world.clone();
                        move |_| {
                            let mut w = w.borrow_mut();
                            w.running = !w.running;
                        }
                    }>"▶ 播放"</button>
                    <button on:click={
                        let w = world.clone();
                        move |_| { w.borrow_mut().update(); }
                    }>"⏭ 单步"</button>
                    <button on:click={
                        let w = world.clone();
                        move |_| { w.borrow_mut().reset(); }
                    }>"↺ 重置"</button>
                    <div class="speed-control">
                        <span>"速度"</span>
                        <input type="range" min="1" max="10" value="3"
                            on:input={
                                let w = world.clone();
                                move |ev: web_sys::Event| {
                                    let target = ev.target().unwrap();
                                    let input: web_sys::HtmlInputElement = target.unchecked_into();
                                    let val = input.value().parse::<u32>().unwrap_or(3);
                                    w.borrow_mut().speed = val;
                                }
                            }
                        />
                        <span id="speedLabel">{move || format!("{}x", 3)}</span>
                    </div>
                    <button on:click={
                        let w = world.clone();
                        move |_| {
                            w.borrow_mut().add_species("新物种", SpeciesType::Herbivore, (78, 205, 196), 20, 0.02, 1.5, 0.6);
                        }
                    }>"+ 添加物种"</button>
                    <button on:click={
                        let w = world.clone();
                        move |_| {
                            w.borrow_mut().trigger_event("boom");
                        }
                    }>"⚡ 环境事件"</button>
                    <button on:click={
                        move |_| {
                            set_settings_open.update(|v| *v = !*v);
                        }
                    }>"⚙ 设置"</button>
                </div>
            </div>

            <div class="left-panel">
                <div class="panel-section">
                    <h3>"物种概览"</h3>
                    <div id="speciesList" inner_html={move || species_html.get()}></div>
                </div>
                <div class="panel-section">
                    <h3>"环境状态"</h3>
                    <div>
                        <div class="env-stat">
                            <span class="label">"季节"</span>
                            <span class="value">{move || season.get()}</span>
                        </div>
                        <div class="env-stat">
                            <span class="label">"年份"</span>
                            <span class="value">{move || format!("第 {} 年", year.get())}</span>
                        </div>
                        <div class="env-stat">
                            <span class="label">"温度"</span>
                            <span class="value">{move || format!("{:.1}°C", temperature.get())}</span>
                        </div>
                        <div class="env-stat">
                            <span class="label">"湿度"</span>
                            <span class="value">{move || format!("{:.0}%", moisture.get())}</span>
                        </div>
                        <div class="env-stat">
                            <span class="label">"光照"</span>
                            <span class="value">{move || format!("{:.0}%", sunlight.get())}</span>
                        </div>
                    </div>
                </div>
                <div class="panel-section">
                    <h3>"系统成熟度"</h3>
                    <div>
                        <div class="env-stat">
                            <span class="label">"代谢率 (TST/TB)"</span>
                            <span class="value">{move || format!("{:.2}", metabolic_rate.get())}</span>
                        </div>
                        <div class="env-stat">
                            <span class="label">"产消比 (TPP/TR)"</span>
                            <span class="value">{move || format!("{:.2}", tpp_tr_ratio.get())}</span>
                        </div>
                        <div class="env-stat">
                            <span class="label">"循环指数"</span>
                            <span class="value">{move || format!("{:.1}%", cycling_index.get())}</span>
                        </div>
                    </div>
                </div>
            </div>

            <div class="canvas-area">
                <canvas id="mainCanvas"></canvas>
                <div class="canvas-overlay">
                    <div class="overlay-badge">"时间: " <span>{move || format!("第 {} 天", tick.get())}</span></div>
                    <div class="overlay-badge">"季节: " <span>{move || season.get()}</span></div>
                    <div class="overlay-badge">"种群: " <span>{move || population.get()}</span></div>
                </div>
            </div>

            <div class="right-panel">
                <div class="panel-section">
                    <h3>"演化日志"</h3>
                    <div class="event-log" inner_html={move || events_text.get()}></div>
                </div>
                <div class="panel-section">
                    <h3>"物种详情"</h3>
                    <div id="speciesDetail" style="font-size:11.5px;color:#71717a;">
                        {move || detail_html.get()}
                    </div>
                </div>
            </div>

            <div class="status-bar">
                <div class="left">
                    <span><span class="indicator"></span>"万物共生 - 生命演化模拟器"</span>
                </div>
                <div class="right">
                    <span>{move || format!("种群: {}", population.get())}</span>
                </div>
            </div>

            // Settings overlay
            <div class={move || if settings_open.get() { "settings-overlay open" } else { "settings-overlay" }}
                on:click=move |_| set_settings_open.set(false)>
            </div>
            <div class={move || if settings_open.get() { "settings-panel open" } else { "settings-panel" }}>
                <h2>
                    <span>"模拟参数设置"</span>
                    <button class="close-btn" on:click=move |_| set_settings_open.set(false)>"✕"</button>
                </h2>

                <div class="settings-group">
                    <h4>"能量与代谢"</h4>
                    <div class="settings-row">
                        <label>"基础代谢率"</label>
                        <div class="input-group">
                            <input type="range" min="0.001" max="0.015" step="0.001" value="0.005"
                                on:input=move |ev| { if let Ok(mut w) = w_cfg1.try_borrow_mut() { w.config.base_metabolism = event_value_f64(&ev); } }/>
                            <input type="number" min="0.001" max="0.015" step="0.001" value="0.005"
                                on:change=move |ev| { if let Ok(mut w) = w_cfg2.try_borrow_mut() { w.config.base_metabolism = event_value_f64(&ev); } }/>
                        </div>
                    </div>
                    <div class="settings-row">
                        <label>"草食转化率"</label>
                        <div class="input-group">
                            <input type="range" min="0.05" max="0.35" step="0.01" value="0.15"
                                on:input=move |ev| { if let Ok(mut w) = w_cfg3.try_borrow_mut() { w.config.herbivore_conversion = event_value_f64(&ev); } }/>
                            <input type="number" min="0.05" max="0.35" step="0.01" value="0.15"
                                on:change=move |ev| { if let Ok(mut w) = w_cfg4.try_borrow_mut() { w.config.herbivore_conversion = event_value_f64(&ev); } }/>
                        </div>
                    </div>
                    <div class="settings-row">
                        <label>"肉食转化率"</label>
                        <div class="input-group">
                            <input type="range" min="0.3" max="0.9" step="0.05" value="0.60"
                                on:input=move |ev| { if let Ok(mut w) = w_cfg5.try_borrow_mut() { w.config.carnivore_conversion = event_value_f64(&ev); } }/>
                            <input type="number" min="0.3" max="0.9" step="0.05" value="0.60"
                                on:change=move |ev| { if let Ok(mut w) = w_cfg6.try_borrow_mut() { w.config.carnivore_conversion = event_value_f64(&ev); } }/>
                        </div>
                    </div>
                    <div class="settings-row">
                        <label>"繁殖能量消耗"</label>
                        <div class="input-group">
                            <input type="range" min="0.1" max="0.6" step="0.05" value="0.35"
                                on:input=move |ev| { if let Ok(mut w) = w_cfg7.try_borrow_mut() { w.config.reproduction_cost = event_value_f64(&ev); } }/>
                            <input type="number" min="0.1" max="0.6" step="0.05" value="0.35"
                                on:change=move |ev| { if let Ok(mut w) = w_cfg8.try_borrow_mut() { w.config.reproduction_cost = event_value_f64(&ev); } }/>
                        </div>
                    </div>
                </div>

                <div class="settings-group">
                    <h4>"种群动力学"</h4>
                    <div class="settings-row">
                        <label>"密度制约强度"</label>
                        <div class="input-group">
                            <input type="range" min="0.3" max="3.0" step="0.1" value="1.0"
                                on:input=move |ev| { if let Ok(mut w) = w_cfg9.try_borrow_mut() { w.config.density_theta = event_value_f64(&ev); } }/>
                            <input type="number" min="0.3" max="3.0" step="0.1" value="1.0"
                                on:change=move |ev| { if let Ok(mut w) = w_cfg10.try_borrow_mut() { w.config.density_theta = event_value_f64(&ev); } }/>
                        </div>
                    </div>
                    <div class="settings-row">
                        <label>"突变率"</label>
                        <div class="input-group">
                            <input type="range" min="0.0" max="0.2" step="0.01" value="0.05"
                                on:input=move |ev| { if let Ok(mut w) = w_cfg11.try_borrow_mut() { w.config.mutation_rate = event_value_f64(&ev); } }/>
                            <input type="number" min="0.0" max="0.2" step="0.01" value="0.05"
                                on:change=move |ev| { if let Ok(mut w) = w_cfg12.try_borrow_mut() { w.config.mutation_rate = event_value_f64(&ev); } }/>
                        </div>
                    </div>
                    <div class="settings-row">
                        <label>"最大生物数"</label>
                        <div class="input-group">
                            <input type="range" min="500" max="5000" step="100" value="2000"
                                on:input=move |ev| { if let Ok(mut w) = w_cfg13.try_borrow_mut() { w.config.max_organisms = event_value_f64(&ev) as usize; } }/>
                            <input type="number" min="500" max="5000" step="100" value="2000"
                                on:change=move |ev| { if let Ok(mut w) = w_cfg14.try_borrow_mut() { w.config.max_organisms = event_value_f64(&ev) as usize; } }/>
                        </div>
                    </div>
                </div>

                <div class="settings-group">
                    <h4>"环境承载力"</h4>
                    <div class="settings-row">
                        <label>"生产者"</label>
                        <div class="input-group">
                            <input type="range" min="100" max="800" step="50" value="400"
                                on:input=move |ev| { if let Ok(mut w) = w_cfg15.try_borrow_mut() { w.config.plant_carrying_capacity = event_value_f64(&ev); } }/>
                            <input type="number" min="100" max="800" step="50" value="400"
                                on:change=move |ev| { if let Ok(mut w) = w_cfg16.try_borrow_mut() { w.config.plant_carrying_capacity = event_value_f64(&ev); } }/>
                        </div>
                    </div>
                    <div class="settings-row">
                        <label>"消费者"</label>
                        <div class="input-group">
                            <input type="range" min="20" max="200" step="10" value="80"
                                on:input=move |ev| { if let Ok(mut w) = w_cfg17.try_borrow_mut() { w.config.herbivore_carrying_capacity = event_value_f64(&ev); } }/>
                            <input type="number" min="20" max="200" step="10" value="80"
                                on:change=move |ev| { if let Ok(mut w) = w_cfg18.try_borrow_mut() { w.config.herbivore_carrying_capacity = event_value_f64(&ev); } }/>
                        </div>
                    </div>
                    <div class="settings-row">
                        <label>"捕食者"</label>
                        <div class="input-group">
                            <input type="range" min="5" max="60" step="5" value="20"
                                on:input=move |ev| { if let Ok(mut w) = w_cfg19.try_borrow_mut() { w.config.carnivore_carrying_capacity = event_value_f64(&ev); } }/>
                            <input type="number" min="5" max="60" step="5" value="20"
                                on:change=move |ev| { if let Ok(mut w) = w_cfg20.try_borrow_mut() { w.config.carnivore_carrying_capacity = event_value_f64(&ev); } }/>
                        </div>
                    </div>
                    <div class="settings-row">
                        <label>"分解者"</label>
                        <div class="input-group">
                            <input type="range" min="20" max="150" step="10" value="60"
                                on:input=move |ev| { if let Ok(mut w) = w_cfg21.try_borrow_mut() { w.config.decomposer_carrying_capacity = event_value_f64(&ev); } }/>
                            <input type="number" min="20" max="150" step="10" value="60"
                                on:change=move |ev| { if let Ok(mut w) = w_cfg22.try_borrow_mut() { w.config.decomposer_carrying_capacity = event_value_f64(&ev); } }/>
                        </div>
                    </div>
                </div>

                <div class="settings-group">
                    <h4>"物种繁殖率调节"</h4>
                    <div class="settings-row">
                        <label>"全局倍率"</label>
                        <div class="input-group">
                            <input type="range" min="0.1" max="3.0" step="0.1" value="1.0"
                                on:input=move |ev| {
                                    if let Ok(mut w) = w_cfg23.try_borrow_mut() {
                                        let v = event_value_f64(&ev);
                                        for sp in &mut w.species { sp.repro_rate = (sp.repro_rate * v).min(0.2); }
                                    }
                                }/>
                            <span class="unit">"×"</span>
                        </div>
                    </div>
                </div>

                <button class="settings-apply-btn" on:click=move |_| set_settings_open.set(false)>"应用并关闭"</button>
                <button class="settings-reset-btn" on:click=move |_| {
                    if let Ok(mut w) = w_cfg24.try_borrow_mut() { w.config = simulation::SimulationConfig::default(); }
                    set_settings_open.set(false);
                }>"恢复默认参数"</button>
            </div>
        </div>
    }
}

fn event_value_f64(ev: &web_sys::Event) -> f64 {
    let target = ev.target().unwrap();
    let input: web_sys::HtmlInputElement = target.unchecked_into();
    input.value().parse::<f64>().unwrap_or(0.0)
}
