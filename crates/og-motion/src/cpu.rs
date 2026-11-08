//! The student and the decoder on the CPU, in f32 over PyTorch weight
//! layouts: the engine's path (docs/design.md §2.7) and the one every other
//! path is checked against.
//!
//! Every value is the same on every host: each sum runs in one fixed order
//! ([`dot`]: eight lanes, each in increasing index, then a fixed tree), a
//! multiply and an add are never fused (Rust does not contract them, and
//! wasm has no fused operation outside relaxed SIMD), and the one
//! transcendental is og-core's `exp`. A character's generation is a
//! [`Job`] of [`Cpu::units`] units of about equal cost (a half encoder
//! layer, a decoder stage), so a host spreads it over ticks and threads
//! without changing a bit of it.

use og_core::math::exp;

use crate::bundle::Bundle;
use crate::bundle::{BundleError, Config};
use crate::contract::{COUNTS, FRAMES, Inputs, MAX_TOKENS, SLOTS};
use crate::glue::{CODES, HEADS, LOCAL_FEATURES, SLOT_FEATURES, TARGET_FEATURES};
use crate::glue::{DecoderInputs, Statics, StudentInputs, StudentOutput};

/// Lanes of [`dot`]'s accumulator.
const LANES: usize = 8;

/// `Σ a[j]·b[j]` in one fixed order on every target: lane `j mod 8`
/// accumulates its products in increasing `j` (a separate multiply and
/// add), then the lanes are summed in a fixed tree. The lanes are
/// independent, so the compiler vectorises them without reordering.
#[inline(always)]
fn dot(a: &[f32], b: &[f32]) -> f32 {
    let mut acc = [0.0f32; LANES];
    let (ac, at) = a.as_chunks::<LANES>();
    let (bc, bt) = b.as_chunks::<LANES>();
    for (x, y) in ac.iter().zip(bc) {
        for l in 0..LANES {
            acc[l] += x[l] * y[l];
        }
    }
    for (l, (x, y)) in at.iter().zip(bt).enumerate() {
        acc[l] += x * y;
    }
    fold(acc)
}

#[inline(always)]
fn fold(a: [f32; LANES]) -> f32 {
    ((a[0] + a[4]) + (a[2] + a[6])) + ((a[1] + a[5]) + (a[3] + a[7]))
}

/// Rows of `x` and of the weights a [`block`] takes at a time: eight sums
/// of two vectors each, sixteen accumulators, in the registers of every
/// target (NEON's 32, simd128's and SSE's 16 with their loads).
const BLOCK_ROWS: usize = 2;
const BLOCK_OUTPUTS: usize = 4;

/// [`dot`] of `R` rows of `x` with `O` rows of `w` (all of one length),
/// each sum in [`dot`]'s order, every chunk loaded once for all the sums
/// it is in: the multiplies and adds are lane for lane those of [`dot`],
/// so the target's vectors change no bit.
#[cfg(target_arch = "aarch64")]
#[inline(always)]
fn block<const R: usize, const O: usize>(x: [&[f32]; R], w: [&[f32]; O]) -> [[f32; O]; R] {
    use std::arch::aarch64::{
        float32x4_t, vaddq_f32, vdupq_n_f32, vld1q_f32, vmulq_f32, vst1q_f32,
    };
    let k = w[0].len();
    let n = k / LANES;
    assert!(x.iter().chain(&w).all(|r| r.len() == k));
    let mut acc = [[[0.0f32; LANES]; O]; R];
    // SAFETY: NEON is in every aarch64 target; every load is of a chunk
    // `j < n` of a row of `k >= n·LANES` floats, checked above.
    unsafe {
        let mut lo = [[vdupq_n_f32(0.0); O]; R];
        let mut hi = lo;
        let (xp, wp) = (x.map(<[f32]>::as_ptr), w.map(<[f32]>::as_ptr));
        for j in 0..n {
            let wl: [float32x4_t; O] = std::array::from_fn(|o| vld1q_f32(wp[o].add(j * LANES)));
            let wh: [float32x4_t; O] = std::array::from_fn(|o| vld1q_f32(wp[o].add(j * LANES + 4)));
            for r in 0..R {
                let xl = vld1q_f32(xp[r].add(j * LANES));
                let xh = vld1q_f32(xp[r].add(j * LANES + 4));
                for o in 0..O {
                    lo[r][o] = vaddq_f32(lo[r][o], vmulq_f32(xl, wl[o]));
                    hi[r][o] = vaddq_f32(hi[r][o], vmulq_f32(xh, wh[o]));
                }
            }
        }
        for r in 0..R {
            for o in 0..O {
                vst1q_f32(acc[r][o].as_mut_ptr(), lo[r][o]);
                vst1q_f32(acc[r][o].as_mut_ptr().add(4), hi[r][o]);
            }
        }
    }
    finish(x, w, n * LANES, acc)
}

/// [`block`] in wasm's simd128 (the engine's wasm is built without it;
/// every browser with WebGPU has it).
#[cfg(target_arch = "wasm32")]
#[target_feature(enable = "simd128")]
fn block<const R: usize, const O: usize>(x: [&[f32]; R], w: [&[f32]; O]) -> [[f32; O]; R] {
    use std::arch::wasm32::{f32x4_add, f32x4_mul, f32x4_splat, v128, v128_load, v128_store};
    let k = w[0].len();
    let n = k / LANES;
    assert!(x.iter().chain(&w).all(|r| r.len() == k));
    let mut acc = [[[0.0f32; LANES]; O]; R];
    // SAFETY: every load is of a chunk `j < n` of a row of `k >= n·LANES`
    // floats, checked above; v128_load and v128_store take any alignment.
    unsafe {
        let mut lo = [[f32x4_splat(0.0); O]; R];
        let mut hi = lo;
        let xp = x.map(|r| r.as_ptr().cast::<v128>());
        let wp = w.map(|r| r.as_ptr().cast::<v128>());
        for j in 0..n {
            let wl: [v128; O] = std::array::from_fn(|o| v128_load(wp[o].add(2 * j)));
            let wh: [v128; O] = std::array::from_fn(|o| v128_load(wp[o].add(2 * j + 1)));
            for r in 0..R {
                let xl = v128_load(xp[r].add(2 * j));
                let xh = v128_load(xp[r].add(2 * j + 1));
                for o in 0..O {
                    lo[r][o] = f32x4_add(lo[r][o], f32x4_mul(xl, wl[o]));
                    hi[r][o] = f32x4_add(hi[r][o], f32x4_mul(xh, wh[o]));
                }
            }
        }
        for r in 0..R {
            for o in 0..O {
                let at = acc[r][o].as_mut_ptr().cast::<v128>();
                v128_store(at, lo[r][o]);
                v128_store(at.add(1), hi[r][o]);
            }
        }
    }
    finish(x, w, n * LANES, acc)
}

/// [`block`] in portable Rust, which LLVM vectorises (x86's SSE).
#[cfg(not(any(target_arch = "aarch64", target_arch = "wasm32")))]
#[inline(always)]
fn block<const R: usize, const O: usize>(x: [&[f32]; R], w: [&[f32]; O]) -> [[f32; O]; R] {
    let k = w[0].len();
    let n = k / LANES;
    let xc = x.map(|r| &r.as_chunks::<LANES>().0[..n]);
    let wc = w.map(|r| &r.as_chunks::<LANES>().0[..n]);
    let mut acc = [[[0.0f32; LANES]; O]; R];
    for j in 0..n {
        let ws: [[f32; LANES]; O] = std::array::from_fn(|o| wc[o][j]);
        for r in 0..R {
            let xs = xc[r][j];
            for o in 0..O {
                for l in 0..LANES {
                    acc[r][o][l] += xs[l] * ws[o][l];
                }
            }
        }
    }
    finish(x, w, n * LANES, acc)
}

/// A [`block`]'s end: the products past the last whole chunk into lanes
/// 0.., as [`dot`] adds them, then each sum's [`fold`].
#[inline(always)]
fn finish<const R: usize, const O: usize>(
    x: [&[f32]; R],
    w: [&[f32]; O],
    whole: usize,
    mut acc: [[[f32; LANES]; O]; R],
) -> [[f32; O]; R] {
    for j in whole..w[0].len() {
        for r in 0..R {
            for o in 0..O {
                acc[r][o][j - whole] += x[r][j] * w[o][j];
            }
        }
    }
    acc.map(|r| r.map(fold))
}

/// `y[r,o] = b[o] + Σk x[r,k]·w[o,k]` for `rows` rows, each sum a [`dot`],
/// in [`block`]s of [`BLOCK_ROWS`] rows by [`BLOCK_OUTPUTS`] outputs (and
/// single rows and outputs at the edges); a row's values never depend on
/// the rows beside it.
pub(crate) fn linear(x: &[f32], rows: usize, w: &[f32], b: &[f32]) -> Vec<f32> {
    let out = b.len();
    let k = w.len() / out;
    debug_assert_eq!(x.len(), rows * k);
    let mut y = vec![0.0; rows * out];
    let row = |r: usize| &x[r * k..(r + 1) * k];
    let weights = |o: usize| &w[o * k..(o + 1) * k];
    let mut put = |r: usize, o: usize, v: f32| y[r * out + o] = b[o] + v;
    let mut o = 0;
    while o < out {
        let outputs = if o + BLOCK_OUTPUTS <= out {
            BLOCK_OUTPUTS
        } else {
            1
        };
        let mut r = 0;
        while r < rows {
            let rs = if r + BLOCK_ROWS <= rows {
                BLOCK_ROWS
            } else {
                1
            };
            macro_rules! run {
                ($rs:literal, $os:literal) => {{
                    // SAFETY: `block` enables simd128 on wasm, which every
                    // browser the engine runs in has; elsewhere it is safe.
                    #[allow(unused_unsafe)]
                    let s = unsafe {
                        block::<$rs, $os>(
                            std::array::from_fn(|i| row(r + i)),
                            std::array::from_fn(|i| weights(o + i)),
                        )
                    };
                    for (i, v) in s.iter().enumerate() {
                        for (j, v) in v.iter().enumerate() {
                            put(r + i, o + j, *v);
                        }
                    }
                }};
            }
            match (rs, outputs) {
                (BLOCK_ROWS, BLOCK_OUTPUTS) => run!(2, 4),
                (BLOCK_ROWS, _) => run!(2, 1),
                (_, BLOCK_OUTPUTS) => run!(1, 4),
                _ => run!(1, 1),
            }
            r += rs;
        }
        o += outputs;
    }
    y
}

fn layer_norm(x: &mut [f32], width: usize, w: &[f32], b: &[f32]) {
    for row in x.chunks_exact_mut(width) {
        let mean = row.iter().sum::<f32>() / width as f32;
        let var = row.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / width as f32;
        let inv = 1.0 / (var + 1e-5).sqrt();
        for (i, v) in row.iter_mut().enumerate() {
            *v = (*v - mean) * inv * w[i] + b[i];
        }
    }
}

fn relu(x: &mut [f32]) {
    for v in x {
        *v = v.max(0.0);
    }
}

/// `x [C,T]`, `w [O,C,K]`, 'same' padding (`dilation · (K/2)` zeros each
/// side): each position's taps gathered in `w`'s order (channel, then tap),
/// then a [`linear`] over the positions, transposed back to `[O,T]`.
fn conv1d(x: &[f32], t: usize, w: &[f32], b: &[f32], kernel: usize, dilation: usize) -> Vec<f32> {
    let out = b.len();
    let channels = x.len() / t;
    let pad = (dilation * (kernel / 2)) as isize;
    let taps = channels * kernel;
    let mut cols = vec![0.0; t * taps];
    for s in 0..t {
        let row = &mut cols[s * taps..(s + 1) * taps];
        for c in 0..channels {
            for k in 0..kernel {
                let at = s as isize + (k * dilation) as isize - pad;
                if at >= 0 && (at as usize) < t {
                    row[c * kernel + k] = x[c * t + at as usize];
                }
            }
        }
    }
    let ty = linear(&cols, t, w, b);
    let mut y = vec![0.0; out * t];
    for s in 0..t {
        for o in 0..out {
            y[o * t + s] = ty[s * out + o];
        }
    }
    y
}

struct EncoderLayer {
    norm1: (Vec<f32>, Vec<f32>),
    in_proj: (Vec<f32>, Vec<f32>),
    out_proj: (Vec<f32>, Vec<f32>),
    norm2: (Vec<f32>, Vec<f32>),
    linear1: (Vec<f32>, Vec<f32>),
    linear2: (Vec<f32>, Vec<f32>),
}

/// Both networks' weights, f32, PyTorch layouts.
pub struct Cpu {
    config: Config,
    get: std::collections::HashMap<String, Vec<f32>>,
    layers: Vec<EncoderLayer>,
    /// `[26·26]`, true where attention is not allowed.
    mask: Vec<bool>,
}

/// The rows of pass 2 that pass 1 has not computed: the count and the
/// queries. The condition rows read none of them (the mask), so pass 2's
/// condition rows are pass 1's, bit for bit: pass 2 reads their keys and
/// values from pass 1's layers.
const PASS2_ROWS: [usize; 1 + MAX_TOKENS] = {
    let mut rows = [1; 1 + MAX_TOKENS];
    let mut i = 0;
    while i < MAX_TOKENS {
        rows[1 + i] = 2 + SLOTS + i;
        i += 1;
    }
    rows
};

/// One character's generation on the CPU, run unit by unit
/// ([`Cpu::run`]): pass 1's half layers, pass 2's, then the decoder's
/// stages. Its result never depends on how its units were spread.
pub struct Job {
    unit: usize,
    inputs: Inputs,
    student_inputs: StudentInputs,
    /// Pass 1's rows `[9,w]`, then pass 2's `[17,w]`.
    x: Vec<f32>,
    /// Pass 1's keys and values of the condition rows, by layer: `[9,2w]`.
    kv: Vec<Vec<f32>>,
    student: Option<StudentOutput>,
    decoder: Option<(DecoderInputs, Vec<f32>, Vec<f32>)>,
    local: Option<Vec<f32>>,
}

impl Job {
    /// Its units still to run.
    pub fn left(&self, cpu: &Cpu) -> usize {
        cpu.units() - self.unit
    }

    /// The student's output and the decoder's `[64,365]` (n = 1), once
    /// every unit has run.
    pub fn result(&self) -> Option<(&StudentOutput, &[f32])> {
        Some((self.student.as_ref()?, self.local.as_deref()?))
    }
}

pub(crate) const SEQUENCE: usize = 2 + SLOTS + MAX_TOKENS;
/// The rows of the sequence that read only each other: CLS and the slots.
pub(crate) const CONDITION_ROWS: [usize; 9] = [0, 2, 3, 4, 5, 6, 7, 8, 9];

/// The tensors of both networks with their expected shapes, by name.
pub(crate) fn tensor_shapes(config: &Config) -> Vec<(String, Vec<usize>)> {
    let s = &config.student;
    let (w, ff) = (s.width, s.width * s.ff);
    let mut shapes: Vec<(String, Vec<usize>)> = vec![
        ("student.slot_pos".into(), vec![SLOTS, w]),
        ("student.cls".into(), vec![w]),
        ("student.queries".into(), vec![MAX_TOKENS, w]),
        ("student.attn_mask".into(), vec![SEQUENCE, SEQUENCE]),
        ("student.slot_in.weight".into(), vec![w, SLOT_FEATURES]),
        ("student.slot_in.bias".into(), vec![w]),
        ("student.allowed_in.weight".into(), vec![w, COUNTS]),
        ("student.allowed_in.bias".into(), vec![w]),
        ("student.count_emb.weight".into(), vec![COUNTS, w]),
        ("student.norm.weight".into(), vec![w]),
        ("student.norm.bias".into(), vec![w]),
        ("student.count_out.weight".into(), vec![COUNTS + 1, w]),
        ("student.count_out.bias".into(), vec![COUNTS + 1]),
        ("student.root_out.weight".into(), vec![20, w]),
        ("student.root_out.bias".into(), vec![20]),
        ("student.pose_out.weight".into(), vec![HEADS * CODES, w]),
        ("student.pose_out.bias".into(), vec![HEADS * CODES]),
    ];
    for l in 0..s.layers {
        let p = format!("student.encoder.layers.{l}.");
        for (name, shape) in [
            ("self_attn.in_proj_weight", vec![3 * w, w]),
            ("self_attn.in_proj_bias", vec![3 * w]),
            ("self_attn.out_proj.weight", vec![w, w]),
            ("self_attn.out_proj.bias", vec![w]),
            ("linear1.weight", vec![ff, w]),
            ("linear1.bias", vec![ff]),
            ("linear2.weight", vec![w, ff]),
            ("linear2.bias", vec![w]),
            ("norm1.weight", vec![w]),
            ("norm1.bias", vec![w]),
            ("norm2.weight", vec![w]),
            ("norm2.bias", vec![w]),
        ] {
            shapes.push((format!("{p}{name}"), shape));
        }
    }
    let d = &config.decoder;
    let dw = d.width;
    let mut conv = |name: String, o: usize, c: usize, k: usize| {
        shapes.push((format!("{name}.weight"), vec![o, c, k]));
        shapes.push((format!("{name}.bias"), vec![o]));
    };
    conv("decoder.model.0".into(), dw, d.output_emb_width, 3);
    for i in 0..d.down_t {
        for b in 0..d.depth {
            conv(
                format!("decoder.model.{}.0.model.{b}.conv1", i + 2),
                dw,
                dw,
                3,
            );
            conv(
                format!("decoder.model.{}.0.model.{b}.conv2", i + 2),
                dw,
                dw,
                1,
            );
        }
        conv(format!("decoder.model.{}.2", i + 2), dw, dw, 3);
    }
    conv(format!("decoder.model.{}", 2 + d.down_t), dw, dw, 3);
    conv(
        format!("decoder.model.{}", 4 + d.down_t),
        d.input_emb_width,
        dw,
        3,
    );
    for i in 0..d.down_t {
        let per = 1 << (d.down_t - i);
        shapes.push((
            format!("decoder.external_cond_blocks.{}.weight", 2 * i),
            vec![dw, per * d.external_cond_dim + dw],
        ));
        shapes.push((
            format!("decoder.external_cond_blocks.{}.bias", 2 * i),
            vec![dw],
        ));
        shapes.push((
            format!("decoder.target_cond_blocks.{}.weight", 2 * i),
            vec![dw / per, d.target_cond_dim],
        ));
        shapes.push((
            format!("decoder.target_cond_blocks.{}.bias", 2 * i),
            vec![dw / per],
        ));
    }
    shapes
}

/// The configuration both paths support: the teacher decoder's fixed sizes.
pub(crate) fn check_config(config: &Config) -> Result<(), BundleError> {
    let d = &config.decoder;
    let bad = |what: &str| {
        Err(BundleError::Header(format!(
            "unsupported architecture: {what}"
        )))
    };
    if d.input_emb_width != LOCAL_FEATURES
        || d.output_emb_width != HEADS * 32
        || d.target_cond_dim != TARGET_FEATURES
    {
        return bad("decoder sizes");
    }
    if d.external_cond_dim != 2 || d.cond_fusion_last_layer || FRAMES != MAX_TOKENS << d.down_t {
        return bad("decoder conditions");
    }
    if !d.width.is_multiple_of(1 << d.down_t)
        || !config.student.width.is_multiple_of(config.student.heads)
    {
        return bad("widths");
    }
    Ok(())
}

impl Cpu {
    pub fn new(bundle: &Bundle) -> Result<Cpu, BundleError> {
        let config = bundle.config.clone();
        check_config(&config)?;
        let mut get = std::collections::HashMap::new();
        for (name, shape) in tensor_shapes(&config) {
            get.insert(name.clone(), bundle.get(&name, &shape)?.to_vec());
        }
        let pair =
            |get: &std::collections::HashMap<String, Vec<f32>>, p: &str, a: &str, b: &str| {
                (
                    get[&format!("{p}{a}")].clone(),
                    get[&format!("{p}{b}")].clone(),
                )
            };
        let layers = (0..config.student.layers)
            .map(|l| {
                let p = format!("student.encoder.layers.{l}.");
                EncoderLayer {
                    norm1: pair(&get, &p, "norm1.weight", "norm1.bias"),
                    in_proj: pair(
                        &get,
                        &p,
                        "self_attn.in_proj_weight",
                        "self_attn.in_proj_bias",
                    ),
                    out_proj: pair(
                        &get,
                        &p,
                        "self_attn.out_proj.weight",
                        "self_attn.out_proj.bias",
                    ),
                    norm2: pair(&get, &p, "norm2.weight", "norm2.bias"),
                    linear1: pair(&get, &p, "linear1.weight", "linear1.bias"),
                    linear2: pair(&get, &p, "linear2.weight", "linear2.bias"),
                }
            })
            .collect();
        let mask = get["student.attn_mask"].iter().map(|v| *v > 0.5).collect();
        Ok(Cpu {
            config,
            get,
            layers,
            mask,
        })
    }

    fn w(&self, name: &str) -> &[f32] {
        &self.get[name]
    }

    /// The student's units: two per encoder layer in each of the two passes.
    fn student_units(&self) -> usize {
        4 * self.config.student.layers
    }

    /// The decoder's units: the first convolution, per level its
    /// conditions, residual blocks and upsampling, and the last two
    /// convolutions.
    fn decoder_units(&self) -> usize {
        let d = &self.config.decoder;
        2 + d.down_t * (d.depth + 2)
    }

    /// A [`Job`]'s units, the same for every generation.
    pub fn units(&self) -> usize {
        self.student_units() + self.decoder_units()
    }

    /// One character's generation (`inputs.n` is 1), none of it run yet.
    pub fn job(&self, statics: &Statics, inputs: Inputs) -> Job {
        assert_eq!(inputs.n, 1, "a job is one character's generation");
        let width = self.config.student.width;
        let student_inputs = crate::glue::student_inputs(statics, &inputs);
        let slot_tokens = linear(
            &student_inputs.slots,
            SLOTS,
            self.w("student.slot_in.weight"),
            self.w("student.slot_in.bias"),
        );
        let allowed = linear(
            &student_inputs.allowed,
            1,
            self.w("student.allowed_in.weight"),
            self.w("student.allowed_in.bias"),
        );
        let cls = (0..width).map(|c| self.w("student.cls")[c] + allowed[c]);
        let slots = (0..SLOTS * width).map(|k| slot_tokens[k] + self.w("student.slot_pos")[k]);
        Job {
            unit: 0,
            inputs,
            student_inputs,
            x: cls.chain(slots).collect(),
            kv: Vec::with_capacity(self.layers.len()),
            student: None,
            decoder: None,
            local: None,
        }
    }

    /// Runs at most `units` of the job's units; answers how many ran.
    pub fn run(&self, statics: &Statics, job: &mut Job, units: usize) -> usize {
        let total = self.units();
        let mut ran = 0;
        while ran < units && job.unit < total {
            self.unit(statics, job);
            job.unit += 1;
            ran += 1;
        }
        ran
    }

    fn unit(&self, statics: &Statics, job: &mut Job) {
        let layers = self.layers.len();
        let u = job.unit;
        if u < 2 * layers {
            let layer = &self.layers[u / 2];
            if u % 2 == 0 {
                let mut kv = Vec::new();
                self.attention(&mut job.x, &CONDITION_ROWS, layer, None, Some(&mut kv));
                job.kv.push(kv);
            } else {
                self.feed_forward(&mut job.x, layer);
            }
            if u == 2 * layers - 1 {
                self.count(job);
            }
        } else if u < 4 * layers {
            let l = (u - 2 * layers) / 2;
            let layer = &self.layers[l];
            if u % 2 == 0 {
                let kv = std::mem::take(&mut job.kv[l]);
                self.attention(&mut job.x, &PASS2_ROWS, layer, Some(&kv), None);
                job.kv[l] = kv;
            } else {
                self.feed_forward(&mut job.x, layer);
            }
            if u == 4 * layers - 1 {
                self.heads(statics, job);
            }
        } else {
            self.decoder_unit(job, u - 4 * layers);
        }
    }

    /// Pass 1's end: the count from CLS, and pass 2's rows (the count's
    /// embedding and the queries).
    fn count(&self, job: &mut Job) {
        let width = self.config.student.width;
        layer_norm(
            &mut job.x,
            width,
            self.w("student.norm.weight"),
            self.w("student.norm.bias"),
        );
        let logits = linear(
            &job.x[..width],
            1,
            self.w("student.count_out.weight"),
            self.w("student.count_out.bias"),
        );
        let allowed = &job.student_inputs.allowed[..COUNTS];
        let index = argmax((0..COUNTS).map(|c| {
            if allowed[c] > 0.0 {
                logits[c]
            } else {
                f32::NEG_INFINITY
            }
        }));
        let count = &self.w("student.count_emb.weight")[index * width..(index + 1) * width];
        job.x = [count, self.w("student.queries")].concat();
        job.student = Some(StudentOutput {
            counts: vec![(index + 6) as u32],
            root: Vec::new(),
            tokens: Vec::new(),
        });
    }

    /// Pass 2's end: the root path and the tokens (argmax per head, first
    /// maximum on ties), and the decoder's inputs.
    fn heads(&self, statics: &Statics, job: &mut Job) {
        let width = self.config.student.width;
        layer_norm(
            &mut job.x,
            width,
            self.w("student.norm.weight"),
            self.w("student.norm.bias"),
        );
        let queries = &job.x[width..];
        let root = linear(
            queries,
            MAX_TOKENS,
            self.w("student.root_out.weight"),
            self.w("student.root_out.bias"),
        );
        let pose = linear(
            queries,
            MAX_TOKENS,
            self.w("student.pose_out.weight"),
            self.w("student.pose_out.bias"),
        );
        let student = job.student.as_mut().expect("pass 1 ran");
        student.root = root;
        student.tokens = (0..MAX_TOKENS * HEADS)
            .map(|t| argmax(pose[t * CODES..(t + 1) * CODES].iter().copied()) as u32)
            .collect();
        let inputs = crate::glue::decoder_inputs(statics, &job.inputs, student);
        let mask = inputs.token_mask[..MAX_TOKENS].to_vec();
        let mut x = inputs.codes[..160 * MAX_TOKENS].to_vec();
        masked(&mut x, &mask);
        job.decoder = Some((inputs, mask, x));
        job.x = Vec::new();
        job.kv = Vec::new();
    }

    /// One pre-norm encoder layer's attention half on `x`, whose rows are
    /// the sequence's `rows`. Keys and values are `x`'s own and, with
    /// `cached`, the condition rows' from pass 1 (`store` keeps `x`'s).
    /// Every row reads its keys in sequence order, so a row's values are
    /// those of the whole sequence's pass.
    fn attention(
        &self,
        x: &mut [f32],
        rows: &[usize],
        layer: &EncoderLayer,
        cached: Option<&[f32]>,
        store: Option<&mut Vec<f32>>,
    ) {
        let width = self.config.student.width;
        let heads = self.config.student.heads;
        let dh = width / heads;
        let length = rows.len();
        let mut h = x.to_vec();
        layer_norm(&mut h, width, &layer.norm1.0, &layer.norm1.1);
        let qkv = linear(&h, length, &layer.in_proj.0, &layer.in_proj.1);
        if let Some(store) = store {
            for r in 0..length {
                store.extend_from_slice(&qkv[r * 3 * width + width..(r + 1) * 3 * width]);
            }
        }
        // Each key row by its place in the sequence: its keys then values.
        let mut keys: Vec<(usize, &[f32])> = rows
            .iter()
            .enumerate()
            .map(|(i, &row)| (row, &qkv[i * 3 * width + width..(i + 1) * 3 * width]))
            .collect();
        if let Some(cached) = cached {
            keys.extend(
                CONDITION_ROWS
                    .iter()
                    .enumerate()
                    .map(|(i, &row)| (row, &cached[i * 2 * width..(i + 1) * 2 * width])),
            );
        }
        keys.sort_by_key(|k| k.0);
        let mut attended = vec![0.0; length * width];
        let scale = 1.0 / (dh as f32).sqrt();
        let mut scores = vec![0.0f32; keys.len()];
        for head in 0..heads {
            for (qi, &qrow) in rows.iter().enumerate() {
                let q = &qkv[qi * 3 * width + head * dh..qi * 3 * width + head * dh + dh];
                let mut max = f32::NEG_INFINITY;
                for (ki, &(krow, kv)) in keys.iter().enumerate() {
                    scores[ki] = if self.mask[qrow * SEQUENCE + krow] {
                        f32::NEG_INFINITY
                    } else {
                        dot(q, &kv[head * dh..head * dh + dh]) * scale
                    };
                    max = max.max(scores[ki]);
                }
                let mut total = 0.0;
                for s in scores.iter_mut() {
                    *s = exp(*s - max);
                    total += *s;
                }
                let out = &mut attended[qi * width + head * dh..qi * width + head * dh + dh];
                for (ki, s) in scores.iter().enumerate() {
                    if *s == 0.0 {
                        continue;
                    }
                    let v = &keys[ki].1[width + head * dh..width + head * dh + dh];
                    let p = s / total;
                    for (o, vv) in out.iter_mut().zip(v) {
                        *o += p * vv;
                    }
                }
            }
        }
        let projected = linear(&attended, length, &layer.out_proj.0, &layer.out_proj.1);
        for (a, b) in x.iter_mut().zip(&projected) {
            *a += b;
        }
    }

    /// One pre-norm encoder layer's feed-forward half on `x`.
    fn feed_forward(&self, x: &mut [f32], layer: &EncoderLayer) {
        let width = self.config.student.width;
        let length = x.len() / width;
        let mut h = x.to_vec();
        layer_norm(&mut h, width, &layer.norm2.0, &layer.norm2.1);
        let mut hidden = linear(&h, length, &layer.linear1.0, &layer.linear1.1);
        relu(&mut hidden);
        let out = linear(&hidden, length, &layer.linear2.0, &layer.linear2.1);
        for (a, b) in x.iter_mut().zip(&out) {
            *a += b;
        }
    }

    /// The decoder's unit `u` of [`Cpu::decoder_units`].
    fn decoder_unit(&self, job: &mut Job, u: usize) {
        let d = &self.config.decoder;
        let (width, down, depth) = (d.width, d.down_t, d.depth);
        let (inputs, mask, h) = job.decoder.as_mut().expect("the student ran");
        if u == 0 {
            let mut y = conv1d(
                h,
                MAX_TOKENS,
                self.w("decoder.model.0.weight"),
                self.w("decoder.model.0.bias"),
                3,
                1,
            );
            relu(&mut y);
            *h = y;
            return;
        }
        if u == self.decoder_units() - 1 {
            masked(h, mask);
            let name = format!("decoder.model.{}", 2 + down);
            let mut y = conv1d(
                h,
                FRAMES,
                self.w(&format!("{name}.weight")),
                self.w(&format!("{name}.bias")),
                3,
                1,
            );
            relu(&mut y);
            masked(&mut y, mask);
            let name = format!("decoder.model.{}", 4 + down);
            let mut y = conv1d(
                &y,
                FRAMES,
                self.w(&format!("{name}.weight")),
                self.w(&format!("{name}.bias")),
                3,
                1,
            );
            masked(&mut y, mask);
            let mut out = vec![0.0; FRAMES * LOCAL_FEATURES];
            for f in 0..FRAMES {
                for c in 0..LOCAL_FEATURES {
                    out[f * LOCAL_FEATURES + c] = y[c * FRAMES + f];
                }
            }
            job.local = Some(out);
            job.decoder = None;
            return;
        }
        let level = (u - 1) / (depth + 2);
        let stage = (u - 1) % (depth + 2);
        let per = 1 << (down - level);
        let positions = FRAMES / per;
        let part = width / per;
        if stage == 0 {
            let target = &inputs.target[..FRAMES * TARGET_FEATURES];
            let has = &inputs.has_target[..FRAMES];
            let external = &inputs.external[..FRAMES * 2];
            let mut ht = linear(
                target,
                FRAMES,
                self.w(&format!("decoder.target_cond_blocks.{}.weight", 2 * level)),
                self.w(&format!("decoder.target_cond_blocks.{}.bias", 2 * level)),
            );
            relu(&mut ht);
            // h [W,P] → rows [P, W + 2·per]: the frames' pose condition replaces
            // their part of h, then the ground velocity of the position's frames.
            let fused_width = width + 2 * per;
            let mut fused = vec![0.0; positions * fused_width];
            for p in 0..positions {
                for j in 0..width {
                    let frame = p * per + j / part;
                    fused[p * fused_width + j] = if has[frame] > 0.0 {
                        ht[frame * part + j % part]
                    } else {
                        h[j * positions + p]
                    };
                }
                fused[p * fused_width + width..(p + 1) * fused_width]
                    .copy_from_slice(&external[p * per * 2..(p + 1) * per * 2]);
            }
            let mut rows = linear(
                &fused,
                positions,
                self.w(&format!(
                    "decoder.external_cond_blocks.{}.weight",
                    2 * level
                )),
                self.w(&format!("decoder.external_cond_blocks.{}.bias", 2 * level)),
            );
            relu(&mut rows);
            *h = (0..width * positions)
                .map(|k| rows[(k % positions) * width + k / positions])
                .collect();
        } else if stage <= depth {
            let b = stage - 1;
            let p = format!("decoder.model.{}.0.model.{b}.", level + 2);
            let dilation = d.dilation_growth_rate.pow((depth - 1 - b) as u32);
            masked(h, mask);
            let mut y = h.clone();
            relu(&mut y);
            masked(&mut y, mask);
            let mut y = conv1d(
                &y,
                positions,
                self.w(&format!("{p}conv1.weight")),
                self.w(&format!("{p}conv1.bias")),
                3,
                dilation,
            );
            relu(&mut y);
            masked(&mut y, mask);
            let y = conv1d(
                &y,
                positions,
                self.w(&format!("{p}conv2.weight")),
                self.w(&format!("{p}conv2.bias")),
                1,
                1,
            );
            for (a, b) in h.iter_mut().zip(&y) {
                *a += b;
            }
            masked(h, mask);
        } else {
            // Nearest upsampling ×2.
            let up: Vec<f32> = (0..width * positions * 2)
                .map(|k| h[(k / (positions * 2)) * positions + (k % (positions * 2)) / 2])
                .collect();
            *mask = (0..positions * 2).map(|t| mask[t / 2]).collect();
            let mut up = up;
            masked(&mut up, mask);
            let name = format!("decoder.model.{}.2", level + 2);
            *h = conv1d(
                &up,
                positions * 2,
                self.w(&format!("{name}.weight")),
                self.w(&format!("{name}.bias")),
                3,
                1,
            );
        }
    }

    /// Every character's job, run to its end.
    fn jobs(&self, statics: &Statics, inputs: &Inputs) -> Vec<Job> {
        (0..inputs.n)
            .map(|i| {
                let mut job = self.job(statics, inputs.character(i));
                self.run(statics, &mut job, usize::MAX);
                job
            })
            .collect()
    }

    /// The student and the decoder for every character of `inputs`: each
    /// character's [`Job`], gathered.
    pub fn generate(&self, statics: &Statics, inputs: &Inputs) -> (StudentOutput, Vec<f32>) {
        gather(&self.jobs(statics, inputs))
    }
}

/// Finished jobs' outputs as one batch's.
pub fn gather(jobs: &[Job]) -> (StudentOutput, Vec<f32>) {
    let mut student = StudentOutput {
        counts: Vec::with_capacity(jobs.len()),
        root: Vec::with_capacity(jobs.len() * FRAMES * 5),
        tokens: Vec::with_capacity(jobs.len() * MAX_TOKENS * HEADS),
    };
    let mut local = Vec::with_capacity(jobs.len() * FRAMES * LOCAL_FEATURES);
    for job in jobs {
        let (s, l) = job.result().expect("every unit ran");
        student.counts.extend_from_slice(&s.counts);
        student.root.extend_from_slice(&s.root);
        student.tokens.extend_from_slice(&s.tokens);
        local.extend_from_slice(l);
    }
    (student, local)
}

fn masked(h: &mut [f32], mask: &[f32]) {
    let t = mask.len();
    for (k, v) in h.iter_mut().enumerate() {
        *v *= mask[k % t];
    }
}

/// Index of the first maximum.
pub(crate) fn argmax(values: impl Iterator<Item = f32>) -> usize {
    let mut best = (0, f32::NEG_INFINITY);
    for (i, v) in values.enumerate() {
        if v > best.1 {
            best = (i, v);
        }
    }
    best.0
}
