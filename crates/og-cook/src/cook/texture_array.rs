//! Texture arrays: an `.ogt` of `type texture-array` naming N images of one
//! size, cooked into an `og.TextureArray` blob (its size, kind and layer
//! ids) and one `og.Texture` per layer (`<id>#layer<n>`, encoded like any
//! texture of its kind), which the runtime uploads as one 2D-array texture.
//!
//! ```text
//! type texture-array
//! version 1
//! ---
//! kind color                 # color (default) | normal | linear | pixel
//! layer "tiles/grass.png"    # relative to this file when it exists there, else under assets/
//! layer "tiles/dirt.png"
//! ```

use super::CookError;
use super::texture;
use crate::format::{self, *};
use og_blob::BlobWriter;
use og_core::ogt::Document;
use std::path::{Path, PathBuf};

/// The `.ogt` type of a texture array.
pub const ARRAY_TYPE: &str = "texture-array";

/// The most layers an array holds (WebGL2's guaranteed minimum is 256).
pub const MAX_LAYERS: usize = 256;

/// A parsed texture array file: its kind and its layers' files.
#[derive(Clone, Debug, PartialEq)]
pub struct ArrayDecl {
    pub kind: u32,
    /// Each layer's path under `assets/`.
    pub layers: Vec<String>,
}

/// Parses a texture array file; `exists(rel)` says whether a path under
/// `assets/` is a file (a layer names one relative to the array file first).
pub fn parse(rel: &str, text: &str, exists: &dyn Fn(&str) -> bool) -> Result<ArrayDecl, String> {
    let doc = Document::parse(text).map_err(|e| e.to_string())?;
    if doc.ty != ARRAY_TYPE {
        return Err(format!("a texture array is `type {ARRAY_TYPE}`"));
    }
    if doc.version != 1 {
        return Err(format!(
            "texture array version {} (this engine reads 1)",
            doc.version
        ));
    }
    let dir = rel.rsplit_once('/').map_or("", |(d, _)| d);
    let mut out = ArrayDecl {
        kind: format::kind::COLOR,
        layers: Vec::new(),
    };
    for e in &doc.body.entries {
        let err = |x: og_core::Error| format!("line {}: {x}", e.line);
        match e.key.as_str() {
            "kind" => {
                out.kind = match e.word(0).map_err(err)? {
                    "color" => format::kind::COLOR,
                    "normal" => format::kind::NORMAL,
                    "linear" => format::kind::LINEAR,
                    "pixel" => format::kind::PIXEL,
                    w => {
                        return Err(format!(
                            "line {}: kind '{w}' (color, normal, linear or pixel)",
                            e.line
                        ));
                    }
                }
            }
            "layer" => {
                let p = e.string(0).map_err(err)?;
                if p.is_empty() || p.starts_with('/') || p.split('/').any(|c| c == "..") {
                    return Err(format!(
                        "line {}: '{p}' is not a path under assets/",
                        e.line
                    ));
                }
                let sibling = if dir.is_empty() {
                    p.to_string()
                } else {
                    format!("{dir}/{p}")
                };
                let path = if exists(&sibling) {
                    sibling
                } else if exists(p) {
                    p.to_string()
                } else {
                    return Err(format!(
                        "line {}: layer '{p}' is neither beside the array nor under assets/",
                        e.line
                    ));
                };
                out.layers.push(path);
            }
            k => return Err(format!("line {}: unknown key '{k}' (kind, layer)", e.line)),
        }
    }
    if out.layers.is_empty() || out.layers.len() > MAX_LAYERS {
        return Err(format!(
            "a texture array has 1 to {MAX_LAYERS} layers, not {}",
            out.layers.len()
        ));
    }
    Ok(out)
}

/// The id of layer `n` of a texture array.
pub fn layer_id(id: &str, n: usize) -> String {
    format!("{id}#layer{n}")
}

/// Reads and decodes every layer, refusing layers of different sizes.
fn layers(root: &Path, decl: &ArrayDecl) -> Result<Vec<texture::Image>, CookError> {
    let mut out: Vec<texture::Image> = Vec::with_capacity(decl.layers.len());
    for (n, p) in decl.layers.iter().enumerate() {
        let bytes =
            std::fs::read(root.join(p)).map_err(|e| CookError(format!("layer {n} '{p}': {e}")))?;
        let img =
            texture::decode(&bytes).map_err(|e| CookError(format!("layer {n} '{p}': {e}")))?;
        if let Some(first) = out.first()
            && (first.width, first.height) != (img.width, img.height)
        {
            return Err(CookError(format!(
                "layer {n} '{p}' is {}x{}, layer 0 '{}' {}x{}: an array's layers have one size",
                img.width, img.height, decl.layers[0], first.width, first.height
            )));
        }
        out.push(img);
    }
    Ok(out)
}

/// The array's blob: its size, kind, layer count and the layers' ids.
pub fn cook(
    id: &str,
    root: &Path,
    decl: &ArrayDecl,
    opts: &super::TextureOptions,
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let images = layers(root, decl)?;
    let (w, h) = (images[0].width, images[0].height);
    let blob = array_blob(id, (w, h), decl.layers.len(), decl.kind, opts, stamp);
    Ok((
        blob,
        vec![format!(
            "texture array: {} layers of {w}x{h} ({})",
            decl.layers.len(),
            decl.layers.join(", ")
        )],
    ))
}

/// An `og.TextureArray` blob: `count` layers of `size` (ids
/// [`layer_id`]`(id, n)`) of texture kind `kind`.
pub fn array_blob(
    id: &str,
    (w, h): (u32, u32),
    count: usize,
    kind: u32,
    opts: &super::TextureOptions,
    stamp: u64,
) -> Vec<u8> {
    let levels = if kind == format::kind::PIXEL || !opts.mipmaps {
        1
    } else {
        32 - w.max(h).leading_zeros()
    };
    let mut names = String::new();
    let mut rows = Vec::with_capacity(count);
    for n in 0..count {
        let lid = layer_id(id, n);
        rows.push(ArrayLayerRow {
            name_offset: names.len() as u32,
            name_len: lid.len() as u32,
        });
        names.push_str(&lid);
    }
    let meta = TextureArrayMeta {
        width: w,
        height: h,
        layers: count as u32,
        levels,
        kind,
    };
    let mut wr = BlobWriter::new(
        format::TextureArray::type_hash(),
        format::TextureArray::schema_version(),
        stamp,
    );
    wr.table("meta", &[meta], row_hash::<TextureArrayMeta>())
        .table("layers", &rows, row_hash::<ArrayLayerRow>())
        .raw("strings", names.into_bytes(), 1);
    wr.finish()
}

/// Layer `n`'s texture blob, encoded as a texture of the array's kind.
pub fn cook_layer(
    root: &Path,
    decl: &ArrayDecl,
    n: usize,
    opts: &super::TextureOptions,
    stamp: u64,
) -> Result<Vec<u8>, CookError> {
    let p = &decl.layers[n];
    let bytes =
        std::fs::read(root.join(p)).map_err(|e| CookError(format!("layer {n} '{p}': {e}")))?;
    let img = texture::decode(&bytes).map_err(|e| CookError(format!("layer {n} '{p}': {e}")))?;
    texture::cook(&img, decl.kind, opts, stamp)
}

/// The layers' files, for the stamp: `path=hash` lines.
pub fn dependencies(root: &Path, decl: &ArrayDecl) -> String {
    decl.layers
        .iter()
        .map(|p| match std::fs::read(root.join(p)) {
            Ok(b) => format!("{p}={:016x}", og_core::hash::hash_bytes(&b)),
            Err(_) => format!("{p}=missing"),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// `assets/` of a source path and its id.
pub fn assets_dir(path: &Path, rel: &str) -> PathBuf {
    let mut root = path.to_path_buf();
    for _ in rel.split('/') {
        root.pop();
    }
    root
}
