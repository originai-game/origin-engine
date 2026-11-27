//! Cook's dependency and image boundary for the shared part-local transfer kernel.
use super::{Block, Line, Part, Scene, TransferInput, input, projection, whole};
use og_bake::projection::{Texels, Transfer, fill_unpainted, padded};
use og_shape::tools::{self, Image, Texture};

pub(crate) const LINES: &[Line] = &[
    Line::new(
        "source",
        "\"<model asset>\"",
        "required",
        "Asset-root-relative base model, normally imported and processed before this model. Source contents, external files and parameters enter the cache key; cycles fail.",
        "",
    ),
    Line::new(
        "size",
        "<integer, 1..8192>",
        "1024",
        "Destination map side. Source painted maps must be square and share one size; they are not resampled.",
        "",
    ),
    Line::new(
        "padding",
        "<integer, 0..8192>",
        "4",
        "Destination chart gutters after nearest-painted-surface hole filling.",
        "",
    ),
];

pub(super) fn settings(b: &Block) -> Result<input::Transfer, String> {
    let e = b.get("source").ok_or("transfer requires source")?;
    if e.values.len() != 1 || e.block.is_some() {
        return Err("transfer.source takes one model asset path".into());
    }
    let id = e.string(0).map_err(|e| e.to_string())?;
    Ok(input::Transfer {
        source: crate::cook::source_schemas::ContentRef {
            id: crate::cook::normalise_rel(crate::runtime::asset_id(id)),
        },
        size: Some(whole(b, "size", 1024, 8192)?),
        padding: Some(whole(b, "padding", 4, 8192)?),
    })
}

pub(crate) fn source(process: &input::Process) -> Option<&str> {
    process.transfer.as_ref().map(|t| t.source.id.as_str())
}

pub(super) fn apply(
    scene: &mut Scene,
    b: &input::Transfer,
    input: &TransferInput,
) -> Result<(), String> {
    let size = b.size.unwrap_or(1024);
    if size == 0 {
        return Err("transfer.size must be positive".into());
    }
    let padding = b.padding.unwrap_or(4);
    let imported = og_shape_formats::import(&input.bytes, &Default::default(), &|uri| {
        input
            .files
            .iter()
            .find(|(name, _)| name == uri)
            .map(|(_, bytes)| bytes.clone())
            .ok_or_else(|| format!("base model did not track {uri:?}"))
    })
    .map_err(|e| e.to_string())?;
    if imported.parts.is_empty() || !imported.skipped.is_empty() {
        return Err("transfer.source needs mesh primitives without skipped input".into());
    }
    let solids = imported
        .parts
        .iter()
        .map(tools::apply_transform)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let base = Scene {
        name: String::new(),
        parts: imported
            .parts
            .iter()
            .zip(&solids)
            .map(|(p, solid)| Part {
                material: p.material.clone(),
                ..Part::new(&p.name, solid)
            })
            .collect(),
    };
    let mut base_size = None;
    let maps = base
        .parts
        .iter()
        .map(|p| {
            let Some(map) = &p.material.map else {
                return Ok(None);
            };
            let image = crate::cook::decode_image(&map.bytes)?;
            if image.width == 0
                || image.width > 8192
                || image.width != image.height
                || base_size.is_some_and(|s| s != image.width)
            {
                return Err(
                    "transfer.source maps must be square, side 1..8192, and share one size".into(),
                );
            }
            base_size = Some(image.width);
            Ok(Some(
                image
                    .rgba
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .map(|p| [p[0], p[1], p[2]])
                    .collect(),
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let base_level = projection::surface(&base, None)?;
    let names = |level: &og_bake::projection::Level| {
        level
            .triangle_label
            .iter()
            .map(|&i| level.labels[i as usize].clone())
            .collect::<Vec<_>>()
    };
    let transfer = Transfer::new(
        &base_level,
        &names(&base_level),
        &maps,
        base_size.ok_or("transfer.source has no painted base-colour map")?,
    )?;
    let level = projection::surface(scene, None)?;
    let parts = names(&level);
    let mut maps = Vec::new();
    for (i, part) in scene.parts.iter().enumerate() {
        let texels = Texels::rasterise(&projection::surface(scene, Some(i))?, size, 0);
        let (mut paint, _) = transfer.paint(&level, &parts, &texels)?;
        if paint.coverage() == 0.0 {
            maps.push(None);
            continue;
        }
        fill_unpainted(&texels, &mut paint, 0);
        let rgb = padded(&texels, &paint, padding, [0; 3]);
        let image = Image {
            width: size,
            height: size,
            rgba: rgb
                .into_iter()
                .flat_map(|p| [p[0], p[1], p[2], 255])
                .collect(),
        };
        maps.push(Some(Texture::from_bytes(
            &format!("{}.transfer.png", part.name),
            (og_shape_formats::texture::CODEC.png)(&image),
        )));
    }
    if maps.iter().all(Option::is_none) {
        return Err(
            "transfer painted no texel: destination parts must match named base parts".into(),
        );
    }
    for (part, map) in scene.parts.iter_mut().zip(maps) {
        if let Some(map) = map {
            part.material.map = Some(map);
            part.material.color = Some([1.0; 3]);
        }
    }
    Ok(())
}
