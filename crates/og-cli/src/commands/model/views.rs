//! Named views of a model, the ones concept sheets draw: `front`, `back`,
//! `left`, `right`, `top`, `three_quarter`, and `camera:{…}` for any
//! other. Preview, compare and bake all frame a view the same way, so an
//! image of one lines up with the others.
//!
//! Axes (the GLB's: metres, +Y up): the model faces +Z. `front` looks
//! from +Z, `back` from −Z, `right` from the model's right (−X), `left`
//! from +X, all level; `top` looks down with the model's front at the
//! bottom of the image; `three_quarter` is 45° from the front toward the
//! right, 25° up. These are orthographic, framed on the model's bounding
//! sphere, so a part has one size in every view. `camera:{"eye": [x, y,
//! z], "target": [x, y, z], "up": [x, y, z], "fov": radians}` is in the
//! GLB's frame; without `fov` it is orthographic along `target − eye`.
//!
//! Entry: [`parse`], then [`render`].

use og_core::math::double;
use og_shape::math::DVec3;
use og_shape::tools::{Aim, Image, Preview, Scene, ViewCamera};
use serde_json::{Value, json};

use super::Failure;

/// One view: a name concept sheets use, or a camera.
#[derive(Clone, Debug, PartialEq)]
pub enum View {
    Front,
    Back,
    Left,
    Right,
    Top,
    ThreeQuarter,
    /// A camera in the GLB's frame (metres, +Y up); `fov` in radians,
    /// `None` orthographic.
    Camera {
        eye: [f64; 3],
        target: [f64; 3],
        up: [f64; 3],
        fov: Option<f64>,
    },
}

/// The names `--views` and `--view` take.
pub const NAMES: [&str; 6] = ["front", "back", "left", "right", "top", "three_quarter"];

impl View {
    /// A view by its name, or `camera:{…}`.
    pub fn parse(text: &str) -> Result<View, Failure> {
        let text = text.trim();
        if let Some(json) = text.strip_prefix("camera:") {
            return camera(json);
        }
        Ok(match text {
            "front" => View::Front,
            "back" => View::Back,
            "left" => View::Left,
            "right" => View::Right,
            "top" => View::Top,
            "three_quarter" => View::ThreeQuarter,
            other => {
                return Err(Failure::Usage(format!(
                    "no view {other:?}: {}, or camera:{{\"eye\", \"target\", \"up\", \"fov\"?}}",
                    NAMES.join(", ")
                )));
            }
        })
    }

    /// The view's name; a camera's is `camera`.
    pub fn name(&self) -> &'static str {
        match self {
            View::Front => "front",
            View::Back => "back",
            View::Left => "left",
            View::Right => "right",
            View::Top => "top",
            View::ThreeQuarter => "three_quarter",
            View::Camera { .. } => "camera",
        }
    }

    /// The kernel's preview of this one view, `size` pixels (width,
    /// height).
    pub fn preview(&self, size: (u32, u32)) -> Preview {
        let one = |azimuth: f64, elevation: f64| Preview {
            width: size.0,
            height: size.1,
            views: 1,
            azimuth,
            elevation: Some(elevation),
            ..Default::default()
        };
        // The kernel: millimetres, +Z up, the model facing −Y; azimuth 0
        // stands at −Y and turns counter-clockwise seen from above, so
        // the model's right (−X) is at −π/2.
        let quarter = double::PI / 2.0;
        match self {
            View::Front => one(0.0, 0.0),
            View::Back => one(double::PI, 0.0),
            View::Right => one(-quarter, 0.0),
            View::Left => one(quarter, 0.0),
            View::Top => one(0.0, quarter),
            View::ThreeQuarter => one(-quarter / 2.0, double::to_radians(25.0)),
            View::Camera {
                eye,
                target,
                up,
                fov,
            } => Preview {
                width: size.0,
                height: size.1,
                views: 1,
                fov: *fov,
                aim: Some(Aim {
                    eye: kernel(*eye) * 1000.0,
                    target: kernel(*target) * 1000.0,
                    up: kernel(*up),
                }),
                ..Default::default()
            },
        }
    }
}

/// A comma-separated list of views; a camera's JSON may hold commas.
pub fn parse(list: &str) -> Result<Vec<View>, Failure> {
    let (mut out, mut depth, mut start) = (Vec::new(), 0i32, 0);
    for (i, c) in list.char_indices() {
        match c {
            '{' | '[' => depth += 1,
            '}' | ']' => depth -= 1,
            ',' if depth == 0 => {
                out.push(View::parse(&list[start..i])?);
                start = i + 1;
            }
            _ => {}
        }
    }
    if !list[start..].trim().is_empty() {
        out.push(View::parse(&list[start..])?);
    }
    if out.is_empty() {
        return Err(Failure::Usage("no views".into()));
    }
    Ok(out)
}

/// Each view's file stem: its name, cameras numbered when there is more
/// than one (`camera0`, `camera1`, …).
pub fn stems(views: &[View]) -> Vec<String> {
    let cameras = views
        .iter()
        .filter(|v| matches!(v, View::Camera { .. }))
        .count();
    let mut k = 0;
    views
        .iter()
        .map(|v| match v {
            View::Camera { .. } if cameras > 1 => {
                k += 1;
                format!("camera{}", k - 1)
            }
            v => v.name().to_string(),
        })
        .collect()
}

fn camera(text: &str) -> Result<View, Failure> {
    let bad = |why: &str| Failure::Usage(format!("camera:{text}: {why}"));
    let v: Value = serde_json::from_str(text).map_err(|e| bad(&e.to_string()))?;
    let triple = |k: &str, default: Option<[f64; 3]>| -> Result<[f64; 3], Failure> {
        match v.get(k) {
            None => default.ok_or_else(|| bad(&format!("no {k:?}"))),
            Some(a) => {
                let a: Vec<f64> = a
                    .as_array()
                    .map(|a| a.iter().filter_map(Value::as_f64).collect())
                    .unwrap_or_default();
                <[f64; 3]>::try_from(a).map_err(|_| bad(&format!("{k:?} is [x, y, z]")))
            }
        }
    };
    Ok(View::Camera {
        eye: triple("eye", None)?,
        target: triple("target", Some([0.0; 3]))?,
        up: triple("up", Some([0.0, 1.0, 0.0]))?,
        fov: v.get("fov").and_then(Value::as_f64),
    })
}

/// A point or direction in the GLB's frame (+Y up) in the kernel's (+Z
/// up), unscaled: the inverse of the export's `(x, z, −y)`.
fn kernel(v: [f64; 3]) -> DVec3 {
    DVec3::new(v[0], -v[2], v[1])
}

/// A kernel point or direction in the GLB's frame, scaled by `k`.
fn gltf(v: DVec3, k: f64) -> [f64; 3] {
    [v.x * k, v.z * k, -v.y * k]
}

/// How a view is drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Look {
    /// Colour only, no light: each pixel its material's colour.
    pub unlit: bool,
    /// The real edges drawn.
    pub edges: bool,
}

/// One view of the scene on a transparent background (alpha the
/// silhouette), with its camera.
pub fn render(
    scene: &Scene,
    view: &View,
    size: (u32, u32),
    look: Look,
) -> Result<(Image, ViewCamera), Failure> {
    let d = Preview {
        transparent: true,
        no_ground: true,
        no_edges: !look.edges,
        unlit: look.unlit,
        ..view.preview(size)
    };
    let mut v = og_shape::tools::views(scene, &d, &og_shape_formats::texture::CODEC)
        .map_err(super::failed)?;
    match (v.images.pop(), v.cameras.pop()) {
        (Some(i), Some(c)) => Ok((i, c)),
        _ => Err(Failure::Failed("the view drew no image".into())),
    }
}

/// A camera as the platform reads it: in the GLB's frame, metres;
/// `scale` the metres the image's height spans (orthographic), `fov` its
/// vertical angle (perspective).
pub fn camera_json(c: &ViewCamera) -> Value {
    json!({
        "eye": gltf(c.position, 0.001),
        "target": gltf(c.target, 0.001),
        "up": gltf(c.up, 1.0),
        "scale": c.ortho_height * 0.001,
        "fov": c.fov,
    })
}
