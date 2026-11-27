//! Cooked shader data. Author declarations enter through the cook's typed
//! schemas; runtime loading never parses an author language or opens WGSL files.
use crate::format::{self, ShaderParamRow};
use crate::runtime::{LoadError, invalid};
use og_blob::{Blob, BlobWriter};
use og_render_data::{ParamKind, ShaderParam};

/// Shared payload of material shaders, post passes and UI materials. The root
/// schema owns its mode; names are runtime content IDs, not stored in the blob.
pub fn write(w: &mut BlobWriter, code: &str, params: &[ShaderParam]) {
    let mut names = Vec::new();
    let rows: Vec<_> = params
        .iter()
        .map(|p| {
            let offset = names.len() as u32;
            names.extend_from_slice(p.name.as_bytes());
            ShaderParamRow {
                name_offset: offset,
                name_len: p.name.len() as u32,
                components: p.kind.count() as u32,
                x: p.default[0],
                y: p.default[1],
                z: p.default[2],
                w: p.default[3],
            }
        })
        .collect();
    w.raw("code", code.as_bytes().to_vec(), 1)
        .raw("names", names, 1)
        .table("params", &rows, format::row_hash::<ShaderParamRow>());
}

/// Checked blob data, before the shader family's existing domain validation.
pub fn read(blob: &Blob, code_bytes: u32) -> Result<(String, Vec<ShaderParam>), LoadError> {
    let text = |bytes: &[u8]| {
        std::str::from_utf8(bytes)
            .map(str::to_string)
            .map_err(|_| invalid(blob, "shader strings are not UTF-8"))
    };
    let code = blob.raw("code")?;
    if code.len() != code_bytes as usize {
        return Err(invalid(
            blob,
            "shader code length disagrees with its metadata",
        ));
    }
    let code = text(code)?;
    let names = blob.raw("names")?;
    let params = blob
        .table::<ShaderParamRow>("params", format::row_hash::<ShaderParamRow>())?
        .iter()
        .map(|p| {
            let name = (p.name_offset as usize)
                .checked_add(p.name_len as usize)
                .and_then(|end| names.get(p.name_offset as usize..end))
                .ok_or_else(|| invalid(blob, "shader parameter name is out of range"))?;
            let kind = match p.components {
                1 => ParamKind::Float,
                2 => ParamKind::Vec2,
                3 => ParamKind::Vec3,
                4 => ParamKind::Vec4,
                _ => return Err(invalid(blob, "shader parameter has an invalid type")),
            };
            let default = [p.x, p.y, p.z, p.w];
            if default.iter().any(|v| !v.is_finite()) {
                return Err(invalid(blob, "shader parameter default is not finite"));
            }
            Ok(ShaderParam {
                name: text(name)?,
                kind,
                default,
            })
        })
        .collect::<Result<_, _>>()?;
    Ok((code, params))
}
