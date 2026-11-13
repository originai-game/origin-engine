//! Shared results. FNV stamps stay in blobs; cache identity uses full inputs.
use super::{Action, CookError, CookOptions, Planned, cache, format};
use og_blob::Blob;
use std::path::Path;

pub(crate) fn run(
    directory: Option<&Path>,
    p: &Planned,
    options: &CookOptions,
    project_key: Option<&str>,
    cook: impl FnOnce() -> Result<(Vec<u8>, Vec<String>), CookError>,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let job = p.work.as_ref().map(|j| j.action());
    let texture = matches!(job, Some(Action::Input(.., true)));
    let input = match job {
        Some(Action::Never) => None,
        Some(Action::Input(bytes, settings, _)) => Some((bytes, settings)),
        Some(Action::Project) | None => project_key.map(|key| (key.as_bytes(), p.id.clone())),
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
