//! The render origin: the frame is drawn relative to a point near the
//! camera, so the GPU's f32 arithmetic works on small numbers however far
//! from the world's zero the camera is (`docs/design.md` §2.6).
//!
//! A scene's positions are in a frame whose zero is [`RenderScene::origin`]
//! in the caller's world. [`RenderScene::rebase`] moves that zero: every
//! world-space position the frame draws is moved by the same exact amount
//! (the caller picks an origin on a coarse power-of-two grid, so the
//! subtraction is exact for every value near it). The renderer carries what
//! it keeps between frames (TAA and motion history, world-space particles,
//! trails) across a change of origin, so a move of the origin is invisible,
//! and shaders that read the world position for its look (terrain layers,
//! lake normal maps, caustics, the wind on scattered copies, the stylised
//! canvas) add `Frame.origin` back, so their patterns stay put.

use crate::RenderScene;
use crate::particles::SceneCollider;
use glam::{Mat4, Vec2, Vec3};

fn moved(m: &mut Mat4, d: Vec3) {
    m.w_axis += d.extend(0.0) * m.w_axis.w;
}

impl RenderScene {
    /// Moves the scene's zero to `origin`, a point in its current
    /// coordinates: subtracts it from every world-space position the frame
    /// draws (instances, the camera and secondary cameras, lights, sprites,
    /// text, particle systems, debris, trails, ribbons, beams, lines and points,
    /// decals, probes and light volumes, water, terrains, fog heights, 2D
    /// lights and fluids, wind motors, grass pushers) and adds it to [`RenderScene::origin`]. Screen
    /// lines, screen points and the overlay are screen space and stay.
    /// `origin` should lie on a grid coarse enough that `p - origin` is
    /// exact for the positions near it (og-runtime uses 4096 m).
    pub fn rebase(&mut self, origin: Vec3) {
        if origin == Vec3::ZERO {
            return;
        }
        self.origin += origin;
        let d = -origin;
        let d2 = d.truncate();
        for i in &mut self.instances {
            moved(&mut i.transform, d);
        }
        for m in &mut self.previous_transforms {
            moved(m, d);
        }
        for m in self
            .previous_joint_palettes
            .iter_mut()
            .flat_map(|p| &mut p.matrices)
        {
            moved(m, d);
        }
        for i in &mut self.skinned_instances {
            moved(&mut i.transform, d);
        }
        self.camera.position += d;
        self.camera.target += d;
        if let crate::ShadowBounds::Sphere { center, .. } = &mut self.light.shadow_bounds {
            *center += d;
        }
        for m in &mut self.atmosphere.wind_motors {
            m.position -= origin;
        }
        for p in &mut self.atmosphere.grass_pushers {
            p.position -= origin;
        }
        for l in &mut self.lights {
            l.position += d;
        }
        for v in &mut self.views {
            v.camera.position += d;
            v.camera.target += d;
            if let Some(m) = &mut v.mirror {
                moved(&mut m.transform, d);
            }
        }
        for s in &mut self.sprites {
            s.position += d;
        }
        for s in &mut self.sprite_shadows {
            s.ground.height += d.y;
            if let Some(b) = &mut s.ground.bend {
                for h in b.heights.iter_mut().flatten() {
                    *h += d.y;
                }
            }
        }
        for t in &mut self.texts {
            t.position += d;
        }
        for m in &mut self.sprite_meshes {
            for v in &mut m.vertices {
                v.position += d;
            }
            if let Some(skin) = &mut m.skin {
                for bone in &mut skin.bones {
                    moved(bone, d);
                }
            }
        }
        for t in &mut self.tile_layers {
            t.origin += d;
        }
        for p in &mut self.particles {
            moved(&mut p.transform, d);
        }
        for c in &mut self.colliders {
            match c {
                SceneCollider::Plane { normal, offset } => *offset += normal.dot(d),
                SceneCollider::Box { transform, .. }
                | SceneCollider::HeightField { transform, .. } => moved(transform, d),
                SceneCollider::Sphere { center, .. } => *center += d,
                SceneCollider::Capsule { a, b, .. } => {
                    *a += d;
                    *b += d;
                }
            }
        }
        for p in &mut self.debris {
            p.position += d;
        }
        for t in &mut self.trails {
            t.position += d;
        }
        for r in &mut self.ribbons {
            for p in &mut r.points {
                p.position += d;
            }
        }
        for b in &mut self.beams {
            b.start += d;
            b.end += d;
        }
        for l in self.lines.iter_mut().filter(|l| !l.screen) {
            moved(&mut l.transform, d);
        }
        for p in self.points.iter_mut().filter(|p| !p.screen) {
            moved(&mut p.transform, d);
        }
        for x in &mut self.decals {
            moved(&mut x.transform, d);
        }
        for l in &mut self.lights_2d {
            l.position += d2;
        }
        for o in &mut self.occluders_2d {
            o.center += d2;
        }
        for p in &mut self.reflection_probes {
            p.position += d;
        }
        for v in &mut self.baked.volumes {
            v.min += d;
            v.max += d;
        }
        if let Some(g) = &mut self.environment.ground {
            g.center += d;
        }
        if let Some(f) = &mut self.atmosphere.fog {
            f.height += d.y;
        }
        if let Some(f) = &mut self.atmosphere.volumetric_fog {
            f.height += d.y;
        }
        for w in &mut self.water {
            moved(&mut w.transform, d);
            if let Some(s) = &mut w.shore {
                s.origin += Vec2::new(d.x, d.z);
            }
        }
        for t in &mut self.terrain_stamps {
            moved(&mut t.transform, d);
        }
        for t in &mut self.terrains {
            moved(&mut t.transform, d);
        }
        for w in &mut self.water_2d {
            w.origin += d2;
        }
        for m in &mut self.metaballs {
            m.particles = m.particles.iter().map(|p| *p + d2).collect();
        }
        for f in &mut self.fluids_3d {
            f.particles = f.particles.iter().map(|p| *p + d).collect();
            if let Some(v) = &mut f.visual {
                let v = std::sync::Arc::make_mut(v);
                v.centre += d;
                v.emit_at += d;
                v.sink = v.sink.map(|(lo, hi)| (lo + d, hi + d));
            }
        }
    }
}

/// What the renderer kept from frames drawn at another origin, carried to
/// this frame's: `d` is the old origin minus the new (added to positions
/// in the old frame's coordinates).
pub(crate) fn shift_matrix(m: Mat4, d: Vec3) -> Mat4 {
    // A clip position of the old frame: M · p_old = M · (p_new - d).
    m * Mat4::from_translation(-d)
}
