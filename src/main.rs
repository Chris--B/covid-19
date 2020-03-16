use covid;
use gl;
use std::time::Instant;

// We need our dependencies to match exactly,
// so turn this off and use the private export directly.
#[allow(pub_use_of_private_extern_crate)]
use imgui_sdl2::imgui;

fn build_ui(ui: &imgui::Ui) {
    ui.show_demo_window(&mut true);
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
        .window("rust-imgui-sdl2 demo", 1000, 1000)
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

    let mut sim = covid::Simulation::sample_set();

    let mut last_frame = Instant::now();
    'running: loop {
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
        build_ui(&ui);

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

        // Sleep to hit our target framerate.
        ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }
}
