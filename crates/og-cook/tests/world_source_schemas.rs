//! Domain schema contracts: closed inputs, complete tagged geometry and real TS.
use super::temp;
use og_cook::cook::source_schemas::{
    self,
    world::{geometry::Geometry, pack::PackInput},
};
use og_core::reflect::source::typescript;
use serde_json::json;

#[test]
fn import_and_model_program_author_contracts() {
    use source_schemas::world::{import::ImportInput, model_program::ModelProgramInput};
    let animation = json!({"rate":60,"events":[{"clip":"walk","marker":"step","time":0.2}],
            "rig":{"$ogRef":"base/rig/person"},"libraries":[{"$ogRef":"base/model/walk"}],"footIk":true,
            "mirrors":[{"clip":"walk","name":"left"}],"compress":true,"tolerance":0.001,
            "clips":[{"name":"blink","channels":[{"node":"lid","path":"visibility","interpolation":"step",
                "keys":[{"time":0,"values":[1]},{"time":0.2,"values":[0]}]}]}]});
    let model = json!({"kind":"model","source":"assets/person.glb","relaxSkin":true,
        "lods":[1,0.6,0.2],"lodError":0.04,"raycast":true,"collider":"vhacd",
        "impostor":true,"impostorGrid":8,"impostorSize":128,"impostorBelow":false,"streamTextures":true,
        "process":{"unwrap":{"resolution":512}},"front":"-z","up":"+y","conform":"character",
        "voxScale":0.02,"voxOrigin":"bottom","voxFps":12,"voxClips":[{"name":"walk","first":1,"last":7}],
        "animation":animation});
    let imports = [
        model.clone(),
        json!({"kind":"texture","source":"assets/leaf.png","usage":"color","stream":true,
            "normal":{"kind":"image","source":"assets/leaf-n.png"},"sway":{"kind":"tiers","parts":"assets/leaf-p.png"}}),
        json!({"kind":"sound","source":"assets/music.ogg","codec":"opus","bitrate":96,"stream":false,"chunk":4}),
        json!({"kind":"font","source":"assets/grid.ttf","pixel":12}),
        json!({"kind":"tilemap","source":"sources/city.ldtk","unit":2,"level":"North","solid":[-1,2],"tileset":{"$ogRef":"base/tileset/city"}}),
        json!({"kind":"rive","source":"assets/menu.riv","artboards":["Menu"],"settings":{"Menu":{"stateMachine":"main","role":"button","name":"Open","fit":"scale-down","align":"top-right"}},"fonts":{"Body":{"source":"assets/body.ttf"}}}),
    ];
    let ImportInput::Model(defaults) = ImportInput::schema()
        .decode(json!({"kind":"model","source":"assets/person.glb"}))
        .unwrap()
    else {
        panic!()
    };
    assert!(!defaults.relax_skin);
    for (path, bad) in [
        ("/relaxSkin", json!(1)),
        ("/lods", json!([1, 0])),
        ("/lods", json!([1, 1.01])),
        ("/lodError", json!(0)),
        ("/impostorGrid", json!(33)),
        ("/impostorSize", json!(15)),
        ("/voxScale", json!(0)),
        ("/voxScale", json!(1e39)),
        ("/voxFps", json!(121)),
        ("/animation/tolerance", json!(0)),
        ("/animation/rate", json!(121)),
        ("/animation/libraries/0", json!("base/model/a")),
        ("/animation/clips/0/channels/0/keys/0/values", json!([1e39])),
    ] {
        let mut bad_model = model.clone();
        *bad_model.pointer_mut(path).unwrap() = bad;
        assert!(ImportInput::schema().decode(bad_model).is_err(), "{path}");
    }
    for bad in [
        json!({"kind":"model","source":"a.glb","match":"**"}),
        json!({"kind":"texture","source":"a.png","normal":{"kind":"auto","source":"b.png"}}),
        json!({"kind":"sound","source":"a.wav","bitrate":257}),
        json!({"kind":"font","source":"a.ttf","pixel":3}),
        json!({"kind":"rive","source":"a.riv","settings":{"Main":{"role":"invented"}}}),
    ] {
        assert!(ImportInput::schema().decode(bad).is_err());
    }

    // Exercise every operation through the public input schema. These are
    // independent structural examples, not a topologically valid single model.
    let mut ops: Vec<serde_json::Value> = serde_json::from_str(r##"[
        {"op":"frame","origin":[0,0,0],"normal":[0,0,1]},
        {"op":"bbox","ids":[0,{"id":1}],"sides":"top"},
        {"op":"face","id":0,"name":"top"},{"op":"edge","id":0,"name":"top"},
        {"op":"anchor","id":0,"name":"front"},{"op":"placement","id":0},
        {"op":"grid","nx":3,"ny":2,"px":1,"py":2,"stagger":true},
        {"op":"ring","n":8,"radius":3},{"op":"along","path":{"points":[[0,0,0],[1,2,3]],"closed":false}},
        {"op":"snap","ids":0,"points":[[1,2,3]],"direction":[0,0,-1]},
        {"op":"names","id":0},{"op":"measure","posed":true},{"op":"units","unit":"m"},{"op":"uvs"},
        {"op":"box","name":"b","width":"$width","depth":2,"height":3},
        {"op":"cylinder","name":"c","radius":1,"radiusTop":0.5,"height":3,"segments":"$sides"},
        {"op":"cone","name":"c","radius":1,"height":3},{"op":"sphere","name":"s","radius":2,"axis":"+Z"},
        {"op":"dome","name":"d","radius":1},{"op":"tube","name":"t","outer":2,"inner":1,"height":3},
        {"op":"torus","name":"t","radius":2,"tube":0.2,"tubeSegments":8},{"op":"wedge","name":"w","width":2,"depth":3,"height":4},
        {"op":"extrude","name":"e","profile":{"rect":{"width":2,"height":3}},"height":4},
        {"op":"extrude","name":"e","profile":{"circle":{"radius":1}},"until":{"part":0,"face":"top"}},
        {"op":"revolve","name":"r","profile":{"slot":{"length":3,"width":1}}},
        {"op":"sweep","name":"s","profile":{"circle":{"radius":0.1}},"path":[[0,0,0],{"arc":{"through":[1,1,0],"to":[2,0,0]}},{"tangentArc":{"to":[3,1,0]}},{"spline":{"points":[[4,2,0],[5,3,0]]}}]},
        {"op":"loft","name":"l","sections":[{"profile":{"circle":{"radius":1}},"z":0},{"profile":{"circle":{"radius":2}},"z":3}]},
        {"op":"thread","name":"t","radius":2,"pitch":0.2,"length":3,"depth":0.1},
        {"op":"wire","name":"w","path":{"helix":{"radius":2,"pitch":1,"turns":3}},"diameter":0.1},
        {"op":"body","name":"b","plan":2,"top":[[0,1],[3,2]],"bottom":{"points":[[0,0],[3,-1]],"smooth":true}},
        {"op":"knurl","name":"k","diameter":2,"height":3},{"op":"gear","name":"g","teeth":12,"module":1,"width":2},
        {"op":"group","name":"g","children":[0,1]},{"op":"wrap","name":"w","parts":0},
        {"op":"attach","to":0,"name":"a","children":[1]},{"op":"copy","id":0,"name":"c"},
        {"op":"cut","target":0,"tool":1},{"op":"join","target":0,"tool":1},{"op":"intersect","target":0,"tool":1},
        {"op":"merge","target":0,"tool":1},{"op":"delete","id":0},{"op":"seat","host":0,"part":1},
        {"op":"scale","id":0,"x":2,"y":3,"z":4},{"op":"pocket","id":0,"profile":{"rect":{"width":1,"height":2}},"depth":1},
        {"op":"shell","id":0,"thickness":0.1,"open":["top"]},{"op":"dent","id":0,"at":[[1,2,3]],"radius":1,"depth":0.1},
        {"op":"subdivide","id":0,"levels":2,"sharp":"top"},{"op":"displace","id":0,"noise":"ridged","scale":1,"amount":0.1,"seed":7},
        {"op":"fillet","id":0,"radius":0.1,"edges":["top"]},{"op":"chamfer","id":0,"distance":0.1,"distance2":0.2},
        {"op":"soften","id":0,"radius":0.2,"angle":0.5},
        {"op":"material","ids":0,"color":"#abcdef","surface":{"density":64,"layers":[{"pattern":"veins","axis":"+X","color":[0.1,0.2,0.3],"relief":0.1}]}},
        {"op":"pose","joint":"hinge","value":0.5},{"op":"pose","clip":"open","time":0.2},
        {"op":"clip","name":"open","keys":[{"time":0,"pose":{"hinge":0}},{"time":1,"pose":{"hinge":0.5}}]},
        {"op":"skin","id":0,"bones":[{"name":"root","pivot":[0,0,0],"regions":["body"]}],"blend":0.1},
        {"op":"skin","id":0,"joints":[{"name":"root","at":[0,0,0]}]},
        {"op":"humanoid","id":0,"bind":false},
        {"op":"morph","id":0,"name":"target","target":1},{"op":"morph","id":0,"name":"delta","deltas":[[1,0,0]]},
        {"op":"morph","id":0,"name":"region","regions":["top"],"offset":[0,1,0],"scale":[1,2,1]},
        {"op":"morph","id":0,"name":"offset","offset":[0,1,0]},{"op":"morph","id":0,"name":"scale","scale":[1,2,1]}
    ]"##).unwrap();
    let frame = json!({"origin":[1,2,3],"x":[1,0,0],"y":[0,1,0],"z":[0,0,1]});
    for op in [
        "offset",
        "translate",
        "turn",
        "local",
        "at",
        "transform",
        "split",
    ] {
        let mut value = json!({"op":op,"frame":frame});
        match op {
            "offset" => value["distance"] = json!(2),
            "translate" | "turn" => (),
            _ => value["id"] = json!(0),
        }
        ops.push(value);
    }
    ops.extend([json!({"op":"compose","frame":frame,"local":frame}),
        json!({"op":"assembly","name":"a","children":[0,1],"anchors":{"tip":frame}}),
        json!({"op":"mirror","id":0,"plane":frame,"name":"m"}),
        json!({"op":"hole","id":0,"diameter":1,"at":[frame],"counterbore":[2,0.1],"countersink":[3,0.5]}),
        json!({"op":"joint","name":"hinge","child":1,"parent":0,"on":frame,"kind":"revolute","axis":[0,1,0],"range":[0,1],"mimic":{"joint":"other","multiplier":-1}}),
    ]);
    let profiles: Vec<serde_json::Value> = serde_json::from_str(r#"[
        {"rect":{"width":2,"height":3,"radius":0.1}}, {"circle":{"radius":1,"segments":16}},
        {"ellipse":{"rx":2,"ry":1,"segments":16}}, {"slot":{"length":3,"width":1}},
        {"polygon":{"sides":6,"flats":2}}, {"poly":{"points":[[0,0],{"x":2,"y":0,"radius":0.1,"bulge":0.2},[0,3]]}},
        {"gear":{"teeth":12,"module":1,"pressureAngle":0.3}},
        {"union":[{"circle":{"radius":1}},{"rect":{"width":3,"height":1}}]},
        {"subtract":[{"circle":{"radius":2}},{"circle":{"radius":1}}]},
        {"intersect":[{"circle":{"radius":1}},{"rect":{"width":3,"height":1}}]},
        {"offset":{"profile":{"circle":{"radius":1}},"distance":0.2}},
        {"translate":{"profile":{"circle":{"radius":1}},"x":2,"y":3}},
        {"rotate":{"profile":{"rect":{"width":2,"height":3}},"angle":0.5}},
        {"scale":{"profile":{"circle":{"radius":1}},"x":2,"y":3}},
        {"superellipse":{"rx":2,"ry":1,"exponent":3,"segments":16}},
        {"spline":{"points":[[0,0],[2,1],[0,3]]}},
        {"path":[[0,0],[2,0],{"arc":{"through":[3,1],"to":[2,2]}},{"tangentArc":{"to":[1,3]}},{"spline":{"points":[[0,3],[0,0]]}}]},
        {"stroke":{"path":[[0,0],[2,1]],"width":0.2,"join":"miter","roundCaps":true,"closed":false}},
        {"hull":[{"circle":{"radius":1}},{"rect":{"width":3,"height":1}}]},
        {"mirror":{"profile":{"rect":{"width":2,"height":3}},"axis":"x"}},
        {"text":{"text":"Origin","font":"assets/body.ttf","size":1,"spacing":0.1}}
    ]"#).unwrap();
    for (i, profile) in profiles.into_iter().enumerate() {
        ops.push(json!({"op":"extrude","name":format!("profile{i}"),"profile":profile,"height":2}));
    }
    ops.extend([
        json!({"op":"loft","name":"placed","sections":[{"profile":{"circle":{}},"at":frame}],"curve":"catmullRom","closed":true}),
        json!({"op":"material","ids":0,"color":[0.1,0.2,0.3],"map":"assets/color.png","mapSize":2,"clearcoat":0.3,"sheenColor":"#123456","transmission":0.2,"iridescence":0.1}),
        json!({"op":"material","ids":0,"color":"#fedcba"}),
    ]);
    let program = json!({"name":"vocabulary","params":{"width":2,"sides":16},"ops":ops});
    let mut ts = typescript(ImportInput::schema().document(), "Import").unwrap();
    ts.push_str(&typescript(ModelProgramInput::schema().document(), "Program").unwrap());
    for (i, value) in imports.iter().enumerate() {
        ImportInput::schema().decode(value.clone()).unwrap();
        ts.push_str(&format!("const imported{i}: Import = {value};\n"));
    }
    ModelProgramInput::schema().decode(program.clone()).unwrap();
    ts.push_str(&format!("const program: Program = {program};\n"));
    // The structural value can enter the actual shared kernel unchanged.
    let executable = json!({"name":"box","params":{"width":2},"ops":[
        {"op":"units","unit":"m"},{"op":"box","name":"body","width":"$width * 2","height":3,"depth":5} ]});
    ModelProgramInput::schema()
        .decode(executable.clone())
        .unwrap();
    let resolved =
        og_shape::model::program::with_params(&executable.to_string(), &Default::default())
            .unwrap();
    let executed = og_shape::model::program::run(&resolved).unwrap();
    assert_eq!(executed.name(), "box");
    let measured: serde_json::Value =
        serde_json::from_str(&og_shape::model::program::measures_json(&executed).unwrap()).unwrap();
    assert!((measured["volume"].as_f64().unwrap() - 60.0).abs() < 1e-10);
    for bad in [
        json!({"op":"box","name":"x","width":true,"depth":2,"height":3}),
        json!({"op":"cylinder","name":"c","radius":1,"height":2,"segments":1.5}),
        json!({"op":"wire","name":"w","diameter":1,"path":[[0,0,0],{"arc":{"through":[1,2],"to":[3,4,5]}}]}),
        json!({"op":"pose","joint":"a","value":1,"clip":"c","time":2}),
        json!({"op":"skin","id":0,"bones":[],"joints":[]}),
        json!({"op":"morph","id":0,"name":"a","target":1,"deltas":[]}),
        json!({"op":"extrude","name":"a","profile":{"circle":{}},"height":2,"until":{"part":0,"face":"top"}}),
        json!({"op":"extrude","name":"a","profile":{"rect":{},"circle":{}},"height":2}),
        json!({"op":"material","ids":0,"map":"a.png","surface":{}}),
        json!({"op":"material","ids":0,"mapSize":2}),
        json!({"op":"wire","name":"w","diameter":1,"path":[{"arc":{"through":[0,1,0],"to":[2,3,4]}}]}),
        json!({"op":"loft","name":"l","sections":[{"profile":{"circle":{}},"z":0},{"profile":{"circle":{}},"at":frame}]}),
    ] {
        assert!(
            ModelProgramInput::schema()
                .decode(json!({"name":"bad","ops":[bad]}))
                .is_err()
        );
    }
    ts.push_str(r#"
// @ts-expect-error Closed imported models do not take sound codec options.
const badImport: Import = {kind:"model",source:"a.glb",codec:"opus"};
// @ts-expect-error Skin relaxation is a boolean, not a pass count.
const badRelax: Import = {kind:"model",source:"a.glb",relaxSkin:12};
// @ts-expect-error Domain options do not create an implicit directory-matching import rule.
const oldSidecar: Import = {kind:"model",source:"a.glb",match:"**"};
// @ts-expect-error Ref targets are content identities, not cooked handles.
const badRig: Import = {kind:"model",source:"a.glb",animation:{rig:7}};
// @ts-expect-error No arbitrary operation payload or old JSON source escape.
const oldProgram: Program = {name:"a",ops:[{op:"raw",text:"{}"}]};
// @ts-expect-error Operation discriminants select closed fields.
const badOp: Program = {name:"a",ops:[{op:"box",name:"b",width:1,height:2,depth:3,teeth:16}]};
// @ts-expect-error 2D profile points are not 3D vectors.
const badProfile: Program = {name:"a",ops:[{op:"extrude",name:"e",profile:{poly:{points:[[0,1,2]]}},height:3}]};
// @ts-expect-error Pose modes are exclusive.
const mixedPose: Program = {name:"a",ops:[{op:"pose",joint:"hinge",value:1,clip:"c",time:1}]};
// @ts-expect-error Skin modes are exclusive.
const mixedSkin: Program = {name:"a",ops:[{op:"skin",id:0,bones:[],joints:[]}]};
// @ts-expect-error Profiles have exactly one tag.
const mixedProfile: Program = {name:"a",ops:[{op:"extrude",name:"e",profile:{rect:{},circle:{}},height:3}]};
// @ts-expect-error A path starts with a point, not a curve continuation.
const noStart: Program = {name:"a",ops:[{op:"wire",name:"w",diameter:1,path:[{spline:{points:[[0,1,2]]}}]}]};
const sharedPaint = {op:"material" as const,ids:0,map:"a.png",surface:{}};
// @ts-expect-error Material map and surface are exclusive, even for a shared value.
const mixedPaint: Program = {name:"a",ops:[sharedPaint]};
// @ts-expect-error Extrusion height and until are exclusive.
const mixedExtrude: Program = {name:"a",ops:[{op:"extrude",name:"e",profile:{circle:{}},height:2,until:{part:0,face:"top"}}]};
// @ts-expect-error Sections cannot mix height and frame placement.
const mixedSections: Program = {name:"a",ops:[{op:"loft",name:"l",sections:[{profile:{circle:{}},z:0},{profile:{circle:{}},at:{origin:[0,0,1],x:[1,0,0],y:[0,1,0],z:[0,0,1]}}]}]};
// @ts-expect-error Generated programs are readonly.
program.ops.push({op:"uvs"});
"#);
    let dir = temp::TempDir::new("import-program-schema");
    std::fs::write(dir.join("inputs.ts"), ts).unwrap();
    let result = std::process::Command::new("tsrs")
        .args(["--ignoreConfig", "--strict", "--noEmit", "inputs.ts"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn model_and_materials_author_contracts() {
    use source_schemas::world::{materials::MaterialsInput, model::ModelInput};
    let surface = json!({"color":[0.2,0.4,0.6],"opacity":0.9,"metalness":0.7,"roughness":0.3,
        "emissive":[1,2,3],"emissiveIntensity":2,"doubleSided":true,"alphaTest":0.4,
        "map":"stone.png","normalMap":"n.png","normalScale":-1,"metalnessMap":"mr.png","aoMap":"ao.png",
        "transmission":0.5,"ior":1.5,"thickness":0.2,"clearcoat":0.7,"clearcoatRoughness":0.2,
        "translucency":0.4,"domeNormals":0.6,"season":1,"autumnColor":[0.3,0.1,0.01],
        "autumnColor2":[0.4,0.2,0.01],"seasonShift":-10,"grime":0.1,"grimeColor":[0.1,0.02,0.03],
        "grimeHeight":0.4,"stains":0.2,"stochastic":0.7});
    let plant = json!({"kind":"oak","levels":3,"seed":7,"growth":"deciduous",
            "trunk":{"length":3,"radius":0.2,"sections":8,"segments":6,"taper":0.5,"children":2},
            "branch1":{"angle":45,"start":0.3,"gnarliness":0.1,"twist":0.2},"branch2":{},"branch3":{},
            "force":[0,1,0,0.01],"barkColor":[0.1,0.2,0.03],"material":"stone",
            "leaves":{"shape":"blade","count":20,"rows":3,"size":0.2,"width":0.5,"sizeVariance":0.3,
                "start":0.1,"end":0.8,"angle":30,"facing":0.6,"color":[0.1,0.3,0.02],"color2":[0.2,0.4,0.03],
                "colorVariation":0.2,"translucency":0.5,"domeNormals":0.3,"season":1,"seasonShift":3},
            "flowers":{"share":0.5,"count":4,"petals":7,"size":0.1,"width":0.2,"cup":25,"color":[1,0.2,0.4],
                "color2":[1,0.4,0.5],"centerColor":[1,0.8,0.1]},
            "branches":[{"parent":-1,"path":[[0,0,0],[0,1,0]],"radii":[0.1,0.01],"segments":6}],
            "bounds":[[-1,0,-2],[2,5,3]],"path":[[0,0,0],[0,3,0]],
            "leafGeometry":{"kind":"planeGeometry","desc":{"width":0.1}},
            "flowerGeometry":{"kind":"circleGeometry","desc":{}}});
    let process = json!({"repair":{"weld":0.001,"weldAll":true,"floaters":0.1,"floaterFraction":0.02,"nonManifold":true,
            "holeEdges":5,"holesThrough":true,"thicken":0.01,"fuse":true,"minEdge":0.001},
            "decimate":{"target":50,"ratio":0.5,"maxError":0.01,"lockBorder":true,"mergeCharts":0.001,"crossColorBorders":true},
            "unwrap":{"angle":60,"resolution":512,"padding":2,"shared":true,"conformal":true,"seams":true},
            "bake":{"size":128,"distance":0.1,"padding":4,"dilate":2,"normals":true},
            "project":{"size":128,"views":[{"image":"front.png","position":[0,2,5],"target":[0,1,0],
                "projection":{"kind":"perspective","fov":45}},{"image":"top.png","position":[0,5,0],"target":[0,0,0],
                "projection":{"kind":"orthographic","ortho":3}}]},
            "transfer":{"source":{"$ogRef":"base/model/painted"},"size":128,"padding":4},
            "rig":{"kind":"joints","joints":[{"name":"root","parent":"","position":[0,0,0]}]}});
    let model = json!({"kind":"geometry","desc":{"conform":"character","items":[
        {"name":"stone","input":{"kind":"material","desc":surface}},
        {"name":"base","input":{"kind":"pivot","desc":{"transform":{"position":[1,2,3]}}}},
        {"name":"upper","input":{"kind":"pivot","desc":{"parent":"base"}}},
        {"name":"gate","input":{"kind":"part","desc":{"material":"stone","pivot":"upper",
            "geometry":{"kind":"boxGeometry","desc":{"width":1,"height":3,"depth":0.4}},
            "placements":[{"transform":{"position":[2,3,5]},"mirror":"x"},{}]}}},
        {"name":"tree","input":{"kind":"plant","desc":plant}}],"process":process,
        "fracture":{"style":"brick","chunks":8,"children":3,"seed":9,"pattern":"slab","brick":[0.2,0.1],"grain":"x",
            "rings":2,"impact":{"position":[1,2,3],"radius":0.4},"anchor":{"kind":"supports","bottom":true,"ends":true},
            "interiorScale":2,"interior":{},"mortar":{}}}});
    let materials = json!({"materials":[{"name":"stone","labels":["wall","roof"],"weight":2,"tile":[2,3],
        "vary":{"value":0.2,"warmth":0.1},"brief":"rough stone","generate":{"source":"stone-source.png"},
        "facade":true,"relief":-0.3,"roughnessRange":[0.2,0.8],"occlusion":{"strength":0.7,"radius":5},"surface":surface}]});
    ModelInput::schema().decode(model.clone()).unwrap();
    MaterialsInput::schema().decode(materials.clone()).unwrap();
    let program = json!({"kind":"program","desc":{"program":{"$ogRef":"base/model-program/gate"},"params":{"height":3},
        "atlas":128,"process":{"rig":{"kind":"auto"}},"materials":[{"name":"stone","color":[0.2,0.3,0.4],
            "normalMap":"n.png","normalScale":1,"roughnessMap":"mr.png","emissiveMap":"e.png","aoMapIntensity":0.5,
            "sheen":[0.1,0.2,0.3],"sheenRoughness":0.4,"iridescence":0.3,"transparent":true}]}});
    ModelInput::schema().decode(program.clone()).unwrap();
    let mut ts = String::new();
    for (kind, name, value) in [
        ("model", "Model", model.clone()),
        ("materials", "Materials", materials.clone()),
    ] {
        let schema = (source_schemas::get(kind).unwrap().schema)();
        ts.push_str(&typescript(schema.document(), name).unwrap());
        ts.push_str(&format!("const value{name}: {name} = {value};\n"));
    }
    ts.push_str(&format!("const program: Model = {program};\n"));
    for (path, bad) in [
        ("/desc/items/3/input/desc/placements", json!([])),
        ("/desc/items/0/input/desc/ior", json!(3.01)),
        ("/desc/items/0/input/desc/autumnColor", json!([1, -0.01, 0])),
        ("/desc/items/4/input/desc/leaves/count", json!(2001)),
        ("/desc/items/4/input/desc/trunk/segments", json!(2)),
        ("/desc/items/4/input/desc/leaves/color", json!([1, 1e39, 0])),
        (
            "/desc/process/project/views/0/projection",
            json!({"kind":"perspective","fov":45,"ortho":2}),
        ),
        ("/desc/process/project/views/0/projection/fov", json!(0)),
        ("/desc/process/unwrap/angle", json!(90)),
        ("/desc/fracture/chunks", json!(257)),
        ("/desc/fracture/brick", json!([0.009, 0.1])),
        ("/desc/fracture/impact/radius", json!(0)),
        ("/desc/process/transfer/source", json!("base/model/a")),
    ] {
        let mut wrong = model.clone();
        *wrong.pointer_mut(path).unwrap() = bad;
        assert!(
            ModelInput::schema().decode(wrong.clone()).is_err(),
            "accepted {wrong}"
        );
    }
    for (key, bad) in [
        ("labels", json!([])),
        ("tile", json!([2, 0])),
        ("roughnessRange", json!([0.2, 1.01])),
        ("occlusion", json!({"strength":0.5,"radius":2.5})),
        ("vary", json!({"value":0.51})),
    ] {
        let mut wrong = materials.clone();
        wrong["materials"][0][key] = bad;
        assert!(MaterialsInput::schema().decode(wrong).is_err());
    }
    ts.push_str(r#"
// @ts-expect-error Reflected Transform fields are shared, not an untyped patch.
const badTransform: Model = {kind:"geometry",desc:{items:[{name:"p",input:{kind:"pivot",desc:{transform:{typo:3}}}}]}};
// @ts-expect-error Program dependencies are content references, not paths.
const pathProgram: Model = {kind:"program",desc:{program:"gate.shape.json"}};
// @ts-expect-error Geometry and program branches cannot be mixed.
const mixedModel: Model = {kind:"geometry",desc:{items:[],program:{$ogRef:"base/model-program/gate"}}};
// @ts-expect-error A projection chooses one camera mechanism.
const mixedProjection: Model = {kind:"geometry",desc:{items:[],process:{project:{views:[{image:"a.png",position:[0,1,2],target:[0,0,0],projection:{kind:"orthographic",ortho:2,fov:45}}]}}}};
// @ts-expect-error Source material maps are image inputs, never runtime handles.
const mapHandle: Materials = {materials:[{name:"a",labels:["wall"],surface:{map:3}}]};
// @ts-expect-error glTF source materials do not accept runtime side fields.
const runtimeMaterial: Materials = {materials:[{name:"a",labels:["wall"],surface:{side:2}}]};
// @ts-expect-error Authored declarations are readonly and ordered.
valueModel.desc.items.push({name:"p",input:{kind:"pivot",desc:{}}});
"#);
    let dir = temp::TempDir::new("model-materials-schema");
    std::fs::write(dir.join("inputs.ts"), ts).unwrap();
    let result = std::process::Command::new("tsrs")
        .args(["--ignoreConfig", "--strict", "--noEmit", "inputs.ts"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn district_and_grammar_author_contracts() {
    let grammar = json!({"params":{"floor":3.2},"start":"Lot","rules":{
        "Lot":[{"op":"recess","depth":0.1,"jamb":"Wall"},{"op":"detail","level":2},
            {"op":"extrude","depth":"floor","faces":{"sides":"Facade","top":"Roof"}}],
        "Facade":[{"op":"split","axis":"y","parts":[{"size":"abs","length":1,"rule":"Wall"},
            {"size":"repeat","length":"floor","rule":"Wall"},{"size":"fill","length":1,"rule":"Wall"}]}],
        "Wall":[{"op":"label","label":"stone"}],
        "Roof":[{"op":"roof","roof":{"kind":"gable","angle":35},"slope":"Wall","overhang":0.2}],
        "Hip":[{"op":"roof","roof":{"kind":"hip","angle":25},"slope":"Wall"}],
        "Flat":[{"op":"roof","roof":{"kind":"flat"},"slope":"Wall"}],
        "Inset":[{"op":"inset","width":0.2,"border":"Wall","inner":"Wall"}],
        "Insert":[{"op":"insert","model":{"$ogRef":"base/model/gate"}}],
        "Pick":[{"op":"pick","options":[{"weight":2,"rule":"Wall"}]}],
        "Case":[{"op":"case","arms":[{"condition":{"kind":"wider","value":4},"rule":"Wall"},
            {"condition":{"kind":"taller","value":5},"rule":"Wall"},
            {"condition":{"kind":"param","name":"floor","value":3.2},"rule":"Wall"}],"otherwise":"Wall"}],
        "Empty":[{"op":"drop"}]}});
    let district = json!({"seed":7,"region":{"kind":"area","points":[[0,0],[100,0],[0,60]]},
        "streets":{"grid":{"size":[15,22],"turn":35},"kerb":0.15,
            "streets":[{"name":"market","points":[[1,2],[30,7]],"width":4}]},
        "lots":{"width":[3,7],"depth":[4,8]},"materials":{"$ogRef":"base/materials/stone"},
        "buildings":{"density":0.6,"height":[3,12],"heightNoise":{"frequency":0.03,"share":0.7},
            "rules":[{"grammar":{"$ogRef":"base/grammar/blocks"},"weight":2}]},
        "props":[{"name":"lamp","model":{"$ogRef":"base/model/lamp"},"collider":{"kind":"box","size":[1,3,2]}}],
        "cells":{"size":64,"load":90},"terrain":{"terrain":{"$ogRef":"base/terrain/north"},"depth":0.3}});
    let mut ts = String::new();
    for (kind, name, value) in [
        ("grammar", "Grammar", grammar),
        ("district", "District", district.clone()),
    ] {
        let schema = (source_schemas::get(kind).unwrap().schema)();
        schema
            .check(&value)
            .unwrap_or_else(|e| panic!("{kind}: {e}"));
        ts.push_str(&typescript(schema.document(), name).unwrap());
        ts.push_str(&format!("const value{name}: {name} = {value};\n"));
    }
    let mut along = district.clone();
    along["region"] = json!({"kind":"along","street":{"points":[[0,0],[0,15]]},"depth":10});
    along["props"][0]["collider"] = json!({"kind":"capsule","radius":0.4,"height":0});
    source_schemas::world::district::DistrictInput::schema()
        .decode(along.clone())
        .unwrap();
    ts.push_str(&format!("const along: District = {along};\n"));
    for (path, bad) in [
        ("/buildings/height", json!([-1, 5])),
        ("/buildings/density", json!(1.01)),
        ("/props/0/collider/size", json!([1, 0, 2])),
        ("/cells/size", json!(7.99)),
        ("/terrain/depth", json!(1e39)),
        ("/materials", json!("base/materials/stone")),
        (
            "/region",
            json!({"kind":"area","points":[],"street":{"points":[]}}),
        ),
    ] {
        let mut invalid = district.clone();
        *invalid.pointer_mut(path).unwrap() = bad;
        assert!(
            source_schemas::world::district::DistrictInput::schema()
                .decode(invalid.clone())
                .is_err(),
            "accepted {invalid}"
        );
    }
    for op in [
        json!({"op":"detail","level":17}),
        json!({"op":"insert","model":3}),
        json!({"op":"pick","options":[]}),
        json!({"op":"drop","depth":2}),
        json!({"op":"roof","roof":{"kind":"flat","angle":30},"slope":"Wall"}),
        json!({"op":"split","axis":"z","parts":[]}),
    ] {
        assert!(
            source_schemas::world::grammar::GrammarInput::schema()
                .decode(json!({"start":"A","rules":{"A":[op]}}))
                .is_err()
        );
    }
    ts.push_str(r#"
// @ts-expect-error Branches are closed; area cannot include an along street.
const mixedRegion: District["region"] = {kind:"area",points:[],street:{points:[]}};
// @ts-expect-error No parser source-text escape hatch.
const rawGrammar: Grammar = {source:"rule A { drop }"};
// @ts-expect-error Each operation has its own fields.
const mixedOperation: Grammar = {start:"A",rules:{A:[{op:"drop",depth:2}]}};
// @ts-expect-error Flat roofs cannot carry an angle.
const flatAngle: Grammar = {start:"A",rules:{A:[{op:"roof",roof:{kind:"flat",angle:30},slope:"A"}]}};
// @ts-expect-error Model dependencies are content references, not handles.
const handle: Grammar = {start:"A",rules:{A:[{op:"insert",model:3}]}};
// @ts-expect-error Rules retain readonly operation order.
valueGrammar.rules.Lot.push({op:"drop"});
"#);
    let dir = temp::TempDir::new("district-grammar-schema");
    std::fs::write(dir.join("inputs.ts"), ts).unwrap();
    let result = std::process::Command::new("tsrs")
        .args(["--ignoreConfig", "--strict", "--noEmit", "inputs.ts"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}

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
        ("assets/icons", 48, 8)
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
