use covid;
use gl;
use std::time::Instant;

use imgui::im_str;
// We need our dependencies to match exactly,
// so turn this off and use the private export directly.
#[allow(pub_use_of_private_extern_crate)]
use imgui_sdl2::imgui;

mod renderer;
use renderer::Renderer;

/// A container for state that's passed into ImGui.
/// Because we're passing pointers here into C++ code,
/// we will only use this through Box
#[derive(Debug)]
struct UiState {
    show_demo_window: bool,
    frame_metrics: FrameMetrics,
    simulation_params: SimulationParams,
}

impl UiState {
    fn new() -> Box<UiState> {
        Box::new(UiState {
            show_demo_window: true,
            frame_metrics: FrameMetrics::default(),
            simulation_params: SimulationParams::default(),
        })
    }

    fn build_ui(&mut self, ui: &imgui::Ui) {
        use imgui::*;

        ui.main_menu_bar(|| {
            ui.menu(im_str!("Menu"), true, || {
                MenuItem::new(im_str!("Foo")).build(ui);
                MenuItem::new(im_str!("Bar")).build(ui);
                ui.separator();

                MenuItem::new(im_str!("Baz")).build(ui);
                MenuItem::new(im_str!("Secret")).enabled(false).build(ui);
            });
        });

        ui.show_demo_window(&mut true);

        Window::new(im_str!("Covid-19 Knobs")).build(ui, || {
            if ui
                .collapsing_header(im_str!("Frame Metrics"))
                .default_open(true)
                .build()
            {
                self.frame_metrics.build_ui(ui);
            }

            if ui
                .collapsing_header(im_str!("Simulation Params"))
                .default_open(true)
                .build()
            {
                self.simulation_params.build_ui(ui);
            }
        });

        Window::new(im_str!("Simulation")).build(ui, || {
            // insert sim render target
        });
    }
}

// Call it roughly 5 seconds
const FRAME_TIMES_TO_SAVE: usize = 5 * 60;

#[derive(Debug)]
struct FrameMetrics {
    millis: Vec<f32>,
    min: f32,
    max: f32,
    mean: f32,
    median: f32,
}

impl FrameMetrics {
    fn update(&mut self, next_ms: f32) {
        // Update the list first
        self.millis.push(next_ms);

        if self.millis.len() > FRAME_TIMES_TO_SAVE {
            self.millis.remove(0);
        }

        self.min = self.min.min(next_ms);
        self.max = self.max.max(next_ms);

        let sum: f32 = self.millis.iter().sum();
        self.mean = sum / self.millis.len() as f32;
    }

    fn build_ui(&mut self, ui: &imgui::Ui) {
        use imgui::*;

        let size = ui.item_rect_size();

        ui.text(im_str!("Frame Times"));
        ui.text(format!("Min:    {:>5.1} ms", self.min));
        ui.text(format!("Max:    {:>5.1} ms", self.max));
        ui.text(format!("Mean:   {:>5.1} ms", self.mean));
        ui.text(format!("Median: {:>5.1} ms", self.median));

        ui.plot_lines(im_str!(""), &self.millis)
            .scale_min(0.)
            .scale_max(self.max)
            .graph_size([size[0], 150.])
            .build();
    }
}

impl Default for FrameMetrics {
    fn default() -> FrameMetrics {
        FrameMetrics {
            millis: vec![],
            min: std::f32::MAX,
            max: 0.,
            mean: 0.,
            median: 0.,
        }
    }
}

#[derive(Debug)]
struct SimulationParams {
    initial_people_count: i32,
    infection_chance: f32,
}

impl SimulationParams {
    fn build_ui(&mut self, ui: &imgui::Ui) {
        use imgui::*;

        ui.input_int(
            im_str!("Initial Human Count"),
            &mut self.initial_people_count,
        )
        .build();

        // It's simpler to take a % has an integer and convert it here.
        let mut percent: u32 = (100. * self.infection_chance + 0.5) as u32;
        if Slider::new(im_str!("% Chance of Infection"), 0..=100).build(ui, &mut percent) {
            self.infection_chance = percent as f32 / 100.;
        }
    }
}

impl Default for SimulationParams {
    fn default() -> SimulationParams {
        SimulationParams {
            initial_people_count: 10,
            infection_chance: 1.00,
        }
    }
}

fn main() {
    use imgui_sdl2::ImguiSdl2;

    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();

    {
        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(3, 0);
    }

    let window = video
        .window("Covid-19 Simulation", 1000, 1000)
        .position_centered()
        .resizable()
        .opengl()
        .allow_highdpi()
        .build()
        .unwrap();

    let _gl_context = window
        .gl_create_context()
        .expect("Couldn't create GL context");
    gl::load_with(|s| video.gl_get_proc_address(s) as _);

    let mut imgui = imgui::Context::create();
    let mut imgui_sdl2 = ImguiSdl2::new(&mut imgui, &window);
    let mut event_pump = sdl_context.event_pump().unwrap();
    let renderer = Renderer::new(&mut imgui, |s| video.gl_get_proc_address(s) as _);

    // simulation & ui
    let mut state = UiState::new();
    let mut sim = covid::Simulation::sample_set();

    // Event loop
    let mut frame_count = 0;
    let mut last_frame = Instant::now();
    'running: loop {
        let start = Instant::now();

        frame_count += 1;
        use sdl2::event::Event;
        use sdl2::keyboard::Keycode;

        for event in event_pump.poll_iter() {
            imgui_sdl2.handle_event(&mut imgui, &event);
            if imgui_sdl2.ignore_event(&event) {
                continue;
            }

            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        imgui_sdl2.prepare_frame(imgui.io_mut(), &window, &event_pump.mouse_state());

        let now = Instant::now();
        let delta = now - last_frame;
        last_frame = now;

        imgui.io_mut().delta_time = delta.as_secs_f32();

        // Update state
        sim.tick(delta);

        // Build UI
        let ui = imgui.frame();

        state.build_ui(&ui);

        unsafe {
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        // Render non-UI things

        // Render UI
        imgui_sdl2.prepare_render(&ui, &window);
        renderer.render(ui);

        // Present
        window.gl_swap_window();

        let finish = Instant::now();
        if frame_count > 10 {
            state
                .frame_metrics
                .update((finish - start).as_millis() as f32);
        }
    }
}
