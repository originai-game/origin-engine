//! Scene/prefab author contract: checked TS records become native packed rows.
use super::temp;
use og_cook::cook::{CookOptions, cook_project};
use serde_json::json;

#[test]
fn native_scene_prefab_records_round_trip_and_reject_invalid_source() {
    let dir = temp::TempDir::new("native-scene");
    crate::init_project(&dir);
    let write = |file: &str, text: &str| {
        let path = dir.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    };
    write(
        "project.ts",
        "import {defineProject} from '@origingame/engine/author'; export default defineProject({id:'native-scenes',entry:'src/main.ts',schemas:['src/schema/records.ts']});",
    );
    write(
        "src/schema/records.ts",
        "import * as s from '@origingame/engine/schema'; export const Profile=s.defineComponent('game.Profile',{label:s.str(8,'北'),level:s.u32(7),ammo:s.list(s.u32(),4),position:s.Vector3({x:1,y:3,z:7})}); export const Settings=s.defineResource('game.Settings',{left:s.u32(11),right:s.u32(29)});",
    );
    write(
        "content/prefab.ts",
        "import {file} from '@origingame/engine/author'; export default file.prefab({entity:{name:'root',components:{'game.Profile':{level:13}},children:[{name:'child',components:{'game.Profile':{label:'南',ammo:[3,11],position:[2,5,17]}}}]}});",
    );
    let scene = "import {file,ref} from '@origingame/engine/author';\nexport default file.scene({entities:[\n{name:'placed',prefab:ref('base/prefab'),edits:[{kind:'override',path:'child',components:{'game.Profile':{level:23}}}]}\n],resources:{'game.Settings':{right:31},'og.Physics':{profile:2}},paths:{rail:{points:[[0,0,0],[1,3,7]],kind:'linear'}}});";
    write("content/scene.ts", scene);
    let options = CookOptions {
        threads: 2,
        ..Default::default()
    };
    let report = cook_project(&dir, &options).unwrap();
    assert!(report.errors().is_empty(), "{:?}", report.errors());
    let bundle = og_cook::runtime::Bundle::open(&dir.join("cooked")).unwrap();
    assert_eq!(
        bundle.asset("base/scene").unwrap().modules,
        ["physics", "physics2d"]
    );
    let scene = bundle.load_scene("base/scene").unwrap();
    assert_eq!(
        scene
            .entities
            .iter()
            .map(|e| e.name.as_str())
            .collect::<Vec<_>>(),
        ["placed", "placed/child"]
    );
    assert!(scene.entities[1].prefab.is_some());
    assert_eq!(scene.entities[0].line, Some(3));
    let table = scene
        .components
        .iter()
        .find(|t| t.name == "game.Profile")
        .unwrap();
    let layout = og_core::Layout::of(&table.schema);
    let field = |entity, name: &str| {
        &table.get(entity).unwrap()[layout.range(table.schema.field_index(name).unwrap())]
    };
    assert_eq!(og_core::unpack_str(field(0, "label")), "北");
    assert_eq!(
        u32::from_le_bytes(field(0, "level").try_into().unwrap()),
        13
    );
    assert_eq!(
        u32::from_le_bytes(field(1, "level").try_into().unwrap()),
        23
    );
    assert_eq!(og_core::unpack_str(field(1, "label")), "南");
    let position: Vec<_> = field(1, "position")
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| f32::from_le_bytes(*b))
        .collect();
    assert_eq!(position, [2., 5., 17.]);
    let ammo = &table.schema.fields[table.schema.field_index("ammo").unwrap()];
    assert_eq!(
        og_core::list_elements(ammo.ty, field(1, "ammo"))
            .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
            .collect::<Vec<_>>(),
        [3, 11]
    );
    assert_eq!(scene.paths[0].1.points[1].to_array(), [1., 3., 7.]);
    assert_eq!(
        scene.resources[0]
            .schema
            .fields
            .iter()
            .map(|f| f.name.as_str())
            .collect::<Vec<_>>(),
        ["right"]
    );
    assert_eq!(
        u32::from_le_bytes(scene.resources[0].bytes.as_slice().try_into().unwrap()),
        31
    );
    let again = cook_project(&dir, &options).unwrap();
    assert!(again.errors().is_empty(), "{:?}", again.errors());
    for (patch, message) in [
        (json!({"label":"北北北"}), "UTF-8"),
        (json!({"level":"three"}), "integer"),
        (json!({"unknown":3}), "Additional properties"),
    ] {
        write(
            "content/scene.ts",
            &format!(
                "import {{file}} from '@origingame/engine/author';\nexport default file.scene({{entities:[\n{{name:'bad',components:{{'game.Profile':{patch}}}}}\n]}});"
            ),
        );
        let error = match cook_project(&dir, &options) {
            Err(e) => e.to_string(),
            Ok(r) => r.errors().join("\n"),
        };
        assert!(
            error.contains("content/scene.ts:") && error.contains(message),
            "{error}"
        );
    }
    std::fs::remove_file(dir.join("content/scene.ts")).unwrap();
    write(
        "assets/removed.ogt",
        "type scene\nversion 1\n---\nentity \"old\" {}\n",
    );
    let result = cook_project(&dir, &options);
    let error = match result {
        Err(e) => e.to_string(),
        Ok(r) => r.errors().join("\n"),
    };
    assert!(
        !error.is_empty(),
        "removed author OGT must not silently cook"
    );
}
