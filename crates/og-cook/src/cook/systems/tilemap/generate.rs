//! A script supplies tile ids; the authored map owns its tilesets, layer
//! properties and objects. Expansion feeds the ordinary map parser/cook.

use super::{Source, TILEMAP};
use crate::cook::{assets_root, generator};
use og_core::ogt::{Document, Entry, Number};
use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Grid {
    width: u32,
    height: u32,
    layers: Vec<Layer>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Layer {
    name: String,
    tiles: Vec<i32>,
}

fn make(s: &Source) -> Result<Option<Vec<u8>>, String> {
    if s.kind != &TILEMAP || !s.rel.ends_with(".ogt") || s.error.is_some() {
        return Ok(None);
    }
    let mut doc = Document::parse(&String::from_utf8_lossy(&s.bytes)).map_err(|e| e.to_string())?;
    let entries: Vec<_> = doc.body.get_all("generate").collect();
    if entries.is_empty() {
        return Ok(None);
    }
    if entries.len() != 1 || !entries[0].values.is_empty() {
        return Err("a tilemap has one generate block".into());
    }
    let block = entries[0].block.as_ref().ok_or("generate is a block")?;
    let spec = generator::Spec::standalone(block)?;
    let program = generator::Program::prepare(&assets_root(s), &s.rel, &spec)?;
    let grid: Grid = serde_json::from_value(program.run()?).map_err(|e| e.to_string())?;
    let count = u64::from(grid.width) * u64::from(grid.height);
    if grid.width == 0 || grid.height == 0 || count > 16_777_216 {
        return Err(
            "a generated map needs positive width/height and at most 16,777,216 cells".into(),
        );
    }
    let sizes: Vec<_> = doc.body.get_all("size").collect();
    if sizes.len() > 1
        || sizes.first().is_some_and(|e| {
            e.block.is_some()
                || e.values.len() != 2
                || e.number(0).and_then(Number::as_u64).ok() != Some(u64::from(grid.width))
                || e.number(1).and_then(Number::as_u64).ok() != Some(u64::from(grid.height))
        })
    {
        return Err(
            "tilemap size must match the generated width and height; no implicit resampling".into(),
        );
    }
    let mut names = BTreeSet::new();
    if grid.layers.is_empty() {
        return Err("a generated map supplies at least one named layer".into());
    }
    // Validate the complete result before changing the document.
    for layer in &grid.layers {
        if !names.insert(&layer.name) || layer.tiles.len() as u64 != count {
            return Err(format!(
                "generated layer '{}': needs a unique name and width × height tile ids",
                layer.name
            ));
        }
        let matches: Vec<_> = doc
            .body
            .get_all("layer")
            .filter(|e| e.string(0).ok() == Some(layer.name.as_str()))
            .collect();
        if matches.len() != 1 {
            return Err(format!(
                "generated layer '{}' must name exactly one authored layer",
                layer.name
            ));
        }
        let body = matches[0].block.as_ref().ok_or("layer is a block")?;
        if body.get("row").is_some() {
            return Err(format!(
                "generated layer '{}' cannot also have authored rows",
                layer.name
            ));
        }
        let terrain = body.get("terrain").is_some();
        if layer.tiles.iter().any(|&v| {
            if terrain {
                !matches!(v, -1..=1)
            } else {
                v < -1 || v >= og_tilemap::TILE_MASK as i32
            }
        }) {
            return Err(format!(
                "generated layer '{}': expected tile ids or -1 for empty (terrain layers: 0/1)",
                layer.name
            ));
        }
    }
    doc.body
        .entries
        .retain(|e| e.key != "generate" && e.key != "size");
    doc.body.push(
        Entry::new("size")
            .with(Number::from(grid.width))
            .with(Number::from(grid.height)),
    );
    for layer in grid.layers {
        let body = doc
            .body
            .entries
            .iter_mut()
            .find(|e| e.key == "layer" && e.string(0).ok() == Some(layer.name.as_str()))
            .and_then(|e| e.block.as_mut())
            .expect("validated layer");
        for row in layer.tiles.chunks_exact(grid.width as usize) {
            let mut e = Entry::new("row");
            e.values.extend(row.iter().map(|&v| Number::from(v).into()));
            body.push(e);
        }
    }
    let text = doc.to_text().map_err(|e| e.to_string())?;
    og_tilemap::TilemapAsset::parse(&text)
        .and_then(|m| m.validate())
        .map_err(|e| e.to_string())?;
    Ok(Some(text.into_bytes()))
}

pub(crate) fn expand(sources: &mut [Source]) {
    for s in sources {
        match make(s) {
            Ok(Some(bytes)) => s.bytes = bytes,
            Ok(None) => {}
            Err(e) => s.error = Some(e),
        }
    }
}
