//! A navmesh asset's G2-4 entries cooked and loaded back: baked areas on
//! the polygons (and the shapes the mesh re-marks after an obstacle), and
//! a navmesh without them cooked with no new sections, so the blobs of
//! earlier navmeshes stay as they were.

use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use og_core::glam::Vec3;
use std::path::{Path, PathBuf};

fn project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-navmesh")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    for (path, text) in files {
        let p = root.join("assets").join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }
    root
}

fn cook(root: &Path) -> Result<og_cook::Report, String> {
    let report = cook_project(root, &CookOptions::default()).map_err(|e| e.to_string())?;
    if report.ok() {
        Ok(report)
    } else {
        Err(format!("{:?}", report.errors()))
    }
}

const HEAD: &str = "type navmesh\nversion 1\n---\ncellSize 0.25\ntileSize 16\nbounds {\n  min -1 -1 -1\n  max 21 3 11\n}\nbox {\n  center 10 -0.1 5\n  size 20 0.2 10\n}\n";

fn mesh(root: &Path, id: &str) -> og_nav::NavMesh {
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let bytes = bundle.read(id).unwrap();
    let (field, tiles) = bundle.navmesh(id, &bytes).unwrap();
    let areas = bundle.navmesh_areas(id, &bytes).unwrap();
    og_nav::NavMesh::from_tiles_with_areas(field, tiles, areas).expect("tiles link")
}

#[test]
fn baked_areas_reach_the_polygons_and_survive_an_obstacle_s_rebuild() {
    let text = format!(
        "{HEAD}area {{\n  id 3\n  center 10 0 3.5\n  size 2 2 9\n}}\narea {{\n  id 5\n  center 4 0 5\n  radius 1.5\n}}\n"
    );
    let root = project("areas", &[("nav/yard.ogt", &text), ("nav/plain.ogt", HEAD)]);
    let report = cook(&root).unwrap();
    let notes: Vec<String> = report.assets.iter().flat_map(|a| a.notes.clone()).collect();
    assert!(
        notes.iter().any(|n| n.starts_with("areas: 2 shapes")),
        "{notes:?}"
    );
    let m = mesh(&root, "nav/yard.ogt");
    let area_at = |m: &og_nav::NavMesh, x: f32, z: f32| {
        let (r, _) = m
            .nearest(Vec3::new(x, 0.0, z), Vec3::new(0.05, 1.0, 0.05))
            .expect("on the mesh");
        m.poly(r).unwrap().1.area
    };
    assert_eq!(area_at(&m, 10.0, 3.0), 3);
    assert_eq!(area_at(&m, 4.0, 5.0), 5);
    assert_eq!(area_at(&m, 15.0, 5.0), 0);
    assert_eq!(m.base_areas().len(), 2);
    // An obstacle beside the water rebuilds its tiles: the water stays water.
    let mut m2 = m.clone();
    m2.set_obstacle(
        1,
        Some(og_nav::Obstacle {
            min: Vec3::new(11.5, 0.0, 2.0),
            max: Vec3::new(12.0, 1.0, 3.0),
        }),
    );
    assert_eq!(area_at(&m2, 10.0, 3.0), 3);
    m2.set_obstacle(1, None);
    assert_eq!(m2.tiles, m.tiles);
    // A navmesh without areas has no new sections and is all ground.
    let plain = mesh(&root, "nav/plain.ogt");
    assert!(plain.base_areas().is_empty());
    assert!(
        plain
            .tiles
            .iter()
            .flat_map(|t| &t.polys)
            .all(|p| p.area == 0)
    );
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let bytes = bundle.read("nav/plain.ogt").unwrap();
    for section in ["areas", "poly_areas"] {
        assert!(
            !bytes
                .windows(section.len())
                .any(|w| w == section.as_bytes()),
            "a navmesh without areas holds no '{section}' section"
        );
    }
}

#[test]
fn a_malformed_area_is_refused_naming_its_line() {
    for (bad, want) in [
        (
            "area {\n  center 1 0 1\n}\n",
            "an area needs an id from 1 to 63",
        ),
        ("area {\n  id 64\n}\n", "an area needs an id from 1 to 63"),
        (
            "area {\n  id 2\n  point 0 0 0\n  point 1 0 0\n}\n",
            "a convex area needs three or more points",
        ),
    ] {
        let root = project("bad-area", &[("nav/bad.ogt", &format!("{HEAD}{bad}"))]);
        let e = cook(&root).expect_err("refused");
        assert!(e.contains(want) && e.contains("line 1"), "{e}");
    }
}
