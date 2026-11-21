//! og.ui's world-space members (`docs/ui.md` §1, §3.3) and what the
//! engine hands og-ui for them each tick: the camera anchored elements
//! project with and world panels' rays start from, the poses of the
//! entities they sit on, the frame's own mass elements in the overlay, and
//! their state (spawned mass elements, particles, anchors' eased values) as
//! a state block of its own, so snapshots and replays carry it.
//!
//! The simulation never reads any of it: the UI's end reads og.Camera and
//! the entities' transforms after the game's systems, and writes nothing.

use super::*;
use og_audio::schema::{Parent, Transform};

/// What the UI's end reads to place world-space UI.
pub(crate) struct Feed {
    camera: Option<(og_ecs::SchemaId, crate::host::codec::Layout)>,
}

impl Feed {
    pub(crate) fn of(world: &World) -> Feed {
        let camera = world.id_by_name("og.Camera").ok().and_then(|id| {
            let schema = world.schema(id).ok()?;
            Some((
                id,
                crate::host::codec::Layout::from_schema(
                    schema,
                    crate::host::codec::RecordKind::Resource,
                ),
            ))
        });
        Feed { camera }
    }

    /// The UI's end system, reading what it places with.
    pub(crate) fn desc(&self, name: &str) -> SystemDesc {
        let mut d = SystemDesc::new(name).read::<Transform>().read::<Parent>();
        if self.camera.is_some() {
            d.reads.push("og.Camera".into());
        }
        d
    }

    /// An entity's world pose by its bits, its parents' transforms applied;
    /// `None` for a dead one or one without a transform.
    pub(crate) fn world_pose(
        ctx: &og_ecs::SystemContext<'_>,
        bits: u64,
    ) -> Option<og_core::math::Pose> {
        let e = Entity::from_bits(bits);
        if !ctx.is_alive(e) {
            return None;
        }
        let pose = |e: Entity| {
            ctx.get::<Transform>(e)
                .ok()
                .map(|t| (t.position, t.quaternion, t.scale))
        };
        let local = pose(e)?;
        Some(og_core::math::world_pose(
            e,
            local,
            |c| {
                ctx.get::<Parent>(c)
                    .ok()
                    .map(|p| Entity::from_bits(p.entity))
            },
            pose,
        ))
    }

    /// The tick's camera and the poses of the entities this frame's anchors
    /// and panels sit on, handed to the UI before it ends.
    pub(crate) fn before_end(&self, ctx: &og_ecs::SystemContext<'_>, ui: &mut Ui) {
        let vp = ui.viewport();
        let view = self.camera.as_ref().and_then(|(id, layout)| {
            let bytes = ctx.resource_bytes(*id).ok()?;
            let cam: crate::host::wire::record::Camera = layout.read(&bytes).ok()?;
            Some(crate::host::api::camera_of(
                &cam,
                vp.width.max(1.0),
                vp.height.max(1.0),
            ))
        });
        ui.set_world_view(view);
        let wanted = ui.world_entities();
        if wanted.is_empty() {
            return;
        }
        let poses: Vec<(u64, og_core::math::Pose)> = wanted
            .into_iter()
            .filter_map(|bits| Some((bits, Self::world_pose(ctx, bits)?)))
            .collect();
        ui.set_world_poses(&poses);
    }
}
