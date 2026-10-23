//! What a reflection probe's capture shows, as one hash
//! (`docs/design.md` §2.6): the key a baked capture is stored under
//! includes it, so a changed room is a different key and its old bake is
//! never sampled (`og bake probes`, og-runtime's `probes.rs`).
//!
//! The room is everything the capture draws that touches the probe's box:
//! each instance and skinned instance whose bounds reach it (its mesh's
//! vertices and indices, its material's factors and texture pixels, its
//! transform to the millimetre; a skinned one in its bind pose, since a
//! pose changes every frame), each terrain whose bounds reach it (its
//! height field's revision, transform and layers), each scatter over such
//! a terrain, the local lights whose range reaches it, the sun and the
//! environment. Content is hashed by value, never by handle, so the same
//! room loaded in another order is the same key. Meshes and textures are
//! hashed once per revision ([`ProbeContent`] keeps them).

use crate::probes::ReflectionProbe;
use crate::{MaterialHandle, MeshHandle, RenderScene, TextureHandle};
use glam::{Mat4, Vec3};
use og_core::hash::StableHasher;
use std::collections::HashMap;

/// A probe's room: its hash, and the meshes and materials drawn in it (by
/// which `og bake probes` names the assets a bake depends on).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProbeRoom {
    pub hash: u64,
    pub meshes: Vec<MeshHandle>,
    pub materials: Vec<MaterialHandle>,
}

/// A mesh's content hash and local bounds (min, max).
type MeshEntry = (u64, Option<(Vec3, Vec3)>);

/// The hashes of meshes and textures by (index, revision), and each mesh's
/// local bounds, kept across frames.
#[derive(Default)]
pub struct ProbeContent {
    meshes: HashMap<(usize, u64), MeshEntry>,
    textures: HashMap<(usize, u64), u64>,
    /// Materials hashed during one [`ProbeContent::room`].
    materials: HashMap<MaterialHandle, u64>,
}

fn mm(h: &mut StableHasher, v: f32) {
    h.write_u64((v * 1000.0).round() as i64 as u64);
}

fn matrix(h: &mut StableHasher, m: &Mat4) {
    for v in m.to_cols_array() {
        mm(h, v);
    }
}

/// Whether a sphere touches a box.
fn touches(lo: Vec3, hi: Vec3, c: Vec3, r: f32) -> bool {
    c.clamp(lo, hi).distance_squared(c) <= r * r
}

impl ProbeContent {
    /// A mesh's content hash and local bounds (min, max).
    fn mesh(&mut self, scene: &RenderScene, m: MeshHandle) -> MeshEntry {
        let key = (m.index(), scene.mesh_revision(m.index()));
        *self.meshes.entry(key).or_insert_with(|| {
            let Some(data) = scene.mesh(m) else {
                return (0, None);
            };
            let mut h = StableHasher::new();
            h.write(bytemuck::cast_slice(&data.vertices));
            h.write(bytemuck::cast_slice(&data.indices));
            let bounds = data
                .vertices
                .iter()
                .fold(None, |b: Option<(Vec3, Vec3)>, v| {
                    let p = Vec3::from(v.position);
                    Some(b.map_or((p, p), |(lo, hi)| (lo.min(p), hi.max(p))))
                });
            (h.finish(), bounds)
        })
    }

    fn texture(&mut self, scene: &RenderScene, t: TextureHandle) -> u64 {
        let key = (t.index(), scene.texture_revision(t.index()));
        *self.textures.entry(key).or_insert_with(|| {
            let mut h = StableHasher::new();
            if let Some(data) = scene.texture(t) {
                h.write_u32(data.width);
                h.write_u32(data.height);
                h.write(&data.rgba);
            } else if let Some(up) = scene.texture_upload(t) {
                h.write_str(&format!("{:?}", up.desc));
                for level in &up.levels {
                    h.write(level);
                }
            } else {
                // A render target: what it holds changes as it is drawn.
                h.write_str(&format!("{:?}", scene.texture_size(t)));
            }
            h.finish()
        })
    }

    /// A material's hash: its factors and its textures' pixels.
    fn material(&mut self, scene: &RenderScene, m: MaterialHandle) -> u64 {
        if let Some(&h) = self.materials.get(&m) {
            return h;
        }
        let h = self.material_hash(scene, m);
        self.materials.insert(m, h);
        h
    }

    fn material_hash(&mut self, scene: &RenderScene, m: MaterialHandle) -> u64 {
        let Some(sm) = scene.standard_material(m) else {
            return 0;
        };
        let textures = sm.textures();
        let mut plain = *sm;
        for slot in [
            &mut plain.map,
            &mut plain.metalnessMap,
            &mut plain.roughnessMap,
            &mut plain.normalMap,
            &mut plain.emissiveMap,
            &mut plain.aoMap,
            &mut plain.gradientMap,
            &mut plain.dissolveMap,
        ] {
            *slot = 0;
        }
        let mut h = StableHasher::new();
        h.write_str(&format!("{plain:?}"));
        for t in textures {
            h.write_u64(t.map_or(0, |t| self.texture(scene, t)));
        }
        h.finish()
    }

    /// The room of `probe` in `scene`.
    pub fn room(&mut self, scene: &RenderScene, probe: &ReflectionProbe) -> ProbeRoom {
        let (lo, hi) = probe.bounds();
        self.materials.clear();
        let mut room = ProbeRoom::default();
        // Per drawn thing, a hash; sorted, so submission order does not count.
        let mut items: Vec<u64> = Vec::new();
        let draw = |this: &mut ProbeContent,
                    room: &mut ProbeRoom,
                    kind: u32,
                    mesh: MeshHandle,
                    material: MaterialHandle,
                    transform: &Mat4|
         -> Option<u64> {
            let (mesh_hash, bounds) = this.mesh(scene, mesh);
            let (c, r) = match bounds {
                Some((a, b)) => {
                    let scale = transform
                        .x_axis
                        .truncate()
                        .length()
                        .max(transform.y_axis.truncate().length())
                        .max(transform.z_axis.truncate().length());
                    (
                        transform.transform_point3((a + b) * 0.5),
                        (b - a).length() * 0.5 * scale,
                    )
                }
                None => (transform.w_axis.truncate(), 0.0),
            };
            if !touches(lo, hi, c, r) {
                return None;
            }
            let mut h = StableHasher::new();
            h.write_u32(kind);
            h.write_u64(mesh_hash);
            h.write_u64(this.material(scene, material));
            matrix(&mut h, transform);
            if !room.meshes.contains(&mesh) {
                room.meshes.push(mesh);
            }
            if !room.materials.contains(&material) {
                room.materials.push(material);
            }
            Some(h.finish())
        };
        for inst in &scene.instances {
            items.extend(draw(
                self,
                &mut room,
                0,
                inst.mesh,
                inst.material,
                &inst.transform,
            ));
        }
        for s in &scene.skinned_instances {
            items.extend(draw(self, &mut room, 1, s.mesh, s.material, &s.transform));
        }
        let terrain_bounds = |t: &crate::Terrain| {
            let d = scene.terrain(t.terrain)?;
            let (s, range) = (d.size, d.range());
            let a = Vec3::new(-s.x * 0.5, range.0, -s.y * 0.5);
            let b = Vec3::new(s.x * 0.5, range.1, s.y * 0.5);
            Some(crate::terrain::world_sphere(t.transform, a, b))
        };
        let mut terrains_in = Vec::new();
        for (i, t) in scene.terrains.iter().enumerate() {
            let Some((c, r)) = terrain_bounds(t) else {
                continue;
            };
            if !touches(lo, hi, c, r) {
                continue;
            }
            terrains_in.push(i);
            let mut h = StableHasher::new();
            h.write_u32(2);
            h.write_u64(scene.terrain_revision(t.terrain.index()));
            matrix(&mut h, &t.transform);
            h.write_str(&format!("{:?}", t.layers));
            for tex in t.layers.iter().flat_map(|l| [l.albedo, l.normal]) {
                h.write_u64(tex.map_or(0, |tex| self.texture(scene, tex)));
            }
            items.push(h.finish());
        }
        for s in scene
            .scatters
            .iter()
            .filter(|s| terrains_in.contains(&s.terrain))
        {
            let mut h = StableHasher::new();
            h.write_u32(3);
            h.write_u64(self.mesh(scene, s.mesh).0);
            h.write_u64(self.material(scene, s.material));
            h.write_str(&format!("{:?}", s.scatter));
            if !room.meshes.contains(&s.mesh) {
                room.meshes.push(s.mesh);
            }
            if !room.materials.contains(&s.material) {
                room.materials.push(s.material);
            }
            items.push(h.finish());
        }
        for l in &scene.lights {
            if !touches(lo, hi, l.position, l.range.max(0.0)) {
                continue;
            }
            let mut h = StableHasher::new();
            h.write_u32(4);
            h.write_str(&format!("{l:?}"));
            items.push(h.finish());
        }
        items.sort_unstable();
        let mut h = StableHasher::new();
        for i in items {
            h.write_u64(i);
        }
        // The sun only when it shines (a closed room's sun is off, whatever
        // way it points).
        if scene.light.intensity > 0.0 {
            h.write_str(&format!("{:?}", scene.light));
        }
        h.write_str(&format!("{:?}", scene.environment));
        room.hash = h.finish();
        room
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{StandardMaterial, shapes};

    fn room(red: f32, at: f32, order: bool) -> RenderScene {
        let mut scene = RenderScene::new();
        let a = scene.add_mesh(shapes::cuboid(Vec3::ONE));
        let b = scene.add_mesh(shapes::sphere(8, 6));
        let m = scene.add_standard_material(StandardMaterial {
            color: Vec3::new(red, 0.2, 0.2),
            ..Default::default()
        });
        let items = [(a, Vec3::new(at, 0.0, 0.0)), (b, Vec3::new(1.0, 1.0, 0.0))];
        for k in 0..2 {
            let (mesh, p) = items[if order { k } else { 1 - k }];
            scene.add_instance(mesh, m, Mat4::from_translation(p));
        }
        // Far outside the box: never in the room.
        scene.add_instance(a, m, Mat4::from_translation(Vec3::new(50.0 + at, 0.0, 0.0)));
        scene
    }

    /// The room's hash follows what is in the box: a recoloured or moved
    /// mesh in it changes it; the submission order and anything outside the
    /// box do not.
    #[test]
    fn the_hash_follows_the_rooms_content() {
        let probe = ReflectionProbe::new(1, Vec3::ZERO, Vec3::splat(10.0));
        let mut c = ProbeContent::default();
        let base = c.room(&room(0.8, 0.0, true), &probe);
        assert_eq!(base.meshes.len(), 2);
        assert_eq!(base.hash, c.room(&room(0.8, 0.0, false), &probe).hash);
        assert_ne!(base.hash, c.room(&room(0.3, 0.0, true), &probe).hash);
        assert_ne!(base.hash, c.room(&room(0.8, 2.0, true), &probe).hash);
        let mut far = room(0.8, 0.0, true);
        far.instances[2].transform = Mat4::from_translation(Vec3::new(80.0, 0.0, 0.0));
        assert_eq!(base.hash, c.room(&far, &probe).hash);
    }
}
