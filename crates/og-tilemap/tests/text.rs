//! The two `.ogt` grammars: tilesets round-trip through their canonical
//! text; hand-written maps read legends, numeric rows and objects.

mod support;
use glam::Vec2;
use og_tilemap::{Collision, Filter, MapState, TilemapAsset, Tileset, cell_center, world_to_cell};
use support::*;

#[test]
fn a_tileset_round_trips_through_its_canonical_text() {
    let t = Tileset::parse(&blob_tileset("empty")).unwrap();
    assert_eq!(t.collision(60), Some(Collision::Full));
    assert_eq!(
        t.collision(61),
        Some(Collision::Rect {
            x: 0.0,
            y: 8.0,
            w: 16.0,
            h: 8.0
        })
    );
    assert_eq!(t.tiles[&50].anim.as_ref().unwrap().frames, vec![50, 51, 52]);
    assert_eq!(t.filter, Filter::Nearest);
    assert!(!t.terrains[0].edge_filled);
    assert_eq!(Tileset::parse(&t.to_text()).unwrap(), t);
    let full = Tileset::parse("type tileset\nversion 1\n---\ntexture \"a.png\"\ngrid 8 8 2 2\ntile 1 {\n  collide 0 0 8 8\n}\n").unwrap();
    assert_eq!(
        full.collision(1),
        Some(Collision::Full),
        "a rect covering the tile is full"
    );
}

#[test]
fn tileset_mistakes_name_their_line() {
    let bad = [
        ("grid 16 16 2 2\n", "no texture"),
        (
            "texture \"a.png\"\ngrid 16 16 2 2\nsolid 4\n",
            "the grid has 4 tiles",
        ),
        (
            "texture \"a.png\"\ngrid 16 16 2 2\ntile 1 {\n  frames 1 2\n}\n",
            "line 7",
        ),
        (
            "texture \"a.png\"\ngrid 16 16 2 2\nfilter blurry\n",
            "line 6",
        ),
    ];
    for (body, want) in bad {
        let e = Tileset::parse(&format!("type tileset\nversion 1\n---\n{body}")).unwrap_err();
        assert!(e.to_string().contains(want), "{body}: {e}");
    }
}

const MAP: &str = r#####"type tilemap
version 1
---
size 4 3
tile 16 8
unit 2
tileset "tiles/t.ogt"
layer floor {
  z -10
  parallax 0.5 1
  repeat x
  legend "#" 7
  legend "o" 50
  row "#..o"
  row "####"
  row ".#.."
}
layer walls {
  terrain wall
  collision true
  row 1 1 0 0
  row 0 1 0 0
  row 0 0 0 0
}
object door {
  kind "door"
  rect 3 1 1 2
  prop target "room2"
}
"#####;

#[test]
fn a_hand_written_map_reads_legends_rows_and_objects() {
    let m = TilemapAsset::parse(MAP).unwrap();
    assert_eq!(
        (m.width, m.height, m.tile_size()),
        (4, 3, Vec2::new(2.0, 1.0))
    );
    let f = &m.layers[0];
    assert_eq!(
        (f.z, f.parallax, f.repeat_x, f.repeat_y),
        (-10, Vec2::new(0.5, 1.0), true, false)
    );
    assert_eq!(f.cells, vec![8, 0, 0, 51, 8, 8, 8, 8, 0, 8, 0, 0]);
    let w = &m.layers[1];
    assert_eq!(
        (w.terrain.as_deref(), w.collision, w.z),
        (Some("wall"), true, 1)
    );
    let o = &m.objects[0];
    assert_eq!((o.name.as_str(), o.kind.as_str()), ("door", "door"));
    assert_eq!((o.x, o.y, o.width, o.height), (6.0, -1.0, 2.0, 2.0));
    assert_eq!(o.properties, vec![("target".into(), "room2".into())]);
    let s = MapState::new(&m, &[tileset("empty")]).unwrap();
    assert_eq!(
        s.tile(1, 0, 0).unwrap().0,
        blob_tile(og_tilemap::mask::E),
        "(0,0) has only E (and the SE corner needs S)"
    );
    assert_eq!(world_to_cell(&m, Vec2::new(6.5, -1.5)), Some((3, 1)));
    assert_eq!(world_to_cell(&m, Vec2::new(6.5, 0.5)), None);
    assert_eq!(cell_center(&m, 3, 1), Vec2::new(7.0, -1.5));
}

#[test]
fn map_mistakes_name_their_line() {
    let e = TilemapAsset::parse(&MAP.replace("row \"####\"", "row \"###\"")).unwrap_err();
    assert!(
        e.to_string().contains("3 cells wide") && e.to_string().contains("line 15"),
        "{e}"
    );
    let e = TilemapAsset::parse(&MAP.replace("row \"####\"", "row \"##x#\"")).unwrap_err();
    assert!(
        e.to_string().contains("'x' is not in the layer's legend"),
        "{e}"
    );
    let e = TilemapAsset::parse(&MAP.replace("row 0 1 0 0", "row 0 2 0 0")).unwrap_err();
    assert!(e.to_string().contains("terrain cell"), "{e}");
}
