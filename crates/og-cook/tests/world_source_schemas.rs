//! Domain schema contracts: closed inputs, complete tagged geometry and real TS.
use super::temp;
use og_cook::cook::source_schemas::{
    self,
    world::{geometry::Geometry, pack::PackInput},
};
use og_core::reflect::source::typescript;
use serde_json::json;

#[test]
fn bake_and_localization_author_contracts() {
    let nav = json!({"scene":{"$ogRef":"base/scene/world"},"agents":[{}, {"name":"small","radius":0.2}],
        "cellSize":0.25,"cellHeight":0.1,"tileSize":32.5,
        "boxes":[{"center":[1,2,3],"size":[4,1,2]}],
        "models":[{"model":{"$ogRef":"base/model/gate"},"transform":{"position":[1,2,3]}}],
        "terrains":[{"terrain":{"$ogRef":"base/terrain/north"},"step":2}],
        "bounds":{"min":[-10,-1,-20],"max":[15,5,25]},
        "areas":[{"id":1,"shape":{"kind":"box","yaw":0.3}},
            {"id":2,"shape":{"kind":"cylinder","radius":2}},
            {"id":3,"shape":{"kind":"convex","points":[[0,0],[2,0],[0,3]],"bottom":-1,"top":2}}],
        "links":[{"id":1,"start":[1,2,3],"end":[5,6,7],"kind":"door","traverse":"game"}]});
    let lightmap = json!({"scene":{"$ogRef":"base/scene/world"},"sceneHash":"ffffffffffffffff","pages":"world.ktx2",
        "samples":256,"seconds":12.5,"models":[{"entity":"Gate","asset":{"$ogRef":"base/model/gate"},"hash":"0123456789abcdef",
            "parts":[{"vertices":3,"xref":[0,1,2],"indices":[0,1,2],"uv":[[0,0,0],[1,0,0],[0,1,0]]}]}],
        "volumes":[{"entity":"Probe","min":[-1,0,-2],"max":[1,3,2],"counts":[1,1,1],"probes":[[[1,2,3,4],[5,6,7,8],[9,10,11,12]]]}]});
    let strings = json!({"locale":"ar","base":true,"direction":"rtl","fallback":"en",
        "fonts":[{"$ogRef":"base/font/arabic"}],"entries":{"items":"{count, plural, one {One item} other {{count} items}}"}});
    let font = json!({"kind":"font","name":"Arabic","languages":["ar"],"scripts":["Arab"],
        "faces":[{"$ogRef":"base/font/arabic"}],"hyphenation":["ar.hyph"]});
    let mut ts = String::new();
    for (kind, name, value) in [
        ("navmesh", "Navmesh", nav),
        ("lightmap", "Lightmap", lightmap),
        ("strings", "Strings", strings),
        ("font-extension", "FontExtension", font),
    ] {
        let schema = (source_schemas::get(kind).unwrap().schema)();
        schema
            .check(&value)
            .unwrap_or_else(|e| panic!("{kind}: {e}"));
        ts.push_str(&typescript(schema.document(), name).unwrap());
        ts.push_str(&format!("const value{name}: {name} = {value};\n"));
    }
    for (kind, wrong) in [
        ("navmesh", json!({"cellSize":0})),
        ("navmesh", json!({"agents":[{"name":"has/slash"}]})),
        ("navmesh", json!({"models":[{"model":34}]})),
        (
            "navmesh",
            json!({"models":[{"model":{"$ogRef":"base/model/a"},"transform":{"typo":3}}]}),
        ),
        (
            "navmesh",
            json!({"areas":[{"id":0,"shape":{"kind":"box"}}]}),
        ),
        (
            "navmesh",
            json!({"areas":[{"id":2,"shape":{"kind":"convex","points":[[0,0],[2,0]]}}]}),
        ),
        (
            "navmesh",
            json!({"areas":[{"id":2,"shape":{"kind":"box","radius":2}}]}),
        ),
        ("navmesh", json!({"links":[{"id":1,"start":[0,0,0]}]})),
        (
            "navmesh",
            json!({"links":[{"id":1,"start":[0,0,0],"end":[1,2,3],"traverse":"automatic"}]}),
        ),
        (
            "lightmap",
            json!({"scene":{"$ogRef":"base/scene/a"},"sceneHash":34,"pages":"a.ktx2"}),
        ),
        (
            "lightmap",
            json!({"scene":{"$ogRef":"base/scene/a"},"sceneHash":"10000000000000000","pages":"a.ktx2"}),
        ),
        (
            "lightmap",
            json!({"scene":{"$ogRef":"base/scene/a"},"sceneHash":"zz","pages":"a.ktx2"}),
        ),
        (
            "lightmap",
            json!({"scene":{"$ogRef":"base/scene/a"},"sceneHash":"ab","pages":"a.ktx2","models":[{"entity":"x","asset":{"$ogRef":"base/model/a"},"hash":"ab","parts":[{"uv":[[0,1]]}]}]}),
        ),
        (
            "lightmap",
            json!({"scene":{"$ogRef":"base/scene/a"},"sceneHash":"ab","pages":"a.ktx2","seconds":1e39}),
        ),
        ("strings", json!({"locale":"en","entries":{"x":4}})),
        (
            "strings",
            json!({"locale":"en","entries":{},"direction":"auto"}),
        ),
        (
            "strings",
            json!({"locale":"en","entries":{},"fonts":["fonts/body.ttf"]}),
        ),
        (
            "font-extension",
            json!({"kind":"sound","name":"x","faces":[{"$ogRef":"base/font/a"}]}),
        ),
        (
            "font-extension",
            json!({"kind":"font","name":"x","faces":[]}),
        ),
    ] {
        assert!(
            (source_schemas::get(kind).unwrap().schema)()
                .check(&wrong)
                .is_err(),
            "{kind} accepted {wrong}"
        );
    }
    ts.push_str(r#"
const minimal: Navmesh = {boxes:[{}]};
const scriptOnly: FontExtension = {kind:"font",name:"Emoji",scripts:["Zsye"],faces:[{$ogRef:"base/font/emoji"}]};
// @ts-expect-error Area variants cannot mix branch keys.
const mixed: Navmesh = {areas:[{id:1,shape:{kind:"box",radius:2}}]};
// @ts-expect-error Navigation uses actual reflected Transform fields.
const transform: Navmesh = {models:[{model:{$ogRef:"base/model/a"},transform:{typo:1}}]};
// @ts-expect-error Convex points are x/z pairs, not arbitrary vectors.
const points: Navmesh = {areas:[{id:1,shape:{kind:"convex",points:[[0,1,2],[0,2,3],[0,3,4]]}}]};
// @ts-expect-error Hashes are lossless hex text, not numeric handles.
const hash: Lightmap = {scene:{$ogRef:"base/scene/a"},sceneHash:42,pages:"a.ktx2"};
// @ts-expect-error A lightmap UV includes its page.
const uv: Lightmap = {scene:{$ogRef:"base/scene/a"},sceneHash:"ab",pages:"a.ktx2",models:[{entity:"x",asset:{$ogRef:"base/model/a"},hash:"ab",parts:[{uv:[[0,1]]}]}]};
// @ts-expect-error Message values are ICU source strings.
const message: Strings = {locale:"en",entries:{x:{message:"Hi"}}};
// @ts-expect-error Locale direction uses the explicit domain vocabulary.
const direction: Strings = {locale:"ar",entries:{},direction:"auto"};
// @ts-expect-error Font faces are content references, not raw paths.
const face: FontExtension = {kind:"font",name:"x",faces:["body.ttf"]};
// @ts-expect-error Nested derived coefficients stay readonly.
valueLightmap.volumes[0].probes[0][1][2] = 0;
"#);
    let dir = temp::TempDir::new("bake-locale-schema");
    std::fs::write(dir.join("inputs.ts"), ts).unwrap();
    let result = std::process::Command::new("tsrs")
        .args(["--ignoreConfig", "--strict", "--noEmit", "inputs.ts"])
        .current_dir(&dir)
        .output()
        .expect("pinned tsrs checker");
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn contextual_world_author_contracts() {
    use og_cook::cook::source_schemas::world::{
        scene::{PrefabInput, SceneInput},
        table::TableInput,
    };
    use og_core::reflect::source::InputSchema;
    use og_core::{
        Schema,
        reflect::{FieldSchema, FieldType, Scalar},
    };
    let mut power = FieldSchema::new("power", FieldType::F32);
    power.default = Some(vec![3.5]);
    power.min = Some(-2.0);
    power.max = Some(8.0);
    let item = Schema {
        name: "game.Item".into(),
        version: 1,
        doc: String::new(),
        fields: vec![
            power,
            FieldSchema::new("title", FieldType::Str(16)),
            FieldSchema::new("tags", FieldType::List(Scalar::U32, 4)),
            FieldSchema::enumeration("rarity", &["common", "rare"]),
        ],
    };
    let resource = Schema {
        name: "game.Settings".into(),
        fields: vec![FieldSchema::new("enabled", FieldType::Bool)],
        ..item.clone()
    };
    let scene = SceneInput::schema_with_records(
        std::slice::from_ref(&item),
        std::slice::from_ref(&resource),
    )
    .unwrap();
    let prefab = PrefabInput::schema_with_records(std::slice::from_ref(&item)).unwrap();
    let table = TableInput::schema_with_record(&item).unwrap();
    for kind in ["scene", "prefab", "table"] {
        assert!(
            source_schemas::get(kind).is_none(),
            "contextual kind must not masquerade as a static schema"
        );
    }
    let valid = json!({"entities":[{"name":"Gate","components":{"game.Item":{"power":2.5}},
        "children":[{"name":"Panel","prefab":{"$ogRef":"base/prefab/panel"},"edits":[
            {"kind":"override","path":"","components":{"game.Item":{"title":"Wide"}}},
            {"kind":"remove","path":"Latch","component":"game.Item"},
            {"kind":"override","path":"Latch","static":true}
        ]}]}], "resources":{"game.Settings":{"enabled":true}},
        "streams":{"nearby":{"assets":[{"$ogRef":"north/scene/nearby"}],"trigger":{"kind":"distance","anchor":[1,2,-3],"radius":12,"unload":18}}},
        "hlod":{"size":64,"texels":128},"paths":{"rail":{"points":[[0,0,0],[2,1,-3]],"kind":"linear"}}});
    let decoded = scene.decode(valid.clone()).unwrap();
    assert_eq!(decoded.entities[0].components["game.Item"].len(), 1);
    assert_eq!(decoded.entities[0].children[0].edits.len(), 3);
    for wrong in [
        json!({"entities":[{"name":"Gate","components":{"game.Item":{"typo":3}}}]}),
        json!({"entities":[{"name":"Gate","components":{"game.Settings":{}}}]}),
        json!({"resources":{"game.Item":{}}}),
        json!({"entities":[{"name":"Gate","children":[{"name":"Deep","components":{"Unknown":{}}}]}]}),
        json!({"entities":[{"name":"Gate","edits":[{"kind":"remove","path":"Child","component":"Missing"}]}]}),
        json!({"entities":[{"name":"Gate","edits":[{"kind":"override","path":"Child","components":{"game.Item":{"power":9}}}]}]}),
        json!({"entities":[{"name":"Gate","edits":[{"kind":"override","path":"Child","prefab":{"$ogRef":"base/prefab/other"}}]}]}),
        json!({"streams":{"bad/key":{}}}),
        json!({"streams":{"near":{"trigger":{"kind":"distance","anchor":[0,0,0],"radius":0}}}}),
        json!({"hlod":{"size":3}}),
    ] {
        assert!(scene.decode(wrong.clone()).is_err(), "accepted {wrong}");
    }
    let valid_prefab = json!({"base":{"$ogRef":"base/prefab/door"},"entity":{"name":"Root","components":{"game.Item":{}}}});
    prefab.decode(valid_prefab.clone()).unwrap();
    for wrong in [
        json!({}),
        json!({"entities":[{"name":"Root"}]}),
        json!({"entity":{"name":"Root"},"resources":{}}),
    ] {
        assert!(prefab.decode(wrong.clone()).is_err(), "accepted {wrong}");
    }
    // No context accepts no dynamic value, rather than a widening map fallback.
    assert!(
        InputSchema::<TableInput>::new()
            .unwrap()
            .decode(json!({"record":"game.Item","rows":[]}))
            .is_err()
    );
    assert!(
        InputSchema::<SceneInput>::new()
            .unwrap()
            .decode(valid.clone())
            .is_err()
    );
    SceneInput::schema_with_records(&[], &[])
        .unwrap()
        .decode(json!({"entities":[{"name":"Empty","components":{}}],"resources":{}}))
        .unwrap();
    assert!(SceneInput::schema_with_records(&[item.clone(), item.clone()], &[]).is_err());
    // A second project owns a different layout with the same record name.
    let other = Schema {
        fields: vec![FieldSchema::new("enabled", FieldType::Bool)],
        ..item.clone()
    };
    let project_b = TableInput::schema_with_record(&other).unwrap();
    project_b
        .decode(json!({"record":"game.Item","rows":[{"id":"one","values":{"enabled":true}}]}))
        .unwrap();
    assert!(
        project_b
            .decode(json!({"record":"game.Item","rows":[{"id":"one","values":{"power":2}}]}))
            .is_err()
    );
    let valid_table = json!({"record":"game.Item","locale":"en","rows":[
        {"id":"z-last","values":{"title":"Heavy","power":-1.5,"tags":[2,7],"rarity":"rare"}},
        {"id":"a-first","values":{}}
    ]});
    let decoded = table.decode(valid_table.clone()).unwrap();
    decoded.validate().unwrap();
    assert_eq!(decoded.rows[0].id, "z-last");
    assert_eq!(decoded.rows[1].id, "a-first");
    assert!(
        decoded.rows[1].values.is_empty(),
        "omitted defaults, strings and lists stay absent"
    );
    for wrong in [
        json!({"rows":[]}),
        json!({"record":"other.Item","rows":[]}),
        json!({"record":"game.Item"}),
        json!({"record":"game.Item","rows":[{"id":"one"}]}),
        json!({"record":"game.Item","rows":[{"id":"","values":{}}]}),
        json!({"record":"game.Item","rows":[{"id":"one","values":{"enabled":true}}]}),
        json!({"record":"game.Item","rows":[{"id":"one","values":{"power":8.1}}]}),
        json!({"record":"game.Item","rows":[{"id":"one","values":{"tags":[1,2,3,4,5]}}]}),
        json!({"record":"game.Item","rows":[{"id":"one","values":{"rarity":"mythic"}}]}),
    ] {
        assert!(table.decode(wrong.clone()).is_err(), "accepted {wrong}");
    }
    let duplicate = table.decode(json!({"record":"game.Item","rows":[{"id":"one","values":{}},{"id":"one","values":{}}]})).unwrap();
    assert!(
        duplicate
            .validate()
            .unwrap_err()
            .contains("duplicate row id")
    );
    let mut ts = typescript(scene.document(), "Scene").unwrap();
    ts.push_str(&typescript(prefab.document(), "Prefab").unwrap());
    ts.push_str(&typescript(table.document(), "Table").unwrap());
    ts.push_str(&format!("const scene: Scene = {valid};\nconst prefab: Prefab = {valid_prefab};\nconst table: Table = {valid_table};\n"));
    ts.push_str(r#"
const omitted: Table = {record:"game.Item",rows:[{id:"one",values:{}}]};
// @ts-expect-error Record selection is a context-fixed identity.
const record: Table = {record:"other.Item",rows:[]};
// @ts-expect-error Structural values is required even when no columns are supplied.
const missing: Table = {record:"game.Item",rows:[{id:"one"}]};
// @ts-expect-error A row's actual layout closes its columns.
const typo: Table = {record:"game.Item",rows:[{id:"one",values:{powre:2}}]};
// @ts-expect-error Enum columns use declared names, not ordinals.
const ordinal: Table = {record:"game.Item",rows:[{id:"one",values:{rarity:0}}]};
// @ts-expect-error Roles are not interchangeable.
const wrongRole: Scene = {resources:{"game.Item":{}}};
// @ts-expect-error Recursive children retain the complete project schema.
const child: Scene = {entities:[{name:"Root",children:[{name:"Child",components:{"Unknown":{}}}]}]};
// @ts-expect-error Removal names come from the component role, not arbitrary strings.
const removal: Prefab = {entity:{name:"Root",edits:[{kind:"remove",path:"Child",component:"Missing"}]}};
// @ts-expect-error A prefab has exactly one root.
const roots: Prefab = {entities:[{name:"Root"}]};
// @ts-expect-error Prefabs do not declare top-level resources.
const resources: Prefab = {entity:{name:"Root"},resources:{}};
// @ts-expect-error Nested row columns are readonly.
table.rows[0].values.power = 2;
"#);
    let dir = temp::TempDir::new("context-world-schema");
    std::fs::write(dir.join("inputs.ts"), ts).unwrap();
    let result = std::process::Command::new("tsrs")
        .args(["--ignoreConfig", "--strict", "--noEmit", "inputs.ts"])
        .current_dir(&dir)
        .output()
        .expect("pinned tsrs checker");
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}

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

#[test]
fn source_data_author_contracts() {
    use og_cook::cook::source_schemas::world::source_data::{
        EnvironmentInput, FieldData, IconSet, PcmInput, TextureArrayInput, TextureKind, VectorInput,
    };
    let inputs = [
        (
            "field",
            "Field",
            json!({"dimensions":[2,1,1], "channels":2,
            "origin":[-2,3,5], "spacing":[0.5,2,3], "values":[-1,0,2,4]}),
        ),
        (
            "sound",
            "Sound",
            json!({"rate":44100,"channels":2,"samples":[-0.5,0.25,0.75,-1]}),
        ),
        (
            "icons",
            "Icons",
            json!({"icons":[["close","xmark"],["heart","heart"]],
            "join":"round", "sticker":[1.5,-2,3]}),
        ),
        (
            "vector",
            "Vector",
            json!({"svg":"<svg viewBox='0 0 2 3'><path d='M0 0L2 3'/></svg>", "tolerance":0.01}),
        ),
        (
            "texture-array",
            "TextureArray",
            json!({"kind":"single", "layers":["rough/a.png","rough/b.png"]}),
        ),
        (
            "environment",
            "Environment",
            json!({"source":{"faces":["px.hdr","nx.hdr","py.hdr","ny.hdr","pz.hdr","nz.hdr"]},
            "intensity":0, "rotation":-30, "size":64, "sun":"keep", "samples":16, "expand":0, "sunBoost":100}),
        ),
    ];
    let mut ts = String::new();
    for (kind, name, input) in &inputs {
        let schema = (source_schemas::get(kind).unwrap().schema)();
        schema
            .check(input)
            .unwrap_or_else(|e| panic!("{kind}: {e}"));
        ts.push_str(&typescript(schema.document(), name).unwrap());
        ts.push_str(&format!("const value{name}: {name} = {input};\n"));
    }
    FieldData::schema()
        .decode(inputs[0].2.clone())
        .unwrap()
        .validate()
        .unwrap();
    let field = FieldData::schema()
        .decode(json!({"dimensions":[1,1,1],"channels":1,"values":[3]}))
        .unwrap();
    assert_eq!(field.origin, [0.0; 3]);
    assert_eq!(field.spacing, [1.0; 3]);
    assert_eq!(
        PcmInput::schema()
            .decode(inputs[1].2.clone())
            .unwrap()
            .samples[1],
        0.25
    );
    let icons = IconSet::schema()
        .decode(json!({"icons":[["a","b"]]}))
        .unwrap();
    assert_eq!(
        (icons.source.as_str(), icons.size, icons.range),
        (".", 48, 8)
    );
    assert!(matches!(
        TextureArrayInput::schema()
            .decode(json!({"layers":["a.png"]}))
            .unwrap()
            .kind,
        TextureKind::Color
    ));
    assert!(matches!(
        VectorInput::schema()
            .decode(json!({"source":"icon.svg"}))
            .unwrap(),
        VectorInput::File { .. }
    ));
    assert!(
        EnvironmentInput::schema()
            .decode(json!({"source":{"equirect":"sky.png"}}))
            .unwrap()
            .sun
            .is_none()
    );
    EnvironmentInput::schema()
        .decode(json!({"source":{"equirect":"sky.png"},"hemisphere":"upper", "ground":[0,0.3,1]}))
        .unwrap();
    EnvironmentInput::schema()
        .decode(json!({"source":{"ktx2":"studio.ktx2"},"prefiltered":true}))
        .unwrap();
    for (kind, wrong) in [
        (
            "field",
            json!({"dimensions":[2,0,1],"channels":1,"values":[0,1]}),
        ),
        (
            "field",
            json!({"dimensions":[1,1],"channels":1,"values":[0]}),
        ),
        (
            "field",
            json!({"dimensions":[1,1,1],"channels":5,"values":[0]}),
        ),
        (
            "field",
            json!({"dimensions":[1,1,1],"channels":1,"values":[0],"spacing":[1,0,2]}),
        ),
        (
            "field",
            json!({"dimensions":[1,1,1],"channels":1,"values":[1e39]}),
        ),
        ("sound", json!({"rate":7999,"channels":1,"samples":[0]})),
        ("sound", json!({"rate":192001,"channels":1,"samples":[0]})),
        ("sound", json!({"rate":48000,"channels":3,"samples":[0]})),
        ("sound", json!({"rate":48000,"channels":1,"samples":[]})),
        ("sound", json!({"rate":48000,"channels":1,"samples":[1e39]})),
        ("icons", json!({})),
        ("icons", json!({"icons":[]})),
        ("icons", json!({"icons":[["a","b"]],"size":513})),
        ("icons", json!({"icons":[["a","b"]],"join":"arcs"})),
        ("icons", json!({"icons":[["a","b"]],"sticker":[-0.1,0,0]})),
        ("icons", json!({"icons":[["a","b"]],"sticker":[0,4.1,0]})),
        ("vector", json!({})),
        ("vector", json!({"source":"a.svg","svg":"<svg/>"})),
        ("vector", json!({"source":"a.svg","tolerance":0})),
        ("vector", json!({"source":"a.svg","tolerance":1e39})),
        ("texture-array", json!({"kind":5,"layers":["a.png"]})),
        ("texture-array", json!({"kind":"height","layers":["a.png"]})),
        ("texture-array", json!({"layers":[]})),
        ("texture-array", json!({"layers":[""]})),
        ("environment", json!({"source":"studio.hdr"})),
        ("environment", json!({"source":{"faces":["a.hdr"]}})),
        (
            "environment",
            json!({"source":{"ktx2":"a.ktx2","equirect":"a.hdr"}}),
        ),
        (
            "environment",
            json!({"source":{"equirect":"a.hdr"},"samples":15}),
        ),
        (
            "environment",
            json!({"source":{"equirect":"a.hdr"},"ground":[0,-0.1,1],"hemisphere":"upper"}),
        ),
        (
            "environment",
            json!({"source":{"equirect":"a.hdr"},"sun":"auto"}),
        ),
        (
            "environment",
            json!({"source":{"equirect":"a.hdr"},"sunBoost":101}),
        ),
    ] {
        assert!(
            (source_schemas::get(kind).unwrap().schema)()
                .check(&wrong)
                .is_err(),
            "{kind} accepted {wrong}"
        );
    }
    ts.push_str(
        r#"
const fileVector: Vector = {source:"icon.svg"};
const defaultArray: TextureArray = {layers:["mask.png"]};
const imageSky: Environment = {source:{equirect:"sky.png"},hemisphere:"upper",ground:[0,0.5,1]};
const cubeSky: Environment = {source:{ktx2:"studio.ktx2"},prefiltered:true};
// @ts-expect-error Grid dimensions are a fixed three-axis tuple.
const badGrid: Field = {dimensions:[1,2],channels:1,values:[0]};
// @ts-expect-error PCM data is structured numbers, not an encoded string.
const badPcm: Sound = {rate:48000,channels:1,samples:"AA=="};
// @ts-expect-error Icons need named file-stem pairs.
const badIcons: Icons = {icons:["heart"]};
// @ts-expect-error SVG's internal join aliases are not author enum members.
const badJoin: Icons = {icons:[["a","b"]],join:"arcs"};
// @ts-expect-error A vector must name an actual source.
const missingSvg: Vector = {tolerance:0.1};
// @ts-expect-error No arbitrary parser block is exposed as an author escape hatch.
const vectorBlock: Vector = {source:"a.svg",config:{}};
// @ts-expect-error File and inline SVG branches are mutually exclusive.
const mixedVector: Vector = {source:"a.svg",svg:"<svg/>"};
// @ts-expect-error Texture kind names are not runtime numeric codes.
const numericKind: TextureArray = {kind:5,layers:["a.png"]};
// @ts-expect-error Six cubemap faces have fixed order and arity.
const shortCube: Environment = {source:{faces:["px.hdr","nx.hdr"]}};
// @ts-expect-error An environment has exactly one source branch.
const mixedSky: Environment = {source:{ktx2:"a.ktx2",equirect:"a.hdr"}};
// @ts-expect-error Sun selection uses the domain vocabulary.
const badSun: Environment = {source:{equirect:"sky.png"},sun:true};
// @ts-expect-error Author data is readonly, including nested arrays.
valueSound.samples[0] = 1;
"#,
    );
    let dir = temp::TempDir::new("source-data-schema");
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
}
