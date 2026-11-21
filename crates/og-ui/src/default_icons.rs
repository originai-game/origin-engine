//! The draw commands of the engine's last-resort icons (og-ui-next's
//! [`og_ui_next::default_icons`]), parsed once per icon on first draw.
use crate::paint::{Cap, ElementDraw, Join, Paint, Placed, ShapeKind};
use crate::{DrawCmd, Rect, Srgba, Theme};
use og_ui_next::default_icons::{DEFAULT_ICON_NAMES, svg};
use std::sync::OnceLock;

/// The draw commands of built-in icon `name` in `rect`: what an icon
/// that no atlas names draws, for a host that paints its own elements.
pub fn commands(name: &str, rect: Rect, clip: Rect, color: Srgba, theme: &Theme) -> Vec<DrawCmd> {
    static PATHS: [OnceLock<Vec<ShapeKind>>; DEFAULT_ICON_NAMES.len()] =
        [const { OnceLock::new() }; DEFAULT_ICON_NAMES.len()];
    let Some(at) = DEFAULT_ICON_NAMES.iter().position(|&n| n == name) else {
        return Vec::new();
    };
    let paths = PATHS[at].get_or_init(|| {
        let svg =
            crate::svg::parse_xml(svg(name).expect("a built-in icon")).expect("bundled Lucide SVG");
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
