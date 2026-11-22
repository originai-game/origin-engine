//! A deterministic CPU turntable preview.
//!
//! - Views from `views` azimuths around the scene's vertical (kernel +Z)
//!   axis, all at one scale, so a part keeps its size from view to view.
//!   Orthographic by default; with [`Preview::fov`], perspective from a
//!   distance that fits the scene's bounding sphere as the orthographic
//!   view does. [`views`] gives each view's camera ([`ViewCamera`]).
//! - Triangles are snapped to a fixed-point grid (1/256 pixel) and covered
//!   by integer edge functions with a top-left rule: a pixel centre on an
//!   edge two triangles share belongs to exactly one of them. In
//!   perspective, values are interpolated perspective-correctly (the
//!   barycentrics weighted by 1 / depth).
//! - Lambert shading from one world light, interpolated in f64 from the
//!   integer barycentrics. Smooth only across facets of one surface that
//!   meet at less than [`CREASE_DEGREES`]: a corner's normal is the
//!   area-weighted mean of the triangles around its vertex on its surface
//!   whose normals lie within that angle of its own triangle's. A
//!   hexagonal loft shows its facets; a 12-sided or finer turn is round.
//!   (The GLB's normals stay smooth per surface.) [`Preview::unlit`]
//!   leaves the light out: colour only.
//! - Colour. A part with neither a drawable map nor vertex colours is its
//!   material's `color`, shaded in sRGB as it always was. A part with a
//!   map (PNG or JPEG, on a part with UVs) or [`Solid::colors`] is
//!   coloured per pixel: the texture sampled at the pixel's UV (a
//!   box-filtered mip chain, the level by the pixel's UV footprint,
//!   bilinear within it, repeat wrap; `crate::tools::sampler`), times the
//!   material's `color` (in `color_space`: an imported material's is
//!   linear), times the interpolated vertex colour, all in
//!   linear light, then back to sRGB and shaded as an untextured part is.
//!   A white map and white vertex colours give the untextured pixel. A
//!   material's `mapSize` is already in the UVs a model's scene carries.
//!   Each distinct map is decoded once per call; one that cannot be
//!   decoded is listed in [`Views::undrawn`] and the part drawn without
//!   it.
//! - Opaque parts first, with a depth buffer. Then the triangles of parts
//!   whose material's `opacity` is below 1, back to front, blended over
//!   what is drawn (alpha × their colour + (1 − alpha) × the pixel),
//!   tested against the opaque depth and not writing it. Alpha is the
//!   `opacity`, times the map's alpha and the vertex colour's alpha where
//!   the part has them; a part with `opacity` 1 is opaque whatever its
//!   map's alpha (glTF's `OPAQUE`, which export writes for it). The
//!   order is the sum of a triangle's three corner depths, farthest
//!   first, ties to the earlier part and then the earlier triangle: an
//!   exact comparison of values computed the same way on every host. A
//!   part with `opacity` 0 is not drawn.
//! - Real edges are drawn as one-pixel lines, hidden behind nearer faces.
//! - A ground square under the scene, with the scene's shadow cast on it
//!   along the light; where only transparent parts cast it, it is lighter
//!   by their opacity (the most opaque caster at the pixel).
//! - A material's `emissive` (times `emissive_intensity`, in its
//!   `color_space`) is added to the lit colour in linear light, before
//!   blending: it glows the same in shade and in light. A part without it
//!   is drawn as before, to the byte. The emissive map multiplies it.
//! - `metalness` and `roughness` (and the metallic-roughness map's blue
//!   and green) add what the surface reflects, in linear light: a
//!   highlight of the light, and a sky over a ground (lighter the more
//!   the reflection points up), blurred as the surface roughens; a
//!   metal's own colour tints both and it keeps no matte colour. A
//!   matte part that is no metal is drawn as before, to the byte. The
//!   occlusion map's red dims the light from all round by
//!   `ao_map_intensity` (`docs/research.md` §53).
//! - A material's `normal_map` (with the part's UVs) bends the light per
//!   pixel: the map is read as data, at its finest level, in the
//!   triangle's own tangent frame (u along the tangent, v along the
//!   bitangent, from its positions and UVs, round the interpolated
//!   normal), times `normal_scale`. A baked normal map and a file's own
//!   show their detail; a seam between charts shows as a line if the
//!   frames disagree. Unlit views do not read it.
//!
//! No GPU, threads or SIMD; every step is integer arithmetic or f64
//! `+ - * /`, `sqrt`, `floor` and `trig` in a fixed order, so the
//! framebuffer's bytes are the same on every host.

use crate::math::{DVec2, DVec3, trig};

use crate::mesh::Solid;

use crate::tools::export::{ColorSpace, ExportError, Material, Scene};
use crate::tools::image::Texture;
use crate::tools::sampler::{Mips, to_linear, to_srgb};
use crate::tools::surface::face_normal;

mod camera;
mod edges;
mod environment;
mod ground;
mod raster;
mod shade;
mod sheet;
pub(crate) use camera::*;
pub(crate) use edges::*;
pub use environment::*;
pub(crate) use ground::*;
pub(crate) use raster::*;
pub(crate) use shade::*;
pub use sheet::*;

/// An RGBA image, 8 bits per channel, rows top to bottom.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Image {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// `width * height * 4` bytes.
    pub rgba: Vec<u8>,
}

/// What [`preview`] draws. The zero value is four 256 × 256 views.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Preview {
    /// Pixels across; 0 is 256.
    pub width: u32,
    /// Pixels down; 0 is the width.
    pub height: u32,
    /// Views evenly around the turntable; 0 is 4.
    pub views: u32,
    /// Camera height above the horizon, radians; `None` is 25°. Must lie
    /// in (0, π/2]: the ground is seen from above.
    pub elevation: Option<f64>,
    /// Azimuth of the first view, radians from the side the kernel's −Y
    /// looks at (the front), turning counter-clockwise seen from above.
    pub azimuth: f64,
    /// Leave the real edges out.
    pub no_edges: bool,
    /// Leave the ground and its shadow out.
    pub no_ground: bool,
    /// Perspective: the vertical field of view, radians, in (0, π/3];
    /// `None` is orthographic. The camera stands as far off as fits the
    /// scene's bounding sphere into 92 % of the smaller side, as the
    /// orthographic view's scale does.
    pub fov: Option<f64>,
    /// Colour only: no light, so every pixel is its material's colour
    /// times its map and vertex colour. Edges and the ground stay unless
    /// [`Preview::no_edges`] and [`Preview::no_ground`] leave them out.
    pub unlit: bool,
    /// The box the camera frames and the ground lies under, corner to
    /// corner; `None` is the scene's own. One box for every frame of a
    /// motion keeps the camera still while the scene moves in it.
    pub frame: Option<(DVec3, DVec3)>,
}

/// Where a view looks from, in the scene's frame (kernel units, +Z up).
/// For three.js: a camera at `position` with this `up`, looking at
/// `target`; `PerspectiveCamera(fov in degrees, width / height)`, or an
/// `OrthographicCamera` `ortho_height` tall and `ortho_height × width /
/// height` wide. Pixel (0, 0) is the image's top left.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ViewCamera {
    /// The eye. Orthographic: on the view's axis, three bounding-sphere
    /// radii from the target (any point on that line sees the same).
    pub position: DVec3,
    /// The point at the image's centre: the centre of the scene's box.
    pub target: DVec3,
    /// The image's up, a unit vector square to the view.
    pub up: DVec3,
    /// The vertical field of view, radians; `None` is orthographic.
    pub fov: Option<f64>,
    /// Orthographic: the scene units the image's height spans; 0 in
    /// perspective.
    pub ortho_height: f64,
    /// Pixels across.
    pub width: u32,
    /// Pixels down.
    pub height: u32,
}

/// What [`views`] draws: the images, their cameras, and the maps it could
/// not draw.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Views {
    /// One image a view.
    pub images: Vec<Image>,
    /// The camera of each image.
    pub cameras: Vec<ViewCamera>,
    /// Parts whose map or vertex colours were not drawn, and why (a map
    /// that is not PNG or JPEG, or does not decode; a part with no UVs).
    pub undrawn: Vec<(String, String)>,
}

pub(crate) const SUB: i64 = 256; // fixed-point steps per pixel
pub(crate) const BACKGROUND: [u8; 3] = [246, 246, 244];
pub(crate) const GROUND: [u8; 3] = [222, 222, 218];
pub(crate) const SHADOW: [u8; 3] = [178, 178, 174];
/// Shading: ambient plus diffuse, of the base colour.
pub(crate) const AMBIENT: f64 = 0.38;
pub(crate) const DIFFUSE: f64 = 0.62;
/// Facets of one surface meeting at less than this many degrees are
/// shaded smooth; at this angle or more the edge between them shows.
pub const CREASE_DEGREES: f64 = 35.0;

/// The scene from `views` directions around its vertical axis.
///
/// Errors as [`crate::tools::glb`] does on an invalid scene. [`views`] gives
/// the cameras and the maps not drawn too.
pub fn preview(scene: &Scene, d: &Preview) -> Result<Vec<Image>, ExportError> {
    Ok(views(scene, d)?.images)
}

/// [`preview`], lit by a picture of what is all round ([`views_in`]).
pub fn preview_in(
    scene: &Scene,
    d: &Preview,
    environment: Option<&Environment>,
) -> Result<Vec<Image>, ExportError> {
    Ok(views_in(scene, d, environment)?.images)
}

/// The longest side of a preview image, in pixels.
pub const MAX_SIDE: u32 = 8192;
/// The most views of one preview.
pub const MAX_VIEWS: u32 = 720;
/// The most pixels of one preview, all its views together (2²⁸: a
/// gigabyte of RGBA).
pub const MAX_PIXELS: u64 = 1 << 28;

/// The scene from `views` directions around its vertical axis, with each
/// view's camera and the maps not drawn.
///
/// Refuses more than [`MAX_SIDE`] pixels a side, [`MAX_VIEWS`] views or
/// [`MAX_PIXELS`] pixels in all ([`ExportError::TooLarge`]).
/// Errors as [`crate::tools::glb`] does on an invalid scene, and on an elevation
/// or field of view out of range.
pub fn views(scene: &Scene, d: &Preview) -> Result<Views, ExportError> {
    views_in(scene, d, None)
}

/// [`views`], lit by a picture of what is all round ([`Environment`])
/// in place of the plain sky over the ground.
pub fn views_in(
    scene: &Scene,
    d: &Preview,
    environment: Option<&Environment>,
) -> Result<Views, ExportError> {
    // The same checks as export, and each part's pose.
    let placed = crate::tools::export::check(scene)?;
    let width = if d.width == 0 { 256 } else { d.width };
    let height = if d.height == 0 { width } else { d.height };
    let views = if d.views == 0 { 4 } else { d.views };
    // Bounds, so a caller that passes a request on need not check it:
    // an image side, the views, and the pixels of them all.
    if width > MAX_SIDE || height > MAX_SIDE {
        return Err(ExportError::TooLarge(format!(
            "{width} x {height} pixels; a side is at most {MAX_SIDE}"
        )));
    }
    if views > MAX_VIEWS {
        return Err(ExportError::TooLarge(format!(
            "{views} views; at most {MAX_VIEWS}"
        )));
    }
    let pixels = u64::from(width) * u64::from(height) * u64::from(views);
    if pixels > MAX_PIXELS {
        return Err(ExportError::TooLarge(format!(
            "{views} views of {width} x {height} are {pixels} pixels; at most {MAX_PIXELS} in all"
        )));
    }
    let elevation = d.elevation.unwrap_or(trig::to_radians(25.0));
    if !(elevation > 0.0 && elevation <= trig::PI / 2.0) || !d.azimuth.is_finite() {
        return Err(ExportError::NotFinite("the preview's elevation".into()));
    }
    if let Some(f) = d.fov
        && !(f > 0.0 && f <= trig::PI / 3.0)
    {
        return Err(ExportError::NotFinite(
            "the preview's field of view (over 0, at most pi/3)".into(),
        ));
    }
    let crease = trig::cos(trig::to_radians(CREASE_DEGREES));
    let mut maps = Maps::new();
    let mut undrawn = Vec::new();
    let mut parts: Vec<Drawn> = Vec::with_capacity(scene.parts.len());
    for (p, m) in scene.parts.iter().zip(&placed) {
        let opacity = p.material.opacity.unwrap_or(1.0);
        if !opacity.is_finite() {
            return Err(ExportError::NotFinite(format!(
                "the opacity of part {:?}",
                p.name
            )));
        }
        let tris = p
            .solid
            .triangles
            .iter()
            .map(|t| t.map(|v| m.apply(p.solid.positions[v as usize])))
            .collect();
        let normals = corner_normals(p.solid, crease)
            .into_iter()
            .map(|c| c.map(|n| m.rotate(n)))
            .collect();
        let edges = p
            .solid
            .real_edges()
            .into_iter()
            .map(|(a, b)| {
                (
                    m.apply(p.solid.positions[a as usize]),
                    m.apply(p.solid.positions[b as usize]),
                )
            })
            .collect();
        // The preview's colours are sRGB; an imported material's factors
        // are linear (`ColorSpace::Linear`).
        let rgb = p.material.color.unwrap_or([1.0; 3]);
        let rgb = match p.material.color_space {
            ColorSpace::Srgb => rgb,
            ColorSpace::Linear => rgb.map(|c| to_srgb(c.clamp(0.0, 1.0))),
        };
        let strength = p.material.emissive_intensity.unwrap_or(1.0);
        if !(strength.is_finite() && p.material.emissive.iter().all(|c| c.is_finite())) {
            return Err(ExportError::NotFinite(format!(
                "the emissive colour of part {:?}",
                p.name
            )));
        }
        let emissive = p.material.emissive.map(|c| {
            let c = c.clamp(0.0, 1.0);
            let c = match p.material.color_space {
                ColorSpace::Srgb => to_linear(c),
                ColorSpace::Linear => c,
            };
            c * strength.max(0.0)
        });
        let paint = paint(
            p.name.as_str(),
            p.solid,
            &p.material,
            rgb,
            &mut maps,
            &mut undrawn,
            environment.is_some(),
        );
        parts.push(Drawn {
            tris,
            normals,
            edges,
            rgb,
            emissive,
            opacity: opacity.clamp(0.0, 1.0),
            paint,
        });
    }
    let mut lo = DVec3::splat(f64::INFINITY);
    let mut hi = DVec3::splat(f64::NEG_INFINITY);
    for p in &parts {
        for &q in p.tris.iter().flatten() {
            lo = lo.min(q);
            hi = hi.max(q);
        }
    }
    if lo.x > hi.x {
        // Nothing to draw: the background only.
        lo = DVec3::ZERO;
        hi = DVec3::ZERO;
    }
    if let Some((a, b)) = d.frame {
        if !(a.is_finite() && b.is_finite()) {
            return Err(ExportError::NotFinite("the preview's frame".into()));
        }
        lo = a.min(b);
        hi = a.max(b);
    }
    let centre = (lo + hi) * 0.5;
    let radius = ((hi - lo) * 0.5).length().max(1e-9);
    let fit = f64::from(width.min(height)) * 0.5 * 0.92 / radius;
    // Perspective: the focal length in pixels, and the distance at which
    // the bounding sphere's silhouette fills 92 % of the smaller side:
    // tan(half-angle) = t, so radius / distance = t / √(1 + t²).
    let lens = d.fov.map(|f| {
        let tan = trig::tan(f * 0.5);
        let focal = f64::from(height) * 0.5 / tan;
        let t = 0.92 * tan * f64::from(width.min(height)) / f64::from(height);
        (focal, radius * (1.0 + t * t).sqrt() / t)
    });
    // One world light, from above, front left; it also casts the shadow.
    let light = DVec3::new(-0.45, -0.6, 1.0).normalize();
    let ground_z = lo.z;
    let half = radius * 1.15;
    let ground = [
        DVec3::new(centre.x - half, centre.y - half, ground_z),
        DVec3::new(centre.x + half, centre.y - half, ground_z),
        DVec3::new(centre.x + half, centre.y + half, ground_z),
        DVec3::new(centre.x - half, centre.y + half, ground_z),
    ];
    let (se, ce) = trig::sin_cos(elevation);
    let mut out = Views {
        undrawn,
        ..Default::default()
    };
    for k in 0..views {
        let az = d.azimuth + trig::TAU * (f64::from(k) / f64::from(views));
        let (sa, ca) = trig::sin_cos(az);
        // At azimuth 0 the camera stands at −Y looking at +Y.
        let toward = DVec3::new(sa * ce, -ca * ce, se); // from the scene to the camera
        let right = DVec3::new(ca, sa, 0.0);
        // Right-handed: right × up = toward; up leans to +Z.
        let up = toward.cross(right);
        let cam = Camera {
            centre,
            right,
            up,
            forward: -toward,
            scale: fit,
            width,
            height,
            eye: lens.map(|(focal, dist)| (centre + toward * dist, focal)),
        };
        out.cameras.push(ViewCamera {
            position: match lens {
                Some((_, dist)) => centre + toward * dist,
                None => centre + toward * (radius * 3.0),
            },
            target: centre,
            up,
            fov: d.fov,
            ortho_height: if lens.is_some() {
                0.0
            } else {
                f64::from(height) / fit
            },
            width,
            height,
        });
        let n = (width * height) as usize;
        let mut t = Target {
            width,
            height,
            rgb: vec![BACKGROUND; n],
            depth: vec![f64::INFINITY; n],
            ground: vec![false; n],
        };
        let look = Look {
            cam: &cam,
            light,
            toward,
            unlit: d.unlit,
            maps: &maps,
            environment,
        };
        if !d.no_ground {
            draw_ground(&mut t, &cam, &ground, &parts, light, ground_z);
        }
        for p in parts.iter().filter(|p| p.opacity == 1.0) {
            for k in 0..p.tris.len() {
                draw_triangle(&mut t, &look, p, k);
            }
        }
        for (part, k) in back_to_front(&cam, &parts) {
            draw_triangle(&mut t, &look, &parts[part], k);
        }
        if !d.no_edges {
            for p in parts.iter().filter(|p| p.opacity > 0.0) {
                draw_edges(&mut t, &cam, &p.edges, radius);
            }
        }
        let mut rgba = Vec::with_capacity(n * 4);
        for px in &t.rgb {
            rgba.extend_from_slice(px);
            rgba.push(255);
        }
        out.images.push(Image {
            width,
            height,
            rgba,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests;
