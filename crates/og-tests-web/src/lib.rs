//! og-tests' gfx scenes, and og-ui's reference scenes (`og-ui/<name>`,
//! crates/og-ui/tests/support/scenes.rs, the file og-ui's own gfx tests
//! build them with), in a browser (docs/ci.md §Browser reference tiers).
//!
//! One scene at a time: [`og_gfx_open`] builds it and a renderer on WebGPU,
//! [`og_gfx_render`] draws frames to the renderer's offscreen target exactly
//! as `og_tests::gfx::render_built` does natively (the scene's update before
//! each), and [`og_gfx_capture`] reads the last one back as RGBA8.
//! og-ui's cooked-font text frames (`og-ui/text-bidi`, `text-kerning`,
//! `text-edit`, `text-mixed-runs`: crates/og-tests-cooked/tests/support/text_scenes.rs)
//! draw with fonts the module cannot cook: the runner cooks
//! crates/og-ui/tests/text-fonts with `og cook` and hands over the cook's
//! manifest ([`og_gfx_text_manifest`]) and each file under `cooked/`
//! ([`og_gfx_text_file`]) before it opens them.
//! [`og_gfx_cpu_culling`] makes the scenes opened after it cull on the CPU
//! (`Culling::Cpu`, as `OG_GFX_CULLING=cpu` does natively). The
//! comparison and the timing are the runner's
//! (`packages/engine/tests/web/reference-gfx.ts`).

// og-ui's reference scenes, as og-ui's tests build them (`text_scenes`
// reaches `super::scenes`, as it does in og-tests-cooked's `support`).
#[cfg(target_arch = "wasm32")]
#[allow(dead_code)]
#[path = "../../og-ui/tests/support/scenes.rs"]
mod scenes;
#[cfg(target_arch = "wasm32")]
#[allow(dead_code)]
#[path = "../../og-tests-cooked/tests/support/text_scenes.rs"]
mod text_scenes;

#[cfg(target_arch = "wasm32")]
mod web {
    use og_render::{Backend, RenderTarget, Renderer};
    use og_tests::compare::Tolerance;
    use og_tests::scenes::{self, GfxScene};
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
        /// A bisect's cut-down ([`og_gfx_bisect`]), applied to every scene
        /// opened after it.
        static BISECT: RefCell<String> = const { RefCell::new(String::new()) };
        /// The text fonts' cook: its manifest and its files by path under
        /// `cooked/`.
        static TEXT_MANIFEST: RefCell<Option<String>> = const { RefCell::new(None) };
        static TEXT_FILES: RefCell<BTreeMap<String, Vec<u8>>> = const { RefCell::new(BTreeMap::new()) };
        /// The open renderer's uncaptured WebGPU errors since it opened. The
        /// browser only prints them to its own console (natively wgpu
        /// panics on one: presentation.rs `watch`); a capture with any fails
        /// with them, so the runner names the error behind a broken frame.
        static GPU_ERRORS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
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

    /// Every WGSL module the renderer compiles (`og_render::shader_sources`),
    /// as JSON `[{label, wgsl}]`: reference-gfx.ts hands each to the
    /// browser's own WGSL compiler, whose uniformity analysis refuses what
    /// naga's (crates/og-render/tests/shaders.rs) lets through: a derivative
    /// after a return that depends on a per-pixel value made the whole
    /// terrain module invalid in Chrome, and every frame with it.
    #[wasm_bindgen]
    pub fn og_gfx_shader_sources() -> String {
        let all: Vec<_> = og_render::shader_sources()
            .into_iter()
            .map(|s| serde_json::json!({ "label": s.label, "wgsl": s.wgsl }))
            .collect();
        serde_json::Value::from(all).to_string()
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
    fn text_fonts() -> Result<Vec<og_bundle::runtime::CookedFont>, String> {
        let manifest = TEXT_MANIFEST.with(|m| m.borrow().clone()).ok_or(
            "no text fonts: the runner cooks crates/og-ui/tests/text-fonts and calls og_gfx_text_manifest and og_gfx_text_file first",
        )?;
        let bundle =
            og_bundle::runtime::Bundle::from_manifest(&manifest).map_err(|e| e.to_string())?;
        TEXT_FILES.with(|f| {
            let files = f.borrow();
            super::text_scenes::fonts_from(&bundle, |path| files.get(path).cloned())
        })
    }

    /// og-ui's reference scene `name` as a gfx scene: one frame, og-tests'
    /// default tolerance (og-ui's rule); the test font compiled
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
            cross_tolerance: Tolerance::CROSS_PLATFORM,
            frames: 1,
            update: None,
            render_scale: 1.0,
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

    /// Whether the scenes opened from now on cull on the CPU
    /// (`Culling::Cpu`) rather than on the GPU where the adapter can.
    #[wasm_bindgen]
    pub fn og_gfx_cpu_culling(cpu: bool) {
        CPU_CULLING.with(|c| c.set(cpu));
    }

    /// Cuts every scene opened from now on down, to bisect a frame that
    /// draws wrong on one backend: a comma list of `gpu` (GPU culling
    /// whatever the frame's size), `keep:<n>` (the first n instances of the
    /// first instance's mesh, the rest kept), `first` (only the first
    /// instance's mesh), `noshadow` (the sun casts none), `nomeshlets` (no
    /// batch draws through the meshlet path), `apart` (the cull submitted
    /// before the frame, not with it), `direct` (the lit passes draw the
    /// culled regions with `draw_indexed`, not the indirect args), `samemat` and `samemesh` (every
    /// instance the first's material or mesh), `sphere:<segments>x<rings>`
    /// (the first instance's mesh's instances drawn with that sphere).
    /// Empty: none.
    #[wasm_bindgen]
    pub fn og_gfx_bisect(spec: String) {
        BISECT.with(|b| *b.borrow_mut() = spec);
    }

    fn cut_down(built: &mut GfxScene) -> Result<(), String> {
        let spec = BISECT.with(|b| b.borrow().clone());
        Renderer::debug_cull_apart(false);
        Renderer::debug_cull_direct(false);
        for knob in spec.split(',').map(str::trim).filter(|k| !k.is_empty()) {
            let scene = &mut built.scene;
            let mesh = scene.instances.first().map(|i| i.mesh);
            match knob.split_once(':') {
                None if knob == "gpu" => built.desc.culling = og_render::Culling::Gpu,
                None if knob == "first" => scene.instances.retain(|i| Some(i.mesh) == mesh),
                None if knob == "noshadow" => scene.light.cast_shadows = false,
                None if knob == "nomeshlets" => scene.meshlets = false,
                None if knob == "apart" => Renderer::debug_cull_apart(true),
                None if knob == "direct" => Renderer::debug_cull_direct(true),
                None if knob == "samemat" => {
                    let m = scene.instances.first().map(|i| i.material);
                    for i in &mut scene.instances {
                        i.material = m.expect("an instance");
                    }
                }
                None if knob == "samemesh" => {
                    for i in &mut scene.instances {
                        i.mesh = mesh.expect("an instance");
                    }
                }
                Some(("sphere", sr)) => {
                    let (a, b) = sr
                        .split_once('x')
                        .and_then(|(a, b)| Some((a.parse().ok()?, b.parse().ok()?)))
                        .ok_or_else(|| format!("bisect: sphere:{sr}"))?;
                    let m = scene.add_mesh(og_render::shapes::sphere(a, b));
                    for i in scene.instances.iter_mut().filter(|i| Some(i.mesh) == mesh) {
                        i.mesh = m;
                    }
                }
                Some(("keep", n)) => {
                    let n: usize = n.parse().map_err(|_| format!("bisect: keep:{n}"))?;
                    let mut kept = 0;
                    scene.instances.retain(|i| {
                        if Some(i.mesh) != mesh {
                            return true;
                        }
                        kept += 1;
                        kept <= n
                    });
                }
                _ => return Err(format!("bisect: no knob {knob}")),
            }
        }
        Ok(())
    }

    /// Builds `scene` (an og-tests scene, or `og-ui/<name>`) and its renderer
    /// (WebGPU, headless). Resolves to a JSON description: tier, adapter,
    /// antialiasing, whether the scene has a reference on this tier, the
    /// frames it renders before its capture and its tolerance.
    #[wasm_bindgen]
    pub async fn og_gfx_open(scene: String) -> Result<String, JsValue> {
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
        built.desc.backend = Backend::WebGpu;
        cut_down(&mut built).map_err(err)?;
        if CPU_CULLING.with(Cell::get) {
            built.desc.culling = og_render::Culling::Cpu;
        }
        let mut renderer = Renderer::new(&built.desc).await.map_err(err)?;
        GPU_ERRORS.with(|e| e.borrow_mut().clear());
        renderer
            .device()
            .on_uncaptured_error(std::sync::Arc::new(|e: og_render::wgpu::Error| {
                GPU_ERRORS.with(|errors| errors.borrow_mut().push(e.to_string()));
            }));
        renderer.set_render_scale(built.render_scale);
        let info = renderer.info();
        // `tolerance` against this tier's own reference, `cross` against
        // another tier's (og_tests::compare::tolerance_for, which a software
        // adapter never gets).
        let t = &built.tolerance;
        let c = built.tolerance.widened(built.cross_tolerance);
        let json = format!(
            "{{\"tier\":{:?},\"adapter\":{:?},\"software\":{},\"antialias\":\"{:?}\",\"frames\":{},\"width\":{},\"height\":{},\"tolerance\":{{\"channel\":{},\"outliers\":{},\"rmse\":{},\"bias\":{}}},\"cross\":{{\"channel\":{},\"outliers\":{},\"rmse\":{},\"bias\":{}}}}}",
            info.backend_tier(),
            info.adapter,
            info.software(),
            info.antialias,
            built.frames.max(1),
            built.desc.width,
            built.desc.height,
            t.channel,
            t.max_outlier_fraction,
            t.rmse,
            t.max_bias,
            c.channel,
            c.max_outlier_fraction,
            c.rmse,
            c.max_bias,
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
        // Poll until the copy lands, yielding to the browser between tries
        // (the readback's map completes on its event loop).
        for _ in 0..10_000 {
            let done = OPEN.with(|o| {
                let o = o.borrow();
                let open = o.as_ref().ok_or_else(|| err("the scene was closed"))?;
                request.try_finish(&open.renderer).map_err(err)
            })?;
            if let Some(image) = done {
                let errors = GPU_ERRORS.with(|e| std::mem::take(&mut *e.borrow_mut()));
                if !errors.is_empty() {
                    return Err(err(format!("WebGPU errors: {}", errors.join("\n"))));
                }
                return Ok(image.pixels);
            }
            tick().await?;
        }
        Err(err("capture: the copy did not land in 10000 polls"))
    }

    /// The open scene's GPU culling counts and buffer names after its last
    /// frame, as JSON (`{"instances", "batches", "buffers": [{name, bytes}]}`),
    /// or `null` when the scene culled on the CPU. A debug readback: the
    /// runner compares one frame's cull between two browsers.
    #[wasm_bindgen]
    pub fn og_gfx_cull_layout() -> Result<String, JsValue> {
        OPEN.with(|o| {
            let o = o.borrow();
            let open = o.as_ref().ok_or_else(|| err("no scene open"))?;
            Ok(match open.renderer.debug_cull_counts() {
                None => "null".to_string(),
                Some((n, b)) => {
                    let buffers = open.renderer.debug_cull_buffers();
                    let list: Vec<String> = buffers
                        .iter()
                        .map(|(name, buf)| {
                            format!("{{\"name\":{name:?},\"bytes\":{}}}", buf.size())
                        })
                        .collect();
                    format!(
                        "{{\"instances\":{n},\"batches\":{b},\"buffers\":[{}]}}",
                        list.join(",")
                    )
                }
            })
        })
    }

    /// The open scene's last frame's render-graph passes in execution
    /// order, as a JSON array of names (a bisect reads which passes drew).
    #[wasm_bindgen]
    pub fn og_gfx_passes() -> Result<String, JsValue> {
        OPEN.with(|o| {
            let o = o.borrow();
            let open = o.as_ref().ok_or_else(|| err("no scene open"))?;
            let names = open
                .renderer
                .frame_graph()
                .map(|p| p.ordered_names())
                .unwrap_or_default();
            Ok(format!("{names:?}"))
        })
    }

    /// The bytes of the open scene's GPU culling buffer `name`
    /// ([`og_gfx_cull_layout`]) as the last frame left them.
    #[wasm_bindgen]
    pub async fn og_gfx_cull_buffer(name: String) -> Result<Vec<u8>, JsValue> {
        let state = std::rc::Rc::new(Cell::new(None::<bool>));
        let staging = OPEN.with(|o| {
            let o = o.borrow();
            let open = o.as_ref().ok_or_else(|| err("no scene open"))?;
            let r = &open.renderer;
            let buffers = r.debug_cull_buffers();
            let (_, src) = buffers
                .iter()
                .find(|(n, _)| *n == name)
                .ok_or_else(|| err(format!("no cull buffer {name}")))?;
            let staging = r
                .device()
                .create_buffer(&og_render::wgpu::BufferDescriptor {
                    label: Some("cull readback"),
                    size: src.size(),
                    usage: og_render::wgpu::BufferUsages::MAP_READ
                        | og_render::wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
            let mut encoder = r.device().create_command_encoder(&Default::default());
            encoder.copy_buffer_to_buffer(src, 0, &staging, 0, src.size());
            r.queue().submit(Some(encoder.finish()));
            let done = state.clone();
            staging
                .slice(..)
                .map_async(og_render::wgpu::MapMode::Read, move |res| {
                    done.set(Some(res.is_ok()))
                });
            Ok::<_, JsValue>(staging)
        })?;
        for _ in 0..10_000 {
            match state.get() {
                Some(true) => {
                    let bytes = staging.slice(..).get_mapped_range().map_err(err)?.to_vec();
                    staging.unmap();
                    return Ok(bytes);
                }
                Some(false) => return Err(err("the cull readback did not map")),
                None => tick().await?,
            }
        }
        Err(err("the cull readback did not land in 10000 polls"))
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
