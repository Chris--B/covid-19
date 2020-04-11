use crate::imgui;

pub struct Renderer {}

impl Renderer {
    pub fn new<F>(context: &mut imgui::Context, _load_fn: F) -> Self
    where
        F: FnMut(&'static str) -> *const ::std::os::raw::c_void,
    {
        let mut atlas = context.fonts();
        let texture = atlas.build_rgba32_texture();

        Renderer {}
    }

    pub fn render<'ui>(&self, ui: imgui::Ui<'ui>) {
        use imgui::{DrawCmd, DrawCmdParams, DrawIdx, DrawVert};

        let [width, height] = ui.io().display_size;
        let [scale_w, scale_h] = ui.io().display_framebuffer_scale;
        let (fb_w, fb_h) = (width * scale_w, height * scale_h);

        dbg!(width, height);
        dbg!(scale_w, scale_h);
        dbg!(fb_w, fb_h);

        let draw_data = ui.render();

        for draw_list in draw_data.draw_lists() {
            let vtx_buffer = draw_list.vtx_buffer();
            let idx_buffer = draw_list.idx_buffer();

            for cmd in draw_list.commands() {
                match cmd {
                    DrawCmd::Elements { count, cmd_params } => {
                        dbg!(count, cmd_params);
                    }
                    DrawCmd::ResetRenderState => {
                        dbg!("RESET RENDER STATE");
                    }
                    DrawCmd::RawCallback { callback, raw_cmd } => {
                        dbg!(callback);
                    }
                }
            }
        }

        // todo!()
    }
}
