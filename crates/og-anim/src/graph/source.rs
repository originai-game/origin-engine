//! Authored asset bindings around the existing animation-graph language.
use super::*;
use og_bundle::grammar::{Line, find};

pub const SOURCE_LINES: &[Line] = &[
    Line::new(
        "bind",
        "\"<name>\" \"<model asset>\" \"<clip>\"",
        "",
        "Bind a graph clip name to a cooked model's animation clip; repeat for each referenced clip. Asset-root relative paths, no runtime handles.",
        "",
    ),
    Line::new(
        "sprite",
        "true | false",
        "false",
        "Resolve clip names on the flipbook/2D rig at playback, exclusive with bind. Graph structure is still checked at cook.",
        "",
    ),
];

#[derive(Clone, Debug)]
pub struct ClipBinding {
    pub name: String,
    pub source: String,
    pub clip: String,
}

#[derive(Clone, Debug)]
pub struct GraphSource {
    /// The original document, including the file's tracked asset bindings.
    pub document: Document,
    /// Only graph declarations, for the same parser used at runtime.
    pub text: String,
    pub sprite: bool,
    pub bindings: Vec<ClipBinding>,
}

impl GraphSource {
    pub fn parse(text: &str) -> Result<Self> {
        let document = Document::parse(text)?;
        let mut graph = document.clone();
        let mut bindings: Vec<ClipBinding> = Vec::new();
        let mut sprite = None;
        for e in &document.body.entries {
            if find(SOURCE_LINES, &e.key, &[]).is_none() {
                continue;
            }
            if e.block.is_some() {
                return Err(err(e, "asset binding fields take values, not a block"));
            }
            if e.key == "sprite" {
                arity(e, 1)?;
                if sprite.is_some() {
                    return Err(err(e, "sprite is declared twice"));
                }
                sprite = Some(
                    e.values[0]
                        .as_bool()
                        .ok_or_else(|| err(e, "sprite takes true or false"))?,
                );
            } else {
                arity(e, 3)?;
                let name = e.string(0)?.to_string();
                let source = e.string(1)?.to_string();
                let clip = e.string(2)?.to_string();
                if name.is_empty() || clip.is_empty() || bindings.iter().any(|b| b.name == name) {
                    return Err(err(
                        e,
                        "bind needs a unique nonempty name and a nonempty clip",
                    ));
                }
                if source.is_empty()
                    || source.contains(['\\', '#', ':'])
                    || source
                        .split('/')
                        .any(|p| p.is_empty() || p == "." || p == "..")
                {
                    return Err(err(e, "bind source is a model path relative to assets/"));
                }
                bindings.push(ClipBinding { name, source, clip });
            }
        }
        let sprite = sprite.unwrap_or(false);
        if sprite && !bindings.is_empty() {
            return Err(Error::new(
                ErrorKind::Parse,
                "sprite and bind are exclusive",
            ));
        }
        graph
            .body
            .entries
            .retain(|e| find(SOURCE_LINES, &e.key, &[]).is_none());
        let text = graph.to_text()?;
        if sprite {
            crate::SpriteGraph::parse(&text)?;
        } else {
            GraphAsset::parse(&text, &|name| {
                bindings
                    .iter()
                    .position(|b| b.name == name)
                    .map(|i| i as u64 + 1)
            })?;
        }
        Ok(Self {
            document,
            text,
            sprite,
            bindings,
        })
    }
}
