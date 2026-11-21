use super::*;
use crate::ViewRect;
use wgpu::util::DeviceExt;

/// The impostors' card rectangles as one table (the scatter group's
/// binding 5, impostor_common.wgsl `imp_views`): a row per impostor
/// material with them ([`crate::RenderScene::set_impostor_views`]), a
/// texel per cell of views ([`crate::impostors::view_cell`]). A texture, read by `textureLoad` in
/// the vertex stage: a storage buffer there needs `VERTEX_STORAGE`.
pub(crate) struct ImpostorFrames {
    pub view: wgpu::TextureView,
    /// Per row: the material's slot and its boxes.
    rows: Vec<(usize, Arc<[ViewRect]>)>,
}

impl ImpostorFrames {
    pub(super) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        rows: &[(usize, Arc<[ViewRect]>)],
    ) -> ImpostorFrames {
        let width = rows.iter().map(|r| r.1.len()).max().unwrap_or(1).max(1);
        let height = rows.len().max(1);
        let mut texels = vec![[0.0f32; 4]; width * height];
        for (k, (_, frames)) in rows.iter().enumerate() {
            for (c, r) in frames.iter().enumerate() {
                texels[k * width + c] = *r;
            }
        }
        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("impostor card rectangles"),
                size: wgpu::Extent3d {
                    width: width as u32,
                    height: height as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            bytemuck::cast_slice(&texels),
        );
        ImpostorFrames {
            view: texture.create_view(&Default::default()),
            rows: rows.to_vec(),
        }
    }

    /// The table made again with the rectangles of the impostors `scene` draws
    /// when one of them is not in it: true then (the scatter group binds
    /// the new one).
    pub(super) fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &crate::RenderScene,
    ) -> bool {
        let mut wanted: Vec<(usize, Arc<[ViewRect]>)> = Vec::new();
        let impostors = scene
            .world
            .scatters
            .iter()
            .filter_map(|d| d.impostor)
            .chain(scene.objects.impostors.iter().map(|d| d.impostor));
        for i in impostors {
            if let Some(f) = scene.impostor_views(i.material)
                && !wanted.iter().any(|w| w.0 == i.material.index())
            {
                wanted.push((i.material.index(), f.clone()));
            }
        }
        let held = |w: &(usize, Arc<[ViewRect]>)| {
            self.rows
                .iter()
                .any(|r| r.0 == w.0 && Arc::ptr_eq(&r.1, &w.1))
        };
        if wanted.iter().all(held) {
            return false;
        }
        *self = ImpostorFrames::new(device, queue, &wanted);
        true
    }

    /// `material`'s row plus one (0: none, the card keeps its
    /// outline), as `ScatterUniform.impostor_trim[1].w` holds it.
    pub(super) fn row(&self, scene: &crate::RenderScene, material: MaterialHandle) -> f32 {
        let Some(f) = scene.impostor_views(material) else {
            return 0.0;
        };
        self.rows
            .iter()
            .position(|r| r.0 == material.index() && Arc::ptr_eq(&r.1, f))
            .map_or(0.0, |k| (k + 1) as f32)
    }
}
