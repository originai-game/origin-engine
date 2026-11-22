//! A mesh file read by its content: GLB by its magic, glTF JSON by a
//! leading `{`, binary FBX by its magic and ASCII FBX by its header,
//! otherwise OBJ text.

use std::collections::BTreeMap;

use og_shape::tools::import::{Import, ImportError, Imported, Resolve, turn_part};
use og_shape::tools::place::{Axis, AxisTurn};

use crate::gltf::read::import_gltf;
use crate::obj::read::import_obj;

/// Imports a file by its content: GLB by its magic, glTF JSON by a
/// leading `{`, binary FBX by its magic and ASCII FBX by its header,
/// otherwise OBJ text.
pub fn import(bytes: &[u8], d: &Import, resolve: Resolve) -> Result<Imported, ImportError> {
    let first = bytes
        .iter()
        .find(|b| !b.is_ascii_whitespace() && **b != 0xEF && **b != 0xBB && **b != 0xBF);
    let head = &bytes[..bytes.len().min(4096)];
    let ascii_fbx = head.windows(18).any(|w| w == b"FBXHeaderExtension");
    if bytes.starts_with(b"glTF") || first == Some(&b'{') {
        let mut m = import_gltf(bytes, resolve)?;
        if d.turns() {
            // From glTF's Y up to the file's own axes, as the kernel
            // reads them.
            let y_up = AxisTurn::new(Some(Axis::PosY), None).map_err(ImportError::Unsupported)?;
            let c = d.axes()?.then(&y_up.inverse());
            for p in &mut m.parts {
                turn_part(p, &c);
            }
        }
        Ok(m)
    } else if bytes.starts_with(b"Kaydara FBX Binary") || ascii_fbx {
        if d.turns() {
            return Err(ImportError::Unsupported(
                "up, front or z_up for FBX: its GlobalSettings say its axes; turn it with `place`"
                    .into(),
            ));
        }
        crate::fbx::read::import_fbx(bytes, resolve)
    } else {
        import_obj(bytes, d, resolve)
    }
}

pub(crate) fn unique(name: &str, used: &mut BTreeMap<String, ()>) -> String {
    let base = if name.is_empty() { "part" } else { name };
    let mut n = base.to_string();
    let mut k = 1;
    while used.contains_key(&n) {
        n = format!("{base}.{k}");
        k += 1;
    }
    used.insert(n.clone(), ());
    n
}
