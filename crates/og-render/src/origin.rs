//! The render origin: the frame is drawn relative to a point near the
//! camera, so the GPU's f32 arithmetic works on small numbers however far
//! from the world's zero the camera is (`docs/design.md` §5.4).
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
use crate::particles::ParticleCollider;
use glam::{Mat4, Vec3};

fn moved(m: &mut Mat4, d: Vec3) {
    m.w_axis += d.extend(0.0) * m.w_axis.w;
}

impl RenderScene {
    /// Moves the scene's zero to `origin`, a point in its current
    /// coordinates: subtracts it from every world-space position the frame
    /// draws (instances, the camera and secondary cameras, lights, sprites,
    /// text, particle systems, trails, ribbons, beams, lines and points,
    /// decals, probes and light volumes, water, terrains, fog heights, 2D
    /// lights and fluids) and adds it to [`RenderScene::origin`]. Screen
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
        for i in &mut self.skinned_instances {
            moved(&mut i.transform, d);
        }
        self.camera.position += d;
        self.camera.target += d;
        if let crate::ShadowBounds::Sphere { center, .. } = &mut self.light.shadow_bounds {
            *center += d;
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
        }
        for t in &mut self.tile_layers {
            t.origin += d;
        }
        for p in &mut self.particles {
            moved(&mut p.transform, d);
        }
        for c in &mut self.particle_colliders {
            match c {
                ParticleCollider::Plane { normal, offset } => *offset += normal.dot(d),
                ParticleCollider::Box { transform, .. }
                | ParticleCollider::HeightField { transform, .. } => moved(transform, d),
                ParticleCollider::Sphere { center, .. } => *center += d,
            }
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
        if let Some(f) = &mut self.fog {
            f.height += d.y;
        }
        if let Some(f) = &mut self.volumetric_fog {
            f.height += d.y;
        }
        for w in &mut self.water {
            moved(&mut w.transform, d);
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
