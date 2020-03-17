use covid;
use gl;
use std::time::{Duration, Instant};

use imgui::im_str;
// We need our dependencies to match exactly,
// so turn this off and use the private export directly.
#[allow(pub_use_of_private_extern_crate)]
use imgui_sdl2::imgui;

/// A container for state that's passed into ImGui.
/// Because we're passing pointers here into C++ code,
/// we will only use this through Box
#[derive(Debug)]
struct UiState {
    frame_metrics: FrameMetrics,
    init_people_count: usize,
    infection_chance: f64,
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
    pub fn update(&mut self, next_ms: f32) {
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

impl UiState {
    fn new() -> Box<UiState> {
        Box::new(UiState {
            frame_metrics: FrameMetrics::default(),
            init_people_count: 10,
            infection_chance: 1.0,
        })
    }
}

fn build_ui(ui: &imgui::Ui, state: &UiState) {
    use imgui::*;

    ui.show_demo_window(&mut true);

    Window::new(im_str!("Simulation Metrics"))
        .always_auto_resize(true)
        // .size([600., 800.], Condition::FirstUseEver)
        .build(ui, || {
            ui.text(im_str!("Frame Times"));

            let metrics = &state.frame_metrics;
            ui.plot_lines(im_str!(""), &metrics.millis)
                .scale_min(0.)
                .scale_max(metrics.max)
                .graph_size([500., 200.])
                .build();

            // No use showing metrics for so few stats
            if metrics.millis.len() < 20 {
                return;
            }

            ui.text(format!("Min:    {:>5.1} ms", metrics.min));
            ui.text(format!("Max:    {:>5.1} ms", metrics.max));
            ui.text(format!("Mean:   {:>5.1} ms", metrics.mean));
            ui.text(format!("Median: {:>5.1} ms", metrics.median));
        });
}

fn main() {
    use imgui_opengl_renderer::Renderer;
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

    let mut state = UiState::new();
    let mut sim = covid::Simulation::sample_set();

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
                    keycode: Some(Keycode::Escape),
                    ..
                }
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
        build_ui(&ui, &state);

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

        // Sleep to hit our target framerate.
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
