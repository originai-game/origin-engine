//! What the device can do, as `docs/design.md` §2.6 Tiers names it: a set
//! of capabilities, never a backend and never a quality setting.
//!
//! [`Caps`] is computed once, when the renderer is made, from the adapter's
//! features and downlevel flags (less [`RenderDesc::without_features`]) and
//! clamped by the tier the caller asks for ([`RenderDesc::tier`], a ceiling:
//! a limited tier runs, and is tested, on a full device). Every decision
//! the renderer makes about the device reads it: a pass that needs compute
//! is selected by [`Caps::COMPUTE`], not by the backend it runs on.
//! [`RenderInfo::caps`] reports it.
//!
//! [`RenderDesc::without_features`]: crate::RenderDesc::without_features
//! [`RenderDesc::tier`]: crate::RenderDesc::tier
//! [`RenderInfo::caps`]: crate::RenderInfo::caps

use std::fmt;

/// A set of device capabilities. See the module documentation.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Caps(u16);

impl Caps {
    /// No capability.
    pub const NONE: Caps = Caps(0);
    /// Compute passes (`DownlevelFlags::COMPUTE_SHADERS`).
    pub const COMPUTE: Caps = Caps(1 << 0);
    /// More than one colour attachment in a render pass.
    pub const MRT: Caps = Caps(1 << 1);
    /// Pipelines compiled off the frame: on worker threads natively
    /// (`deferred.rs`), asked for asynchronously on the web (`FrameSpan`).
    pub const ASYNC_PIPELINES: Caps = Caps(1 << 2);
    /// Timestamp queries, granted by the device (`TIMESTAMP_QUERY`).
    pub const TIMESTAMPS: Caps = Caps(1 << 3);
    /// Indirect draws and dispatches (`DownlevelFlags::INDIRECT_EXECUTION`).
    pub const INDIRECT: Caps = Caps(1 << 4);
    /// Indirect draws with a first instance (`INDIRECT_FIRST_INSTANCE`).
    pub const INDIRECT_FIRST_INSTANCE: Caps = Caps(1 << 5);
    /// Storage written from fragment shaders
    /// (`DownlevelFlags::FRAGMENT_WRITABLE_STORAGE`).
    pub const FRAGMENT_STORAGE: Caps = Caps(1 << 6);
    /// Storage read in the vertex stage (`DownlevelFlags::VERTEX_STORAGE`).
    pub const VERTEX_STORAGE: Caps = Caps(1 << 7);
    /// Every capability above: the full tier.
    pub const FULL: Caps = Caps((1 << 8) - 1);

    const NAMES: [(Caps, &'static str); 8] = [
        (Caps::COMPUTE, "compute"),
        (Caps::MRT, "mrt"),
        (Caps::ASYNC_PIPELINES, "asyncPipelines"),
        (Caps::TIMESTAMPS, "timestamps"),
        (Caps::INDIRECT, "indirect"),
        (Caps::INDIRECT_FIRST_INSTANCE, "indirectFirstInstance"),
        (Caps::FRAGMENT_STORAGE, "fragmentStorage"),
        (Caps::VERTEX_STORAGE, "vertexStorage"),
    ];

    /// What an adapter with `features` (already less the caller's
    /// `without_features`), `flags` and `limits` can do, before any tier's
    /// ceiling. Timestamps count here when the adapter has them; the device
    /// takes them away again when it is made without the feature.
    pub fn of(
        features: wgpu::Features,
        flags: wgpu::DownlevelFlags,
        limits: &wgpu::Limits,
    ) -> Caps {
        let mut caps = Caps::ASYNC_PIPELINES;
        let flag =
            |f: wgpu::DownlevelFlags, c: Caps| if flags.contains(f) { c } else { Caps::NONE };
        let feature =
            |f: wgpu::Features, c: Caps| if features.contains(f) { c } else { Caps::NONE };
        caps |= flag(wgpu::DownlevelFlags::COMPUTE_SHADERS, Caps::COMPUTE);
        caps |= flag(wgpu::DownlevelFlags::INDIRECT_EXECUTION, Caps::INDIRECT);
        caps |= flag(
            wgpu::DownlevelFlags::FRAGMENT_WRITABLE_STORAGE,
            Caps::FRAGMENT_STORAGE,
        );
        caps |= flag(wgpu::DownlevelFlags::VERTEX_STORAGE, Caps::VERTEX_STORAGE);
        caps |= feature(wgpu::Features::TIMESTAMP_QUERY, Caps::TIMESTAMPS);
        caps |= feature(
            wgpu::Features::INDIRECT_FIRST_INSTANCE,
            Caps::INDIRECT_FIRST_INSTANCE,
        );
        if limits.max_color_attachments > 1 {
            caps |= Caps::MRT;
        }
        caps
    }

    /// Whether every capability of `needs` is in the set.
    pub const fn contains(self, needs: Caps) -> bool {
        self.0 & needs.0 == needs.0
    }

    /// The capabilities of `needs` the set lacks.
    pub const fn missing(self, needs: Caps) -> Caps {
        Caps(needs.0 & !self.0)
    }

    /// The set less `other`.
    pub const fn without(self, other: Caps) -> Caps {
        Caps(self.0 & !other.0)
    }

    /// The capabilities' names, as `RenderCapabilities` would spell them.
    pub fn names(self) -> impl Iterator<Item = &'static str> {
        Caps::NAMES
            .into_iter()
            .filter(move |(c, _)| self.contains(*c))
            .map(|(_, n)| n)
    }

    /// `flags` less the downlevel flags of the capabilities the set lacks:
    /// what code that asks wgpu's flags sees under the set.
    pub fn downlevel(self, flags: wgpu::DownlevelFlags) -> wgpu::DownlevelFlags {
        let mut flags = flags;
        for (c, f) in [
            (Caps::COMPUTE, wgpu::DownlevelFlags::COMPUTE_SHADERS),
            (Caps::INDIRECT, wgpu::DownlevelFlags::INDIRECT_EXECUTION),
            (
                Caps::FRAGMENT_STORAGE,
                wgpu::DownlevelFlags::FRAGMENT_WRITABLE_STORAGE,
            ),
            (Caps::VERTEX_STORAGE, wgpu::DownlevelFlags::VERTEX_STORAGE),
        ] {
            if !self.contains(c) {
                flags.remove(f);
            }
        }
        flags
    }

    /// `features` less the features of the capabilities the set lacks.
    pub fn features(self, features: wgpu::Features) -> wgpu::Features {
        let mut features = features;
        for (c, f) in [
            (Caps::TIMESTAMPS, wgpu::Features::TIMESTAMP_QUERY),
            (
                Caps::INDIRECT_FIRST_INSTANCE,
                wgpu::Features::INDIRECT_FIRST_INSTANCE,
            ),
        ] {
            if !self.contains(c) {
                features.remove(f);
            }
        }
        features
    }
}

impl std::ops::BitOr for Caps {
    type Output = Caps;
    fn bitor(self, other: Caps) -> Caps {
        Caps(self.0 | other.0)
    }
}

impl std::ops::BitOrAssign for Caps {
    fn bitor_assign(&mut self, other: Caps) {
        self.0 |= other.0;
    }
}

impl std::ops::BitAnd for Caps {
    type Output = Caps;
    fn bitand(self, other: Caps) -> Caps {
        Caps(self.0 & other.0)
    }
}

impl fmt::Debug for Caps {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.names()).finish()
    }
}

/// The tier a renderer is made for ([`crate::RenderDesc::tier`]): a ceiling
/// on the device's capabilities, so a limited tier's passes run on a full
/// device too. Not a quality setting (`docs/design.md` §2.6).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Tier {
    /// Whatever the device can do (WebGPU, Vulkan, Metal, DX12).
    #[default]
    Full,
    /// The mini-game WebGPU tier: no multiple render targets, asynchronous
    /// pipelines or timestamps.
    MiniGame,
    /// The GL tier: as [`Tier::MiniGame`], and no compute, nor the storage
    /// and indirect execution that WebGL2 lacks with it.
    Gl,
}

impl Tier {
    /// The capabilities a device may have on this tier.
    pub const fn ceiling(self) -> Caps {
        let mini_game = Caps::FULL.without(Caps(
            Caps::MRT.0 | Caps::ASYNC_PIPELINES.0 | Caps::TIMESTAMPS.0,
        ));
        match self {
            Tier::Full => Caps::FULL,
            Tier::MiniGame => mini_game,
            Tier::Gl => mini_game.without(Caps(
                Caps::COMPUTE.0
                    | Caps::INDIRECT.0
                    | Caps::INDIRECT_FIRST_INSTANCE.0
                    | Caps::FRAGMENT_STORAGE.0
                    | Caps::VERTEX_STORAGE.0,
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_adapter() -> Caps {
        Caps::of(
            wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::INDIRECT_FIRST_INSTANCE,
            wgpu::DownlevelFlags::all(),
            &wgpu::Limits::default(),
        )
    }

    #[test]
    fn a_full_adapter_has_every_capability() {
        assert_eq!(full_adapter(), Caps::FULL);
    }

    #[test]
    fn tiers_clamp_a_full_adapter() {
        let mini = full_adapter() & Tier::MiniGame.ceiling();
        assert!(mini.contains(Caps::COMPUTE | Caps::INDIRECT | Caps::FRAGMENT_STORAGE));
        for lacking in [Caps::MRT, Caps::ASYNC_PIPELINES, Caps::TIMESTAMPS] {
            assert!(!mini.contains(lacking), "{lacking:?}");
        }
        let gl = full_adapter() & Tier::Gl.ceiling();
        assert_eq!(gl, Caps::NONE);
        assert_eq!(full_adapter() & Tier::Full.ceiling(), Caps::FULL);
    }

    #[test]
    fn a_ceiling_never_adds_what_the_adapter_lacks() {
        let adapter = Caps::of(
            wgpu::Features::empty(),
            wgpu::DownlevelFlags::all() - wgpu::DownlevelFlags::FRAGMENT_WRITABLE_STORAGE,
            &wgpu::Limits::downlevel_webgl2_defaults(),
        );
        let caps = adapter & Tier::Full.ceiling();
        assert!(!caps.contains(Caps::FRAGMENT_STORAGE));
        assert!(!caps.contains(Caps::TIMESTAMPS));
        assert!(!caps.contains(Caps::INDIRECT_FIRST_INSTANCE));
        assert!(caps.contains(Caps::COMPUTE | Caps::MRT));
    }

    #[test]
    fn masks_take_away_only_what_the_set_lacks() {
        let gl = Tier::Gl.ceiling();
        let flags = gl.downlevel(wgpu::DownlevelFlags::all());
        assert!(!flags.contains(wgpu::DownlevelFlags::COMPUTE_SHADERS));
        assert!(!flags.contains(wgpu::DownlevelFlags::VERTEX_STORAGE));
        // Not a capability: kept on every tier.
        assert!(flags.contains(wgpu::DownlevelFlags::ANISOTROPIC_FILTERING));
        assert_eq!(
            Caps::FULL.downlevel(wgpu::DownlevelFlags::all()),
            wgpu::DownlevelFlags::all()
        );
        let all = wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::PIPELINE_CACHE;
        assert_eq!(
            Tier::MiniGame.ceiling().features(all),
            wgpu::Features::PIPELINE_CACHE
        );
        assert_eq!(Caps::FULL.features(all), all);
    }

    #[test]
    fn names_follow_the_set() {
        let names: Vec<_> = (Caps::COMPUTE | Caps::TIMESTAMPS).names().collect();
        assert_eq!(names, ["compute", "timestamps"]);
    }
}
