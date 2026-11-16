//! Remote `<iframe>`s (docs/ui.md §2.1 Frames): the documents a game embeds
//! from another origin, and what such a document may be.
//!
//! A remote document is untrusted text. The host fetches it from an origin
//! the project allows (project.ogt's `frames`), within its budgets, parses
//! it and defines it under its request key; the frame compiles it only when
//! [`refusal`] finds nothing it could not show. It is given data only
//! through its iframe's attributes and its events reach the game only as
//! frame messages ([`crate::Ui::take_frame_messages`]).
//!
//! Showing one is og-ui's `frames` feature, which `og build` ships only
//! to a game whose project.ogt lists `frames`: without it a remote src is
//! refused when its document is parsed, and no budget, nesting walk or
//! message split is built. The allowlist itself ([`project_origins`],
//! [`allowed`]) and [`FrameMessage`] are always here, for the cook and
//! for `og.ui.frameMessages`, which then stays empty.

#[cfg(feature = "frames")]
use super::{NodeKind, UiDocument};
use crate::css::request;
use og_core::ogt::Block;

/// Something a remote frame's document did (`og.ui.frameMessages`, HTML's
/// `postMessage` as a frame's events): a click, a changed control, a named
/// event or action. It never reaches the game as a document event, so no
/// handler of the game's own documents answers it.
#[derive(Clone, Debug, PartialEq)]
pub struct FrameMessage {
    /// The embedding document's mount.
    pub mount: String,
    /// The `<iframe>`'s path.
    pub frame: String,
    /// The origin its document was loaded from (`https://events.example.com`).
    pub origin: String,
    /// What it did, as a document event's kind: `click`, `changed`,
    /// `select`, `emit`, `action`, `set`.
    pub kind: String,
    /// The element's id, the prop or event's name.
    pub name: String,
    /// The element's path inside the frame (after the iframe's).
    pub path: String,
    /// The new value (`changed`), the row's index (`select`); null otherwise.
    pub value: serde_json::Value,
}

/// `events` without those from inside a remote frame of `frames` (path
/// and request key by frame), which become messages of the innermost one.
#[cfg(feature = "frames")]
pub(crate) fn split_messages(
    events: Vec<super::build::DocEvent>,
    frames: &std::collections::BTreeMap<crate::reactive::graph::MountedNode, (String, String)>,
    messages: &mut Vec<FrameMessage>,
) -> Vec<super::build::DocEvent> {
    if frames.is_empty() {
        return events;
    }
    events
        .into_iter()
        .filter(|e| {
            let inside = frames
                .values()
                .filter_map(|(path, key)| {
                    Some((e.path.strip_prefix(path)?.strip_prefix('/')?, path, key))
                })
                .min_by_key(|(rest, _, _)| rest.len());
            let Some((rest, frame, key)) = inside else {
                return true;
            };
            messages.push(FrameMessage {
                mount: e.mount.clone(),
                frame: frame.clone(),
                origin: request::origin(key.split(' ').next().unwrap_or(key)).unwrap_or_default(),
                kind: e.kind.clone(),
                name: e.name.clone(),
                path: rest.to_string(),
                value: e.value.clone(),
            });
            false
        })
        .collect()
}

/// Why a remote `<iframe>` src is refused by an engine without og-ui's
/// `frames` feature.
pub const ABSENT: &str = "remote <iframe>s are not built into this engine: og build ships them to a game whose project.ogt lists `frames` (og-ui's `frames` feature)";

/// The most bytes of source a remote document may have.
pub const MAX_SOURCE: usize = 256 * 1024;
/// The most nodes (elements, blocks, text) a remote document may have.
pub const MAX_NODES: usize = 4096;
/// The most remote images a remote document may name.
pub const MAX_IMAGES: usize = 64;
/// How deep frames may nest: a frame in a frame in a frame.
pub const MAX_DEPTH: usize = 3;

/// project.ogt's `frames "https://events.example.com" …`: the origins a
/// remote `<iframe>` may load from, each an https origin (http only to
/// localhost) with no path. Absent: none. An error names its line.
pub fn project_origins(body: &Block) -> Result<Vec<String>, String> {
    let Some(e) = body.get("frames") else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for v in &e.values {
        let given = v.as_str().unwrap_or_default();
        // An origin is a URL with no path, query or fragment past `/`.
        let rest = given.split_once("://").map_or("", |(_, r)| r);
        let bare = rest.find(['/', '?', '#']).is_none_or(|i| &rest[i..] == "/");
        let origin = request::origin(given).filter(|_| bare);
        match origin {
            Some(o) if request::secure(&o) => {
                if !out.contains(&o) {
                    out.push(o);
                }
            }
            _ => {
                return Err(format!(
                    "{}: `frames` lists the origins remote <iframe>s load from, each quoted with no path, such as `frames \"https://events.example.com\"` (https://; http:// only for localhost); `{given}` is not one",
                    e.line
                ));
            }
        }
    }
    if e.block.is_some() || out.is_empty() {
        return Err(format!(
            "{}: `frames` lists one or more quoted origins, such as `frames \"https://events.example.com\"`",
            e.line
        ));
    }
    Ok(out)
}

/// Whether the remote frame `key` (a request key) loads from one of
/// `allowed`, the project's `frames`.
pub fn allowed(key: &str, allowed: &[String]) -> bool {
    let url = key.split(' ').next().unwrap_or(key);
    request::origin(url).is_some_and(|o| allowed.contains(&o))
}

/// Why the remote document `doc` cannot be shown, if it cannot: past its
/// budgets, reading a store (a frame is given data only through its
/// attributes), or using a component that is not one of the engine's
/// (`@ui/`) or that `defined` does not have. `None`: it compiles.
#[cfg(feature = "frames")]
pub fn refusal<'a>(
    doc: &UiDocument,
    defined: &dyn Fn(&str) -> Option<&'a UiDocument>,
) -> Option<String> {
    if doc.text.len() > MAX_SOURCE {
        return Some(format!("its source is past {} KiB", MAX_SOURCE / 1024));
    }
    if doc.nodes.len() > MAX_NODES {
        return Some(format!("it has more than {MAX_NODES} nodes"));
    }
    if doc.sheet.remote.len() > MAX_IMAGES {
        return Some(format!("it names more than {MAX_IMAGES} images"));
    }
    // `$ui` (the device, the reduced-motion preference) is the engine's;
    // every other store is the game's.
    for (i, n) in doc.nodes.iter().enumerate() {
        let locals = doc.locals(i);
        let mut paths = Vec::new();
        doc.node_paths(n, &mut paths);
        for path in paths {
            let root = path.split('.').next().unwrap_or_default();
            if let Some(name) = root.strip_prefix('$')
                && name != "ui"
                && !locals.iter().any(|local| local == root)
            {
                return Some(format!(
                    "it reads ${name}; a frame is given data only through its <iframe>'s attributes"
                ));
            }
        }
    }
    for (i, n) in doc.nodes.iter().enumerate() {
        let NodeKind::Use(name) = &n.kind else {
            continue;
        };
        let Some(component) = defined(name).filter(|c| c.source.starts_with("@ui/")) else {
            return Some(format!(
                "it uses <{name}>, which is not one of the engine's components the game loaded"
            ));
        };
        if let Err(e) = super::build::retained::content::unpassed(doc, i, component) {
            return Some(e);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_origins_are_secure_origins_without_a_path() {
        let read = |text: &str| {
            let text = format!("type project\nversion 1\n---\n{text}");
            project_origins(&og_core::ogt::parse(&text).unwrap().body)
        };
        assert_eq!(
            read("frames \"https://Events.example.com:443/\" \"http://localhost:8080\"\n"),
            Ok(vec![
                "https://events.example.com".to_string(),
                "http://localhost:8080".to_string()
            ])
        );
        assert_eq!(read("name \"g\"\n"), Ok(Vec::new()));
        for bad in [
            "frames \"http://events.example.com\"\n",
            "frames \"https://events.example.com/page\"\n",
            "frames events\n",
            "frames\n",
        ] {
            assert!(read(bad).is_err(), "{bad}");
        }
        let allow = ["https://events.example.com".to_string()];
        assert!(allowed(
            "https://events.example.com/a.svelte integrity(\"x\")",
            &allow
        ));
        assert!(!allowed(
            "https://events.example.com.evil.net/a.svelte",
            &allow
        ));
        assert!(!allowed("https://user@events.example.com/a.svelte", &allow));
    }
}
