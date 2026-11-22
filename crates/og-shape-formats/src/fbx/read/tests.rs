use super::*;
use og_shape::tools::import::no_files;

const CUBE: &str = r#"; FBX 7.4.0 project file
FBXHeaderExtension:  {
	FBXVersion: 7400
}
GlobalSettings:  {
	Properties70:  {
		P: "UpAxis", "int", "Integer", "",2
		P: "UpAxisSign", "int", "Integer", "",1
		P: "FrontAxis", "int", "Integer", "",1
		P: "FrontAxisSign", "int", "Integer", "",-1
		P: "CoordAxis", "int", "Integer", "",0
		P: "CoordAxisSign", "int", "Integer", "",1
		P: "UnitScaleFactor", "double", "Number", "",0.1
	}
}
Objects:  {
	Geometry: 10, "Geometry::Quad", "Mesh" {
		Vertices: *12 {
			a: 0,0,0,1,0,0,1,1,0,0,1,0
		}
		PolygonVertexIndex: *4 {
			a: 0,1,2,-4
		}
		LayerElementUV: 0 {
			MappingInformationType: "ByPolygonVertex"
			ReferenceInformationType: "IndexToDirect"
			UV: *8 {
				a: 0,0,1,0,1,1,0,1
			}
			UVIndex: *4 {
				a: 0,1,2,3
			}
		}
		LayerElementColor: 0 {
			MappingInformationType: "ByVertice"
			ReferenceInformationType: "IndexToDirect"
			Colors: *8 {
				a: 1,0,0,1,0,0,1,0.5
			}
			ColorIndex: *4 {
				a: 0,0,1,1
			}
		}
	}
	Model: 20, "Model::Plate", "Mesh" {
		Properties70:  {
			P: "Lcl Translation", "Lcl Translation", "", "A",0,0,5
			P: "Lcl Rotation", "Lcl Rotation", "", "A",0,0,90
		}
	}
}
Connections:  {
	C: "OO",20,0
	C: "OO",10,20
}
"#;

#[test]
fn reads_an_ascii_quad_with_z_up_millimetres() {
    let imported = import_fbx(CUBE.as_bytes(), &no_files).unwrap();
    assert_eq!(imported.parts.len(), 1);
    assert_eq!(imported.polygons_triangulated, 1);
    let p = &imported.parts[0];
    assert_eq!(p.name, "Plate");
    // Units of 0.1 cm (1 mm), Z up: kernel coordinates are the file's.
    let s = og_shape::tools::repair::apply_transform(p).unwrap();
    let mut pts = s.positions.clone();
    for q in &mut pts {
        *q = DVec3::new(q.x.round(), q.y.round(), q.z.round()) + DVec3::ZERO;
    }
    pts.sort_by(|a, b| og_shape::math::order::cmp_dvec3(*a, *b));
    // Turned 90° about z, lifted 5 mm.
    assert_eq!(
        pts,
        [
            DVec3::new(-1.0, 0.0, 5.0),
            DVec3::new(-1.0, 1.0, 5.0),
            DVec3::new(0.0, 0.0, 5.0),
            DVec3::new(0.0, 1.0, 5.0),
        ]
    );
    // The quad faces +z still.
    let m = s.measure();
    assert!((m.area - 1.0).abs() < 1e-9);
    let [a, b, c] = s.triangles[0].map(|v| s.positions[v as usize]);
    assert!((b - a).cross(c - a).z > 0.0);
    assert_eq!(p.solid.uvs.as_ref().map(Vec::len), Some(2));
    // The colour layer, by vertex through its index: vertices 0 and
    // 1 red, 2 and 3 blue at half alpha.
    let colors = p.solid.colors.as_ref().unwrap();
    for (t, c) in p.solid.triangles.iter().zip(colors) {
        for (v, rgba) in t.iter().zip(c) {
            let want = if *v < 2 {
                [1.0, 0.0, 0.0, 1.0]
            } else {
                [0.0, 0.0, 1.0, 0.5]
            };
            assert_eq!(*rgba, want, "vertex {v}");
        }
    }
    assert!(imported.skipped.is_empty(), "{:?}", imported.skipped);
}

/// Two legs' worth of FBX skin and animation, as exporters write it:
/// each cluster's `TransformLink` is its bone in the world at bind
/// time and its `Transform` the mesh in the bone's space (here the
/// bone's inverse, the mesh standing at the origin). The knee turns a
/// quarter turn about y in one second.
const LEGS: &str = r#"; FBX 7.4.0 project file
FBXHeaderExtension:  {
	FBXVersion: 7400
}
GlobalSettings:  {
	Properties70:  {
		P: "UpAxis", "int", "Integer", "",2
		P: "UpAxisSign", "int", "Integer", "",1
		P: "FrontAxis", "int", "Integer", "",1
		P: "FrontAxisSign", "int", "Integer", "",-1
		P: "CoordAxis", "int", "Integer", "",0
		P: "CoordAxisSign", "int", "Integer", "",1
		P: "UnitScaleFactor", "double", "Number", "",0.1
	}
}
Objects:  {
	Geometry: 10, "Geometry::Leg", "Mesh" {
		Vertices: *12 {
			a: 0,0,0,100,0,0,100,0,500,0,0,500
		}
		PolygonVertexIndex: *4 {
			a: 0,1,2,-4
		}
	}
	Model: 20, "Model::Legs", "Mesh" {
	}
	Model: 30, "Model::hip", "LimbNode" {
	}
	Model: 31, "Model::knee", "LimbNode" {
		Properties70:  {
			P: "Lcl Translation", "Lcl Translation", "", "A",0,0,250
		}
	}
	Deformer: 40, "Deformer::", "Skin" {
	}
	Deformer: 41, "SubDeformer::", "Cluster" {
		Indexes: *2 {
			a: 0,1
		}
		Weights: *2 {
			a: 1,1
		}
		Transform: *16 {
			a: 1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1
		}
		TransformLink: *16 {
			a: 1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1
		}
	}
	Deformer: 42, "SubDeformer::", "Cluster" {
		Indexes: *2 {
			a: 2,3
		}
		Weights: *2 {
			a: 1,1
		}
		Transform: *16 {
			a: 1,0,0,0,0,1,0,0,0,0,1,0,0,0,-250,1
		}
		TransformLink: *16 {
			a: 1,0,0,0,0,1,0,0,0,0,1,0,0,0,250,1
		}
	}
	AnimationStack: 50, "AnimStack::bend", "" {
	}
	AnimationLayer: 51, "AnimLayer::base", "" {
	}
	AnimationCurveNode: 52, "AnimCurveNode::R", "" {
	}
	AnimationCurve: 53, "AnimCurve::", "" {
		KeyTime: *2 {
			a: 0,46186158000
		}
		KeyValueFloat: *2 {
			a: 0,90
		}
	}
}
Connections:  {
	C: "OO",20,0
	C: "OO",10,20
	C: "OO",30,0
	C: "OO",31,30
	C: "OO",40,10
	C: "OO",41,40
	C: "OO",42,40
	C: "OO",30,41
	C: "OO",31,42
	C: "OO",51,50
	C: "OO",52,51
	C: "OP",52,31, "Lcl Rotation"
	C: "OP",53,52, "d|Y"
}
"#;

#[test]
fn reads_a_skin_s_bones_where_they_stand_and_its_animation() {
    let imported = import_fbx(LEGS.as_bytes(), &no_files).unwrap();
    let part = &imported.parts[0];
    let skin = part.solid.skin.as_ref().expect("a skin");
    let names: Vec<&str> = skin.bones.iter().map(|b| b.name.as_str()).collect();
    assert_eq!(names, ["hip", "knee"]);
    // The knee stands at its `TransformLink`, not at twice it (which
    // reading `Transform` as the mesh in the world gave).
    assert_eq!(skin.bones[0].rest.origin, DVec3::ZERO);
    assert_eq!(skin.bones[1].rest.origin, DVec3::new(0.0, 0.0, 250.0));
    assert_eq!(skin.bones[1].parent, Some(0));
    assert_eq!(part.clips.len(), 1);
    let clip = &part.clips[0];
    assert_eq!((clip.name.as_str(), clip.duration()), ("bend", 1.0));
    let top = |time: f64| {
        let i = part
            .solid
            .positions
            .iter()
            .position(|p| (*p - DVec3::new(0.0, 0.0, 500.0)).length() < 1e-9)
            .unwrap();
        let s = og_shape::skin::pose_by(&part.solid, &clip.at(time).unwrap()).unwrap();
        s.positions[i]
    };
    let near = |a: DVec3, b: DVec3| assert!((a - b).length() < 0.05, "{a:?} {b:?}");
    near(top(0.0), DVec3::new(0.0, 0.0, 500.0));
    // About +y, +z turns towards +x: 45° halfway, a quarter at the end.
    let d = 250.0 * trig::sin(trig::PI / 4.0);
    near(top(0.5), DVec3::new(d, 0.0, 250.0 + d));
    near(top(1.0), DVec3::new(250.0, 0.0, 250.0));
}

/// A second layer on the knee's bend (a steady 30° about the same
/// axis) lies on the first as its blend mode and weight say: added
/// (120° at the end), added at half weight (105°), replacing (30°),
/// and mixed half way (60°); added angle by angle it is 120° too.
#[test]
fn an_animation_s_layers_lie_on_each_other() {
    let with_layer = |props: &str| {
        let objects = format!(
            "\tAnimationLayer: 60, \"AnimLayer::over\", \"\" {{\n\t\tProperties70:  {{\n{props}\t\t}}\n\t}}\n\
             \tAnimationCurveNode: 61, \"AnimCurveNode::R\", \"\" {{\n\t}}\n\
             \tAnimationCurve: 62, \"AnimCurve::\", \"\" {{\n\t\tKeyTime: *2 {{\n\t\t\ta: 0,46186158000\n\t\t}}\n\
             \t\tKeyValueFloat: *2 {{\n\t\t\ta: 30,30\n\t\t}}\n\t}}\n}}\nConnections:  {{"
        );
        let links = "\tC: \"OP\",53,52, \"d|Y\"\n\tC: \"OO\",60,50\n\tC: \"OO\",61,60\n\
                     \tC: \"OP\",61,31, \"Lcl Rotation\"\n\tC: \"OP\",62,61, \"d|Y\"\n";
        let text = LEGS
            .replace("}\nConnections:  {", &objects)
            .replace("\tC: \"OP\",53,52, \"d|Y\"\n", links);
        assert_ne!(text.len(), LEGS.len());
        text
    };
    let p = |name: &str, kind: &str, value: i64| {
        format!("\t\t\tP: \"{name}\", \"{kind}\", \"\", \"\",{value}\n")
    };
    let cases = [
        (String::new(), 120.0),
        (p("Weight", "Number", 50), 105.0),
        (p("BlendMode", "enum", 1), 30.0),
        (p("BlendMode", "enum", 2) + &p("Weight", "Number", 50), 60.0),
        (p("RotationAccumulationMode", "enum", 1), 120.0),
    ];
    // The layer's weight driven from 0 to 100 over the second: at
    // the end it is whole.
    let driven = with_layer("")
        .replace(
            "}\nConnections:  {",
            "\tAnimationCurveNode: 63, \"AnimCurveNode::Weight\", \"\" {\n\t}\n\
             \tAnimationCurve: 64, \"AnimCurve::\", \"\" {\n\t\tKeyTime: *2 {\n\t\t\ta: 0,46186158000\n\t\t}\n\
             \t\tKeyValueFloat: *2 {\n\t\t\ta: 0,100\n\t\t}\n\t}\n}\nConnections:  {",
        )
        .replace(
            "\tC: \"OO\",60,50\n",
            "\tC: \"OO\",60,50\n\tC: \"OP\",63,60, \"Weight\"\n\tC: \"OP\",64,63, \"d|Weight\"\n",
        );
    let imported = import_fbx(driven.as_bytes(), &no_files).unwrap();
    let part = &imported.parts[0];
    let i = part
        .solid
        .positions
        .iter()
        .position(|p| (*p - DVec3::new(0.0, 0.0, 500.0)).length() < 1e-9)
        .unwrap();
    for (time, degrees) in [(0.0, 0.0), (0.5, 45.0 + 15.0), (1.0, 120.0)] {
        let at = part.clips[0].at(time).unwrap();
        let top = og_shape::skin::pose_by(&part.solid, &at).unwrap().positions[i];
        let (sin, cos) = trig::sin_cos(trig::to_radians(degrees));
        let want = DVec3::new(250.0 * sin, 0.0, 250.0 + 250.0 * cos);
        assert!(
            (top - want).length() < 0.05,
            "weight at {time}: {top:?} for {want:?}"
        );
    }
    for (props, degrees) in cases {
        let imported = import_fbx(with_layer(&props).as_bytes(), &no_files).unwrap();
        assert!(
            imported.skipped.iter().all(|s| !s.contains("layers")),
            "{:?}",
            imported.skipped
        );
        let part = &imported.parts[0];
        let i = part
            .solid
            .positions
            .iter()
            .position(|p| (*p - DVec3::new(0.0, 0.0, 500.0)).length() < 1e-9)
            .unwrap();
        let clip = &part.clips[0];
        let top = og_shape::skin::pose_by(&part.solid, &clip.at(1.0).unwrap())
            .unwrap()
            .positions[i];
        let (sin, cos) = trig::sin_cos(trig::to_radians(degrees));
        let want = DVec3::new(250.0 * sin, 0.0, 250.0 + 250.0 * cos);
        assert!(
            (top - want).length() < 0.05,
            "{props:?}: {top:?} for {want:?}"
        );
    }
}

/// The first curve of a file that moves, as [`Curve`] reads it.
fn moving_curve(bytes: &[u8]) -> Curve {
    fn find(n: &Node) -> Option<Curve> {
        if n.name == "AnimationCurve" {
            let v = n.array("KeyValueFloat")?;
            if v.iter().any(|x| (x - v[0]).abs() > 1e-3) {
                let t = n.array("KeyTime")?;
                return Some(Curve::new(
                    t.iter().map(|x| x / 46_186_158_000.0).collect(),
                    v.to_vec(),
                    n.array("KeyAttrFlags")?,
                    n.array("KeyAttrDataFloat")?,
                    n.array("KeyAttrRefCount")?,
                ));
            }
        }
        n.children.iter().find_map(find)
    }
    find(&parse_binary(bytes).unwrap().1).unwrap()
}

/// Curves Maya wrote, against the values Maya gives them at 24 frames
/// a second (ufbx's test files and tables, `third_party/ufbx`): keys
/// that hold, run straight, and ease by automatic, clamped, broken
/// and weighted tangents.
#[test]
fn eased_curves_are_what_maya_evaluates() {
    macro_rules! file {
        ($name:literal) => {
            include_bytes!(concat!("../../../../../third_party/ufbx/data/", $name)).as_slice()
        };
    }
    #[rustfmt::skip]
    let modes = [
        -8.653366,
        -8.653366,-8.602998,-8.464664,-8.257528,-8.00075,-7.713489,-7.414906,-7.124163,-6.86042,
        -6.642837,-6.490576,-6.388305,-6.306414,-6.242637,-6.19471,-6.160368,-6.137348,-6.123385,
        -6.116215,-6.113573,-6.113196,-5.969524,-5.825851,-5.682179,-5.538507,-5.394835,-5.251163,
        -5.107491,-4.963819,-4.820146,-4.676474,-4.532802,-4.38913,-4.245458,-4.101785,-3.958113,-4.1529,
        -4.347686,-4.542472,-4.737258,-4.932045,-5.126832,-5.321618,-5.516404,-5.71119,-5.905977,-5.767788,
        -5.315578,-4.954943,-4.83559,-4.856855,-4.960766,-5.118543,-4.976541,-4.885909,-4.865979,-4.93845,
        -5.099224,-5.270246,-5.359269,-5.349404,-5.261964,-5.118543,-5.264501,-5.33535,-5.285445,-5.058857,
        -4.69383,-4.357775,-4.124978,-3.981697,-3.904232,-3.875225,-3.875225,-3.875225,-3.875225,-3.875225,
        -3.875225,-3.875225,-2.942738,-2.942738,-2.942738,-2.942738,-2.942738,-2.942738,-2.942738,-2.942738,
        -2.942738,-1.243537,-1.243537,-1.243537,-1.243537,-1.243537,-1.243537,-1.243537,5.603338,5.603338,
        5.603338,5.603338,5.603338,5.603338,5.603338,5.603338,5.603338,5.603338,5.603338,5.603338,5.603338,
        5.603338,5.603338,5.603338,5.603338,5.603338,5.603338,5.603338,5.603338,5.603338,5.603338,5.603338,5.603338,
    ];
    #[rustfmt::skip]
    let clamp = [
        0.000, 0.000, 0.273, 0.515, 0.718, 0.868, 0.945, 0.920, 0.779, 0.611,
        0.591, 0.747, 1.206, 2.059, 3.191, 4.489, 5.837, 7.121, 8.228, 9.042,
        9.449, 9.694, 10.128, 10.610, 10.873, 10.927, 10.854, 10.704, 10.502,
        10.264, 10.000,
    ];
    #[rustfmt::skip]
    let spline = [
        0.00000, 3.50000, 6.00000, 6.51562, 6.37500, 6.04688, 6.00000,
        6.67313, 7.71167, 8.39438, 8.00000, 3.63667, -1.04000, -1.96474,
        -1.68059, -1.00000, 0.27700, 2.00000,
    ];
    // The first table is read a moment after each frame, so a key
    // that holds has given way to the next.
    let cases: [(&[u8], &[f64], f64, f64); 3] = [
        (
            file!("maya_interpolation_modes_7500_binary.fbx"),
            &modes,
            1e-6,
            1e-4,
        ),
        (file!("maya_auto_clamp_7700_binary.fbx"), &clamp, 0.0, 1e-3),
        (
            file!("maya_tangent_spline_7700_binary.fbx"),
            &spline,
            0.0,
            1e-4,
        ),
    ];
    for (k, (bytes, values, after, within)) in cases.into_iter().enumerate() {
        let curve = moving_curve(bytes);
        for (frame, want) in values.iter().enumerate() {
            let got = curve.at(frame as f64 / 24.0 + after);
            assert!(
                (got - want).abs() < within,
                "case {k}, frame {frame}: {got} for {want}"
            );
        }
    }
}

/// A cube Maya gave two blend shapes and animated (ufbx's test file
/// and its expected values, `third_party/ufbx`): each channel is a
/// morph target with the offsets and rest weight Maya has, and the
/// clip drives the two weights as Maya evaluates them.
#[test]
fn blend_shapes_are_morph_targets_with_maya_s_weights() {
    let bytes = include_bytes!(
        "../../../../../third_party/ufbx/data/maya_blend_shape_cube_7700_binary.fbx"
    );
    let imported = import_fbx(bytes, &no_files).unwrap();
    assert!(imported.skipped.is_empty(), "{:?}", imported.skipped);
    let part = &imported.parts[0];
    let m = part.solid.morphs.as_ref().expect("targets");
    m.check(part.solid.positions.len()).unwrap();
    let names: Vec<&str> = m.targets.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, ["Squish.TopH", "Squish.TopV"]);
    assert!(m.targets.iter().all(|t| (t.weight - 1.0).abs() < 1e-6));
    // The file's centimetres, Y up: 0.317 along x is 3.17 mm along
    // x, and 0.284 along z is 2.84 mm along −y.
    let (h, v) = (3.17, 2.84);
    let want = [
        [
            [0.0; 3],
            [0.0; 3],
            [h, 0.0, 0.0],
            [-h, 0.0, 0.0],
            [h, 0.0, 0.0],
            [-h, 0.0, 0.0],
            [0.0; 3],
            [0.0; 3],
        ],
        [
            [0.0; 3],
            [0.0; 3],
            [0.0, v, 0.0],
            [0.0, v, 0.0],
            [0.0, -v, 0.0],
            [0.0, -v, 0.0],
            [0.0; 3],
            [0.0; 3],
        ],
    ];
    for (t, want) in m.targets.iter().zip(want) {
        for (d, w) in t.deltas.iter().zip(want) {
            assert!(
                (*d - DVec3::from_array(w)).length() < 0.01,
                "{}: {d:?} for {w:?}",
                t.name
            );
        }
    }
    // Maya's weights at frames 16 and 53 (the clip starts at frame 1).
    let clip = &part.clips[0];
    for (frame, want) in [
        (1.0, [1.0, 1.0]),
        (16.0, [0.279, 0.670]),
        (53.0, [0.901, 0.168]),
    ] {
        let got = clip.weights_at((frame - 1.0) / 24.0).unwrap();
        for k in 0..2 {
            assert!(
                (got[k] - want[k]).abs() < 2e-3,
                "frame {frame}: {got:?} for {want:?}"
            );
        }
    }
}

/// A cube with no skin that Maya moved along x by a curve (ufbx's
/// file again): it is read as a skin of one bone, and the clip puts
/// it where Maya's values say, frame by frame.
#[test]
fn a_model_s_animation_moves_its_mesh_as_one_bone() {
    let bytes = include_bytes!(
        "../../../../../third_party/ufbx/data/maya_interpolation_modes_7500_binary.fbx"
    );
    let imported = import_fbx(bytes, &no_files).unwrap();
    let part = &imported.parts[0];
    let skin = part.solid.skin.as_ref().expect("one bone");
    assert_eq!(skin.bones.len(), 1);
    assert!(skin.bones[0].name.ends_with(".node"));
    let clip = &part.clips[0];
    let centre = |s: &Solid| s.positions.iter().sum::<DVec3>() / s.positions.len() as f64;
    // Maya's x at frames 1, 11, 21, 36, 46 and 60 (the file's
    // centimetres; the clip starts at frame 1).
    for (frame, x) in [
        (1.0, -8.653366),
        (11.0, -6.490576),
        (21.0, -6.113196),
        (36.0, -3.958113),
        (46.0, -5.905977),
        (60.0, -5.359269),
    ] {
        let at = clip.at((frame - 1.0) / 24.0).unwrap();
        let posed = og_shape::skin::pose_by(&part.solid, &at).unwrap();
        // Between its samples (sixty a second) a clip runs straight.
        assert!(
            (centre(&posed).x - x * 10.0).abs() < 0.05,
            "frame {frame}: {:?}",
            centre(&posed)
        );
    }
}

/// A channel with shapes on the way to its full one (ufbx's file,
/// and three frames of it as Maya wrote them out,
/// `third_party/ufbx`): each shape is a target, the clip gives each
/// its part at every time, and the mesh then stands where Maya's
/// does.
#[test]
fn in_between_shapes_are_targets_of_their_own() {
    let bytes =
        include_bytes!("../../../../../third_party/ufbx/data/maya_blend_inbetween_7500_binary.fbx");
    let imported = import_fbx(bytes, &no_files).unwrap();
    assert!(imported.skipped.is_empty(), "{:?}", imported.skipped);
    let part = &imported.parts[0];
    let rest = og_shape::tools::repair::apply_transform(part).unwrap();
    let m = rest.morphs.as_ref().expect("targets");
    let names: Vec<&str> = m.targets.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "KeyBlend.LiftTop@50",
            "KeyBlend.LiftTop",
            "KeyBlend.MoveToSide@-49.9",
            "KeyBlend.MoveToSide@-24.9",
            "KeyBlend.MoveToSide"
        ]
    );
    let clip = &part.clips[0];
    let frames: [(f64, &str); 3] = [
        (
            30.0,
            include_str!("../../../../../third_party/ufbx/data/maya_blend_inbetween_30.obj"),
        ),
        (
            65.0,
            include_str!("../../../../../third_party/ufbx/data/maya_blend_inbetween_65.obj"),
        ),
        (
            89.0,
            include_str!("../../../../../third_party/ufbx/data/maya_blend_inbetween_89.obj"),
        ),
    ];
    for (frame, obj) in frames {
        let weights = clip.weights_at((frame - 1.0) / 24.0).unwrap();
        let posed = rest.morphed(&weights);
        // Maya's corners, in its centimetres with Y up.
        let want: Vec<DVec3> = obj
            .lines()
            .filter_map(|l| l.strip_prefix("v "))
            .map(|l| {
                let v: Vec<f64> = l.split_whitespace().map(|x| x.parse().unwrap()).collect();
                DVec3::new(v[0], -v[2], v[1]) * 10.0
            })
            .collect();
        assert_eq!(want.len(), posed.positions.len());
        for (p, w) in posed.positions.iter().zip(&want) {
            assert!((*p - *w).length() < 0.02, "frame {frame}: {p:?} for {w:?}");
        }
    }
}
