extern crate gl;
extern crate sdl2;
#[macro_use]
extern crate imgui;
extern crate imgui_gl_renderer;

use sdl2::event::Event;

use imgui::{ImGui, Ui, ImGuiCond};
use imgui_gl_renderer::Renderer;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();

    let window = video.window("imgui GL", 800, 600)
        .opengl()
        .position_centered()
        .build()
        .unwrap();

    let gl_context = window.gl_create_context().unwrap();
    window.gl_make_current(&gl_context).unwrap();

    let mut pump = sdl_context.event_pump().unwrap();

    gl::load_with(|s| video.gl_get_proc_address(s) as _ );

    let mut imgui = ImGui::init();

    //imgui.set_font_global_scale(1.0);

    let renderer = unsafe { Renderer::init(&mut imgui) };

    'mainLoop: loop {
        for event in pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'mainLoop,
                _ => {},
            }
        }

        unsafe {
            gl::ClearColor(0.5, 0.5, 0.5, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            let ui = imgui.frame((800, 600), (1, 1), 1.0/60.0);

            if !imgui_demo(&ui) {
                break 'mainLoop;
            }

            renderer.render(ui);
        }

        window.gl_swap_window();
    }
}

fn imgui_demo(ui: &Ui) -> bool {
    let mut open = true;
    ui.show_test_window(&mut open);
    open
}
