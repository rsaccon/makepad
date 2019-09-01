use render::*;
use std::{
    panic::{self, UnwindSafe},
    process,
};

/// This is called by the true `main` function of our application.
#[no_mangle]
pub extern "C" fn main_rs() -> std::os::raw::c_int {
    // See `safe_unwind` below.
    stop_unwind(start_makepad_app)
}

/// Panicking out of rust into another language is Undefined Behavior!
///
/// Catching a panic at the FFI boundary is one of the few generally agreed
/// upon use cases for `catch_unwind`.
/// https://doc.rust-lang.org/nomicon/unwinding.html
fn stop_unwind<F: FnOnce() -> T + UnwindSafe, T>(f: F) -> T {
    match panic::catch_unwind(f) {
        Ok(t) => t,
        Err(_) => {
            eprintln!("Attempt to Unwind out of rust code");

            // We should handle the error somehow, and, without knowing what the
            // error is, aborting is an OK choice.
            process::abort()
        }
    }
}

struct App {
    window: Window,
    pass: Pass,
    color_texture: Texture,
    main_view: View<NoScrollBar>,
    quad: Quad,
}

main_app!(App);

// TODO: adapt the main_app! macro for this
/// This is the main function of our application in spirit.
fn start_makepad_app() -> std::os::raw::c_int {
    println!("Hello iOS!!!");

    main(); // TODO: call here the app makepad main

    // let mut cx = Cx {
    //     ..Default::default()
    // };

    // let mut app = App {
    //     ..Style::style(&mut cx)
    // };

    // cx.event_loop(|cx, mut event| {
    //     if let Event::Draw = event {
    //         return app.draw_app(cx);
    //     }
    //     app.handle_app(cx, &mut event);
    // });

    0
}

impl Style for App {
    fn style(cx: &mut Cx) -> Self {
        Self {
            window: Window::style(cx),
            pass: Pass::default(),
            color_texture: Texture::default(),
            main_view: View::style(cx),
            quad: Quad {
                shader: cx.add_shader(Self::def_quad_shader(), "quad"),
                ..Style::style(cx)
            },
        }
    }
}

impl App {
    pub fn def_quad_shader() -> ShaderGen {
        Quad::def_quad_shader().compose(shader_ast!({
            fn pixel() -> vec4 {
                return vec4(1.0, 0.0, 0.0, 1.0);
            }
        }))
    }
    fn handle_app(&mut self, cx: &mut Cx, event: &mut Event) {
        match event {
            Event::Construct => {}
            _ => (),
        }
    }

    fn draw_app(&mut self, cx: &mut Cx) {
        self.window.begin_window(cx);
        self.pass.begin_pass(cx);
        self.pass
            .add_color_texture(cx, &mut self.color_texture, Some(color256(30, 30, 30)));

        let _ = self.main_view.begin_view(cx, Layout::default());

        self.quad.draw_quad_abs(
            cx,
            Rect {
                x: 30.,
                y: 30.,
                w: 100.,
                h: 100.,
            },
        );

        self.main_view.end_view(cx);
        self.pass.end_pass(cx);
        self.window.end_window(cx);
    }
}
