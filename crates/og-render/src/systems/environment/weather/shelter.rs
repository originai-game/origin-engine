//! The weather occlusion map: where a roof keeps the rain and snow off.

use super::SurfaceWeather;
use glam::Vec3;

/// How far above and below the camera the shelter map reaches, in metres
/// along the fall.
pub(crate) const SHELTER_REACH: f32 = 400.0;
/// The shelter map's cells per side of its square: the camera moving a
/// cell redraws it.
pub(crate) const SHELTER_CELLS: f32 = 8.0;
/// How wide the drip line's soft edge is, in metres.
pub(crate) const DRIP_EDGE: f32 = 0.6;

/// The weather occlusion map's view (the R4 shelter): an orthographic
/// projection looking along `w.fall` over the `w.shelter` square around
/// `camera` (frame coordinates), centred on the snapped cell of the
/// camera's world position (`camera + origin`), so it moves only when the
/// camera crosses a cell. `None` without shelter. Its depth is what the
/// shadow pass writes for a directional cascade (clip z), and a surface is
/// sheltered where something nearer the sky along the fall was drawn.
/// Returns the view projection and the cell.
pub(crate) fn shelter_view(
    w: &SurfaceWeather,
    camera: Vec3,
    origin: Vec3,
) -> Option<(glam::Mat4, [i64; 2])> {
    if w.shelter <= 0.0 {
        return None;
    }
    let side = w.shelter.clamp(4.0, 1024.0);
    let cell = side / SHELTER_CELLS;
    let world = camera + origin;
    let ix = (world.x / cell).floor() as i64;
    let iz = (world.z / cell).floor() as i64;
    let centre = Vec3::new((ix as f32 + 0.5) * cell, world.y, (iz as f32 + 0.5) * cell) - origin;
    let d = w.fall.normalize_or(Vec3::NEG_Y);
    let up = if d.y.abs() > 0.9 { Vec3::Z } else { Vec3::Y };
    let eye = centre - d * SHELTER_REACH;
    let view = glam::camera::rh::view::look_at_mat4(eye, centre, up);
    let h = side * 0.5;
    let proj =
        glam::camera::rh::proj::directx::orthographic(-h, h, -h, h, 0.0, 2.0 * SHELTER_REACH);
    Some((proj * view, [ix, iz]))
}

/// What the shelter map was drawn from: its cell, its fall and side, and
/// the casters inside its volume (their bounding spheres, hashed). The map
/// is drawn again only when this changes: the camera crosses a cell, the
/// weather turns, or geometry under the map moves, appears or goes (a
/// static shadow cache, when the shadows have one, is where this should
/// later live with theirs).
pub(crate) fn shelter_key(
    w: &SurfaceWeather,
    cell: [i64; 2],
    spheres: impl Iterator<Item = (Vec3, f32)>,
    terrains: usize,
) -> u64 {
    let bits = |v: f32| u64::from(v.to_bits());
    let weather = [w.fall.x, w.fall.y, w.fall.z, w.shelter].map(bits);
    let spheres = spheres.flat_map(|(c, r)| [c.x, c.y, c.z, r].map(bits));
    og_core::hash::hash_words(
        [cell[0] as u64, cell[1] as u64]
            .into_iter()
            .chain(weather)
            .chain([terrains as u64])
            .chain(spheres),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shelter view moves by whole cells only, looks along the fall,
    /// and maps a point under the camera into the map; the key changes
    /// with the cell and with a caster moving, not with a camera move
    /// inside its cell.
    #[test]
    fn the_shelter_map_moves_by_cells_and_keys_on_its_casters() {
        let w = SurfaceWeather::default();
        let (a, cell) = shelter_view(&w, Vec3::new(1.0, 2.0, 1.0), Vec3::ZERO).unwrap();
        let (b, same) = shelter_view(&w, Vec3::new(3.0, 2.0, 2.0), Vec3::ZERO).unwrap();
        assert_eq!(cell, same);
        assert_eq!(a, b);
        let (_, next) = shelter_view(&w, Vec3::new(9.0, 2.0, 1.0), Vec3::ZERO).unwrap();
        assert_ne!(cell, next);
        // A roof 3 m up is nearer the sky (smaller depth) than the ground.
        let depth = |p: Vec3| {
            let c = a * p.extend(1.0);
            c.z / c.w
        };
        assert!(depth(Vec3::new(1.0, 3.0, 1.0)) < depth(Vec3::new(1.0, 0.0, 1.0)));
        let c = a * Vec3::new(1.0, 0.0, 1.0).extend(1.0);
        assert!(c.x.abs() < 1.0 && c.y.abs() < 1.0 && (0.0..1.0).contains(&(c.z / c.w)));
        let caster = [(Vec3::new(0.0, 3.0, 0.0), 1.0)];
        let k = shelter_key(&w, cell, caster.into_iter(), 0);
        assert_eq!(k, shelter_key(&w, same, caster.into_iter(), 0));
        assert_ne!(k, shelter_key(&w, next, caster.into_iter(), 0));
        let moved = [(Vec3::new(0.5, 3.0, 0.0), 1.0)];
        assert_ne!(k, shelter_key(&w, cell, moved.into_iter(), 0));
        assert!(
            shelter_view(
                &SurfaceWeather { shelter: 0.0, ..w },
                Vec3::ZERO,
                Vec3::ZERO
            )
            .is_none()
        );
    }
}
