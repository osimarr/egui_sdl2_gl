use std::time::{Duration, Instant};

use egui::Checkbox;
use egui_backend::sdl2::video::GLProfile;
use egui_backend::{egui, gl, sdl2};
use egui_backend::{sdl2::event::Event, DpiScaling};
// Alias the backend to something less mouthful
use egui_sdl2_gl as egui_backend;
use sdl2::video::SwapInterval;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);

    // Let OpenGL know we are dealing with SRGB colors so that it
    // can do the blending correctly. Not setting the framebuffer
    // leads to darkened, oversaturated colors.
    gl_attr.set_framebuffer_srgb_compatible(true);
    gl_attr.set_double_buffer(true);
    gl_attr.set_multisample_samples(4);
    // OpenGL 3.2 is the minimum that we will support.
    gl_attr.set_context_version(3, 2);

    let window = video_subsystem
        .window(
            "Demo: Egui backend for SDL2 + GL",
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
        )
        .opengl()
        .resizable()
        .build()
        .unwrap();

    // Create a window context
    let _ctx = window.gl_create_context().unwrap();
    debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
    debug_assert_eq!(gl_attr.context_version(), (3, 2));

    // Init egui stuff
    let (mut painter, mut egui_state) = egui_backend::with_sdl2(&window, DpiScaling::Custom(2.0));
    let mut egui_ctx = egui::CtxRef::default();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut test_str: String =
        "A text box to write in. Cut, copy, paste commands are available.".to_owned();

    let mut enable_vsync = false;
    let mut quit = false;
    let mut slider = 0.0;

    let start_time = Instant::now();

    'running: loop {
        if enable_vsync {
            window
                .subsystem()
                .gl_set_swap_interval(SwapInterval::VSync)
                .unwrap()
        } else {
            window
                .subsystem()
                .gl_set_swap_interval(SwapInterval::Immediate)
                .unwrap()
        }

        egui_state.input.time = Some(start_time.elapsed().as_secs_f64());
        egui_ctx.begin_frame(egui_state.input.take());

        egui::CentralPanel::default().show(&egui_ctx, |ui| {
            ui.label(" ");
            ui.text_edit_multiline(&mut test_str);
            ui.label(" ");
            ui.add(egui::Slider::new(&mut slider, 0.0..=50.0).text("Slider"));
            ui.label(" ");
            ui.add(Checkbox::new(&mut enable_vsync, "Reduce CPU Usage?"));
            ui.separator();
            if ui.button("Quit?").clicked() {
                quit = true;
            }
        });

        let (egui_output, paint_cmds) = egui_ctx.end_frame();
        // Fuse output
        egui_state.fuse_output(&egui_output);

        // For default dpi scaling only, Update window when the size of resized window is very small (to avoid egui::CentralPanel distortions).
        // if egui_ctx.used_size() != painter.screen_rect.size() {
        //     println!("resized.");
        //     let _size = egui_ctx.used_size();
        //     let (w, h) = (_size.x as u32, _size.y as u32);
        //     window.set_size(w, h).unwrap();
        // }

        let paint_jobs = egui_ctx.tessellate(paint_cmds);

        // An example of how OpenGL can be used to draw custom stuff with egui
        // overlaying it:
        // First clear the background to something nice.
        unsafe {
            // Clear the screen to black
            gl::ClearColor(0.3, 0.6, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        if !egui_output.needs_repaint {
            std::thread::sleep(Duration::from_millis(10))
        }

        painter.paint_jobs(None, paint_jobs, &egui_ctx.texture());

        window.gl_swap_window();

        if !egui_output.needs_repaint {
            if let Some(event) = event_pump.wait_event_timeout(5) {
                match event {
                    Event::Quit { .. } => break 'running,
                    _ => {
                        // Fuse event
                        egui_state.fuse_input(&window, event, &mut painter);
                    }
                }
            }
        } else {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'running,
                    _ => {
                        // Fuse event
                        egui_state.fuse_input(&window, event, &mut painter);
                    }
                }
            }
        }

        if quit {
            break;
        }
    }
}
