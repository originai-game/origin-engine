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
#[expect(dead_code, reason = "read by the jobs of the systems still to move")]
pub(crate) struct Exec<'a> {
    pub(crate) options: &'a CookOptions,
    pub(crate) threads: usize,
    pub(crate) schemas: &'a BTreeMap<String, Schema>,
    pub(crate) known: &'a BTreeSet<String>,
    pub(crate) prefab_texts: &'a BTreeMap<String, String>,
    pub(crate) budgets: &'a stats::Budgets,
}

/// A planned output's work, cooked by the system that planned it: the
/// blob and the cook's notes.
pub(crate) trait Job: Sync {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError>;
}

/// Every system's kinds, one line a system.
pub(crate) static SYSTEMS: &[&[&KindDef]] = &[
    behaviour::KINDS,
    story::KINDS,
    strings::KINDS,
    navmesh::KINDS,
    terrain::KINDS,
    lightmap::KINDS,
    environment::KINDS,
    timeline::KINDS,
    icons::KINDS,
    font_pack::KINDS,
    look::KINDS,
    effect::KINDS,
    vfx::KINDS,
    shader_check::KINDS,
    anim::KINDS,
    vector::KINDS,
    tilemap::KINDS,
];

fn kinds() -> impl Iterator<Item = &'static KindDef> {
    SYSTEMS.iter().flat_map(|s| s.iter().copied())
}

/// The registered kind that claims a file extension.
pub(crate) fn by_extension(ext: &str) -> Option<Kind> {
    kinds()
        .find(|k| k.extensions.contains(&ext))
        .map(Kind::System)
}

/// The registered kind that claims an `.ogt` type.
pub(crate) fn by_type(ty: &str) -> Option<Kind> {
    kinds().find(|k| k.types.contains(&ty)).map(Kind::System)
}
