//! What a caller hands physics: the shapes it registers and the scene
//! queries' requests and answers.

use super::*;

/// A height field for [`SHAPE_HEIGHTFIELD`] colliders, registered under the
/// handle the colliders name ([`Collider::mesh`]): `columns` × `rows`
/// heights over a `size` rectangle centred on the collider, row-major from
/// -z to +z and -x to +x (og-render's `TerrainData`). Each cell is split
/// along the diagonal with the smaller corner height difference; ties keep
/// (-x, -z) to (+x, +z), as the renderer draws it.
#[derive(Clone, Debug, PartialEq)]
pub struct HeightfieldDesc {
    pub columns: u32,
    pub rows: u32,
    pub size: Vec2,
    pub heights: Arc<[f32]>,
    /// The cells cut out (a terrain's holes): `(columns - 1)` ×
    /// `(rows - 1)` bytes, row-major like the heights, non-zero removed;
    /// nothing collides with a removed cell. `None`: none.
    pub holes: Option<Arc<[u8]>>,
}

/// The shape of a shape cast or an overlap test: the shape fields of a
/// [`Collider`]. `Default` is a 1 m box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShapeDesc {
    /// One of the `SHAPE_*` codes ([`SHAPE_BOX`], [`SHAPE_SPHERE`], ...).
    pub shape: u32,
    pub size: Vec3,
    pub radius: f32,
    pub height: f32,
    pub mesh: u64,
}

impl Default for ShapeDesc {
    fn default() -> Self {
        let c = Collider::default();
        ShapeDesc {
            shape: c.shape,
            size: c.size,
            radius: c.radius,
            height: c.height,
            mesh: c.mesh,
        }
    }
}

impl ShapeDesc {
    pub(crate) fn collider(&self) -> Collider {
        Collider {
            shape: self.shape,
            size: self.size,
            radius: self.radius,
            height: self.height,
            mesh: self.mesh,
            ..Collider::default()
        }
    }
}

/// Which colliders a query considers. The zero value considers every solid
/// collider on every layer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct QueryFilter {
    /// Layers to consider (a collider counts when its `layers` meet this); 0 means every layer.
    pub mask: u32,
    /// Also report sensors.
    pub include_sensors: bool,
    /// An entity to ignore (the caster itself, say).
    pub exclude: Option<Entity>,
}

/// A ray. `max_distance` 0 means unbounded.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RaycastDesc {
    pub origin: Vec3,
    /// Need not be normalised; must not be zero.
    pub direction: Vec3,
    pub max_distance: f32,
    pub filter: QueryFilter,
}

/// The nearest collider a ray meets.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RaycastHit {
    pub entity: Entity,
    /// The body the collider belongs to: the entity itself, or for a
    /// compound child collider the ancestor body it is part of.
    pub body: Entity,
    /// Distance from the origin along the normalised direction.
    pub distance: f32,
    pub point: Vec3,
    /// The surface normal at `point` (z = 0 in 2D).
    pub normal: Vec3,
}

/// Every collider along a ray ([`PhysicsQuery::raycast_all`]): the ray, and
/// at most `max_hits` hits (0: [`MAX_RAYCAST_HITS`]).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RaycastAllDesc {
    pub ray: RaycastDesc,
    pub max_hits: u32,
}

/// The most hits [`PhysicsQuery::raycast_all`] returns.
pub const MAX_RAYCAST_HITS: u32 = 256;

/// The nearest point on a collider to a point ([`PhysicsQuery::project_point`]).
/// `max_distance` 0 means unbounded; `solid`: a point inside a collider is
/// its own projection (else it projects onto the surface).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ProjectPointDesc {
    pub point: Vec3,
    pub max_distance: f32,
    pub solid: bool,
    pub filter: QueryFilter,
}

/// Where a point projects onto the nearest collider.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointProjection {
    pub entity: Entity,
    pub body: Entity,
    /// The projected point, in world space.
    pub point: Vec3,
    /// The direction from the surface towards the queried point (out of the
    /// collider; into it when the point is inside); zero when the point is
    /// on the surface or a solid projection of a point inside.
    pub normal: Vec3,
    /// From the queried point to the projection.
    pub distance: f32,
    /// Whether the point is inside the collider.
    pub inside: bool,
}

/// A shape swept along a direction. `max_distance` 0 means unbounded.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ShapeCastDesc {
    pub shape: ShapeDesc,
    pub position: Vec3,
    /// Zero is treated as no rotation.
    pub rotation: Quat,
    /// Need not be normalised; must not be zero.
    pub direction: Vec3,
    pub max_distance: f32,
    pub filter: QueryFilter,
}

/// The first collider a swept shape meets.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShapeCastHit {
    pub entity: Entity,
    /// How far the shape travelled before touching.
    pub distance: f32,
    /// The contact point on the hit collider, in world space.
    pub point: Vec3,
    /// The hit collider's surface normal there, in world space.
    pub normal: Vec3,
}

/// A shape placed in the world, for an overlap test.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OverlapDesc {
    pub shape: ShapeDesc,
    pub position: Vec3,
    /// Zero is treated as no rotation.
    pub rotation: Quat,
    pub filter: QueryFilter,
}
