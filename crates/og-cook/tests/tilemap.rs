//! Tilesets and maps through the cook and back: a tileset `.ogt`, a
//! hand-written map, a Tiled `.tmj` (external and embedded tilesets, flips,
//! an auto-tiled layer, objects) and an LDtk project (tiles, IntGrid
//! collision, entities) cook to blobs the runtime loads into og-tilemap
//! assets; the refusals name what to fix.

use glam::Vec2;
use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use og_tilemap::{
    Collision, FLIP_X, FLIP_Y, MapState, Rect, TilemapAsset, Tileset, TilesetRef,
    collision_rects_all,
};
use std::path::PathBuf;
use std::sync::Arc;

const TILESET: &str = r#"type tileset
version 1
---
texture "tiles/dungeon.png"
grid 16 16 8 8
solid 1
tile 5 {
  frames 5 6
  fps 2
}
terrain wall {
  mode blob47
  rule 0 10
  rule 4 11
  rule 64 12
  rule 68 13
  default 9
  edge empty
}
"#;

const ROOM: &str = r#####"type tilemap
version 1
---
size 4 2
tile 16 16
tileset "tiles/dungeon.ogt"
layer floor {
  collision true
  legend "#" 1
  row "##.."
  row "...."
}
object exit {
  kind "door"
  rect 3 0 1 1
}
"#####;

const TILED: &str = r#"{
  "type": "map", "orientation": "orthogonal", "infinite": false,
  "width": 4, "height": 3, "tilewidth": 16, "tileheight": 16,
  "tilesets": [
    { "firstgid": 1, "source": "../tiles/dungeon.tsx" },
    { "firstgid": 65, "name": "water", "image": "../tiles/water.png", "imagewidth": 32, "imageheight": 32,
      "tilewidth": 16, "tileheight": 16, "columns": 2, "tilecount": 4, "margin": 0, "spacing": 0,
      "tiles": [ { "id": 1,
        "animation": [ { "tileid": 1, "duration": 200 }, { "tileid": 2, "duration": 300 } ],
        "objectgroup": { "objects": [ { "x": 0, "y": 8, "width": 16, "height": 8 } ] } } ] }
  ],
  "layers": [
    { "type": "tilelayer", "name": "ground", "width": 4, "height": 3, "opacity": 0.5, "visible": true,
      "offsetx": 8, "offsety": 16, "parallaxx": 0.5, "parallaxy": 1,
      "properties": [ { "name": "collision", "type": "bool", "value": true }, { "name": "z", "type": "int", "value": -5 } ],
      "data": [2, 2147483651, 0, 0,  1073741826, 0, 0, 0,  0, 0, 0, 0] },
    { "type": "tilelayer", "name": "walls", "width": 4, "height": 3, "visible": true,
      "properties": [ { "name": "autotile", "type": "string", "value": "wall" } ],
      "data": [0, 0, 0, 0,  9, 9, 0, 0,  0, 0, 0, 9] },
    { "type": "tilelayer", "name": "water", "width": 4, "height": 3, "visible": true,
      "encoding": "base64", "data": "AAAAAAAAAAAAAAAAQgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA" },
    { "type": "objectgroup", "name": "things", "objects": [
      { "name": "spawn", "type": "player", "x": 16, "y": 32, "width": 16, "height": 16,
        "properties": [ { "name": "facing", "type": "string", "value": "down" } ] } ] }
  ]
}"#;

const LDTK: &str = r#"{
  "jsonVersion": "1.5.3", "defaultGridSize": 16, "externalLevels": false,
  "defs": { "tilesets": [ { "uid": 7, "identifier": "Plain", "relPath": "../tiles/plain.png",
    "tileGridSize": 16, "__cWid": 4, "__cHei": 2, "padding": 0, "spacing": 0 } ] },
  "levels": [ { "identifier": "Level_0", "pxWid": 48, "pxHei": 32, "layerInstances": [
    { "__identifier": "Entities", "__type": "Entities", "__gridSize": 16, "__cWid": 3, "__cHei": 2, "visible": true,
      "entityInstances": [ { "__identifier": "Chest", "iid": "c-1", "px": [24, 16], "__pivot": [0.5, 1],
        "width": 16, "height": 16, "fieldInstances": [ { "__identifier": "loot", "__value": "key" } ] } ] },
    { "__identifier": "Ground", "__type": "Tiles", "__gridSize": 16, "__cWid": 3, "__cHei": 2, "visible": true,
      "__tilesetDefUid": 7, "__opacity": 1,
      "gridTiles": [ { "px": [0, 0], "src": [16, 16], "f": 1 }, { "px": [32, 16], "src": [48, 0], "f": 2 } ] },
    { "__identifier": "Walls", "__type": "IntGrid", "__gridSize": 16, "__cWid": 3, "__cHei": 2, "visible": true,
      "intGridCsv": [1, 1, 0, 0, 2, 0] }
  ] } ]
}"#;

fn png(w: u32, h: u32) -> Vec<u8> {
    let mut out = Vec::new();
    image::RgbaImage::from_pixel(w, h, image::Rgba([200, 100, 50, 255]))
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

fn project(name: &str, files: &[(&str, Vec<u8>)]) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-tilemap")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    for (path, bytes) in files {
        let p = root.join("assets").join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    }
    root
}

fn rect(x0: f32, y0: f32, x1: f32, y1: f32) -> Rect {
    Rect {
        min: Vec2::new(x0, y0),
        max: Vec2::new(x1, y1),
    }
}

/// The loaded map's tilesets, asset ones through the bundle.
fn tilesets(bundle: &Bundle, map: &TilemapAsset) -> Vec<Arc<Tileset>> {
    map.tilesets
        .iter()
        .map(|t| match t {
            TilesetRef::Asset(id) => Arc::new(bundle.load_tileset(id).unwrap()),
            TilesetRef::Embedded(t) => Arc::new(t.clone()),
        })
        .collect()
}

fn files() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        ("tiles/dungeon.png", png(128, 128)),
        ("tiles/dungeon.ogt", TILESET.as_bytes().to_vec()),
        ("tiles/water.png", png(32, 32)),
        ("tiles/plain.png", png(64, 32)),
        ("maps/room.ogt", ROOM.as_bytes().to_vec()),
        ("maps/level.tmj", TILED.as_bytes().to_vec()),
        ("maps/world.ldtk", LDTK.as_bytes().to_vec()),
    ]
}

#[test]
fn tilesets_and_maps_from_three_sources_cook_and_load() {
    let root = project("ok", &files());
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();

    let ts = bundle.load_tileset("tiles/dungeon.ogt").unwrap();
    assert_eq!(ts, Tileset::parse(TILESET).unwrap());

    // The hand-written map.
    let room = bundle.load_tilemap("maps/room.ogt").unwrap();
    assert_eq!(room, TilemapAsset::parse(ROOM).unwrap());
    let s = MapState::new(&room, &tilesets(&bundle, &room)).unwrap();
    assert_eq!(collision_rects_all(&s), vec![rect(0.0, -1.0, 2.0, 0.0)]);
    assert_eq!(room.objects[0].kind, "door");

    // Tiled: an external tileset (the sibling .ogt), an embedded one, flips,
    // offsets, parallax, properties, an auto-tiled layer and objects.
    let tm = bundle.load_tilemap("maps/level.tmj").unwrap();
    assert_eq!((tm.width, tm.height, tm.tile_width_px), (4, 3, 16));
    assert_eq!(
        tm.tilesets[0],
        TilesetRef::Asset("tiles/dungeon.ogt".into())
    );
    let TilesetRef::Embedded(water) = &tm.tilesets[1] else {
        panic!("the water tileset is embedded")
    };
    assert_eq!(water.texture, "tiles/water.png");
    let anim = water.tiles[&1].anim.as_ref().unwrap();
    assert_eq!(
        (anim.frames.clone(), anim.durations.clone()),
        (vec![1, 2], vec![0.2, 0.3])
    );
    assert_eq!(
        water.collision(1),
        Some(Collision::Rect {
            x: 0.0,
            y: 8.0,
            w: 16.0,
            h: 8.0
        })
    );
    let g = &tm.layers[0];
    assert_eq!(&g.cells[..5], &[2, 3 | FLIP_X, 0, 0, 2 | FLIP_Y]);
    assert_eq!(
        (g.offset, g.parallax, g.opacity, g.z, g.collision),
        (Vec2::new(0.5, -1.0), Vec2::new(0.5, 1.0), 0.5, -5, true)
    );
    let w = &tm.layers[1];
    assert_eq!((w.terrain.as_deref(), w.z), (Some("wall"), 1));
    assert_eq!(w.cells, vec![0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1]);
    assert_eq!(
        (tm.layers[2].tileset, tm.layers[2].cells[3]),
        (1, 2),
        "base64 data, gid 66 is water tile 1"
    );
    let o = &tm.objects[0];
    assert_eq!(
        (
            o.name.as_str(),
            o.kind.as_str(),
            o.x,
            o.y,
            o.width,
            o.height
        ),
        ("spawn", "player", 1.0, -2.0, 1.0, 1.0)
    );
    assert_eq!(o.properties, vec![("facing".into(), "down".into())]);
    let s = MapState::new(&tm, &tilesets(&bundle, &tm)).unwrap();
    assert_eq!(
        s.tile(1, 0, 1).map(|t| t.0),
        Some(11),
        "W of the pair: E only (mask 4)"
    );
    assert_eq!(
        s.tile(1, 1, 1).map(|t| t.0),
        Some(12),
        "E of the pair: W only (mask 64)"
    );
    assert_eq!(
        s.tile(1, 3, 2).map(|t| t.0),
        Some(10),
        "isolated, the edge is empty"
    );
    assert_eq!(
        collision_rects_all(&s),
        vec![rect(0.5, -3.0, 1.5, -1.0)],
        "ground's solid tile 1 in two rows (one flipped), merged, offset"
    );

    // LDtk: the Tiles layer with an embedded tileset (no sibling .ogt),
    // IntGrid value 1 as collision, the entity at its pivot.
    let lm = bundle.load_tilemap("maps/world.ldtk").unwrap();
    assert_eq!((lm.width, lm.height), (3, 2));
    let ground = &lm.layers[lm.layer("Ground").unwrap()];
    let TilesetRef::Embedded(plain) = &lm.tilesets[ground.tileset as usize] else {
        panic!("plain is embedded")
    };
    assert_eq!(
        (plain.texture.as_str(), plain.columns, plain.rows),
        ("tiles/plain.png", 4, 2)
    );
    assert_eq!(ground.cells, vec![6 | FLIP_X, 0, 0, 0, 0, 4 | FLIP_Y]);
    let walls = &lm.layers[lm.layer("Walls.collision").unwrap()];
    assert_eq!(
        (walls.cells.clone(), walls.visible, walls.collision),
        (vec![1, 1, 0, 0, 0, 0], false, true)
    );
    assert!(
        lm.layer("Ground").unwrap() > lm.layer("Walls.collision").unwrap(),
        "LDtk's first layer draws last"
    );
    let chest = &lm.objects[0];
    assert_eq!(
        (chest.kind.as_str(), chest.name.as_str(), chest.x, chest.y),
        ("Chest", "c-1", 1.0, 0.0)
    );
    assert_eq!(chest.properties, vec![("loot".into(), "key".into())]);
    let s = MapState::new(&lm, &tilesets(&bundle, &lm)).unwrap();
    assert_eq!(collision_rects_all(&s), vec![rect(0.0, -1.0, 2.0, 0.0)]);
}

#[test]
fn a_compressed_tiled_layer_and_a_missing_tileset_ogt_are_refused_by_name() {
    let compressed = TILED.replace(
        "\"encoding\": \"base64\",",
        "\"encoding\": \"base64\", \"compression\": \"zlib\",",
    );
    let mut f = files();
    f.retain(|(p, _)| *p != "maps/level.tmj" && *p != "maps/world.ldtk");
    f.push(("maps/level.tmj", compressed.into_bytes()));
    let root = project("zlib", &f);
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    let errors = report.errors().join("\n");
    assert!(
        errors.contains("layer 'water' is zlib-compressed") && errors.contains("CSV or Base64"),
        "{errors}"
    );

    let mut f = files();
    f.retain(|(p, _)| *p != "tiles/dungeon.ogt" && *p != "maps/room.ogt");
    let root = project("missing", &f);
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    let errors = report.errors().join("\n");
    assert!(
        errors.contains("needs a tileset .ogt at assets/tiles/dungeon.ogt"),
        "{errors}"
    );
}

#[test]
fn import_settings_pick_the_ldtk_level_unit_and_solid_values() {
    let mut f = files();
    f.push((
        "maps/world.ldtk.ogt",
        b"type import\nversion 1\n---\ntilemap {\n  unit 2\n  level \"Level_0\"\n  solid 2\n}\n"
            .to_vec(),
    ));
    let root = project("settings", &f);
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let lm = bundle.load_tilemap("maps/world.ldtk").unwrap();
    assert_eq!(lm.unit, 2.0);
    let walls = &lm.layers[lm.layer("Walls.collision").unwrap()];
    assert_eq!(
        walls.cells,
        vec![0, 0, 0, 0, 1, 0],
        "only IntGrid value 2 is solid"
    );
    assert_eq!((lm.objects[0].x, lm.objects[0].y), (2.0, 0.0));

    let mut f = files();
    f.push((
        "maps/world.ldtk.ogt",
        b"type import\nversion 1\n---\ntilemap {\n  level \"Nope\"\n}\n".to_vec(),
    ));
    let root = project("no-level", &f);
    let errors = cook_project(&root, &CookOptions::default())
        .unwrap()
        .errors()
        .join("\n");
    assert!(errors.contains("no level 'Nope'"), "{errors}");
}
