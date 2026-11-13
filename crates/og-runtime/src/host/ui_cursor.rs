//! The pointer's shape a host shows (CSS UI 4 §5.1): the UI's resolved
//! `cursor` keyword and the first of the `url()` images declared with it
//! that loads, as RGBA pixels with its hotspot. The web shell sets the
//! canvas's CSS cursor from it; Origin Desktop's shared frame hears it as
//! its `Cursor` message (og-render's `shared_frames`).
//!
//! An image is a texture asset URL, as `background-image` names one,
//! read from the pixels the engine holds: an
//! RGBA texture, or a cooked one decoded again from the bundle (a Basis
//! texture natively; on the web only a lossless one, `kind pixel`). One
//! that cannot be read or is larger than
//! [`MAX_SIDE`] does not load, and the next image or the keyword is shown,
//! as a browser falls back. Decoded images are kept by their source, a few
//! at once, so a cursor that comes back is not decoded again.

use std::rc::Rc;

use crate::engine::Engine;

/// The largest cursor image, each side, CSS px (the browsers' limit: a
/// larger one is not shown).
pub const MAX_SIDE: u32 = 128;
/// How many decoded images are kept.
const KEPT: usize = 8;

/// A cursor image ready to show.
#[derive(Debug, PartialEq)]
pub struct CursorPixels {
    pub width: u32,
    pub height: u32,
    /// Row-major RGBA8, sRGB, straight alpha.
    pub rgba: Vec<u8>,
    /// The point of the image that is the pointer's position, image px.
    pub hotspot: [u32; 2],
    /// Its source (the texture, the region and the hotspot), so a host
    /// tells a new image from the one it shows without comparing pixels.
    pub id: u64,
}

/// The pointer's shape this frame.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CursorShape {
    /// The CSS keyword (`default` for `auto`), shown when `image` is none
    /// and where the image cannot be.
    pub keyword: &'static str,
    pub image: Option<Rc<CursorPixels>>,
}

impl CursorShape {
    /// What tells this shape from another: its keyword and its image's id.
    pub fn id(&self) -> (&'static str, u64) {
        (self.keyword, self.image.as_ref().map_or(0, |i| i.id))
    }
}

/// Decoded images by source (`None`: that source does not load), newest
/// last.
#[derive(Default)]
pub struct Kept(Vec<(u64, Option<Rc<CursorPixels>>)>);

/// The UI's cursor this frame: its keyword and the first image that loads.
pub(crate) fn shape(eng: &Engine) -> CursorShape {
    let Some(ui) = eng.state.borrow().ui.clone() else {
        return CursorShape {
            keyword: "default",
            image: None,
        };
    };
    let (cursor, sources) = {
        let ui = ui.lock();
        let frame = ui.frame();
        let sources: Vec<_> = frame
            .cursor_images
            .iter()
            .map(|c| (source(eng, &c.url), c.hotspot))
            .collect();
        (frame.cursor, sources)
    };
    let keyword = match cursor {
        og_ui::Cursor::Auto => "default",
        c => c.css(),
    };
    let image = sources
        .into_iter()
        .find_map(|(src, hotspot)| pixels(eng, src?, hotspot));
    CursorShape { keyword, image }
}

/// The texture asset a stylesheet-relative `url()` names.
fn source(eng: &Engine, url: &str) -> Option<(og_render::TextureHandle, [f32; 4])> {
    let i = *eng
        .assets
        .borrow()
        .by_path
        .get(&format!("texture:{}", url.trim_start_matches('/')))?;
    Some((*eng.render.borrow().textures.get(i)?, [0.0, 0.0, 1.0, 1.0]))
}

/// The image of `region` with `hotspot` (kept, else read now).
fn pixels(
    eng: &Engine,
    (texture, uv): (og_render::TextureHandle, [f32; 4]),
    hotspot: Option<[f32; 2]>,
) -> Option<Rc<CursorPixels>> {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    texture.hash(&mut h);
    uv.map(f32::to_bits).hash(&mut h);
    hotspot.map(|p| p.map(f32::to_bits)).hash(&mut h);
    let id = h.finish() | 1;
    let mut st = eng.state.borrow_mut();
    let kept = &mut st.cursor_images.0;
    if let Some(k) = kept.iter().position(|(k, _)| *k == id) {
        let entry = kept.remove(k);
        let image = entry.1.clone();
        kept.push(entry);
        return image;
    }
    drop(st);
    let read = read(eng, texture, uv);
    let image = match read {
        Ok((width, height, rgba)) => {
            let at = |v: Option<f32>, side: u32| v.map_or(0, |v| (v.max(0.0) as u32).min(side - 1));
            Some(Rc::new(CursorPixels {
                width,
                height,
                rgba,
                hotspot: [
                    at(hotspot.map(|p| p[0]), width),
                    at(hotspot.map(|p| p[1]), height),
                ],
                id,
            }))
        }
        Err(Some(why)) => {
            let mut rs = eng.render.borrow_mut();
            rs.notes.push(format!(
                "cursor: url() {why}; the next image or the keyword is shown"
            ));
            None
        }
        // Not readable yet (a streamed texture still resident elsewhere):
        // asked again on a later frame, not kept.
        Err(None) => return None,
    };
    let mut st = eng.state.borrow_mut();
    let kept = &mut st.cursor_images.0;
    if kept.len() >= KEPT {
        kept.remove(0);
    }
    kept.push((id, image.clone()));
    image
}

/// The RGBA pixels of `uv` of `texture` (`Err(None)`: not yet).
pub(crate) fn read(
    eng: &Engine,
    texture: og_render::TextureHandle,
    uv: [f32; 4],
) -> Result<(u32, u32, Vec<u8>), Option<String>> {
    let decoded;
    let rs = eng.render.borrow();
    let (w, h, rgba): (u32, u32, &[u8]) = if let Some(t) = rs.scene.texture(texture) {
        (t.width, t.height, &t.rgba)
    } else if let Some(u) = rs
        .scene
        .texture_upload(texture)
        .filter(|u| u.desc.format == og_render::TextureFormat::Rgba8)
    {
        (u.desc.width, u.desc.height, &u.levels[0])
    } else {
        let Some(id) = rs.decoded.ids.get(&texture).cloned() else {
            return Err(None);
        };
        drop(rs);
        let assets = eng.assets.borrow();
        decoded = assets
            .blob(&id)
            .and_then(|(bytes, bundle)| {
                bundle
                    .texture(&id, bytes.bytes())
                    .map_err(|e| e.to_string())
            })
            .and_then(|t| t.to_texture_data().map_err(|e| e.to_string()))
            .map_err(|e| Some(format!("{id}: {e}")))?;
        return crop(decoded.width, decoded.height, &decoded.rgba, uv);
    };
    crop(w, h, rgba, uv)
}

/// The region `uv` of a `w`×`h` RGBA image.
fn crop(w: u32, h: u32, rgba: &[u8], uv: [f32; 4]) -> Result<(u32, u32, Vec<u8>), Option<String>> {
    let x0 = (uv[0] * w as f32).round() as u32;
    let y0 = (uv[1] * h as f32).round() as u32;
    let x1 = ((uv[2] * w as f32).round() as u32).min(w);
    let y1 = ((uv[3] * h as f32).round() as u32).min(h);
    let (cw, ch) = (x1.saturating_sub(x0), y1.saturating_sub(y0));
    if cw == 0 || ch == 0 || rgba.len() < (w * h * 4) as usize {
        return Err(Some("names an empty image".into()));
    }
    if cw > MAX_SIDE || ch > MAX_SIDE {
        return Err(Some(format!(
            "is {cw}x{ch}, larger than a cursor ({MAX_SIDE}x{MAX_SIDE} at most)"
        )));
    }
    let mut out = Vec::with_capacity((cw * ch * 4) as usize);
    for y in y0..y1 {
        let row = ((y * w + x0) * 4) as usize;
        out.extend_from_slice(&rgba[row..row + (cw * 4) as usize]);
    }
    Ok((cw, ch, out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_region_is_cut_from_its_atlas() {
        // A 4x2 atlas of 2x2 cells, each pixel its own index.
        let rgba: Vec<u8> = (0..8u8).flat_map(|p| [p, 0, 0, 255]).collect();
        let (w, h, px) = crop(4, 2, &rgba, [0.5, 0.0, 1.0, 1.0]).unwrap();
        assert_eq!((w, h), (2, 2));
        let reds: Vec<u8> = px.chunks(4).map(|p| p[0]).collect();
        assert_eq!(reds, [2, 3, 6, 7]);
        assert!(crop(400, 2, &vec![0; 400 * 2 * 4], [0.0, 0.0, 1.0, 1.0]).is_err());
    }
}
