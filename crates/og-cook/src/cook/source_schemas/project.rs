//! Project/module inputs are bootstrap data, never content factories or games.
//! Their consumers switch together in the project frontend cutover.
use super::ContentRef;
use og_core::reflect::source::{InputSchema, schemars::JsonSchema};
use serde::Deserialize;
use std::{collections::BTreeMap, sync::OnceLock};

macro_rules! input {
    ($(#[$doc:meta])* $name:ident { $($(#[$attr:meta])* $field:ident: $ty:ty),* $(,)? }) => {
        $(#[$doc])*
        #[derive(Debug, Deserialize, JsonSchema)]
        #[schemars(crate = "og_core::reflect::source::schemars")]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        pub struct $name { $($(#[$attr])* pub $field: $ty,)* }
        impl $name {
            pub fn schema() -> &'static InputSchema<Self> {
                static SCHEMA: OnceLock<InputSchema<$name>> = OnceLock::new();
                SCHEMA.get_or_init(|| InputSchema::new().expect(stringify!($name)))
            }
        }
    };
}
macro_rules! words {
    ($name:ident { $($variant:ident => $word:literal),* $(,)? }) => {
        #[derive(Debug, Deserialize, JsonSchema)]
        #[schemars(crate = "og_core::reflect::source::schemars")]
        pub enum $name { $(#[serde(rename = $word)] $variant,)* }
    };
}

input!(
    /// One root manifest for small and large projects. Omitted simulation
    /// settings remain the game's responsibility, not inferred cook choices.
    ProjectInput {
        #[schemars(regex(pattern = "^[a-z0-9]+(-[a-z0-9]+)*$"))]
        id: String,
        /// Project-relative TypeScript game entry; bootstrap never executes it.
        #[schemars(length(min = 1))]
        entry: String,
        #[schemars(range(min = 1, max = 240))]
        tick: Option<u32>,
        seed: Option<u32>,
        #[schemars(range(min = 1))]
        save_version: Option<u32>,
        #[serde(default)]
        modules: Vec<ModuleInput>,
        #[serde(default)]
        packs: Vec<DeliveryPack>,
        /// Direct dependencies of the implicit base source module.
        #[serde(default)]
        depends_on: Vec<String>,
        /// Explicit input globs for the base module, relative to its root.
        #[serde(default)]
        inputs: Vec<String>,
        /// Pure record declaration files, never a running game entry.
        #[serde(default)]
        schemas: Vec<String>,
        #[serde(default)]
        roots: Vec<ContentRef>,
        #[serde(default)]
        #[schemars(extend("additionalProperties" = {"$ref":"#/$defs/ContentRef","x-og-ref-kinds":["scene"]}))]
        scenes: BTreeMap<String, ContentRef>,
        scene: Option<String>,
        flow: Option<ProjectFlow>,
        create: Option<Creation>,
        cook: Option<ProjectCook>,
        publish: Option<Publish>,
        /// Named median-time limits in milliseconds, not cook quality choices.
        #[serde(default)]
        #[schemars(extend("additionalProperties" = {"type":"number","minimum":0}))]
        perf: BTreeMap<String, f64>,
        budgets: Option<Budgets>,
        conform: Option<Conform>,
        /// Named command scripts used by UI and style capture tools.
        #[serde(default)]
        #[schemars(extend("additionalProperties" = {"type":"string","minLength":1}))]
        screens: BTreeMap<String, String>,
        /// Pure Yarn expression functions, checked before dialogue compilation.
        #[serde(default)]
        story_functions: BTreeMap<String, StoryFunction>,
    }
);

input!(StoryFunction {
    returns: StoryType,
    /// Declaration order is the function's argument order.
    #[serde(default)]
    parameters: Vec<StoryParameter>,
    body: String,
});
input!(StoryParameter {
    name: String,
    kind: StoryType
});
words!(StoryType { Number => "number", String => "string", Bool => "bool" });

input!(
    /// A namespace and source root, independent of delivery and streaming.
    #[derive(Clone)]
    ModuleInput {
        #[schemars(regex(pattern = "^[a-z0-9]+(-[a-z0-9]+)*$"))]
        id: String,
        /// Canonical project-relative directory. Moving it preserves IDs.
        #[schemars(length(min = 1))]
        root: String,
        #[serde(default)]
        depends_on: Vec<String>,
        #[serde(default)]
        inputs: Vec<String>,
    }
);

input!(
    /// An output delivery unit, not a module, stream group or world cell.
    #[derive(Clone)]
    DeliveryPack {
        #[schemars(regex(pattern = "^[a-z0-9]+(-[a-z0-9]+)*$"))]
        id: String,
        /// Namespaced content patterns. Overlap is an error, never first-wins.
        #[serde(default)]
        include: Vec<String>,
        #[serde(default)]
        roots: Vec<ContentRef>,
        #[serde(default)]
        depends_on: Vec<String>,
        #[serde(default)]
        optional: bool,
    }
);

input!(ProjectFlow {
    #[schemars(extend("x-og-ref-kinds" = ["dialogue"]))]
    source: ContentRef,
    screen: Option<FlowScreen>,
});
words!(FlowScreen { Engine => "engine", Game => "game" });

input!(
    /// Creation metadata describes the chosen shell. No genre composition or
    /// balancing is selected by bootstrap; open catalogue IDs remain strings.
    Creation {
        form: Option<Form>, genre: Option<String>, view: Option<String>,
        look: Option<String>, variant: Option<String>, ui: Option<String>,
        platform: Option<Platform>, input: Option<Controls>, players: Option<Players>,
        #[schemars(length(min = 1))]
        lang: Option<Vec<String>>,
        music: Option<String>,
    }
);
words!(Form { TwoD => "2d", TwoHalfD => "2.5d", ThreeD => "3d" });
words!(Platform {
    PhonePortrait => "phone-portrait", PhoneLandscape => "phone-landscape",
    Desktop => "desktop", Multi => "multi"
});
words!(Controls { Touch => "touch", KeyboardMouse => "keyboard-mouse", Gamepad => "gamepad" });
words!(Players { Single => "single", Local => "local-multiplayer", Online => "online-lockstep" });

input!(ProjectCook {
    color_textures: Option<ColorEncoding>,
    clusters: Option<bool>,
    /// Explicit prebuilt bundle inputs. Names must remain globally unique.
    #[serde(default)]
    uses: Vec<String>,
});
words!(ColorEncoding { Etc1s => "etc1s", Uastc => "uastc" });

input!(Publish {
    out: Option<String>,
    #[serde(default)]
    keep: Vec<String>,
    color_textures: Option<ColorEncoding>,
    probe: Option<Probe>, isolation: Option<Isolation>, fit: Option<Fit>,
    aspect: Option<String>, aspect_min: Option<String>, aspect_max: Option<String>,
    orientation: Option<Orientation>,
    #[schemars(range(min = 0.5, max = 4))]
    dpr_cap: Option<f64>,
    fullscreen: Option<Fullscreen>, gate: Option<Gate>,
    made_with: Option<bool>, icon: Option<String>, short_name: Option<String>,
    telemetry: Option<Telemetry>, pipelines: Option<Pipelines>,
    loading: Option<Loading>, loading_labels: Option<LoadingLabels>,
    assets: Option<String>, content: Option<String>,
});
words!(Probe { Player => "player", All => "all" });
words!(Isolation { Required => "required", Preferred => "preferred", Off => "off" });
words!(Fit { Fill => "fill", Letterbox => "letterbox" });
words!(Orientation { Any => "any", Landscape => "landscape", Portrait => "portrait" });
words!(Fullscreen { Off => "off", Gesture => "gesture" });
words!(Gate { Tap => "tap", None => "none" });
words!(Telemetry { On => "on", Off => "off" });
words!(Pipelines { Auto => "auto", Sync => "sync", Async => "async", Prewarm => "prewarm" });
words!(Loading { Engine => "engine", Game => "game", Page => "page" });
input!(LoadingLabels {
    fetch: Option<String>, transcode: Option<String>, prime: Option<String>, present: Option<String>,
});

input!(
    /// Explicit overrides of the existing content-report limits. Absence leaves
    /// that limit's interpretation with the report, not the author evaluator.
    Budgets {
        triangles: Option<u64>, vertices: Option<u64>, texture_size: Option<u64>,
        bones: Option<u64>, clips: Option<u64>, asset_bytes: Option<u64>,
        total_bytes: Option<u64>, sound_seconds: Option<u64>, video_seconds: Option<u64>,
        nav_polygons: Option<u64>, scene_entities: Option<u64>, emitter_particles: Option<u64>,
        bvh_share: Option<u64>, gpu_bytes: Option<u64>,
    }
);
input!(Conform {
    #[serde(default)]
    off: Vec<String>,
});
