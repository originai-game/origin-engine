//! Camera stage.

use super::*;

/// A planar reflection: its water's index, the mirrored camera and its
/// frame, its view-projection and its target size.
pub(super) type PlanarReflection = (usize, Camera, frame::CameraFrame, Mat4, (u32, u32));

impl Renderer {
    /// This frame's TAA jitter in scene pixels (x right, y down: the
    /// image moves by it), over more phases under TAAU
    /// ([`frame::jitter_phases`]).
    pub(super) fn taa_jitter(&self) -> Vec2 {
        if self.info.antialias == Antialias::SmaaT2x {
            return frame::smaa_sample(self.phase()).0;
        }
        let phases = frame::jitter_phases((self.width, self.height), self.history_size());
        frame::halton_jitter(self.phase(), phases)
    }

    pub(super) fn prepare_frame_camera(
        &self,
        scene: &RenderScene,
    ) -> (f32, bool, Camera, frame::CameraFrame, Frustum) {
        // Camera (with the shake), jitter, culling.
        let aspect = self.width as f32 / self.height as f32;
        let taa = self.info.antialias.temporal();
        let jitter = if taa {
            let px = self.taa_jitter();
            Vec2::new(
                px.x * 2.0 / self.width as f32,
                -px.y * 2.0 / self.height as f32,
            )
        } else {
            Vec2::ZERO
        };
        let camera = shaken_camera(&scene.camera, scene.shake);
        let cam = frame::camera_frame(&camera, aspect, jitter);
        let view_frustum = Frustum::from_view_proj(cam.view_proj);
        (aspect, taa, camera, cam, view_frustum)
    }

    #[allow(clippy::type_complexity)]
    pub(super) fn prepare_frame_reflections(
        &mut self,
        scene: &RenderScene,
        scene_water: &[Water],
        &camera: &Camera,
        cam: &frame::CameraFrame,
        view_frustum: &Frustum,
        &aspect: &f32,
    ) -> (Option<VolumetricFog>, Option<Fog>, Vec<PlanarReflection>) {
        // Water: the surface the camera is under (its fog replaces the
        // frame's), and the one drawn with a planar reflection (the camera
        // mirrored in it, clipped at the surface by an oblique near plane).
        // W6: where the near plane crosses a surface the frame is the air's,
        // the waterline pass fogging what lies under it (water.rs).
        let line = water::waterline(scene_water, &camera, water::seconds(scene), scene.origin);
        let under = water::underwater(
            scene_water,
            camera.position,
            water::seconds(scene),
            scene.origin,
        )
        .filter(|_| line.is_none());
        // Volumetric fog replaces the plain fog (volumetric.rs): above
        // water, for a perspective camera, where the quality lets it.
        let volumetric_fog = scene.volumetric_fog.filter(|_| {
            WORLD_3D && scene.quality.volumetric_fog && under.is_none() && !cam.orthographic
        });
        let frame_fog = match under {
            Some(i) => Some(water::underwater_fog(&scene_water[i])),
            None if volumetric_fog.is_some() => None,
            None => scene.fog,
        };
        // Up to `planar_reflections` surfaces mirror the scene, each in its own pass: slot 0 a grid surface first,
        // such as a lake, else the first that asks (with two, the one material shaders read: `planar_reflection`),
        // slot 1 a second surface, only while it is in view (only it reads its mirror). With one, a second surface
        // that asks reflects as without it (RenderDesc::planar_reflections), never in the first's mirror.
        let planar: Vec<usize> = match (under, cam.orthographic) {
            (None, false) => {
                let asks: Vec<(usize, bool, bool)> = scene_water
                    .iter()
                    .enumerate()
                    .filter_map(|(i, w)| {
                        let (n, at, lo, hi) = water::surface(w, w.mesh.and_then(|m| scene.mesh(m)));
                        let c = Vec3::new((lo.x + hi.x) * 0.5, at.y, (lo.y + hi.y) * 0.5);
                        let seen =
                            view_frustum.intersects_sphere(c, (hi - lo).length() * 0.5 + 0.5);
                        (w.planar_reflection > 0.0 && (camera.position - at).dot(n) > 0.0)
                            .then_some((i, w.mesh.is_none(), seen))
                    })
                    .collect();
                let first = asks.iter().find(|a| a.1).or(asks.first()).map(|a| a.0);
                let second = asks.iter().find(|a| Some(a.0) != first && a.2).map(|a| a.0);
                if second.is_some()
                    && self.info.planar_reflections < 2
                    && !self.reflections.warned_planar
                {
                    self.reflections.warned_planar = true;
                    log::warn!(
                        "og-render: a second water surface in view asks for a planar reflection; the renderer \
                         draws one (RenderOptions.planarReflections 1), so it reflects without one"
                    );
                }
                first
                    .into_iter()
                    .chain(second)
                    .take(self.info.planar_reflections as usize)
                    .collect()
            }
            _ => Vec::new(),
        };
        let reflections: Vec<_> = planar
            .iter()
            .map(|&i| {
                let w = &scene_water[i];
                let (n, at, ..) = water::surface(w, w.mesh.and_then(|m| scene.mesh(m)));
                let mirrored = water::mirrored_camera(&camera, n, at);
                let f = frame::camera_frame(&mirrored, aspect, Vec2::ZERO);
                let proj = f.view_proj * f.view.inverse();
                let view_proj = water::oblique(proj, f.view, n, at, 0.02) * f.view;
                let scale = w.planar_reflection.clamp(0.1, 1.0);
                let size = (
                    ((self.width as f32 * scale) as u32).max(1),
                    ((self.height as f32 * scale) as u32).max(1),
                );
                (i, mirrored, f, view_proj, size)
            })
            .collect();
        (volumetric_fog, frame_fog, reflections)
    }
}
