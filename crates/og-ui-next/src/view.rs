//! The tick's camera, as the UI projects anchored elements and casts
//! pointer rays with it.

use og_core::glam::{Mat4, Vec2, Vec3, Vec4};

/// The camera anchored elements project with and panel rays start from:
/// the tick's camera, handed in by the host before og-ui's `Ui::end`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldView {
    /// Projection times view (right handed, depth 0..1 as og-render's).
    pub view_proj: Mat4,
    pub eye: Vec3,
    /// Unit vector the camera looks along.
    pub forward: Vec3,
    pub orthographic: bool,
}

impl WorldView {
    /// A perspective camera at `eye` looking at `target` (`fov_y` degrees,
    /// `aspect` width over height), built with og-core's math.
    pub fn look_at(
        eye: Vec3,
        target: Vec3,
        up: Vec3,
        fov_y: f32,
        aspect: f32,
        near: f32,
        far: f32,
    ) -> WorldView {
        // look_at_rh by hand (products only).
        let f = (target - eye).normalize_or(Vec3::NEG_Z);
        let side = f.cross(up).normalize_or(Vec3::X);
        let u = side.cross(f);
        let view = Mat4::from_cols(
            Vec4::new(side.x, u.x, -f.x, 0.0),
            Vec4::new(side.y, u.y, -f.y, 0.0),
            Vec4::new(side.z, u.z, -f.z, 0.0),
            Vec4::new(-side.dot(eye), -u.dot(eye), f.dot(eye), 1.0),
        );
        let (s, c) = og_core::math::sin_cos(fov_y.to_radians() / 2.0);
        let f = c / s;
        let r = far / (near - far);
        let proj = Mat4::from_cols(
            Vec4::new(f / aspect.max(1e-6), 0.0, 0.0, 0.0),
            Vec4::new(0.0, f, 0.0, 0.0),
            Vec4::new(0.0, 0.0, r, -1.0),
            Vec4::new(0.0, 0.0, r * near, 0.0),
        );
        WorldView {
            view_proj: proj * view,
            eye,
            forward: (target - eye).normalize_or(Vec3::NEG_Z),
            orthographic: false,
        }
    }

    /// `p` on a `w`×`h` CSS px surface: (x, y, depth along the view axis);
    /// `None` behind the eye (its mirrored direction is in
    /// [`WorldView::direction`]).
    pub fn project(&self, p: Vec3, w: f32, h: f32) -> Option<(f32, f32, f32)> {
        let c = self.view_proj * p.extend(1.0);
        let depth = (p - self.eye).dot(self.forward);
        if c.w <= 1e-5 || depth <= 0.0 {
            return None;
        }
        let n = c.truncate() / c.w;
        Some(((n.x + 1.0) * 0.5 * w, (1.0 - n.y) * 0.5 * h, depth))
    }

    /// The screen direction from the centre toward `p`, unit, y down; a
    /// point behind the eye points the way it would turn to. Clip space's
    /// x and y before the divide are the view's sideways and up offsets
    /// scaled by the projection, so their signs hold behind the eye too
    /// (dividing by a negative w would mirror them).
    pub fn direction(&self, p: Vec3, w: f32, h: f32) -> Vec2 {
        let c = self.view_proj * p.extend(1.0);
        // Clip space to CSS px proportions, y down.
        Vec2::new(c.x * w, -c.y * h).normalize_or(Vec2::Y)
    }

    /// The ray through CSS px (`x`, `y`) of a `w`×`h` surface.
    pub fn ray(&self, x: f32, y: f32, w: f32, h: f32) -> (Vec3, Vec3) {
        let inv = self.view_proj.inverse();
        let nx = x / w.max(1e-6) * 2.0 - 1.0;
        let ny = 1.0 - y / h.max(1e-6) * 2.0;
        let near = inv.project_point3(Vec3::new(nx, ny, 0.0));
        let far = inv.project_point3(Vec3::new(nx, ny, 1.0));
        (near, (far - near).normalize_or(self.forward))
    }
}
