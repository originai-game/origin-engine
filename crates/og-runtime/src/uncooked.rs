//! Uncooked loads at run time (IDL `load`: `imageData`, `imageInfo`,
//! `completions`, `replayCompletions`): content a player or a generator made,
//! from bytes, without `og cook`.
//!
//! The source is untrusted, so the budget (`LoadBudget`) is checked from its
//! headers before anything is decoded. Decoding never runs on the engine's
//! thread on the web: the host decodes (the Worker's `createImageBitmap`,
//! [`Hooks::image_decode_start`]); natively a worker thread does, with the
//! `image` crate. No decoder is in the wasm.
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
    Err("not a PNG or JPEG (og.load.imageData reads those two; convert other formats first)".into())
}

/// An image's header checked against a budget: the reasons name the limit,
/// the value and the fix.
pub(crate) fn check_image(
    name: &str,
    bytes: &[u8],
    budget: &Budget,
) -> Result<ImageHeader, String> {
    if bytes.len() as f64 > budget.bytes {
        return Err(format!(
            "{name}: {} bytes is over LoadBudget.bytes ({}); make the file smaller or raise the budget",
            bytes.len(),
            budget.bytes
        ));
    }
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

/// Where a load's decode is.
enum Decode {
    /// On a worker thread of this process.
    #[cfg(not(target_arch = "wasm32"))]
    Thread(std::sync::mpsc::Receiver<Result<Decoded, String>>),
    /// With the host, by key ([`Hooks::image_decode_take`]).
    Host(u64),
    Done(Result<Decoded, String>),
}

struct Job {
    request: u32,
    /// Its entry in the engine's assets.
    entry: usize,
    name: String,
    header: ImageHeader,
    pixel: bool,
    linear: bool,
    frame_ms: f64,
    decode: Decode,
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
    let decode = start(u, bytes, header);
    u.jobs.push(Job {
        request,
        entry,
        name: name.to_string(),
        header,
        pixel,
        linear,
        frame_ms: budget.frame_ms,
        decode,
    });
    Ok(entry)
}

fn start(u: &mut Uncooked, bytes: Vec<u8>, header: ImageHeader) -> Decode {
    let key = u.next_key;
    u.next_key += 1;
    if let Some(h) = hooks()
        && h.image_decode_start(key, &bytes)
    {
        return Decode::Host(key);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let (send, recv) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = send.send(decode_native(&bytes, header));
        });
        Decode::Thread(recv)
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = header;
        Decode::Done(Err(
            "this host decodes no images (its Worker installed no image decoder)".into(),
        ))
    }
}

/// Whether a job's decode has finished; with `wait`, waits for it (a
/// worker thread; a host decode cannot be waited for).
fn finished(job: &mut Job, h: Option<&dyn Hooks>, wait: bool) -> bool {
    #[cfg(target_arch = "wasm32")]
    let _ = wait;
    match &job.decode {
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
                    job.decode = Decode::Done(r);
                    true
                }
                Err(()) if wait => {
                    job.decode = Decode::Done(Err("the decoding thread stopped".into()));
                    true
                }
                Err(()) => false,
            }
        }
        Decode::Host(key) => match h.map(|h| h.image_decode_take(*key)) {
            Some(Fetch::Ready(d)) => {
                job.decode = Decode::Done(Ok(d));
                true
            }
            Some(Fetch::Failed(e)) => {
                job.decode = Decode::Done(Err(e));
                true
            }
            Some(Fetch::Pending) => false,
            None => {
                job.decode = Decode::Done(Err("no host is installed to decode images".into()));
                true
            }
        },
    }
}

/// At a tick boundary: the finished loads put in place (natively every load
/// called since the last boundary, waited for; on the web those decoded, at
/// most a frame budget's worth after the first), or with a replayed journal
/// each at its recorded tick. `tick` is the world's tick (the next tick is
/// the first to see them).
pub(crate) fn poll(eng: &Engine, tick: u64) {
    let h = hooks();
    let start = h.as_ref().map_or(0.0, |h| h.now_ms());
    let wait_all = cfg!(not(target_arch = "wasm32"));
    let mut jobs = std::mem::take(&mut eng.assets.borrow_mut().uncooked.jobs);
    let schedule = eng.assets.borrow().uncooked.schedule.clone();
    let mut kept = Vec::new();
    let mut applied = 0;
    for mut job in jobs.drain(..) {
        let due = match &schedule {
            Some(s) => match s.get(&job.request) {
                Some(c) if c.tick <= tick => Some(Some(c.ok)),
                _ => None,
            },
            None => Some(None),
        };
        // The frame budget holds on the web; natively every load is put in place (determinism).
        let over = !wait_all
            && applied > 0
            && h.as_ref()
                .is_some_and(|h| h.now_ms() - start > job.frame_ms);
        let Some(recorded) = due else {
            kept.push(job);
            continue;
        };
        // A replayed load is waited for at its tick; natively every load is.
        let wait = wait_all || recorded.is_some();
        if (over && recorded.is_none()) || !finished(&mut job, h.as_deref(), wait) {
            kept.push(job);
            continue;
        }
        let Decode::Done(result) =
            std::mem::replace(&mut job.decode, Decode::Done(Err(String::new())))
        else {
            unreachable!("finished");
        };
        let result = match recorded {
            Some(false) => Err(format!("{}: failed in the recorded run", job.name)),
            _ => result,
        };
        let ok = apply(eng, &job, result);
        eng.assets.borrow_mut().uncooked.journal.push(Completion {
            request: job.request,
            tick,
            ok,
        });
        applied += 1;
    }
    eng.assets.borrow_mut().uncooked.jobs.extend(kept);
}

/// Puts a decoded image in the renderer and its entry ready, or the entry
/// failed; whether it is ready.
fn apply(eng: &Engine, job: &Job, decoded: Result<Decoded, String>) -> bool {
    let checked = decoded.and_then(|d| {
        if (d.width, d.height) != (job.header.width, job.header.height)
            || d.rgba.len() != (d.width * d.height * 4) as usize
        {
            return Err(format!(
                "{}: decoded to {}x{}, but its header says {}x{}",
                job.name, d.width, d.height, job.header.width, job.header.height
            ));
        }
        Ok(d)
    });
    let status = match checked {
        Ok(d) => {
            let mut rs = eng.render.borrow_mut();
            let handle = rs.scene.add_texture(og_render::TextureData {
                width: d.width,
                height: d.height,
                rgba: d.rgba,
                color_space: if job.linear {
                    og_render::ColorSpace::Linear
                } else {
                    og_render::ColorSpace::Srgb
                },
                mips: og_render::Mips::Generate,
            });
            if job.pixel {
                rs.scene
                    .set_texture_filter(handle, og_render::TextureFilter::Nearest);
            }
            rs.textures.push(handle);
            let texture = rs.textures.len() - 1;
            drop(rs);
            let mut a = eng.assets.borrow_mut();
            a.entries[job.entry].payload = Payload::Image {
                texture,
                width: d.width,
                height: d.height,
            };
            Status::Ready
        }
        Err(e) => {
            let reason = if e.starts_with(&job.name) {
                e
            } else {
                format!("{}: {e}", job.name)
            };
            Status::Failed(reason)
        }
    };
    let ok = status == Status::Ready;
    eng.assets.borrow_mut().entries[job.entry].status = status;
    ok
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
