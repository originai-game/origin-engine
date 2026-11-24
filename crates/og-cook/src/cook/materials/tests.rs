use super::*;

const LIBRARY: &str = "type materials
version 1
---
material \"brick\" {
  label wall
  tile 2 1
  brief \"London stock brick, soot-darkened, lime mortar\"
  relief 0.02
  map \"textures/brick.png\"
  normalMap \"textures/brick-n.png\"
}
material \"shopfront\" {
  label shopfront
  tile 4 4
  facade true
  brief \"a Victorian shopfront bay\"
  map \"textures/shop.png\"
}
material \"glass\" {
  label glass
  color 0.02 0.03 0.04
}
";

#[test]
fn material_destinations_cannot_escape_or_overwrite_inputs() {
    for path in [
        "../brick.png",
        "/brick.png",
        "textures/../brick.png",
        "textures/./brick.png",
        "textures//brick.png",
        "C:\\brick.png",
    ] {
        let text = LIBRARY.replace("textures/brick.png", path);
        assert!(requests(&text).is_err(), "{path}");
    }
    let text = LIBRARY.replace("textures/brick-n.png", "textures/brick.source.png");
    assert!(requests(&text).unwrap_err().contains("used twice"));
}
