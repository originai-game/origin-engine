//! Cooked graph bindings; no author grammar or text transport.
use super::*;

#[derive(Clone, Debug)]
pub struct ClipBinding {
    pub name: String,
    pub source: String,
    pub clip: String,
}

#[derive(Clone, Debug)]
pub struct GraphSource {
    /// Clip handles are one-based slots in `clips`, not runtime handles.
    pub asset: GraphAsset,
    pub clips: Vec<String>,
    pub sprite: bool,
    pub bindings: Vec<ClipBinding>,
}

impl GraphSource {
    pub fn validate(&self) -> Result<()> {
        self.asset.validate()?;
        let mut names = std::collections::BTreeSet::new();
        if self.bindings.iter().any(|b| {
            b.name.is_empty() || b.source.is_empty() || b.clip.is_empty() || !names.insert(&b.name)
        }) || (self.sprite && !self.bindings.is_empty())
            || (!self.sprite && self.clips.iter().any(|c| !names.contains(c)))
            || self.clips.iter().any(String::is_empty)
            || self
                .clips
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != self.clips.len()
            || self
                .asset
                .clips()
                .iter()
                .any(|&c| c == 0 || c > self.clips.len() as u64)
        {
            return Err(Error::new(
                ErrorKind::Parse,
                "animation graph: invalid clip slots or bindings",
            ));
        }
        Ok(())
    }
}
