//! The primitives: box, cylinder, cone, tube, sphere, dome, torus and wedge, each a zero-value desc.

use super::*;

/// A box.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoxDesc {
    /// Size along x.
    pub width: f64,
    /// Size along y.
    pub depth: f64,
    /// Size along z.
    pub height: f64,
}

/// A box centred on the origin in x and y, from z = 0 up. Its walls are
/// also named `right` (+x), `left` (−x), `back` (+y) and `front` (−y).
pub fn cuboid(d: &BoxDesc) -> Result<Solid, GenError> {
    cuboid_with(d, false)
}

pub(crate) fn cuboid_with(d: &BoxDesc, uvs: bool) -> Result<Solid, GenError> {
    let p = rect(&Rect {
        width: d.width,
        height: d.depth,
        radius: 0.0,
    })?;
    let mut s = extrude_with(
        &p,
        &Extrude {
            height: d.height,
            ..Default::default()
        },
        uvs,
    )?;
    let (w, dd) = (d.width / 2.0, d.depth / 2.0);
    let names: Vec<(u32, &str)> = s
        .triangles
        .iter()
        .enumerate()
        .filter_map(|(i, t)| {
            let [a, b, c] = t.map(|v| s.positions[v as usize]);
            let side = |f: &dyn Fn(DVec3) -> bool| f(a) && f(b) && f(c);
            let name = if side(&|p| p.x == w) {
                "right"
            } else if side(&|p| p.x == -w) {
                "left"
            } else if side(&|p| p.y == dd) {
                "back"
            } else if side(&|p| p.y == -dd) {
                "front"
            } else {
                return None;
            };
            Some((i as u32, name))
        })
        .collect();
    for (i, n) in names {
        let id = s.names.intern(n);
        let tags = std::sync::Arc::make_mut(&mut s.face_tags[i as usize]);
        tags.push(id);
        tags.sort_unstable();
    }
    s.canonicalize()
        .map_err(|_| GenError::NotFinite("positions"))?;
    Ok(s)
}

/// A cylinder or cone.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CylinderDesc {
    /// Bottom radius.
    pub radius: f64,
    /// Top radius; 0 keeps `radius` (a cylinder). Use [`cone`] for an apex.
    pub radius_top: f64,
    /// Height along +z.
    pub height: f64,
    /// Segments round; 0 for the chord rule.
    pub segments: u32,
}

/// A cylinder (or truncated cone) on z = 0 about the z axis.
pub fn cylinder(d: &CylinderDesc) -> Result<Solid, GenError> {
    cylinder_with(d, false)
}

pub(crate) fn cylinder_with(d: &CylinderDesc, uvs: bool) -> Result<Solid, GenError> {
    let r = positive(d.radius, "radius")?;
    let top = if d.radius_top == 0.0 {
        r
    } else {
        positive(d.radius_top, "radius_top")?
    };
    let base = circle(&Circle {
        radius: r,
        segments: count(
            segments_field(d.segments, "segments")?,
            r.max(top),
            double::TAU,
        ),
    })?;
    extrude_with(
        &base,
        &Extrude {
            height: d.height,
            taper: 1.0 - top / r,
            ..Default::default()
        },
        uvs,
    )
}

/// A cone with its apex on the z axis at `height`.
pub fn cone(radius: f64, height: f64) -> Result<Solid, GenError> {
    cone_desc(&ConeDesc {
        radius,
        height,
        segments: 0,
    })
}

/// A cone.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ConeDesc {
    /// Base radius, on z = 0.
    pub radius: f64,
    /// Height of the apex along +z.
    pub height: f64,
    /// Segments round; 0 for the chord rule.
    pub segments: u32,
}

/// A cone from its desc (as [`cone`]).
pub fn cone_desc(d: &ConeDesc) -> Result<Solid, GenError> {
    cone_with(d, false)
}

pub(crate) fn cone_with(d: &ConeDesc, uvs: bool) -> Result<Solid, GenError> {
    let (radius, height) = (d.radius, d.height);
    let p = circle(&Circle {
        radius: positive(radius, "radius")?,
        segments: segments_field(d.segments, "segments")?,
    })?;
    extrude_with(
        &p,
        &Extrude {
            height,
            taper: 1.0,
            ..Default::default()
        },
        uvs,
    )
}

/// A tube: a cylinder with a coaxial bore (walls `side` and `side`+`inner`).
pub fn tube(outer: f64, inner: f64, height: f64) -> Result<Solid, GenError> {
    tube_desc(&TubeDesc {
        outer,
        inner,
        height,
        segments: 0,
    })
}

/// A tube.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TubeDesc {
    /// Outer radius.
    pub outer: f64,
    /// Bore radius, below `outer`.
    pub inner: f64,
    /// Height along +z.
    pub height: f64,
    /// Segments round (both walls); 0 for the chord rule.
    pub segments: u32,
}

/// A tube from its desc (as [`tube`]).
pub fn tube_desc(d: &TubeDesc) -> Result<Solid, GenError> {
    tube_with(d, false)
}

pub(crate) fn tube_with(d: &TubeDesc, uvs: bool) -> Result<Solid, GenError> {
    let (outer, inner, height) = (d.outer, d.inner, d.height);
    let i = positive(inner, "inner")?;
    if i >= positive(outer, "outer")? {
        return Err(GenError::NotPositive("outer - inner"));
    }
    let n = count(segments_field(d.segments, "segments")?, outer, double::TAU);
    let p = circle(&Circle {
        radius: outer,
        segments: n,
    })?
    .subtract(&circle(&Circle {
        radius: i,
        segments: n,
    })?);
    extrude_with(
        &p,
        &Extrude {
            height,
            ..Default::default()
        },
        uvs,
    )
}

/// A sphere centred on the origin.
pub fn sphere(radius: f64) -> Result<Solid, GenError> {
    sphere_n(radius, 0, false)
}

/// A sphere with `per_turn` segments round (0: the chord rule) and half
/// as many from pole to pole.
pub(crate) fn sphere_n(radius: f64, per_turn: u32, uvs: bool) -> Result<Solid, GenError> {
    let r = positive(radius, "radius")?;
    let n = count(per_turn, r, double::PI).max(2);
    let mut half: Vec<DVec2> = (0..=n)
        .map(|k| {
            let (s, c) = double::sin_cos(-double::PI / 2.0 + double::PI * (k as f64 / n as f64));
            DVec2::new(r * c, r * s)
        })
        .collect();
    half[0].x = 0.0;
    half[n as usize].x = 0.0;
    let p = Profile::from_contour(Contour::smooth(half));
    revolve_with(
        &p,
        &Revolve {
            segments: per_turn,
            ..Default::default()
        },
        uvs,
    )
}

/// A dome: the upper half of a sphere, flat `bottom` on y = 0.
pub fn dome(radius: f64) -> Result<Solid, GenError> {
    dome_n(radius, 0, false)
}

pub(crate) fn dome_n(radius: f64, per_turn: u32, uvs: bool) -> Result<Solid, GenError> {
    let r = positive(radius, "radius")?;
    let n = count(per_turn, r, double::PI / 2.0);
    let mut pts: Vec<DVec2> = (0..=n)
        .map(|k| {
            let (s, c) = double::sin_cos(double::PI / 2.0 * (k as f64 / n as f64));
            DVec2::new(r * c, r * s)
        })
        .collect();
    pts[n as usize].x = 0.0;
    pts.push(DVec2::ZERO);
    let mut sharp = vec![false; pts.len()];
    sharp[0] = true;
    sharp[n as usize] = true;
    sharp[n as usize + 1] = true;
    let mut s = revolve_with(
        &Profile::from_contour(Contour { points: pts, sharp }),
        &Revolve {
            segments: per_turn,
            ..Default::default()
        },
        uvs,
    )?;
    // The flat disc on y = 0 is the dome's bottom.
    flat_bottom(&mut s);
    s.canonicalize()
        .map_err(|_| GenError::NotFinite("positions"))?;
    Ok(s)
}

/// Names the flat disc on y = 0 of a turned solid `bottom`, and gives it
/// planar UVs (x and z) if the solid has UVs.
pub(crate) fn flat_bottom(s: &mut Solid) {
    let bottom = s.names.intern("bottom");
    for (i, t) in s.triangles.iter().enumerate() {
        if t.iter().all(|&v| s.positions[v as usize].y == 0.0) {
            s.face_tags[i] = vec![bottom].into();
            if let Some(u) = &mut s.uvs {
                u[i] = t.map(|v| {
                    let p = s.positions[v as usize];
                    DVec2::new(p.x, p.z)
                });
            }
        }
    }
    unmirror_uvs(s);
}

/// A torus about the y axis.
pub fn torus(radius: f64, tube_radius: f64) -> Result<Solid, GenError> {
    torus_y(
        &TorusDesc {
            radius,
            tube_radius,
            ..Default::default()
        },
        false,
    )
}

/// A torus about the y axis from its desc, ignoring its `axis`.
pub(crate) fn torus_y(d: &TorusDesc, uvs: bool) -> Result<Solid, GenError> {
    let (r, t) = (
        positive(d.radius, "radius")?,
        positive(d.tube_radius, "tube_radius")?,
    );
    if t >= r {
        return Err(GenError::NotPositive("radius - tube_radius"));
    }
    let p = circle(&Circle {
        radius: t,
        segments: segments_field(d.tube_segments, "tube_segments")?,
    })?
    .translate(DVec2::new(r, 0.0));
    revolve_with(
        &p,
        &Revolve {
            angle: d.angle,
            segments: d.segments,
            ..Default::default()
        },
        uvs,
    )
}

/// A sphere.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SphereDesc {
    /// Radius.
    pub radius: f64,
    /// [`Axis::Y`]: centred on the origin, poles on y (as [`sphere`]).
    /// [`Axis::Z`]: poles on z, resting on z = 0 (sand3d's).
    pub axis: Axis,
    /// Segments round the poles' axis, and half as many from pole to
    /// pole; 0 for the chord rule.
    pub segments: u32,
}

/// A sphere from its desc.
pub fn sphere_desc(d: &SphereDesc) -> Result<Solid, GenError> {
    sphere_with(d, false)
}

pub(crate) fn sphere_with(d: &SphereDesc, uvs: bool) -> Result<Solid, GenError> {
    let mut s = sphere_n(d.radius, segments_field(d.segments, "segments")?, uvs)?;
    if d.axis == Axis::Z {
        y_to_z(&mut s, d.radius)?;
    }
    Ok(s)
}

/// A torus.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TorusDesc {
    /// Radius of the tube's centre line.
    pub radius: f64,
    /// Radius of the tube.
    pub tube_radius: f64,
    /// [`Axis::Y`]: about y, centred on the origin (as [`torus`]).
    /// [`Axis::Z`]: about z, resting on z = 0 (sand3d's).
    pub axis: Axis,
    /// The part of a turn the tube runs, radians, from +x as [`revolve`]
    /// turns; 0 for a whole ring. A part ring has flat `start` and `end`
    /// caps.
    pub angle: f64,
    /// Segments round the ring's axis (a part ring gets its share); 0 for
    /// the chord rule.
    pub segments: u32,
    /// Segments round the tube; 0 for the chord rule.
    pub tube_segments: u32,
}

/// A torus from its desc.
pub fn torus_desc(d: &TorusDesc) -> Result<Solid, GenError> {
    torus_with(d, false)
}

pub(crate) fn torus_with(d: &TorusDesc, uvs: bool) -> Result<Solid, GenError> {
    let mut s = torus_y(d, uvs)?;
    if d.axis == Axis::Z {
        y_to_z(&mut s, d.tube_radius)?;
    }
    Ok(s)
}

/// A dome: a spherical cap on a flat `bottom`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DomeDesc {
    /// Radius of the base.
    pub radius: f64,
    /// Height of the cap; 0 for a half sphere (`radius` high). Above
    /// `radius`, more than half a sphere.
    pub height: f64,
    /// [`Axis::Y`]: base on y = 0, rising up y (as [`dome`]).
    /// [`Axis::Z`]: base on z = 0, rising up z (sand3d's).
    pub axis: Axis,
    /// Segments round, and a quarter as many from the rim of a half
    /// sphere to its top (a cap its share); 0 for the chord rule.
    pub segments: u32,
}

/// A dome from its desc: sand3d's `dome(d, h)` with `radius = d / 2`.
pub fn dome_desc(d: &DomeDesc) -> Result<Solid, GenError> {
    dome_with(d, false)
}

pub(crate) fn dome_with(d: &DomeDesc, uvs: bool) -> Result<Solid, GenError> {
    let per_turn = segments_field(d.segments, "segments")?;
    let mut s = if d.height == 0.0 {
        dome_n(d.radius, per_turn, uvs)?
    } else {
        cap(
            positive(d.radius, "radius")?,
            positive(d.height, "height")?,
            per_turn,
            uvs,
        )?
    };
    if d.axis == Axis::Z {
        y_to_z(&mut s, 0.0)?;
    }
    Ok(s)
}

/// A wedge: a right triangle (legs `width` along x, `height` along y)
/// extruded `depth` along z.
pub fn wedge(width: f64, height: f64, depth: f64) -> Result<Solid, GenError> {
    wedge_with(width, height, depth, false)
}

pub(crate) fn wedge_with(
    width: f64,
    height: f64,
    depth: f64,
    uvs: bool,
) -> Result<Solid, GenError> {
    let (w, h) = (positive(width, "width")?, positive(height, "height")?);
    let p = Profile::from_contour(Contour::corners(vec![
        DVec2::ZERO,
        DVec2::new(w, 0.0),
        DVec2::new(0.0, h),
    ]));
    extrude_with(
        &p,
        &Extrude {
            height: depth,
            ..Default::default()
        },
        uvs,
    )
}
