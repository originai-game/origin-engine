//! Structured geometry calls the existing rendering and modelling kernels.
//! Model operands are supplied by the import graph, never reopened from disk.
use crate::cook::source_schemas::{reflected::Reflected, world::geometry as input};
use glam::{DQuat, DVec2, DVec3, Vec3};
use og_shape::generators::{self as shape, Mesh, Mirror, Placement};

pub(crate) type Models<'a> = dyn Fn(&str, Option<&str>) -> Result<Mesh, String> + 'a;

pub(crate) fn placement<M>(value: Option<&Reflected<M>>) -> Result<Placement, String> {
    let Some(value) = value else {
        return Ok(Placement::default());
    };
    let fields = value.fields();
    let vector = |key, default: DVec3| {
        fields.get(key).map_or(default, |v| {
            let v = v.as_array().expect("checked reflected vector");
            DVec3::new(
                v[0].as_f64().unwrap(),
                v[1].as_f64().unwrap(),
                v[2].as_f64().unwrap(),
            )
        })
    };
    let quaternion = match fields.get("quaternion") {
        None => DQuat::IDENTITY,
        Some(v) => {
            let q = v.as_array().expect("checked reflected quaternion");
            let q = DQuat::from_xyzw(
                q[0].as_f64().unwrap(),
                q[1].as_f64().unwrap(),
                q[2].as_f64().unwrap(),
                q[3].as_f64().unwrap(),
            );
            if q.length_squared() < 1e-12 {
                return Err("transform.quaternion is not a rotation (zero length)".into());
            }
            q.normalize()
        }
    };
    Ok(Placement {
        position: vector("position", DVec3::ZERO),
        quaternion,
        scale: vector("scale", DVec3::ONE),
    })
}

fn placed(value: &input::PlacedGeometry, models: &Models<'_>, detail: f64) -> Result<Mesh, String> {
    let mirror = match value.mirror {
        Some(input::Mirror::X) => Mirror {
            x: true,
            ..Default::default()
        },
        Some(input::Mirror::Y) => Mirror {
            y: true,
            ..Default::default()
        },
        Some(input::Mirror::Z) => Mirror {
            z: true,
            ..Default::default()
        },
        _ => Mirror::default(),
    };
    shape::merge(&[shape::Part {
        mesh: &build_at(&value.geometry, models, detail)?,
        placement: placement(value.transform.as_ref())?,
        mirror,
    }])
}

fn path(value: &input::Curve) -> Result<og_timeline::Path, String> {
    use input::CurveKind;
    og_timeline::Path::new(og_timeline::PathDesc {
        points: value.points.iter().copied().map(Vec3::from_array).collect(),
        kind: match value.kind {
            None | Some(CurveKind::CatmullRom) => og_timeline::PathKind::CatmullRom,
            Some(CurveKind::Bezier) => og_timeline::PathKind::Bezier,
            Some(CurveKind::Linear) => og_timeline::PathKind::Linear,
        },
        closed: value.closed.unwrap_or(false),
        tension: value.tension.unwrap_or(0.5),
    })
}

fn sample(path: &og_timeline::Path, t: f64) -> (DVec3, DVec3) {
    let s = path.sample(t as f32);
    (s.position.as_dvec3(), s.tangent.as_dvec3())
}

fn contour(commands: &[input::Path2Command]) -> shape::Path2 {
    use input::Path2Command::*;
    let mut p = shape::Path2::new();
    for command in commands {
        match command {
            MoveTo([x, y]) => {
                p.move_to(*x, *y);
            }
            LineTo([x, y]) => {
                p.line_to(*x, *y);
            }
            QuadraticCurveTo([cx, cy, x, y]) => {
                p.quadratic_curve_to(*cx, *cy, *x, *y);
            }
            BezierCurveTo([ax, ay, bx, by, x, y]) => {
                p.bezier_curve_to(DVec2::new(*ax, *ay), DVec2::new(*bx, *by), *x, *y);
            }
            SplineThru(points) | Points(points) => {
                let points: Vec<_> = points.iter().copied().map(DVec2::from_array).collect();
                if matches!(command, Points(_)) {
                    p.set_from_points(&points);
                } else {
                    p.spline_thru(&points);
                }
            }
            Arc(a) | AbsArc(a) => {
                let [x, y] = a.center;
                if matches!(command, Arc(_)) {
                    p.arc(x, y, a.radius, a.start, a.end, a.clockwise);
                } else {
                    p.absarc(x, y, a.radius, a.start, a.end, a.clockwise);
                }
            }
            Ellipse(a) | AbsEllipse(a) => {
                let c = DVec2::from_array(a.center);
                let r = DVec2::from_array(a.radii);
                if matches!(command, Ellipse(_)) {
                    p.ellipse(c, r, a.start, a.end, a.clockwise, a.rotation);
                } else {
                    p.absellipse(c, r, a.start, a.end, a.clockwise, a.rotation);
                }
            }
            ClosePath => {
                p.close_path();
            }
        }
    }
    p
}

fn shapes(
    values: Option<&[input::Shape]>,
    divisions: u32,
    default: &[[f64; 2]],
) -> Vec<shape::ShapePoints> {
    match values {
        None | Some([]) => vec![shape::ShapePoints {
            points: default.iter().copied().map(DVec2::from_array).collect(),
            holes: Vec::new(),
        }],
        Some(values) => values
            .iter()
            .map(|s| {
                shape::Shape {
                    path: contour(&s.commands),
                    holes: s.holes.iter().map(|c| contour(c)).collect(),
                }
                .extract_points(divisions)
            })
            .collect(),
    }
}

fn from_render(m: og_render_data::MeshData) -> Mesh {
    Mesh {
        positions: m.vertices.iter().map(|v| v.position).collect(),
        normals: m.vertices.iter().map(|v| v.normal).collect(),
        uvs: m.vertices.iter().map(|v| v.uv).collect(),
        colors: if m.vertices.iter().any(|v| v.color != [1.0; 4]) {
            m.vertices.iter().map(|v| v.color).collect()
        } else {
            Vec::new()
        },
        tangents: if m.vertices.iter().any(|v| v.tangent[3] != 0.0) {
            m.vertices.iter().map(|v| v.tangent).collect()
        } else {
            Vec::new()
        },
        indices: m.indices,
    }
}

pub(crate) fn build(value: &input::Geometry, models: &Models<'_>) -> Result<Mesh, String> {
    build_at(value, models, 1.0)
}

/// Rebuild authored geometry at a face-budget factor, keeping the same floors
/// as the model cook. Terrain uses the unscaled entry above.
pub(crate) fn build_at(
    value: &input::Geometry,
    models: &Models<'_>,
    detail: f64,
) -> Result<Mesh, String> {
    use input::Geometry::*;
    use og_render_data::geometry::primitives as primitive;
    let tau = og_core::math::double::TAU;
    let scaled = |n: u32, floor: u32| {
        if detail >= 1.0 {
            n
        } else {
            ((f64::from(n) * detail).round() as u32).clamp(floor.min(n), n)
        }
    };
    let subdivisions = |n: u32| scaled(n + 1, 1) - 1;
    // Every descriptor has the same UV field; physics/dynamic are runtime
    // flags only. Collision is owned by the consuming model/terrain pipeline.
    macro_rules! repeat { ($($kind:ident),+ $(,)?) => { match value { $($kind(d) => d.uv_repeat),+ } }; }
    let repeat = repeat!(
        ModelGeometry,
        BufferGeometry,
        BoxGeometry,
        PlaneGeometry,
        SphereGeometry,
        CylinderGeometry,
        ConeGeometry,
        CapsuleGeometry,
        TorusGeometry,
        RingGeometry,
        CircleGeometry,
        TubeGeometry,
        ShapeGeometry,
        ExtrudeGeometry,
        LatheGeometry,
        PolyhedronGeometry,
        IcosahedronGeometry,
        DodecahedronGeometry,
        OctahedronGeometry,
        TetrahedronGeometry,
        SweepGeometry,
        CsgGeometry,
        MergeGeometries
    );
    let mut mesh = match value {
        ModelGeometry(d) => models(&d.source.id, d.node.as_deref())?,
        BufferGeometry(d) => from_render(og_render_data::geometry::buffer_geometry(
            &og_render_data::geometry::BufferAttributes {
                positions: &d.positions,
                normals: d.normals.as_deref(),
                uvs: d.uvs.as_deref(),
                colors: d.colors.as_deref(),
                tangents: d.tangents.as_deref(),
                indices: d.indices.as_deref(),
            },
        )?),
        BoxGeometry(d) => from_render(og_render_data::shapes::cuboid(Vec3::new(
            d.width.unwrap_or(1.0) as f32,
            d.height.unwrap_or(1.0) as f32,
            d.depth.unwrap_or(1.0) as f32,
        ))),
        PlaneGeometry(d) => from_render(og_render_data::shapes::plane_xy(
            d.width.unwrap_or(1.0) as f32,
            d.height.unwrap_or(1.0) as f32,
        )),
        SphereGeometry(d) => {
            let mut mesh = og_render_data::shapes::sphere(
                scaled(d.width_segments.unwrap_or(32), 5),
                scaled(d.height_segments.unwrap_or(16), 3),
            );
            for vertex in &mut mesh.vertices {
                for c in &mut vertex.position {
                    *c *= d.radius.unwrap_or(1.0) as f32 * 2.0;
                }
            }
            from_render(mesh)
        }
        CylinderGeometry(d) => from_render(primitive::cylinder(&primitive::Cylinder {
            radius_top: d.radius_top.unwrap_or(1.0) as f32,
            radius_bottom: d.radius_bottom.unwrap_or(1.0) as f32,
            height: d.height.unwrap_or(1.0) as f32,
            radial_segments: scaled(d.radial_segments.unwrap_or(32), 5),
            height_segments: scaled(d.height_segments.unwrap_or(1), 1),
            open_ended: d.open_ended.unwrap_or(false),
            theta_start: d.theta_start.unwrap_or(0.0) as f32,
            theta_length: d.theta_length.unwrap_or(tau) as f32,
        })),
        ConeGeometry(d) => from_render(primitive::cone(
            d.radius.unwrap_or(1.0) as f32,
            d.height.unwrap_or(1.0) as f32,
            scaled(d.radial_segments.unwrap_or(32), 5),
            scaled(d.height_segments.unwrap_or(1), 1),
            d.open_ended.unwrap_or(false),
        )),
        CapsuleGeometry(d) => from_render(primitive::capsule(
            d.radius.unwrap_or(1.0) as f32,
            d.height.unwrap_or(1.0) as f32,
            scaled(d.cap_segments.unwrap_or(4), 1),
            scaled(d.radial_segments.unwrap_or(8), 5),
            scaled(d.height_segments.unwrap_or(1), 1),
        )),
        TorusGeometry(d) => from_render(primitive::torus(
            d.radius.unwrap_or(1.0) as f32,
            d.tube.unwrap_or(0.4) as f32,
            scaled(d.radial_segments.unwrap_or(12), 3),
            scaled(d.tubular_segments.unwrap_or(48), 6),
            d.arc.unwrap_or(tau) as f32,
        )),
        RingGeometry(d) => from_render(primitive::ring(
            d.inner_radius.unwrap_or(0.5) as f32,
            d.outer_radius.unwrap_or(1.0) as f32,
            scaled(d.theta_segments.unwrap_or(32), 5),
            scaled(d.phi_segments.unwrap_or(1), 1),
            d.theta_start.unwrap_or(0.0) as f32,
            d.theta_length.unwrap_or(tau) as f32,
        )),
        CircleGeometry(d) => from_render(primitive::circle(
            d.radius.unwrap_or(1.0) as f32,
            scaled(d.segments.unwrap_or(32), 5),
            d.theta_start.unwrap_or(0.0) as f32,
            d.theta_length.unwrap_or(tau) as f32,
        )),
        TubeGeometry(d) => {
            let t = scaled(d.tubular_segments.unwrap_or(64), 6);
            let r = d.radius.unwrap_or(1.0) as f32;
            let n = scaled(d.radial_segments.unwrap_or(8), 5);
            from_render(match &d.path {
                input::TubePath::Points { points, closed } => primitive::tube(
                    &points
                        .iter()
                        .copied()
                        .map(Vec3::from_array)
                        .collect::<Vec<_>>(),
                    t,
                    r,
                    n,
                    closed.unwrap_or(false),
                ),
                input::TubePath::Curve { curve } => {
                    let p = path(curve)?;
                    primitive::tube_along(
                        |t| {
                            let s = p.sample(t);
                            (s.position, s.tangent)
                        },
                        t,
                        r,
                        n,
                        p.desc().closed,
                    )
                }
            })
        }
        ShapeGeometry(d) => shape::shape_geometry(&shapes(
            d.shapes.as_deref(),
            scaled(d.curve_segments.unwrap_or(12), 2),
            &[[0.0, 0.5], [-0.5, -0.5], [0.5, -0.5]],
        )),
        ExtrudeGeometry(d) => {
            let shapes = shapes(
                d.shapes.as_deref(),
                scaled(d.curve_segments.unwrap_or(12), 2),
                &[[0.5, 0.5], [-0.5, 0.5], [-0.5, -0.5], [0.5, -0.5]],
            );
            let thickness = d.bevel_thickness.unwrap_or(0.2);
            let options = shape::Extrude {
                depth: d.depth.unwrap_or(1.0),
                steps: d.steps.unwrap_or(1),
                bevel_enabled: d.bevel_enabled.unwrap_or(true),
                bevel_thickness: thickness,
                bevel_size: d.bevel_size.unwrap_or(thickness - 0.1),
                bevel_offset: d.bevel_offset.unwrap_or(0.0),
                bevel_segments: scaled(d.bevel_segments.unwrap_or(3), 1),
            };
            match &d.extrude_path {
                None => shape::extrude(&shapes, &options, None),
                Some(curve) => {
                    let p = path(curve)?;
                    shape::extrude(
                        &shapes,
                        &options,
                        Some(&shape::extrude::ExtrudePath {
                            sample: &|t| sample(&p, t),
                            closed: p.desc().closed,
                        }),
                    )
                }
            }
        }
        LatheGeometry(d) => shape::lathe(&shape::Lathe {
            points: d
                .points
                .as_ref()
                .map(|p| p.iter().copied().map(DVec2::from_array).collect())
                .unwrap_or_else(|| shape::Lathe::default().points),
            segments: scaled(d.segments.unwrap_or(12), 5),
            phi_start: d.phi_start.unwrap_or(0.0),
            phi_length: d.phi_length.unwrap_or(tau),
        }),
        PolyhedronGeometry(d) => shape::polyhedron(&shape::Polyhedron {
            vertices: d.vertices.clone(),
            indices: d.indices.clone(),
            radius: d.radius.unwrap_or(1.0),
            detail: subdivisions(d.detail.unwrap_or(0)),
        })?,
        IcosahedronGeometry(d) => {
            shape::icosahedron(d.radius.unwrap_or(1.0), subdivisions(d.detail.unwrap_or(0)))
        }
        DodecahedronGeometry(d) => {
            shape::dodecahedron(d.radius.unwrap_or(1.0), subdivisions(d.detail.unwrap_or(0)))
        }
        OctahedronGeometry(d) => {
            shape::octahedron(d.radius.unwrap_or(1.0), subdivisions(d.detail.unwrap_or(0)))
        }
        TetrahedronGeometry(d) => {
            shape::tetrahedron(d.radius.unwrap_or(1.0), subdivisions(d.detail.unwrap_or(0)))
        }
        SweepGeometry(d) => {
            let p = path(&d.path)?;
            let keys = |v: Option<&Vec<[f64; 2]>>| {
                v.into_iter()
                    .flatten()
                    .map(|[t, value]| shape::Key {
                        t: *t,
                        value: *value,
                    })
                    .collect()
            };
            shape::sweep(
                &shape::Sweep {
                    section: contour(&d.section)
                        .get_points(scaled(d.curve_segments.unwrap_or(12), 2)),
                    steps: scaled(d.steps.unwrap_or(64), 6),
                    scale: keys(d.scale.as_ref()),
                    twist: keys(d.twist.as_ref()),
                    caps: d.caps.unwrap_or(true),
                },
                &shape::extrude::ExtrudePath {
                    sample: &|t| sample(&p, t),
                    closed: p.desc().closed,
                },
            )?
        }
        CsgGeometry(d) => {
            let op = match d.op {
                None | Some(input::CsgOp::Union) => og_shape::boolean::CsgOp::Union,
                Some(input::CsgOp::Subtract) => og_shape::boolean::CsgOp::Subtract,
                Some(input::CsgOp::Intersect) => og_shape::boolean::CsgOp::Intersect,
            };
            og_shape::boolean::csg(
                &placed(&d.a, models, detail)?,
                Placement::default(),
                &placed(&d.b, models, detail)?,
                Placement::default(),
                op,
            )?
        }
        MergeGeometries(d) => {
            let meshes = d
                .parts
                .iter()
                .map(|v| placed(v, models, detail))
                .collect::<Result<Vec<_>, _>>()?;
            shape::merge(
                &meshes
                    .iter()
                    .map(|mesh| shape::Part {
                        mesh,
                        placement: Placement::default(),
                        mirror: Mirror::default(),
                    })
                    .collect::<Vec<_>>(),
            )?
        }
    };
    if let Some([u, v]) = repeat {
        for uv in &mut mesh.uvs {
            uv[0] *= u;
            uv[1] *= v;
        }
    }
    if mesh.indices.is_empty() || mesh.positions.iter().flatten().any(|x| !x.is_finite()) {
        return Err("geometry must contain triangles and finite positions".into());
    }
    Ok(mesh)
}
