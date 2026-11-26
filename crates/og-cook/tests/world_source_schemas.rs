//! Domain schema contracts: closed inputs, complete tagged geometry and real TS.
use super::temp;
use og_cook::cook::source_schemas::{
    self,
    world::{geometry::Geometry, pack::PackInput},
};
use og_core::reflect::source::typescript;
use serde_json::json;

#[test]
fn geometry_and_pack_author_contracts() {
    let geometry = Geometry::schema();
    let pack = PackInput::schema();
    assert_eq!(source_schemas::schema("pack"), Some(pack.document()));
    let mut geometries = Vec::new();
    for kind in [
        "boxGeometry",
        "planeGeometry",
        "sphereGeometry",
        "cylinderGeometry",
        "coneGeometry",
        "capsuleGeometry",
        "torusGeometry",
        "ringGeometry",
        "circleGeometry",
        "shapeGeometry",
        "extrudeGeometry",
        "latheGeometry",
        "icosahedronGeometry",
        "dodecahedronGeometry",
        "octahedronGeometry",
        "tetrahedronGeometry",
    ] {
        geometries.push(json!({"kind":kind, "desc":{}}));
    }
    let path = json!({"points":[[0,0,0],[2,3,1]], "kind":"linear"});
    let commands = json!([
        {"op":"moveTo", "args":[0,0]}, {"op":"lineTo", "args":[2,0]},
        {"op":"lineTo", "args":[1,3]}, {"op":"closePath"}
    ]);
    let leaf = json!({"geometry":{"kind":"boxGeometry","desc":{"width":2}},
        "transform":{"position":[1,2,3]}, "mirror":"z"});
    geometries.extend([
        json!({"kind":"modelGeometry", "desc":{"source":{"$ogRef":"base/model/door"}, "node":"Panel"}}),
        json!({"kind":"bufferGeometry", "desc":{
            "positions":[0,0,0,1,0,0,0,2,0],"indices":[0,1,2],
            "normals":[0,0,1,0,0,1,0,0,1], "uvs":[0,0,1,0,0,1],
            "colors":[1,0,0,0,1,0,0,0,1], "tangents":[1,0,0,1,1,0,0,1,1,0,0,1]
        }}),
        json!({"kind":"tubeGeometry", "desc":{"path":{"kind":"curve","curve":path}}}),
        json!({"kind":"polyhedronGeometry", "desc":{"vertices":[1,0,0,0,1,0,0,0,1], "indices":[0,1,2]}}),
        json!({"kind":"sweepGeometry", "desc":{"section":commands,"path":path,"scale":[[0,1],[1,2]]}}),
        json!({"kind":"csgGeometry", "desc":{"op":"subtract", "a":leaf, "b":leaf}}),
        json!({"kind":"mergeGeometries", "desc":{"parts":[leaf]}}),
    ]);
    assert_eq!(geometries.len(), 23);
    geometries.extend([
        json!({"kind":"bufferGeometry", "desc":{"positions":[0,0,0],"indices":[0,0,0]}}),
        json!({"kind":"polyhedronGeometry", "desc":{"vertices":[1,0,0],"indices":[0,0,0]}}),
        json!({"kind":"boxGeometry", "desc":{"width":0,"height":2}}),
        json!({"kind":"sphereGeometry", "desc":{"widthSegments":512,"heightSegments":2}}),
        json!({"kind":"tubeGeometry", "desc":{"path":{"kind":"points","points":[[1,2,3],[4,5,6]],"closed":false}}}),
        json!({"kind":"shapeGeometry", "desc":{"shapes":[{"commands":[
            {"op":"moveTo","args":[0,0]},
            {"op":"quadraticCurveTo","args":[1,2,3,4]},
            {"op":"bezierCurveTo","args":[1,2,3,4,5,6]},
            {"op":"splineThru","args":[[1,2],[3,4]]},
            {"op":"points","args":[[1,2],[3,4]]},
            {"op":"arc","args":{"center":[1,2],"radius":3,"start":0,"end":1}},
            {"op":"absarc","args":{"center":[1,2],"radius":3,"start":0,"end":1,"clockwise":true}},
            {"op":"ellipse","args":{"center":[1,2],"radii":[3,4],"start":0,"end":1}},
            {"op":"absellipse","args":{"center":[1,2],"radii":[3,4],"start":0,"end":1,"rotation":0.5}},
            {"op":"closePath"}
        ], "holes":[commands]}]}}),
    ]);
    for value in &geometries {
        geometry
            .decode(value.clone())
            .unwrap_or_else(|error| panic!("rejected {value}: {error}"));
    }
    for wrong in [
        json!({"kind":"boxGeometry", "desc":{"width":-1}}),
        json!({"kind":"sphereGeometry", "desc":{"widthSegments":2}}),
        json!({"kind":"boxGeometry", "desc":{"radius":1}}),
        json!({"kind":"boxGeometry", "desc":{}, "position":[0,0,0]}),
        json!({"kind":"torusKnotGeometry", "desc":{}}),
        json!({"kind":"modelGeometry", "desc":{"source":34}}),
        json!({"kind":"bufferGeometry", "desc":{"positions":[1e39,0,0],"indices":[0,0,0]}}),
        json!({"kind":"bufferGeometry", "desc":{"positions":[0,0,0],"normals":[0,1e39,0],"indices":[0,0,0]}}),
        json!({"kind":"tubeGeometry", "desc":{"path":{"kind":"curve","curve":{"points":[[0,0,0],[1e39,0,0]]}}}}),
        json!({"kind":"tubeGeometry", "desc":{"path":{"kind":"points","points":[[0,0,0]]}}}),
        json!({"kind":"polyhedronGeometry", "desc":{"vertices":[],"indices":[0,0,0]}}),
        json!({"kind":"csgGeometry", "desc":{"a":leaf}}),
        json!({"kind":"mergeGeometries", "desc":{"parts":[]}}),
        json!({"kind":"mergeGeometries", "desc":{"parts":[{"geometry":geometries[0],"transform":{"typo":1}}]}}),
        json!({"kind":"shapeGeometry", "desc":{"shapes":[{"commands":[{"op":"lineTo","args":[1,2,3]}]}]}}),
        json!({"kind":"shapeGeometry", "desc":{"shapes":[{"commands":[],"holes":[{"commands":[]}]}]}}),
    ] {
        assert!(geometry.decode(wrong.clone()).is_err(), "accepted {wrong}");
    }

    let valid_pack = json!({
        "scheme":"dark", "seeds":{"primary":{"light":"#346","dark":"#abc"}},
        "theme":{
            "spacing":4, "rem":16,
            "colors":{"brand":{"100":"#abc", "DEFAULT":{"light":"#345","dark":"#def"}}},
            "fontSize":{"body":[16,1.4]}, "fontFamily":{"sans":"Inter"},
            "ease":{"spring":[0.2,0,0,1]}, "radius":{"card":{"light":4,"dark":8}}
        },
        "fonts":[{"$ogRef":"base/font/body"}],
        "icons":{"coin":{"$ogRef":"base/texture/coin"}},
        "iconSet":{"$ogRef":"base/icons/menu"},
        "images":{"panel":{"src":{"$ogRef":"base/texture/panel"},"slice":[1,2,3,4],
            "repeat":{"light":"stretch","dark":"tile"},"width":{"light":2,"dark":[2,3,4,5]}}},
        "sounds":{"press":{"clips":[{"$ogRef":"base/sound/press"}],"volume":[0.4,0.8]},
            "Button.root":{"hover":{"volume":0}}},
        "haptics":{"press":"tap","Button.root":{"error":"warning"}},
        "variants":{"Button.root":"px-2"}
    });
    pack.decode(valid_pack.clone()).unwrap();
    for wrong in [
        json!({"theme":{"colours":{}}}),
        json!({"theme":{"spacing":{"small":2}}}),
        json!({"theme":{"fontSize":{"body":[12,1.5,3]}}}),
        json!({"theme":{"ease":{"spring":[0,1]}}}),
        json!({"seeds":{"primary":{"light":"#abc"}}}),
        json!({"images":{"panel":{"src":{"$ogRef":"base/texture/panel"},"slice":[1,-2,3,4]}}}),
        json!({"fonts":["base/font/body"]}),
        json!({"haptics":{"press":"shake"}}),
        json!({"sounds":{"Button.root":{"typo":{"volume":0}}}}),
        json!({"unknown":true}),
    ] {
        assert!(pack.decode(wrong.clone()).is_err(), "accepted {wrong}");
    }

    // Every geometry is assigned against the emitted discriminated union, not
    // merely compiled as an untyped literal. @ts-expect-error must itself fail
    // when the declaration accidentally widens a rejected input.
    let mut ts = typescript(geometry.document(), "Geometry").unwrap();
    ts.push_str(&typescript(pack.document(), "Pack").unwrap());
    for (i, value) in geometries.iter().enumerate() {
        ts.push_str(&format!("const geometry{i}: Geometry = {value};\n"));
    }
    ts.push_str(&format!("const pack: Pack = {valid_pack};\n"));
    ts.push_str(r##"
// @ts-expect-error A descriptor belongs to its geometry discriminator.
const wrongDesc: Geometry = {kind:"boxGeometry",desc:{radius:2}};
// @ts-expect-error Runtime handles are not content references.
const handle: Geometry = {kind:"modelGeometry",desc:{source:1}};
// @ts-expect-error CSG needs both typed operands.
const missingOperand: Geometry = {kind:"csgGeometry",desc:{a:{geometry:geometry0}}};
// @ts-expect-error Transform fields come from actual engine reflection.
const unknownTransform: Geometry = {kind:"mergeGeometries",desc:{parts:[{geometry:geometry0,transform:{typo:3}}]}};
// @ts-expect-error Theme section names are closed.
const themeTypo: Pack = {theme:{colours:{}}};
// @ts-expect-error Font size is a number or an exact two-number tuple.
const badSize: Pack = {theme:{fontSize:{body:[12,1.5,3]}}};
// @ts-expect-error A seed scheme pair needs both sides.
const halfPair: Pack = {seeds:{primary:{light:"#abc"}}};
// @ts-expect-error Fonts are content references, never raw paths.
const path: Pack = {fonts:["fonts/body.ttf"]};
// @ts-expect-error Typed haptic presets reject arbitrary words.
const haptic: Pack = {haptics:{press:"shake"}};
// @ts-expect-error Author inputs are immutable.
pack.scheme = "light";
"##);
    let dir = temp::TempDir::new("world-source-schema");
    std::fs::write(dir.join("inputs.ts"), ts).unwrap();
    let result = std::process::Command::new("tsrs")
        .args(["--ignoreConfig", "--strict", "--noEmit", "inputs.ts"])
        .current_dir(&dir)
        .output()
        .expect("pinned tsrs checker (docs/ci.md)");
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    // An actual Rust enum remains accessible after schema validation.
    assert!(matches!(
        geometry.decode(geometries.remove(0)).unwrap(),
        Geometry::BoxGeometry(_)
    ));
}
