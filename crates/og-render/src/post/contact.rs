//! Screen-space contact shadows (contact.wgsl): Bend Studio's screen-space
//! shadows (Days Gone; Graham Aldridge, SIGGRAPH 2023;
//! `third_party/bend-sss`, Apache-2.0) as a compute pass, one dispatch list
//! per contact light (`frame::contact_lights`, up to [`CONTACT_LIGHTS`]),
//! then a compose of the AO and the lights' masks into the AO texture.
//!
//! Bend's `BuildDispatchList` (bend_sss_cpu.h) tiles the screen around the
//! light's pixel in up to eight dispatches of 64-pixel runs of the lines
//! through it, one 64-thread workgroup per run; `WriteScreenSpaceShadow`
//! (bend_sss_gpu.h) reads each run's depths once into workgroup memory and
//! walks them for every pixel of the run. The pass writes one screen of
//! keys per light into one buffer ([`mask_buffer`]), each thread's
//! `atomicMax` keeping the line nearest a pixel's centre where several
//! cross it; the compose reads them with the AO passes' target into `ao`.

use super::fsr::{self, B};
use crate::deferred::Deferred;
use crate::frame::{CONTACT_LIGHTS, CameraFrame};
use glam::Vec4;

const WGSL: &str = concat!(wgsl!("fxpost_uniform.wgsl"), wgsl!("contact.wgsl"));

/// The lights' masks for a screen of `size` pixels: one u32 key per pixel
/// and light row (contact.wgsl `masks`), zeroed. Four bytes a pixel, as
/// the rgba8unorm mask each light had; kept across frames (the keys carry
/// their frame), so it is the graph's own buffer, not a transient.
pub(crate) fn mask_buffer(device: &wgpu::Device, size: (u32, u32)) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("contact masks"),
        size: (CONTACT_LIGHTS as u64 * size.0 as u64 * size.1 as u64 * 4).max(4),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

/// The contact module's source, for validation (`shaders.rs`).
pub(crate) fn source() -> String {
    WGSL.to_string()
}

/// Bend's WAVE_SIZE: the threads of a workgroup, the pixels of a run.
pub(crate) const WAVE_SIZE: i32 = 64;
/// The most dispatches a light's list holds (four quadrants, each split
/// in two at most).
const MAX_DISPATCHES: usize = 8;

/// `struct ContactDispatch` in contact.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct DispatchUniform {
    light: [f32; 4],
    wave_offset: [i32; 2],
    index: u32,
    frame: u32,
}

/// One of Bend's `DispatchData`: its workgroups and its `WaveOffset`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Dispatch {
    pub workgroups: [u32; 3],
    pub wave_offset: [i32; 2],
}

/// Bend's `LightCoordinate` for a light at view-space `view` on a screen
/// of `size` pixels: its pixel coordinate (from `clip`, the projection
/// of `view`, with Bend's precision floor on w), its depth in the pass's
/// terms (1 / view depth, or `1 + z / far` orthographic: contact.wgsl
/// `contact_depth`), and +1 in front of the camera, -1 behind.
pub(crate) fn light_coordinate(
    clip: Vec4,
    view_z: f32,
    orthographic: bool,
    far: f32,
    size: (u32, u32),
) -> [f32; 4] {
    let fp_limit = 0.000002 * WAVE_SIZE as f32;
    let mut w = clip.w;
    if w >= 0.0 && w < fp_limit {
        w = fp_limit;
    } else if w < 0.0 && w > -fp_limit {
        w = -fp_limit;
    }
    let x = (clip.x / w * 0.5 + 0.5) * size.0 as f32;
    let y = (clip.y / w * -0.5 + 0.5) * size.1 as f32;
    let z = if orthographic {
        1.0 + view_z / far
    } else if clip.w != 0.0 {
        1.0 / clip.w
    } else {
        0.0
    };
    [x, y, z, if clip.w > 0.0 { 1.0 } else { -1.0 }]
}

/// Bend's `BuildDispatchList` for a light at pixel `light_xy` (its
/// [`light_coordinate`]) over the whole screen of `size` pixels: the
/// dispatches whose runs cover every pixel (the runs read and write up to
/// two runs past the screen, which the shader bounds).
pub(crate) fn dispatch_list(light_xy: [f32; 2], size: (u32, u32)) -> Vec<Dispatch> {
    let wave = WAVE_SIZE;
    let light = [(light_xy[0] + 0.5) as i32, (light_xy[1] + 0.5) as i32];
    let (w, h) = (size.0 as i32, size.1 as i32);
    // The bounds, inclusive, relative to the light.
    let biased = [-light[0], -(h - light[1]), w - light[0], -(0 - light[1])];
    let mut list = Vec::with_capacity(MAX_DISPATCHES);
    // The four quadrants around the light, each a rectangle with one
    // corner on it: 0 bottom left, 1 bottom right, 2 top left, 3 top right.
    for q in 0..4 {
        // Quadrants 0 and 3 take one more run vertically, 1 and 2 horizontally.
        let vertical = q == 0 || q == 3;
        let bounds = [
            0.max(if q & 1 != 0 { biased[0] } else { -biased[2] }) / wave,
            0.max(if q & 2 != 0 { biased[1] } else { -biased[3] }) / wave,
            0.max(
                (if q & 1 != 0 { biased[2] } else { -biased[0] })
                    + wave * if vertical { 1 } else { 2 }
                    - 1,
            ) / wave,
            0.max(
                (if q & 2 != 0 { biased[3] } else { -biased[1] })
                    + wave * if vertical { 2 } else { 1 }
                    - 1,
            ) / wave,
        ];
        if bounds[2] - bounds[0] <= 0 || bounds[3] - bounds[1] <= 0 {
            continue;
        }
        let bias_x = i32::from(q == 2 || q == 3);
        let bias_y = i32::from(q == 1 || q == 3);
        let mut disp = Dispatch {
            workgroups: [
                wave as u32,
                (bounds[2] - bounds[0]) as u32,
                (bounds[3] - bounds[1]) as u32,
            ],
            wave_offset: [
                (if q & 1 != 0 { bounds[0] } else { -bounds[2] }) + bias_x,
                (if q & 2 != 0 { -bounds[3] } else { bounds[1] }) + bias_y,
            ],
        };
        // The quadrant's far corner relative to the light: where its
        // diagonal meets the bounds, the rectangle splits in two along
        // the larger axis.
        let axis_delta = match q {
            0 => biased[0] - biased[1],
            1 => biased[2] + biased[1],
            2 => -biased[0] - biased[3],
            _ => -biased[2] + biased[3],
        };
        let axis_delta = (axis_delta + wave - 1) / wave;
        if axis_delta > 0 {
            let mut disp2 = disp;
            let (count, offset) = (&mut disp.workgroups, &mut disp.wave_offset);
            match q {
                0 => {
                    // Split on y; the split grows by one on x.
                    disp2.workgroups[2] = count[2].min(axis_delta as u32);
                    count[2] -= disp2.workgroups[2];
                    disp2.wave_offset[1] = offset[1] + count[2] as i32;
                    disp2.wave_offset[0] -= 1;
                    disp2.workgroups[1] += 1;
                }
                1 => {
                    // Split on x; the split grows by one on y.
                    disp2.workgroups[1] = count[1].min(axis_delta as u32);
                    count[1] -= disp2.workgroups[1];
                    disp2.wave_offset[0] = offset[0] + count[1] as i32;
                    disp2.workgroups[2] += 1;
                }
                2 => {
                    disp2.workgroups[1] = count[1].min(axis_delta as u32);
                    count[1] -= disp2.workgroups[1];
                    offset[0] += disp2.workgroups[1] as i32;
                    disp2.workgroups[2] += 1;
                    disp2.wave_offset[1] -= 1;
                }
                _ => {
                    disp2.workgroups[2] = count[2].min(axis_delta as u32);
                    count[2] -= disp2.workgroups[2];
                    offset[1] += disp2.workgroups[2] as i32;
                    disp2.workgroups[1] += 1;
                }
            }
            if disp.workgroups[1] > 0 && disp.workgroups[2] > 0 {
                list.push(disp);
            }
            if disp2.workgroups[1] > 0 && disp2.workgroups[2] > 0 {
                list.push(disp2);
            }
        } else {
            list.push(disp);
        }
    }
    for d in &mut list {
        d.wave_offset = d.wave_offset.map(|o| o * wave);
    }
    list
}

/// The pixel a thread of a dispatch writes (contact.wgsl `cs_contact`'s
/// `write_xy`, Bend's `ComputeWavefrontExtents`), for checking that the
/// dispatch list covers the screen.
#[cfg(test)]
fn thread_pixel(light: [f32; 4], d: Dispatch, group: [u32; 3], tid: u32) -> [i32; 2] {
    let wave = WAVE_SIZE;
    let mut xy = [
        group[1] as i32 * wave + d.wave_offset[0],
        group[2] as i32 * wave + d.wave_offset[1],
    ];
    let light_xy = [light[0].floor() + 0.5, light[1].floor() + 0.5];
    let fraction = [light[0] - light_xy[0], light[1] - light_xy[1]];
    let reverse = light[3] > 0.0;
    let sign = xy.map(i32::signum);
    let horizontal = (xy[0] + sign[1]).abs() < (xy[1] - sign[0]).abs();
    let axis = if horizontal {
        [sign[1], 0]
    } else {
        [0, -sign[0]]
    };
    xy = [
        axis[0] * group[0] as i32 + xy[0],
        axis[1] * group[0] as i32 + xy[1],
    ];
    let xy_f = [xy[0] as f32, xy[1] as f32];
    let x_major = xy_f[0].abs() > xy_f[1].abs();
    let major = if x_major { xy_f[0] } else { xy_f[1] };
    let start = major.abs();
    let end = major.abs() - wave as f32;
    let mut frac = if x_major { fraction[0] } else { fraction[1] };
    if major > 0.0 {
        frac = -frac;
    }
    let start_xy = [xy_f[0] + light_xy[0], xy_f[1] + light_xy[1]];
    let t = (end + frac) / (start + frac);
    let end_xy = [
        light[0] + (start_xy[0] - light[0]) * t,
        light[1] + (start_xy[1] - light[1]) * t,
    ];
    let step = (tid ^ if reverse { 0 } else { wave as u32 - 1 }) as f32 / wave as f32;
    [
        (start_xy[0] + (end_xy[0] - start_xy[0]) * step).floor() as i32,
        (start_xy[1] + (end_xy[1] - start_xy[1]) * step).floor() as i32,
    ]
}

/// The pass's pipelines and the per-dispatch uniforms of the frame.
pub(crate) struct ContactPasses {
    pub layout: wgpu::BindGroupLayout,
    pipeline: Deferred<wgpu::ComputePipeline>,
    pub compose_layout: wgpu::BindGroupLayout,
    compose: Deferred<wgpu::ComputePipeline>,
    /// Every dispatch's `ContactDispatch`, `stride` apart (the dynamic
    /// offsets' alignment).
    pub buffer: wgpu::Buffer,
    pub stride: u32,
    /// The frame's lights' dispatch lists (each dispatch's uniform names
    /// its light's row).
    lights: Vec<Vec<Dispatch>>,
    size: (u32, u32),
    /// The frame's tag in its keys (contact.wgsl `ContactDispatch`), 1 to
    /// 255; at 1 the masks are cleared first, so no key of a frame 255
    /// tags back outranks the frame's own.
    frame: u32,
}

impl ContactPasses {
    pub fn new(device: &wgpu::Device) -> ContactPasses {
        let stride = (std::mem::size_of::<DispatchUniform>() as u32)
            .next_multiple_of(device.limits().min_uniform_buffer_offset_alignment);
        let dispatch = wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: true,
                min_binding_size: wgpu::BufferSize::new(
                    std::mem::size_of::<DispatchUniform>() as u64
                ),
            },
            count: None,
        };
        let bindings =
            |label, numbered: &[(u32, B)], extra: Option<&wgpu::BindGroupLayoutEntry>| {
                let mut entries: Vec<_> = numbered
                    .iter()
                    .map(|&(binding, b)| wgpu::BindGroupLayoutEntry {
                        binding,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: fsr::binding_type(b),
                        count: None,
                    })
                    .collect();
                entries.extend(extra.cloned());
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some(label),
                    entries: &entries,
                })
            };
        let layout = bindings(
            "contact",
            &[
                (0, B::Uniform),
                (2, B::Tex(false)),
                (3, B::Tex(false)),
                (4, B::Rw),
            ],
            Some(&dispatch),
        );
        let compose_layout = bindings(
            "contact compose",
            &[
                (0, B::Uniform),
                (5, B::Tex(false)),
                (6, B::Ro),
                (9, B::Storage(super::CONTACT_FORMAT)),
            ],
            None,
        );
        let module = crate::deferred::module(device, "contact", WGSL.to_string());
        let pipeline =
            |label: &'static str, layout: &wgpu::BindGroupLayout, entry: &'static str| {
                let (device, layout, module) = (device.clone(), layout.clone(), module.clone());
                Deferred::new(move || {
                    let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some(label),
                        bind_group_layouts: &[Some(&layout)],
                        immediate_size: 0,
                    });
                    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                        label: Some(label),
                        layout: Some(&pl),
                        module: module.get(),
                        entry_point: Some(entry),
                        compilation_options: Default::default(),
                        cache: crate::pipeline_cache::of(&device).as_ref(),
                    })
                })
            };
        ContactPasses {
            pipeline: pipeline("contact", &layout, "cs_contact"),
            compose: pipeline("contact compose", &compose_layout, "cs_contact_compose"),
            layout,
            compose_layout,
            buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("contact dispatches"),
                size: (stride as usize * CONTACT_LIGHTS * MAX_DISPATCHES) as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            stride,
            lights: Vec::new(),
            size: (0, 0),
            frame: 0,
        }
    }

    /// Records the frame's lights (`rows`: `FxPostUniform::contact`, the
    /// view-space positions with w 1 drawn) for a screen of `size`
    /// pixels: their dispatch lists, and the uniforms the dispatches read.
    pub fn prepare(
        &mut self,
        queue: &wgpu::Queue,
        cam: &CameraFrame,
        far: f32,
        rows: &[[f32; 4]; 4],
        size: (u32, u32),
    ) {
        self.lights.clear();
        self.size = size;
        self.frame = self.frame % 255 + 1;
        let mut uniforms = Vec::with_capacity(CONTACT_LIGHTS * MAX_DISPATCHES);
        for (index, row) in rows.iter().take(CONTACT_LIGHTS).enumerate() {
            if row[3] < 0.5 {
                continue;
            }
            let view = Vec4::new(row[0], row[1], row[2], 1.0);
            let light = light_coordinate(cam.proj * view, row[2], cam.orthographic, far, size);
            let list = dispatch_list([light[0], light[1]], size);
            uniforms.extend(list.iter().map(|d| DispatchUniform {
                light,
                wave_offset: d.wave_offset,
                index: index as u32,
                frame: self.frame,
            }));
            self.lights.push(list);
        }
        if uniforms.is_empty() {
            return;
        }
        let mut bytes = vec![0u8; uniforms.len() * self.stride as usize];
        for (i, u) in uniforms.iter().enumerate() {
            let at = i * self.stride as usize;
            bytes[at..at + std::mem::size_of::<DispatchUniform>()]
                .copy_from_slice(bytemuck::bytes_of(u));
        }
        queue.write_buffer(&self.buffer, 0, &bytes);
    }

    /// Records the lights' dispatches (`lights`: their bind group, over
    /// `masks`, the [`mask_buffer`]) and the compose; returns the
    /// dispatches.
    pub fn encode(
        &self,
        encoder: &mut crate::Encoder,
        lights: &wgpu::BindGroup,
        compose: &wgpu::BindGroup,
        masks: &wgpu::Buffer,
    ) -> u32 {
        let mut dispatches = 0;
        if self.frame == 1 {
            encoder.clear_buffer(masks, 0, None);
        }
        if !self.lights.is_empty() {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("contact"),
                timestamp_writes: None,
            });
            pass.set_pipeline(self.pipeline.get());
            let mut slice = 0;
            for list in &self.lights {
                for d in list {
                    pass.set_bind_group(0, lights, &[slice * self.stride]);
                    pass.dispatch_workgroups(d.workgroups[0], d.workgroups[1], d.workgroups[2]);
                    slice += 1;
                    dispatches += 1;
                }
            }
        }
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("contact compose"),
            timestamp_writes: None,
        });
        pass.set_pipeline(self.compose.get());
        pass.set_bind_group(0, compose, &[]);
        pass.dispatch_workgroups(self.size.0.div_ceil(8), self.size.1.div_ceil(8), 1);
        dispatches + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// Every pixel of the screen lies in some run of the light's dispatch
    /// list, for lights on the screen, off it and behind the camera.
    #[test]
    fn dispatch_list_covers_the_screen() {
        for (size, light) in [
            ((640, 360), [320.3, 180.7, 0.5, 1.0]),
            ((640, 360), [3.0, 350.0, 0.5, 1.0]),
            ((645, 353), [-200.0, 100.0, 0.5, 1.0]),
            ((645, 353), [900.0, -50.0, 0.5, 1.0]),
            ((256, 256), [128.0, 128.0, 0.5, -1.0]),
            ((300, 200), [-1000.0, -1000.0, 0.5, -1.0]),
            ((300, 200), [150.5, 100.5, 0.5, 1.0]),
        ] {
            let list = dispatch_list([light[0], light[1]], size);
            assert!(list.len() <= MAX_DISPATCHES);
            let mut written = HashSet::new();
            for d in &list {
                for gy in 0..d.workgroups[1] {
                    for gz in 0..d.workgroups[2] {
                        for gx in 0..d.workgroups[0] {
                            for tid in 0..WAVE_SIZE as u32 {
                                written.insert(thread_pixel(light, *d, [gx, gy, gz], tid));
                            }
                        }
                    }
                }
            }
            // The light's own pixel is no run's (the runs end one pixel
            // short of it): the shader writes it apart.
            let own = [light[0].floor() as i32, light[1].floor() as i32];
            let missed: Vec<_> = (0..size.1 as i32)
                .flat_map(|y| (0..size.0 as i32).map(move |x| [x, y]))
                .filter(|p| *p != own && !written.contains(p))
                .take(8)
                .collect();
            assert!(
                missed.is_empty(),
                "{size:?} light {light:?}: {} dispatches miss {missed:?}",
                list.len()
            );
        }
    }

    /// A light's pixel and depth as the pass reads them: in front of the
    /// camera 1 / its view depth and +1; behind it -1.
    #[test]
    fn light_coordinate_follows_bend() {
        let size = (200, 100);
        let c = light_coordinate(Vec4::new(0.0, 0.0, 1.0, 2.0), -2.0, false, 100.0, size);
        assert_eq!(c, [100.0, 50.0, 0.5, 1.0]);
        let c = light_coordinate(Vec4::new(1.0, -1.0, 1.0, -2.0), 2.0, false, 100.0, size);
        assert_eq!(c, [50.0, 25.0, -0.5, -1.0]);
        let c = light_coordinate(Vec4::new(0.0, 0.0, 0.5, 1.0), -25.0, true, 100.0, size);
        assert_eq!(c[2], 0.75);
    }
}
