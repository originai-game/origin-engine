//! Shared accepted graph fields for the runtime parser and file reference.
use super::*;
use og_bundle::grammar::Line;

macro_rules! lines {
    ($($key:literal, $args:literal, $default:literal, $doc:literal);* $(;)?) => {
        &[$(Line::new($key, $args, $default, $doc, "")),*]
    };
}

pub const BLOCKS: &[(&str, &[Line])] = &[
    (
        "",
        lines![
            "param", "<name>", "", "Declare at most eight unique parameters, in component p0..p7 order.";
            "clip", "<motion> \"<bound clip>\" { ... }", "", "Clip leaf, resolved by bind for skeletal assets or by the 2D target for sprite assets.";
            "blend1d", "<motion> <parameter> { ... }", "", "Threshold interpolation; at least one child.";
            "blend2d", "<motion> <x parameter> <y parameter> { ... }", "", "Freeform Cartesian interpolation; at least one child.";
            "layer", "<name> { ... }", "required", "Layered state machine; at least one state."
        ],
    ),
    (
        "clip",
        lines![
            "speed", "<number > 0>", "1", "Clip playback rate.";
            "loop", "true | false", "true", "Loop playback.";
            "mirror", "true | false", "false", "Mirror the skeletal pose."
        ],
    ),
    (
        "blend1d",
        lines![
            "child",
            "<motion> <threshold>",
            "required",
            "Repeat children; thresholds are sorted, clamped at the ends."
        ],
    ),
    (
        "blend2d",
        lines![
            "child",
            "<motion> <x> <y>",
            "required",
            "Repeat children at 2D parameter points."
        ],
    ),
    (
        "layer",
        lines![
            "state", "<name> <motion> { ... }", "required", "Declare a unique state, with optional speed and sync.";
            "entry", "<state>", "first state", "Initial state.";
            "mode", "override | additive", "override", "Pose combination mode.";
            "weight", "<number> | <parameter>", "1", "Fixed or parameter-driven layer weight, clamped to 0..1.";
            "mask", "\"<node>\" [weight]", "", "Repeat subtree weights 0..1 (omitted weight one), resolved by skeleton node names.";
            "transition", "<state> | any <target> { ... }", "", "Repeat transitions in priority order; needs a when or exit."
        ],
    ),
    (
        "layer.state",
        lines![
            "speed", "<number >= 0>", "1", "State playback multiplier.";
            "sync", "<group>", "", "Named phase synchronization group."
        ],
    ),
    (
        "layer.transition",
        lines![
            "when", "<parameter> gt | ge | lt | le | eq | ne <number>", "", "Repeat conditions; all must hold.";
            "exit", "<cycles>", "", "Minimum source phase before the transition may fire.";
            "fade", "<seconds>", "0.2", "Cross-fade duration, clamped at zero.";
            "interrupt", "any | none | source | next", "any", "Which transitions may interrupt this fade."
        ],
    ),
];

pub(super) fn check(block: &Block, path: &str) -> Result<()> {
    let Some((_, lines)) = BLOCKS.iter().find(|(p, _)| *p == path) else {
        return Ok(());
    };
    for e in &block.entries {
        if og_bundle::grammar::find(lines, &e.key, &[]).is_none() {
            return Err(err(
                e,
                format!("unknown graph field '{}' in '{path}'", e.key),
            ));
        }
        if let Some(b) = &e.block {
            let child = if path.is_empty() {
                e.key.clone()
            } else {
                format!("{path}.{}", e.key)
            };
            check(b, &child)?;
        }
    }
    Ok(())
}
