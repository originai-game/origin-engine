//! project.ogt's `cook { uses "<cooked dir>" }`: other cooked bundles whose
//! assets this project's assets may name by asset id, where a kind allows it
//! (a terrain's layer textures and layer arrays). The cook reads each
//! bundle's manifest, checks a name against its outputs and their types, and
//! packs nothing of them: the game opens that bundle (as its boot bundle, or
//! a ready pack or bundle) before it loads what names them.

use super::*;

/// The assets the bundles of `cook { uses }` provide, by asset id.
#[derive(Default)]
pub(crate) struct Provided {
    /// Each output's blob type and the bundle (as `uses` names it) it is in.
    by_id: BTreeMap<String, (String, String)>,
}

impl Provided {
    /// The bundles `cook { uses }` names in project.ogt (`body`), read
    /// relative to the project `root`: each a cooked directory (one with
    /// `.cook.ogt`) or a project whose `cooked/` has one. The first bundle
    /// naming an id provides it.
    pub(crate) fn read(
        root: &Path,
        body: Option<&og_core::ogt::Block>,
    ) -> Result<Provided, String> {
        let mut by_id = BTreeMap::new();
        let Some(cook) = body
            .and_then(|b| b.get("cook"))
            .and_then(|e| e.block.as_ref())
        else {
            return Ok(Provided { by_id });
        };
        for e in cook.get_all("uses") {
            for v in &e.values {
                let Some(name) = v.as_str() else {
                    return Err(format!(
                        "project.ogt:{}: cook.uses takes paths in quotes: uses \"../materials/cooked\"",
                        e.line
                    ));
                };
                let dir = root.join(name);
                let manifest = [dir.join(MANIFEST), dir.join("cooked").join(MANIFEST)]
                    .into_iter()
                    .find(|p| p.is_file())
                    .ok_or_else(|| {
                        format!(
                            "project.ogt:{}: cook.uses \"{name}\" has no {MANIFEST} (name a cooked directory, or a project cooked first)",
                            e.line
                        )
                    })?;
                let text = std::fs::read_to_string(&manifest)
                    .map_err(|err| format!("{}: {err}", manifest.display()))?;
                let parsed = Manifest::parse(&text)
                    .map_err(|err| format!("{}: {err}", manifest.display()))?;
                for o in parsed.outputs() {
                    by_id
                        .entry(o.id.clone())
                        .or_insert_with(|| (o.ty.clone(), name.to_string()));
                }
            }
        }
        Ok(Provided { by_id })
    }

    /// The blob type of `id` when a used bundle provides it.
    pub(crate) fn ty(&self, id: &str) -> Option<&str> {
        self.by_id
            .get(crate::runtime::asset_id(id))
            .map(|(ty, _)| ty.as_str())
    }

    /// Why `id` cannot be named as a `want` from a used bundle: None when one
    /// provides it with that type, or when none provides it (the caller's
    /// own check reports a missing asset).
    pub(crate) fn wrong_type(&self, id: &str, want: &str) -> Option<String> {
        let (ty, from) = self.by_id.get(crate::runtime::asset_id(id))?;
        (ty != want).then(|| format!("{id} in \"{from}\" is {ty}, not {want}"))
    }
}
