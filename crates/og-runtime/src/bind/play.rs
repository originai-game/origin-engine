//! og-play over the runtime's engines: the poses, shapes and query answers
//! its mechanisms take as data, fed from og-anim and og-physics
//! (`docs/design.md` §2.1, mechanisms that span domains).

use crate::registry::Assembly;
use og_ecs::World;

/// og-play's systems, after og-anim (the tick's poses), before physics'
/// back. Nothing yet.
pub(crate) fn mount(_world: &mut World, _modules: &Assembly<'_>) -> Result<(), String> {
    Ok(())
}
