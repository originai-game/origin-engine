//! The UI pass's scene that cooks (og-tests' ui_scenes has the others):
//!
//! - `ui-vector` (native: cooked by og-cook): an SVG emblem tessellated and
//!   drawn at three sizes and turned.
//! - `ui-rive` (native, `docs/ui.md` §3.3): rive-runtime's test artboards
//!   (`tests/fixtures/rive`) converted by og-cook and drawn by og-ui's Rive
//!   player: a circle on its artboard's ground; a nested artboard at its
//!   container's opacity; two rectangles keyed by a cubic value
//!   interpolator at four ticks of their animation; a second artboard of
//!   a file. Each in a 280 px box, fitted by contain.

use og_render::{Overlay, RenderScene, UiShape};
use og_tests::ui_scenes::overlay_scene;

/// An emblem in the SVG subset: a gradient shield with a stroked rim, a
/// star with round joins, a dashed ring of arcs, and an even-odd hole.
pub const EMBLEM_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 120 120">
  <defs>
    <linearGradient id="metal" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#ffe082"/>
      <stop offset="1" stop-color="#ff6f00"/>
    </linearGradient>
    <radialGradient id="glow" cx="60" cy="56" r="40" gradientUnits="userSpaceOnUse">
      <stop offset="0" stop-color="#4fc3f7"/>
      <stop offset="1" stop-color="#1a237e"/>
    </radialGradient>
  </defs>
  <path d="M60 6 L108 24 C108 70 90 98 60 114 C30 98 12 70 12 24 Z" fill="url(#metal)" stroke="#3e2723" stroke-width="4" stroke-linejoin="round"/>
  <path fill-rule="evenodd" fill="url(#glow)" d="M60 22 a34 34 0 1 0 0.01 0 Z M60 38 a18 18 0 1 1 -0.01 0 Z"/>
  <polygon points="60,40 65,53 79,53 68,61 72,75 60,67 48,75 52,61 41,53 55,53" fill="#fffde7" stroke="#ff6f00" stroke-width="2" stroke-linejoin="round"/>
  <circle cx="60" cy="56" r="44" fill="none" stroke="#fff" stroke-opacity="0.7" stroke-width="2" stroke-dasharray="6 4"/>
  <g transform="translate(60 102) rotate(45)"><rect x="-5" y="-5" width="10" height="10" rx="2" fill="#3e2723"/></g>
</svg>"##;

/// `ui-vector`: the emblem at 64, 160 and 360 CSS px, turned 20° at 160,
/// and tinted at 96.
#[cfg(not(target_arch = "wasm32"))]
pub fn vector() -> Result<RenderScene, String> {
    let data = og_cook::cook::vector::tessellate(EMBLEM_SVG, None)?;
    let mut err = None;
    let scene = overlay_scene(|scene, o| {
        let v = scene.add_ui_vector(data);
        let draw = |o: &mut Overlay, rect: [f32; 4], transform: [f32; 4], tint: [f32; 4]| {
            o.ui.vectors.push(og_render::UiVectorDraw {
                vector: v,
                rect,
                transform,
                tint,
                layer: 0,
                clip: None,
            });
        };
        let id = [1.0, 0.0, 0.0, 1.0];
        o.ui.shapes.push(UiShape::rect(
            [20.0, 20.0, 1240.0, 680.0],
            24.0,
            [0.08, 0.09, 0.12, 1.0],
        ));
        draw(o, [60.0, 60.0, 64.0, 64.0], id, [1.0; 4]);
        draw(o, [160.0, 60.0, 160.0, 160.0], id, [1.0; 4]);
        let (s, c) = 20f32.to_radians().sin_cos();
        draw(o, [360.0, 60.0, 160.0, 160.0], [c, s, -s, c], [1.0; 4]);
        draw(o, [60.0, 260.0, 96.0, 96.0], id, [0.4, 0.9, 1.0, 1.0]);
        draw(o, [600.0, 40.0, 640.0, 640.0], id, [1.0; 4]);
        if o.ui.vectors.is_empty() {
            err = Some("no vector drawn".to_string());
        }
    });
    match err {
        Some(e) => Err(e),
        None => Ok(scene),
    }
}

/// The `.riv` fixtures' directory.
#[cfg(not(target_arch = "wasm32"))]
fn rive_fixture(name: &str) -> Result<Vec<u8>, String> {
    let p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/rive/assets")
        .join(name);
    std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))
}

/// `ui-rive`. See the module documentation.
#[cfg(not(target_arch = "wasm32"))]
pub fn rive() -> Result<RenderScene, String> {
    use og_ui::rive::Player;
    use std::sync::Arc;
    let load = |file: &str, artboard: &str| -> Result<Arc<og_ui::rive::Artboard>, String> {
        Ok(Arc::new(og_cook::cook::riv::artboard(
            &rive_fixture(file)?,
            artboard,
        )?))
    };
    // (artboard, seconds played, box)
    let mut draws: Vec<(Player, [f32; 4])> = Vec::new();
    let cell = |col: usize, row: usize| {
        [
            40.0 + col as f32 * 300.0,
            60.0 + row as f32 * 320.0,
            280.0,
            280.0,
        ]
    };
    draws.push((
        Player::new(load("shapetest.riv", "New Artboard")?, None),
        cell(0, 0),
    ));
    draws.push((
        Player::new(
            load("nested_artboard_opacity.riv", "Parent Artboard")?,
            None,
        ),
        cell(1, 0),
    ));
    draws.push((
        Player::new(load("two_artboards.riv", "One")?, None),
        cell(2, 0),
    ));
    let cubic = load("cubic_value_test.riv", "New Artboard")?;
    for (k, frames) in [0.0f32, 9.0, 19.0, 40.0].into_iter().enumerate() {
        let mut p = Player::new(cubic.clone(), None);
        // The UI tick's fixed step, 1/60 s.
        for _ in 0..frames as u32 {
            p.advance(1.0 / 60.0);
        }
        draws.push((p, cell(k, 1)));
    }
    Ok(overlay_scene(|_, o| {
        for (p, rect) in &draws {
            o.ui.shapes
                .push(UiShape::rect(*rect, 12.0, [0.03, 0.035, 0.045, 1.0]));
            // The artboard cuts its content to its box.
            o.clips.push(*rect);
            let clip = (o.clips.len() - 1) as u32;
            // These artboards have no text: their shapes are all they draw.
            for d in p.draw(*rect, 1.0) {
                if let og_ui::rive::Drawn::Shape(mut s) = d {
                    s.clip = Some(clip);
                    o.ui.shapes.push(s);
                }
            }
        }
    }))
}
