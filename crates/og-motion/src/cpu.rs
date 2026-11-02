//! The student and the decoder on the CPU: the reference path every other
//! path is checked against, in plain f32 loops over PyTorch weight layouts.
//! The engine runs the model on the GPU only (docs/design.md §2.7): [`Cpu`]
//! is built with the `reference` feature alone, which the crate's tests,
//! examples and tools turn on and no host does. The tables both paths read
//! (the tensors' shapes, the supported architecture) are always built.

#[cfg(feature = "reference")]
use og_core::math::exp;

#[cfg(feature = "reference")]
use crate::bundle::Bundle;
use crate::bundle::{BundleError, Config};
use crate::contract::{COUNTS, FRAMES, MAX_TOKENS, SLOTS};
use crate::glue::{CODES, HEADS, LOCAL_FEATURES, SLOT_FEATURES, TARGET_FEATURES};
#[cfg(feature = "reference")]
use crate::glue::{DecoderInputs, StudentInputs, StudentOutput};

#[cfg(feature = "reference")]
/// `y[r,o] = b[o] + Σk x[r,k]·w[o,k]` for `rows` rows.
pub(crate) fn linear(x: &[f32], rows: usize, w: &[f32], b: &[f32]) -> Vec<f32> {
    let out = b.len();
    let k = w.len() / out;
    debug_assert_eq!(x.len(), rows * k);
    let mut y = vec![0.0; rows * out];
    for r in 0..rows {
        let xr = &x[r * k..(r + 1) * k];
        for o in 0..out {
            let wo = &w[o * k..(o + 1) * k];
            y[r * out + o] = b[o] + xr.iter().zip(wo).map(|(a, b)| a * b).sum::<f32>();
        }
    }
    y
}

#[cfg(feature = "reference")]
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

#[cfg(feature = "reference")]
fn relu(x: &mut [f32]) {
    for v in x {
        *v = v.max(0.0);
    }
}

#[cfg(feature = "reference")]
/// `x [C,T]`, `w [O,C,K]`, 'same' padding (`dilation · (K/2)` zeros each side).
fn conv1d(x: &[f32], t: usize, w: &[f32], b: &[f32], kernel: usize, dilation: usize) -> Vec<f32> {
    let out = b.len();
    let channels = x.len() / t;
    let pad = (dilation * (kernel / 2)) as isize;
    let mut y = vec![0.0; out * t];
    for o in 0..out {
        let yo = &mut y[o * t..(o + 1) * t];
        yo.fill(b[o]);
        for c in 0..channels {
            let xc = &x[c * t..(c + 1) * t];
            for k in 0..kernel {
                let weight = w[(o * channels + c) * kernel + k];
                let shift = k as isize * dilation as isize - pad;
                for (s, yv) in yo.iter_mut().enumerate() {
                    let at = s as isize + shift;
                    if at >= 0 && (at as usize) < t {
                        *yv += weight * xc[at as usize];
                    }
                }
            }
        }
    }
    y
}

#[cfg(feature = "reference")]
struct EncoderLayer {
    norm1: (Vec<f32>, Vec<f32>),
    in_proj: (Vec<f32>, Vec<f32>),
    out_proj: (Vec<f32>, Vec<f32>),
    norm2: (Vec<f32>, Vec<f32>),
    linear1: (Vec<f32>, Vec<f32>),
    linear2: (Vec<f32>, Vec<f32>),
}

#[cfg(feature = "reference")]
/// Both networks' weights, f32, PyTorch layouts.
pub struct Cpu {
    config: Config,
    get: std::collections::HashMap<String, Vec<f32>>,
    layers: Vec<EncoderLayer>,
    /// `[26·26]`, true where attention is not allowed.
    mask: Vec<bool>,
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

#[cfg(feature = "reference")]
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

    /// One pre-norm encoder layer on `[length, width]`, `rows` naming which
    /// rows of the full sequence these are (for the mask).
    fn layer(&self, x: &mut [f32], rows: &[usize], layer: &EncoderLayer) {
        let width = self.config.student.width;
        let heads = self.config.student.heads;
        let dh = width / heads;
        let length = rows.len();
        let mut h = x.to_vec();
        layer_norm(&mut h, width, &layer.norm1.0, &layer.norm1.1);
        let qkv = linear(&h, length, &layer.in_proj.0, &layer.in_proj.1);
        let mut attended = vec![0.0; length * width];
        let scale = 1.0 / (dh as f32).sqrt();
        let mut scores = vec![0.0f32; length];
        for head in 0..heads {
            for (qi, &qrow) in rows.iter().enumerate() {
                let q = &qkv[qi * 3 * width + head * dh..qi * 3 * width + head * dh + dh];
                let mut max = f32::NEG_INFINITY;
                for (ki, &krow) in rows.iter().enumerate() {
                    scores[ki] = if self.mask[qrow * SEQUENCE + krow] {
                        f32::NEG_INFINITY
                    } else {
                        let k = &qkv[ki * 3 * width + width + head * dh
                            ..ki * 3 * width + width + head * dh + dh];
                        q.iter().zip(k).map(|(a, b)| a * b).sum::<f32>() * scale
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
                    let v = &qkv[ki * 3 * width + 2 * width + head * dh
                        ..ki * 3 * width + 2 * width + head * dh + dh];
                    for (o, vv) in out.iter_mut().zip(v) {
                        *o += s / total * vv;
                    }
                }
            }
        }
        let projected = linear(&attended, length, &layer.out_proj.0, &layer.out_proj.1);
        for (a, b) in x.iter_mut().zip(&projected) {
            *a += b;
        }
        let mut h = x.to_vec();
        layer_norm(&mut h, width, &layer.norm2.0, &layer.norm2.1);
        let mut hidden = linear(&h, length, &layer.linear1.0, &layer.linear1.1);
        relu(&mut hidden);
        let out = linear(&hidden, length, &layer.linear2.0, &layer.linear2.1);
        for (a, b) in x.iter_mut().zip(&out) {
            *a += b;
        }
    }

    fn encode(&self, x: &mut [f32], rows: &[usize]) {
        for layer in &self.layers {
            self.layer(x, rows, layer);
        }
        layer_norm(
            x,
            self.config.student.width,
            self.w("student.norm.weight"),
            self.w("student.norm.bias"),
        );
    }

    /// Pass 1 on CLS and the slots picks the count; pass 2 with it gives the
    /// root path and the tokens (argmax per head, first maximum on ties).
    pub fn student(&self, inputs: &StudentInputs) -> StudentOutput {
        let width = self.config.student.width;
        let n = inputs.n;
        let slot_tokens = linear(
            &inputs.slots,
            n * SLOTS,
            self.w("student.slot_in.weight"),
            self.w("student.slot_in.bias"),
        );
        let allowed = linear(
            &inputs.allowed,
            n,
            self.w("student.allowed_in.weight"),
            self.w("student.allowed_in.bias"),
        );
        let mut out = StudentOutput {
            counts: vec![0; n],
            root: vec![0.0; n * FRAMES * 5],
            tokens: vec![0; n * MAX_TOKENS * HEADS],
        };
        let all_rows: Vec<usize> = (0..SEQUENCE).collect();
        for i in 0..n {
            let cls: Vec<f32> = (0..width)
                .map(|c| self.w("student.cls")[c] + allowed[i * width + c])
                .collect();
            let slots: Vec<f32> = (0..SLOTS * width)
                .map(|k| slot_tokens[i * SLOTS * width + k] + self.w("student.slot_pos")[k])
                .collect();
            let mut conditions = [cls.clone(), slots.clone()].concat();
            self.encode(&mut conditions, &CONDITION_ROWS);
            let logits = linear(
                &conditions[..width],
                1,
                self.w("student.count_out.weight"),
                self.w("student.count_out.bias"),
            );
            let allowed = &inputs.allowed[i * COUNTS..(i + 1) * COUNTS];
            let index = argmax((0..COUNTS).map(|c| {
                if allowed[c] > 0.0 {
                    logits[c]
                } else {
                    f32::NEG_INFINITY
                }
            }));
            out.counts[i] = (index + 6) as u32;

            let count = &self.w("student.count_emb.weight")[index * width..(index + 1) * width];
            let mut sequence = [
                cls,
                count.to_vec(),
                slots,
                self.w("student.queries").to_vec(),
            ]
            .concat();
            self.encode(&mut sequence, &all_rows);
            let queries = &sequence[(2 + SLOTS) * width..];
            let root = linear(
                queries,
                MAX_TOKENS,
                self.w("student.root_out.weight"),
                self.w("student.root_out.bias"),
            );
            out.root[i * FRAMES * 5..(i + 1) * FRAMES * 5].copy_from_slice(&root);
            let pose = linear(
                queries,
                MAX_TOKENS,
                self.w("student.pose_out.weight"),
                self.w("student.pose_out.bias"),
            );
            for t in 0..MAX_TOKENS * HEADS {
                out.tokens[i * MAX_TOKENS * HEADS + t] =
                    argmax(pose[t * CODES..(t + 1) * CODES].iter().copied()) as u32;
            }
        }
        out
    }

    /// The decoder: `[n,64,365]` normalised local motion features.
    pub fn decoder(&self, inputs: &DecoderInputs) -> Vec<f32> {
        let d = &self.config.decoder;
        let (width, down) = (d.width, d.down_t);
        let mut out = vec![0.0; inputs.n * FRAMES * LOCAL_FEATURES];
        for i in 0..inputs.n {
            let mut mask: Vec<f32> =
                inputs.token_mask[i * MAX_TOKENS..(i + 1) * MAX_TOKENS].to_vec();
            let masked = |h: &mut [f32], mask: &[f32]| {
                let t = mask.len();
                for (k, v) in h.iter_mut().enumerate() {
                    *v *= mask[k % t];
                }
            };
            let mut x = inputs.codes[i * 160 * MAX_TOKENS..(i + 1) * 160 * MAX_TOKENS].to_vec();
            masked(&mut x, &mask);
            let mut h = conv1d(
                &x,
                MAX_TOKENS,
                self.w("decoder.model.0.weight"),
                self.w("decoder.model.0.bias"),
                3,
                1,
            );
            relu(&mut h);
            let target =
                &inputs.target[i * FRAMES * TARGET_FEATURES..(i + 1) * FRAMES * TARGET_FEATURES];
            let has = &inputs.has_target[i * FRAMES..(i + 1) * FRAMES];
            let external = &inputs.external[i * FRAMES * 2..(i + 1) * FRAMES * 2];
            for level in 0..down {
                let per = 1 << (down - level);
                let positions = FRAMES / per;
                let part = width / per;
                let ht = {
                    let mut ht = linear(
                        target,
                        FRAMES,
                        self.w(&format!("decoder.target_cond_blocks.{}.weight", 2 * level)),
                        self.w(&format!("decoder.target_cond_blocks.{}.bias", 2 * level)),
                    );
                    relu(&mut ht);
                    ht
                };
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
                h = (0..width * positions)
                    .map(|k| rows[(k % positions) * width + k / positions])
                    .collect();
                for b in 0..d.depth {
                    let p = format!("decoder.model.{}.0.model.{b}.", level + 2);
                    let dilation = d.dilation_growth_rate.pow((d.depth - 1 - b) as u32);
                    masked(&mut h, &mask);
                    let mut y = h.clone();
                    relu(&mut y);
                    masked(&mut y, &mask);
                    let mut y = conv1d(
                        &y,
                        positions,
                        self.w(&format!("{p}conv1.weight")),
                        self.w(&format!("{p}conv1.bias")),
                        3,
                        dilation,
                    );
                    relu(&mut y);
                    masked(&mut y, &mask);
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
                    masked(&mut h, &mask);
                }
                // Nearest upsampling ×2.
                h = (0..width * positions * 2)
                    .map(|k| h[(k / (positions * 2)) * positions + (k % (positions * 2)) / 2])
                    .collect();
                mask = (0..positions * 2).map(|t| mask[t / 2]).collect();
                masked(&mut h, &mask);
                let name = format!("decoder.model.{}.2", level + 2);
                h = conv1d(
                    &h,
                    positions * 2,
                    self.w(&format!("{name}.weight")),
                    self.w(&format!("{name}.bias")),
                    3,
                    1,
                );
            }
            masked(&mut h, &mask);
            let name = format!("decoder.model.{}", 2 + down);
            h = conv1d(
                &h,
                FRAMES,
                self.w(&format!("{name}.weight")),
                self.w(&format!("{name}.bias")),
                3,
                1,
            );
            relu(&mut h);
            masked(&mut h, &mask);
            let name = format!("decoder.model.{}", 4 + down);
            h = conv1d(
                &h,
                FRAMES,
                self.w(&format!("{name}.weight")),
                self.w(&format!("{name}.bias")),
                3,
                1,
            );
            masked(&mut h, &mask);
            for f in 0..FRAMES {
                for c in 0..LOCAL_FEATURES {
                    out[(i * FRAMES + f) * LOCAL_FEATURES + c] = h[c * FRAMES + f];
                }
            }
        }
        out
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
