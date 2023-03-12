use crate::primitives::color::Color;

use glow::HasContext;
use glutin::event_loop::EventLoop;
use imgui_glow_renderer;
use imgui_winit_support::WinitPlatform;

pub struct Window {
    windowed_context: glutin::WindowedContext<glutin::PossiblyCurrent>,
    winit_platform: WinitPlatform,
    gl: glow::Context,
    imgui_renderer: imgui_glow_renderer::Renderer,
    imgui_context: imgui::Context,
    imgui_texture_map: imgui_glow_renderer::SimpleTextureMap,
}

impl Window {
    pub fn new(title: &str, width: u32, height: u32) -> (Window, EventLoop<()>) {
        let event_loop = glutin::event_loop::EventLoop::new();
        let window = glutin::window::WindowBuilder::new()
            .with_title(title)
            .with_inner_size(glutin::dpi::LogicalSize::new(width, height));

        let window = glutin::ContextBuilder::new()
            .with_vsync(true)
            .build_windowed(window, &event_loop)
            .unwrap();

        let windowed_context = unsafe { window.make_current() }.unwrap();

        let (mut imgui_context, winit_platform) = Self::create_imgui_context(&windowed_context);

        let gl = unsafe {
            glow::Context::from_loader_function(|s| windowed_context.get_proc_address(s).cast())
        };

        let mut imgui_texture_map = imgui_glow_renderer::SimpleTextureMap::default();
        let imgui_renderer = imgui_glow_renderer::Renderer::initialize(
            &gl,
            &mut imgui_context,
            &mut imgui_texture_map,
            true,
        )
        .unwrap();

        (
            Window {
                windowed_context,
                winit_platform,
                gl,
                imgui_renderer,
                imgui_context,
                imgui_texture_map,
            },
            event_loop,
        )
    }

    fn create_imgui_context(
        windowed_context: &glutin::WindowedContext<glutin::PossiblyCurrent>,
    ) -> (imgui::Context, WinitPlatform) {
        let mut imgui_context = imgui::Context::create();
        imgui_context.set_ini_filename(None);

        let mut winit_platform = WinitPlatform::init(&mut imgui_context);
        winit_platform.attach_window(
            imgui_context.io_mut(),
            windowed_context.window(),
            imgui_winit_support::HiDpiMode::Rounded,
        );

        imgui_context
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

        imgui_context.io_mut().font_global_scale = (1.0 / winit_platform.hidpi_factor()) as f32;

        (imgui_context, winit_platform)
    }

    pub fn gl(&self) -> &glow::Context {
        &self.gl
    }

    pub fn set_clear_color(&self, color: Color) {
        unsafe { self.gl.clear_color(color.r, color.g, color.b, color.a) };
    }

    pub fn update_delta_time(&mut self, duration: std::time::Duration) {
        self.imgui_context.io_mut().update_delta_time(duration);
    }

    pub fn request_redraw(&mut self) {
        self.winit_platform
            .prepare_frame(self.imgui_context.io_mut(), self.windowed_context.window())
            .unwrap();
        self.windowed_context.window().request_redraw();
    }

    pub fn render<F: FnOnce(&mut imgui::Ui)>(&mut self, build_ui: F) {
        let ui = self.imgui_context.frame();
        build_ui(ui);

        self.winit_platform
            .prepare_render(ui, self.windowed_context.window());
        let draw_data = self.imgui_context.render();

        self.imgui_renderer
            .render(&self.gl, &self.imgui_texture_map, draw_data)
            .unwrap();
        self.windowed_context.swap_buffers().unwrap();
    }

    pub fn imgui_using_mouse(&self) -> bool {
        self.imgui_context.io().want_capture_mouse
    }

    pub fn handle_event(&mut self, event: glutin::event::Event<()>) {
        use glutin::event::{Event, WindowEvent};

        if let Event::WindowEvent {
            event: WindowEvent::Resized(size),
            ..
        } = event
        {
            self.windowed_context.resize(size);
            unsafe {
                self.gl
                    .viewport(0, 0, size.width as i32, size.height as i32)
            };
        }

        self.winit_platform.handle_event(
            self.imgui_context.io_mut(),
            self.windowed_context.window(),
            &event,
        );
    }
}
