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
pub struct Caps(u32);

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
    /// Render-attachment support for `rg32float` textures (2D shadow maps
    /// and dual-channel float depth targets); devices lacking it fall back
    /// to `rgba16float`.
    pub const RG32_RENDERABLE: Caps = Caps(1 << 10);
    /// Extended per-stage binding budgets beyond WebGPU's 16-texture
    /// compatibility floor (`max_sampled_textures_per_shader_stage >= 24`
    /// and `max_storage_buffers_per_shader_stage >= 8`), allowing unpacked
    /// screen-space AO/contact/SSGI/bent-normal channels on `Tier::Full`.
    pub const HIGH_BINDING_SLOTS: Caps = Caps(1 << 11);
    /// Render-attachment support for packed 32-bit HDR RGB (`rg11b10ufloat`),
    /// halving VRAM and bandwidth for alpha-free HDR post targets.
    pub const RG11B10_RENDERABLE: Caps = Caps(1 << 12);
    /// Unclipped depth (`Features::DEPTH_CLIP_CONTROL`), preventing near-plane
    /// shadow pancaking clips on shadow cascades.
    pub const DEPTH_CLIP_CONTROL: Caps = Caps(1 << 13);
    /// Native 16-bit floating-point WGSL arithmetic (`Features::SHADER_F16`).
    pub const SHADER_F16: Caps = Caps(1 << 14);
    /// Subgroup / warp wave intrinsics (`Features::SUBGROUP`).
    pub const SUBGROUP: Caps = Caps(1 << 15);
    /// Every capability above: the full tier.
    pub const FULL: Caps = Caps((1 << 16) - 1);

    const NAMES: [(Caps, &'static str); 16] = [
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
        (Caps::RG32_RENDERABLE, "rg32Renderable"),
        (Caps::HIGH_BINDING_SLOTS, "highBindingSlots"),
        (Caps::RG11B10_RENDERABLE, "rg11b10Renderable"),
        (Caps::DEPTH_CLIP_CONTROL, "depthClipControl"),
        (Caps::SHADER_F16, "shaderF16"),
        (Caps::SUBGROUP, "subgroup"),
    ];

    /// What an adapter with `features` (already less the caller's
    /// `without_features`), `flags` and `limits` can do, before any tier's
    /// ceiling. Timestamps count here when the adapter has them; the device
    /// takes them away again when it is made without the feature.
    /// [`Caps::RG32_STORAGE`] and [`Caps::RG32_RENDERABLE`] count as every
    /// core device has them; the renderer takes them away where the adapter's
    /// format says otherwise.
    pub fn of(
        features: wgpu::Features,
        flags: wgpu::DownlevelFlags,
        limits: &wgpu::Limits,
    ) -> Caps {
        let mut caps = Caps::ASYNC_PIPELINES | Caps::RG32_STORAGE | Caps::RG32_RENDERABLE;
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
        caps |= feature(
            wgpu::Features::RG11B10UFLOAT_RENDERABLE,
            Caps::RG11B10_RENDERABLE,
        );
        caps |= feature(wgpu::Features::DEPTH_CLIP_CONTROL, Caps::DEPTH_CLIP_CONTROL);
        caps |= feature(wgpu::Features::SHADER_F16, Caps::SHADER_F16);
        caps |= feature(wgpu::Features::SUBGROUP, Caps::SUBGROUP);
        if limits.max_color_attachments > 1 {
            caps |= Caps::MRT;
        }
        if limits.max_sampled_textures_per_shader_stage >= 24
            && limits.max_storage_buffers_per_shader_stage >= 8
        {
            caps |= Caps::HIGH_BINDING_SLOTS;
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
            (
                Caps::RG11B10_RENDERABLE,
                wgpu::Features::RG11B10UFLOAT_RENDERABLE,
            ),
            (Caps::DEPTH_CLIP_CONTROL, wgpu::Features::DEPTH_CLIP_CONTROL),
            (Caps::SHADER_F16, wgpu::Features::SHADER_F16),
            (Caps::SUBGROUP, wgpu::Features::SUBGROUP),
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

    /// The tier a renderer asked for `asked` is made for on this adapter:
    /// [`Tier::Compat`] for [`Tier::Full`] on a WebGPU adapter without
    /// `core-features-and-limits`, a compatibility-mode device (the browser
    /// validates by that mode's rules); `asked` everywhere else. A lower
    /// tier keeps its own ceiling there, under the adapter's capabilities,
    /// which wgpu reports as the mode allows them. The native backends
    /// enforce no compatibility rules: a native adapter keeps `asked`.
    pub fn for_device(asked: Tier, info: &wgpu::AdapterInfo, features: wgpu::Features) -> Tier {
        let compat_device = info.backend == wgpu::Backend::BrowserWebGpu
            && !features.contains(wgpu::Features::CORE_FEATURES_AND_LIMITS);
        if asked == Tier::Full && compat_device {
            Tier::Compat
        } else {
            asked
        }
    }

    /// The capabilities a device may have on this tier.
    pub const fn ceiling(self) -> Caps {
        let full_only = Caps::HIGH_BINDING_SLOTS.0
            | Caps::RG11B10_RENDERABLE.0
            | Caps::DEPTH_CLIP_CONTROL.0
            | Caps::SHADER_F16.0
            | Caps::SUBGROUP.0;
        let mini_game = Caps::FULL.without(Caps(
            Caps::MRT.0 | Caps::ASYNC_PIPELINES.0 | Caps::TIMESTAMPS.0 | full_only,
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
                    | Caps::RG32_STORAGE.0
                    | Caps::RG32_RENDERABLE.0,
            )),
            Tier::Compat => Caps::FULL.without(Caps(
                Caps::VERTEX_STORAGE.0
                    | Caps::INDEPENDENT_BLEND.0
                    | Caps::RG32_STORAGE.0
                    | Caps::RG32_RENDERABLE.0
                    | full_only,
            )),
        }
    }

    /// `limits` at most the tier's: compatibility mode's defaults on
    /// [`Tier::Compat`] (`wgpu::Limits::compatibility_defaults`: 2D and 1D
    /// textures of 4096, four colour attachments, 16 KiB uniform bindings,
    /// 15 inter-stage variables, compute workgroups of 128 invocations and
    /// 128 along x and y), `limits` itself on the others. The storage a
    /// stage may bind stays `limits`' (the mode's defaults, none in the
    /// vertex stage, are what every compatibility device has, not a
    /// ceiling: the UI's paint reads its quads from storage in the vertex
    /// stage, and the adapter says how many it has). The tier's ceiling
    /// takes [`Caps::VERTEX_STORAGE`] away from the GPU-driven paths.
    pub fn compat_limits(self, limits: wgpu::Limits) -> wgpu::Limits {
        if self != Tier::Compat {
            return limits;
        }
        let mode = wgpu::Limits::compatibility_defaults();
        wgpu::Limits {
            max_texture_dimension_1d: limits
                .max_texture_dimension_1d
                .min(mode.max_texture_dimension_1d),
            max_texture_dimension_2d: limits
                .max_texture_dimension_2d
                .min(mode.max_texture_dimension_2d),
            max_color_attachments: limits.max_color_attachments.min(mode.max_color_attachments),
            max_uniform_buffer_binding_size: limits
                .max_uniform_buffer_binding_size
                .min(mode.max_uniform_buffer_binding_size),
            max_inter_stage_shader_variables: limits
                .max_inter_stage_shader_variables
                .min(mode.max_inter_stage_shader_variables),
            max_compute_invocations_per_workgroup: limits
                .max_compute_invocations_per_workgroup
                .min(mode.max_compute_invocations_per_workgroup),
            max_compute_workgroup_size_x: limits
                .max_compute_workgroup_size_x
                .min(mode.max_compute_workgroup_size_x),
            max_compute_workgroup_size_y: limits
                .max_compute_workgroup_size_y
                .min(mode.max_compute_workgroup_size_y),
            ..limits
        }
    }
}
