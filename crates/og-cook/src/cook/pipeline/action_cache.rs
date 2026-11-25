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
    let texture = matches!(job, Some(Action::Input(_, true)));
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

/// An output's store and key: what it was made from, its type and layout,
/// and its stamp.
///
/// An output of its own input ([`Action::Input`]) is keyed by that
/// input's bytes and nothing else of the cook: its stamp holds the options
/// and settings its cook reads and its areas' revisions, as the
/// incremental cook needs it to, so the same input cooked the same way is
/// one entry for every project, checkout and engine build that would
/// write the same bytes. (The whole [`CookOptions`] was keyed here once: a
/// texture's entry then held the paths of the game's code, the mesh and
/// font options and whether the cook was a publish's, and no two projects
/// with code shared one.)
///
/// An output of the project ([`Action::Project`]) is keyed by the
/// project's inputs, its id and the cook's options ([`project_options`]).
fn entry(
    directory: Option<&Path>,
    job: Option<Action<'_>>,
    id: &str,
    ty: &'static str,
    stamp: u64,
    options: &CookOptions,
    project_key: Option<&str>,
) -> Option<(cache::Store, String)> {
    let input = match job {
        Some(Action::Never) => None,
        Some(Action::Input(bytes, _)) => Some((bytes, String::new(), String::new())),
        Some(Action::Project) | None => {
            project_key.map(|key| (key.as_bytes(), id.to_owned(), project_options(options)))
        }
    };
    directory.zip(input).map(|(dir, (bytes, id, options))| {
        let layout = format!("{:?}", format::expected_header(ty));
        (
            cache::Store::new(dir.to_owned()),
            // No area's revision: the stamp holds its output's.
            cache::key(
                &[],
                "output-2",
                &[
                    bytes,
                    id.as_bytes(),
                    options.as_bytes(),
                    ty.as_bytes(),
                    layout.as_bytes(),
                    &stamp.to_le_bytes(),
                ],
            ),
        )
    })
}

/// The cook's options as a project output's key holds them: all that can
/// change an output, and no path. The game's code is among the project's
/// inputs by its content (`index::Index::content_key`), and an engine
/// table by its id, so a project keeps its entries when it or the engine
/// moves.
fn project_options(options: &CookOptions) -> String {
    let mut options = options.clone();
    options.threads = 0;
    options.progress = false;
    options.code.files.clear();
    for path in options.code.tables.values_mut() {
        path.clear();
    }
    format!(
        "{options:?} {:?}",
        super::texture::encoder_name(&options.texture)
    )
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
