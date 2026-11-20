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
    let action = entry(directory, job, &p.id, p.ty, p.stamp, options, project_key);
    if let Some((bytes, mut notes)) = action.as_ref().and_then(|a| hit(a, p.ty, p.stamp)) {
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

/// The cached bytes of an output keyed by the project ([`Action::Project`],
/// the default), when the cache holds them under this stamp: what a plan
/// reads before the output cooks (a model's LOD surface ids).
pub(crate) fn project_output(
    directory: Option<&Path>,
    id: &str,
    ty: &'static str,
    stamp: u64,
    options: &CookOptions,
    project_key: Option<&str>,
) -> Option<Vec<u8>> {
    let action = entry(directory, None, id, ty, stamp, options, project_key)?;
    hit(&action, ty, stamp).map(|(bytes, _)| bytes)
}

/// An output's store and key: its action (`None`: the project's), id,
/// type, stamp and the options.
fn entry(
    directory: Option<&Path>,
    job: Option<Action<'_>>,
    id: &str,
    ty: &'static str,
    stamp: u64,
    options: &CookOptions,
    project_key: Option<&str>,
) -> Option<(cache::Store, String)> {
    let own_input = matches!(job, Some(Action::Input(..)));
    let input = match job {
        Some(Action::Never) => None,
        Some(Action::Input(bytes, settings, _)) => Some((bytes, settings)),
        Some(Action::Project) | None => project_key.map(|key| (key.as_bytes(), id.to_owned())),
    };
    directory.zip(input).map(|(dir, (bytes, settings))| {
        let mut options = options.clone();
        options.threads = 0;
        // The game's component schemas pack scenes, which are the project's
        // actions; an output of its own input never reads them, so a code
        // edit or another game still shares it.
        if own_input {
            options.schemas.clear();
        }
        let options = format!(
            "{options:?} {:?} {:?}",
            super::texture::encoder_name(&options.texture),
            format::expected_header(ty)
        );
        (
            cache::Store::new(dir.to_owned()),
            cache::key(
                "output",
                &[
                    bytes,
                    settings.as_bytes(),
                    options.as_bytes(),
                    ty.as_bytes(),
                    &stamp.to_le_bytes(),
                ],
            ),
        )
    })
}

/// A stored result whose blob is still of the type, schema and stamp.
fn hit(
    (store, key): &(cache::Store, String),
    ty: &'static str,
    stamp: u64,
) -> Option<(Vec<u8>, Vec<String>)> {
    store.action(key).filter(|(bytes, _)| {
        Blob::parse(bytes, "cached output").is_ok_and(|blob| {
            let h = blob.header();
            format::expected_header(ty) == Some((h.type_hash, h.schema_version))
                && h.cook_stamp == stamp
        })
    })
}
