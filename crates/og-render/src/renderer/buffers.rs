//! Camera shake, sprite groups and buffer uploads.

use super::*;

/// The camera moved by a [`CameraShake`]: offset in view space (a fraction
/// of the view height at the target) and rolled about the view axis.
pub(crate) fn shaken_camera(camera: &Camera, shake: CameraShake) -> Camera {
    if shake == CameraShake::default() {
        return *camera;
    }
    let forward = (camera.target - camera.position).normalize_or(og_core::facing::CAMERA_FORWARD);
    let right = forward.cross(camera.up).normalize_or(Vec3::X);
    let up = right.cross(forward);
    let height = match camera.projection {
        Projection::Perspective { fov_y_degrees } => {
            2.0 * (fov_y_degrees.to_radians() * 0.5).tan()
                * (camera.target - camera.position).length()
        }
        Projection::Orthographic { height } => height,
    };
    let offset = (right * shake.offset.x + up * shake.offset.y) * height;
    let roll = glam::Quat::from_axis_angle(forward, shake.roll_degrees.to_radians());
    Camera {
        position: camera.position + offset,
        target: camera.target + offset,
        up: roll * camera.up,
        ..*camera
    }
}

pub(crate) fn bind_group(
    device: &wgpu::Device,
    label: &str,
    layout: &wgpu::BindGroupLayout,
    resources: &[wgpu::BindingResource],
) -> wgpu::BindGroup {
    let entries: Vec<_> = resources
        .iter()
        .enumerate()
        .map(|(i, r)| wgpu::BindGroupEntry {
            binding: i as u32,
            resource: r.clone(),
        })
        .collect();
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &entries,
    })
}

pub(crate) fn sprite_bind_group(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
    params: [f32; 4],
) -> wgpu::BindGroup {
    let buffer = crate::buffer_init(
        device,
        queue,
        &wgpu::util::BufferInitDescriptor {
            label: Some("sprite params"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM,
        },
    );
    bind_group(
        device,
        "sprite",
        layout,
        &[
            wgpu::BindingResource::TextureView(view),
            wgpu::BindingResource::Sampler(sampler),
            buffer.as_entire_binding(),
        ],
    )
}

/// The sprite-mode particles' instances: written in compute, read as
/// vertices (`size` bytes).
pub(crate) fn particle_sprite_buffer(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("particle sprites"),
        size,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    })
}
