#![allow(missing_docs, clippy::disallowed_types)]
//! Determinism: the same input builds the same bytes, twice in one run, on
//! every host (the gate's `shape-wasm32` step runs this module on
//! wasm32-wasip1 against the bits recorded natively), and as the
//! TypeScript runtime flattens the same paths. A recorded model program
//! replays to the same GLB.

use glam::{DVec2, DVec3};

use og_core::math::double;
use og_shape::boolean::*;
use og_shape::generators::extrude::ExtrudePath;
use og_shape::generators::*;
use og_shape::model::program::{Session, run};
use og_shape_formats::ModelFiles;

/// A box of three.js `BoxGeometry(w, h, d)`'s layout: four vertices a face,
/// two triangles each, counter-clockwise seen from outside.
fn boxed(w: f32, h: f32, d: f32) -> Mesh {
    let (x, y, z) = (w / 2.0, h / 2.0, d / 2.0);
    let faces: [([f32; 3], [f32; 3], [f32; 3]); 6] = [
        ([1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0, 1.0, 0.0]),
        ([-1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]),
        ([0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0]),
        ([0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
        ([0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
        ([0.0, 0.0, -1.0], [-1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
    ];
    let mut m = Mesh::default();
    for (n, u, v) in faces {
        let base = m.positions.len() as u32;
        for (su, sv) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
            let p: [f32; 3] =
                std::array::from_fn(|k| (n[k] + u[k] * su + v[k] * sv) * [x, y, z][k]);
            m.positions.push(p);
            m.normals.push(n);
            m.uvs.push([(su + 1.0) / 2.0, (1.0 - sv) / 2.0]);
        }
        m.indices
            .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    m
}

/// A sphere as a lathe of a half circle (closed: the poles and the seam
/// meet exactly).
fn sphere(r: f64, segments: u32, rows: u32) -> Mesh {
    let points = (0..=rows)
        .map(|j| {
            let (s, c) = double::sin_cos(j as f64 / rows as f64 * double::PI);
            DVec2::new(if j == 0 || j == rows { 0.0 } else { r * s }, -r * c)
        })
        .collect();
    lathe(&Lathe {
        points,
        segments,
        ..Default::default()
    })
}

fn circle(cx: f64, cy: f64, r: f64, n: usize, clockwise: bool) -> Vec<DVec2> {
    (0..n)
        .map(|i| {
            let a = i as f64 / n as f64 * double::TAU;
            let (s, c) = double::sin_cos(if clockwise { -a } else { a });
            DVec2::new(cx + r * c, cy + r * s)
        })
        .collect()
}

/// A gear: `teeth` trapezoid teeth round a disc, a round hole in the middle.
fn gear(teeth: usize) -> ShapePoints {
    let mut points = Vec::new();
    for i in 0..teeth {
        for (f, r) in [(0.0, 1.0), (0.2, 1.25), (0.45, 1.25), (0.65, 1.0)] {
            let a = (i as f64 + f) / teeth as f64 * double::TAU;
            let (s, c) = double::sin_cos(a);
            points.push(DVec2::new(r * c, r * s));
        }
    }
    ShapePoints {
        points,
        holes: vec![circle(0.0, 0.0, 0.35, 24, true)],
    }
}

fn fnv(bits: &[u32]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for b in bits {
        for byte in b.to_le_bytes() {
            h ^= byte as u64;
            h = h.wrapping_mul(0x100_0000_01b3);
        }
    }
    h
}

// ── determinism ─────────────────────────────────────────────────────────────

fn everything() -> Vec<Mesh> {
    let g = gear(10);
    let bevelled = extrude(std::slice::from_ref(&g), &Extrude::default(), None);
    let vase = lathe(&Lathe {
        points: (0..12)
            .map(|j| {
                DVec2::new(
                    0.3 + 0.2 * double::sin_cos(j as f64 * 0.5).0,
                    j as f64 * 0.1,
                )
            })
            .collect(),
        segments: 24,
        ..Default::default()
    });
    let sample = |t: f64| {
        let (s, c) = double::sin_cos(t * 2.0);
        (
            DVec3::new(c, t, s),
            DVec3::new(-2.0 * s, 1.0, 2.0 * c).normalize(),
        )
    };
    let horn = sweep(
        &Sweep {
            section: circle(0.0, 0.0, 0.3, 12, false),
            steps: 20,
            scale: vec![Key { t: 0.0, value: 1.0 }, Key { t: 1.0, value: 0.1 }],
            ..Default::default()
        },
        &ExtrudePath {
            sample: &sample,
            closed: false,
        },
    )
    .unwrap();
    let dice = csg(
        &boxed(1.0, 1.0, 1.0),
        Placement::default(),
        &sphere(0.2, 16, 8),
        Placement {
            position: DVec3::new(0.5, 0.1, 0.05),
            ..Default::default()
        },
        CsgOp::Subtract,
    )
    .unwrap();
    vec![
        bevelled,
        vase,
        horn,
        icosahedron(1.0, 2),
        dodecahedron(1.0, 1),
        dice,
    ]
}

#[test]
fn two_builds_are_byte_identical() {
    let (a, b) = (everything(), everything());
    for (x, y) in a.iter().zip(&b) {
        assert_eq!(x.bits(), y.bits());
    }
    // The bits themselves, recorded on x86_64 Linux: a host that rounds
    // differently (a fused multiply-add, a libm sine) fails here.
    let hashes: Vec<String> = a
        .iter()
        .map(|m| format!("{:016x}", fnv(&m.bits())))
        .collect();
    assert_eq!(
        hashes,
        [
            "bd3a1315442aefbe",
            "5c426bd443c4462a",
            "b7ddbf14c9c171e7",
            "dd8e55eccf0c7012",
            "a6bf8294bc6c52f2",
            "c205aaae210975cd"
        ],
        "extrude, lathe, sweep, icosahedron, dodecahedron, csg"
    );
}

// ── og.Path2 and og.Shape flattened in Rust, as the TypeScript runtime does ─

fn fnv_points(points: &[DVec2]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for p in points {
        for v in [p.x, p.y] {
            for byte in v.to_le_bytes() {
                h ^= byte as u64;
                h = h.wrapping_mul(0x100_0000_01b3);
            }
        }
    }
    h
}

/// The same calls as packages/engine/src/geom.ts's og.Path2 and og.Shape
/// make, with the counts and FNV hashes of the f64 points bun printed for
/// them (angles written as numbers, as a `type model` part names them): a
/// `.ogt` shape and a TypeScript one flatten to the same bits.
// The angles are written as numbers on purpose: a `.ogt` names them so.
#[allow(clippy::approx_constant)]
#[test]
fn path2_flattens_to_the_typescript_runtime_s_points() {
    let v = |x: f64, y: f64| DVec2::new(x, y);
    let mut arcs = Path2::new();
    arcs.move_to(1.0, 0.0)
        .absarc(
            0.0,
            0.0,
            1.0,
            -0.7853981633974483,
            3.9269908169872414,
            false,
        )
        .line_to(0.2, -0.4)
        .arc(0.1, 0.1, 0.3, 0.0, 3.141592653589793, true);
    let mut curves = Path2::new();
    curves
        .move_to(0.0, 0.0)
        .quadratic_curve_to(0.5, 1.0, 1.0, 0.0)
        .bezier_curve_to(v(1.2, -0.5), v(1.8, 0.4), 2.0, 0.0)
        .spline_thru(&[v(2.5, 0.5), v(3.0, -0.2), v(3.4, 0.1)])
        .close_path();
    let mut ellipse = Path2::new();
    ellipse
        .absellipse(v(0.5, 0.25), v(2.0, 1.0), 0.3, -0.3, true, 0.7)
        .ellipse(
            v(1.0, 1.0),
            v(0.5, 0.25),
            -0.7853981633974483,
            0.7853981633974483,
            false,
            0.0,
        );
    let mut shape = Shape::default();
    shape
        .path
        .absarc(0.0, 0.0, 1.0, 0.0, 6.283185307179586, false);
    let mut hole = Path2::new();
    hole.absarc(0.0, 0.0, 0.3, 0.0, 6.283185307179586, true);
    shape.holes.push(hole);
    let e = shape.extract_points(12);
    let mut holed = e.points.clone();
    holed.extend_from_slice(&e.holes[0]);
    let mut from_points = Path2::new();
    from_points.set_from_points(&[v(0.0, 0.0), v(1.0, 0.0), v(1.0, 1.0), v(1.0, 1.0)]);
    let got: Vec<(usize, String)> = [
        arcs.get_points(5),
        curves.get_points(7),
        ellipse.get_points(12),
        holed,
        from_points.get_points(3),
    ]
    .iter()
    .map(|p| (p.len(), format!("{:016x}", fnv_points(p))))
    .collect();
    let want: Vec<(usize, String)> = [
        (23, "2c62b7366a66b61a"),
        (44, "33e053fbf600b013"),
        (50, "e4828e5598c0691d"),
        (50, "051e04a684644f88"),
        (3, "328e239e448cbc18"),
    ]
    .iter()
    .map(|(n, h)| (*n, h.to_string()))
    .collect();
    assert_eq!(
        got, want,
        "arcs, curves, ellipse, a shape with a hole, set_from_points"
    );
}

#[test]
fn a_recorded_program_replays_to_the_same_bytes() {
    let mut s = Session::new("replay");
    let ap = |s: &mut Session, op: &str| -> serde_json::Value {
        serde_json::from_str(&s.apply(op).unwrap()).unwrap()
    };
    let cab = ap(
        &mut s,
        r#"{"op":"box","name":"cab","width":100,"depth":80,"height":120}"#,
    );
    s.apply(r#"{"op":"fillet","id":0,"radius":4,"edges":"side"}"#)
        .unwrap();
    let front = ap(&mut s, r#"{"op":"face","id":0,"name":"+X"}"#);
    let porthole = ap(
        &mut s,
        &format!(r#"{{"op":"translate","frame":{front},"y":10}}"#),
    );
    s.apply(&format!(
        r#"{{"op":"pocket","id":0,"profile":{{"circle":{{"radius":20}}}},"depth":30,"at":[{porthole}]}}"#
    ))
    .unwrap();
    let lid = ap(
        &mut s,
        &format!(r#"{{"op":"cylinder","name":"lid","radius":24,"height":4,"on":{porthole}}}"#),
    );
    s.apply(&format!(
        r##"{{"op":"material","ids":[{lid}],"color":"#3a3d42","metalness":0.5}}"##
    ))
    .unwrap();
    s.apply(&format!(
        r#"{{"op":"joint","child":{lid},"parent":{cab},"name":"door","on":{porthole},"axis":"-Z","range":[0,1.5]}}"#
    ))
    .unwrap();
    s.apply(r#"{"op":"pose","joint":"door","value":1}"#)
        .unwrap();
    let program = s.program();
    // Readers are not recorded.
    assert!(!program.contains(r#""op":"face""#));
    let replayed = run(&program).unwrap();
    assert_eq!(replayed.glb().unwrap(), s.model().glb().unwrap());
    // Errors are values, with the operation and field named.
    let e = s.apply(r#"{"op":"box","name":"x","width":1}"#).unwrap_err();
    assert!(e.to_string().contains("depth"), "{e}");
    let e = s
        .apply(r#"{"op":"face","id":0,"name":"nope"}"#)
        .unwrap_err();
    assert!(e.to_string().contains("top"), "{e}");
}
