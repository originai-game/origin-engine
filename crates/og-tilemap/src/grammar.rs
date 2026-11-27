//! File fields consumed by the map/tileset parsers and emitted by og api.
use og_core::ogt::{Block, line::Line};
use og_core::{Error, ErrorKind, Result};

macro_rules! lines {
    ($($key:literal, $args:literal, $default:literal, $doc:literal;)+) => {
        &[$(Line::new($key, $args, $default, $doc, "")),+]
    };
}

pub const MAP: &[Line] = lines! {
    "size", "<columns> <rows>", "required", "Positive map dimensions in cells; generation may supply them at cook.";
    "tile", "<width> <height>", "required", "Positive tile pixel dimensions.";
    "unit", "<number>", "1", "World metres per tile height; tile aspect ratio is retained.";
    "orientation", "orthogonal | isometric | staggered | hex [pointy | flat]", "orthogonal", "Cell placement; no orientation is inferred from tile art.";
    "tileset", "\"<asset>\"", "required", "Repeat ordered tileset asset references; layer.tileset indexes this list.";
    "layer", "\"<name>\" { ... }", "", "Repeat uniquely named tile layers.";
    "object", "\"<name>\" { ... }", "", "Repeat authored objects in cell coordinates.";
};
pub const LAYER: &[Line] = lines! {
    "tileset", "<index>", "0", "Index into the map's tilesets.";
    "z", "<integer>", "layer index", "Draw order.";
    "parallax", "<x> <y>", "1 1", "Camera-relative parallax factors.";
    "offset", "<x> <y>", "0 0", "World-space layer offset.";
    "opacity", "<number>", "1", "Layer opacity.";
    "visible", "true | false", "true", "Initial visibility.";
    "collision", "true | false", "false", "Generate colliders from the tileset's collision declarations.";
    "colliderLayers", "<bits>", "0", "Collision membership; requires collision true.";
    "colliderMask", "<bits>", "0", "Collision mask; requires collision true.";
    "cellColliders", "true | false", "false", "Keep individual cell colliders; requires collision true.";
    "lit", "true | false", "true", "Receive 2D lighting.";
    "repeat", "x | y | x y", "", "Repeat along the named axes.";
    "terrain", "\"<name>\"", "", "Tileset autotile terrain; rows are occupancy 0/1 instead of tile ids.";
    "legend", "\"<character>\" <tile id>", "", "Repeat character-to-id mappings for quoted rows; -1 is empty.";
    "row", "<tile id> ... | \"<characters>\"", "empty layer", "Repeat one row per map row, each exactly map width; nonnegative ids are zero based, negative ids empty.";
};
pub const OBJECT: &[Line] = lines! {
    "kind", "<name>", "", "Game-authored object type string.";
    "rect", "<x> <y> <width> <height>", "0 0 0 0", "Top-left and size in cells; y down is converted to world y up.";
    "at", "<x> <y>", "0 0", "Position in cells, y down; does not change rect size.";
    "prop", "<key> <value>", "", "Repeat string-valued game properties; values may be numbers, words or strings.";
};
pub(crate) fn check(block: &Block, lines: &[Line]) -> Result<()> {
    for e in &block.entries {
        if !lines.iter().any(|line| line.key == e.key) {
            return Err(Error::new(
                ErrorKind::Parse,
                format!("line {}: unknown key '{}'", e.line, e.key),
            ));
        }
    }
    Ok(())
}
