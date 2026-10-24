//! og-tests' gfx scenes, and og-ui's reference scenes (`og-ui/<name>`,
//! crates/og-ui/tests/support/scenes.rs, the file og-ui's own gfx tests
//! build them with), in a browser (docs/ci.md §Browser reference tiers).
//!
//! One scene at a time: [`og_gfx_open`] builds it and a renderer on WebGPU
//! or WebGL2 (an OffscreenCanvas, which WebGL2 needs for its context),
//! [`og_gfx_render`] draws frames to the renderer's offscreen target exactly
//! as `og_tests::gfx::render_built` does natively (the scene's update before
//! each), and [`og_gfx_capture`] reads the last one back as RGBA8.
//! og-ui's cooked-font text frames (`og-ui/text-bidi`, `text-kerning`,
//! `text-edit`, `text-mixed-runs`: crates/og-ui/tests/support/text_scenes.rs)
//! draw with fonts the module cannot cook: the runner cooks
//! crates/og-ui/tests/text-fonts with `og cook` and hands over the cook's
//! manifest ([`og_gfx_text_manifest`]) and each file under `cooked/`
//! ([`og_gfx_text_file`]) before it opens them.
//! [`og_gfx_cpu_culling`] makes the scenes opened after it cull on the CPU
//! (`Culling::Cpu`, as `OG_GFX_CULLING=cpu` does natively). The
//! comparison and the timing are the runner's
//! (`packages/engine/tests/web/reference-gfx.ts`).

// og-ui's reference scenes, as og-ui's tests build them (`text_scenes`
// reaches `super::scenes`, as it does in og-ui's `support`).
#[cfg(target_arch = "wasm32")]
#[allow(dead_code)]
#[path = "../../og-ui/tests/support/scenes.rs"]
mod scenes;
#[cfg(target_arch = "wasm32")]
#[allow(dead_code)]
#[path = "../../og-ui/tests/support/text_scenes.rs"]
mod text_scenes;

#[cfg(target_arch = "wasm32")]
mod web {
    use og_render::{Backend, RenderTarget, Renderer};
    use og_tests::compare::Tolerance;
    use og_tests::scenes::{self, Applies, GfxScene};
    use std::cell::{Cell, RefCell};
    use std::collections::BTreeMap;
    use wasm_bindgen::prelude::*;

    struct Open {
        built: GfxScene,
        renderer: Renderer,
        frame: u32,
    }

    thread_local! {
        static OPEN: RefCell<Option<Open>> = const { RefCell::new(None) };
        static CPU_CULLING: Cell<bool> = const { Cell::new(false) };
        /// The text fonts' cook: its manifest and its files by path under
        /// `cooked/`.
        static TEXT_MANIFEST: RefCell<Option<String>> = const { RefCell::new(None) };
        static TEXT_FILES: RefCell<BTreeMap<String, Vec<u8>>> = const { RefCell::new(BTreeMap::new()) };
    }

    fn err(e: impl std::fmt::Display) -> JsValue {
        JsValue::from_str(&e.to_string())
    }

    /// Every scene og-tests knows, in its order.
    #[wasm_bindgen]
    pub fn og_gfx_scenes() -> Vec<String> {
        scenes::SCENES.iter().map(|s| s.to_string()).collect()
    }

    /// og-ui's reference scenes, as `og-ui/<name>` (`<name>.png` under
    /// crates/og-ui/references/<tier>/).
    #[wasm_bindgen]
    pub fn og_gfx_ui_scenes() -> Vec<String> {
        super::scenes::REFERENCE_SCENES
            .iter()
            .chain(super::text_scenes::TEXT_SCENES.iter())
            .map(|s| format!("og-ui/{s}"))
            .collect()
    }

    /// The text fonts' cook manifest (`cooked/.cook.ogt` of
    /// crates/og-ui/tests/text-fonts, cooked by the runner).
    #[wasm_bindgen]
    pub fn og_gfx_text_manifest(text: String) {
        TEXT_MANIFEST.with(|m| *m.borrow_mut() = Some(text));
    }

    /// One file of the text fonts' cook, by its path under `cooked/`.
    #[wasm_bindgen]
    pub fn og_gfx_text_file(path: String, bytes: Vec<u8>) {
        TEXT_FILES.with(|f| f.borrow_mut().insert(path, bytes));
    }

    /// The cooked text fonts, from the manifest and files the runner gave.
    fn text_fonts() -> Result<Vec<og_cook::runtime::CookedFont>, String> {
        let manifest = TEXT_MANIFEST.with(|m| m.borrow().clone()).ok_or(
            "no text fonts: the runner cooks crates/og-ui/tests/text-fonts and calls og_gfx_text_manifest and og_gfx_text_file first",
        )?;
        let bundle =
            og_cook::runtime::Bundle::from_manifest(&manifest).map_err(|e| e.to_string())?;
        TEXT_FILES.with(|f| {
            let files = f.borrow();
            super::text_scenes::fonts_from(&bundle, |path| files.get(path).cloned())
        })
    }

    /// og-ui's reference scene `name` as a gfx scene: one frame, og-tests'
    /// default tolerance (og-ui's rule), every tier; the test font compiled
    /// in, or for a cooked-font text frame the fonts the runner gave.
    fn ui_scene(name: &str) -> Result<GfxScene, String> {
        let (scene, desc) = if super::text_scenes::TEXT_SCENES.contains(&name) {
            super::text_scenes::text_scene(name, &text_fonts()?)?
        } else {
            test_font_scene(name)?
        };
        Ok(GfxScene {
            desc,
            scene,
            tolerance: Tolerance::default(),
            frames: 1,
            update: None,
            applies: Applies::Every,
        })
    }

    /// One of og-ui's scenes drawn with the test font (compiled in).
    fn test_font_scene(
        name: &str,
    ) -> Result<(og_render::RenderScene, og_render::RenderDesc), String> {
        let text = |rel: &str| scenes::asset(rel).map(|b| String::from_utf8_lossy(&b).into_owned());
        let png = scenes::asset("fonts/noto-sans-sc-msdf.png")?;
        let json = text("fonts/noto-sans-sc-msdf.json")?;
        let csv = text("fonts/noto-sans-sc-msdf.csv")?;
        let font = super::scenes::FontFiles {
            png: &png,
            json: &json,
            csv: &csv,
        };
        super::scenes::reference_scene(name, &font)
    }

    /// Whether the scenes opened from now on cull on the CPU on every tier
    /// (`Culling::Cpu`) rather than on the GPU where the adapter can.
    #[wasm_bindgen]
    pub fn og_gfx_cpu_culling(cpu: bool) {
        CPU_CULLING.with(|c| c.set(cpu));
    }

    /// Builds `scene` (an og-tests scene, or `og-ui/<name>`) and its renderer (WebGL2 when `webgl`, on `canvas`; else
    /// WebGPU, headless). Resolves to a JSON description: tier, adapter,
    /// antialiasing, whether the scene has a reference on this tier, the
    /// frames it renders before its capture and its tolerance.
    #[wasm_bindgen]
    pub async fn og_gfx_open(
        scene: String,
        webgl: bool,
        canvas: web_sys::OffscreenCanvas,
    ) -> Result<String, JsValue> {
        // A panic aborts with only "unreachable" in the console; its message
        // (a wgpu validation error, say) is what the runner reports.
        static HOOK: std::sync::Once = std::sync::Once::new();
        HOOK.call_once(|| {
            std::panic::set_hook(Box::new(|info| {
                web_sys::console::error_1(&format!("og-tests-web panicked: {info}").into());
            }))
        });
        OPEN.with(|o| o.borrow_mut().take());
        let mut built = match scene.strip_prefix("og-ui/") {
            Some(name) => ui_scene(name),
            None => scenes::build(&scene),
        }
        .map_err(err)?;
        built.desc.backend = if webgl {
            Backend::WebGl2
        } else {
            Backend::WebGpu
        };
        if CPU_CULLING.with(Cell::get) {
            built.desc.culling = og_render::Culling::Cpu;
        }
        canvas.set_width(built.desc.width);
        canvas.set_height(built.desc.height);
        let renderer = if webgl {
            let instance = built.desc.backend.instance();
            let surface = instance
                .create_surface(wgpu::SurfaceTarget::OffscreenCanvas(canvas))
                .map_err(|e| err(format!("surface: {e}")))?;
            Renderer::with_surface(&built.desc, &instance, surface).await
        } else {
            Renderer::new(&built.desc).await
        }
        .map_err(err)?;
        let info = renderer.info();
        let t = &built.tolerance;
        let json = format!(
            "{{\"tier\":{:?},\"adapter\":{:?},\"antialias\":\"{:?}\",\"applies\":{},\"frames\":{},\"width\":{},\"height\":{},\"tolerance\":{{\"channel\":{},\"outliers\":{},\"rmse\":{}}}}}",
            info.backend_tier(),
            info.adapter,
            info.antialias,
            built.applies.includes(info.tier),
            built.frames.max(1),
            built.desc.width,
            built.desc.height,
            t.channel,
            t.max_outlier_fraction,
            t.rmse,
        );
        OPEN.with(|o| {
            *o.borrow_mut() = Some(Open {
                built,
                renderer,
                frame: 0,
            })
        });
        Ok(json)
    }

    /// Draws `n` frames of the open scene, each after its update (frame
    /// indices continue across calls). Returns the last frame's counts as
    /// JSON; with GPU culling they are `counts_lag` frames old (the browser
    /// cannot wait for the frame's own; every frame asks for its counts).
    #[wasm_bindgen]
    pub fn og_gfx_render(n: u32) -> Result<String, JsValue> {
        OPEN.with(|o| {
            let mut o = o.borrow_mut();
            let open = o.as_mut().ok_or_else(|| err("no scene open"))?;
            let renderer = &mut open.renderer;
            let mut stats = og_render::FrameStats::default();
            for _ in 0..n {
                if let Some(update) = open.built.update {
                    update(&mut open.built.scene, open.frame);
                }
                open.frame += 1;
                renderer.request_culling_counts();
                stats = renderer
                    .render(&open.built.scene, &RenderTarget::Offscreen)
                    .map_err(err)?;
            }
            Ok(format!(
                "{{\"gpu_culling\":{},\"draws\":{},\"instances\":{},\"culled\":{},\"occluded\":{},\"triangles\":{},\"shadow_casters\":{},\"counts_lag\":{}}}",
                stats.gpu_culling,
                stats.draw_calls,
                stats.instances,
                stats.culled,
                stats.occluded,
                stats.triangles,
                stats.shadow_casters,
                stats.counts_lag,
            ))
        })
    }

    /// The open scene's last frame as RGBA8, rows top to bottom; resolves
    /// when the GPU has finished every frame submitted before it.
    #[wasm_bindgen]
    pub async fn og_gfx_capture() -> Result<Vec<u8>, JsValue> {
        let mut request = OPEN.with(|o| {
            let o = o.borrow();
            let open = o.as_ref().ok_or_else(|| err("no scene open"))?;
            open.renderer.begin_capture().map_err(err)
        })?;
        // Poll until the copy lands: WebGL2 maps only when the device is
        // polled (Renderer::capture's future alone never resolves there).
        for _ in 0..10_000 {
            let done = OPEN.with(|o| {
                let o = o.borrow();
                let open = o.as_ref().ok_or_else(|| err("the scene was closed"))?;
                request.try_finish(&open.renderer).map_err(err)
            })?;
            if let Some(image) = done {
                return Ok(image.pixels);
            }
            tick().await?;
        }
        Err(err("capture: the copy did not land in 10000 polls"))
    }

    /// One macrotask (setTimeout 0), so the browser can finish GPU work.
    async fn tick() -> Result<(), JsValue> {
        let set_timeout: js_sys::Function =
            js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("setTimeout"))?.into();
        let promise = js_sys::Promise::new(&mut |resolve, _| {
            let _ = set_timeout.call2(&JsValue::NULL, &resolve, &JsValue::from(0));
        });
        wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map(|_| ())
    }

    /// Drops the open scene and its renderer.
    #[wasm_bindgen]
    pub fn og_gfx_close() {
        OPEN.with(|o| o.borrow_mut().take());
    }
}
