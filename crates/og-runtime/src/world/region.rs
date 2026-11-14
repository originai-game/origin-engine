//! The og.Volume on an entity, read in one place: the forces, the fog
//! volumes, the stream cells, the reflection probes and the light volumes
//! take their region's shape from here, so none decodes it its own way.

use crate::host::codec::Layout;
use crate::host::wire::record;
use og_ecs::{Entity, World};
use og_env::volume::Volume;

/// The og.Volume on `e` (`layout`: og.Volume's, none while no world names
/// it): its region's shape, `None` for an entity without one.
pub(crate) fn volume_of(world: &World, layout: Option<&Layout>, e: Entity) -> Option<Volume> {
    let l = layout?;
    let id = world.id_by_name(&l.schema).ok()?;
    let v: record::Volume = l.read(&world.get_bytes(e, id).ok()?).ok()?;
    Some(Volume {
        size: v.size,
        radius: v.radius,
        fade: v.fade,
    })
}
