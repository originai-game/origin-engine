//! A navmesh asset's G2-4 entries cooked and loaded back: baked areas on
//! the polygons (and the shapes the mesh re-marks after an obstacle), and
//! a navmesh without them cooked with no new sections, so the blobs of
//! earlier navmeshes stay as they were.

use og_ai::nav::cooked::BundleNav;
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

fn mesh(root: &Path, id: &str) -> og_ai::nav::NavMesh {
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let bytes = bundle.read(id).unwrap();
    let (field, tiles) = bundle.navmesh(id, &bytes).unwrap();
    let areas = bundle.navmesh_areas(id, &bytes).unwrap();
    og_ai::nav::NavMesh::from_tiles_with_areas(field, tiles, areas).expect("tiles link")
}

#[test]
fn baked_links_load_with_their_ids_and_a_link_off_the_mesh_is_noted() {
    // Two floors with a gap at x 8–10: a jump across it, a one-way ladder
    // and a door that starts shut, and one link that lands in the gap.
    let floors = "type navmesh\nversion 1\n---\ncellSize 0.25\ntileSize 16\nbounds {\n  min -1 -1 -1\n  max 21 3 11\n}\nbox {\n  center 4 -0.1 5\n  size 8 0.2 10\n}\nbox {\n  center 15 -0.1 5\n  size 10 0.2 10\n}\n";
    let text = format!(
        "{floors}link {{\n  id 4\n  start 7.2 0 2\n  end 10.8 0 2\n  kind jump\n}}\nlink {{\n  id 9\n  start 7.2 0 6\n  end 10.8 0 6\n  kind ladder\n  bidirectional false\n  traverse game\n  duration 1.5\n}}\nlink {{\n  id 2\n  start 7.2 0 8\n  end 10.8 0 8\n  kind door\n  open false\n  cost 20\n}}\nlink {{\n  id 7\n  start 7.2 0 4\n  end 9 0 -4\n}}\n"
    );
    let root = project("links", &[("nav/gap.ogt", &text)]);
    let report = cook(&root).unwrap();
    let notes: Vec<String> = report.assets.iter().flat_map(|a| a.notes.clone()).collect();
    assert!(notes.iter().any(|n| n == "links: 4"), "{notes:?}");
    assert!(
        notes
            .iter()
            .any(|n| n.starts_with("warning: links 7 have an end")),
        "{notes:?}"
    );
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let bytes = bundle.read("nav/gap.ogt").unwrap();
    let links = bundle.navmesh_links("nav/gap.ogt", &bytes).unwrap();
    let ids: Vec<u32> = links.iter().map(|(id, _)| *id).collect();
    assert_eq!(ids, [4, 9, 2, 7], "in the file's order, with their ids");
    let ladder = &links[1].1;
    assert!(ladder.kind == og_ai::nav::LinkKind::Ladder && !ladder.bidirectional && ladder.game);
    assert_eq!(ladder.duration, 1.5);
    let door = &links[2].1;
    assert!(door.kind == og_ai::nav::LinkKind::Door && !door.open && door.cost == 20.0);
    let m = mesh(&root, "nav/gap.ogt").with_links(links);
    let p = m
        .find_path(
            Vec3::new(3.0, 0.0, 2.0),
            Vec3::new(14.0, 0.0, 2.0),
            Vec3::new(1.0, 2.0, 1.0),
        )
        .unwrap();
    assert!(p.complete);
    assert_eq!(p.links.len(), 1);
    assert_eq!(p.links[0].link, 4);
    assert!(m.link_ends(7).is_none());
}

/// A 20 × 10 m floor split at x 9.5–10.5 by a wall with a 1 m doorway at
/// z 4.5–5.5: wide enough for a 0.2 m agent, not for a 0.8 m one.
const DOORWAY: &str = "type navmesh\nversion 1\n---\ncellSize 0.1\ntileSize 32\nbounds {\n  min -1 -1 -1\n  max 21 3 11\n}\nbox {\n  center 10 -0.1 5\n  size 20 0.2 10\n}\nbox {\n  center 10 1 2.25\n  size 1 2 4.5\n}\nbox {\n  center 10 1 7.75\n  size 1 2 4.5\n}\n";

#[test]
fn agent_profiles_bake_one_mesh_each_and_the_first_fills_the_blob_s_own_tables() {
    let big = "agent {\n  radius 0.8\n}\n";
    let two = format!("{DOORWAY}{big}agent small {{\n  radius 0.2\n  height 0.6\n}}\n");
    let named = format!(
        "{DOORWAY}agent ogre {{\n  radius 0.8\n}}\nagent rat {{\n  radius 0.2\n  height 0.6\n}}\n"
    );
    let root = project(
        "profiles",
        &[
            ("nav/one.ogt", &format!("{DOORWAY}{big}")),
            ("nav/two.ogt", &two),
            ("nav/named.ogt", &named),
        ],
    );
    let report = cook(&root).unwrap();
    let notes: Vec<String> = report.assets.iter().flat_map(|a| a.notes.clone()).collect();
    assert!(
        notes
            .iter()
            .any(|n| n.starts_with("agent profile 'small': ")),
        "{notes:?}"
    );
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let load = |id: &str, profile: &str| -> Option<og_ai::nav::NavMesh> {
        let bytes = bundle.read(id).unwrap();
        let b = bundle.navmesh_profile(id, &bytes, profile).unwrap()?;
        let (field, tiles) = bundle.navmesh(id, b).unwrap();
        Some(og_ai::nav::NavMesh::from_tiles(field, tiles).expect("tiles link"))
    };
    let through = |m: &og_ai::nav::NavMesh| {
        m.find_path(
            Vec3::new(3.0, 0.0, 5.0),
            Vec3::new(17.0, 0.0, 5.0),
            Vec3::new(1.0, 2.0, 1.0),
        )
        .is_some_and(|p| p.complete)
    };
    let one = load("nav/one.ogt", "").unwrap();
    assert!(!through(&one), "the doorway is too narrow for 0.8 m");
    // The unnamed agent is the first profile: the blob's own tables are
    // the single-agent mesh, and the small one fits through.
    let default = load("nav/two.ogt", "").unwrap();
    assert_eq!(default.tiles, one.tiles);
    let small = load("nav/two.ogt", "small").unwrap();
    assert!(through(&small));
    assert!(small.field.params.agent_radius == 0.2 && small.field.params.agent_height == 0.6);
    assert!(load("nav/two.ogt", "medium").is_none());
    let names = |id: &str| {
        bundle
            .navmesh_profiles(id, &bundle.read(id).unwrap())
            .unwrap()
    };
    assert_eq!(names("nav/two.ogt"), ["", "small"]);
    // All named: the first is the default and answers to its name too.
    assert_eq!(names("nav/named.ogt"), ["ogre", "rat"]);
    assert_eq!(load("nav/named.ogt", "ogre").unwrap().tiles, one.tiles);
    assert_eq!(load("nav/named.ogt", "").unwrap().tiles, one.tiles);
    assert!(through(&load("nav/named.ogt", "rat").unwrap()));
    // One unnamed agent writes no profiles section: earlier blobs stay as they were.
    assert!(names("nav/one.ogt").is_empty());
    let bytes = bundle.read("nav/one.ogt").unwrap();
    assert!(!bytes.windows(8).any(|w| w == b"profiles"));
}

#[test]
fn the_heightfield_loads_as_baked_from_its_packed_streams() {
    // A floor in two halves with a gap at x 8–10, and a platform 2 m above
    // the second half: columns of no, one and two floors.
    let text = "type navmesh\nversion 1\n---\nagent {\n  radius 0.25\n  height 1\n}\ncellSize 0.25\ncellHeight 0.1\ntileSize 16\nbounds {\n  min -1 -1 -1\n  max 21 4 11\n}\nbox {\n  center 4 -0.1 5\n  size 8 0.2 10\n}\nbox {\n  center 15 -0.1 5\n  size 10 0.2 10\n}\nbox {\n  center 15 2 5\n  size 4 0.2 4\n}\n";
    let root = project("heightfield", &[("nav/floors.ogt", text)]);
    cook(&root).unwrap();
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let bytes = bundle.read("nav/floors.ogt").unwrap();
    let (field, _) = bundle.navmesh("nav/floors.ogt", &bytes).unwrap();
    assert_eq!((field.width, field.depth), (88, 48));
    // Every column's spans follow the previous column's, and none is left over.
    let mut next = 0;
    for &[first, count] in &field.columns {
        assert_eq!(first, next);
        next += count;
    }
    assert_eq!(next as usize, field.spans.len());
    // Heights in metres above the bounds' floor, 0.1 m a cell.
    let floors = |x: f32, z: f32| -> Vec<(f32, Option<f32>)> {
        let cell = |v: f32, o: f32| ((v - o) / field.params.cell_size) as i32;
        field
            .column(cell(x, field.origin.x), cell(z, field.origin.z))
            .map(|s| field.spans[s])
            .map(|s| {
                let open = s.clear == u32::MAX / 4;
                (
                    field.origin.y + s.floor as f32 * 0.1,
                    (!open).then_some(s.clear as f32 * 0.1),
                )
            })
            .collect()
    };
    let near = |a: f32, b: f32| (a - b).abs() <= 0.11;
    assert_eq!(floors(9.0, 5.0), [], "the gap");
    let ground = floors(4.0, 5.0);
    assert!(
        matches!(ground[..], [(y, None)] if near(y, 0.0)),
        "{ground:?}"
    );
    let stacked = floors(15.0, 5.0);
    assert!(
        matches!(stacked[..], [(low, Some(room)), (high, None)]
            if near(low, 0.0) && near(room, 1.9) && near(high, 2.1)),
        "{stacked:?}"
    );
    // The two tables are streams: far under the 4 bytes a column's count
    // alone takes as a row.
    assert!(
        bytes.len() < field.columns.len() * 4,
        "{} bytes for {} columns",
        bytes.len(),
        field.columns.len()
    );
}
