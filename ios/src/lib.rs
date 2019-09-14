use render::*;

struct App {
    window: Window,
    pass: Pass,
    color_texture: Texture,
    main_view: View<NoScrollBar>,
    quad: Quad,
}

main_app!(App);

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
