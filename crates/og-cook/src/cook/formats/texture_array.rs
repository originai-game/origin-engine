//! Typed texture arrays name explicit module-relative images of one size,
//! cooked into an `og.TextureArray` blob (its size, kind and layer
//! ids) and one `og.Texture` per layer (`<id>#layer<n>`, encoded like any
//! texture of its kind), which the runtime uploads as one 2D-array texture.
//!
//! ```ts
//! import {file} from '@origingame/engine/author';
//! export default file['texture-array']({
//!   kind: 'color', layers: ['assets/tiles/grass.png', 'assets/tiles/dirt.png'],
//! });
//! ```

use super::texture;
use super::{CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, stamp};
use crate::cook::source_schemas::world::source_data::{TextureArrayInput, TextureKind};
use crate::format::{self, *};
use og_blob::BlobWriter;
use std::path::Path;

/// The most layers an array holds (WebGPU's guaranteed
/// `maxTextureArrayLayers` is 256).
pub const MAX_LAYERS: usize = 256;

/// A texture array's encoding kind and module-relative layer files.
#[derive(Clone, Debug, PartialEq)]
pub struct ArrayDecl {
    pub kind: u32,
    /// Each layer's explicit path under the owning module.
    pub layers: Vec<String>,
}

fn declaration(s: &Source) -> Result<ArrayDecl, String> {
    let input = s.input::<TextureArrayInput>()?;
    let root = crate::cook::assets_root(s);
    for layer in &input.layers {
        let path = crate::cook::source_path::checked(&root, layer)
            .map_err(|e| format!("layer '{layer}': {e}"))?;
        if !path.is_file() {
            return Err(format!("layer '{layer}' is not a regular image file"));
        }
    }
    Ok(ArrayDecl {
        kind: match input.kind {
            TextureKind::Color => format::kind::COLOR,
            TextureKind::Normal => format::kind::NORMAL,
            TextureKind::Linear => format::kind::LINEAR,
            TextureKind::Single => format::kind::SINGLE,
            TextureKind::Pixel => format::kind::PIXEL,
        },
        layers: input.layers,
    })
}

/// The checked declaration reports exactly the layer textures it produces.
pub(crate) fn derived(
    data: &crate::cook::generator::value::Data,
) -> Result<std::collections::BTreeMap<String, String>, String> {
    let file: crate::cook::source_schemas::File =
        serde_json::from_value(data.root().into_json()).map_err(|e| e.to_string())?;
    let input: TextureArrayInput = serde_json::from_value(file.value).map_err(|e| e.to_string())?;
    Ok((0..input.layers.len())
        .map(|n| (format!("layer{n}"), "texture".into()))
        .collect())
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

pub(crate) static TEXTURE_ARRAY: KindDef = KindDef {
    name: "texture-array",
    extensions: &[],
    types: &[],
    by_content: false,
    streams: true,
    plan: plan_texture_array,
};

/// The kinds this module cooks.
pub(crate) static KINDS: &[&KindDef] = &[&TEXTURE_ARRAY];

fn plan_texture_array<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx { tex_desc, .. } = *cx;
    let PlanState { early_errors, .. } = state;
    let root = crate::cook::assets_root(s);
    match declaration(s) {
        Ok(decl) => {
            let deps = dependencies(&root, &decl);
            let base = [
                b"texture-array 1".as_slice(),
                tex_desc.as_bytes(),
                &hash,
                deps.as_bytes(),
            ];
            outs.push((
                s.rel.clone(),
                format::TextureArray::NAME,
                stamp(&[Area::Texture], &base),
                Box::new(TextureArrayJob(s, decl.clone())),
            ));
            for n in 0..decl.layers.len() {
                let st = stamp(
                    &[Area::Texture],
                    &[&base[..], &[&(n as u32).to_le_bytes()[..]]].concat(),
                );
                outs.push((
                    layer_id(&s.rel, n),
                    format::Texture::NAME,
                    st,
                    Box::new(ArrayLayerJob(s, decl.clone(), n)),
                ));
            }
        }
        Err(e) => early_errors.push((s.rel.clone(), "texture-array", e)),
    }
}

/// A texture array's one blob.
struct TextureArrayJob<'a>(&'a Source, ArrayDecl);

impl Job for TextureArrayJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let TextureArrayJob(s, decl) = self;
        let Exec { options, .. } = ex;
        let root = crate::cook::assets_root(s);
        cook(&s.rel, &root, decl, &options.texture, p.stamp)
    }
}

/// One layer of a texture array, cooked alone.
struct ArrayLayerJob<'a>(&'a Source, ArrayDecl, usize);

impl Job for ArrayLayerJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let ArrayLayerJob(s, decl, n) = self;
        let Exec { options, .. } = ex;
        let root = crate::cook::assets_root(s);
        cook_layer(&root, decl, *n, &options.texture, p.stamp).map(|b| (b, Vec::new()))
    }
}
