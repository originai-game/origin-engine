use super::*;

#[test]
fn material_destinations_cannot_escape_or_overwrite_inputs() {
    use crate::cook::source_schemas::world::materials::MaterialsInput;
    let library = serde_json::json!({"materials":[
        {"name":"brick","labels":["wall"],"tile":[2,1],"brief":"London stock brick, soot-darkened, lime mortar","relief":0.02,"surface":{"map":"textures/brick.png","normalMap":"textures/brick-n.png"}},
        {"name":"shopfront","labels":["shopfront"],"tile":[4,4],"facade":true,"brief":"a Victorian shopfront bay","surface":{"map":"textures/shop.png"}},
        {"name":"glass","labels":["glass"],"surface":{"color":[0.02,0.03,0.04]}}
    ]});
    for path in [
        "../brick.png",
        "/brick.png",
        "textures/../brick.png",
        "textures/./brick.png",
        "textures//brick.png",
        "C:\\brick.png",
    ] {
        let mut value = library.clone();
        value["materials"][0]["surface"]["map"] = path.into();
        assert!(
            requests(MaterialsInput::schema().decode(value).unwrap()).is_err(),
            "{path}"
        );
    }
    let mut value = library;
    value["materials"][0]["surface"]["normalMap"] = "textures/brick.source.png".into();
    assert!(
        requests(MaterialsInput::schema().decode(value).unwrap())
            .unwrap_err()
            .contains("used twice")
    );
}
