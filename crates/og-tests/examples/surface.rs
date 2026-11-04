//! Real native surface acceptance, using the same scenes as the gfx harness.
//! PNG output is only an SDR preview, never evidence of HDR luminance.

use og_render::{RenderTarget, Renderer};
use og_tests::scenes::{self, GfxScene};
use std::{path::PathBuf, sync::Arc};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

const USAGE: &str = "surface [scene=bloom] [hdr|sdr=hdr] [paper-white=203] [peak=1000] [frames=0] [sdr-preview.png]\nFrames 0 keeps the window open until closed. OG_GFX_BACKEND selects metal/vulkan.\nObserve HDR on the actual display; PNGs clip highlights to SDR paper white.";

struct App {
    name: String,
    built: GfxScene,
    limit: u32,
    frame: u32,
    preview: Option<PathBuf>,
    renderer: Option<Renderer>,
    window: Option<Arc<Window>>,
    error: Option<String>,
}

impl App {
    fn open(&mut self, event_loop: &ActiveEventLoop) -> Result<(), String> {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title(format!("Origin surface — {}", self.name))
                        .with_inner_size(PhysicalSize::new(
                            self.built.desc.width,
                            self.built.desc.height,
                        )),
                )
                .map_err(|e| e.to_string())?,
        );
        let size = window.inner_size();
        self.built.desc.width = size.width;
        self.built.desc.height = size.height;
        let instance = self.built.desc.backend.instance();
        // Owning the Arc gives the surface a safe 'static window lifetime.
        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| e.to_string())?;
        let mut renderer =
            pollster::block_on(Renderer::with_surface(&self.built.desc, &instance, surface))
                .map_err(|e| e.to_string())?;
        renderer.set_render_scale(self.built.render_scale);
        if self.preview.is_some() {
            renderer.set_keep_frame(true).map_err(|e| e.to_string())?;
        }
        let info = renderer.info();
        println!(
            "surface scene={} monitor={:?} size={}x{} backend={:?} adapter={} requested_hdr={} hdr_supported={} hdr_resolved={} paper_white={} peak={}",
            self.name,
            window.current_monitor().and_then(|m| m.name()),
            size.width,
            size.height,
            info.backend,
            info.adapter,
            self.built.desc.hdr,
            info.hdr_supported,
            info.hdr,
            self.built.desc.paper_white_nits,
            self.built.desc.peak_nits,
        );
        if self.built.desc.hdr && !info.hdr {
            return Err(
                "HDR requested but surface resolved SDR; this run cannot validate HDR".into(),
            );
        }
        window.request_redraw();
        self.renderer = Some(renderer);
        self.window = Some(window);
        Ok(())
    }

    fn draw(&mut self) -> Result<bool, String> {
        let window = self.window.as_ref().expect("opened");
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            return Ok(false);
        }
        let renderer = self.renderer.as_mut().expect("opened");
        if let Some(update) = self.built.update {
            update(&mut self.built.scene, self.frame);
        }
        window.pre_present_notify();
        renderer
            .render(&self.built.scene, &RenderTarget::Surface)
            .map_err(|e| e.to_string())?;
        self.frame += 1;
        if self.limit != 0 && self.frame >= self.limit {
            return Ok(true);
        }
        window.request_redraw();
        Ok(false)
    }

    fn finish(&mut self) -> Result<(), String> {
        if self.frame > 0
            && let Some(path) = &self.preview
        {
            let image = self
                .renderer
                .as_ref()
                .expect("drawn")
                .read_offscreen()
                .map_err(|e| e.to_string())?;
            std::fs::write(path, og_render::png(&image)).map_err(|e| e.to_string())?;
            println!(
                "SDR preview only: {} (not HDR display evidence)",
                path.display()
            );
        }
        println!("presented {} real-surface frames", self.frame);
        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none()
            && let Err(error) = self.open(event_loop)
        {
            self.error = Some(error);
            event_loop.exit();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let result = match event {
            WindowEvent::CloseRequested => Ok(true),
            WindowEvent::RedrawRequested => self.draw(),
            WindowEvent::Resized(size) => {
                if size.width != 0 && size.height != 0 {
                    self.renderer
                        .as_mut()
                        .expect("opened")
                        .resize(size.width, size.height);
                    self.window.as_ref().expect("opened").request_redraw();
                }
                Ok(false)
            }
            _ => Ok(false),
        };
        match result {
            Ok(true) => {
                self.error = self.finish().err();
                event_loop.exit();
            }
            Err(error) => {
                self.error = Some(error);
                event_loop.exit();
            }
            Ok(false) => {}
        }
    }
}

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|s| s == "--help" || s == "-h") {
        println!("{USAGE}");
        return Ok(());
    }
    if args.len() > 6 {
        return Err(USAGE.into());
    }
    let name = args.first().map_or("bloom", String::as_str).to_string();
    let mut built = scenes::build(&name)?;
    built.desc.hdr = match args.get(1).map_or("hdr", String::as_str) {
        "hdr" => true,
        "sdr" => false,
        _ => return Err(USAGE.into()),
    };
    built.desc.paper_white_nits = args
        .get(2)
        .map_or("203", String::as_str)
        .parse::<f32>()
        .map_err(|e| e.to_string())?;
    built.desc.peak_nits = args
        .get(3)
        .map_or("1000", String::as_str)
        .parse::<f32>()
        .map_err(|e| e.to_string())?;
    let limit = args
        .get(4)
        .map_or("0", String::as_str)
        .parse::<u32>()
        .map_err(|e| e.to_string())?;
    let mut app = App {
        name,
        built,
        limit,
        frame: 0,
        preview: args.get(5).map(PathBuf::from),
        renderer: None,
        window: None,
        error: None,
    };
    EventLoop::new()
        .map_err(|e| e.to_string())?
        .run_app(&mut app)
        .map_err(|e| e.to_string())?;
    app.error.map_or(Ok(()), Err)
}
