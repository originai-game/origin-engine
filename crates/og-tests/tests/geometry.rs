//! The depth wave's geometry (docs/depth.md §1): the reference frames of
//! the three.js primitives and a buffer-geometry height field
//! (`geometry-procedural`), lines (`lines`), point clouds (`points`),
//! wireframe materials (`wireframe`) and debug draw over the finished
//! frame (`debug-overlay`), and from frames: a wireframe draws edges, not
//! its faces; debug draw is not hidden by the geometry in front of it.

fn px(img: &og_render::RgbaImage, x: u32, y: u32) -> [u8; 3] {
    let p = &img.pixels[((y * img.width + x) * 4) as usize..][..3];
    [p[0], p[1], p[2]]
}

/// An unlit cube seen from a corner on a blue background, the material
/// `m` (its map `map`) under the material style `style`.
fn wire_cube(
    m: og_render::StandardMaterial,
    map: Option<og_render::TextureData>,
    style: og_render::MaterialStyle,
) -> og_render::RgbaImage {
    use glam::{Mat4, Vec3};
    use og_render::{
        Background, Camera, DirectionalLight, Exposure, RenderScene, RenderTarget, Renderer,
        StandardMaterial, shapes,
    };
    let mut scene = RenderScene::new();
    let map = map.map_or(0, |t| scene.add_texture(t).slot());
    let material = scene.add_standard_material(StandardMaterial { map, ..m });
    let cube = scene.add_mesh(shapes::cuboid(Vec3::ONE));
    scene.add_instance(cube, material, Mat4::IDENTITY);
    scene.stylise.material = style;
    scene.environment.background = Background::Color(Vec3::new(0.0, 0.0, 0.6));
    scene.environment.intensity = 0.0;
    scene.light = DirectionalLight {
        intensity: 0.0,
        ..Default::default()
    };
    scene.camera = Camera {
        position: Vec3::new(2.2, 1.8, 2.6),
        target: Vec3::ZERO,
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    let mut renderer = Renderer::new_blocking(&og_tests::scenes::base_desc()).expect("renderer");
    renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("render");
    renderer.read_offscreen().expect("capture")
}

/// A look's wireframe under its outline (neon-vector): the outline owns the
/// silhouette, the wire the creases inside it. An unlit black cube seen
/// from a corner, its red feature edges drawn over its faces and a green
/// outline inside its silhouette, on a blue background: the three edges
/// meeting at the near corner are red, and no red pixel touches the
/// background. Before, the wire's line was drawn on the silhouette too,
/// half outside the outline: a second, red edge round the shape.
#[test]
#[ignore = "requires GPU"]
fn an_outlined_wireframe_leaves_the_silhouette_to_the_outline() {
    let img = wire_cube(
        og_render::StandardMaterial {
            color: glam::Vec3::new(1.0, 0.0, 0.0),
            ..Default::default()
        },
        None,
        og_render::MaterialStyle {
            outline_color: [0.0, 1.0, 0.0],
            outline_width: 2.0,
            wireframe: true,
            ..Default::default()
        },
    );
    let is_red = |p: [u8; 3]| p[0] > 100 && p[1] < 80;
    let is_background = |p: [u8; 3]| p[2] > 100 && p[0] < 60 && p[1] < 60;
    let (mut wire, mut touching) = (0, 0);
    for y in 1..img.height - 1 {
        for x in 1..img.width - 1 {
            if !is_red(px(&img, x, y)) {
                continue;
            }
            wire += 1;
            if [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| {
                is_background(px(&img, (x as i32 + dx) as u32, (y as i32 + dy) as u32))
            }) {
                touching += 1;
            }
        }
    }
    assert!(wire > 100, "the creases are drawn: {wire} red pixels");
    assert_eq!(
        touching, 0,
        "{touching} of {wire} red pixels touch the background"
    );
}

/// A wireframe takes the colour its surface shows along each edge: a white
/// material whose map is green draws green edges, never white (the
/// neon-vector crate's white wire beside its texture's cyan borders); with
/// the look's `wireframe_color` every edge is that colour instead. Ways
/// this could fail: the wire in the material's colour alone (white), the
/// map not bound or sampled at the wrong group (white or black), the line
/// colour ignored (green).
#[test]
#[ignore = "requires GPU"]
fn a_wireframe_takes_its_surface_colour_or_the_looks_line_colour() {
    let green = og_render::TextureData {
        width: 4,
        height: 4,
        rgba: [20, 220, 40, 255].repeat(16),
        color_space: og_render::ColorSpace::Linear,
        mips: og_render::Mips::None,
    };
    let count = |img: &og_render::RgbaImage, f: &dyn Fn([u8; 3]) -> bool| {
        (0..img.height)
            .flat_map(|y| (0..img.width).map(move |x| (x, y)))
            .filter(|&(x, y)| f(px(img, x, y)))
            .count()
    };
    let white = |p: [u8; 3]| p.iter().all(|&c| c > 150);
    let is_green = |p: [u8; 3]| p[1] > 120 && p[0] < 80 && p[2] < 80;
    let is_red = |p: [u8; 3]| p[0] > 120 && p[1] < 80 && p[2] < 80;
    let style = og_render::MaterialStyle {
        wireframe: true,
        ..Default::default()
    };
    let surface = wire_cube(Default::default(), Some(green.clone()), style);
    assert_eq!(count(&surface, &white), 0, "no white wire");
    let edges = count(&surface, &is_green);
    assert!(edges > 300, "the edges are the map's green: {edges} pixels");
    let lined = wire_cube(
        Default::default(),
        Some(green),
        og_render::MaterialStyle {
            wireframe_color: [1.0, 0.0, 0.0],
            ..style
        },
    );
    assert_eq!(count(&lined, &is_green), 0, "no green with a line colour");
    let red = count(&lined, &is_red);
    assert!(red > 300, "the edges are the line colour: {red} pixels");
}
