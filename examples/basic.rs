use pixels::{wgpu, PixelsContext};
use std::process::abort;
use std::time::Instant;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

pub(crate) struct Gui {
    imgui: imgui::Context,
    platform: imgui_winit_support::WinitPlatform,
    renderer: imgui_wgpu::Renderer,
    last_frame: Instant,
    last_cursor: Option<imgui::MouseCursor>,
    open_file_dialog: bool,
}

impl Gui {
    /// Create Dear ImGui.
    pub(crate) fn new(window: &winit::window::Window, pixels: &pixels::Pixels) -> Self {
        // Create Dear ImGui context
        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);

        // Initialize winit platform support
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        platform.attach_window(
            imgui.io_mut(),
            window,
            imgui_winit_support::HiDpiMode::Default,
        );

        // Configure Dear ImGui fonts
        let hidpi_factor = window.scale_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        imgui
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData {
                config: Some(imgui::FontConfig {
                    oversample_h: 1,
                    pixel_snap_h: true,
                    size_pixels: font_size,
                    ..Default::default()
                }),
            }]);

        // Create Dear ImGui WGPU renderer
        let device = pixels.device();
        let queue = pixels.queue();
        let config = imgui_wgpu::RendererConfig {
            texture_format: pixels.render_texture_format(),
            ..Default::default()
        };
        let renderer = imgui_wgpu::Renderer::new(&mut imgui, device, queue, config);

        // Return GUI context
        Self {
            open_file_dialog: true,
            imgui,
            platform,
            renderer,
            last_frame: Instant::now(),
            last_cursor: None,
        }
    }

    /// Prepare Dear ImGui.
    pub(crate) fn prepare(
        &mut self,
        window: &winit::window::Window,
    ) -> Result<(), winit::error::ExternalError> {
        // Prepare Dear ImGui
        let now = Instant::now();
        self.imgui.io_mut().update_delta_time(now - self.last_frame);
        self.last_frame = now;
        self.platform.prepare_frame(self.imgui.io_mut(), window)
    }

    /// Render Dear ImGui.
    pub(crate) fn render(
        &mut self,
        window: &winit::window::Window,
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
        context: &PixelsContext,
    ) -> imgui_wgpu::RendererResult<()> {
        // Start a new Dear ImGui frame and update the cursor
        let ui = self.imgui.new_frame();

        let mouse_cursor = ui.mouse_cursor();
        if self.last_cursor != mouse_cursor {
            self.last_cursor = mouse_cursor;
            self.platform.prepare_render(ui, window);
        }

        if self.open_file_dialog == true {
            if let Some(file) = imfile::FileDialog::new()
                .accept_text("Open file")
                .for_save()
                .cancel_text("Close")
                .title("Open File")
                .spawn(&ui)
            {
                println!("Filename: {}", file.display());
                self.open_file_dialog = false;
            }
        }

        // Render Dear ImGui with WGPU
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("imgui"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: render_target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        self.renderer.render(
            self.imgui.render(),
            &context.queue,
            &context.device,
            &mut rpass,
        )
    }

    /// Handle any outstanding events.
    pub(crate) fn handle_event(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::Event<()>,
    ) {
        self.platform
            .handle_event(self.imgui.io_mut(), window, event);
    }
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(1280 as f64, 720 as f64);
        WindowBuilder::new()
            .with_title("ImFile example")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut scale_factor = window.scale_factor();

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(1280, 720, surface_texture)?
    };
    
    let mut gui = Gui::new(&window, &pixels);

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            pixels.frame_mut().into_iter().for_each(|pix| *pix = 0x0);
            gui.prepare(&window).expect("gui.prepare() failed");
            let render_result = pixels.render_with(|encoder, render_target, context| {
                context.scaling_renderer.render(encoder, render_target);
                gui.render(&window, encoder, render_target, context)?;

                Ok(())
            });
            if render_result.is_err() {
                log::error!("Can't render!");
                abort();
            }
        }

        gui.handle_event(&window, &event);
        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if let Some(factor) = input.scale_factor() {
                scale_factor = factor;
            }

            if let Some(size) = input.window_resized() {
                if size.width > 0 && size.height > 0 {
                    pixels.resize_surface(size.width, size.height).expect("resize error");

                    let LogicalSize { width, height } = size.to_logical(scale_factor);
                    if let Err(err) = pixels.resize_buffer(width, height) {
                        panic!("Error: {err}");
                    }
                }
            }
            window.request_redraw();
        }
    });
}