//! The http(s) images documents and style packs name in `url()`, by
//! og-ui's request key (`og_ui::css::request`, docs/ui.md §3.3 Remote
//! images): fetched once per engine, decoded, and bound in the UI as a
//! texture under that key.
//!
//! - **Who fetches.** On the web the host does, from the engine Worker:
//!   `fetch()` in CORS mode with the request's credentials, integrity and
//!   referrer policy, then the browser's image decoder
//!   ([`Hooks::remote_image_start`], answered through
//!   [`Hooks::image_decode_take`]). Natively a worker thread does both:
//!   og-net's HTTP client under the same CORS, integrity and referrer rules,
//!   then the `image` crate (PNG, JPEG, WebP) or resvg (SVG, at its own size,
//!   as the cook draws a package's SVG). A build without the `fetch`
//!   feature fetches nothing natively and says so once.
//! - **Arrival is presentation.** A layer naming a key not bound yet draws
//!   nothing (og-ui), and an image is bound at the first UI build after it
//!   lands. Nothing fetched reaches the world: no tick waits for one, and
//!   a load's arrival is never journaled.
//! - **Executors wait.** The native host waits at each UI build for every
//!   fetch its documents started, as it waits for a video's frame, so its
//!   frames and captures are the same on every run. The web draws what has
//!   landed.
//! - **Failures are warnings**, one per key, naming the URL and what
//!   refused it (CORS, integrity, an HTTP status, the decoder); the layer
//!   keeps drawing nothing.
//! - **Frames' documents.** A remote `<iframe>` (og_ui::doc::frames)
//!   names its document by request key as an image is named; it is fetched
//!   the same way (the bytes only: the Worker's `fetch()`, natively og-net),
//!   only from an origin project.ogt's `frames` lists, at most
//!   `MAX_SOURCE` bytes, and defined in the UI under its key
//!   (`Ui::define_frame_document`). The images and frames it names are
//!   fetched in turn, frames at most `MAX_DEPTH` deep; an executor waits
//!   for the whole chain.
//! - **The game's own URL.** The origin a request is made from and the
//!   referrer it is taken from are the engine's root when it is an http(s)
//!   URL (a package served from an asset base, docs/web-export.md §Asset
//!   base); a game read from files has an opaque origin (`null`) and sends
//!   no referrer.

use crate::engine::{Engine, hooks};
use crate::world::assets::Fetch;
#[cfg(not(target_arch = "wasm32"))]
use crate::world::uncooked::Decoded;
use og_ui::css::request::UrlRequest;
use std::collections::BTreeMap;

/// Every remote image an engine was asked for, by request key.
#[derive(Default)]
pub(crate) struct Remote {
    images: BTreeMap<String, Image>,
    /// Remote frames' documents, by request key, with how deep each is.
    documents: BTreeMap<String, (Text, usize)>,
}

enum Text {
    Host(u64),
    #[cfg(not(target_arch = "wasm32"))]
    Thread(std::sync::mpsc::Receiver<Result<Vec<u8>, String>>),
    Done,
}

enum Image {
    /// With the host, answered under this key.
    Host(u64),
    /// On a worker thread of this process.
    #[cfg(not(target_arch = "wasm32"))]
    Thread(std::sync::mpsc::Receiver<Result<Decoded, String>>),
    /// Bound in the UI under its key.
    Bound,
    Failed,
}

/// Keys of host decodes this module starts: above og.load.imageData's
/// (which count up from 0 per engine), unique across the thread's engines.
fn next_key() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(1 << 62);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

/// Starts fetching each of `keys` this engine has not asked for yet.
pub(crate) fn request<'a>(eng: &Engine, keys: impl IntoIterator<Item = &'a String>) {
    let root = eng.assets.borrow().root.clone();
    let mut a = eng.assets.borrow_mut();
    for key in keys {
        if a.remote.images.contains_key(key) {
            continue;
        }
        let image = match UrlRequest::from_key(key) {
            Ok(request) => start(&root, &request),
            Err(e) => Err(e),
        };
        let image = image.unwrap_or_else(|e| {
            drop_warning(eng, key, &e);
            Image::Failed
        });
        a.remote.images.insert(key.clone(), image);
    }
}

/// Starts fetching each remote frame's document of `keys` not asked for
/// yet, `depth` frames deep (1: in a document of the game). One from an
/// origin the project does not list, or nested too deep, warns and is
/// never fetched: its frame stays empty.
pub(crate) fn request_frames(eng: &Engine, keys: &[String], depth: usize) {
    let allowed = eng
        .state
        .borrow()
        .project
        .as_ref()
        .map(|p| p.frames.clone())
        .unwrap_or_default();
    let root = eng.assets.borrow().root.clone();
    for key in keys {
        if eng.assets.borrow().remote.documents.contains_key(key) {
            continue;
        }
        let started = if !og_ui::doc::frames::allowed(key, &allowed) {
            Err("its origin is not in project.ogt's `frames`".to_string())
        } else if depth > og_ui::doc::frames::MAX_DEPTH {
            Err(format!(
                "frames nest at most {} deep",
                og_ui::doc::frames::MAX_DEPTH
            ))
        } else {
            UrlRequest::from_key(key).and_then(|r| start_text(&root, &r))
        };
        let text = started.unwrap_or_else(|e| {
            frame_warning(eng, key, &e);
            Text::Done
        });
        eng.assets
            .borrow_mut()
            .remote
            .documents
            .insert(key.clone(), (text, depth));
    }
}

fn start_text(root: &str, request: &UrlRequest) -> Result<Text, String> {
    let key = next_key();
    if let Some(h) = hooks()
        && h.remote_text_start(key, request)
    {
        return Ok(Text::Host(key));
    }
    let _ = root;
    #[cfg(all(feature = "fetch", not(target_arch = "wasm32")))]
    {
        let (send, recv) = std::sync::mpsc::channel();
        let request = request.clone();
        let root = root.to_owned();
        std::thread::spawn(move || {
            let _ = send.send(native::fetch(&root, &request));
        });
        Ok(Text::Thread(recv))
    }
    #[cfg(not(all(feature = "fetch", not(target_arch = "wasm32"))))]
    Err("this build fetches no remote documents (og-runtime's `fetch` feature)".into())
}

/// Defines every frame document that has landed (natively, every one
/// started, and those they name in turn) and starts what each names.
fn settle_documents(eng: &Engine, wait: bool) {
    loop {
        let waiting: Vec<String> = eng
            .assets
            .borrow()
            .remote
            .documents
            .iter()
            .filter(|(_, (t, _))| !matches!(t, Text::Done))
            .map(|(k, _)| k.clone())
            .collect();
        if waiting.is_empty() {
            return;
        }
        let h = hooks();
        let mut landed = Vec::new();
        {
            let mut a = eng.assets.borrow_mut();
            for key in waiting {
                let Some((text, depth)) = a.remote.documents.get_mut(&key) else {
                    continue;
                };
                let got = match text {
                    Text::Host(k) => match h.as_deref().map(|h| h.fetch_take(*k)) {
                        Some(Fetch::Ready(b)) => Some(Ok(b)),
                        Some(Fetch::Failed(e)) => Some(Err(e)),
                        Some(Fetch::Pending) => None,
                        None => Some(Err("no host is installed to fetch it".into())),
                    },
                    #[cfg(not(target_arch = "wasm32"))]
                    Text::Thread(recv) => match if wait {
                        recv.recv().ok()
                    } else {
                        recv.try_recv().ok()
                    } {
                        Some(r) => Some(r),
                        None if wait => Some(Err("the fetching thread stopped".into())),
                        None => None,
                    },
                    Text::Done => None,
                };
                if let Some(got) = got {
                    *text = Text::Done;
                    landed.push((key, *depth, got));
                }
            }
        }
        if landed.is_empty() {
            return;
        }
        for (key, depth, got) in landed {
            let defined = got.and_then(|bytes| {
                if bytes.len() > og_ui::doc::frames::MAX_SOURCE {
                    return Err(format!(
                        "its source is past {} KiB",
                        og_ui::doc::frames::MAX_SOURCE / 1024
                    ));
                }
                let text =
                    String::from_utf8(bytes).map_err(|_| "it is not UTF-8 text".to_string())?;
                let ui = eng.state.borrow().ui.clone().ok_or("the world has no UI")?;
                ui.lock().define_frame_document(&key, &text)
            });
            match defined {
                Ok((images, frames)) => {
                    request(eng, &images);
                    request_frames(eng, &frames, depth + 1);
                }
                Err(e) => frame_warning(eng, &key, &e),
            }
        }
        let _ = wait;
    }
}

fn frame_warning(eng: &Engine, key: &str, why: &str) {
    let url = key.split(' ').next().unwrap_or(key);
    let tick = eng.world().map(|w| w.time().tick).unwrap_or(0);
    eng.state.borrow_mut().log.push(serde_json::json!({
        "tick": tick,
        "warning": true,
        "text": format!("og.ui: <iframe src=\"{url}\"> did not load: {why}; it shows nothing"),
    }));
}

fn start(root: &str, request: &UrlRequest) -> Result<Image, String> {
    let key = next_key();
    if let Some(h) = hooks()
        && h.remote_image_start(key, request)
    {
        return Ok(Image::Host(key));
    }
    let _ = root;
    #[cfg(all(feature = "fetch", not(target_arch = "wasm32")))]
    {
        let (send, recv) = std::sync::mpsc::channel();
        let request = request.clone();
        let root = root.to_owned();
        std::thread::spawn(move || {
            let _ = send.send(native::fetch(&root, &request).and_then(|b| native::decode(&b)));
        });
        Ok(Image::Thread(recv))
    }
    #[cfg(not(all(feature = "fetch", not(target_arch = "wasm32"))))]
    Err("this build fetches no remote images (og-runtime's `fetch` feature)".into())
}

/// At a UI build: binds every image that has landed (natively, every one
/// started: an executor waits for them) and warns of every one that
/// failed.
pub(crate) fn settle(eng: &Engine) {
    let wait = cfg!(not(target_arch = "wasm32"));
    // A frame's document first: the images it names are then started and
    // (natively) waited for below.
    settle_documents(eng, wait);
    let waiting: Vec<String> = eng
        .assets
        .borrow()
        .remote
        .images
        .iter()
        .filter(|(_, i)| !matches!(i, Image::Bound | Image::Failed))
        .map(|(k, _)| k.clone())
        .collect();
    if waiting.is_empty() {
        return;
    }
    let h = hooks();
    let mut landed = Vec::new();
    {
        let mut a = eng.assets.borrow_mut();
        for key in waiting {
            let Some(image) = a.remote.images.get_mut(&key) else {
                continue;
            };
            let got = match image {
                Image::Host(k) => match h.as_deref().map(|h| h.image_decode_take(*k)) {
                    Some(Fetch::Ready(d)) => Some(Ok(d)),
                    Some(Fetch::Failed(e)) => Some(Err(e)),
                    Some(Fetch::Pending) => None,
                    None => Some(Err("no host is installed to fetch it".into())),
                },
                #[cfg(not(target_arch = "wasm32"))]
                Image::Thread(recv) => {
                    let got = if wait {
                        recv.recv().ok()
                    } else {
                        recv.try_recv().ok()
                    };
                    match got {
                        Some(r) => Some(r),
                        None if wait => Some(Err("the fetching thread stopped".into())),
                        None => None,
                    }
                }
                Image::Bound | Image::Failed => None,
            };
            if let Some(got) = got {
                *image = Image::Failed;
                landed.push((key, got));
            }
        }
    }
    let _ = wait;
    for (key, got) in landed {
        match got {
            Ok(d) => {
                let mut rs = eng.render.borrow_mut();
                let handle = rs.scene.add_texture(og_render::TextureData {
                    width: d.width,
                    height: d.height,
                    rgba: d.rgba,
                    color_space: og_render::ColorSpace::Srgb,
                    mips: og_render::Mips::Generate,
                });
                // Stored as sent; CSS image-orientation turns it at draw time.
                rs.scene.set_texture_orientation(
                    handle,
                    og_render::UiOrientation::from_exif(d.orientation),
                );
                drop(rs);
                if let Some(ui) = eng.state.borrow().ui.clone() {
                    ui.lock().define_texture(&key, handle);
                }
                eng.assets
                    .borrow_mut()
                    .remote
                    .images
                    .insert(key, Image::Bound);
            }
            Err(e) => drop_warning(eng, &key, &e),
        }
    }
}

/// Whether any image or frame's document is still being fetched
/// (og.loading.idle is false).
pub(crate) fn pending(eng: &Engine) -> bool {
    let a = eng.assets.borrow();
    a.remote
        .images
        .values()
        .any(|i| !matches!(i, Image::Bound | Image::Failed))
        || a.remote
            .documents
            .values()
            .any(|(t, _)| !matches!(t, Text::Done))
}

fn drop_warning(eng: &Engine, key: &str, why: &str) {
    let url = key.split(' ').next().unwrap_or(key);
    let why = why
        .strip_prefix(url)
        .map_or(why, |w| w.trim_start_matches([':', ' ']));
    let tick = eng.world().map(|w| w.time().tick).unwrap_or(0);
    eng.state.borrow_mut().log.push(serde_json::json!({
        "tick": tick,
        "warning": true,
        "text": format!("og.ui: url({url}) did not load: {why}; its layer draws nothing"),
    }));
}

/// The EXIF orientation (1 to 8) a PNG (`eXIf`), JPEG (APP1 `Exif`) or
/// WebP (`EXIF`) names for its stored texels, read from its metadata alone;
/// 1 when it names none or is another format. Both hosts read a remote
/// image's with it: natively beside the `image` crate's decode, on the web
/// over the bytes the Worker fetched (og-host-web's
/// `og_web_image_orientation`), whose `createImageBitmap` has turned the
/// texels and so turns them back as stored.
pub fn exif_orientation(bytes: &[u8]) -> u32 {
    let tiff = if bytes.starts_with(&[0xff, 0xd8]) {
        jpeg_exif(bytes)
    } else if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        chunks(
            &bytes[8..],
            |b| {
                let len = u32::from_be_bytes(b.get(..4)?.try_into().ok()?) as usize;
                Some((b.get(4..8)?, 8, len, len + 12))
            },
            b"eXIf",
        )
    } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        chunks(
            &bytes[12..],
            |b| {
                let len = u32::from_le_bytes(b.get(4..8)?.try_into().ok()?) as usize;
                Some((b.get(..4)?, 8, len, 8 + len + (len & 1)))
            },
            b"EXIF",
        )
        // Some writers keep JPEG's `Exif\0\0` before the TIFF header.
        .map(|t| t.strip_prefix(b"Exif\0\0").unwrap_or(t))
    } else {
        None
    };
    tiff.and_then(tiff_orientation)
        .filter(|o| (1..=8).contains(o))
        .unwrap_or(1)
}

/// The first chunk named `want` of a chunked container: `head` reads one
/// chunk's (name, data offset, data length, whole length).
fn chunks<'a>(
    mut b: &'a [u8],
    head: impl Fn(&[u8]) -> Option<(&[u8], usize, usize, usize)>,
    want: &[u8; 4],
) -> Option<&'a [u8]> {
    while let Some((name, at, len, whole)) = head(b) {
        if name == want {
            return b.get(at..at + len);
        }
        b = b.get(whole..)?;
    }
    None
}

/// A JPEG's APP1 `Exif` payload: its TIFF bytes. Stops at the scan.
fn jpeg_exif(bytes: &[u8]) -> Option<&[u8]> {
    let mut i = 2;
    while i + 4 <= bytes.len() {
        if bytes[i] != 0xff {
            return None;
        }
        let marker = bytes[i + 1];
        match marker {
            // Fill bytes and markers without a length.
            0xff => i += 1,
            0x01 | 0xd0..=0xd7 => i += 2,
            0xd9 | 0xda => return None,
            _ => {
                let len = usize::from(u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]));
                let seg = bytes.get(i + 4..i + 2 + len)?;
                if marker == 0xe1
                    && let Some(tiff) = seg.strip_prefix(b"Exif\0\0")
                {
                    return Some(tiff);
                }
                i += 2 + len;
            }
        }
    }
    None
}

/// IFD0's Orientation (tag 0x0112, a SHORT) of TIFF bytes, either byte order.
fn tiff_orientation(t: &[u8]) -> Option<u32> {
    let big = match t.get(..4)? {
        b"II\x2a\0" => false,
        b"MM\0\x2a" => true,
        _ => return None,
    };
    let u16_at = |at: usize| -> Option<u16> {
        let b: [u8; 2] = t.get(at..at + 2)?.try_into().ok()?;
        Some(if big {
            u16::from_be_bytes(b)
        } else {
            u16::from_le_bytes(b)
        })
    };
    let u32_at = |at: usize| -> Option<u32> {
        let b: [u8; 4] = t.get(at..at + 4)?.try_into().ok()?;
        Some(if big {
            u32::from_be_bytes(b)
        } else {
            u32::from_le_bytes(b)
        })
    };
    let ifd = u32_at(4)? as usize;
    for n in 0..usize::from(u16_at(ifd)?) {
        let entry = ifd + 2 + n * 12;
        if u16_at(entry)? == 0x0112 && u16_at(entry + 2)? == 3 {
            return u16_at(entry + 8).map(u32::from);
        }
    }
    None
}

#[cfg(all(feature = "fetch", not(target_arch = "wasm32")))]
mod native {
    use super::*;

    /// The request as a browser's engine Worker makes it from `root`.
    pub(super) fn fetch(root: &str, request: &UrlRequest) -> Result<Vec<u8>, String> {
        let base =
            og_net::http::origin_of(root).map(|_| format!("{}/", root.trim_end_matches('/')));
        let origin = base.as_deref().and_then(og_net::http::origin_of);
        og_net::http::get(&og_net::http::Request {
            url: &request.url,
            origin: origin.as_deref(),
            referrer: base.as_deref(),
            credentials: request.cross_origin == og_ui::css::request::CrossOrigin::UseCredentials,
            integrity: request.integrity.as_deref(),
            referrer_policy: request.referrer_policy.as_deref(),
        })
    }

    /// PNG, JPEG and WebP by their signature, SVG by its markup, with the
    /// EXIF orientation their metadata names ([`exif_orientation`]; 1 for
    /// SVG).
    pub(super) fn decode(bytes: &[u8]) -> Result<Decoded, String> {
        let format = if bytes.starts_with(b"\x89PNG") {
            image::ImageFormat::Png
        } else if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
            image::ImageFormat::Jpeg
        } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
            image::ImageFormat::WebP
        } else if is_svg(bytes) {
            return svg(bytes);
        } else {
            return Err("not a PNG, JPEG, WebP or SVG image".into());
        };
        let img = image::load_from_memory_with_format(bytes, format)
            .map_err(|e| format!("the image does not decode ({e})"))?
            .into_rgba8();
        Ok(Decoded {
            width: img.width(),
            height: img.height(),
            rgba: img.into_raw(),
            orientation: exif_orientation(bytes),
        })
    }

    /// The cook's test: UTF-8 text holding an `<svg` element.
    fn is_svg(bytes: &[u8]) -> bool {
        std::str::from_utf8(bytes).is_ok_and(|text| text.contains("<svg"))
    }

    /// An SVG at its own size, straight alpha, as the cook draws a
    /// package's (og-cook's texture format).
    fn svg(bytes: &[u8]) -> Result<Decoded, String> {
        let tree = resvg::usvg::Tree::from_data(bytes, &resvg::usvg::Options::default())
            .map_err(|e| format!("the SVG does not parse ({e})"))?;
        let size = tree.size().to_int_size();
        let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width(), size.height())
            .ok_or("the SVG has no size")?;
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::identity(),
            &mut pixmap.as_mut(),
        );
        // tiny-skia is premultiplied; a texture takes straight RGBA.
        let rgba = pixmap
            .pixels()
            .iter()
            .flat_map(|pixel| {
                let p = pixel.demultiply();
                [p.red(), p.green(), p.blue(), p.alpha()]
            })
            .collect();
        Ok(Decoded {
            width: size.width(),
            height: size.height(),
            rgba,
            orientation: 1,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::exif_orientation;

    /// TIFF bytes whose IFD0 holds Orientation `o`, in either byte order,
    /// after one other tag so the reader walks the directory.
    fn tiff(o: u16, big: bool) -> Vec<u8> {
        let u16b = |v: u16| {
            if big {
                v.to_be_bytes()
            } else {
                v.to_le_bytes()
            }
        };
        let u32b = |v: u32| {
            if big {
                v.to_be_bytes()
            } else {
                v.to_le_bytes()
            }
        };
        let mut t = if big {
            b"MM\0\x2a".to_vec()
        } else {
            b"II\x2a\0".to_vec()
        };
        t.extend(u32b(8));
        t.extend(u16b(2));
        // ImageWidth (0x0100), LONG 1 = 640: not the orientation.
        t.extend(u16b(0x0100));
        t.extend(u16b(4));
        t.extend(u32b(1));
        t.extend(u32b(640));
        t.extend(u16b(0x0112));
        t.extend(u16b(3));
        t.extend(u32b(1));
        t.extend(u16b(o));
        t.extend([0, 0]);
        t.extend(u32b(0));
        t
    }

    fn jpeg(app: &[(u8, Vec<u8>)]) -> Vec<u8> {
        let mut out = vec![0xff, 0xd8];
        for (marker, data) in app {
            out.extend([0xff, *marker]);
            out.extend(((data.len() + 2) as u16).to_be_bytes());
            out.extend(data);
        }
        out.extend([0xff, 0xda, 0, 2, 0xff, 0xd9]);
        out
    }

    fn png(chunks: &[(&[u8; 4], Vec<u8>)]) -> Vec<u8> {
        let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
        for (name, data) in chunks {
            out.extend((data.len() as u32).to_be_bytes());
            out.extend(*name);
            out.extend(data);
            out.extend([0; 4]);
        }
        out
    }

    fn webp(chunks: &[(&[u8; 4], Vec<u8>)]) -> Vec<u8> {
        let mut body = b"WEBP".to_vec();
        for (name, data) in chunks {
            body.extend(*name);
            body.extend((data.len() as u32).to_le_bytes());
            body.extend(data);
            if data.len() % 2 == 1 {
                body.push(0);
            }
        }
        [
            b"RIFF".to_vec(),
            (body.len() as u32).to_le_bytes().to_vec(),
            body,
        ]
        .concat()
    }

    /// Each container's EXIF, in both byte orders, behind a chunk or
    /// segment that is not it (an odd-length one in WebP, whose padding a
    /// reader must skip); and what names no orientation reads upright.
    #[test]
    fn exif_orientation_reads_each_containers_metadata() {
        for big in [false, true] {
            let exif = [b"Exif\0\0".to_vec(), tiff(6, big)].concat();
            let j = jpeg(&[(0xe0, b"JFIF\0\x01\x02".to_vec()), (0xe1, exif.clone())]);
            assert_eq!(exif_orientation(&j), 6, "JPEG big-endian {big}");
            let p = png(&[(b"IHDR", vec![0; 13]), (b"eXIf", tiff(8, big))]);
            assert_eq!(exif_orientation(&p), 8, "PNG big-endian {big}");
            let w = webp(&[(b"VP8X", vec![0; 9]), (b"EXIF", tiff(3, big))]);
            assert_eq!(exif_orientation(&w), 3, "WebP big-endian {big}");
            let w = webp(&[(b"VP8X", vec![0; 9]), (b"EXIF", exif)]);
            assert_eq!(exif_orientation(&w), 6, "WebP with JPEG's Exif prefix");
        }
        assert_eq!(exif_orientation(&jpeg(&[(0xe0, b"JFIF\0".to_vec())])), 1);
        assert_eq!(exif_orientation(&png(&[(b"IHDR", vec![0; 13])])), 1);
        // Out of range, truncated, and not an image.
        let j = jpeg(&[(0xe1, [b"Exif\0\0".to_vec(), tiff(9, false)].concat())]);
        assert_eq!(exif_orientation(&j), 1);
        assert_eq!(exif_orientation(&j[..12]), 1);
        assert_eq!(exif_orientation(b"<svg/>"), 1);
    }
}
