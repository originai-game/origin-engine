//! The student and the decoder on wgpu compute: the shipping path, the
//! same functions as [`crate::cpu`] (its tests compare the two).
//!
//! [`Gpu::new`] takes the caller's device (og-render's, beside which the
//! model runs) and uploads the weights once, f32. Each network is one
//! compute pass and one readback; [`Gpu::student`] and [`Gpu::decoder`] are
//! async so the web never blocks on them (natively they poll the device).

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

use wgpu::util::DeviceExt;

use crate::bundle::{Bundle, BundleError, Config};
use crate::contract::{COUNTS, FRAMES, MAX_TOKENS, SLOTS};
use crate::cpu::{CONDITION_ROWS, SEQUENCE, argmax, check_config, tensor_shapes};
use crate::glue::{
    CODES, DecoderInputs, HEADS, LOCAL_FEATURES, SLOT_FEATURES, Statics, StudentInputs,
    StudentOutput, TARGET_FEATURES, decoder_inputs, finish, student_inputs,
};
use crate::{Generation, Inputs};

/// A buffer holding `desc.contents`, as `create_buffer_init` makes it; on
/// the web (fork s71, as og-render's `buffer_init`) created unmapped and
/// written with `queue.writeBuffer` straight from wasm memory, since
/// Chrome's mapped-at-creation path held its GPU process in long tasks.
fn buffer_init(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    desc: &wgpu::util::BufferInitDescriptor,
) -> wgpu::Buffer {
    #[cfg(target_arch = "wasm32")]
    if !desc.contents.is_empty() {
        let align = wgpu::COPY_BUFFER_ALIGNMENT as usize;
        let whole = desc.contents.len() / align * align;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: desc.label,
            size: desc.contents.len().div_ceil(align) as u64 * wgpu::COPY_BUFFER_ALIGNMENT,
            usage: desc.usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        if whole > 0 {
            queue.write_buffer(&buffer, 0, &desc.contents[..whole]);
        }
        if whole < desc.contents.len() {
            let mut tail = [0u8; wgpu::COPY_BUFFER_ALIGNMENT as usize];
            tail[..desc.contents.len() - whole].copy_from_slice(&desc.contents[whole..]);
            queue.write_buffer(&buffer, whole as u64, &tail);
        }
        return buffer;
    }
    let _ = queue;
    #[allow(clippy::disallowed_methods)]
    device.create_buffer_init(desc)
}

#[derive(Clone, Copy)]
enum Kernel {
    Linear,
    LinearSmall,
    LayerNorm,
    Attention,
    Assemble,
    SelectCount,
    Conv1d,
    Fuse,
    Transpose,
}

const KERNELS: [(Kernel, &str, &str); 9] = [
    (
        Kernel::Linear,
        "linear",
        include_str!("shaders/linear.wgsl"),
    ),
    (
        Kernel::LinearSmall,
        "linear_small",
        include_str!("shaders/linear_small.wgsl"),
    ),
    (
        Kernel::LayerNorm,
        "layer_norm",
        include_str!("shaders/layer_norm.wgsl"),
    ),
    (
        Kernel::Attention,
        "attention",
        include_str!("shaders/attention.wgsl"),
    ),
    (
        Kernel::Assemble,
        "assemble",
        include_str!("shaders/assemble.wgsl"),
    ),
    (
        Kernel::SelectCount,
        "select_count",
        include_str!("shaders/select_count.wgsl"),
    ),
    (
        Kernel::Conv1d,
        "conv1d",
        include_str!("shaders/conv1d.wgsl"),
    ),
    (Kernel::Fuse, "fuse", include_str!("shaders/fuse.wgsl")),
    (
        Kernel::Transpose,
        "transpose",
        include_str!("shaders/transpose.wgsl"),
    ),
];

/// Every kernel's WGSL, by name (for translation tests).
pub fn shader_sources() -> impl Iterator<Item = (&'static str, &'static str)> {
    KERNELS.iter().map(|(_, name, source)| (*name, *source))
}

/// Below this many rows a linear runs on the small kernel.
const SMALL_ROWS: usize = 512;

/// conv1d flags.
const RELU_IN: u32 = 1;
const MASK_IN: u32 = 2;
const UPSAMPLE: u32 = 4;
const RELU_OUT: u32 = 8;
const RESIDUAL: u32 = 16;
const MASK_OUT: u32 = 32;
/// linear flags.
const RELU: u32 = 1;
const ACCUMULATE: u32 = 2;

#[derive(Debug, Clone, PartialEq)]
pub enum GpuError {
    Bundle(BundleError),
    NoAdapter(String),
    NoDevice(String),
    Readback(String),
}

impl std::fmt::Display for GpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuError::Bundle(e) => write!(f, "{e}"),
            GpuError::NoAdapter(e) => write!(f, "no GPU adapter: {e}"),
            GpuError::NoDevice(e) => write!(f, "no GPU device: {e}"),
            GpuError::Readback(e) => write!(f, "readback failed: {e}"),
        }
    }
}

impl std::error::Error for GpuError {}

impl From<BundleError> for GpuError {
    fn from(e: BundleError) -> Self {
        GpuError::Bundle(e)
    }
}

/// One recorded dispatch.
struct Op {
    kernel: Kernel,
    group: wgpu::BindGroup,
    workgroups: (u32, u32),
    /// Its work in floating-point operations, estimated from its kernel's
    /// parameters: what a slice of a frame's budget is cut by.
    cost: f64,
}

/// A dispatch's fixed cost in FLOP at [`FIRST_RATE`] (0.4 ms): on the Mac
/// satellite's virtual M4 Pro a slice of 15 small dispatches took 7.8 ms
/// (median) however little work each had.
const DISPATCH: f64 = 4.0e8;

/// The floating-point operations of one dispatch of `kernel` with `params`
/// (its uniform, as recorded), plus [`DISPATCH`] for its fixed cost.
fn cost(kernel: Kernel, p: [u32; 8]) -> f64 {
    let v = p.map(f64::from);
    let work = match kernel {
        // rows, k, out
        Kernel::Linear | Kernel::LinearSmall => 2.0 * v[0] * v[1] * v[2],
        // rows, width
        Kernel::LayerNorm => 8.0 * v[0] * v[1],
        // n, heads, head width, length: scores and the weighted sum
        Kernel::Attention => 4.0 * v[0] * v[1] * v[3] * v[3] * v[2],
        // n, width, length
        Kernel::Assemble => v[0] * v[1] * v[2],
        // n, length, logits
        Kernel::SelectCount => v[0] * v[1] * v[2],
        // n, cin, cout, t, kernel
        Kernel::Conv1d => 2.0 * v[0] * v[1] * v[2] * v[3] * v[4],
        // n, width, positions, per
        Kernel::Fuse => 4.0 * v[0] * v[2] * (v[1] + 2.0 * v[3]),
        // n, a, b
        Kernel::Transpose => v[0] * v[1] * v[2],
    };
    work + DISPATCH
}

/// Records a network's dispatches, then runs them in one compute pass.
struct Recorder<'a> {
    gpu: &'a Gpu,
    ops: Vec<Op>,
}

fn groups_for(elements: usize, per_group: usize) -> (u32, u32) {
    let groups = elements.div_ceil(per_group).max(1);
    let x = groups.min(65535);
    (x as u32, groups.div_ceil(x) as u32)
}

/// `ops` cut into consecutive runs of at most `limit` estimated work each
/// (a run of one when a dispatch alone exceeds it); one run without a limit.
fn slices(ops: &[Op], limit: Option<f64>) -> Vec<std::ops::Range<usize>> {
    let Some(limit) = limit else {
        return std::iter::once(0..ops.len()).collect();
    };
    let (mut out, mut start, mut work) = (Vec::new(), 0, 0.0);
    for (i, op) in ops.iter().enumerate() {
        if i > start && work + op.cost > limit {
            out.push(start..i);
            (start, work) = (i, 0.0);
        }
        work += op.cost;
    }
    out.push(start..ops.len());
    out
}

impl Recorder<'_> {
    fn op(
        &mut self,
        kernel: Kernel,
        params: [u32; 8],
        buffers: &[&wgpu::Buffer],
        workgroups: (u32, u32),
    ) {
        let gpu = self.gpu;
        let uniform = buffer_init(
            &gpu.device,
            &gpu.queue,
            &wgpu::util::BufferInitDescriptor {
                label: Some("og-motion params"),
                contents: bytemuck::cast_slice(&params),
                usage: wgpu::BufferUsages::UNIFORM,
            },
        );
        let pipeline = &gpu.pipelines[kernel as usize];
        let mut entries = vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform.as_entire_binding(),
        }];
        for (i, b) in buffers.iter().enumerate() {
            entries.push(wgpu::BindGroupEntry {
                binding: i as u32 + 1,
                resource: b.as_entire_binding(),
            });
        }
        let group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("og-motion"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &entries,
        });
        self.ops.push(Op {
            kernel,
            group,
            workgroups,
            cost: cost(kernel, params),
        });
    }

    fn linear(
        &mut self,
        x: &wgpu::Buffer,
        rows: usize,
        w: &str,
        b: &str,
        y: &wgpu::Buffer,
        flags: u32,
    ) {
        let (wt, bt) = (&self.gpu.weights[w], &self.gpu.weights[b]);
        let (k, out) = (
            self.gpu.lengths[w] / self.gpu.lengths[b],
            self.gpu.lengths[b],
        );
        // Few rows leave 64-row tiles mostly idle: spread them over 16×16 tiles.
        let (kernel, tile) = if rows < SMALL_ROWS {
            (Kernel::LinearSmall, 16)
        } else {
            (Kernel::Linear, 64)
        };
        self.op(
            kernel,
            [rows as u32, k as u32, out as u32, flags, 0, 0, 0, 0],
            &[x, wt, bt, y],
            (out.div_ceil(tile) as u32, rows.div_ceil(tile) as u32),
        );
    }

    fn layer_norm(
        &mut self,
        x: &wgpu::Buffer,
        rows: usize,
        width: usize,
        prefix: &str,
        y: &wgpu::Buffer,
    ) {
        let (w, b) = (
            &self.gpu.weights[&format!("{prefix}weight")],
            &self.gpu.weights[&format!("{prefix}bias")],
        );
        self.op(
            Kernel::LayerNorm,
            [rows as u32, width as u32, 0, 0, 0, 0, 0, 0],
            &[x, w, b, y],
            groups_for(rows, 1),
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn conv(
        &mut self,
        x: &wgpu::Buffer,
        n: usize,
        t: usize,
        name: &str,
        dilation: usize,
        flags: u32,
        mask: &wgpu::Buffer,
        y: &wgpu::Buffer,
    ) {
        let shape = &self.gpu.shapes[&format!("{name}.weight")];
        let (cout, cin, kernel) = (shape[0], shape[1], shape[2]);
        let fpt = t / MAX_TOKENS;
        let (w, b) = (
            &self.gpu.weights[&format!("{name}.weight")],
            &self.gpu.weights[&format!("{name}.bias")],
        );
        self.op(
            Kernel::Conv1d,
            [
                n as u32,
                cin as u32,
                cout as u32,
                t as u32,
                kernel as u32,
                dilation as u32,
                flags,
                fpt as u32,
            ],
            &[x, w, b, mask, y],
            (cout.div_ceil(64) as u32, (n * t).div_ceil(64) as u32),
        );
    }

    fn transpose(&mut self, x: &wgpu::Buffer, n: usize, a: usize, b: usize, out: &wgpu::Buffer) {
        self.op(
            Kernel::Transpose,
            [n as u32, a as u32, b as u32, 0, 0, 0, 0, 0],
            &[x, out],
            groups_for(n * a * b, 256),
        );
    }

    fn plan(self, inputs: Vec<wgpu::Buffer>, outputs: Vec<wgpu::Buffer>) -> Plan {
        let timer = self.gpu.timer.as_ref().map_or(0, |t| t.resolve.size());
        let staging = self.gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("og-motion readback"),
            size: outputs.iter().map(|b| b.size()).sum::<u64>() + timer,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Plan {
            inputs,
            outputs,
            ops: self.ops,
            staging,
        }
    }
}

/// One network's dispatches for one batch size, with every buffer and bind
/// group they use: made once, then each call only writes the inputs.
struct Plan {
    inputs: Vec<wgpu::Buffer>,
    outputs: Vec<wgpu::Buffer>,
    ops: Vec<Op>,
    staging: wgpu::Buffer,
}

/// The most storage buffers one kernel binds (`assemble`): the device limit
/// [`Gpu::new`] needs.
pub const STORAGE_BUFFERS: u32 = 7;

/// Batch sizes whose plans are kept; past it the cache starts over.
const PLANS: usize = 8;

/// The slices of one plan that are timed; later ones run untimed.
const TIMED_SLICES: u32 = 64;

/// The device's rate before any slice was timed, and where the device has
/// no timestamp queries (a publish build, a virtual GPU): FLOP a
/// millisecond, 1 TFLOP/s, between the M4 Pro's 1.7 and the satellite's
/// 0.5 at large batches (`og-motion`'s README).
const FIRST_RATE: f64 = 1.0e9;

/// Timestamp queries around each slice's pass, where the device has them.
struct Timer {
    set: wgpu::QuerySet,
    resolve: wgpu::Buffer,
    /// Nanoseconds a tick.
    period: f64,
}

/// Pending once, then ready: a slice's turn ends, and the next poll (a
/// frame later, in a frame loop) submits the next one.
struct Yield(bool);

impl Future for Yield {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.0 {
            return Poll::Ready(());
        }
        self.0 = true;
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

#[derive(Default)]
struct MapState {
    result: Option<Result<(), wgpu::BufferAsyncError>>,
    waker: Option<Waker>,
}

/// Resolves when `map_async`'s callback has run. With a device, each poll
/// polls it once first, without waiting (a non-blocking [`Gpu`]: natively the
/// callback runs only inside a device poll).
struct Mapped(Arc<Mutex<MapState>>, Option<wgpu::Device>);

impl Future for Mapped {
    type Output = Result<(), wgpu::BufferAsyncError>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(device) = &self.1 {
            // An error here is a lost device: the map's callback reports it.
            let _ = device.poll(wgpu::PollType::Poll);
        }
        let mut state = self.0.lock().unwrap_or_else(|e| e.into_inner());
        match state.result.take() {
            Some(result) => Poll::Ready(result),
            None => {
                state.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

/// Both networks' weights on a device, with the compute pipelines.
pub struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: Config,
    pipelines: Vec<wgpu::ComputePipeline>,
    weights: HashMap<String, wgpu::Buffer>,
    lengths: HashMap<String, usize>,
    shapes: HashMap<String, Vec<usize>>,
    /// Pass 1's mask (its 9 rows: CLS and the slots) and pass 2's (26 rows).
    mask_count: wgpu::Buffer,
    mask_full: wgpu::Buffer,
    statics: Statics,
    students: HashMap<usize, Plan>,
    decoders: HashMap<usize, Plan>,
    /// Readbacks block in a device poll ([`Gpu::set_blocking`]).
    blocking: bool,
    /// The GPU time one submit may take, ms ([`Gpu::set_frame_budget`]).
    budget: Option<f64>,
    /// The device's measured rate, FLOP a millisecond.
    rate: f64,
    timer: Option<Timer>,
    /// Native wall time in pipeline creation and weight/mask buffer creation.
    /// Buffer creation is CPU upload submission, not a GPU completion fence.
    #[cfg(not(target_arch = "wasm32"))]
    pub startup_ms: (f64, f64),
}

impl Gpu {
    /// A device of its own on the first high-performance adapter (tests and tools).
    pub async fn request(bundle: &Bundle) -> Result<Gpu, GpuError> {
        Gpu::request_on(bundle, wgpu::Backends::PRIMARY).await
    }

    /// [`Gpu::request`] limited to `backends` (the bench's `--backend`).
    ///
    /// On Windows the DX12 shader compiler is the static DXC, as og-render's
    /// instance: without it wgpu falls back to FXC, which took 26-30 s to
    /// compile the kernels on an RTX 5060 against 0.4 s on Vulkan.
    ///
    /// `OG_RENDERER=none` answers [`GpuError::NoAdapter`] natively, as it makes
    /// og-render's renderers answer theirs (og-render's `RENDERER_SWITCH`):
    /// a machine told it has no GPU never gets a software adapter here.
    pub async fn request_on(bundle: &Bundle, backends: wgpu::Backends) -> Result<Gpu, GpuError> {
        #[cfg(not(target_arch = "wasm32"))]
        if std::env::var("OG_RENDERER").is_ok_and(|v| v.trim().eq_ignore_ascii_case("none")) {
            return Err(GpuError::NoAdapter("OG_RENDERER=none".into()));
        }
        let descriptor = wgpu::InstanceDescriptor {
            backends,
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        };
        #[cfg(all(target_os = "windows", target_arch = "x86_64", target_env = "msvc"))]
        let descriptor = {
            let mut descriptor = descriptor;
            descriptor.backend_options.dx12.shader_compiler = wgpu::Dx12Compiler::StaticDxc;
            descriptor
        };
        let instance = wgpu::Instance::new(descriptor);
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await
            .map_err(|e| GpuError::NoAdapter(e.to_string()))?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("og-motion"),
                ..Default::default()
            })
            .await
            .map_err(|e| GpuError::NoDevice(e.to_string()))?;
        Gpu::new(device, queue, bundle)
    }

    /// The device the model runs on.
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// How a generation's readbacks wait. Blocking (the default, for tools
    /// and tests driving a future with `pollster::block_on`): natively the
    /// device is polled until the readback lands. Not blocking (a frame loop
    /// on a device it shares with a renderer): the future never blocks, and
    /// each poll of it polls the device once, so polling it every frame
    /// completes it a frame or two after the GPU finishes. On the web the
    /// browser completes readbacks on its own and nothing blocks either way.
    pub fn set_blocking(&mut self, blocking: bool) {
        self.blocking = blocking;
    }

    /// Splits each network's dispatches into slices of about `ms` of GPU
    /// time, one submitted per poll of the generation (a frame loop's
    /// frame), so a batch never holds a frame past its budget. `None` (the
    /// default) submits each network at once. Slices are cut by each
    /// dispatch's estimated work and the device's rate, measured with
    /// timestamp queries where the device has them ([`Gpu::rate`]).
    pub fn set_frame_budget(&mut self, ms: Option<f64>) {
        self.budget = ms.filter(|ms| *ms > 0.0);
    }

    /// The device's rate as measured on the slices run so far, GFLOP/s
    /// (1000 before the first timed slice, or without timestamp queries).
    pub fn rate(&self) -> f64 {
        self.rate / 1.0e6
    }

    /// The model on `device` (one of its own, or a renderer's). Refused for
    /// a device whose limits the kernels exceed: a stage binds up to
    /// [`STORAGE_BUFFERS`] storage buffers.
    pub fn new(device: wgpu::Device, queue: wgpu::Queue, bundle: &Bundle) -> Result<Gpu, GpuError> {
        let config = bundle.config.clone();
        check_config(&config)?;
        let have = device.limits().max_storage_buffers_per_shader_stage;
        if have < STORAGE_BUFFERS {
            return Err(GpuError::NoDevice(format!(
                "the device allows {have} storage buffers a shader stage, and the kernels bind {STORAGE_BUFFERS}"
            )));
        }
        #[cfg(not(target_arch = "wasm32"))]
        let compile_start = std::time::Instant::now();
        let pipelines = KERNELS
            .iter()
            .map(|(_, name, source)| {
                // SAFETY: every loop in these kernels runs to a count read from its
                // uniform parameters, so each one ends; skipping wgpu's forced loop
                // bounding is what that requires. Bounds checks stay on. On Metal the
                // bounding counter in the inner loops cost 11× (2.5 TFLOP/s against
                // 0.23 for the linear kernel on the Mac satellite).
                let module = unsafe {
                    device.create_shader_module_trusted(
                        wgpu::ShaderModuleDescriptor {
                            label: Some(name),
                            source: wgpu::ShaderSource::Wgsl((*source).into()),
                        },
                        wgpu::ShaderRuntimeChecks {
                            force_loop_bounding: false,
                            ..wgpu::ShaderRuntimeChecks::checked()
                        },
                    )
                };
                device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some(name),
                    layout: None,
                    module: &module,
                    entry_point: Some("main"),
                    compilation_options: Default::default(),
                    cache: None,
                })
            })
            .collect();
        #[cfg(not(target_arch = "wasm32"))]
        let compile_ms = compile_start.elapsed().as_secs_f64() * 1000.0;
        #[cfg(not(target_arch = "wasm32"))]
        let upload_start = std::time::Instant::now();
        let (mut weights, mut lengths, mut shapes) =
            (HashMap::new(), HashMap::new(), HashMap::new());
        let width = config.student.width;
        for (name, shape) in tensor_shapes(&config) {
            let mut data = bundle.get(&name, &shape)?.to_vec();
            if name == "student.allowed_in.bias" {
                // CLS is a constant added to the allowed counts' projection: fold it in.
                let cls = bundle.get("student.cls", &[width])?;
                data.iter_mut().zip(cls).for_each(|(a, c)| *a += c);
            }
            let buffer = buffer_init(
                &device,
                &queue,
                &wgpu::util::BufferInitDescriptor {
                    label: Some(&name),
                    contents: bytemuck::cast_slice(&data),
                    usage: wgpu::BufferUsages::STORAGE,
                },
            );
            lengths.insert(name.clone(), data.len());
            shapes.insert(name.clone(), shape);
            weights.insert(name, buffer);
        }
        let trained = bundle.get("student.attn_mask", &[SEQUENCE, SEQUENCE])?;
        let full: Vec<u32> = trained.iter().map(|v| (*v > 0.5) as u32).collect();
        // Pass 1 runs CLS and the slots alone: the trained mask's rows and columns of them.
        let count: Vec<u32> = CONDITION_ROWS
            .iter()
            .flat_map(|&r| CONDITION_ROWS.iter().map(move |&c| (r, c)))
            .map(|(r, c)| full[r * SEQUENCE + c])
            .collect();
        let mask = |data: &[u32], label| {
            buffer_init(
                &device,
                &queue,
                &wgpu::util::BufferInitDescriptor {
                    label: Some(label),
                    contents: bytemuck::cast_slice(data),
                    usage: wgpu::BufferUsages::STORAGE,
                },
            )
        };
        let (mask_count, mask_full) = (mask(&count, "mask count"), mask(&full, "mask full"));
        #[cfg(not(target_arch = "wasm32"))]
        let upload_ms = upload_start.elapsed().as_secs_f64() * 1000.0;
        let timer = device
            .features()
            .contains(wgpu::Features::TIMESTAMP_QUERY)
            .then(|| Timer {
                set: device.create_query_set(&wgpu::QuerySetDescriptor {
                    label: Some("og-motion slices"),
                    ty: wgpu::QueryType::Timestamp,
                    count: 2 * TIMED_SLICES,
                }),
                resolve: device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("og-motion slice times"),
                    size: u64::from(2 * TIMED_SLICES) * 8,
                    usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                }),
                period: f64::from(queue.get_timestamp_period()),
            });
        Ok(Gpu {
            device,
            queue,
            config,
            pipelines,
            weights,
            lengths,
            shapes,
            mask_count,
            mask_full,
            statics: Statics::new(bundle)?,
            students: HashMap::new(),
            decoders: HashMap::new(),
            blocking: true,
            budget: None,
            rate: FIRST_RATE,
            timer,
            #[cfg(not(target_arch = "wasm32"))]
            startup_ms: (compile_ms, upload_ms),
        })
    }

    /// Generates motion for encoded inputs: [`crate::Model::generate`] on the GPU.
    pub async fn generate(&mut self, inputs: &Inputs) -> Result<Generation, GpuError> {
        let student = self.student(&student_inputs(&self.statics, inputs)).await?;
        let local = self
            .decoder(&decoder_inputs(&self.statics, inputs, &student))
            .await?;
        let motion = finish(&self.statics, inputs, &student, &local);
        Ok(Generation {
            counts: student.counts.clone(),
            motion,
            student,
        })
    }

    fn input(&self, floats: usize, label: &str) -> wgpu::Buffer {
        self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: (floats.max(1) * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    /// Runs `plan` with `inputs` written into its input buffers and returns
    /// its outputs as 32-bit words: in one submit, or with a frame budget
    /// in slices, yielding after each but the last.
    async fn run(&mut self, plan: &Plan, inputs: &[&[u8]]) -> Result<Vec<Vec<u32>>, GpuError> {
        for (buffer, data) in plan.inputs.iter().zip(inputs) {
            self.queue.write_buffer(buffer, 0, data);
        }
        let slices = slices(&plan.ops, self.budget.map(|ms| ms * self.rate));
        let timed = if self.budget.is_some() && self.timer.is_some() {
            slices.len().min(TIMED_SLICES as usize)
        } else {
            0
        };
        let last = slices.len() - 1;
        let mut encoder = self.encoder();
        for (i, range) in slices.iter().enumerate() {
            {
                let timestamp_writes = (i < timed).then(|| wgpu::ComputePassTimestampWrites {
                    query_set: &self.timer.as_ref().expect("timed above").set,
                    beginning_of_pass_write_index: Some(2 * i as u32),
                    end_of_pass_write_index: Some(2 * i as u32 + 1),
                });
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("og-motion"),
                    timestamp_writes,
                });
                for op in &plan.ops[range.clone()] {
                    pass.set_pipeline(&self.pipelines[op.kernel as usize]);
                    pass.set_bind_group(0, &op.group, &[]);
                    pass.dispatch_workgroups(op.workgroups.0, op.workgroups.1, 1);
                }
            }
            if i < last {
                self.queue.submit(Some(encoder.finish()));
                Yield(false).await;
                encoder = self.encoder();
            }
        }
        let mut sources: Vec<&wgpu::Buffer> = plan.outputs.iter().collect();
        if let Some(t) = &self.timer {
            if timed > 0 {
                encoder.resolve_query_set(&t.set, 0..2 * timed as u32, &t.resolve, 0);
            }
            sources.push(&t.resolve);
        }
        let mut out = self.read(encoder, &sources, &plan.staging).await?;
        if let Some(t) = &self.timer {
            let ticks = out.pop().expect("the timer's buffer was read");
            let (mut ms, mut work) = (0.0, 0.0);
            for (i, range) in slices.iter().take(timed).enumerate() {
                let at = |k: usize| u64::from(ticks[2 * k]) | (u64::from(ticks[2 * k + 1]) << 32);
                let (begin, end) = (at(2 * i), at(2 * i + 1));
                ms += end.saturating_sub(begin) as f64 * t.period / 1.0e6;
                work += plan.ops[range.clone()]
                    .iter()
                    .map(|op| op.cost)
                    .sum::<f64>();
            }
            if ms > 0.05 {
                self.rate = 0.5 * self.rate + 0.5 * (work / ms);
            }
        }
        Ok(out)
    }

    fn scratch(&self, floats: usize, label: &str) -> wgpu::Buffer {
        self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: (floats.max(1) * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        })
    }

    /// Submits `encoder` after copying `sources` into one staging buffer and
    /// returns their contents as 32-bit words.
    async fn read(
        &self,
        mut encoder: wgpu::CommandEncoder,
        sources: &[&wgpu::Buffer],
        staging: &wgpu::Buffer,
    ) -> Result<Vec<Vec<u32>>, GpuError> {
        let sizes: Vec<u64> = sources.iter().map(|b| b.size()).collect();
        let mut offset = 0;
        for (b, size) in sources.iter().zip(&sizes) {
            encoder.copy_buffer_to_buffer(b, 0, staging, offset, *size);
            offset += size;
        }
        self.queue.submit(Some(encoder.finish()));
        let state = Arc::new(Mutex::new(MapState::default()));
        let callback = state.clone();
        staging
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let mut s = callback.lock().unwrap_or_else(|e| e.into_inner());
                s.result = Some(result);
                if let Some(waker) = s.waker.take() {
                    waker.wake();
                }
            });
        #[cfg(not(target_arch = "wasm32"))]
        if self.blocking {
            self.device
                .poll(wgpu::PollType::wait_indefinitely())
                .map_err(|e| GpuError::Readback(e.to_string()))?;
        }
        let poll =
            (!self.blocking && cfg!(not(target_arch = "wasm32"))).then(|| self.device.clone());
        Mapped(state, poll)
            .await
            .map_err(|e| GpuError::Readback(e.to_string()))?;
        let words: Vec<u32> = {
            let view = staging
                .slice(..)
                .get_mapped_range()
                .map_err(|e| GpuError::Readback(e.to_string()))?;
            bytemuck::cast_slice::<u8, u32>(&view).to_vec()
        };
        staging.unmap();
        let mut out = Vec::with_capacity(sizes.len());
        let mut at = 0;
        for size in sizes {
            let len = size as usize / 4;
            out.push(words[at..at + len].to_vec());
            at += len;
        }
        Ok(out)
    }

    fn encoder(&self) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("og-motion"),
            })
    }

    /// The student: pass 1 picks the count on the GPU, pass 2 gives the root
    /// path and the token logits, whose argmax is taken on readback.
    pub async fn student(&mut self, inputs: &StudentInputs) -> Result<StudentOutput, GpuError> {
        let n = inputs.n;
        let plan = match self.students.remove(&n) {
            Some(plan) => plan,
            None => self.student_plan(n),
        };
        let read = self
            .run(
                &plan,
                &[
                    bytemuck::cast_slice(&inputs.slots),
                    bytemuck::cast_slice(&inputs.allowed),
                ],
            )
            .await;
        if self.students.len() >= PLANS {
            self.students.clear();
        }
        self.students.insert(n, plan);
        let read = read?;
        let root_all: &[f32] = bytemuck::cast_slice(&read[1]);
        let pose_all: &[f32] = bytemuck::cast_slice(&read[2]);
        let mut out = StudentOutput {
            counts: read[0].iter().map(|c| c + 6).collect(),
            root: vec![0.0; n * FRAMES * 5],
            tokens: vec![0; n * MAX_TOKENS * HEADS],
        };
        for i in 0..n {
            for q in 0..MAX_TOKENS {
                let row = i * SEQUENCE + 2 + SLOTS + q;
                out.root[(i * FRAMES + q * 4) * 5..(i * FRAMES + q * 4 + 4) * 5]
                    .copy_from_slice(&root_all[row * 20..row * 20 + 20]);
                for head in 0..HEADS {
                    let at = (row * HEADS + head) * CODES;
                    out.tokens[(i * MAX_TOKENS + q) * HEADS + head] =
                        argmax(pose_all[at..at + CODES].iter().copied()) as u32;
                }
            }
        }
        Ok(out)
    }

    fn student_plan(&self, n: usize) -> Plan {
        let s = &self.config.student;
        let (width, rows) = (s.width, n * SEQUENCE);
        let slots = self.input(n * SLOTS * SLOT_FEATURES, "slots");
        let allowed = self.input(n * COUNTS, "allowed");
        let count = self.scratch(n, "count");
        let (slot_tokens, cls) = (
            self.scratch(n * SLOTS * width, "slot tokens"),
            self.scratch(n * width, "cls"),
        );
        let (x, h) = (
            self.scratch(rows * width, "x"),
            self.scratch(rows * width, "h"),
        );
        let qkv = self.scratch(rows * 3 * width, "qkv");
        let attended = self.scratch(rows * width, "attended");
        let hidden = self.scratch(rows * s.ff * width, "hidden");
        let logits = self.scratch(rows * (COUNTS + 1), "count logits");
        let root = self.scratch(rows * 20, "root");
        let pose = self.scratch(rows * HEADS * CODES, "pose");
        let w = &self.weights;

        let mut r = Recorder {
            gpu: self,
            ops: Vec::new(),
        };
        debug_assert_eq!(
            self.lengths["student.slot_in.weight"],
            width * SLOT_FEATURES
        );
        r.linear(
            &slots,
            n * SLOTS,
            "student.slot_in.weight",
            "student.slot_in.bias",
            &slot_tokens,
            0,
        );
        r.linear(
            &allowed,
            n,
            "student.allowed_in.weight",
            "student.allowed_in.bias",
            &cls,
            0,
        );
        for pass in 0..2 {
            let length = if pass == 0 {
                CONDITION_ROWS.len()
            } else {
                SEQUENCE
            };
            let rows = n * length;
            r.op(
                Kernel::Assemble,
                [n as u32, width as u32, length as u32, 0, 0, 0, 0, 0],
                &[
                    &cls,
                    &w["student.count_emb.weight"],
                    &count,
                    &slot_tokens,
                    &w["student.slot_pos"],
                    &w["student.queries"],
                    &x,
                ],
                groups_for(rows * width, 256),
            );
            let mask = if pass == 0 {
                &self.mask_count
            } else {
                &self.mask_full
            };
            for l in 0..s.layers {
                let p = format!("student.encoder.layers.{l}.");
                r.layer_norm(&x, rows, width, &format!("{p}norm1."), &h);
                r.linear(
                    &h,
                    rows,
                    &format!("{p}self_attn.in_proj_weight"),
                    &format!("{p}self_attn.in_proj_bias"),
                    &qkv,
                    0,
                );
                r.op(
                    Kernel::Attention,
                    [
                        n as u32,
                        s.heads as u32,
                        (width / s.heads) as u32,
                        length as u32,
                        0,
                        0,
                        0,
                        0,
                    ],
                    &[&qkv, mask, &attended],
                    groups_for(n * s.heads * length, 1),
                );
                r.linear(
                    &attended,
                    rows,
                    &format!("{p}self_attn.out_proj.weight"),
                    &format!("{p}self_attn.out_proj.bias"),
                    &x,
                    ACCUMULATE,
                );
                r.layer_norm(&x, rows, width, &format!("{p}norm2."), &h);
                r.linear(
                    &h,
                    rows,
                    &format!("{p}linear1.weight"),
                    &format!("{p}linear1.bias"),
                    &hidden,
                    RELU,
                );
                r.linear(
                    &hidden,
                    rows,
                    &format!("{p}linear2.weight"),
                    &format!("{p}linear2.bias"),
                    &x,
                    ACCUMULATE,
                );
            }
            r.layer_norm(&x, rows, width, "student.norm.", &h);
            if pass == 0 {
                r.linear(
                    &h,
                    rows,
                    "student.count_out.weight",
                    "student.count_out.bias",
                    &logits,
                    0,
                );
                r.op(
                    Kernel::SelectCount,
                    [
                        n as u32,
                        length as u32,
                        (COUNTS + 1) as u32,
                        COUNTS as u32,
                        0,
                        0,
                        0,
                        0,
                    ],
                    &[&logits, &allowed, &count],
                    (n.div_ceil(64) as u32, 1),
                );
            } else {
                r.linear(
                    &h,
                    rows,
                    "student.root_out.weight",
                    "student.root_out.bias",
                    &root,
                    0,
                );
                r.linear(
                    &h,
                    rows,
                    "student.pose_out.weight",
                    "student.pose_out.bias",
                    &pose,
                    0,
                );
            }
        }
        r.plan(vec![slots, allowed], vec![count, root, pose])
    }

    /// The decoder: `[n,64,365]` normalised local motion features.
    pub async fn decoder(&mut self, inputs: &DecoderInputs) -> Result<Vec<f32>, GpuError> {
        let n = inputs.n;
        let plan = match self.decoders.remove(&n) {
            Some(plan) => plan,
            None => self.decoder_plan(n),
        };
        let read = self
            .run(
                &plan,
                &[
                    bytemuck::cast_slice(&inputs.codes),
                    bytemuck::cast_slice(&inputs.token_mask),
                    bytemuck::cast_slice(&inputs.external),
                    bytemuck::cast_slice(&inputs.target),
                    bytemuck::cast_slice(&inputs.has_target),
                ],
            )
            .await;
        if self.decoders.len() >= PLANS {
            self.decoders.clear();
        }
        self.decoders.insert(n, plan);
        Ok(bytemuck::cast_slice(&read?[0]).to_vec())
    }

    fn decoder_plan(&self, n: usize) -> Plan {
        let d = &self.config.decoder;
        let (width, down) = (d.width, d.down_t);
        let codes = self.input(n * HEADS * 32 * MAX_TOKENS, "codes");
        let mask = self.input(n * MAX_TOKENS, "token mask");
        let external = self.input(n * FRAMES * 2, "external");
        let target = self.input(n * FRAMES * TARGET_FEATURES, "target");
        let has = self.input(n * FRAMES, "has target");
        let mut h = self.scratch(n * width * FRAMES, "h");
        let mut y = self.scratch(n * width * FRAMES, "y");
        let ht = self.scratch(n * FRAMES * width / 2, "ht");
        let fused = self.scratch(n * FRAMES * (width + 2 * (1 << down)), "fused");
        let rows = self.scratch(n * FRAMES * width, "rows");
        let local = self.scratch(n * LOCAL_FEATURES * FRAMES, "local");
        let out = self.scratch(n * FRAMES * LOCAL_FEATURES, "out");

        let mut r = Recorder {
            gpu: self,
            ops: Vec::new(),
        };
        r.conv(
            &codes,
            n,
            MAX_TOKENS,
            "decoder.model.0",
            1,
            MASK_IN | RELU_OUT,
            &mask,
            &h,
        );
        for level in 0..down {
            let per = 1 << (down - level);
            let positions = FRAMES / per;
            debug_assert_eq!(TARGET_FEATURES, d.target_cond_dim);
            r.linear(
                &target,
                n * FRAMES,
                &format!("decoder.target_cond_blocks.{}.weight", 2 * level),
                &format!("decoder.target_cond_blocks.{}.bias", 2 * level),
                &ht,
                RELU,
            );
            r.op(
                Kernel::Fuse,
                [
                    n as u32,
                    width as u32,
                    positions as u32,
                    per as u32,
                    0,
                    0,
                    0,
                    0,
                ],
                &[&h, &ht, &has, &external, &fused],
                groups_for(n * positions * (width + 2 * per), 256),
            );
            r.linear(
                &fused,
                n * positions,
                &format!("decoder.external_cond_blocks.{}.weight", 2 * level),
                &format!("decoder.external_cond_blocks.{}.bias", 2 * level),
                &rows,
                RELU,
            );
            r.transpose(&rows, n, positions, width, &h);
            for b in 0..d.depth {
                let p = format!("decoder.model.{}.0.model.{b}.", level + 2);
                let dilation = d.dilation_growth_rate.pow((d.depth - 1 - b) as u32);
                r.conv(
                    &h,
                    n,
                    positions,
                    &format!("{p}conv1"),
                    dilation,
                    RELU_IN | MASK_IN | RELU_OUT | MASK_OUT,
                    &mask,
                    &y,
                );
                r.conv(
                    &y,
                    n,
                    positions,
                    &format!("{p}conv2"),
                    1,
                    RESIDUAL | MASK_OUT,
                    &mask,
                    &h,
                );
            }
            r.conv(
                &h,
                n,
                positions * 2,
                &format!("decoder.model.{}.2", level + 2),
                1,
                UPSAMPLE | MASK_IN,
                &mask,
                &y,
            );
            // The upsampled level is in y: it is the next level's h.
            std::mem::swap(&mut h, &mut y);
        }
        let tail = self.scratch(n * width * FRAMES, "tail");
        r.conv(
            &h,
            n,
            FRAMES,
            &format!("decoder.model.{}", 2 + down),
            1,
            MASK_IN | RELU_OUT | MASK_OUT,
            &mask,
            &tail,
        );
        r.conv(
            &tail,
            n,
            FRAMES,
            &format!("decoder.model.{}", 4 + down),
            1,
            MASK_OUT,
            &mask,
            &local,
        );
        r.transpose(&local, n, LOCAL_FEATURES, FRAMES, &out);
        r.plan(vec![codes, mask, external, target, has], vec![out])
    }
}
