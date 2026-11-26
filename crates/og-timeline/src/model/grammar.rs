//! File fields used by the shared parser and generated file reference.
use og_core::ogt::{Block, line::Line};

macro_rules! rows {
    ($($in:expr, $k:literal, $a:literal, $d:literal, $doc:literal);* $(;)?) => {
        &[$(Line::of($in, $k, $a, $d, $doc, "")),*]
    };
}
pub const TIMELINE_LINES: &[Line] = rows![
    &[], "name", "\"<name>\"", "", "Timeline name.";
    &[], "duration", "<seconds > 0>", "last key end", "No key may lie after the duration.";
    &[], "path", "\"<name>\" { ... }", "", "Named scene-local path.";
    &[], "track", "<kind> [\"<entity>\"] { ... }", "", "Repeated tracks; transform/animation/light/visibility/blendshape/material require an entity, property/video allow one, other kinds take none."
];
pub const PATH_LINES: &[Line] = rows![
    &[], "kind", "catmullRom | bezier | linear", "catmullRom", "Spline kind.";
    &[], "closed", "true | false", "false", "Join the last point to the first.";
    &[], "tension", "<number>", "0.5", "Catmull-Rom tangent scale.";
    &[], "point", "<x> <y> <z>", "required", "Repeated local points, metres; point count must satisfy the spline kind."
];
pub const TRACK_LINES: &[Line] = rows![
    &[], "key", "<seconds >= 0> { ... }", "", "Repeated keys, sorted stably by time; nested fields depend on track kind.";
    &["camera"], "rail", "\"<path>\"", "", "Follow a named path, exclusive with follow and key positions.";
    &["camera"], "follow", "\"<entity>\"", "", "Follow an entity, exclusive with rail.";
    &["camera"], "blendIn", "<seconds >= 0>", "0", "Entry blend duration.";
    &["camera"], "blendOut", "<seconds >= 0>", "0", "Exit blend duration.";
    &["animation"], "model", "\"<asset>\"", "", "Explicit clip source, checked at cook.";
    &["caption"], "captions", "\"<file.srt|vtt|json>\" [<offset seconds>]", "", "Import cues; files enter cook dependencies.";
    &["caption"], "locale", "\"<tag>\" { captions ...; key ... }", "", "Localized cues, no nested locale.";
    &["property"], "field", "\"<record.field>\"", "required", "Reflected target field, declared before keys.";
    &["material"], "index", "<integer >= 0>", "0", "Target material index.";
    &["video"], "layer", "over | under", "over", "Only for the timeline's own player, not entity players."
];
pub const KEY_LINES: &[Line] = rows![
    &["camera", "transform", "vfx"], "position", "<x> <y> <z>", "context-dependent", "Local metres; camera/transform keep unspecified values, vfx defaults to origin.";
    &["camera"], "target", "<x> <y> <z>", "", "Look-at target, metres.";
    &["camera"], "at", "<number>", "", "Rail parameter.";
    &["camera"], "fov", "<1..179 degrees>", "", "Vertical field of view.";
    &["camera"], "roll", "<degrees>", "", "Camera roll.";
    &["camera"], "focus", "<metres > 0>", "", "Focus distance.";
    &["camera"], "focusOn", "\"<entity>\"", "", "Focus target.";
    &["camera"], "focusRange", "<metres >= 0>", "", "Focus range.";
    &["camera", "transform", "property", "light", "visibility", "post", "blendshape", "material", "shot", "hotspot"], "ease", "<word or quoted easing expression>", "linear", "Shared og-tween easing parser, including cubicBezier, steps and points.";
    &["transform"], "rotation", "<x> <y> <z>", "", "Euler XYZ degrees.";
    &["transform"], "quaternion", "<x> <y> <z> <w>", "", "Nonzero quaternion, normalized.";
    &["transform"], "scale", "<x> <y> <z>", "", "Scale factors.";
    &["animation"], "clip", "\"<name>\"", "required", "Authored clip name.";
    &["animation", "timeline", "video"], "speed", "<number>", "1", "Animation speed multiplier; nested timeline >0; video 0..4.";
    &["animation"], "blendIn", "<seconds >= 0>", "0.2", "Clip transition duration.";
    &["animation", "music", "timeline"], "loop", "true | false", "context-dependent", "Animation/music default true; nested timeline false.";
    &["audio", "music"], "sound", "\"<asset>\"", "required", "Cooked sound asset.";
    &["audio", "music"], "volume", "<number >= 0>", "1", "Linear gain.";
    &["audio"], "playbackRate", "<number > 0>", "1", "Playback rate multiplier.";
    &["audio", "music"], "bus", "master | music | sfx | ui", "context-dependent", "Audio defaults sfx; music defaults music.";
    &["music"], "fadeIn", "<seconds >= 0>", "0", "Fade in duration.";
    &["music"], "fadeOut", "<seconds >= 0>", "0", "Fade out duration.";
    &["music", "caption"], "end", "<seconds > key time>", "context-dependent", "Absolute end; caption can derive it from following cues.";
    &["caption"], "text", "\"<text>\"", "required", "Caption text, not visual styling.";
    &["caption"], "speaker", "\"<speaker>\"", "", "Caption speaker.";
    &["caption"], "position", "bottom | top", "", "Caption placement.";
    &["caption"], "span", "<name> <position> <length> [\"<color>\"]", "", "Repeated spans with unsigned character indices.";
    &["event"], "do", "\"<action:arguments>\"", "required", "Command-channel action; action uses lowercase ASCII, digits and underscores.";
    &["event"], "retroactive", "true | false", "false", "Fire across a seek as defined by timeline playback.";
    &["transition"], "kind", "<transition kind>", "fade", "Shared TransitionKind names.";
    &["transition"], "mode", "through | in | out", "through", "Transition mode.";
    &["transition"], "duration", "<seconds >= 0>", "1", "Transition duration.";
    &["transition"], "color", "<r> <g> <b>", "0 0 0", "Linear transition colour.";
    &["transition"], "from", "left | right | top | bottom", "left", "Transition origin.";
    &["marker"], "name", "\"<1..63 byte name>\"", "required", "Named timeline marker.";
    &["property"], "value", "<1..4 numbers or word>", "required", "Value of the declared reflected field.";
    &["visibility"], "visible", "true | false", "", "Entity visibility.";
    &["blendshape"], "w0..w7", "<number>", "", "Morph weight by index; unspecified weights keep their values.";
    &["timeline"], "play", "\"<timeline asset>\"", "required", "Nested timeline; cook checks cycles and references.";
    &["vfx"], "effect", "\"<asset>\"", "required", "Effect asset.";
    &["vfx"], "at", "\"<entity>\"", "", "Effect anchor entity.";
    &["vfx"], "scale", "<number > 0>", "1", "Uniform effect size.";
    &["vfx"], "seed", "<u32>", "0", "Explicit effect seed.";
    &["video"], "video", "\"<asset>\"", "required", "Video asset.";
    &["video"], "from", "<seconds >= 0>", "0", "Source offset.";
    &["scene"], "scene", "\"<asset>\" | none", "required", "Place a scene or clear it.";
    &["shot"], "camera", "\"<entity>\"", "", "Virtual camera, exclusive in intent with none.";
    &["shot"], "none", "", "", "Release virtual camera control.";
    &["shot"], "blend", "<seconds >= 0>", "camera setting", "Zero cuts immediately.";
    &["hotspot"], "id", "\"<1..63 byte id>\"", "required", "Stable hotspot id.";
    &["hotspot"], "rect", "<x> <y> <width> <height>", "", "Fractions from top-left, positive size within the frame; exclusive with off true.";
    &["hotspot"], "shape", "rect | circle", "rect", "Hit shape.";
    &["hotspot"], "kind", "choice | inspect", "choice", "Hotspot kind.";
    &["hotspot"], "label", "\"<label>\"", "", "Hotspot label.";
    &["hotspot"], "off", "true | false", "false", "End this hotspot instead of providing rect."
];

pub(super) fn check(block: &Block, lines: &[Line], kind: &str) -> Result<(), String> {
    for e in &block.entries {
        if !lines
            .iter()
            .any(|f| f.key == e.key && (f.within.is_empty() || f.within.contains(&kind)))
        {
            return Err(super::at(e, format!("unknown {kind} field '{}'", e.key)));
        }
    }
    Ok(())
}
