//! A document's `url()` names http(s) images (docs/ui.md §3.3 Remote
//! images): the engine fetches each under its request modifiers, decodes
//! it and draws it in its layer; one refused draws nothing and warns.
//!
//! The game is served from one origin (its root is an http URL, read here
//! from disk by the hooks) and its art from another, a local server that
//! records what each request carried. The native host is an executor: the
//! first frame after the screen is pushed already draws every image that
//! loads.
//!
//! Ways this could fail: a remote URL is looked up as a package file and
//! warns "names no texture asset"; a modifier is dropped on the way from the
//! stylesheet to the request, so a wrong integrity hash or a refused CORS
//! response still draws; `cross-origin(use-credentials)` takes `*`; the
//! `Referer` ignores the policy (sent under `no-referrer`, the full URL
//! cross-origin by default); `Origin` is missing on a cross-origin request;
//! the draw waits a frame natively; a format is not decoded; an image still
//! being fetched warns, or never draws once it lands.

use og_runtime::{Dispatch, Fetch, Hooks, og_engine_create, og_game_game, set_hooks};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// What the art server saw of one request: its path, `Origin` and `Referer`.
type Seen = Arc<Mutex<Vec<(String, Option<String>, Option<String>)>>>;

/// Each path the art server answers: its bytes and response headers.
type Files = BTreeMap<String, (Vec<u8>, Vec<(String, String)>)>;

/// An HTTP server on its own port (another origin than the game's): each
/// path's bytes and response headers.
fn serve(files: Files) -> (u16, Seen) {
    serve_shared(Arc::new(Mutex::new(files)))
}

/// [`serve`] from a table the test may fill once it knows the port.
fn serve_shared(files: Arc<Mutex<Files>>) -> (u16, Seen) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let seen: Seen = Arc::default();
    let log = seen.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let path = line.split(' ').nth(1).unwrap_or("").to_string();
            let mut headers = BTreeMap::new();
            loop {
                let mut h = String::new();
                if reader.read_line(&mut h).unwrap() == 0 || h.trim().is_empty() {
                    break;
                }
                if let Some((k, v)) = h.split_once(':') {
                    headers.insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
                }
            }
            log.lock().unwrap().push((
                path.clone(),
                headers.get("origin").cloned(),
                headers.get("referer").cloned(),
            ));
            let response = match files.lock().unwrap().get(&path) {
                Some((body, extra)) => {
                    let mut head = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n",
                        body.len()
                    );
                    for (k, v) in extra {
                        head.push_str(&format!("{k}: {v}\r\n"));
                    }
                    [head.into_bytes(), b"\r\n".to_vec(), body.clone()].concat()
                }
                None => b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    .to_vec(),
            };
            let _ = stream.write_all(&response);
        }
    });
    (port, seen)
}

fn png(width: u32, height: u32) -> Vec<u8> {
    let mut out = Vec::new();
    image::RgbaImage::from_pixel(width, height, image::Rgba([200, 40, 40, 255]))
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

fn encoded(width: u32, height: u32, format: image::ImageFormat) -> Vec<u8> {
    let mut out = Vec::new();
    image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
        width,
        height,
        image::Rgb([40, 200, 40]),
    ))
    .write_to(&mut std::io::Cursor::new(&mut out), format)
    .unwrap();
    out
}

/// `jpeg` with an EXIF APP1 segment naming `orientation` (tag 0x0112).
fn with_exif(jpeg: &[u8], orientation: u8) -> Vec<u8> {
    let mut app1 = vec![0xff, 0xe1, 0x00, 0x22];
    app1.extend_from_slice(b"Exif\0\0II\x2a\x00\x08\x00\x00\x00\x01\x00");
    app1.extend_from_slice(&[0x12, 0x01, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00]);
    app1.extend_from_slice(&[orientation, 0, 0, 0, 0, 0, 0, 0]);
    [&jpeg[..2], &app1[..], &jpeg[2..]].concat()
}

fn sri(bytes: &[u8]) -> String {
    let digest = ring::digest::digest(&ring::digest::SHA384, bytes);
    format!("sha384-{}", og_core::base64::encode(digest.as_ref()))
}

/// The game's root as the engine is given it: an http URL whose files the
/// hooks read from `dir`.
const GAME: &str = "http://127.0.0.1:9/game";

struct Served {
    dir: PathBuf,
    /// The host's own fetch, as the web's engine Worker answers it: keys it
    /// was asked for, and answers it holds back until the test lets them go.
    host: Option<Arc<Mutex<HostFetches>>>,
}

#[derive(Default)]
struct HostFetches {
    asked: Vec<(u64, String)>,
    answers: BTreeMap<u64, og_runtime::Decoded>,
}

impl Hooks for Served {
    fn dispatch(&self, _call: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        let file = path.strip_prefix(GAME).map_or_else(
            || PathBuf::from(path),
            |rest| self.dir.join(rest.trim_start_matches('/')),
        );
        match std::fs::read(&file) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(e) => Fetch::Failed(format!("{path}: {e}")),
        }
    }
    fn remote_image_start(&self, key: u64, request: &og_runtime::UrlRequest) -> bool {
        let Some(host) = &self.host else { return false };
        host.lock().unwrap().asked.push((key, request.key()));
        true
    }
    fn image_decode_take(&self, key: u64) -> Fetch<og_runtime::Decoded> {
        match self
            .host
            .as_ref()
            .and_then(|h| h.lock().unwrap().answers.remove(&key))
        {
            Some(d) => Fetch::Ready(d),
            None => Fetch::Pending,
        }
    }
}

/// Each texture's size as the last frame's scene holds it, by handle.
type Sizes = Arc<Mutex<BTreeMap<u32, (u32, u32)>>>;

thread_local! {
    /// Each texture's orientation bits as the last frame's scene holds them.
    static TURNS: std::cell::RefCell<BTreeMap<u32, u8>> = Default::default();
}

struct Backend(Sizes);
impl og_runtime::RenderBackend for Backend {
    fn tier(&self) -> String {
        "test".into()
    }
    fn size(&self) -> (u32, u32) {
        (800, 100)
    }
    fn render(&mut self, scene: &og_render::RenderScene) -> Result<(), String> {
        let mut sizes = self.0.lock().unwrap();
        for i in 0..1024 {
            let h = og_render::TextureHandle::from_index(i);
            if let Some(t) = scene.texture(h) {
                sizes.insert(h.to_bits(), (t.width, t.height));
                TURNS.with(|o| {
                    o.borrow_mut()
                        .insert(h.to_bits(), scene.texture_orientation(h).bits())
                });
            }
        }
        Ok(())
    }
    fn capture(
        &mut self,
        _: &og_render::RenderScene,
        _: u32,
        _: u32,
    ) -> Option<og_render::RgbaImage> {
        None
    }
}

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

/// Cooks a project whose one screen lays `cells` (id, background) out
/// side by side, 40 px apart, and starts an engine on it with the screen
/// pushed and one frame drawn.
fn engine(
    name: &str,
    cells: &[(&str, String)],
    host: Option<Arc<Mutex<HostFetches>>>,
) -> (u32, Sizes) {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-runtime-remote-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("assets/ui")).unwrap();
    let mut markup = String::from("<meta name=\"screen\" content=\"Remote\" />\n<div id=\"Row\">");
    let mut css = String::from(
        "#Row { position: relative; width: 800px; height: 40px; }\n#Row > div { position: absolute; top: 0; width: 30px; height: 30px; background-color: #123456; }\n",
    );
    for (i, (id, background)) in cells.iter().enumerate() {
        markup.push_str(&format!("<div id=\"{id}\"></div>"));
        css.push_str(&format!(
            "#{id} {{ left: {}px; background-image: {background}; }}\n",
            i * 40
        ));
    }
    markup.push_str(&format!("</div>\n<style>\n{css}</style>\n"));
    std::fs::write(dir.join("assets/ui/remote.svelte"), markup).unwrap();
    let project = "type project\nversion 1\n---\nname \"remote images\"\nid \"remote-images\"\nentry \"main.ts\"\n";
    std::fs::write(dir.join("project.ogt"), project).unwrap();
    std::fs::write(dir.join("main.ts"), "").unwrap();
    let report = og_cook::cook_project(&dir, &og_cook::CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    set_hooks(Rc::new(Served { dir, host }));
    og_runtime::og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": GAME, "project": project }).to_string();
    let e = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    // A game that lists no `frames` has no remote frames on any host
    // (on the web its engine has none built in).
    assert!(!og_runtime::og_ui_remote_frames(e));
    let sizes = Sizes::default();
    og_runtime::set_render_backend(e, Some(Box::new(Backend(sizes.clone())))).unwrap();
    value(og_runtime::host_impl::answer(og_runtime::og_load_document(
        e,
        "ui/remote.svelte".into(),
    )));
    value(og_runtime::host_impl::answer(og_runtime::og_ui_push(
        e,
        "Remote".into(),
        None,
    )));
    og_runtime::og_engine_frame(e, 1000.0 / 60.0);
    (e, sizes)
}

/// The images each cell draws, by its id: their texture's size in texels.
fn drawn(e: u32, sizes: &Sizes, cells: &[&str]) -> BTreeMap<String, (u32, u32)> {
    let ui = og_runtime::shared_ui(e).unwrap();
    let ui = ui.lock();
    let mut out = BTreeMap::new();
    for item in &ui.frame().items {
        if let og_ui::DrawCmd::Shape { shape, .. } = &item.cmd
            && let og_render::UiPaint::Image(image) = &shape.paint
        {
            let cell = (shape.center.x / 40.0).floor() as usize;
            let size = sizes.lock().unwrap().get(&image.texture.to_bits()).copied();
            out.insert(cells[cell].to_string(), size.unwrap_or((0, 0)));
        }
    }
    out
}

/// The orientation bits of the texture cell `id` draws (`cells` in order).
fn turn(e: u32, cells: &[&str], id: &str) -> Option<u8> {
    let ui = og_runtime::shared_ui(e).unwrap();
    let ui = ui.lock();
    let cell = cells.iter().position(|c| *c == id).unwrap();
    ui.frame()
        .items
        .iter()
        .find_map(|item| match &item.cmd {
            og_ui::DrawCmd::Shape { shape, .. }
                if (shape.center.x / 40.0).floor() as usize == cell =>
            {
                match &shape.paint {
                    og_render::UiPaint::Image(image) => Some(image.texture.to_bits()),
                    _ => None,
                }
            }
            _ => None,
        })
        .and_then(|t| TURNS.with(|o| o.borrow().get(&t).copied()))
}

fn warnings(e: u32) -> Vec<String> {
    let log: Value = serde_json::from_str(&og_runtime::og_engine_take_log(e)).unwrap();
    log.as_array()
        .unwrap()
        .iter()
        .filter(|l| l["warning"] == json!(true))
        .map(|l| l["text"].as_str().unwrap_or_default().to_string())
        .collect()
}

// The native fetch is the `fetch` feature's.
#[cfg(feature = "fetch")]
#[test]
fn remote_images_load_under_their_request_modifiers() {
    let star = png(3, 5);
    let cors = |o: &str| vec![("Access-Control-Allow-Origin".to_string(), o.to_string())];
    let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="9" height="3"><rect width="9" height="3" fill="blue"/></svg>"#;
    let files: Files = [
        ("/star.png", (star.clone(), cors("*"))),
        (
            "/photo.jpg",
            (encoded(6, 2, image::ImageFormat::Jpeg), cors("*")),
        ),
        (
            "/leaf.webp",
            (encoded(4, 7, image::ImageFormat::WebP), cors("*")),
        ),
        ("/badge.svg", (svg.to_vec(), cors("*"))),
        (
            "/turned.jpg",
            (
                with_exif(&encoded(6, 2, image::ImageFormat::Jpeg), 6),
                cors("*"),
            ),
        ),
        ("/nocors.png", (star.clone(), vec![])),
        (
            "/creds.png",
            (star.clone(), {
                let mut h = cors("http://127.0.0.1:9");
                h.push(("Access-Control-Allow-Credentials".into(), "true".into()));
                h
            }),
        ),
        ("/r1.png", (star.clone(), cors("*"))),
        ("/r2.png", (star.clone(), cors("*"))),
        ("/r3.png", (star.clone(), cors("*"))),
        ("/r4.png", (star.clone(), cors("*"))),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect();
    let (port, seen) = serve(files);
    let at = |p: &str| format!("http://127.0.0.1:{port}{p}");
    let wrong = sri(b"other bytes");
    let cells: Vec<(&str, String)> = vec![
        ("Png", format!("url({})", at("/star.png"))),
        ("Jpeg", format!("url(\"{}\")", at("/photo.jpg"))),
        ("Webp", format!("url(\"{}\")", at("/leaf.webp"))),
        ("Svg", format!("url(\"{}\")", at("/badge.svg"))),
        ("Turned", format!("url(\"{}\")", at("/turned.jpg"))),
        (
            "Good",
            format!("url(\"{}\" integrity(\"{}\"))", at("/star.png"), sri(&star)),
        ),
        (
            "Bad",
            format!("url(\"{}\" integrity(\"{wrong}\"))", at("/star.png")),
        ),
        (
            "Strongest",
            format!(
                "url(\"{}\" integrity(\"sha256-AAAA {}\"))",
                at("/star.png"),
                sri(&star)
            ),
        ),
        (
            "Anonymous",
            format!("url(\"{}\" cross-origin(anonymous))", at("/creds.png")),
        ),
        (
            "CredStar",
            format!("url(\"{}\" cross-origin(use-credentials))", at("/star.png")),
        ),
        (
            "CredOk",
            format!(
                "url(\"{}\" cross-origin(use-credentials))",
                at("/creds.png")
            ),
        ),
        ("NoCors", format!("url(\"{}\")", at("/nocors.png"))),
        (
            "NoRef",
            format!("url(\"{}\" referrer-policy(no-referrer))", at("/r1.png")),
        ),
        (
            "Unsafe",
            format!("url(\"{}\" referrer-policy(unsafe-url))", at("/r2.png")),
        ),
        ("Default", format!("url(\"{}\")", at("/r3.png"))),
        (
            "SameOrigin",
            format!("url(\"{}\" referrer-policy(same-origin))", at("/r4.png")),
        ),
    ];
    let ids: Vec<&str> = cells.iter().map(|(id, _)| *id).collect();
    let (e, sizes) = engine("modifiers", &cells, None);
    let drawn = drawn(e, &sizes, &ids);
    let expect: BTreeMap<String, (u32, u32)> = [
        ("Png", (3, 5)),
        ("Jpeg", (6, 2)),
        ("Webp", (4, 7)),
        ("Svg", (9, 3)),
        ("Turned", (6, 2)),
        ("Good", (3, 5)),
        ("Strongest", (3, 5)),
        ("Anonymous", (3, 5)),
        ("CredOk", (3, 5)),
        ("NoRef", (3, 5)),
        ("Unsafe", (3, 5)),
        ("Default", (3, 5)),
        ("SameOrigin", (3, 5)),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect();
    assert_eq!(
        drawn, expect,
        "the first frame draws every image that loads, at its own size, and none refused"
    );
    // The texels stay as sent; the EXIF orientation is the texture's, for
    // CSS image-orientation to turn at draw time (6: a quarter turn).
    assert_eq!(
        turn(e, &ids, "Turned"),
        Some(og_render::UiOrientation::from_exif(6).bits())
    );
    assert_eq!(
        turn(e, &ids, "Jpeg"),
        Some(0),
        "a JPEG without EXIF is upright"
    );
    let warned = warnings(e);
    let one = |needle: &str, url: &str| {
        assert!(
            warned
                .iter()
                .any(|w| w.contains(&format!("url({url})")) && w.contains(needle)),
            "{needle} for {url} in {warned:#?}"
        );
    };
    one("refused by integrity", &at("/star.png"));
    one("to name the origin, not *", &at("/star.png"));
    one("no Access-Control-Allow-Origin", &at("/nocors.png"));
    assert_eq!(warned.len(), 3, "{warned:#?}");
    assert!(!warned.iter().any(|w| w.contains("names no texture")));

    // What the requests carried: the game's origin, and the referrer each
    // policy gives a request to another origin from http://127.0.0.1:9/game/.
    let seen = seen.lock().unwrap().clone();
    let one_request = |path: &str| {
        let got: Vec<_> = seen.iter().filter(|(p, ..)| p == path).collect();
        assert_eq!(got.len(), 1, "{path} fetched once: {seen:#?}");
        got[0].clone()
    };
    assert!(
        seen.iter()
            .all(|(_, o, _)| o.as_deref() == Some("http://127.0.0.1:9")),
        "every request carries the game's origin: {seen:#?}"
    );
    assert_eq!(one_request("/r1.png").2, None, "no-referrer");
    assert_eq!(
        one_request("/r2.png").2.as_deref(),
        Some("http://127.0.0.1:9/game/"),
        "unsafe-url sends the whole URL"
    );
    assert_eq!(
        one_request("/r3.png").2.as_deref(),
        Some("http://127.0.0.1:9/"),
        "strict-origin-when-cross-origin sends only the origin"
    );
    assert_eq!(
        one_request("/r4.png").2,
        None,
        "same-origin sends none cross-origin"
    );
    assert!(og_runtime::og_engine_destroy(e));
}

#[test]
fn an_image_still_fetched_draws_the_fallback_until_it_lands() {
    let host: Arc<Mutex<HostFetches>> = Arc::default();
    let url = "https://cdn.example/coin.png";
    let (e, sizes) = engine(
        "pending",
        &[("Coin", format!("url(\"{url}\" integrity(\"sha384-x\"))"))],
        Some(host.clone()),
    );
    let colour_only = || {
        let ui = og_runtime::shared_ui(e).unwrap();
        let ui = ui.lock();
        ui.frame()
            .items
            .iter()
            .filter_map(|item| match &item.cmd {
                og_ui::DrawCmd::Shape { shape, .. } => Some(&shape.paint),
                _ => None,
            })
            .all(|p| !matches!(p, og_render::UiPaint::Image(_)))
    };
    let asked = host.lock().unwrap().asked.clone();
    assert_eq!(
        asked.iter().map(|(_, k)| k.as_str()).collect::<Vec<_>>(),
        [format!("{url} integrity(\"sha384-x\")")],
        "the host is asked once, with the request's modifiers"
    );
    for _ in 0..3 {
        og_runtime::og_engine_frame(e, 1000.0 / 60.0);
        assert!(
            colour_only(),
            "only the background colour while it is fetched"
        );
    }
    assert!(
        !og_runtime::og_loading_idle(e),
        "og.loading.idle waits for it"
    );
    assert!(warnings(e).is_empty(), "a pending image warns of nothing");
    host.lock().unwrap().answers.insert(
        asked[0].0,
        og_runtime::Decoded {
            width: 2,
            height: 8,
            rgba: vec![255; 2 * 8 * 4],
            // The web host reads it from the fetched bytes.
            orientation: 6,
        },
    );
    og_runtime::og_engine_frame(e, 1000.0 / 60.0);
    assert_eq!(
        drawn(e, &sizes, &["Coin"]).get("Coin"),
        Some(&(2, 8)),
        "drawn from the frame after it lands"
    );
    assert_eq!(
        turn(e, &["Coin"], "Coin"),
        Some(og_render::UiOrientation::from_exif(6).bits()),
        "the host's orientation reaches the texture"
    );
    assert!(og_runtime::og_engine_destroy(e));
}

/// A remote `<iframe>` (docs/ui.md §2.1 Frames) on the native host, an
/// executor: the frame's document is fetched from an origin project.ogt's
/// `frames` lists, with the iframe's request attributes, and the first frame
/// after the screen is pushed already shows it, the image it names beside
/// itself, and the frame it embeds beside itself. A frame it embeds from an
/// origin the project does not list is never fetched and warns.
///
/// Ways this could fail: the document fetched from any origin; the
/// integrity attribute dropped on the way to the request; a relative url()
/// in the remote document read as a package file; the first frame drawn
/// before the chain lands; a nested frame left unfetched.
#[cfg(all(feature = "fetch", feature = "frames"))]
#[test]
fn a_remote_frame_loads_its_document_chain_from_allowed_origins() {
    let cors = || vec![("Access-Control-Allow-Origin".to_string(), "*".to_string())];
    let event = |port: u16| {
        format!(
            r#"<meta name="screen" content="event">
<meta name="prop:heading" content="string">
<div id="Card"><p id="Heading">{{heading}}</p><div id="Star"></div>
<iframe id="Inner" src="inner.svelte"></iframe>
<iframe id="Elsewhere" src="http://localhost:{port}/og/inner.svelte"></iframe></div>
<style>
#Card {{ display: flex; }}
#Heading {{ margin: 0; width: 60px; }}
#Star {{ width: 30px; height: 30px; background-image: url("art/star.png"); }}
iframe {{ width: 120px; height: 60px; }}
</style>"#
        )
    };
    let inner = r#"<meta name="screen" content="inner"><p id="InnerText">inside</p>"#;
    // Serve first on a free port, then write the documents naming it: the
    // server answers from a table filled once the port is known.
    let table: Arc<Mutex<Files>> = Arc::default();
    let (port, seen) = serve_shared(table.clone());
    {
        let mut t = table.lock().unwrap();
        t.insert(
            "/og/event.svelte".into(),
            (event(port).into_bytes(), cors()),
        );
        t.insert(
            "/og/inner.svelte".into(),
            (inner.as_bytes().to_vec(), cors()),
        );
        t.insert("/og/art/star.png".into(), (png(3, 5), cors()));
    }
    let doc = table.lock().unwrap()["/og/event.svelte"].0.clone();
    let origin = format!("http://127.0.0.1:{port}");
    let page = format!(
        r#"<meta name="screen" content="Remote" />
<div id="Row"><iframe id="Event" src="{origin}/og/event.svelte" integrity="{}" heading="Night"></iframe>
<iframe id="Wrong" src="{origin}/og/inner.svelte" integrity="{}"></iframe></div>
<style>#Row {{ display: flex; }} iframe {{ width: 390px; height: 90px; }}</style>"#,
        sri(&doc),
        sri(b"other")
    );
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-runtime-remote-frames-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("assets/ui")).unwrap();
    std::fs::write(dir.join("assets/ui/remote.svelte"), page).unwrap();
    let project = format!(
        "type project\nversion 1\n---\nname \"remote frames\"\nid \"remote-frames\"\nentry \"main.ts\"\nframes \"{origin}\"\n"
    );
    std::fs::write(dir.join("project.ogt"), &project).unwrap();
    std::fs::write(dir.join("main.ts"), "").unwrap();
    let report = og_cook::cook_project(&dir, &og_cook::CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    set_hooks(Rc::new(Served { dir, host: None }));
    og_runtime::og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": GAME, "project": project }).to_string();
    let e = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    assert!(og_runtime::og_ui_remote_frames(e), "it lists `frames`");
    let sizes = Sizes::default();
    og_runtime::set_render_backend(e, Some(Box::new(Backend(sizes.clone())))).unwrap();
    value(og_runtime::host_impl::answer(og_runtime::og_load_document(
        e,
        "ui/remote.svelte".into(),
    )));
    value(og_runtime::host_impl::answer(og_runtime::og_ui_push(
        e,
        "Remote".into(),
        None,
    )));
    og_runtime::og_engine_frame(e, 1000.0 / 60.0);
    let (texts, star) = {
        let ui = og_runtime::shared_ui(e).unwrap();
        let ui = ui.lock();
        let texts: BTreeMap<String, String> = ui
            .probe_elements()
            .into_iter()
            .map(|el| (el.path.clone(), el.text.clone()))
            .collect();
        let star = ui.frame().items.iter().find_map(|item| match &item.cmd {
            og_ui::DrawCmd::Shape { shape, .. } => match &shape.paint {
                og_render::UiPaint::Image(image) => {
                    sizes.lock().unwrap().get(&image.texture.to_bits()).copied()
                }
                _ => None,
            },
            _ => None,
        });
        (texts, star)
    };
    let text = |end: &str| {
        texts
            .iter()
            .find(|(p, _)| p.ends_with(end))
            .map(|(_, t)| t.clone())
    };
    assert_eq!(
        text("/Event/Card/Heading").as_deref(),
        Some("Night"),
        "{texts:#?}"
    );
    assert_eq!(
        text("/Event/Card/Inner/InnerText").as_deref(),
        Some("inside"),
        "the frame it embeds, beside it: {texts:#?}"
    );
    // Both frames are on screen, and empty.
    assert!(text("/Event/Card/Elsewhere").is_some(), "{texts:#?}");
    assert!(text("/Row/Wrong").is_some(), "{texts:#?}");
    assert!(text("/Elsewhere/InnerText").is_none(), "{texts:#?}");
    assert!(text("/Wrong/InnerText").is_none(), "{texts:#?}");
    assert_eq!(star, Some((3, 5)), "its relative url() is beside it");
    let warned = warnings(e);
    assert!(
        warned.iter().any(
            |w| w.contains(&format!("http://localhost:{port}/og/inner.svelte"))
                && w.contains("frames")
        ),
        "{warned:#?}"
    );
    assert!(
        warned
            .iter()
            .any(|w| w.contains("/og/inner.svelte") && w.contains("integrity")),
        "{warned:#?}"
    );
    let seen = seen.lock().unwrap().clone();
    let paths: Vec<&str> = seen.iter().map(|(p, ..)| p.as_str()).collect();
    assert_eq!(
        paths.iter().filter(|p| **p == "/og/event.svelte").count(),
        1,
        "{paths:?}"
    );
    assert!(paths.contains(&"/og/art/star.png"), "{paths:?}");
    assert!(
        seen.iter()
            .all(|(_, o, _)| o.as_deref() == Some("http://127.0.0.1:9")),
        "every request carries the game's origin: {seen:#?}"
    );
    assert!(og_runtime::og_engine_destroy(e));
}
