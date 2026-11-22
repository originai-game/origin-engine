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
    /// Pipelines compiled off the frame by the host's compiler: on worker
    /// threads natively, asked for asynchronously on the web (`deferred.rs`).
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
    /// Colour targets of one pipeline blended each its own way
    /// (`DownlevelFlags::INDEPENDENT_BLEND`): weighted OIT's three targets,
    /// the 2D light map's two.
    pub const INDEPENDENT_BLEND: Caps = Caps(1 << 8);
    /// Storage bindings of two-channel 32-bit textures (`rg32float`),
    /// which every core device has and compatibility mode's lack: the
    /// scatter cards' lit stage reads the prepass's view depth through one.
    pub const RG32_STORAGE: Caps = Caps(1 << 9);
    /// Every capability above: the full tier.
    pub const FULL: Caps = Caps((1 << 10) - 1);

    const NAMES: [(Caps, &'static str); 10] = [
        (Caps::COMPUTE, "compute"),
        (Caps::MRT, "mrt"),
        (Caps::ASYNC_PIPELINES, "asyncPipelines"),
        (Caps::TIMESTAMPS, "timestamps"),
        (Caps::INDIRECT, "indirect"),
        (Caps::INDIRECT_FIRST_INSTANCE, "indirectFirstInstance"),
        (Caps::FRAGMENT_STORAGE, "fragmentStorage"),
        (Caps::VERTEX_STORAGE, "vertexStorage"),
        (Caps::INDEPENDENT_BLEND, "independentBlend"),
        (Caps::RG32_STORAGE, "rg32Storage"),
    ];

    /// What an adapter with `features` (already less the caller's
    /// `without_features`), `flags` and `limits` can do, before any tier's
    /// ceiling. Timestamps count here when the adapter has them; the device
    /// takes them away again when it is made without the feature.
    /// [`Caps::RG32_STORAGE`] counts as every core device has it; the
    /// renderer takes it away where the adapter's format says otherwise.
    pub fn of(
        features: wgpu::Features,
        flags: wgpu::DownlevelFlags,
        limits: &wgpu::Limits,
    ) -> Caps {
        let mut caps = Caps::ASYNC_PIPELINES | Caps::RG32_STORAGE;
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
        caps |= flag(
            wgpu::DownlevelFlags::INDEPENDENT_BLEND,
            Caps::INDEPENDENT_BLEND,
        );
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

    /// The set with `other`.
    pub const fn union(self, other: Caps) -> Caps {
        Caps(self.0 | other.0)
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
            (
                Caps::INDEPENDENT_BLEND,
                wgpu::DownlevelFlags::INDEPENDENT_BLEND,
            ),
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
    /// WebGPU's compatibility mode (`featureLevel: "compatibility"`, the
    /// GLES and D3D11 devices): no storage in the vertex stage, colour
    /// targets blended each their own way or `rg32float` storage, no MSAA
    /// (it multisamples no `rgba16float`), and the device made at that
    /// mode's limits ([`Tier::compat_limits`]). Its shaders keep the mode's
    /// WGSL rules on every tier (og-render's `tests/wgsl_rules.rs`).
    Compat,
}

impl Tier {
    /// The tier's name, as the IDL's `RenderTier` spells it.
    pub const fn name(self) -> &'static str {
        match self {
            Tier::Full => "full",
            Tier::MiniGame => "miniGame",
            Tier::Gl => "gl",
            Tier::Compat => "compat",
        }
    }

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
                    | Caps::VERTEX_STORAGE.0
                    | Caps::INDEPENDENT_BLEND.0
                    | Caps::RG32_STORAGE.0,
            )),
            Tier::Compat => Caps::FULL.without(Caps(
                Caps::VERTEX_STORAGE.0 | Caps::INDEPENDENT_BLEND.0 | Caps::RG32_STORAGE.0,
            )),
        }
    }

    /// `limits` at most the tier's: compatibility mode's defaults on
    /// [`Tier::Compat`] (gpuweb's compatibility-mode proposal §10: 2D and
    /// 1D textures of 4096, four colour attachments, 16 KiB uniform
    /// bindings, 15 inter-stage variables, compute workgroups of 128
    /// invocations and 128 along x and y), `limits` itself on the others.
    /// The mode's storage per stage (four in the fragment stage, none in
    /// the vertex stage) has no wgpu limit yet: the tier's ceiling takes
    /// [`Caps::VERTEX_STORAGE`] away instead.
    pub fn compat_limits(self, limits: wgpu::Limits) -> wgpu::Limits {
        if self != Tier::Compat {
            return limits;
        }
        wgpu::Limits {
            max_texture_dimension_1d: limits.max_texture_dimension_1d.min(4096),
            max_texture_dimension_2d: limits.max_texture_dimension_2d.min(4096),
            max_color_attachments: limits.max_color_attachments.min(4),
            max_uniform_buffer_binding_size: limits.max_uniform_buffer_binding_size.min(16 << 10),
            max_inter_stage_shader_variables: limits.max_inter_stage_shader_variables.min(15),
            max_compute_invocations_per_workgroup: limits
                .max_compute_invocations_per_workgroup
                .min(128),
            max_compute_workgroup_size_x: limits.max_compute_workgroup_size_x.min(128),
            max_compute_workgroup_size_y: limits.max_compute_workgroup_size_y.min(128),
            ..limits
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
        let compat = full_adapter() & Tier::Compat.ceiling();
        assert!(compat.contains(Caps::COMPUTE | Caps::MRT | Caps::FRAGMENT_STORAGE));
        for lacking in [
            Caps::VERTEX_STORAGE,
            Caps::INDEPENDENT_BLEND,
            Caps::RG32_STORAGE,
        ] {
            assert!(!compat.contains(lacking), "{lacking:?}");
        }
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
    fn only_the_compat_tier_lowers_the_limits() {
        let full = wgpu::Limits::default();
        let compat = Tier::Compat.compat_limits(full.clone());
        assert_eq!(compat.max_texture_dimension_2d, 4096);
        assert_eq!(compat.max_compute_invocations_per_workgroup, 128);
        assert_eq!(compat.max_compute_workgroup_size_x, 128);
        assert_eq!(compat.max_uniform_buffer_binding_size, 16 << 10);
        // Never raised past what was asked.
        assert_eq!(
            compat.max_texture_dimension_3d,
            full.max_texture_dimension_3d
        );
        let small = wgpu::Limits::downlevel_webgl2_defaults();
        assert_eq!(
            Tier::Compat
                .compat_limits(small.clone())
                .max_texture_dimension_2d,
            2048
        );
        assert_eq!(Tier::MiniGame.compat_limits(full.clone()), full);
    }

    #[test]
    fn names_follow_the_set() {
        let names: Vec<_> = (Caps::COMPUTE | Caps::TIMESTAMPS).names().collect();
        assert_eq!(names, ["compute", "timestamps"]);
    }
}
