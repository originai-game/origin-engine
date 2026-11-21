//! The render tests' shared harness: headless frames and their pixels, one
//! copy of the helpers each test binary used to carry (a frame, a pixel and
//! luma read, the world-to-pixel projection, the output directory and the
//! frame it saves there). Headless native runs (a blocking renderer, files
//! in [`out_dir`]); the browser's is og-tests-web.

use crate::gfx::Rendered;
use og_render::{
    Camera, FrameStats, Projection, RenderDesc, RenderScene, RenderTarget, Renderer, RgbaImage,
};
use glam::Vec3;
use std::path::PathBuf;

/// One frame of `scene` at `desc` on a fresh blocking renderer: its image
/// and its stats.
pub fn frame(desc: &RenderDesc, scene: &RenderScene) -> (RgbaImage, FrameStats) {
    let mut renderer = Renderer::new_blocking(desc).expect("renderer");
    let stats = stepped(&mut renderer, scene);
    (renderer.read_offscreen().expect("capture"), stats)
}

/// `frames` frames of `scene` (at least one) on a fresh blocking renderer:
/// the last image and its stats. The repeats are what temporal work (TAA,
/// weather, simulation in the scene) needs to settle.
pub fn repeat(desc: &RenderDesc, scene: &RenderScene, frames: u32) -> (RgbaImage, FrameStats) {
    let mut renderer = Renderer::new_blocking(desc).expect("renderer");
    let mut stats = FrameStats::default();
    for _ in 0..frames.max(1) {
        stats = stepped(&mut renderer, scene);
    }
    (renderer.read_offscreen().expect("capture"), stats)
}

/// One frame of `scene` at the scenes' base desc: its image.
pub fn base_image(scene: &RenderScene) -> RgbaImage {
    let (img, _) = frame(&crate::scenes::base_desc(), scene);
    img
}

/// One frame of `scene` on `renderer`: its stats. The renderer's own; a
/// suite that steps a scene over frames calls this once a frame.
pub fn stepped(renderer: &mut Renderer, scene: &RenderScene) -> FrameStats {
    renderer
        .render(scene, &RenderTarget::Offscreen)
        .expect("render")
}

/// One frame of `scene` on `renderer`: its bytes.
pub fn pixels(renderer: &mut Renderer, scene: &RenderScene) -> Vec<u8> {
    stepped(renderer, scene);
    renderer.read_offscreen().expect("capture").pixels
}

/// The RGB of the pixel at (x, y). Unclamped: out of range is a panic, the
/// test's own bug.
pub fn px(img: &RgbaImage, x: u32, y: u32) -> [u8; 3] {
    let i = ((y * img.width + x) * 4) as usize;
    [img.pixels[i], img.pixels[i + 1], img.pixels[i + 2]]
}

/// [`px`] of the nearest pixel when (x, y) hangs off an edge (a probe a
/// pixel wide walking a silhouette).
pub fn px_clamped(img: &RgbaImage, x: u32, y: u32) -> [u8; 3] {
    px(img, x.min(img.width - 1), y.min(img.height - 1))
}

/// [`px`] as f32 (0 to 255, not 0 to 1).
pub fn px_f32(img: &RgbaImage, x: u32, y: u32) -> [f32; 3] {
    px(img, x, y).map(f32::from)
}

/// The sRGB bytes of pixel (x, y) of a rendered frame (the overlay is
/// 1280×720 CSS px on a 1280×720 frame: one CSS px a pixel).
pub fn px_rendered(r: &Rendered, x: u32, y: u32) -> [f32; 3] {
    let i = ((y * r.image.width + x) * 4) as usize;
    let p = &r.image.pixels;
    [f32::from(p[i]), f32::from(p[i + 1]), f32::from(p[i + 2])]
}

/// Rec. 709 luma of a pixel's RGB bytes.
pub fn luma(p: &[u8]) -> f32 {
    0.2126 * f32::from(p[0]) + 0.7152 * f32::from(p[1]) + 0.0722 * f32::from(p[2])
}

/// [`luma`] of RGB floats, as [`px_f32`] reads them.
pub fn luma_f32(p: [f32; 3]) -> f32 {
    0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2]
}

/// The luma of the pixel at (x, y).
pub fn luma_at(img: &RgbaImage, x: u32, y: u32) -> f32 {
    luma(&px(img, x, y))
}

/// The pixel a world point projects to in a `w` × `h` frame of `cam`.
pub fn project(cam: &Camera, w: u32, h: u32, p: Vec3) -> (u32, u32) {
    let view = glam::camera::rh::view::look_at_mat4(cam.position, cam.target, cam.up);
    let proj = match cam.projection {
        Projection::Perspective { fov_y_degrees } => glam::camera::rh::proj::directx::perspective(
            fov_y_degrees.to_radians(),
            w as f32 / h as f32,
            cam.near,
            cam.far,
        ),
        Projection::Orthographic { height } => {
            let hh = height * 0.5;
            let hw = hh * w as f32 / h as f32;
            glam::camera::rh::proj::directx::orthographic(-hw, hw, -hh, hh, cam.near, cam.far)
        }
    };
    let c = proj * view * p.extend(1.0);
    let ndc = c.truncate() / c.w;
    (
        ((ndc.x * 0.5 + 0.5) * w as f32) as u32,
        ((0.5 - ndc.y * 0.5) * h as f32) as u32,
    )
}

/// The suite's output directory, named `tag` (`og-gfx`, `og-effects`, …),
/// under the test target's temporary directory (`target/tmp`, where cargo
/// puts `CARGO_TARGET_TMPDIR`). Read from the running test binary: cargo
/// sets that variable only for a test target's own compilation, never for
/// this library's.
pub fn out_dir(tag: &str) -> PathBuf {
    let exe = std::env::current_exe().expect("a test harness runs from a test binary");
    let target = exe
        .ancestors()
        .nth(3)
        .expect("a test binary under the target directory");
    target.join("tmp").join(tag)
}

/// Saves `img` as `name.png` in [`out_dir`]`(tag)`, best effort: the frames
/// outlive the assertions that read them.
pub fn save(tag: &str, name: &str, img: &RgbaImage) {
    let _ = crate::gfx::save_png(&out_dir(tag).join(format!("{name}.png")), img);
}
