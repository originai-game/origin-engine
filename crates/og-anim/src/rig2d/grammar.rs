//! Canonical rig grammar shared by runtime validation and file discovery.
use super::*;
use og_core::ogt::line::Line;
macro_rules! rows {
    ($($k:literal, $a:literal, $d:literal, $doc:literal);* $(;)?) => { &[$(Line::new($k, $a, $d, $doc, "")),*] };
}
pub const LINES: &[Line] = rows![
    "texture", "\"<image>\"", "required", "Shared atlas image.";
    "size", "<width> <height>", "from image", "Atlas pixels.";
    "filter", "nearest | linear", "linear", "Atlas sampling.";
    "unit", "<pixels > 0>", "1", "Pixels per world metre.";
    "bone", "<name> { ... }", "required", "Parent-first setup bones.";
    "slot", "<name> { ... }", "", "Slots in draw order.";
    "region", "<name> { ... }", "", "Rectangular attachment.";
    "mesh", "<name> { ... }", "", "Weighted mesh attachment.";
    "box", "<name> { ... }", "", "Polygon attachment, used for queries.";
    "path", "<name> { ... }", "", "Cubic path attachment, three vertices per knot.";
    "child", "<name> { ... }", "", "Child armature attachment.";
    "ik", "<name> { ... }", "", "IK constraint.";
    "transform", "<name> { ... }", "", "Transform constraint.";
    "along", "<name> { ... }", "", "Path constraint.";
    "skin", "<name> { swap <attachment> <attachment or none> ... }", "", "Named attachment swaps; default is reserved.";
    "armature", "<name> { ... }", "", "Child armature using the same bone/slot/attachment/constraint/clip grammar.";
    "clip", "<name> { ... }", "", "Authored animation keys and events.";
    "dragonbones", "\"<export.json>\"", "", "Cook-only DragonBones 5.5 input; exclusive with authored bones/attachments/clips.";
    "atlas", "\"<atlas.json>\"", "", "Cook-only imported atlas metadata."
];
pub const BLOCKS: &[(&str, &[Line])] = &[
    (
        "bone",
        rows![
            "parent", "<bone>", "", "Parent bone declared earlier.";
            "at", "<x> <y>", "0 0", "Setup translation, pixels, +y up.";
            "rotation", "<degrees>", "0", "Counterclockwise setup rotation.";
            "skew", "<degrees>", "0", "Additional rotation of y axis.";
            "scale", "<x> <y>", "1 1", "Setup scale.";
            "length", "<pixels>", "0", "Bone length."
        ],
    ),
    (
        "slot",
        rows![
            "bone", "<bone>", "required", "Owning bone.";
            "attachment", "<attachment or none>", "none", "Setup display.";
            "color", "<r> <g> <b> <a>", "1 1 1 1", "Linear colour multiplier.";
            "offset", "<r> <g> <b> <a>", "0 0 0 0", "Additive colour offset."
        ],
    ),
    (
        "region",
        rows![
            "slot", "<slot>", "required", "Owning slot.";
            "rect", "<x> <y> <w> <h>", "required", "Atlas rectangle, top-left pixels.";
            "at", "<x> <y>", "0 0", "Attachment translation, pixels.";
            "rotation", "<degrees>", "0", "Counterclockwise rotation.";
            "skew", "<degrees>", "0", "Y-axis skew.";
            "size", "<w> <h>", "rectangle size", "Drawn dimensions, pixels."
        ],
    ),
    (
        "mesh",
        rows![
            "slot", "<slot>", "required", "Owning slot.";
            "vertex", "<u> <v> <x> <y> | <u> <v> <bone> <x> <y> <weight> ...", "", "Unweighted slot-local pixels or repeated bone-local influences; weights sum to one.";
            "triangles", "<index> ...", "", "Nonnegative integer indices, triples."
        ],
    ),
    (
        "box",
        rows![
            "slot", "<slot>", "required", "Owning slot.";
            "polygon", "<x> <y> ...", "", "Polygon vertices, pixels."
        ],
    ),
    (
        "path",
        rows![
            "slot", "<slot>", "required", "Owning slot.";
            "closed", "true | false", "false", "Closed spline.";
            "constant", "true | false", "true", "Measure distance by arc length.";
            "lengths", "<pixels> ...", "", "Curve ends, required if constant is false.";
            "vertex", "<x> <y> | <bone> <x> <y> <weight> ...", "", "Handle-in/knot/handle-out, at least two knots; unweighted or bone influences."
        ],
    ),
    (
        "child",
        rows![
            "slot", "<slot>", "required", "Owning slot.";
            "armature", "<name>", "required", "Child armature in this file.";
            "clip", "<name>", "", "Fallback clip when no inherited name matches.";
            "at", "<x> <y>", "0 0", "Translation, pixels.";
            "rotation", "<degrees>", "0", "Counterclockwise rotation.";
            "skew", "<degrees>", "0", "Y-axis skew.";
            "scale", "<x> <y>", "1 1", "Scale factors."
        ],
    ),
    (
        "ik",
        rows![
            "bones", "<bone> ...", "required", "Contiguous parent-first chain.";
            "target", "<bone>", "required", "Target bone.";
            "bend", "clockwise | counterclockwise", "clockwise", "IK bend side.";
            "weight", "<number>", "1", "Constraint weight, clamped 0..1."
        ],
    ),
    (
        "transform",
        rows![
            "bones", "<bone> ...", "required", "Affected bones.";
            "target", "<bone>", "required", "Target bone.";
            "rotation", "<degrees>", "0", "Rotation offset.";
            "at", "<x> <y>", "0 0", "Translation offset, pixels.";
            "scale", "<x> <y>", "0 0", "Scale offset.";
            "shear", "<degrees>", "0", "Shear offset.";
            "mix", "<rotation> <translation> <scale> <shear>", "1 1 1 1", "Weights clamped 0..1."
        ],
    ),
    (
        "along",
        rows![
            "path", "<attachment>", "required", "Path attachment.";
            "bones", "<bone> ...", "required", "Affected bones.";
            "position", "<value> fixed | percent", "0 percent", "Start position in pixels or path fraction.";
            "spacing", "<value> length | fixed | percent", "0 length", "Bone spacing.";
            "rotate", "tangent | chain | chainScale <degrees>", "tangent 0", "Rotation mode and offset.";
            "mix", "<rotation> <translation>", "1 1", "Weights clamped 0..1."
        ],
    ),
    (
        "skin",
        rows![
            "swap",
            "<attachment> <attachment or none>",
            "",
            "Repeated swaps between attachments belonging to the same slot."
        ],
    ),
    (
        "clip",
        rows![
            "duration", "<seconds>", "0", "Clip duration; keys must be ordered and within duration.";
            "loop", "true | false", "true", "Loop playback.";
            "event", "<seconds> \"<name>\"", "", "Repeated named markers.";
            "bone", "<name> { ... }", "", "Bone timeline.";
            "slot", "<name> { ... }", "", "Slot timeline.";
            "deform", "<attachment> { key ... }", "", "Mesh/path influence offsets.";
            "order", "<seconds> [<slot> ...]", "", "Every slot once in draw order, or empty for setup order.";
            "ik", "<constraint> { key ... }", "", "IK weight and bend timeline."
        ],
    ),
    (
        "clip.bone",
        rows![
            "rotate", "<seconds> <degrees> [clockwise|counterclockwise <turns>] [<ease>]", "", "Relative setup rotation; shortest arc unless explicit nonnegative integer extra turns.";
            "skew", "<seconds> <degrees> [<ease>]", "", "Relative setup skew.";
            "translate", "<seconds> <x> <y> [<ease>]", "", "Offset from setup, pixels.";
            "scale", "<seconds> <x> <y> [<ease>]", "", "Multiplier of setup scale."
        ],
    ),
    (
        "clip.slot",
        rows![
            "attachment", "<seconds> <attachment or none>", "", "Held display key.";
            "color", "<seconds> <r> <g> <b> <a> [<ease>]", "", "Colour multiplier.";
            "offset", "<seconds> <r> <g> <b> <a> [<ease>]", "", "Additive colour."
        ],
    ),
    (
        "clip.deform",
        rows![
            "key",
            "<seconds> <x> <y> ... [<ease>]",
            "",
            "One bone-local pixel offset pair per influence, in vertex order."
        ],
    ),
    (
        "clip.ik",
        rows![
            "key",
            "<seconds> <weight> clockwise|counterclockwise [<ease>]",
            "",
            "Eased weight, held bend side."
        ],
    ),
];

pub(super) fn check(entries: &[Entry], scope: &str) -> Result<()> {
    let lines = if scope.is_empty() {
        LINES
    } else {
        BLOCKS
            .iter()
            .find(|(name, _)| *name == scope)
            .map(|(_, lines)| *lines)
            .unwrap_or(&[])
    };
    for e in entries {
        if !lines.iter().any(|f| f.key == e.key) {
            return Err(parse_err(
                e.line,
                format!("unknown rig2d {scope} field '{}'", e.key),
            ));
        }
        if let Some(b) = &e.block {
            let next = if e.key == "armature" && scope.is_empty() {
                String::new()
            } else if scope.is_empty() {
                e.key.clone()
            } else {
                format!("{scope}.{}", e.key)
            };
            check(&b.entries, &next)?;
        }
    }
    Ok(())
}
