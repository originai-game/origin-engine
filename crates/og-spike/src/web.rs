//! Web host: the same renderer on an OffscreenCanvas inside a Worker.

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WebSpike {
    renderer: crate::Renderer,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
}

#[wasm_bindgen]
pub fn init_log() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Info);
}

#[wasm_bindgen]
impl WebSpike {
    /// `glbs` is a list of Uint8Array model files; `count` instances of each are placed.
    pub async fn create(canvas: web_sys::OffscreenCanvas, glbs: js_sys::Array, count: u32, webgl: bool, sphere: bool, moving: bool) -> Result<WebSpike, JsValue> {
        let width = canvas.width();
        let height = canvas.height();
        let mut models = Vec::new();
        for g in glbs.iter() {
            let bytes = js_sys::Uint8Array::new(&g).to_vec();
            models.push(crate::scene::load_glb(&bytes).map_err(|e| JsValue::from_str(&e))?);
        }
        let models = if sphere { vec![crate::scene::sphere(16, 12)] } else { models };
        let scene = crate::reference_scene(models, count, if sphere { 1.2 } else { 3.0 });
        let backends = if webgl { wgpu::Backends::GL } else { wgpu::Backends::BROWSER_WEBGPU };
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor { backends, ..wgpu::InstanceDescriptor::new_without_display_handle() });
        let surface = instance.create_surface(wgpu::SurfaceTarget::OffscreenCanvas(canvas)).map_err(|e| JsValue::from_str(&e.to_string()))?;
        // Probe the surface's preferred format through a throwaway adapter request inside Renderer::new.
        // WebGPU canvases prefer BGRA; wgpu's WebGL2 surface offers RGBA only.
        let format = if webgl { wgpu::TextureFormat::Rgba8Unorm } else { wgpu::TextureFormat::Bgra8Unorm };
        let mut renderer = crate::Renderer::new(&instance, Some(&surface), width, height, Some(format), &scene).await.map_err(|e| JsValue::from_str(&e))?;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        renderer.moving = moving;
        surface.configure(&renderer.device, &config);
        Ok(WebSpike { renderer, surface, config })
    }

    pub fn frame(&mut self, time: f32) -> Result<(), JsValue> {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            other => return Err(JsValue::from_str(&format!("surface: {other:?}"))),
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor { format: Some(self.config.format), ..Default::default() });
        self.renderer.render(&view, time);
        self.renderer.queue.present(frame);
        Ok(())
    }

    pub fn triangles(&self) -> f64 {
        self.renderer.triangles as f64
    }

    pub fn draws(&self) -> u32 {
        self.renderer.draw_calls() as u32
    }
}
