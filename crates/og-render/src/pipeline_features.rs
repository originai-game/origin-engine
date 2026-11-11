//! Lit pipelines specialised to the features a frame uses.
//!
//! The lit shaders carry every feature a scene can turn on: the weather on
//! surfaces, the local lights, rect lights, decals, the probe GI's gather
//! and the water's caustics. Each is behind a pipeline-overridable constant
//! (`OG_WEATHER`, `OG_LOCAL_LIGHTS`, `OG_AREA`, `OG_DECALS`, `OG_GI`,
//! `OG_CAUSTICS`, the terrain's diagnostic views' `OG_DEBUG`, og.Grass's far
//! colour's `OG_GRASS`, MaterialStyle's made surfaces' `OG_SURFACE` and the
//! sun's soft shadows' `OG_PCSS`, true by default) as well as its own runtime test (no rain, no light, no
//! decal: the code is skipped). A pipeline made with a feature's constant
//! false has that code folded away before the backend sees it (naga's
//! `process_overrides`), so it compiles in a fraction of the time: on DX12
//! DXC inlines every call, and the full lit fragment stages took 1.6 to
//! 2 s each.
//!
//! A pipeline made without a feature draws exactly what the full one draws
//! while the frame does not use that feature (its runtime test would have
//! skipped the code). So natively a frame's first use of a lit pipeline
//! makes the variant for the features it uses, and the full variant
//! compiles in the background ([`Priority::Background`]) once that frame
//! is submitted; from then on the full one draws. A feature that turns on
//! before the full variant is ready never stalls a frame: the variant
//! already made draws, without that feature, until the full one is ready.
//! Diagnostic views wait for the full variant too: a beauty-only variant
//! must never silently replace the requested diagnostic or omit its inputs.
//! A capture (`Renderer::render_capture`) draws with the full variants,
//! waiting for them: its bytes are a function of the world and the
//! presentation state (docs/design.md §2.4), never of which variants have
//! compiled, and the variants' binaries differ in their last bit.
//! On the web every pipeline is the full one (the browser compiles them
//! and the host's frame span waits for them asynchronously).
//!
//! A renderer made without some of them ([`LitFeatures`],
//! `RenderDesc::without`) has their overrides false in the text of every
//! module it makes ([`Features::leave_out`]): every pipeline made from it,
//! on every host, the full variant included, is made without their code,
//! and no pipeline switches them back on. It also stops feeding them (a
//! rect light is not kept, the probe GI does not run, decals are not
//! recorded, the shelter layer is not fitted), so a module that carries the
//! code still skips it at its runtime test.

// On the web every lit pipeline is the full variant: the variants'
// machinery is native.
#![cfg_attr(target_arch = "wasm32", allow(dead_code))]

#[cfg(not(target_arch = "wasm32"))]
use crate::deferred::Priority;
use crate::deferred::{Deferred, Sampled};
use std::cell::OnceCell;

/// A set of the lit shaders' optional features.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub(crate) struct Features(u16);

/// Each feature's bit and its override's name.
const NAMES: [(Features, &str); 13] = [
    (Features::WEATHER, "OG_WEATHER"),
    (Features::LOCAL_LIGHTS, "OG_LOCAL_LIGHTS"),
    (Features::DECALS, "OG_DECALS"),
    (Features::GI, "OG_GI"),
    (Features::CAUSTICS, "OG_CAUSTICS"),
    (Features::DEBUG, "OG_DEBUG"),
    (Features::GRASS, "OG_GRASS"),
    (Features::SURFACE, "OG_SURFACE"),
    (Features::SOFT_SHADOWS, "OG_PCSS"),
    (Features::SEA, "OG_SEA"),
    (Features::FLOW, "OG_FLOW"),
    (Features::RIPPLES, "OG_RIPPLES"),
    (Features::AREA, "OG_AREA"),
];

impl Features {
    pub(crate) const NONE: Features = Features(0);
    pub(crate) const WEATHER: Features = Features(1);
    pub(crate) const LOCAL_LIGHTS: Features = Features(2);
    pub(crate) const DECALS: Features = Features(4);
    pub(crate) const GI: Features = Features(8);
    pub(crate) const CAUSTICS: Features = Features(16);
    /// The diagnostic views (`RenderScene::debug`).
    pub(crate) const DEBUG: Features = Features(32);
    /// og.Grass's far colour on its terrain.
    pub(crate) const GRASS: Features = Features(64);
    /// MaterialStyle's made surfaces (`Stylise::material`).
    pub(crate) const SURFACE: Features = Features(128);
    /// The sun's contact-hardening shadows (`DirectionalLight::shadow_softness`
    /// or its diffusion; lighting.wgsl `pcss`).
    pub(crate) const SOFT_SHADOWS: Features = Features(256);
    /// A water surface that is a sea (`Water::wind`: og_core's waves).
    pub(crate) const SEA: Features = Features(512);
    /// A flowing water surface (`Water::flow`, its flow map or a river).
    pub(crate) const FLOW: Features = Features(1024);
    /// A water surface's height field (`Water::heights`, its ripples).
    pub(crate) const RIPPLES: Features = Features(2048);
    /// Rect lights (`LightKind::Rect`; area.wgsl's linearly transformed
    /// cosines and their shadows).
    pub(crate) const AREA: Features = Features(4096);
    pub(crate) const ALL: Features = Features(8191);

    pub(crate) fn contains(self, other: Features) -> bool {
        self.0 & other.0 == other.0
    }

    fn with(self, other: Features, on: bool) -> Features {
        if on { Features(self.0 | other.0) } else { self }
    }

    /// The features whose override `source` (a WGSL module) declares true:
    /// those a pipeline made from it can leave out. One that
    /// [`Self::leave_out`] set false is not among them: it is out of every
    /// pipeline already.
    pub(crate) fn declared(source: &str) -> Features {
        NAMES.iter().fold(Features::NONE, |f, (bit, name)| {
            f.with(
                *bit,
                source.contains(&format!("override {name}: bool = true;")),
            )
        })
    }

    /// `source` (a WGSL module) with the overrides of the features in
    /// `self` false: every pipeline made from it, natively and on the web,
    /// then has their code folded away. A renderer made without them
    /// (`RenderDesc::without`) makes each module that carries them so.
    /// The weather on surfaces also leaves the text: a false override's
    /// code still reaches the browser's shader compilers, and the
    /// weather's (its noise, masks, ripples and snow) is the largest, so
    /// [`crate::WEATHER_SURFACE_STUB`] takes its place.
    pub(crate) fn leave_out(self, source: String) -> String {
        let source = if self.contains(Features::WEATHER) {
            source.replace(crate::WEATHER_SURFACE_WGSL, crate::WEATHER_SURFACE_STUB)
        } else {
            source
        };
        NAMES
            .iter()
            .filter(|(bit, _)| self.contains(*bit))
            .fold(source, |s, (_, name)| {
                s.replace(
                    &format!("override {name}: bool = true;"),
                    &format!("override {name}: bool = false;"),
                )
            })
    }

    /// The features `scene` draws with: every one whose runtime test could
    /// pass this frame (a superset is always safe; a missing one would
    /// leave it undrawn).
    pub(crate) fn of(scene: &crate::RenderScene) -> Features {
        let shore = scene
            .water
            .iter()
            .any(|w| w.mesh.is_none() && w.shore.is_some());
        Features::NONE
            .with(Features::WEATHER, scene.surface_weather.is_some() || shore)
            .with(Features::LOCAL_LIGHTS, !scene.lights.is_empty())
            .with(Features::DECALS, !scene.decals.is_empty())
            .with(Features::GI, scene.look.post.gi_probes.is_some())
            .with(
                Features::CAUSTICS,
                scene.water.iter().any(|w| w.caustics > 0.0),
            )
            .with(Features::DEBUG, scene.debug.lit_code() != 0.0)
            .with(Features::GRASS, !scene.grass.is_empty())
            .with(
                Features::SURFACE,
                scene.look.stylise.material.surface_uniform()[0] > 0.5,
            )
            .with(
                Features::SOFT_SHADOWS,
                scene.light.shadow_softness_and_fill().0 > 0.0,
            )
            .with(
                Features::SEA,
                scene.water.iter().any(|w| crate::water::ocean(w).is_some()),
            )
            .with(
                Features::FLOW,
                scene.water.iter().any(|w| {
                    w.flow.length_squared() >= 1e-12 || w.flow_map.is_some() || w.river.is_some()
                }),
            )
            .with(
                Features::RIPPLES,
                scene.water.iter().any(|w| w.heights.is_some()),
            )
            .with(
                Features::AREA,
                scene
                    .lights
                    .iter()
                    .any(|l| matches!(l.kind, crate::LightKind::Rect { .. })),
            )
    }

    /// The pipeline constants that leave out what `self` lacks, for a
    /// module declaring `declared` (a constant the module does not declare
    /// is an error).
    pub(crate) fn constants(self, declared: Features) -> Vec<(&'static str, f64)> {
        NAMES
            .iter()
            .filter(|(bit, _)| declared.contains(*bit) && !self.contains(*bit))
            .map(|(_, name)| (*name, 0.0))
            .collect()
    }
}

impl std::ops::BitOr for Features {
    type Output = Features;
    fn bitor(self, o: Features) -> Features {
        Features(self.0 | o.0)
    }
}

impl std::ops::Not for Features {
    type Output = Features;
    fn not(self) -> Features {
        Features(!self.0 & Features::ALL.0)
    }
}

/// Optional lit features a renderer can be made without
/// ([`RenderDesc::without`](crate::RenderDesc::without), the IDL's
/// RenderOptions.without). Every lit module carries each of them, reachable
/// behind its runtime test; one left out is in no pipeline the renderer
/// makes (its override false, see the module's documentation) and is not
/// drawn: a scene that uses it draws as if it did not, and the log says so
/// once. Empty (the default): every one, as before the option.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub struct LitFeatures(Features);

impl LitFeatures {
    pub const NONE: LitFeatures = LitFeatures(Features::NONE);
    /// The weather on lit surfaces (`RenderScene::surface_weather`: wet
    /// ground, streaks, puddles, snow, dust) and the shore a sea's surf
    /// wets: weather_surface.wgsl, `OG_WEATHER`.
    pub const SURFACE_WEATHER: LitFeatures = LitFeatures(Features::WEATHER);
    /// The probe GI's gather (`PostEffects::gi_probes`): `OG_GI`.
    pub const GI: LitFeatures = LitFeatures(Features::GI);
    /// Rect lights (`LightKind::Rect`) and their shadows: `OG_AREA`.
    pub const AREA_LIGHTS: LitFeatures = LitFeatures(Features::AREA);
    /// Decals (`RenderScene::decals`): `OG_DECALS`.
    pub const DECALS: LitFeatures = LitFeatures(Features::DECALS);
    /// A water surface's caustics on what lies below it (`Water::caustics`):
    /// `OG_CAUSTICS`.
    pub const CAUSTICS: LitFeatures = LitFeatures(Features::CAUSTICS);
    /// Each one by its IDL name (LitFeature), in the IDL's order.
    pub const NAMED: [(LitFeatures, &'static str); 5] = [
        (LitFeatures::SURFACE_WEATHER, "surfaceWeather"),
        (LitFeatures::GI, "gi"),
        (LitFeatures::AREA_LIGHTS, "areaLights"),
        (LitFeatures::DECALS, "decals"),
        (LitFeatures::CAUSTICS, "caustics"),
    ];

    pub fn contains(self, other: LitFeatures) -> bool {
        self.0.contains(other.0)
    }

    pub fn is_empty(self) -> bool {
        self.0 == Features::NONE
    }

    /// The one named `name` (LitFeature), if there is one.
    pub fn named(name: &str) -> Option<LitFeatures> {
        LitFeatures::NAMED
            .iter()
            .find(|(_, n)| *n == name)
            .map(|(f, _)| *f)
    }

    /// The IDL names of those in `self`, in the IDL's order.
    pub fn names(self) -> Vec<&'static str> {
        LitFeatures::NAMED
            .iter()
            .filter(|(f, _)| self.contains(*f))
            .map(|(_, n)| *n)
            .collect()
    }

    /// Their shaders' features.
    pub(crate) fn features(self) -> Features {
        self.0
    }

    /// The override of a single feature (its first, in `NAMES` order).
    #[cfg(test)]
    fn override_name(self) -> &'static str {
        NAMES
            .iter()
            .find(|(bit, _)| self.0.contains(*bit))
            .map(|(_, n)| *n)
            .expect("a feature")
    }
}

impl std::ops::BitOr for LitFeatures {
    type Output = LitFeatures;
    fn bitor(self, o: LitFeatures) -> LitFeatures {
        LitFeatures(self.0 | o.0)
    }
}

/// What a variant is held in: a [`Deferred`] or a [`Sampled`] pipeline.
pub(crate) trait Cell {
    type Value;
    /// Starts it on the compile workers at `priority` (natively).
    #[cfg(not(target_arch = "wasm32"))]
    fn start_at(&self, priority: Priority);
    /// Its value if it is made, without waiting.
    fn ready(&self) -> Option<&Self::Value>;
    /// Its value, waited for or made now.
    fn get(&self) -> &Self::Value;
    /// Takes what the full variant `full` draws at (its sample count).
    fn follow(&self, _full: &Self) {}
}

impl<T: crate::deferred::AnywhereOnce + 'static> Cell for Deferred<T> {
    type Value = T;
    #[cfg(not(target_arch = "wasm32"))]
    fn start_at(&self, priority: Priority) {
        Deferred::start_at(self, priority);
    }
    fn ready(&self) -> Option<&T> {
        Deferred::ready(self)
    }
    fn get(&self) -> &T {
        Deferred::get(self)
    }
}

impl<T: crate::deferred::AnywhereOnce + 'static> Cell for Sampled<T> {
    type Value = T;
    #[cfg(not(target_arch = "wasm32"))]
    fn start_at(&self, priority: Priority) {
        Sampled::start_at(self, priority);
    }
    fn ready(&self) -> Option<&T> {
        Sampled::ready(self)
    }
    fn get(&self) -> &T {
        Sampled::get(self)
    }
    fn follow(&self, full: &Self) {
        self.set_samples(full.samples());
    }
}

#[cfg(not(target_arch = "wasm32"))]
type MakeCell<C> = std::sync::Arc<dyn Fn(Features) -> C + Send + Sync>;
#[cfg(target_arch = "wasm32")]
type MakeCell<C> = std::rc::Rc<dyn Fn(Features) -> C>;

/// A lit pipeline as the variant for a frame's features and the full one
/// (see the module's documentation).
pub(crate) struct Variants<C> {
    declared: Features,
    make: MakeCell<C>,
    full: C,
    lean: OnceCell<(Features, C)>,
}

impl<C: Cell> Variants<C> {
    /// `make` builds the (unstarted) cell of the variant with these
    /// features, of a module declaring `declared`.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn new(
        declared: Features,
        make: impl Fn(Features) -> C + Send + Sync + 'static,
    ) -> Variants<C> {
        Variants::with(declared, std::sync::Arc::new(make))
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn new(declared: Features, make: impl Fn(Features) -> C + 'static) -> Variants<C> {
        Variants::with(declared, std::rc::Rc::new(make))
    }

    fn with(declared: Features, make: MakeCell<C>) -> Variants<C> {
        Variants {
            declared,
            full: make(Features::ALL),
            make,
            lean: OnceCell::new(),
        }
    }

    /// `features` with what the module does not declare: their code is
    /// always in it. Diagnostics require the full shading path immediately.
    fn wanted(&self, features: Features) -> Features {
        if features.contains(Features::DEBUG) {
            Features::ALL
        } else {
            features | !self.declared
        }
    }

    /// Starts what a frame with `features` will draw with (natively; on the
    /// web nothing): the full variant if that is what it needs, else the
    /// variant for them, and the full one behind it in the background.
    pub(crate) fn start(&self, features: Features) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if self.full.ready().is_some() {
                return;
            }
            let features = self.wanted(features);
            if features == Features::ALL {
                self.full.start_at(Priority::Frame);
                return;
            }
            if self.lean.get().is_none() {
                let lean = (self.make)(features);
                lean.follow(&self.full);
                lean.start_at(Priority::Frame);
                let _ = self.lean.set((features, lean));
            }
            self.full.start_at(Priority::Background);
        }
        #[cfg(target_arch = "wasm32")]
        let _ = features;
    }

    /// What a frame with `features` draws with: the full variant once it
    /// is made; else the variant made for the features, waited for if it is
    /// compiling. A feature it lacks is not drawn until the full one is
    /// ready (that frame does not wait for it). With diagnostics or every
    /// feature (a capture's, `Renderer::capturing`) it is the full variant, waited
    /// for, so what it draws never depends on what has compiled.
    pub(crate) fn get(&self, features: Features) -> &C::Value {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = features;
            self.full.get()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(v) = self.full.ready() {
                return v;
            }
            if self.wanted(features) == Features::ALL {
                return self.full.get();
            }
            self.start(features);
            self.lean.get().expect("started").1.get()
        }
    }

    /// The full variant (what [`Renderer::warm`](crate::Renderer::warm)
    /// and the cells' sample counts reach).
    pub(crate) fn full(&self) -> &C {
        &self.full
    }

    /// Every variant made so far.
    pub(crate) fn each(&self) -> impl Iterator<Item = &C> {
        std::iter::once(&self.full).chain(self.lean.get().map(|(_, c)| c))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_leave_out_only_declared_missing_features() {
        let declared =
            Features::declared("override OG_WEATHER: bool = true;\noverride OG_GI: bool = true;\n");
        assert_eq!(declared, Features::WEATHER | Features::GI);
        assert_eq!(Features::GI.constants(declared), vec![("OG_WEATHER", 0.0)]);
        assert!(Features::ALL.constants(declared).is_empty());
    }

    /// A feature a renderer is made without (RenderDesc::without) is false
    /// in the module's text: no pipeline sets it, so every one made from
    /// it, the full variant included and on every host, lacks its code.
    /// With nothing left out the text is untouched.
    #[test]
    fn a_feature_left_out_is_false_in_the_text_and_never_set() {
        let source = "override OG_WEATHER: bool = true;\noverride OG_GI: bool = true;\noverride OG_AREA: bool = true;\n".to_string();
        assert_eq!(
            LitFeatures::NONE.features().leave_out(source.clone()),
            source
        );
        let off = (LitFeatures::SURFACE_WEATHER | LitFeatures::AREA_LIGHTS).features();
        let lean = off.leave_out(source);
        assert!(
            lean.contains("override OG_WEATHER: bool = false;"),
            "{lean}"
        );
        assert!(lean.contains("override OG_AREA: bool = false;"), "{lean}");
        assert!(lean.contains("override OG_GI: bool = true;"), "{lean}");
        let declared = Features::declared(&lean);
        assert_eq!(declared, Features::GI);
        assert!(Features::ALL.constants(declared).is_empty());
        assert_eq!(Features::NONE.constants(declared), vec![("OG_GI", 0.0)]);
    }

    /// The weather on surfaces left out leaves its text too: each module
    /// that carries it (the standard, physical, grass and terrain lit
    /// modules) has the stub in its place, which declares the override
    /// false, so no pipeline sets it. Another feature left out keeps it.
    #[test]
    fn the_weather_left_out_leaves_the_module_text() {
        let hooks = crate::hooks::default_hooks();
        for source in [
            crate::lit_source_with(&hooks, false),
            crate::physical_source(&hooks),
            crate::grass::lit_source(),
            crate::terrain::terrain_lit_source("", "", ""),
        ] {
            assert!(source.contains(crate::WEATHER_SURFACE_WGSL));
            let others = Features::GI | Features::AREA | Features::DECALS | Features::CAUSTICS;
            assert!(
                others
                    .leave_out(source.clone())
                    .contains(crate::WEATHER_SURFACE_WGSL)
            );
            let lean = Features::WEATHER.leave_out(source.clone());
            assert!(!lean.contains(crate::WEATHER_SURFACE_WGSL));
            assert!(!lean.contains("fn weather_streaks("));
            assert!(lean.contains("override OG_WEATHER: bool = false;"));
            assert_eq!(
                source.len() - lean.len(),
                crate::WEATHER_SURFACE_WGSL.len() - crate::WEATHER_SURFACE_STUB.len()
            );
            assert!(!Features::declared(&lean).contains(Features::WEATHER));
        }
    }

    /// Each listed feature is an override the lit modules declare (a
    /// renamed constant or a feature whose code is not behind one would
    /// leave its code in every pipeline), and leaving every one out leaves
    /// none of them declared in any of those modules.
    #[test]
    fn every_lit_feature_is_an_override_of_the_lit_modules() {
        let all = LitFeatures::NAMED
            .iter()
            .fold(LitFeatures::NONE, |a, (f, _)| a | *f);
        let hooks = crate::hooks::default_hooks();
        let modules = [
            crate::lit_source_with(&hooks, false),
            crate::physical_source(&hooks),
            crate::grass::lit_source(),
        ];
        for source in &modules {
            let declared = Features::declared(source);
            let lean = Features::declared(&all.features().leave_out(source.clone()));
            for (f, name) in LitFeatures::NAMED {
                // Grass carries no decals (and nothing else it lacks is
                // declared there either).
                if source.contains(&format!("override {}", f.override_name())) {
                    assert!(declared.contains(f.features()), "{name}");
                    assert!(!lean.contains(f.features()), "{name} left out");
                }
            }
        }
        let standard = Features::declared(&modules[0]);
        for (f, name) in LitFeatures::NAMED {
            assert!(
                standard.contains(f.features()),
                "the standard lit module lacks {name}"
            );
        }
    }

    /// The IDL's names, each its own feature.
    #[test]
    fn lit_features_by_name() {
        for (f, name) in LitFeatures::NAMED {
            assert_eq!(LitFeatures::named(name), Some(f));
            assert_eq!(f.names(), vec![name]);
        }
        assert_eq!(LitFeatures::named("toon"), None);
        let all = LitFeatures::NAMED
            .iter()
            .fold(LitFeatures::NONE, |a, (f, _)| a | *f);
        assert_eq!(all.names().len(), LitFeatures::NAMED.len());
    }

    /// A frame lacking only a feature the module is made without draws the
    /// full variant: no variant is made for it.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn a_feature_left_out_needs_no_variant() {
        let source = "override OG_WEATHER: bool = true;\noverride OG_GI: bool = true;\n";
        let declared = Features::declared(&Features::WEATHER.leave_out(source.to_string()));
        let made = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let log = made.clone();
        let v = Variants::new(declared, move |f: Features| {
            log.lock().unwrap().push(f);
            Deferred::anywhere(move || f)
        });
        assert_eq!(*v.get(Features::GI), Features::ALL);
        assert_eq!(*made.lock().unwrap(), vec![Features::ALL]);
        assert_eq!(v.each().count(), 1);
    }

    /// The first frame's variant draws; the full one replaces it once made,
    /// and a feature turned on before then does not wait for it.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn the_full_variant_takes_over_without_a_wait() {
        let (go, gate) = std::sync::mpsc::channel::<()>();
        let gate = std::sync::Arc::new(std::sync::Mutex::new(gate));
        let v = Variants::new(Features::ALL, move |f: Features| {
            let gate = gate.clone();
            Deferred::anywhere(move || {
                if f == Features::ALL {
                    // The full variant compiles until the test lets it finish.
                    let _ = gate.lock().unwrap().recv();
                }
                f
            })
        });
        assert_eq!(*v.get(Features::NONE), Features::NONE);
        crate::deferred::release_background();
        // A light turns on: the lean variant still draws, at once.
        assert_eq!(*v.get(Features::LOCAL_LIGHTS), Features::NONE);
        go.send(()).unwrap();
        let t0 = std::time::Instant::now();
        while v.full().ready().is_none() {
            assert!(
                t0.elapsed().as_secs() < 10,
                "the full variant never finished"
            );
            std::thread::yield_now();
        }
        assert_eq!(*v.get(Features::LOCAL_LIGHTS), Features::ALL);
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn diagnostics_never_draw_the_incomplete_beauty_variant() {
        // Keep compilation pending deterministically; get supplies the
        // requested pipeline, without a worker racing the assertion.
        struct Pending(Features);
        impl Cell for Pending {
            type Value = Features;
            fn start_at(&self, _: Priority) {}
            fn ready(&self) -> Option<&Features> {
                None
            }
            fn get(&self) -> &Features {
                &self.0
            }
        }
        let v = Variants::new(Features::ALL, Pending);
        assert_eq!(*v.get(Features::NONE), Features::NONE);
        assert_eq!(*v.get(Features::LOCAL_LIGHTS), Features::NONE);
        assert_eq!(*v.get(Features::DEBUG), Features::ALL);
    }
}
