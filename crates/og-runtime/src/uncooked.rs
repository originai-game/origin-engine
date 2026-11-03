//! Uncooked loads at run time (IDL `load`: `imageData`, `gltfData`,
//! `environmentData`, `imageInfo`, `environmentInfo`, `completions`,
//! `replayCompletions`): content a player or a
//! generator made, from bytes, without `og cook`.
//!
//! The source is untrusted, so the budget (`LoadBudget`) is checked from its
//! headers before anything is decoded. Decoding never runs on the engine's
//! thread on the web: the host decodes (the Worker's `createImageBitmap`,
//! [`Hooks::image_decode_start`]); natively a worker thread does, with the
//! `image` crate. No decoder is in the wasm. A glTF is read at the call
//! (og-bundle's [`og_bundle::runtime::UncookedGltf`]: its JSON, buffers,
//! triangle count and image headers), then its images decode as above while
//! its primitives are converted at the boundaries, one at a time within the
//! frame budget on the web, and it joins the renderer through the path a
//! cooked model takes. An HDR environment (a Radiance `.hdr`, natively an
//! OpenEXR `.exr` too) decodes on a worker thread natively and in rows at
//! the boundaries on the web (og-bundle's [`og_bundle::runtime::HdrRows`]),
//! its texels packed RGB9E5 and its irradiance gathered as it goes; the
//! renderer prefilters it on the GPU when it first draws with it.
//!
//! Completion is host input. A finished load is put in place at a tick
//! boundary and its tick journaled ([`Uncooked::journal`]), as a replay
//! journals key presses: natively at the first boundary after the call (the
//! boundary waits for the worker thread, so a native run is the same on
//! every machine), on the web at the first boundary after the browser has
//! decoded it, within the frame budget. A replay hands the journal back
//! ([`Uncooked::schedule`]) and each load is put in place at its recorded
//! tick, waited for there.

use super::Fetch;
use super::assets::{Entry, Payload, Status};
use super::engine::{Engine, Hooks, hooks};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// What `LoadBudget` leaves unset.
const BYTES: f64 = 32.0 * 1024.0 * 1024.0;
const TEXTURE_SIZE: u32 = 4096;
const FRAME_MS: f64 = 2.0;
/// The largest texture every WebGPU device takes (`maxTextureDimension2D`'s
/// guaranteed minimum): `textureSize` is capped here.
const DEVICE_TEXTURE_SIZE: u32 = 8192;

/// Whether og.load.environmentData reads OpenEXR here
/// (RenderCapabilities.exrDecode): natively, through the `image` crate; the
/// web module carries no EXR decoder.
pub(crate) const EXR_DECODE: bool = cfg!(not(target_arch = "wasm32"));

/// The face sizes an uncooked environment's cube may have.
const ENV_SIZES: std::ops::RangeInclusive<u32> =
    og_render::ENVIRONMENT_MAP_MIN_SIZE..=og_render::ENVIRONMENT_MAP_MAX_SIZE;

/// A `LoadBudget` as a load holds to it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Budget {
    pub bytes: f64,
    pub triangles: u32,
    pub texture_size: u32,
    pub frame_ms: f64,
}

impl Budget {
    /// A desc's `budget` (each field its default when omitted).
    pub fn from_desc(desc: &Value) -> Result<Budget, String> {
        let b = &desc["budget"];
        let num = |k: &str, default: f64| -> Result<f64, String> {
            match &b[k] {
                Value::Null => Ok(default),
                Value::Number(n) => n
                    .as_f64()
                    .filter(|v| v.is_finite() && *v >= 0.0)
                    .ok_or_else(|| format!("LoadBudget.{k} is a number of at least 0, got {n}")),
                other => Err(format!("LoadBudget.{k} is a number, got {other}")),
            }
        };
        Ok(Budget {
            bytes: num("bytes", BYTES)?,
            triangles: num("triangles", 500_000.0)?.min(u32::MAX as f64) as u32,
            texture_size: (num("textureSize", TEXTURE_SIZE as f64)?.min(u32::MAX as f64) as u32)
                .min(DEVICE_TEXTURE_SIZE),
            frame_ms: num("frameMs", FRAME_MS)?,
        })
    }
}

/// An image's format and size, read from its header only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ImageHeader {
    pub format: &'static str,
    pub width: u32,
    pub height: u32,
}

/// The format and size of a PNG or JPEG from its header, without decoding
/// it; why not for anything else.
pub(crate) fn image_header(bytes: &[u8]) -> Result<ImageHeader, String> {
    const PNG: &[u8] = &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    if bytes.starts_with(PNG) {
        if bytes.len() < 24 || &bytes[12..16] != b"IHDR" {
            return Err("a PNG without its IHDR header (the file is cut short)".into());
        }
        let be = |at: usize| {
            u32::from_be_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
        };
        return Ok(ImageHeader {
            format: "png",
            width: be(16),
            height: be(20),
        });
    }
    if bytes.starts_with(&[0xff, 0xd8]) {
        // Markers until a start of frame (SOF0 to SOF15 but DHT, JPG and DAC).
        let mut at = 2;
        while at + 4 <= bytes.len() {
            if bytes[at] != 0xff {
                return Err("a JPEG whose markers do not parse".into());
            }
            let marker = bytes[at + 1];
            if marker == 0xff {
                at += 1;
                continue;
            }
            if (0xd0..=0xd9).contains(&marker) || marker == 0x01 {
                at += 2;
                continue;
            }
            let len = u16::from_be_bytes([bytes[at + 2], bytes[at + 3]]) as usize;
            if (0xc0..=0xcf).contains(&marker) && ![0xc4, 0xc8, 0xcc].contains(&marker) {
                if at + 9 > bytes.len() {
                    break;
                }
                let height = u16::from_be_bytes([bytes[at + 5], bytes[at + 6]]) as u32;
                let width = u16::from_be_bytes([bytes[at + 7], bytes[at + 8]]) as u32;
                return Ok(ImageHeader {
                    format: "jpeg",
                    width,
                    height,
                });
            }
            at += 2 + len;
        }
        return Err("a JPEG without a frame header (the file is cut short)".into());
    }
    Err("not a PNG or JPEG (uncooked loads read those two; convert other formats first)".into())
}

/// An image's header checked against a budget: the reasons name the limit,
/// the value and the fix.
pub(crate) fn check_image(
    name: &str,
    bytes: &[u8],
    budget: &Budget,
) -> Result<ImageHeader, String> {
    check_size(name, bytes, budget)?;
    let h = image_header(bytes).map_err(|e| format!("{name}: {e}"))?;
    if h.width == 0 || h.height == 0 {
        return Err(format!(
            "{name}: a {} of {}x{} texels holds nothing",
            h.format, h.width, h.height
        ));
    }
    if h.width.max(h.height) > budget.texture_size {
        return Err(format!(
            "{name}: {}x{} is over LoadBudget.textureSize ({}); scale the image down or raise the budget (at most {DEVICE_TEXTURE_SIZE}, the largest every device takes)",
            h.width, h.height, budget.texture_size
        ));
    }
    Ok(h)
}

/// A decoded image: RGBA8, rows top to bottom.
pub struct Decoded {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Where an image's decode is.
enum Decode {
    /// On a worker thread of this process.
    #[cfg(not(target_arch = "wasm32"))]
    Thread(std::sync::mpsc::Receiver<Result<Decoded, String>>),
    /// With the host, by key ([`Hooks::image_decode_take`]).
    Host(u64),
    Done(Result<Decoded, String>),
}

/// An image to decode: what its header said and how its texels are read.
struct Image {
    header: ImageHeader,
    decode: Decode,
}

/// A glTF on its way: its images decoding, its primitives converted one
/// at a time.
struct Gltf {
    file: og_bundle::runtime::UncookedGltf,
    images: Vec<(String, Image)>,
    prims: Vec<og_bundle::runtime::CookedPrimitive>,
    error: Option<String>,
}

/// An environment's image decoded: RGB9E5 texels, rows from the top, and
/// its irradiance.
struct Prepared {
    width: u32,
    height: u32,
    texels: Vec<u32>,
    irradiance: [[f32; 3]; 9],
}

/// Rows of an environment's image packed as they come, its irradiance
/// gathered beside them; a non-finite pixel refuses the image.
struct Packer {
    width: u32,
    height: u32,
    intensity: f32,
    texels: Vec<u32>,
    irradiance: og_render::EquirectIrradiance,
}

impl Packer {
    fn new(width: u32, height: u32, intensity: f32) -> Packer {
        Packer {
            width,
            height,
            intensity,
            texels: Vec::with_capacity(width as usize * height as usize),
            irradiance: og_render::EquirectIrradiance::new(width, height),
        }
    }

    fn row(&mut self, y: u32, rgb: &mut [f32]) -> Result<(), String> {
        if rgb.len() != self.width as usize * 3 {
            return Err(format!("row {y} does not hold {} pixels", self.width));
        }
        if let Some(i) = rgb.iter().position(|v| !v.is_finite()) {
            return Err(format!(
                "pixel {} of row {y} holds a value that is not a finite number",
                i / 3
            ));
        }
        for v in rgb.iter_mut() {
            *v = v.max(0.0) * self.intensity;
        }
        self.irradiance.add_row(y, rgb);
        self.texels
            .extend(rgb.as_chunks::<3>().0.iter().map(|c| og_render::rgb9e5(*c)));
        Ok(())
    }

    fn finish(self) -> Prepared {
        Prepared {
            width: self.width,
            height: self.height,
            irradiance: self.irradiance.finish(),
            texels: self.texels,
        }
    }
}

/// Where an environment's decode is (inside the job's box: one a load).
#[allow(clippy::large_enum_variant)]
enum EnvDecode {
    /// A `.hdr` read in rows at the boundaries (the web).
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    Rows {
        rows: og_bundle::runtime::HdrRows,
        packer: Packer,
        row: Vec<f32>,
    },
    /// On a worker thread of this process.
    #[cfg(not(target_arch = "wasm32"))]
    Thread(std::sync::mpsc::Receiver<Result<Prepared, String>>),
    Done(Result<Prepared, String>),
}

struct Env {
    size: u32,
    decode: EnvDecode,
}

enum Work {
    Image {
        image: Image,
        pixel: bool,
        linear: bool,
    },
    Gltf(Box<Gltf>),
    Environment(Box<Env>),
}

struct Job {
    request: u32,
    /// Its entry in the engine's assets.
    entry: usize,
    name: String,
    frame_ms: f64,
    work: Work,
}

/// One load put in place (`LoadCompletion`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Completion {
    pub request: u32,
    pub tick: u64,
    pub ok: bool,
}

/// Every uncooked load of an engine.
#[derive(Default)]
pub(crate) struct Uncooked {
    next: u32,
    jobs: Vec<Job>,
    next_key: u64,
    /// What was put in place, oldest first (og.load.completions).
    pub journal: Vec<Completion>,
    /// A recorded journal to replay (og.load.replayCompletions), by request.
    schedule: Option<BTreeMap<u32, Completion>>,
    /// The entries uncooked loads made and not released (og.load.release
    /// frees these alone), with a ready glTF's textures in the scene.
    owned: BTreeMap<usize, Vec<og_render::TextureHandle>>,
    /// Host decodes of released loads, taken and dropped as they land.
    discard: Vec<u64>,
}

#[cfg(not(target_arch = "wasm32"))]
fn decode_native(bytes: &[u8], header: ImageHeader) -> Result<Decoded, String> {
    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes));
    reader.set_format(if header.format == "png" {
        image::ImageFormat::Png
    } else {
        image::ImageFormat::Jpeg
    });
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(header.width);
    limits.max_image_height = Some(header.height);
    reader.limits(limits);
    let img = reader.decode().map_err(|e| e.to_string())?.into_rgba8();
    Ok(Decoded {
        width: img.width(),
        height: img.height(),
        rgba: img.into_raw(),
    })
}

/// The source's size against `LoadBudget.bytes`.
fn check_size(name: &str, bytes: &[u8], budget: &Budget) -> Result<(), String> {
    if bytes.len() as f64 > budget.bytes {
        return Err(format!(
            "{name}: {} bytes is over LoadBudget.bytes ({}); make the file smaller or raise the budget",
            bytes.len(),
            budget.bytes
        ));
    }
    Ok(())
}

/// A pending entry and its job, queued for the tick boundaries.
fn queue(
    eng: &Engine,
    name: &str,
    budget: &Budget,
    work: impl FnOnce(&mut Uncooked) -> Work,
) -> usize {
    let mut assets = eng.assets.borrow_mut();
    let entry = assets.entries.len();
    assets.entries.push(Entry {
        path: name.to_string(),
        status: Status::Pending,
        payload: Payload::Uncooked,
    });
    let u = &mut assets.uncooked;
    let request = u.next;
    u.next += 1;
    u.owned.insert(entry, Vec::new());
    let work = work(u);
    u.jobs.push(Job {
        request,
        entry,
        name: name.to_string(),
        frame_ms: budget.frame_ms,
        work,
    });
    entry
}

/// `og.load.imageData`: checks the image against its budget, starts its
/// decode and answers its (pending) asset index.
pub(crate) fn image_data(
    eng: &Engine,
    name: &str,
    bytes: Vec<u8>,
    desc: &Value,
) -> Result<usize, String> {
    let budget = Budget::from_desc(desc)?;
    let (pixel, linear) = match &desc["kind"] {
        Value::Null => (false, false),
        Value::String(k) if k == "color" => (false, false),
        Value::String(k) if k == "linear" => (false, true),
        Value::String(k) if k == "pixel" => (true, false),
        other => {
            return Err(format!(
                "ImageDataDesc.kind is color, linear or pixel, got {other}"
            ));
        }
    };
    let header = check_image(name, &bytes, &budget)?;
    Ok(queue(eng, name, &budget, |u| Work::Image {
        image: start(u, bytes, header),
        pixel,
        linear,
    }))
}

/// `og.load.gltfData`: reads the glTF and checks it against its budget
/// (its size, its triangles, each image's header), starts its images'
/// decodes and answers its (pending) asset index.
pub(crate) fn gltf_data(
    eng: &Engine,
    name: &str,
    bytes: Vec<u8>,
    desc: &Value,
) -> Result<usize, String> {
    let budget = Budget::from_desc(desc)?;
    check_size(name, &bytes, &budget)?;
    let mut file =
        og_bundle::runtime::UncookedGltf::read(name, &bytes).map_err(|e| format!("{name}: {e}"))?;
    drop(bytes);
    if file.triangles > budget.triangles as u64 {
        return Err(format!(
            "{name}: {} triangles is over LoadBudget.triangles ({}); simplify the model (a decimate modifier, fewer subdivisions) or raise the budget",
            file.triangles, budget.triangles
        ));
    }
    let images = std::mem::take(&mut file.images);
    let mut headers = Vec::with_capacity(images.len());
    for (what, bytes) in &images {
        headers.push(check_image(&format!("{name}: {what}"), bytes, &budget)?);
    }
    Ok(queue(eng, name, &budget, |u| {
        let images = images
            .into_iter()
            .zip(headers)
            .map(|((what, bytes), header)| (what, start(u, bytes, header)))
            .collect();
        Work::Gltf(Box::new(Gltf {
            file,
            images,
            prims: Vec::new(),
            error: None,
        }))
    }))
}

/// An environment's whole image decoded and packed (a worker thread's
/// work natively).
#[cfg(not(target_arch = "wasm32"))]
fn prepare_native(bytes: Vec<u8>, exr: bool, intensity: f32) -> Result<Prepared, String> {
    if exr {
        let (w, h) = og_bundle::runtime::exr_size(&bytes)?;
        let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes));
        reader.set_format(image::ImageFormat::OpenExr);
        let mut limits = image::Limits::default();
        limits.max_image_width = Some(w);
        limits.max_image_height = Some(h);
        reader.limits(limits);
        let mut img = reader.decode().map_err(|e| e.to_string())?.into_rgb32f();
        if (img.width(), img.height()) != (w, h) {
            return Err(format!(
                "decoded to {}x{}, but its header says {w}x{h}",
                img.width(),
                img.height()
            ));
        }
        let mut packer = Packer::new(w, h, intensity);
        for (y, row) in img.chunks_mut(w as usize * 3).enumerate() {
            packer.row(y as u32, row)?;
        }
        return Ok(packer.finish());
    }
    let mut rows = og_bundle::runtime::HdrRows::new(bytes)?;
    let mut packer = Packer::new(rows.width(), rows.height(), intensity);
    let mut row = Vec::new();
    while rows.next_row(&mut row)? {
        packer.row(rows.row() - 1, &mut row)?;
    }
    Ok(packer.finish())
}

/// `og.load.environmentData`: checks the image against its budget and
/// the desc, starts its decode and answers its (pending) asset index.
pub(crate) fn environment_data(
    eng: &Engine,
    name: &str,
    bytes: Vec<u8>,
    desc: &Value,
) -> Result<usize, String> {
    let budget = Budget::from_desc(desc)?;
    let size = match &desc["size"] {
        Value::Null => 128,
        v => v
            .as_u64()
            .ok_or_else(|| format!("EnvironmentDataDesc.size is a whole number, got {v}"))?
            as u32,
    };
    if !size.is_power_of_two() || !ENV_SIZES.contains(&size) {
        return Err(format!(
            "{name}: EnvironmentDataDesc.size is a power of two from {} to {}, not {size}",
            ENV_SIZES.start(),
            ENV_SIZES.end()
        ));
    }
    if size > budget.texture_size {
        return Err(format!(
            "{name}: EnvironmentDataDesc.size {size} is over LoadBudget.textureSize ({}); ask for a smaller cube or raise the budget",
            budget.texture_size
        ));
    }
    let intensity = match &desc["intensity"] {
        Value::Null => 1.0,
        v => v
            .as_f64()
            .filter(|x| x.is_finite() && *x >= 0.0)
            .ok_or_else(|| {
                format!("EnvironmentDataDesc.intensity is a number of at least 0, got {v}")
            })? as f32,
    };
    check_size(name, &bytes, &budget)?;
    let exr = og_bundle::runtime::is_exr(&bytes);
    let (width, height) = if exr {
        if !EXR_DECODE {
            return Err(format!(
                "{name}: an OpenEXR image, which this host does not decode (RenderCapabilities.exrDecode is false on the web): convert it to a Radiance .hdr, or cook it as an environment"
            ));
        }
        og_bundle::runtime::exr_size(&bytes).map_err(|e| format!("{name}: {e}"))?
    } else if og_bundle::runtime::is_hdr(&bytes) {
        og_bundle::runtime::hdr_size(&bytes).map_err(|e| format!("{name}: {e}"))?
    } else {
        return Err(format!(
            "{name}: not a Radiance .hdr or OpenEXR .exr (an environment is an equirectangular HDR image)"
        ));
    };
    if width.max(height) > budget.texture_size {
        return Err(format!(
            "{name}: {width}x{height} is over LoadBudget.textureSize ({}); scale the image down or raise the budget (at most {DEVICE_TEXTURE_SIZE}, the largest every device takes)",
            budget.texture_size
        ));
    }
    if width < 2 * height.saturating_sub(1) || width > 2 * height + 1 {
        return Err(format!(
            "{name} is {width} × {height}: an equirectangular image is twice as wide as it is tall"
        ));
    }
    #[cfg(not(target_arch = "wasm32"))]
    let decode = {
        let (send, recv) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = send.send(prepare_native(bytes, exr, intensity));
        });
        EnvDecode::Thread(recv)
    };
    #[cfg(target_arch = "wasm32")]
    let decode = {
        let rows = og_bundle::runtime::HdrRows::new(bytes).map_err(|e| format!("{name}: {e}"))?;
        EnvDecode::Rows {
            packer: Packer::new(rows.width(), rows.height(), intensity),
            rows,
            row: Vec::new(),
        }
    };
    Ok(queue(eng, name, &budget, |_| {
        Work::Environment(Box::new(Env { size, decode }))
    }))
}

/// Advances an environment's decode; whether it is finished. Each row is
/// a unit of the frame's work.
fn env_step(env: &mut Env, clock: &mut Clock, frame_ms: f64, wait: bool) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    if let EnvDecode::Thread(recv) = &env.decode {
        let got = if wait {
            recv.recv().map_err(|_| ())
        } else {
            recv.try_recv().map_err(|_| ())
        };
        match got {
            Ok(r) => env.decode = EnvDecode::Done(r),
            Err(()) if wait => {
                env.decode = EnvDecode::Done(Err("the decoding thread stopped".into()))
            }
            Err(()) => return false,
        }
    }
    if let EnvDecode::Rows { rows, packer, row } = &mut env.decode {
        loop {
            // The last row read finishes the image in the same frame.
            if (rows.row() < rows.height()) && !wait && clock.over(frame_ms) {
                return false;
            }
            match rows.next_row(row) {
                Ok(true) => {
                    clock.worked = true;
                    if let Err(e) = packer.row(rows.row() - 1, row) {
                        env.decode = EnvDecode::Done(Err(e));
                        break;
                    }
                }
                Ok(false) => {
                    let EnvDecode::Rows { packer, .. } =
                        std::mem::replace(&mut env.decode, EnvDecode::Done(Err(String::new())))
                    else {
                        unreachable!()
                    };
                    env.decode = EnvDecode::Done(Ok(packer.finish()));
                    break;
                }
                Err(e) => {
                    env.decode = EnvDecode::Done(Err(e));
                    break;
                }
            }
        }
    }
    true
}

/// A decoded environment into the renderer: its map added, the asset's
/// payload naming it.
fn put_environment(eng: &Engine, name: &str, entry: usize, env: Env) -> Result<(), String> {
    let EnvDecode::Done(result) = env.decode else {
        unreachable!("only finished jobs are put in place");
    };
    let p = result?;
    let mut rs = eng.render.borrow_mut();
    let handle = rs.scene.add_environment_equirect(
        og_render::EnvironmentEquirect {
            width: p.width,
            height: p.height,
            texels: p.texels,
            size: env.size,
        },
        p.irradiance,
    )?;
    // Not in by_path: og.load.environment never answers it, and a release
    // never brings it back.
    rs.env
        .maps
        .push((format!("{name} (uncooked)"), Some(handle)));
    let map = rs.env.maps.len() - 1;
    drop(rs);
    eng.assets.borrow_mut().entries[entry].payload = Payload::Environment {
        map,
        width: p.width,
        height: p.height,
        size: env.size,
    };
    Ok(())
}

/// `og.load.environmentInfo`.
pub(crate) fn environment_info(eng: &Engine, index: Option<usize>) -> Result<Value, String> {
    let a = eng.assets.borrow();
    let e = index
        .and_then(|i| a.entries.get(i))
        .ok_or("not an asset handle")?;
    match (&e.status, &e.payload) {
        (Status::Failed(r), _) => Err(r.clone()),
        (Status::Pending, Payload::Uncooked) => Err(format!(
            "{} is still loading: og.load.status answers ready once it is in place",
            e.path
        )),
        (
            Status::Ready,
            Payload::Environment {
                map,
                width,
                height,
                size,
            },
        ) => Ok(json!({
            "map": super::assets::handle(*map) as f64,
            "width": width,
            "height": height,
            "size": size,
        })),
        _ => Err(format!(
            "{} is not an environment og.load.environmentData loaded",
            e.path
        )),
    }
}

fn start(u: &mut Uncooked, bytes: Vec<u8>, header: ImageHeader) -> Image {
    let key = u.next_key;
    u.next_key += 1;
    let decode = match hooks() {
        Some(h) if h.image_decode_start(key, &bytes) => Decode::Host(key),
        #[cfg(not(target_arch = "wasm32"))]
        _ => {
            let (send, recv) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let _ = send.send(decode_native(&bytes, header));
            });
            Decode::Thread(recv)
        }
        #[cfg(target_arch = "wasm32")]
        _ => Decode::Done(Err(
            "this host decodes no images (its Worker installed no image decoder)".into(),
        )),
    };
    Image { header, decode }
}

/// Whether an image's decode has finished; with `wait`, waits for it (a
/// worker thread; a host decode cannot be waited for).
fn decoded(image: &mut Image, h: Option<&dyn Hooks>, wait: bool) -> bool {
    #[cfg(target_arch = "wasm32")]
    let _ = wait;
    match &image.decode {
        Decode::Done(_) => true,
        #[cfg(not(target_arch = "wasm32"))]
        Decode::Thread(recv) => {
            let got = if wait {
                recv.recv().map_err(|_| ())
            } else {
                recv.try_recv().map_err(|_| ())
            };
            match got {
                Ok(r) => {
                    image.decode = Decode::Done(r);
                    true
                }
                Err(()) if wait => {
                    image.decode = Decode::Done(Err("the decoding thread stopped".into()));
                    true
                }
                Err(()) => false,
            }
        }
        Decode::Host(key) => match h.map(|h| h.image_decode_take(*key)) {
            Some(Fetch::Ready(d)) => {
                image.decode = Decode::Done(Ok(d));
                true
            }
            Some(Fetch::Failed(e)) => {
                image.decode = Decode::Done(Err(e));
                true
            }
            Some(Fetch::Pending) => false,
            None => {
                image.decode = Decode::Done(Err("no host is installed to decode images".into()));
                true
            }
        },
    }
}

/// The frame's time at a tick boundary: on the web, work stops once a
/// load's `frameMs` is spent and something was done; natively (and for a
/// replayed load at its tick) it never stops.
struct Clock<'a> {
    now: &'a dyn Fn() -> f64,
    start: f64,
    worked: bool,
}

impl Clock<'_> {
    fn over(&self, frame_ms: f64) -> bool {
        self.worked && (self.now)() - self.start > frame_ms
    }
}

/// Advances a job; whether it is finished (ready to put in place, or
/// failed). `wait` finishes it now.
fn step(job: &mut Job, clock: &mut Clock, h: Option<&dyn Hooks>, wait: bool) -> bool {
    match &mut job.work {
        Work::Image { image, .. } => decoded(image, h, wait),
        Work::Environment(env) => env_step(env, clock, job.frame_ms, wait),
        Work::Gltf(g) => {
            if g.error.is_some() {
                return true;
            }
            let mut images_done = true;
            for (_, image) in &mut g.images {
                images_done &= decoded(image, h, wait);
            }
            // Each primitive converted is a unit of the frame's work.
            while g.prims.len() < g.file.primitive_count() {
                if !wait && clock.over(job.frame_ms) {
                    return false;
                }
                match g.file.primitive(g.prims.len()) {
                    Ok(p) => g.prims.push(p),
                    Err(e) => {
                        g.error = Some(e);
                        return true;
                    }
                }
                clock.worked = true;
            }
            images_done
        }
    }
}

/// At a tick boundary: the finished loads put in place (natively every load
/// called since the last boundary, waited for; on the web those finished,
/// at most a frame budget's worth after the first piece of work), or with a
/// replayed journal each at its recorded tick. `tick` is the world's tick
/// (the next tick is the first to see them).
pub(crate) fn poll(eng: &Engine, tick: u64) {
    let h = hooks();
    let now = || h.as_ref().map_or(0.0, |h| h.now_ms());
    let mut clock = Clock {
        now: &now,
        start: now(),
        worked: false,
    };
    let wait_all = cfg!(not(target_arch = "wasm32"));
    // A released load's host decode is dropped once it lands.
    if let Some(h) = h.as_deref() {
        eng.assets
            .borrow_mut()
            .uncooked
            .discard
            .retain(|&key| matches!(h.image_decode_take(key), Fetch::Pending));
    }
    let mut jobs = std::mem::take(&mut eng.assets.borrow_mut().uncooked.jobs);
    let schedule = eng.assets.borrow().uncooked.schedule.clone();
    let mut kept = Vec::new();
    for mut job in jobs.drain(..) {
        let due = match &schedule {
            Some(s) => match s.get(&job.request) {
                Some(c) if c.tick <= tick => Some(Some(c.ok)),
                _ => None,
            },
            None => Some(None),
        };
        let Some(recorded) = due else {
            kept.push(job);
            continue;
        };
        // A replayed load is waited for at its tick; natively every load
        // is (determinism). On the web the frame budget holds.
        let wait = wait_all || recorded.is_some();
        if (!wait && clock.over(job.frame_ms)) || !step(&mut job, &mut clock, h.as_deref(), wait) {
            kept.push(job);
            continue;
        }
        let request = job.request;
        let ok = apply(eng, job, recorded == Some(false));
        eng.assets
            .borrow_mut()
            .uncooked
            .journal
            .push(Completion { request, tick, ok });
        clock.worked = true;
    }
    eng.assets.borrow_mut().uncooked.jobs.extend(kept);
}

/// A finished decode's image, checked against its header.
fn take_image(image: Image, what: &str) -> Result<Decoded, String> {
    let Decode::Done(result) = image.decode else {
        unreachable!("only finished jobs are put in place");
    };
    let d = result.map_err(|e| format!("{what}: {e}"))?;
    let h = image.header;
    if (d.width, d.height) != (h.width, h.height)
        || d.rgba.len() != (d.width * d.height * 4) as usize
    {
        return Err(format!(
            "{what}: decoded to {}x{}, but its header says {}x{}",
            d.width, d.height, h.width, h.height
        ));
    }
    Ok(d)
}

/// Puts a finished job in place (its entry ready), or its entry failed
/// (`recorded_failure`: it failed in the run a replay follows); whether it
/// is ready.
fn apply(eng: &Engine, job: Job, recorded_failure: bool) -> bool {
    let name = job.name;
    let result = if recorded_failure {
        Err(format!("{name}: failed in the recorded run"))
    } else {
        match job.work {
            Work::Image {
                image,
                pixel,
                linear,
            } => take_image(image, &name).map(|d| {
                let mut rs = eng.render.borrow_mut();
                let handle = rs.scene.add_texture(og_render::TextureData {
                    width: d.width,
                    height: d.height,
                    rgba: d.rgba,
                    color_space: if linear {
                        og_render::ColorSpace::Linear
                    } else {
                        og_render::ColorSpace::Srgb
                    },
                    mips: og_render::Mips::Generate,
                });
                if pixel {
                    rs.scene
                        .set_texture_filter(handle, og_render::TextureFilter::Nearest);
                }
                rs.textures.push(handle);
                let texture = rs.textures.len() - 1;
                drop(rs);
                eng.assets.borrow_mut().entries[job.entry].payload = Payload::Image {
                    texture,
                    width: d.width,
                    height: d.height,
                };
            }),
            Work::Gltf(g) => put_gltf(eng, &name, job.entry, *g),
            Work::Environment(env) => put_environment(eng, &name, job.entry, *env),
        }
    };
    match result {
        Ok(()) => {
            eng.assets.borrow_mut().entries[job.entry].status = Status::Ready;
            true
        }
        Err(e) => {
            let reason = if e.starts_with(&name) {
                e
            } else {
                format!("{name}: {e}")
            };
            eng.assets.borrow_mut().entries[job.entry].status = Status::Failed(reason);
            false
        }
    }
}

/// A converted glTF and its decoded images into the renderer, through the
/// path a cooked model takes ([`super::assets::insert_model`]).
fn put_gltf(eng: &Engine, name: &str, entry: usize, g: Gltf) -> Result<(), String> {
    if let Some(e) = g.error {
        return Err(e);
    }
    let images = g
        .images
        .into_iter()
        .map(|(what, image)| take_image(image, &what))
        .collect::<Result<Vec<_>, _>>()?;
    use og_bundle::format::kind;
    let textures: Vec<og_render::TextureSource> = g
        .file
        .textures
        .iter()
        .map(|(t, i)| {
            let d = &images[*i];
            og_render::TextureSource::Rgba(og_render::TextureData {
                width: d.width,
                height: d.height,
                rgba: d.rgba.clone(),
                color_space: match t.kind {
                    kind::COLOR | kind::PIXEL => og_render::ColorSpace::Srgb,
                    _ => og_render::ColorSpace::Linear,
                },
                mips: match t.kind {
                    kind::PIXEL => og_render::Mips::None,
                    _ => og_render::Mips::Generate,
                },
            })
        })
        .collect();
    let model = g.file.finish(g.prims);
    let (_, handles) =
        super::assets::insert_model(eng, name, None, model, &textures, None, Some(entry), false)?;
    if let Some(owned) = eng.assets.borrow_mut().uncooked.owned.get_mut(&entry) {
        *owned = handles;
    }
    Ok(())
}

/// What a released texture is replaced with, under its handle: one clear
/// texel, so whatever still names it draws nothing and holds nothing.
fn released_texture() -> og_render::TextureData {
    og_render::TextureData {
        width: 1,
        height: 1,
        rgba: vec![0; 4],
        color_space: og_render::ColorSpace::Srgb,
        mips: og_render::Mips::None,
    }
}

/// `og.load.release`: frees an uncooked asset. A pending load is dropped
/// (it never completes, so it is never journaled); a ready image's texture
/// and a ready model's meshes and textures are emptied under their handles,
/// so nothing that still names them breaks or draws, and their GPU and CPU
/// copies go; the model's draw parts and cached raycast BVHs go too. The
/// handle then answers failed. Physics shapes built from its geometries are
/// world state and stay. Releasing again is a no-op.
pub(crate) fn release(eng: &Engine, index: Option<usize>) -> Result<(), String> {
    let i = index.ok_or("not an asset handle")?;
    let mut a = eng.assets.borrow_mut();
    let path = a.entries.get(i).ok_or("not an asset handle")?.path.clone();
    let released = format!("{path} was released (og.load.release)");
    if a.entries[i].status == Status::Failed(released.clone()) {
        return Ok(());
    }
    let Some(textures) = a.uncooked.owned.remove(&i) else {
        return Err(format!(
            "{path} is a cooked asset: og.load.release frees what og.load.imageData, og.load.gltfData and og.load.environmentData loaded (a cooked asset stays with its bundle, a streamed one with its group)"
        ));
    };
    if let Some(at) = a.uncooked.jobs.iter().position(|j| j.entry == i) {
        let job = a.uncooked.jobs.remove(at);
        let images: Vec<Image> = match job.work {
            Work::Image { image, .. } => vec![image],
            Work::Gltf(g) => g.images.into_iter().map(|(_, image)| image).collect(),
            Work::Environment(_) => Vec::new(),
        };
        for image in images {
            if let Decode::Host(key) = image.decode {
                a.uncooked.discard.push(key);
            }
        }
    }
    let payload = std::mem::replace(&mut a.entries[i].payload, Payload::Uncooked);
    a.entries[i].status = Status::Failed(released);
    drop(a);
    let mut rs = eng.render.borrow_mut();
    let rs = &mut *rs;
    match payload {
        Payload::Image { texture, .. } => {
            if let Some(&h) = rs.textures.get(texture) {
                rs.scene.set_texture(h, released_texture());
            }
        }
        Payload::Environment { map, .. } => {
            if let Some(h) = rs.env.maps.get_mut(map).and_then(|m| m.1.take()) {
                // The frame stops naming it now; og.Environment's next read
                // finds no map there and lights by the sky.
                if rs.scene.environment_map.is_some_and(|m| m.map == h) {
                    rs.scene.environment_map = None;
                }
                if let Some(m) = &mut rs.scene.environment_map
                    && m.blend.is_some_and(|b| b.to == h)
                {
                    m.blend = None;
                }
                rs.scene.remove_environment_map(h);
            }
        }
        Payload::Gltf { .. } => {
            let key = super::assets::handle(i);
            let parts = rs.models.insert(key, Vec::new()).unwrap_or_default();
            rs.model_lods.remove(&key);
            rs.fx.models.remove(&key);
            let mut meshes: Vec<og_render::MeshHandle> = parts.iter().map(|p| p.0).collect();
            meshes.sort_by_key(|m| m.index());
            meshes.dedup();
            for m in meshes {
                rs.scene.set_mesh(m, og_render::MeshData::default());
                super::geometry::forget_mesh(rs, m);
            }
            for h in textures {
                rs.scene.set_texture(h, released_texture());
                rs.decoded.ids.remove(&h);
            }
        }
        _ => {}
    }
    Ok(())
}

/// `og.load.imageInfo`.
pub(crate) fn image_info(eng: &Engine, index: Option<usize>) -> Result<Value, String> {
    let a = eng.assets.borrow();
    let e = index
        .and_then(|i| a.entries.get(i))
        .ok_or("not an asset handle")?;
    match (&e.status, &e.payload) {
        (Status::Failed(r), _) => Err(r.clone()),
        (Status::Pending, Payload::Uncooked) => Err(format!(
            "{} is still loading: og.load.status answers ready once it is in place",
            e.path
        )),
        (
            Status::Ready,
            Payload::Image {
                texture,
                width,
                height,
            },
        ) => Ok(
            json!({ "texture": super::assets::handle(*texture) as f64, "width": width, "height": height }),
        ),
        _ => Err(format!(
            "{} is not an image og.load.imageData loaded",
            e.path
        )),
    }
}

/// `og.load.completions`.
pub(crate) fn completions(eng: &Engine) -> Value {
    Value::Array(
        eng.assets
            .borrow()
            .uncooked
            .journal
            .iter()
            .map(|c| json!({ "request": c.request, "tick": c.tick as f64, "ok": c.ok }))
            .collect(),
    )
}

/// `og.load.replayCompletions`: refused once a tick has run.
pub(crate) fn replay(eng: &Engine, tick: u64, list: &Value) -> Result<(), String> {
    if tick > 0 {
        return Err(format!(
            "og.load.replayCompletions is for a world that has not ticked; this one is at tick {tick}"
        ));
    }
    let items = list
        .as_array()
        .ok_or("completions is a list of { request, tick, ok }")?;
    let mut schedule = BTreeMap::new();
    for (i, c) in items.iter().enumerate() {
        let num = |k: &str| {
            c[k].as_f64()
                .filter(|v| *v >= 0.0 && v.fract() == 0.0)
                .ok_or_else(|| format!("completions[{i}].{k} is a whole number"))
        };
        let request = num("request")? as u32;
        let tick = num("tick")? as u64;
        let ok = c["ok"]
            .as_bool()
            .ok_or_else(|| format!("completions[{i}].ok is a boolean"))?;
        schedule.insert(request, Completion { request, tick, ok });
    }
    eng.assets.borrow_mut().uncooked.schedule = Some(schedule);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png(width: u32, height: u32) -> Vec<u8> {
        let mut b = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 13];
        b.extend_from_slice(b"IHDR");
        b.extend_from_slice(&width.to_be_bytes());
        b.extend_from_slice(&height.to_be_bytes());
        b.extend_from_slice(&[8, 6, 0, 0, 0]);
        b
    }

    #[test]
    fn an_image_over_its_budget_is_refused_from_its_header_naming_the_limit() {
        let budget =
            Budget::from_desc(&json!({ "budget": { "textureSize": 512, "bytes": 1000 } })).unwrap();
        // A PNG claiming 4096x16: refused before anything decodes, whatever follows the header.
        let e = check_image("cat.png", &png(4096, 16), &budget).unwrap_err();
        assert!(
            e.contains("4096x16 is over LoadBudget.textureSize (512)"),
            "{e}"
        );
        assert_eq!(
            check_image("cat.png", &png(512, 300), &budget).unwrap(),
            ImageHeader {
                format: "png",
                width: 512,
                height: 300
            }
        );
        let e = check_image("big.png", &[png(8, 8), vec![0; 2000]].concat(), &budget).unwrap_err();
        assert!(
            e.contains("2029 bytes is over LoadBudget.bytes (1000)"),
            "{e}"
        );
        assert!(
            check_image("x.gif", b"GIF89a....", &budget)
                .unwrap_err()
                .contains("not a PNG or JPEG")
        );
        // textureSize never passes what every device takes.
        assert_eq!(
            Budget::from_desc(&json!({ "budget": { "textureSize": 100000 } }))
                .unwrap()
                .texture_size,
            8192
        );
    }

    #[test]
    fn a_gltfs_primitives_are_converted_a_frame_budget_at_a_time_at_least_one_a_frame() {
        // Five one-triangle primitives; every reading of the clock is 1.5 ms later.
        let prim = r#"{"attributes":{"POSITION":0}}"#;
        let gltf = format!(
            r#"{{"asset":{{"version":"2.0"}},"meshes":[{{"primitives":[{p},{p},{p},{p},{p}]}}],
              "buffers":[{{"uri":"data:application/octet-stream;base64,AAAAAAAAAAAAAAAAAACAPwAAAAAAAAAAAAAAAAAAgD8AAAAA","byteLength":36}}],
              "bufferViews":[{{"buffer":0,"byteLength":36}}],
              "accessors":[{{"bufferView":0,"componentType":5126,"count":3,"type":"VEC3"}}]}}"#,
            p = prim
        );
        let job = |frame_ms: f64| Job {
            request: 0,
            entry: 0,
            name: "m".into(),
            frame_ms,
            work: Work::Gltf(Box::new(Gltf {
                file: og_bundle::runtime::UncookedGltf::read("m", gltf.as_bytes()).unwrap(),
                images: Vec::new(),
                prims: Vec::new(),
                error: None,
            })),
        };
        let frames = |mut job: Job, wait: bool| {
            let t = std::cell::Cell::new(0.0);
            let now = || {
                let v = t.get();
                t.set(v + 1.5);
                v
            };
            let mut converted = Vec::new();
            for _ in 0..10 {
                let mut clock = Clock {
                    now: &now,
                    start: now(),
                    worked: false,
                };
                let done = step(&mut job, &mut clock, None, wait);
                let Work::Gltf(g) = &job.work else {
                    unreachable!()
                };
                converted.push(g.prims.len());
                if done {
                    break;
                }
            }
            converted
        };
        // 2 ms a frame: the first primitive, then one more while under budget.
        assert_eq!(frames(job(2.0), false), vec![2, 4, 5]);
        // No budget left at all still makes progress: one a frame.
        assert_eq!(frames(job(0.0), false), vec![1, 2, 3, 4, 5]);
        // Waited for (natively, or a replayed load at its tick): all at once.
        assert_eq!(frames(job(0.0), true), vec![5]);
    }

    /// A .hdr decoded in rows at the boundaries (the web's way): a frame
    /// budget's worth of rows a frame and at least one, all at once when
    /// waited for, and the result whatever the pace: the texels packed and
    /// the irradiance of a uniform sky (its first harmonic only).
    #[test]
    fn an_environment_decodes_a_frame_budget_of_rows_at_a_time() {
        let mut file = b"#?RADIANCE\n\n-Y 5 +X 10\n".to_vec();
        // Every pixel (128, 64, 32) at exponent 129.
        for _ in 0..50 {
            file.extend_from_slice(&[128, 64, 32, 129]);
        }
        let env = || {
            let rows = og_bundle::runtime::HdrRows::new(file.clone()).unwrap();
            Env {
                size: 16,
                decode: EnvDecode::Rows {
                    packer: Packer::new(rows.width(), rows.height(), 2.0),
                    rows,
                    row: Vec::new(),
                },
            }
        };
        let frames = |mut env: Env, frame_ms: f64, wait: bool| {
            let t = std::cell::Cell::new(0.0);
            let now = || {
                let v = t.get();
                t.set(v + 1.5);
                v
            };
            let mut seen = Vec::new();
            for _ in 0..10 {
                let mut clock = Clock {
                    now: &now,
                    start: now(),
                    worked: false,
                };
                let done = env_step(&mut env, &mut clock, frame_ms, wait);
                seen.push(match &env.decode {
                    EnvDecode::Rows { rows, .. } => rows.row(),
                    _ => 5,
                });
                if done {
                    break;
                }
            }
            let EnvDecode::Done(Ok(p)) = env.decode else {
                panic!("not decoded")
            };
            (seen, p)
        };
        assert_eq!(frames(env(), 2.0, false).0, vec![2, 4, 5]);
        assert_eq!(frames(env(), 0.0, false).0, vec![1, 2, 3, 4, 5]);
        let (seen, p) = frames(env(), 0.0, true);
        assert_eq!(seen, vec![5]);
        assert_eq!((p.width, p.height, p.texels.len()), (10, 5, 50));
        // Intensity 2: (128.5, 64.5, 32.5) / 128 doubled.
        let c = og_render::rgb9e5_decode(p.texels[0]);
        assert!((c[0] - 2.0 * 128.5 / 128.0).abs() < 0.01, "{c:?}");
        // A uniform radiance L has irradiance over pi L: Y_0 times
        // 4 pi Y_0 L, and nothing in the higher bands.
        let l = 2.0 * 64.5 / 128.0;
        let y0 = 0.282095f32;
        // Within the quadrature error of five rows (a few per cent).
        let want = 4.0 * std::f32::consts::PI * y0 * l;
        assert!(
            (p.irradiance[0][1] - want).abs() < 0.03 * want,
            "{:?} against {want}",
            p.irradiance[0]
        );
        assert!(
            p.irradiance[1..]
                .iter()
                .flatten()
                .all(|v| v.abs() < 0.05 * want),
            "{:?}",
            p.irradiance
        );
    }

    /// A pixel that is not a finite number refuses the image.
    #[test]
    fn a_non_finite_pixel_refuses_an_environment() {
        let mut p = Packer::new(2, 1, 1.0);
        let e = p
            .row(0, &mut [1.0, 1.0, 1.0, f32::NAN, 0.0, 0.0])
            .unwrap_err();
        assert!(e.contains("pixel 1 of row 0"), "{e}");
    }

    #[test]
    fn a_jpeg_header_is_read_past_its_other_segments() {
        // SOI, an APP0 of 16 bytes, a DHT, then SOF2 (progressive) of 640x480.
        let mut j = vec![0xff, 0xd8, 0xff, 0xe0, 0, 16];
        j.extend_from_slice(&[0; 14]);
        j.extend_from_slice(&[0xff, 0xc4, 0, 3, 0]);
        j.extend_from_slice(&[0xff, 0xc2, 0, 11, 8, 0x01, 0xe0, 0x02, 0x80, 3, 0, 0, 0]);
        assert_eq!(
            image_header(&j).unwrap(),
            ImageHeader {
                format: "jpeg",
                width: 640,
                height: 480
            }
        );
    }
}
