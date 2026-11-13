//! Shared results. FNV stamps stay in blobs; cache identity uses full inputs.
use super::{CookError, CookOptions, Planned, Work, cache, format};
use og_blob::Blob;
use std::path::Path;

pub(crate) fn run(
    directory: Option<&Path>,
    p: &Planned,
    options: &CookOptions,
    project_key: Option<&str>,
    cook: impl FnOnce() -> Result<(Vec<u8>, Vec<String>), CookError>,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let texture = matches!(p.work, Some(Work::Texture(..) | Work::GltfTexture(..)));
    let input = match &p.work {
        Some(Work::Texture(s, kind, split)) => Some((
            s.conformed.as_deref().unwrap_or(&s.bytes).as_slice(),
            format!("{kind} {split:?} {} {:?}", super::ext(&s.rel), s.settings),
        )),
        Some(Work::GltfTexture(s, image, kind, cutout, split)) => Some((
            s.gltf.as_ref().expect("parsed model").images[*image].as_slice(),
            format!("{kind} cutout {cutout:?} {split:?} {:?}", s.settings),
        )),
        Some(Work::Sound(s, chunks)) => Some((
            s.bytes.as_slice(),
            format!("sound {chunks:?} {} {:?}", super::ext(&s.rel), s.settings),
        )),
        Some(Work::SoundChunk(s, chunk, count)) => Some((
            s.bytes.as_slice(),
            format!(
                "sound-chunk {chunk} {count} {} {:?}",
                super::ext(&s.rel),
                s.settings
            ),
        )),
        Some(
            Work::VideoMeta(..)
            | Work::VideoArray(..)
            | Work::VideoFrame(..)
            | Work::VideoAudio(..)
            | Work::VideoStream(..)
            | Work::VideoPoster(..),
        ) => None,
        _ => project_key.map(|key| (key.as_bytes(), p.id.clone())),
    };
    let action = directory.zip(input).map(|(dir, (bytes, settings))| {
        let mut options = options.clone();
        options.threads = 0;
        let options = format!(
            "{options:?} {:?} {:?}",
            super::texture::encoder_name(&options.texture),
            format::expected_header(p.ty)
        );
        (
            cache::Store::new(dir.to_owned()),
            cache::key(
                "output",
                &[
                    bytes,
                    settings.as_bytes(),
                    options.as_bytes(),
                    p.ty.as_bytes(),
                    &p.stamp.to_le_bytes(),
                ],
            ),
        )
    });
    if let Some((store, key)) = &action
        && let Some((bytes, mut notes)) = store.action(key)
        && Blob::parse(&bytes, "cached texture").is_ok_and(|blob| {
            let h = blob.header();
            format::expected_header(p.ty) == Some((h.type_hash, h.schema_version))
                && h.cook_stamp == p.stamp
        })
    {
        notes.push(format!(
            "{}: shared {} cache hit",
            p.id,
            if texture { "texture" } else { "cook" }
        ));
        return Ok((bytes, notes));
    }
    let result = cook()?;
    if let Some((store, key)) = action {
        let _ = store.put_action(&key, &result.0, &result.1);
    }
    Ok(result)
}
