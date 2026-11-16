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
}

enum Image {
    /// With the host, answered under this key.
    Host(u64),
    /// On a worker thread of this process.
    #[cfg(not(target_arch = "wasm32"))]
    Thread(std::sync::mpsc::Receiver<Result<(Decoded, u32), String>>),
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
                    // The browser's decoder has turned it by its EXIF
                    // already (createImageBitmap's `from-image`).
                    Some(Fetch::Ready(d)) => Some(Ok((d, 1))),
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
            Ok((d, exif)) => {
                let mut rs = eng.render.borrow_mut();
                let handle = rs.scene.add_texture(og_render::TextureData {
                    width: d.width,
                    height: d.height,
                    rgba: d.rgba,
                    color_space: og_render::ColorSpace::Srgb,
                    mips: og_render::Mips::Generate,
                });
                // Stored as sent; CSS image-orientation turns it at draw time.
                rs.scene
                    .set_texture_orientation(handle, og_render::UiOrientation::from_exif(exif));
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

/// Whether any image is still being fetched (og.loading.idle is false).
pub(crate) fn pending(eng: &Engine) -> bool {
    eng.assets
        .borrow()
        .remote
        .images
        .values()
        .any(|i| !matches!(i, Image::Bound | Image::Failed))
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
    /// EXIF orientation their metadata names (1, upright, for SVG).
    pub(super) fn decode(bytes: &[u8]) -> Result<(Decoded, u32), String> {
        let format = if bytes.starts_with(b"\x89PNG") {
            image::ImageFormat::Png
        } else if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
            image::ImageFormat::Jpeg
        } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
            image::ImageFormat::WebP
        } else if is_svg(bytes) {
            return svg(bytes).map(|d| (d, 1));
        } else {
            return Err("not a PNG, JPEG, WebP or SVG image".into());
        };
        use image::ImageDecoder;
        let bad = |e: image::ImageError| format!("the image does not decode ({e})");
        let mut decoder = image::ImageReader::with_format(std::io::Cursor::new(bytes), format)
            .into_decoder()
            .map_err(bad)?;
        let exif = decoder.orientation().map_or(1, |o| u32::from(o.to_exif()));
        let img = image::DynamicImage::from_decoder(decoder)
            .map_err(bad)?
            .into_rgba8();
        Ok((
            Decoded {
                width: img.width(),
                height: img.height(),
                rgba: img.into_raw(),
            },
            exif,
        ))
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
        })
    }
}
