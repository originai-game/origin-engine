//! Structured procedural inputs. These schemas do not replace the OGT frontends.
//! Relationships (sample counts, ordered curves, inheritance and graph cycles)
//! remain domain validation, not independent field constraints.
use super::{ContentRef, Kind};
use og_core::reflect::source::{InputSchema, schemars::JsonSchema};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::OnceLock;

mod fx;

macro_rules! input {
    ($(#[$meta:meta])* $name:ident { $($(#[$field:meta])* $key:ident: $ty:ty),* $(,)? }) => {
        #[derive(Debug, Deserialize, JsonSchema)]
        $(#[$meta])*
        #[schemars(crate = "og_core::reflect::source::schemars")]
        #[serde(deny_unknown_fields, rename_all = "camelCase")]
        pub struct $name { $($(#[$field])* pub $key: $ty),* }
        impl $name {
            pub fn schema() -> &'static InputSchema<Self> {
                static SCHEMA: OnceLock<InputSchema<$name>> = OnceLock::new();
                SCHEMA.get_or_init(|| InputSchema::new().expect(stringify!($name)))
            }
        }
    };
}

macro_rules! words {
    ($name:ident { $($variant:ident),* $(,)? }) => {
        #[derive(Debug, Deserialize, JsonSchema)]
        #[schemars(crate = "og_core::reflect::source::schemars")]
        #[serde(rename_all = "kebab-case")]
        pub enum $name { $($variant),* }
    };
}

pub mod terrain;
pub mod theme;
pub use terrain::TerrainInput;
pub use theme::ThemeInput;

// The existing cook configuration is itself the structured input, not a copy.
pub use crate::cook::systems::effect_texture::Config as EffectTextureInput;

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(transparent)]
pub struct Pcm(#[schemars(range(min = -1, max = 1))] pub f32);

input! {
    /// Procedural sound: optional preset plus explicit oscillator/sample layers.
    /// At least one preset/layer and maximum rendered duration are domain checks.
    #[schemars(extend("x-og-output-kind" = "sound"))]
    SfxInput {
        preset: Option<SfxPreset>,
        seed: Option<u32>,
        pitch: Option<f64>,
        length: Option<f64>,
        #[schemars(range(min = 0))]
        volume: Option<f64>,
        layers: Option<Vec<SynthLayer>>,
    }
}
words!(SfxPreset {
    Coin,
    Pickup,
    Powerup,
    Jump,
    Land,
    Footstep,
    Hit,
    Hurt,
    Explosion,
    Laser,
    Whoosh,
    Door,
    Death,
    Click,
    Select,
    Back,
    Error,
    Levelup
});
words!(Wave {
    Sine,
    Square,
    Saw,
    Triangle,
    Noise
});
input! {
    /// A mono sample replaces the oscillator; sample layers reject FM.
    SynthLayer {
        wave: Option<Wave>,
        sample: Option<SampleInput>,
        #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "Hz"))]
        freq: Option<f64>,
        #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "Hz"))]
        freq_end: Option<f64>,
        #[schemars(extend("x-og-unit" = "octaves/s"))]
        slide: Option<f64>,
        #[schemars(range(min = 0))]
        arp: Option<f64>,
        #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
        arp_at: Option<f64>,
        #[schemars(range(min = 0))]
        vib_depth: Option<f64>,
        #[schemars(range(min = 0), extend("x-og-unit" = "Hz"))]
        vib_rate: Option<f64>,
        #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
        attack: Option<f64>,
        #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
        sustain: Option<f64>,
        #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
        decay: Option<f64>,
        #[schemars(range(min = 0, max = 1))]
        sustain_level: Option<f64>,
        #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
        release: Option<f64>,
        #[schemars(range(min = 0))]
        punch: Option<f64>,
        #[schemars(range(min = 0.01, max = 0.99))]
        duty: Option<f64>,
        #[schemars(range(min = 0), extend("x-og-unit" = "Hz"))]
        lowpass: Option<f64>,
        #[schemars(extend("x-og-unit" = "octaves/s"))]
        lp_slide: Option<f64>,
        #[schemars(range(min = 0))]
        q: Option<f64>,
        #[schemars(range(min = 0), extend("x-og-unit" = "Hz"))]
        highpass: Option<f64>,
        #[schemars(range(min = 0), extend("x-og-unit" = "Hz"))]
        fm_freq: Option<f64>,
        fm_index: Option<f64>,
        #[schemars(range(min = 0))]
        drive: Option<f64>,
        #[schemars(range(min = 0))]
        gain: Option<f64>,
        #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
        delay: Option<f64>,
    }
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum SampleInput {
    File {
        /// Explicit original audio input, relative to the owning module.
        source: String,
        #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "Hz"))]
        reference: Option<f64>,
        looping: Option<bool>,
    },
    /// Mono PCM, before pitch and envelope processing.
    Pcm {
        #[schemars(length(min = 1))]
        samples: Vec<Pcm>,
        #[schemars(range(min = 8000, max = 192000), extend("x-og-unit" = "Hz"))]
        rate: u32,
        #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "Hz"))]
        reference: Option<f64>,
        looping: Option<bool>,
    },
}

input! {
    /// Ordered instance transforms. Array counts, nonzero quaternions and
    /// terrain surface/holes are checked when the scatter is expanded.
    ScatterInput {
        target: ScatterTarget,
        #[schemars(length(max = 3_000_000), extend("x-og-unit" = "m"))]
        positions: Vec<f32>,
        /// One xyzw quaternion per position; normalized by domain expansion.
        #[schemars(length(max = 4_000_000))]
        quaternions: Option<Vec<f32>>,
        #[schemars(length(max = 3_000_000))]
        scales: Option<Vec<f32>>,
        #[schemars(extend("x-og-ref-kinds" = ["terrain"]))]
        project: Option<ContentRef>,
        /// Rotate up to the projected normal before the authored quaternion.
        align: Option<bool>,
    }
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ScatterTarget {
    Model {
        #[schemars(extend("x-og-ref-kinds" = ["model"]))]
        asset: ContentRef,
    },
    Prefab {
        #[schemars(extend("x-og-ref-kinds" = ["prefab"]))]
        asset: ContentRef,
    },
}

/// A scalar constant, per-instance range or ordered lifetime curve.
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum EffectValue {
    Constant(f32),
    Variable(ValueInput),
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ValueInput {
    Random {
        min: f32,
        max: f32,
    },
    Curve {
        #[schemars(length(min = 1, max = 8))]
        keys: Vec<[f32; 2]>,
    },
}
words!(Channel {
    Rgba,
    Red,
    Green,
    Blue,
    Alpha
});
words!(Blend {
    Alpha,
    Additive,
    Multiply,
    Subtractive
});
input! {
    /// UV transform evaluated over the surface's life.
    UvInput {
        tile: Option<[f32; 2]>,
        offset: Option<[f32; 2]>,
        #[schemars(extend("x-og-unit" = "uv/s"))]
        scroll: Option<[f32; 2]>,
        #[schemars(extend("x-og-unit" = "degrees/s"))]
        rotate: Option<f32>,
        polar: Option<bool>,
    }
}
input! {
    EffectTextureSlot {
        #[schemars(extend("x-og-ref-kinds" = ["texture"]))]
        texture: Option<ContentRef>,
        channel: Option<Channel>,
        uv: Option<UvInput>,
    }
}
input! {
    ErosionEdge { width: f32, color: [f32; 4] }
}
input! {
    FlowInput { strength: Option<f32>, frequency: Option<f32> }
}
input! {
    FresnelInput { power: f32, strength: Option<f32>, invert: Option<bool> }
}
input! {
    /// Effect material input, not text containing another author language.
    EffectInput {
        main: Option<EffectTextureSlot>,
        #[schemars(length(min = 1, max = 8))]
        ramp: Option<Vec<(f32, [f32; 4])>>,
        mask: Option<EffectTextureSlot>,
        noise: Option<EffectTextureSlot>,
        erosion: Option<EffectValue>,
        #[schemars(range(min = 0))]
        erosion_softness: Option<f32>,
        erosion_edge: Option<ErosionEdge>,
        flow_map: Option<EffectTextureSlot>,
        flow: Option<FlowInput>,
        fresnel: Option<FresnelInput>,
        #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
        soft_depth: Option<f32>,
        color: Option<[f32; 4]>,
        intensity: Option<EffectValue>,
        alpha: Option<EffectValue>,
        offset_u: Option<EffectValue>,
        offset_v: Option<EffectValue>,
        #[schemars(length(max = 8))]
        custom: Option<Vec<EffectValue>>,
        blend: Option<Blend>,
        double_sided: Option<bool>,
        #[schemars(extend("x-og-ref-kinds" = ["shader"]))]
        shader: Option<ContentRef>,
        params: Option<BTreeMap<String, ShaderParam>>,
    }
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(transparent)]
pub struct ShaderParam(#[schemars(length(min = 1, max = 4))] pub Vec<f32>);

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(transparent)]
pub struct Curve(#[schemars(length(min = 1, max = 8))] pub Vec<[f32; 2]>);
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(transparent)]
pub struct ColorCurve(#[schemars(length(min = 1, max = 8))] pub Vec<(f32, [f32; 4])>);

input! {
    /// Emitter declaration. Rendering-mode relationships, curve ordering and
    /// child-emitter cycles require the existing domain/graph validators.
    ParticlesInput {
        #[schemars(range(min = 1, max = 1_048_576))]
        max_particles: Option<u32>,
        shape: Option<EmitterShape>,
        #[schemars(extend("x-og-unit" = "degrees"))]
        spread: Option<f32>,
        #[schemars(range(min = 0), extend("x-og-unit" = "particles/s"))]
        rate: Option<f32>,
        bursts: Option<Vec<Burst>>,
        #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "s"))]
        duration: Option<f32>,
        looping: Option<bool>,
        #[schemars(extend("x-og-unit" = "s"))]
        lifetime: Option<[f32; 2]>,
        #[schemars(extend("x-og-unit" = "m/s"))]
        speed: Option<[f32; 2]>,
        #[schemars(extend("x-og-unit" = "m"))]
        size: Option<[f32; 2]>,
        size_over_life: Option<Curve>,
        color_over_life: Option<ColorCurve>,
        random_color_over_life: Option<ColorCurve>,
        color: Option<[f32; 4]>,
        #[schemars(extend("x-og-unit" = "degrees"))]
        rotation: Option<[f32; 2]>,
        #[schemars(extend("x-og-unit" = "degrees/s"))]
        angular_velocity: Option<[f32; 2]>,
        #[schemars(extend("x-og-unit" = "m/s²"))]
        gravity: Option<[f32; 3]>,
        #[schemars(range(min = 0))]
        drag: Option<f32>,
        #[schemars(range(min = 0, max = 1))]
        wind: Option<f32>,
        #[schemars(range(min = 0, max = 1))]
        force_fields: Option<f32>,
        noise: Option<ParticleNoise>,
        #[schemars(extend("x-og-ref-kinds" = ["texture"]))]
        texture: Option<ContentRef>,
        sheet: Option<Sheet>,
        array: Option<ParticleArray>,
        blend: Option<Blend>,
        space: Option<Space>,
        align: Option<Align>,
        #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
        soft_distance: Option<f32>,
        collision: Option<Collision>,
        render: Option<ParticleRender>,
        lit: Option<bool>,
        #[schemars(range(min = 0, max = 1))]
        roughness: Option<f32>,
        #[schemars(range(min = 0, max = 1))]
        distortion: Option<f32>,
        events: Option<Vec<ParticleEvent>>,
        #[schemars(length(max = 4))]
        sub_emitters: Option<Vec<SubEmitter>>,
        #[schemars(range(min = 0))]
        glow: Option<f32>,
        round: Option<bool>,
        #[schemars(extend("x-og-ref-kinds" = ["effect"]))]
        effect: Option<ContentRef>,
        height_over_life: Option<Curve>,
        speed_over_life: Option<Curve>,
        spin_over_life: Option<Curve>,
        rate_over_time: Option<Curve>,
        trail: Option<ParticleTrail>,
        inherit_velocity: Option<f32>,
        #[schemars(length(max = 4))]
        attractors: Option<Vec<Attractor>>,
        field: Option<ParticleField>,
    }
}
words!(Space { World, Local });
words!(ParticleEvent {
    Birth,
    Death,
    Collision
});
words!(Trigger {
    Birth,
    Death,
    Collision,
    Life
});
words!(CollisionTarget { Depth, Colliders });
words!(ParticleMesh {
    Box,
    Sphere,
    Tetrahedron
});
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum MeshInput {
    Builtin(ParticleMesh),
    Model(#[schemars(extend("x-og-ref-kinds" = ["model"]))] ContentRef),
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum EmitterShape {
    Point,
    Sphere {
        radius: f32,
        surface: Option<bool>,
    },
    Cone {
        angle_degrees: f32,
        radius: f32,
    },
    Box {
        half_extents: [f32; 3],
    },
    Spline {
        #[schemars(length(min = 2, max = 8), extend("x-og-unit" = "m"))]
        points: Vec<[f32; 3]>,
        closed: Option<bool>,
    },
    Mesh {
        mesh: MeshInput,
    },
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ParticleRender {
    Billboard,
    Sprite { layer: Option<i32> },
    Mesh { mesh: MeshInput },
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Align {
    Camera,
    Velocity { stretch: Option<f32> },
    Upright { axis: [f32; 3] },
    Facing { normal: [f32; 3] },
}
input! {
    Burst {
        #[schemars(extend("x-og-unit" = "s"))]
        time: f32,
        count: u32,
        cycles: Option<u32>,
        #[schemars(extend("x-og-unit" = "s"))]
        interval: Option<f32>,
    }
}
input! { ParticleNoise { strength: f32, frequency: Option<f32>, scroll: Option<f32> } }
input! {
    Sheet {
        #[schemars(range(min = 1))]
        columns: u32,
        #[schemars(range(min = 1))]
        rows: u32,
        frames: Option<u32>,
        mode: Option<SheetMode>,
        blend: Option<bool>,
    }
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum SheetMode {
    Off,
    Random,
    Life { cycles: Option<f32> },
    Fps { fps: f32 },
}
input! {
    ParticleArray {
        #[schemars(extend("x-og-ref-kinds" = ["texture-array"]))]
        asset: ContentRef,
        #[schemars(range(min = 1))]
        layers: Option<u32>,
        mode: Option<SheetMode>,
    }
}
input! {
    Collision { bounce: Option<f32>, friction: Option<f32>, against: Option<CollisionTarget>, die: Option<bool> }
}
input! {
    SubEmitter {
        trigger: Trigger,
        #[schemars(extend("x-og-ref-kinds" = ["particles"]))]
        emitter: ContentRef,
        /// Required for event triggers, omitted for life emission.
        #[schemars(range(min = 1, max = 4096))]
        count: Option<u32>,
        inherit_velocity: Option<f32>,
    }
}
input! {
    ParticleTrail {
        #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "s"))]
        lifetime: f32,
        #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
        width: Option<f32>,
        head: Option<bool>,
    }
}
input! {
    Attractor {
        #[schemars(extend("x-og-unit" = "m"))]
        position: [f32; 3],
        strength: f32,
        #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
        radius: Option<f32>,
    }
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ParticleField {
    Curl {
        strength: f32,
        frequency: Option<f32>,
        scroll: Option<f32>,
    },
    Asset {
        #[schemars(extend("x-og-ref-kinds" = ["field", "texture"]))]
        asset: ContentRef,
        strength: f32,
        #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "m"))]
        size: f32,
    },
    Grid {
        #[schemars(range(min = 1, max = 16))]
        cells: u32,
        #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "m"))]
        size: f32,
        strength: f32,
        #[schemars(length(max = 4096))]
        vectors: Vec<[f32; 3]>,
    },
}

input! {
    /// A composed effect timeline. Element end times, parameter identities and
    /// duplicate event names are checked by the domain validator.
    VfxInput {
        #[schemars(extend("exclusiveMinimum" = 0, "x-og-unit" = "s"))]
        duration: f32,
        looping: Option<bool>,
        #[schemars(length(max = 16))]
        events: Option<Vec<VfxEvent>>,
        #[schemars(length(max = 4))]
        params: Option<Vec<VfxParam>>,
        #[schemars(length(max = 64))]
        elements: Vec<VfxElement>,
    }
}
input! {
    VfxEvent {
        #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
        time: f32,
        /// At most 63 UTF-8 bytes (domain validation, not character count).
        name: String,
    }
}
input! { VfxParam { name: String, default: f32 } }
input! {
    VfxElement {
        #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
        at: Option<f32>,
        #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
        life: Option<f32>,
        /// End time instead of life; both together are a domain error.
        #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
        until: Option<f32>,
        #[schemars(extend("x-og-unit" = "m"))]
        offset: Option<[f32; 3]>,
        #[schemars(extend("x-og-unit" = "degrees"))]
        rotate: Option<[f32; 3]>,
        #[schemars(extend("x-og-unit" = "m/s"))]
        velocity: Option<[f32; 3]>,
        #[schemars(extend("x-og-unit" = "m/s²"))]
        acceleration: Option<[f32; 3]>,
        when: Option<String>,
        amount: Option<String>,
        node: VfxNode,
    }
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum VfxNode {
    Emitter {
        #[schemars(extend("x-og-ref-kinds" = ["particles"]))]
        asset: ContentRef,
        seed: Option<u32>,
    },
    Mesh {
        shape: VfxShape,
        #[schemars(extend("x-og-ref-kinds" = ["effect"]))]
        effect: ContentRef,
        scale: Option<EffectValue>,
        #[schemars(extend("x-og-unit" = "degrees/s"))]
        spin: Option<[f32; 3]>,
        color: Option<[f32; 4]>,
        blend: Option<Blend>,
        face_camera: Option<bool>,
        stretch: Option<String>,
    },
    Trail {
        #[schemars(extend("x-og-ref-kinds" = ["effect"]))]
        effect: Option<ContentRef>,
        #[schemars(extend("x-og-unit" = "m"))]
        width: Option<[f32; 2]>,
        #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
        lifetime: Option<f32>,
        color: Option<[f32; 4]>,
        /// Radius in metres and turns per second about local Y.
        orbit: Option<[f32; 2]>,
        #[schemars(extend("x-og-unit" = "m/s"))]
        rise: Option<f32>,
    },
    Beam {
        #[schemars(extend("x-og-ref-kinds" = ["effect"]))]
        effect: Option<ContentRef>,
        #[schemars(extend("x-og-unit" = "m"))]
        to: Option<[f32; 3]>,
        #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
        width: Option<f32>,
        color: Option<[f32; 4]>,
        #[schemars(range(min = 1, max = 256))]
        segments: Option<u32>,
        #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
        jitter: Option<f32>,
        stretch: Option<String>,
    },
    Light {
        color: Option<[f32; 3]>,
        intensity: Option<EffectValue>,
        #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
        distance: Option<f32>,
    },
    Decal {
        /// Region of this decal's texture and normal, not a global atlas.
        uv: Option<[f32; 4]>,
        #[schemars(extend("x-og-unit" = "m"))]
        size: Option<[f32; 3]>,
        #[schemars(range(min = 0), extend("x-og-unit" = "s"))]
        fade: Option<f32>,
        color: Option<[f32; 4]>,
        /// Omitted means a solid-colour decal.
        #[schemars(extend("x-og-ref-kinds" = ["texture"]))]
        texture: Option<ContentRef>,
        /// Tangent-space glTF normal map: +X along U, +Y toward image top.
        /// Shares the colour texture's UV region; omitted leaves normals alone.
        #[schemars(extend("x-og-ref-kinds" = ["texture"]))]
        normal: Option<ContentRef>,
        #[schemars(range(min = 0, max = 4))]
        normal_scale: Option<f32>,
        /// Linear emission; HDR values above one are allowed.
        #[schemars(extend("items" = {"type": "number", "format": "float", "minimum": 0}))]
        emissive: Option<[f32; 3]>,
        /// Omitted preserves the receiving surface's roughness.
        #[schemars(range(min = 0, max = 1))]
        roughness: Option<f32>,
        /// Omitted preserves the receiving surface's metallic value.
        #[schemars(range(min = 0, max = 1))]
        metallic: Option<f32>,
        wet: Option<bool>,
        #[schemars(range(min = 0, max = 90), extend("x-og-unit" = "degrees"))]
        angle_fade: Option<f32>,
        /// Seeded per-instance rotation interval about the projection axis.
        #[schemars(extend("x-og-unit" = "degrees"))]
        turn: Option<[f32; 2]>,
    },
    Glow {
        size: Option<EffectValue>,
        color: Option<[f32; 4]>,
        stretch: Option<String>,
    },
    Shake {
        #[schemars(range(min = 0, max = 1))]
        trauma: Option<f32>,
    },
    Sound {
        #[schemars(extend("x-og-ref-kinds" = ["sound"]))]
        asset: ContentRef,
        #[schemars(range(min = 0))]
        volume: Option<f32>,
        /// Playback-rate interval; 1 is the recording's original pitch.
        pitch: Option<[f32; 2]>,
    },
    Flipbook {
        #[schemars(extend("x-og-ref-kinds" = ["flipbook"]))]
        asset: ContentRef,
        #[schemars(range(min = 0), extend("x-og-unit" = "m"))]
        size: Option<f32>,
        color: Option<[f32; 4]>,
        blend: Option<SpriteBlend>,
    },
}
words!(SpriteBlend { Alpha, Additive });
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum VfxShape {
    Sphere,
    Quad,
    Box,
    Cylinder,
    Ring {
        ring: Option<RingInput>,
    },
    Model {
        #[schemars(extend("x-og-ref-kinds" = ["model"]))]
        asset: ContentRef,
    },
}
input! {
    RingInput {
        #[schemars(range(min = 0, max = 16))]
        inner: Option<f32>,
        #[schemars(range(min = 0, max = 1))]
        band: Option<f32>,
        height: Option<[f32; 2]>,
        #[schemars(extend("x-og-unit" = "degrees"))]
        start: Option<f32>,
        #[schemars(extend("x-og-unit" = "degrees"))]
        length: Option<f32>,
        #[schemars(range(min = 3, max = 256))]
        segments: Option<u32>,
        colors: Option<[[f32; 4]; 3]>,
    }
}

macro_rules! record {
    ($($marker:ident => $record:literal $(, [$($exclude:literal),*])?);* $(;)?) => {
        $(
            #[derive(Debug)]
            pub struct $marker;
            impl super::reflected::RecordMarker for $marker {
                const NAME: &'static str = $record;
                $(const EXCLUDED: &'static [&'static str] = &[$($exclude),*];)?
            }
        )*
    };
}
record! {
    RenderSettings => "RenderSettings";
    PostEffects => "PostEffects", ["flash", "shakeTrauma"];
    Stylise => "Stylise";
    MaterialStyle => "MaterialStyle";
    Fog => "Fog";
    Sky => "Sky";
    Environment => "Environment";
    ArtPalette => "ArtPalette";
    ArtPixels => "ArtPixels";
    ArtTexels => "ArtTexels";
    ArtShape => "ArtShape";
    ArtLine => "ArtLine";
    ArtMotion => "ArtMotion";
    ArtCamera => "ArtCamera";
}
use super::reflected::Reflected;
input! {
    /// A look overlays reflected resource patches on explicitly named bases.
    /// Inheritance order/cycles, preset variants and asset identities are domain checks.
    LookInput {
        name: Option<String>,
        extends: Option<Vec<LookBase>>,
        mood: Option<Mood>,
        #[schemars(regex(pattern = "^[a-z0-9]+(-[a-z0-9]+)*$"))]
        variant: Option<String>,
        ui: Option<UiStyle>,
        ui_seeds: Option<UiSeeds>,
        antialias: Option<Antialias>,
        #[serde(rename = "RenderSettings")]
        render_settings: Option<Reflected<RenderSettings>>,
        #[serde(rename = "PostEffects")]
        post_effects: Option<Reflected<PostEffects>>,
        #[serde(rename = "Stylise")]
        stylise: Option<Reflected<Stylise>>,
        #[serde(rename = "MaterialStyle")]
        material_style: Option<Reflected<MaterialStyle>>,
        #[serde(rename = "Fog")]
        fog: Option<Reflected<Fog>>,
        #[serde(rename = "Sky")]
        sky: Option<Reflected<Sky>>,
        #[serde(rename = "Environment")]
        environment: Option<Reflected<Environment>>,
        #[serde(rename = "ArtPalette")]
        art_palette: Option<Reflected<ArtPalette>>,
        #[serde(rename = "ArtPixels")]
        art_pixels: Option<Reflected<ArtPixels>>,
        #[serde(rename = "ArtTexels")]
        art_texels: Option<Reflected<ArtTexels>>,
        #[serde(rename = "ArtShape")]
        art_shape: Option<Reflected<ArtShape>>,
        #[serde(rename = "ArtLine")]
        art_line: Option<Reflected<ArtLine>>,
        #[serde(rename = "ArtMotion")]
        art_motion: Option<Reflected<ArtMotion>>,
        #[serde(rename = "ArtCamera")]
        art_camera: Option<Reflected<ArtCamera>>,
    }
}
words!(Mood {
    Bright,
    Dark,
    Warm,
    Cold,
    Soft,
    HighContrast
});
words!(Antialias {
    Tier,
    Off,
    Msaa4,
    Taa
});
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum LookBase {
    Preset(String),
    Asset(#[schemars(extend("x-og-ref-kinds" = ["look"]))] ContentRef),
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
pub enum UiStyle {
    Builtin(String),
    Asset(#[schemars(extend("x-og-ref-kinds" = ["pack"]))] ContentRef),
}
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(transparent)]
pub struct HexColor(#[schemars(regex(pattern = "^#[0-9a-fA-F]{6}$"))] pub String);
input! {
    UiSeeds {
        primary: Option<HexColor>, accent: Option<HexColor>, neutral: Option<HexColor>,
        danger: Option<HexColor>, warn: Option<HexColor>, success: Option<HexColor>,
        info: Option<HexColor>, team: Option<HexColor>,
    }
}

pub(super) const KINDS: &[Kind] = &[
    Kind {
        name: "effect-texture",
        schema: || EffectTextureInput::schema(),
    },
    Kind {
        name: "sfx",
        schema: || SfxInput::schema(),
    },
    Kind {
        name: "scatter",
        schema: || ScatterInput::schema(),
    },
    Kind {
        name: "effect",
        schema: || EffectInput::schema(),
    },
    Kind {
        name: "particles",
        schema: || ParticlesInput::schema(),
    },
    Kind {
        name: "vfx",
        schema: || VfxInput::schema(),
    },
    Kind {
        name: "look",
        schema: || LookInput::schema(),
    },
    Kind {
        name: "terrain",
        schema: || TerrainInput::schema(),
    },
    Kind {
        name: "theme",
        schema: || ThemeInput::schema(),
    },
];

#[cfg(test)]
#[path = "procedural/tests.rs"]
mod tests;
