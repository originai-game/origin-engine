//! A frame's views as plans: each view's camera, destination and rate.

use super::*;

/// What a frame draws of one view.
pub(crate) struct ViewPlan {
    /// The view's index in [`crate::RenderScene::views`] (its GPU slot).
    pub index: usize,
    pub frame: CameraFrame,
    pub position: Vec3,
    pub near: f32,
    pub far: f32,
    /// Pixels drawn (the target's, or the viewport rect's).
    pub size: (u32, u32),
    /// The target's colour and depth texture indices, or the rect in
    /// output pixels (x, y, w, h).
    pub dest: Dest,
    pub layers: u32,
    pub clear: Option<Vec3>,
    /// The mirror's image is written mirrored across x (its uv runs as the
    /// quad's).
    pub flip: bool,
    /// The camera's manual exposure (EV); auto exposure draws at 0.
    pub ev: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Dest {
    Texture {
        color: usize,
        format: crate::TargetFormat,
        depth: Option<usize>,
    },
    Viewport([u32; 4]),
}

/// The frame's views to draw: those due this tick with a live target, in
/// order. `main`: the main camera (a mirror's eye); `out`: the output size;
/// `due(i)`: view `i` draws whatever its `every` (never drawn yet, or the
/// renderer restored).
pub(crate) fn plan(
    scene: &crate::RenderScene,
    main: &Camera,
    out: (u32, u32),
    due: impl Fn(usize) -> bool,
) -> Vec<ViewPlan> {
    let mut order: Vec<usize> = (0..scene.frame.views.len()).collect();
    og_core::sort::sort_by_key(&mut order, |&i| scene.frame.views[i].order);
    let mut plans = Vec::new();
    for i in order {
        let v = &scene.frame.views[i];
        if v.every > 1 && !scene.frame.tick.is_multiple_of(v.every as u64) && !due(i) {
            continue;
        }
        let (dest, size) = match v.target {
            ViewTarget::Texture(t) => {
                let Some(c) = scene.target_texture(t.color) else {
                    continue;
                };
                let crate::TargetKind::Color(format) = c.kind else {
                    continue;
                };
                let depth = t
                    .depth
                    .filter(|d| {
                        scene
                            .target_texture(*d)
                            .is_some_and(|d| (d.width, d.height) == (c.width, c.height))
                    })
                    .map(|d| d.index());
                (
                    Dest::Texture {
                        color: t.color.index(),
                        format,
                        depth,
                    },
                    (c.width, c.height),
                )
            }
            ViewTarget::Viewport([x, y, w, h]) => {
                let px = |f: f32, n: u32| ((f.clamp(0.0, 1.0) * n as f32).round() as u32).min(n);
                let (x0, y0) = (px(x, out.0), px(y, out.1));
                let (x1, y1) = (px(x + w, out.0), px(y + h, out.1));
                if x1 <= x0 || y1 <= y0 {
                    continue;
                }
                (
                    Dest::Viewport([x0, y0, x1 - x0, y1 - y0]),
                    (x1 - x0, y1 - y0),
                )
            }
        };
        let aspect = size.0 as f32 / size.1 as f32;
        let ev = match v.camera.exposure {
            crate::Exposure::Manual { ev } => ev,
            crate::Exposure::Auto { .. } => 0.0,
        };
        let (frame, position, near, far, flip) = match v.mirror {
            Some(m) => match mirror_frame(main, &m) {
                Some((f, eye, near)) => (f, eye, near, main.far, true),
                None => continue,
            },
            None => (
                crate::frame::camera_frame(&v.camera, aspect, glam::Vec2::ZERO),
                v.camera.position,
                v.camera.near,
                v.camera.far,
                false,
            ),
        };
        plans.push(ViewPlan {
            index: i,
            frame,
            position,
            near,
            far,
            size,
            dest,
            layers: v.layers,
            clear: v.clear,
            flip,
            ev,
        });
    }
    plans
}

/// The camera a mirror draws with: the eye reflected in the quad's plane,
/// looking through the quad with an off-axis frustum fitted to its corners
/// (Kooima's generalised perspective projection), its near plane a hair
/// past the mirror (nothing behind the glass is drawn, nor the glass
/// itself: a quad on the near plane is kept or clipped by rounding alone,
/// and the probe's CPU id pass keeps it, seeing only the glass). Seen from behind, the quad's
/// +x runs to the image's left: the resolve writes it mirrored, so the
/// target's u runs along +x. `None` when the eye is behind the mirror.
pub(crate) fn mirror_frame(eye: &Camera, m: &MirrorQuad) -> Option<(CameraFrame, Vec3, f32)> {
    let centre = m.transform.w_axis.truncate();
    let right = m.transform.x_axis.truncate();
    let up = m.transform.y_axis.truncate();
    let (sx, sy) = (right.length(), up.length());
    if sx < 1e-6 || sy < 1e-6 {
        return None;
    }
    let (right, up) = (right / sx, up / sy);
    let normal = right.cross(up).normalize_or_zero();
    let d = (eye.position - centre).dot(normal);
    if d <= 1e-4 {
        return None;
    }
    let pe = eye.position - normal * (2.0 * d);
    let hw = m.width * sx * 0.5;
    let hh = m.height * sy * 0.5;
    // From behind: screen right is the quad's -x, the eye looks along +normal.
    let vr = -right;
    let vu = up;
    let vn = -normal;
    let pa = centre + right * hw - up * hh; // lower left as seen from pe
    let pb = centre - right * hw - up * hh; // lower right
    let pc = centre + right * hw + up * hh; // upper left
    let (va, vb, vc) = (pa - pe, pb - pe, pc - pe);
    let dist = -va.dot(vn);
    if dist <= 1e-4 {
        return None;
    }
    let near = (dist * (1.0 + 1e-3)).max(1e-3);
    let far = eye.far.max(near * 2.0);
    let s = near / dist;
    let l = vr.dot(va) * s;
    let r = vr.dot(vb) * s;
    let b = vu.dot(va) * s;
    let t = vu.dot(vc) * s;
    let proj = off_axis(l, r, b, t, near, far);
    let rot = Mat4::from_cols(
        Vec4::new(vr.x, vu.x, vn.x, 0.0),
        Vec4::new(vr.y, vu.y, vn.y, 0.0),
        Vec4::new(vr.z, vu.z, vn.z, 0.0),
        Vec4::W,
    );
    let view = rot * Mat4::from_translation(-pe);
    let view_proj = proj * view;
    Some((
        CameraFrame {
            view,
            proj,
            view_proj,
            jittered_view_proj: view_proj,
            forward: -vn,
            orthographic: false,
        },
        pe,
        near,
    ))
}

/// A right-handed off-centre perspective projection with depth 0..1 (as
/// glam's `perspective_rh`, for an asymmetric frustum).
pub(crate) fn off_axis(l: f32, r: f32, b: f32, t: f32, n: f32, f: f32) -> Mat4 {
    Mat4::from_cols(
        Vec4::new(2.0 * n / (r - l), 0.0, 0.0, 0.0),
        Vec4::new(0.0, 2.0 * n / (t - b), 0.0, 0.0),
        Vec4::new((r + l) / (r - l), (t + b) / (t - b), f / (n - f), -1.0),
        Vec4::new(0.0, 0.0, n * f / (n - f), 0.0),
    )
}
