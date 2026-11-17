//! Last-resort vector icons. No atlas, host asset or renderer registration:
//! only a requested fallback parses its SVG, once, on either host.
use crate::paint::{Cap, ElementDraw, Join, Paint, Placed, ShapeKind};
use crate::{DrawCmd, Rect, Srgba, Theme};
use std::sync::OnceLock;

struct Icon {
    name: &'static str,
    svg: &'static str,
    paths: OnceLock<Vec<ShapeKind>>,
}

macro_rules! icons {
    ($($name:literal),+ $(,)?) => {
        /// The engine's last-resort icon names, after the game's icon layers.
        pub const DEFAULT_ICON_NAMES: &[&str] = &[$($name),+];
        static ICONS: [Icon; DEFAULT_ICON_NAMES.len()] = [$(Icon {
            name: $name,
            svg: include_str!(concat!("../../../third_party/lucide/icons/", $name, ".svg")),
            paths: OnceLock::new(),
        }),+];
    };
}

icons!(
    "back",
    "bag",
    "bolt",
    "check",
    "chevronDown",
    "chevronLeft",
    "chevronRight",
    "clock",
    "close",
    "coin",
    "flag",
    "lock",
    "map",
    "music",
    "person",
    "play",
    "plus",
    "skull",
    "star",
    "trophy",
);

pub(crate) fn find(name: &str) -> Option<&'static str> {
    ICONS
        .iter()
        .find(|icon| icon.name == name)
        .map(|icon| icon.name)
}

pub(crate) fn commands(
    name: &str,
    rect: Rect,
    clip: Rect,
    color: Srgba,
    theme: &Theme,
) -> Vec<DrawCmd> {
    let Some(icon) = ICONS.iter().find(|icon| icon.name == name) else {
        return Vec::new();
    };
    let paths = icon.paths.get_or_init(|| {
        let svg = crate::svg::parse_xml(icon.svg).expect("bundled Lucide SVG");
        svg.children
            .iter()
            .flat_map(|el| {
                let path =
                    crate::svg::shape_path(&el.name, &|name| el.get(name), [0.0, 0.0, 24.0, 24.0])
                        .expect("bundled Lucide shape")
                        .expect("a drawable shape");
                crate::svg::subpaths(&path)
            })
            .map(|(points, closed)| ShapeKind::Path {
                points,
                closed,
                filled: false,
                curves: true,
                fill_rule: Default::default(),
                contours: Vec::new(),
                width: 2.0,
                cap: Cap::Round,
                join: Join::Round,
            })
            .collect()
    });
    let side = rect.width.min(rect.height);
    let rect = Rect::new(
        rect.x + (rect.width - side) * 0.5,
        rect.y + (rect.height - side) * 0.5,
        side,
        side,
    );
    paths
        .iter()
        .flat_map(|shape| {
            ElementDraw {
                shape: shape.clone(),
                view_box: Some([0.0, 0.0, 24.0, 24.0]),
                fills: vec![Paint::Solid(color.into())],
                ..Default::default()
            }
            .enter(
                theme,
                &Placed {
                    rect,
                    radius: 0.0,
                    clip,
                    alpha: 1.0,
                },
            )
        })
        .collect()
}
