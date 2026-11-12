pub use og_render_data::impostor::*;

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Vec2, Vec3};

    #[test]
    fn the_hemi_octahedral_map_round_trips_above_the_horizon() {
        for d in [
            Vec3::Y,
            Vec3::X,
            -Vec3::Z,
            Vec3::new(0.3, 0.8, -0.5).normalize(),
            Vec3::new(-0.7, 0.1, 0.7).normalize(),
            Vec3::new(0.2, 0.01, -0.97).normalize(),
        ] {
            let back = hemi_oct_decode(hemi_oct_encode(d));
            assert!((back - d).length() < 1e-5, "{d} came back {back}");
        }
        // +y at the centre, the horizon on the border.
        assert_eq!(hemi_oct_encode(Vec3::Y), Vec2::ZERO);
        assert!((hemi_oct_encode(Vec3::X) - Vec2::ONE).length() < 1e-6);
        assert!((hemi_oct_encode(Vec3::Z) - Vec2::new(1.0, -1.0)).length() < 1e-6);
        // Below the horizon: the horizon above it; straight down, up.
        let below = Vec3::new(0.6, -0.5, 0.2).normalize();
        let level = Vec3::new(0.6, 0.0, 0.2).normalize();
        assert!((hemi_oct_encode(below) - hemi_oct_encode(level)).length() < 1e-6);
        assert_eq!(hemi_oct_encode(-Vec3::Y), Vec2::ZERO);
    }

    #[test]
    fn frames_span_the_hemisphere_and_blend_to_the_direction() {
        // An 8-sided grid: the centre four frames look down from above,
        // the border's from the horizon, a corner along ±x.
        assert!(frame_direction(3, 3, 8).y > 0.6);
        assert!((frame_direction(7, 7, 8) - Vec3::X).length() < 1e-6);
        assert!((frame_direction(0, 0, 8) + Vec3::X).length() < 1e-6);
        for k in 0..8 {
            assert!(frame_direction(k, 0, 8).y.abs() < 1e-6);
            assert!(frame_direction(0, k, 8).y.abs() < 1e-6);
        }
        // A level view blends frames on the horizon and above it only.
        for &((i, j), w) in &blend_frames(Vec3::new(0.3, 0.02, 0.95).normalize(), 8) {
            assert!(w < 1e-4 || frame_direction(i, j, 8).y >= -1e-6);
        }
        // A direction on a frame weighs that frame alone.
        let d = frame_direction(5, 2, 8);
        let w = blend_frames(d, 8);
        let on: Vec<_> = w.iter().filter(|(_, x)| *x > 1e-4).collect();
        assert_eq!(on.len(), 1);
        assert_eq!(on[0].0, (5, 2));
        // Between frames: three of them, weights summing to 1, the
        // nearest weighing most; their directions blend to near d.
        let d = Vec3::new(0.4, 0.3, -0.6).normalize();
        let w = blend_frames(d, 8);
        assert!((w.iter().map(|x| x.1).sum::<f32>() - 1.0).abs() < 1e-5);
        let mix: Vec3 = w
            .iter()
            .map(|&((i, j), x)| frame_direction(i, j, 8) * x)
            .sum();
        assert!(mix.normalize().dot(d) > 0.98);
        let best = w.iter().max_by(|a, b| a.1.total_cmp(&b.1)).unwrap().0;
        let nearest = (0..8)
            .flat_map(|i| (0..8).map(move |j| (i, j)))
            .max_by(|a, b| {
                frame_direction(a.0, a.1, 8)
                    .dot(d)
                    .total_cmp(&frame_direction(b.0, b.1, 8).dot(d))
            })
            .unwrap();
        assert_eq!(best, nearest);
    }

    #[test]
    fn a_frame_basis_is_right_handed_and_upright() {
        for d in [
            Vec3::X,
            Vec3::new(0.3, 0.5, -0.8).normalize(),
            Vec3::Y,
            -Vec3::Y,
        ] {
            let (r, u) = frame_basis(d);
            assert!(r.dot(d).abs() < 1e-5 && u.dot(d).abs() < 1e-5 && r.dot(u).abs() < 1e-5);
            assert!((r.cross(u) - d).length() < 1e-5);
        }
        // Seen from the side, up is +y and right is horizontal.
        let (r, u) = frame_basis(Vec3::Z);
        assert!((u - Vec3::Y).length() < 1e-6 && (r - Vec3::X).length() < 1e-6);
    }
}
