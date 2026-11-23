use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=og-assets.json");
    if env::var("CARGO_CFG_TARGET_ARCH").as_deref() != Ok("wasm32") {
        return;
    }
    // Native scenes read these same pinned files by path, including names
    // assembled by kit::layer_maps. Do not maintain a second wasm-only list.
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("og-assets.json")).unwrap()).unwrap();
    let mut source = String::from("match rel {\n");
    for file in manifest["files"].as_array().unwrap() {
        let path = file["path"].as_str().unwrap();
        let Some(rel) = path.strip_prefix("assets/") else {
            continue;
        };
        println!("cargo:rerun-if-changed={path}");
        source.push_str(&format!(
            "{rel:?} => include_bytes!({:?}) as &'static [u8],\n",
            root.join(path)
        ));
    }
    source.push_str("_ => return Err(format!(\"{rel}: not in the scene asset manifest\")),\n}");
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(out.join("scene_assets.rs"), source).unwrap();
}
