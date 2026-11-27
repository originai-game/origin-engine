//! The kinds of source each system's module declares, and the one table
//! the pipeline reaches them through: the scan classifies a file by a
//! kind's extensions or `.ogt` types, planning calls its `plan`, and each
//! output it plans carries a [`Job`] that cooks it.
//!
//! Adding a system is its module (its kinds, plans and jobs) and its line
//! in [`SYSTEMS`].

use super::*;

/// What a system's module declares about one kind of source it cooks.
pub(crate) struct KindDef {
    /// Its name in reports and messages.
    pub(crate) name: &'static str,
    /// The file extensions it claims (lower case, without the dot).
    pub(crate) extensions: &'static [&'static str],
    /// The `type` of each `.ogt` it claims.
    pub(crate) types: &'static [&'static str],
    /// Its cook reads only a source's bytes and the options, never where
    /// the source is: its action hashes the extension, not the path.
    pub(crate) by_content: bool,
    /// A streaming group may hold it.
    pub(crate) streams: bool,
    /// Plans a source's outputs, each with its stamp and job.
    pub(in crate::cook) plan: PlanFn,
}

/// A kind's planner: the outputs of a source, each with its stamp and work.
pub(in crate::cook) type PlanFn =
    for<'a> fn(&Cx<'a>, &mut PlanState, &'a Source, [u8; 8], &mut Vec<Out<'a>>);

/// A kind is its declaration: two are the same kind when they are the same
/// static.
impl PartialEq for KindDef {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for KindDef {}

impl fmt::Debug for KindDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name)
    }
}

/// What cooking an output reads beside its own work.
pub(crate) struct Exec<'a> {
    pub(crate) options: &'a CookOptions,
    pub(crate) schemas: &'a BTreeMap<String, Schema>,
    pub(crate) known: &'a BTreeSet<String>,
    pub(crate) prefab_texts: &'a BTreeMap<String, std::sync::Arc<scene::Document>>,
    pub(crate) budgets: &'a stats::Budgets,
}

/// A planned output's work, cooked by the system that planned it: the
/// blob and the cook's notes.
pub(crate) trait Job: Send + Sync {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError>;

    /// What the shared action cache keys its result by.
    fn action(&self) -> Action<'_> {
        Action::Project
    }

    /// The outputs only its cook can name, each as its plan would have
    /// planned it had it known (a model's LOD surfaces, which its cook
    /// charts): asked once it has cooked, and cooked in a round of their
    /// own (`plan::derive`). None for an output whose plan named them all.
    fn derived<'a>(&self, _: &Exec) -> Result<Vec<Out<'a>>, CookError>
    where
        Self: 'a,
    {
        Ok(Vec::new())
    }

    /// How much work its cook is beside the others', for the order and
    /// the threads `execute` gives it: a texture's encode
    /// (`texture::encode_cost`), 0 for work that is short or of no known
    /// size. Never changes what is cooked.
    fn cost(&self, _: &CookOptions) -> u64 {
        0
    }
}

/// What the shared action cache keys an output's result by, beside its
/// type and its stamp (`action_cache`).
pub(crate) enum Action<'s> {
    /// The project, the output's id and the cook's options: the output
    /// reads more of the project than its own input (other assets, by
    /// path).
    Project,
    /// Nothing: the result is never shared.
    Never,
    /// Its own input alone, by its bytes: with the stamp (the options and
    /// settings its cook reads), every project that cooks these bytes the
    /// same way shares the result. Whether it is a texture: a hit is
    /// reported as the shared texture cache's.
    Input(&'s [u8], bool),
}

/// Every system's kinds, one line a system.
pub(crate) static SYSTEMS: &[&[&KindDef]] = &[
    animgraph::KINDS,
    behaviour::KINDS,
    story::KINDS,
    strings::KINDS,
    table::KINDS,
    field::KINDS,
    navmesh::KINDS,
    terrain::KINDS,
    district::KINDS,
    lightmap::KINDS,
    environment::KINDS,
    timeline::KINDS,
    icons::KINDS,
    font_pack::KINDS,
    look::KINDS,
    effect::KINDS,
    effect_texture::KINDS,
    vfx::KINDS,
    shader_check::KINDS,
    anim::KINDS,
    vector::KINDS,
    tilemap::KINDS,
    rig2d::KINDS,
    flipbook::KINDS,
    ui_packs::KINDS,
    riv::KINDS,
    video::KINDS,
    texture_array::KINDS,
    font::KINDS,
    scene::KINDS,
    audio::KINDS,
    sfx::KINDS,
    texture::KINDS,
    model::KINDS,
];

pub(in crate::cook) fn kinds() -> impl Iterator<Item = &'static KindDef> {
    SYSTEMS.iter().flat_map(|s| s.iter().copied())
}

/// The registered kind that claims a file extension.
pub(crate) fn by_extension(ext: &str) -> Option<Kind> {
    kinds().find(|k| k.extensions.contains(&ext))
}

/// The registered kind that claims an `.ogt` type.
pub(crate) fn by_type(ty: &str) -> Option<Kind> {
    kinds().find(|k| k.types.contains(&ty))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The scan takes the first kind that claims a file: two kinds claiming
    /// one extension or `.ogt` type would leave the later one unreachable.
    #[test]
    fn no_two_kinds_claim_one_extension_or_type() {
        let mut seen: BTreeMap<&str, &str> = BTreeMap::new();
        for k in kinds() {
            for claim in k.extensions.iter().chain(k.types) {
                if let Some(other) = seen.insert(claim, k.name) {
                    panic!("{claim:?} is claimed by {other} and {}", k.name);
                }
            }
        }
        assert!(seen.contains_key("glb") && seen.contains_key("quest"));
        assert_eq!(by_extension("glb").map(|k| k.name), Some(seen["glb"]));
        assert_eq!(by_type("quest").map(|k| k.name), Some(seen["quest"]));
    }
}
