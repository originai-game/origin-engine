//! Drawing a terrain's levels.

use super::*;

/// Fills each terrain's corner cache for this frame (`terrain_corners.wgsl`):
/// every level's corners, before any pass draws them.
pub(crate) fn encode_corners(encoder: &mut crate::Encoder, draws: &[TerrainDraw]) {
    if draws.is_empty() {
        return;
    }
    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("terrain corners"),
        timestamp_writes: None,
    });
    for d in draws {
        pass.set_pipeline(d.passes.corners.get());
        pass.set_bind_group(0, &d.corners_write, &[]);
        for l in &d.levels {
            pass.set_bind_group(1, &d.group, &[l.offset]);
            pass.dispatch_workgroups(d.corner_count.div_ceil(64), 1, 1);
        }
    }
}

/// Draws the levels of `draws` that touch `frustum`, each level's blocks
/// culled by their boxes (group 1: the terrain;
/// group 0 bound by the caller, or per level the reflection probe's frame
/// group `probes` picks: probe_select.rs). Returns (draws, triangles).
pub(crate) fn draw_levels(
    pass: &mut wgpu::RenderPass,
    draws: &[TerrainDraw],
    indices: &wgpu::Buffer,
    which: TerrainPass,
    frustum: &crate::frame::Frustum,
    mut probes: Option<&mut crate::probe_select::Binder>,
) -> (u32, u64) {
    let mut calls = 0;
    let mut triangles = 0;
    for d in draws {
        if matches!(which, TerrainPass::Shadow(_)) && !d.cast_shadows {
            continue;
        }
        pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
        // Each terrain's own pipeline: the standard one, or the layer-array
        // variant's.
        pass.set_pipeline(match which {
            TerrainPass::Lit => d.passes.lit.get(d.features),
            TerrainPass::Prepass => d.passes.prepass.get(d.features),
            TerrainPass::Shadow(t) => d.passes.shadow.get().get(t),
        });
        pass.set_bind_group(2, &d.corners, &[]);
        for l in &d.levels {
            if !frustum.intersects_sphere(l.center, l.radius) {
                continue;
            }
            let runs = visible_ranges(&l.blocks, |lo, hi| frustum.intersects_box(lo, hi));
            if runs.is_empty() {
                continue;
            }
            if let Some(b) = probes.as_deref_mut() {
                b.at(pass, l.center, l.radius);
            }
            pass.set_bind_group(1, &d.group, &[l.offset]);
            for r in runs.into_iter().chain(l.skirt.clone()) {
                triangles += (r.len() / 3) as u64;
                pass.draw_indexed(r, 0, 0..1);
                calls += 1;
            }
        }
        // Its stamps, with its group (the level slot unread) and theirs.
        let Some(stamp_group) = &d.stamp_group else {
            continue;
        };
        let mut bound = false;
        for st in &d.stamps {
            if (matches!(which, TerrainPass::Shadow(_)) && !st.cast_shadows)
                || !frustum.intersects_sphere(st.center, st.radius)
            {
                continue;
            }
            if !bound {
                pass.set_pipeline(match which {
                    TerrainPass::Lit => d.passes.stamp_lit.get(d.features),
                    TerrainPass::Prepass => d.passes.stamp_prepass.get(d.features),
                    TerrainPass::Shadow(t) => d.passes.stamp_shadow.get().get(t),
                });
                pass.set_bind_group(1, &d.group, &[0]);
                bound = true;
            }
            if let Some(b) = probes.as_deref_mut() {
                b.at(pass, st.center, st.radius);
            }
            pass.set_bind_group(2, stamp_group, &[st.offset]);
            pass.set_vertex_buffer(0, st.vertices.slice(..));
            pass.set_index_buffer(st.indices.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..st.count, 0, 0..1);
            calls += 1;
            triangles += (st.count / 3) as u64;
        }
    }
    if let Some(b) = probes {
        b.reset(pass);
    }
    (calls, triangles)
}
