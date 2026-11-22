//! FBX through the one reader, the cook's ufbx import, as the commands
//! read it: its glTF imported by og-shape-formats. Native only (ufbx is C).

#[allow(dead_code)]
#[path = "fbx_support.rs"]
mod support;

use og_shape::tools::{Import, join, no_files};
use og_shape_formats::import;

fn read(bytes: &[u8]) -> og_shape::tools::Imported {
    let glb = og_cook::cook::fbx::to_glb(bytes, std::path::Path::new("case.fbx"), "fbx")
        .unwrap()
        .glb;
    import(&glb, &Import::default(), &no_files).unwrap()
}

/// A binary FBX: a 2 cm cube scaled 2, turned 90° about y around the
/// pivot x = 1 cm, moved 10 cm in x by its parent, zlib arrays, two
/// materials and an embedded PNG. In kernel space (x, −z, y) mm:
/// x ∈ [110, 130], y ∈ [−10, 10], z ∈ [0, 20].
#[test]
fn a_binary_fbx_keeps_its_pivots_parents_and_texture() {
    let imported = read(&support::binary_fbx());
    let m = join(&imported.parts).unwrap().measure();
    assert!(m.closed);
    // f32 in the glTF: a micrometre.
    let near = |a: f64, b: f64| (a - b).abs() < 1e-3;
    assert!((m.volume - 8000.0).abs() < 1e-2, "{m:?}");
    assert!(
        near(m.bounds_min.x, 110.0) && near(m.bounds_max.x, 130.0),
        "{m:?}"
    );
    assert!(
        near(m.bounds_min.y, -10.0) && near(m.bounds_max.y, 10.0),
        "{m:?}"
    );
    assert!(
        near(m.bounds_min.z, 0.0) && near(m.bounds_max.z, 20.0),
        "{m:?}"
    );
    assert_eq!(imported.images.len(), 1);
    assert_eq!(imported.images[0].width, Some(16));
}

/// An ASCII FBX, Z up in tenths of a centimetre (millimetres), with a
/// pre-rotation, an Euler order and a geometric offset: whatever its
/// placement, its pentagon keeps its area, 16 mm².
#[test]
fn an_ascii_fbx_keeps_its_units() {
    let ascii = r#"; FBX 7.5.0 project file
FBXHeaderExtension:  {
	FBXVersion: 7500
}
GlobalSettings:  {
	Properties70:  {
		P: "UpAxis", "int", "Integer", "",2
		P: "FrontAxis", "int", "Integer", "",1
		P: "FrontAxisSign", "int", "Integer", "",-1
		P: "CoordAxis", "int", "Integer", "",0
		P: "UnitScaleFactor", "double", "Number", "",0.1
	}
}
Objects:  {
	Geometry: 1, "Geometry::Tri", "Mesh" {
		Vertices: *15 {
			a: 0,0,0,4,0,0,4,3,0,2,5,0,0,3,0
		}
		PolygonVertexIndex: *5 {
			a: 0,1,2,3,-5
		}
	}
	Model: 2, "Model::Roof", "Mesh" {
		Properties70:  {
			P: "PreRotation", "Vector3D", "Vector", "",30,0,0
			P: "Lcl Rotation", "Lcl Rotation", "", "A",10,20,30
			P: "RotationOrder", "enum", "", "",5
			P: "GeometricTranslation", "Vector3D", "Vector", "",0,0,7
		}
	}
}
Connections:  {
	C: "OO",2,0
	C: "OO",1,2
}
"#;
    let imported = read(ascii.as_bytes());
    assert_eq!(imported.parts.len(), 1);
    let m = join(&imported.parts).unwrap().measure();
    assert_eq!(m.triangles, 3);
    assert!((m.area - 16.0).abs() < 1e-3, "{m:?}");
}
