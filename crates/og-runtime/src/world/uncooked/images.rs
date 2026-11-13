//! Images: decoded on a worker thread or by the host, then put as a texture.

use super::*;

/// A decoded image: RGBA8, rows top to bottom.
pub struct Decoded {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Where an image's decode is.
pub(super) enum Decode {
    /// On a worker thread of this process.
    #[cfg(not(target_arch = "wasm32"))]
    Thread(std::sync::mpsc::Receiver<Result<Decoded, String>>),
    /// With the host, by key ([`Hooks::image_decode_take`]).
    Host(u64),
    Done(Result<Decoded, String>),
}

/// An image to decode: what its header said and how its texels are read.
pub(super) struct Image {
    pub(super) header: ImageHeader,
    pub(super) decode: Decode,
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn decode_native(bytes: &[u8], header: ImageHeader) -> Result<Decoded, String> {
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
    Ok(queue(eng, name, &budget, |u| Work::Image {
        image: start(u, bytes, header),
        pixel,
        linear,
    }))
}

pub(super) fn start(u: &mut Uncooked, bytes: Vec<u8>, header: ImageHeader) -> Image {
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
pub(super) fn decoded(image: &mut Image, h: Option<&dyn Hooks>, wait: bool) -> bool {
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

/// A finished decode's image, checked against its header.
pub(super) fn take_image(image: Image, what: &str) -> Result<Decoded, String> {
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

/// What a released texture is replaced with, under its handle: one clear
/// texel, so whatever still names it draws nothing and holds nothing.
pub(super) fn released_texture() -> og_render::TextureData {
    og_render::TextureData {
        width: 1,
        height: 1,
        rgba: vec![0; 4],
        color_space: og_render::ColorSpace::Srgb,
        mips: og_render::Mips::None,
    }
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
            json!({ "texture": crate::world::assets::handle(*texture) as f64, "width": width, "height": height }),
        ),
        _ => Err(format!(
            "{} is not an image og.load.imageData loaded",
            e.path
        )),
    }
}
