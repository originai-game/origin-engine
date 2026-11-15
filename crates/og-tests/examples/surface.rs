//! Real native surface acceptance, using the same scenes as the gfx harness.
//! PNG output is only an SDR preview, never evidence of HDR luminance.

use og_render::{RenderError, RenderTarget, Renderer};
use og_tests::scenes::{self, GfxScene};
use std::{path::PathBuf, sync::Arc};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

const USAGE: &str = "surface [scene=bloom] [best|wide|srgb=best] [paper-white=203] [peak=0] [frames=0] [sdr-preview.png]\nScene `display`: bloom's emissive spheres (highlights past SDR white) under CSS swatches, each the colour mapped to sRGB (left) beside the same colour in the output's gamut (right).\nFrames 0 keeps the window open until closed. OG_GFX_BACKEND selects metal/vulkan.\nObserve HDR on the actual display; PNGs clip highlights to SDR paper white.";

struct App {
    name: String,
    built: GfxScene,
    limit: u32,
    frame: u32,
    occluded: bool,
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
                        ))
                        // The primary display (a Mac's built-in XDR panel).
                        .with_position(event_loop.primary_monitor().map_or_else(
                            || winit::dpi::PhysicalPosition::new(0, 0),
                            |m| {
                                winit::dpi::PhysicalPosition::new(
                                    m.position().x + 80,
                                    m.position().y + 120,
                                )
                            },
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
            "surface scene={} monitor={:?} size={}x{} backend={:?} adapter={} output={:?} hdr_supported={} hdr_resolved={} wide_gamut={} paper_white={} peak={}",
            self.name,
            window.current_monitor().and_then(|m| m.name()),
            size.width,
            size.height,
            info.backend,
            info.adapter,
            self.built.desc.output,
            info.hdr_supported,
            info.hdr,
            info.wide_gamut,
            self.built.desc.paper_white_nits,
            self.built.desc.peak_nits,
        );
        if self.built.desc.output != og_render::DisplayOutput::Srgb && !info.wide_gamut {
            return Err(
                "a wide output was asked for but the surface resolved sRGB; this run cannot validate it".into(),
            );
        }
        if self.name == "display" {
            swatches(&mut self.built, info.wide_gamut)?;
        }
        if let Some(update) = self.built.update {
            update(&mut self.built.scene, self.frame);
        }
        window.focus_window();
        window.request_redraw();
        self.renderer = Some(renderer);
        self.window = Some(window);
        Ok(())
    }

    fn draw(&mut self) -> Result<bool, String> {
        let window = self.window.as_ref().expect("opened");
        let size = window.inner_size();
        if self.occluded || size.width == 0 || size.height == 0 {
            return Ok(false);
        }
        let renderer = self.renderer.as_mut().expect("opened");
        window.pre_present_notify();
        match renderer.render(&self.built.scene, &RenderTarget::Surface) {
            Ok(_) => {}
            Err(RenderError::SurfaceOccluded) => {
                self.occluded = true;
                println!("surface occluded; waiting for window visibility (no frame counted)");
                return Ok(false);
            }
            Err(error) => return Err(error.to_string()),
        }
        self.frame += 1;
        if self.limit != 0 && self.frame >= self.limit {
            return Ok(true);
        }
        if let Some(update) = self.built.update {
            update(&mut self.built.scene, self.frame);
        }
        window.request_redraw();
        Ok(false)
    }

    fn finish(&mut self) -> Result<(), String> {
        if self.frame == 0 {
            return Err("closed without presenting a surface frame; acceptance not run".into());
        }
        if let Some(path) = &self.preview {
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
            WindowEvent::Occluded(occluded) => {
                self.occluded = occluded;
                if !occluded {
                    self.window.as_ref().expect("opened").request_redraw();
                }
                Ok(false)
            }
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
    let mut built = scenes::build(if name == "display" { "bloom" } else { &name })?;
    built.desc.output = match args.get(1).map_or("best", String::as_str) {
        "best" => og_render::DisplayOutput::Best,
        "wide" => og_render::DisplayOutput::Wide,
        "srgb" => og_render::DisplayOutput::Srgb,
        _ => return Err(USAGE.into()),
    };
    built.desc.paper_white_nits = args
        .get(2)
        .map_or("203", String::as_str)
        .parse::<f32>()
        .map_err(|e| e.to_string())?;
    built.desc.peak_nits = args
        .get(3)
        .map_or("0", String::as_str)
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
        occluded: false,
        preview: args.get(5).map(PathBuf::from),
        renderer: None,
        window: None,
        error: None,
    };
    let mut event_loop = EventLoop::builder();
    #[cfg(target_os = "macos")]
    {
        use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};
        event_loop
            .with_activation_policy(ActivationPolicy::Regular)
            .with_activate_ignoring_other_apps(true);
    }
    event_loop
        .build()
        .map_err(|e| e.to_string())?
        .run_app(&mut app)
        .map_err(|e| e.to_string())?;
    app.error.map_or(Ok(()), Err)
}

/// CSS colours outside sRGB, each drawn twice: gamut mapped to sRGB (left)
/// and as og-ui draws it on this output (right: Display P3 where the
/// surface presents it, else the same sRGB colour).
const SWATCHES: [&str; 4] = [
    "color(display-p3 1 0 0)",
    "color(display-p3 0 1 0)",
    "oklch(0.75 0.3 330)",
    "oklch(0.85 0.25 190)",
];

/// The `display` scene's swatch pairs over bloom's spheres, whose
/// highlights run past SDR white.
fn swatches(g: &mut GfxScene, wide: bool) -> Result<(), String> {
    use og_css::color::{Context, Gamut, parse_str};
    let (w, h) = (g.desc.width as f32, g.desc.height as f32);
    let mut o = og_render::Overlay::new(w, h, 0.0);
    let tokens = NoTokens;
    let srgb = Context::of(&tokens);
    let output = Context {
        gamut: if wide { Gamut::P3 } else { Gamut::Srgb },
        ..srgb
    };
    let size = w / 12.0;
    for (k, css) in SWATCHES.iter().enumerate() {
        let c = parse_str(css)?;
        let x = w * 0.08 + k as f32 * size * 2.6;
        let y = h - size * 1.4;
        o.ui.shapes.push(og_render::UiShape::rect(
            [x - 6.0, y - 6.0, size * 2.0 + 12.0, size + 12.0],
            8.0,
            [0.0, 0.0, 0.0, 1.0],
        ));
        o.ui.shapes.push(og_render::UiShape::rect(
            [x, y, size, size],
            4.0,
            c.linear(&srgb),
        ));
        o.ui.shapes.push(og_render::UiShape::rect(
            [x + size, y, size, size],
            4.0,
            c.linear(&output),
        ));
    }
    g.scene.ui.overlay = Some(o);
    Ok(())
}

struct NoTokens;

impl og_css::color::ColorTokens for NoTokens {
    fn color(&self, _: &str) -> Option<og_css::Srgba> {
        None
    }
    fn ink(&self) -> og_css::Srgba {
        og_css::Srgba::rgb(255, 255, 255)
    }
    fn dark(&self) -> bool {
        true
    }
}
