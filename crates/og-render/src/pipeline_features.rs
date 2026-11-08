//! Lit pipelines specialised to the features a frame uses.
//!
//! The lit shaders carry every feature a scene can turn on: the weather on
//! surfaces, the local lights, decals, the probe GI's gather and the
//! water's caustics. Each is behind a pipeline-overridable constant
//! (`OG_WEATHER`, `OG_LOCAL_LIGHTS`, `OG_DECALS`, `OG_GI`, `OG_CAUSTICS`,
//! the terrain's diagnostic views' `OG_DEBUG`, og.Grass's far colour's
//! `OG_GRASS`, MaterialStyle's made surfaces' `OG_SURFACE` and the sun's
//! soft shadows' `OG_PCSS`, true by default) as well as its own runtime test (no rain, no light, no
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
//! A capture (`Renderer::render_capture`) draws with the full variants,
//! waiting for them: its bytes are a function of the world and the
//! presentation state (docs/design.md §2.4), never of which variants have
//! compiled, and the variants' binaries differ in their last bit.
//! On the web every pipeline is the full one (the browser compiles them
//! and the host's frame span waits for them asynchronously).

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
const NAMES: [(Features, &str); 12] = [
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
    pub(crate) const ALL: Features = Features(4095);

    pub(crate) fn contains(self, other: Features) -> bool {
        self.0 & other.0 == other.0
    }

    fn with(self, other: Features, on: bool) -> Features {
        if on { Features(self.0 | other.0) } else { self }
    }

    /// The features whose override `source` (a WGSL module) declares.
    pub(crate) fn declared(source: &str) -> Features {
        NAMES.iter().fold(Features::NONE, |f, (bit, name)| {
            f.with(*bit, source.contains(&format!("override {name}: bool")))
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
            .with(Features::GI, scene.post.gi_probes.is_some())
            .with(
                Features::CAUSTICS,
                scene.water.iter().any(|w| w.caustics > 0.0),
            )
            .with(Features::DEBUG, scene.debug.lit_code() != 0.0)
            .with(Features::GRASS, !scene.grass.is_empty())
            .with(
                Features::SURFACE,
                scene.stylise.material.surface_uniform()[0] > 0.5,
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
    /// always in it.
    fn wanted(&self, features: Features) -> Features {
        features | !self.declared
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
    /// ready (that frame does not wait for it). With every feature (a
    /// capture's, `Renderer::capturing`) it is the full variant, waited
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
}
