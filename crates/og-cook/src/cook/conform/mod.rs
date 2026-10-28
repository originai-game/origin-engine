//! Conformance (docs/art.md §2.3): the cook makes each asset meet the
//! project's art contract (project.ogt's `look`, `look::contract`) as it
//! cooks it. Sources are never written; the cooked blob conforms, and every
//! change and opt-out is a note on the asset's report starting `conform:`.
//! A project without a look, or whose look sets none of the art resources a
//! conformer reads, cooks exactly as before: no stamp and no byte moves.
//!
//! - `mesh`: models (glTF, `type model`, `.vox`), the textures they use,
//!   and animation clips and flipbooks (ArtShape, ArtTexels, ArtPalette,
//!   ArtLine, ArtMotion).

pub mod mesh;
