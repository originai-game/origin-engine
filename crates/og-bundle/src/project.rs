//! Shared project file vocabulary. Each subsystem still owns value validation.
use og_core::ogt::line::Line;

macro_rules! lines {
    ($($key:literal, $args:literal, $default:literal, $doc:literal);* $(;)?) => {
        &[$(Line::new($key, $args, $default, $doc, "")),*]
    };
}
pub const LINES: &[Line] = lines![
    "name", "\"<name>\"", "Origin Engine game", "Display name.";
    "id", "\"<id>\"", "game", "Project identifier.";
    "engine", "\"<version>\"", "installed engine", "Pinned released engine version.";
    "entry", "\"<path.ts>\"", "src/main.ts", "Game entry relative to project.";
    "tick", "<integer 1..240>", "GameDesc", "Fixed ticks per second; duplicate conflicting GameDesc values fail.";
    "seed", "<u32>", "GameDesc", "Simulation seed.";
    "saveVersion", "<u32 >= 1>", "GameDesc", "Save contract version.";
    "scene", "<name>", "GameDesc", "Initial scene from scenes.";
    "scenes", "{ <name> \"<asset path>\" ... }", "GameDesc", "Named scene assets.";
    "look", "\"<asset path>\"", "none", "Look and cook art contract; see file:look.";
    "create", "{ ... }", "none", "Creation choices retained as project data; form 2d selects the 2D profile, otherwise 3D.";
    "flow", "\"<dialogue.yarn>\" { screen engine | game }", "none", "Dialogue started before GameDesc.start; screen defaults to engine.";
    "cook", "{ ... }", "defaults", "Cook options.";
    "publish", "{ ... }", "defaults", "Web export options, not an instruction to publish.";
    "perf", "{ budget \"<line>\" <milliseconds> ... }", "none", "Per-line median time limits for perf reporting.";
    "budgets", "{ ... }", "cook budget defaults", "Content limits, not generation decisions.";
    "conform", "{ off \"<pattern>\" ... }", "none", "Explicit art-contract exemptions; * matches within a path segment, ** across segments, trailing / a directory."
];
pub const COOK: &[Line] = lines![
    "color_textures", "etc1s | uastc", "etc1s", "Colour texture encoding; publish can override.";
    "clusters", "true | false", "true", "Bake meshlet clusters.";
    "uses", "\"<bundle>\" ...", "none", "Explicit additional bundles the project can reference."
];
pub const PUBLISH: &[Line] = lines![
    "out", "\"<directory>\"", "dist", "Export destination.";
    "keep", "\"<asset pattern>\" ...", "none", "Additional asset roots beyond reachable references.";
    "color_textures", "etc1s | uastc", "cook setting", "Export texture encoding.";
    "probe", "player | all", "export default", "Probe visibility.";
    "isolation", "required | preferred | off", "export default", "Cross-origin isolation policy.";
    "fit", "fill | letterbox", "export default", "Canvas fit.";
    "aspect", "\"<width:height>\"", "none", "Required for letterbox.";
    "aspect_min", "\"<width:height>\"", "none", "Minimum fill aspect ratio.";
    "aspect_max", "\"<width:height>\"", "none", "Maximum fill aspect ratio.";
    "orientation", "any | landscape | portrait", "export default", "Screen orientation.";
    "dpr_cap", "<0.5..4>", "export default", "Maximum device pixel ratio.";
    "fullscreen", "off | gesture", "export default", "Request fullscreen after gesture.";
    "gate", "tap | none", "export default", "Ready-to-interact gate.";
    "made_with", "true | false", "export default", "Engine badge.";
    "icon", "\"<path.png>\"", "brand tile", "Square icon at least 192 pixels.";
    "short_name", "\"<name>\"", "export default", "Installed application name.";
    "telemetry", "on | off", "export default", "Field telemetry.";
    "pipelines", "auto | sync | async | prewarm", "export default", "Web pipeline creation mode.";
    "loading", "engine | game | page", "export default", "Loading-screen owner.";
    "loading_labels", "{ fetch|transcode|prime|present \"<caption>\" ... }", "engine captions", "Per-stage loading captions.";
    "assets", "\"<https base URL>\"", "export directory", "Cooked asset base; localhost HTTP also accepted. No credentials, query or fragment.";
    "content", "\"<https base URL>\"", "export default", "Content base; same URL contract as assets."
];
