//! FBX and OBJ through ufbx into glTF: what the written glb says.
//!
//! Ways this could fail: the file's axes or unit are taken as they are
//! (a Z-up centimetre model lies on its back, a hundred times too big);
//! v is not flipped (textures upside down); an OBJ's `.mtl` is not read,
//! or not hashed (an edited material never re-cooks); separate roughness
//! and metalness maps are not packed (or packed into the wrong channels,
//! or glossiness not inverted); an opacity below 1 stays opaque; a
//! missing texture or `mtllib` is skipped silently instead of refused
//! with its fix; an instanced mesh is written twice; a texture embedded
//! in the FBX is lost.

use super::*;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/fbx")
        .join(name)
}

fn import(name: &str) -> Imported {
    let p = fixture(name);
    let ext = name.rsplit('.').next().unwrap();
    to_glb(&std::fs::read(&p).unwrap(), &p, ext).unwrap()
}

/// A folder of its own under the test target directory, with `files`.
fn folder(name: &str, files: &[(&str, &[u8])]) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/tmp/og-cook-fbx-unit")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (f, b) in files {
        std::fs::write(dir.join(f), b).unwrap();
    }
    dir
}

fn png(w: u32, h: u32, f: impl Fn(u32, u32) -> [u8; 4]) -> Vec<u8> {
    let img = image::RgbaImage::from_fn(w, h, |x, y| image::Rgba(f(x, y)));
    let mut out = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

/// The glb's JSON chunk.
fn json_of(glb: &[u8]) -> Value {
    let len = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
    serde_json::from_slice(&glb[20..20 + len]).unwrap()
}

/// Every mesh vertex of the default scene in world space, and each
/// node's name with its world matrix.
fn world_points(glb: &[u8]) -> (Vec<Vec3>, Vec<(String, glam::Mat4)>) {
    let g = gltf::Gltf::from_slice(glb).unwrap();
    let blob = g.blob.as_deref().unwrap();
    let mut pts = Vec::new();
    let mut worlds = Vec::new();
    fn walk(
        n: gltf::Node,
        parent: glam::Mat4,
        blob: &[u8],
        pts: &mut Vec<Vec3>,
        worlds: &mut Vec<(String, glam::Mat4)>,
    ) {
        let w = parent * glam::Mat4::from_cols_array_2d(&n.transform().matrix());
        worlds.push((n.name().unwrap_or("").to_string(), w));
        if let Some(m) = n.mesh() {
            for p in m.primitives() {
                let r = p.reader(|_| Some(blob));
                for v in r.read_positions().unwrap() {
                    pts.push(w.transform_point3(Vec3::from(v)));
                }
            }
        }
        for c in n.children() {
            walk(c, w, blob, pts, worlds);
        }
    }
    for n in g.default_scene().unwrap().nodes() {
        walk(n, glam::Mat4::IDENTITY, blob, &mut pts, &mut worlds);
    }
    (pts, worlds)
}

fn bounds(pts: &[Vec3]) -> (Vec3, Vec3) {
    pts.iter().fold((Vec3::MAX, Vec3::MIN), |(lo, hi), p| {
        (lo.min(*p), hi.max(*p))
    })
}

/// The bytes of image `i`, packed in the glb.
fn image_bytes(glb: &[u8], i: usize) -> Vec<u8> {
    let g = gltf::Gltf::from_slice(glb).unwrap();
    let blob = g.blob.as_deref().unwrap();
    match g.images().nth(i).unwrap().source() {
        gltf::image::Source::View { view, .. } => {
            blob[view.offset()..view.offset() + view.length()].to_vec()
        }
        gltf::image::Source::Uri { uri, .. } => panic!("image {i} is named: {uri}"),
    }
}

/// A hand-written ASCII FBX, Z up in centimetres: a 200 x 100 cm board
/// lying flat 50 cm up comes out 2 x 1 m, in the XZ plane at y 0.5 (the
/// file's +Y, its front, is the engine's -Z), facing +Y, its Phong
/// diffuse as the base colour.
#[test]
fn an_ascii_fbx_in_centimetres_with_z_up_lands_in_metres_with_y_up() {
    let i = import("board.fbx");
    let (pts, worlds) = world_points(&i.glb);
    let world = worlds.iter().find(|w| w.0 == "board").unwrap().1;
    let (lo, hi) = bounds(&pts);
    assert!(lo.abs_diff_eq(Vec3::new(0.0, 0.5, -1.0), 1e-5), "{lo}");
    assert!(hi.abs_diff_eq(Vec3::new(2.0, 0.5, 0.0), 1e-5), "{hi}");
    let g = gltf::Gltf::from_slice(&i.glb).unwrap();
    let prim = g.meshes().next().unwrap().primitives().next().unwrap();
    let r = prim.reader(|_| g.blob.as_deref());
    // Two triangles, normals made up the +Y side, counter-clockwise from there.
    let idx: Vec<u32> = r.read_indices().unwrap().into_u32().collect();
    assert_eq!(idx.len(), 6);
    let p: Vec<Vec3> = r
        .read_positions()
        .unwrap()
        .map(|v| world.transform_point3(Vec3::from(v)))
        .collect();
    for t in idx.chunks(3) {
        let n = (p[t[1] as usize] - p[t[0] as usize]).cross(p[t[2] as usize] - p[t[0] as usize]);
        assert!(n.normalize().abs_diff_eq(Vec3::Y, 1e-5), "winding {n}");
    }
    for n in r.read_normals().unwrap() {
        let n = world.transform_vector3(Vec3::from(n)).normalize();
        assert!(n.abs_diff_eq(Vec3::Y, 1e-5), "normal {n:?}");
    }
    let j = json_of(&i.glb);
    let base = &j["materials"][0]["pbrMetallicRoughness"]["baseColorFactor"];
    let base: Vec<f64> = base
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect();
    assert!(
        (base[0] - 0.2).abs() < 1e-6
            && (base[1] - 0.6).abs() < 1e-6
            && (base[3] - 1.0).abs() < 1e-9,
        "{base:?}"
    );
    assert_eq!(j["materials"][0]["name"], "paint");
}

const OBJ: &str = "# a unit quad facing +Y, uvs from the image's bottom left
mtllib quad.mtl
v 0 0 0
v 1 0 0
v 1 0 -1
v 0 0 -1
vt 0 0
vt 1 0
vt 1 1
vt 0 1
vn 0 1 0
usemtl tile
f 1/1/1 2/2/1 3/3/1 4/4/1
";

/// An OBJ's `.mtl` (Kd, d, map_Kd, map_Pr, map_Pm) becomes a glTF
/// material: the colour map named beside it, an alpha blend for `d 0.5`,
/// roughness (G) and metalness (B) packed into one image at cook time; the
/// `.mtl` and the packed maps are files read (hashed, consumed); v flips.
#[test]
fn an_obj_material_comes_from_its_mtl_with_its_maps_packed() {
    let mtl = b"newmtl tile
Kd 0.5 0.25 1
d 0.5
map_Kd tile.png
map_Pr rough.png
map_Pm metal.png
";
    let tile = png(2, 2, |_, _| [255, 255, 255, 255]);
    // Roughness rises left to right, metalness top to bottom.
    let rough = png(2, 2, |x, _| [if x == 0 { 40 } else { 200 }, 0, 0, 255]);
    let metal = png(2, 2, |_, y| [if y == 0 { 10 } else { 250 }, 0, 0, 255]);
    let dir = folder(
        "obj-mtl",
        &[
            ("quad.obj", OBJ.as_bytes()),
            ("quad.mtl", mtl),
            ("tile.png", &tile),
            ("rough.png", &rough),
            ("metal.png", &metal),
        ],
    );
    let path = dir.join("quad.obj");
    let i = to_glb(OBJ.as_bytes(), &path, "obj").unwrap();
    let j = json_of(&i.glb);
    let m = &j["materials"][0];
    assert_eq!(m["name"], "tile");
    assert_eq!(m["alphaMode"], "BLEND");
    let pbr = &m["pbrMetallicRoughness"];
    // A colour map replaces Kd; d is the alpha.
    let base: Vec<f64> = pbr["baseColorFactor"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect();
    assert!((base[3] - 0.5).abs() < 1e-6, "{base:?}");
    let base_img =
        j["textures"][pbr["baseColorTexture"]["index"].as_u64().unwrap() as usize]["source"]
            .as_u64()
            .unwrap() as usize;
    assert_eq!(j["images"][base_img]["uri"], "tile.png");
    let mr_img = j["textures"][pbr["metallicRoughnessTexture"]["index"].as_u64().unwrap() as usize]
        ["source"]
        .as_u64()
        .unwrap() as usize;
    let packed = image::load_from_memory(&image_bytes(&i.glb, mr_img))
        .unwrap()
        .to_rgba8();
    for (x, y, g, b) in [
        (0, 0, 40, 10),
        (1, 0, 200, 10),
        (0, 1, 40, 250),
        (1, 1, 200, 250),
    ] {
        let p = packed.get_pixel(x, y).0;
        assert_eq!((p[1], p[2]), (g, b), "pixel {x},{y}");
    }
    assert_eq!(pbr["roughnessFactor"], 1.0);
    assert_eq!(pbr["metallicFactor"], 1.0);
    let read: Vec<&str> = i.read.iter().map(|r| r.0.as_str()).collect();
    assert_eq!(read, ["metal.png", "quad.mtl", "rough.png"]);
    // The vt (0, 1) corner, the image's top left, is glTF's uv (0, 0).
    let g = gltf::Gltf::from_slice(&i.glb).unwrap();
    let prim = g.meshes().next().unwrap().primitives().next().unwrap();
    let r = prim.reader(|_| g.blob.as_deref());
    let pos: Vec<[f32; 3]> = r.read_positions().unwrap().collect();
    let uv: Vec<[f32; 2]> = r.read_tex_coords(0).unwrap().into_f32().collect();
    let corner = pos.iter().position(|p| *p == [0.0, 0.0, -1.0]).unwrap();
    assert_eq!(uv[corner], [0.0, 0.0]);
}

/// What an import cannot do without is refused with the fix: a `mtllib`
/// not beside the OBJ, a texture neither embedded nor beside the FBX, and
/// bytes that are no FBX.
#[test]
fn missing_files_and_unreadable_bytes_are_refused_with_the_fix() {
    let dir = folder("refused", &[("quad.obj", OBJ.as_bytes())]);
    let e = to_glb(OBJ.as_bytes(), &dir.join("quad.obj"), "obj")
        .err()
        .unwrap();
    assert!(
        e.contains("quad.mtl is not beside the file; put it there"),
        "{e}"
    );
    let linked = std::fs::read(fixture("props-linked.fbx")).unwrap();
    let dir = folder("refused-texture", &[("props-linked.fbx", &linked)]);
    let e = to_glb(&linked, &dir.join("props-linked.fbx"), "fbx")
        .err()
        .unwrap();
    assert!(
        e.contains("texture 'crate.png' is neither embedded nor beside the model"),
        "{e}"
    );
    assert!(e.contains("Embed Media"), "{e}");
    let e = to_glb(b"not an fbx at all", &dir.join("junk.fbx"), "fbx")
        .err()
        .unwrap();
    assert!(e.contains("re-export it"), "{e}");
}

/// Blender's FBX of two crates sharing a mesh, a lamp parented to one, and
/// a light: the shared mesh is one glTF mesh on both nodes, the texture
/// embedded in the FBX is packed into the glb (its top-left cell red, as
/// painted), the lamp's emission keeps its strength 3, and the point light
/// is kept. The file named its texture beside it instead (props-linked)
/// gives the same glTF but for the image, named by its path.
#[test]
fn blender_props_keep_instancing_embedded_texture_emission_and_light() {
    let i = import("props.fbx");
    let j = json_of(&i.glb);
    let nodes = j["nodes"].as_array().unwrap();
    let mesh_of = |name: &str| nodes.iter().find(|n| n["name"] == name).unwrap()["mesh"].clone();
    assert_eq!(mesh_of("crate"), mesh_of("crate-twin"));
    assert_eq!(j["meshes"].as_array().unwrap().len(), 2);
    let img = image::load_from_memory(&image_bytes(&i.glb, 0))
        .unwrap()
        .to_rgba8();
    assert_eq!(img.dimensions(), (16, 16));
    let tl = img.get_pixel(1, 1).0;
    assert!(tl[0] > 150 && tl[1] < 60, "top left {tl:?}");
    let glow = j["materials"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["name"] == "glow")
        .unwrap();
    assert_eq!(
        glow["extensions"]["KHR_materials_emissive_strength"]["emissiveStrength"],
        3.0
    );
    assert_eq!(
        j["extensions"]["KHR_lights_punctual"]["lights"][0]["type"],
        "point"
    );
    // The crate stands on the ground, 2 m long in x: Blender's Z up is Y.
    let (pts, worlds) = world_points(&i.glb);
    let (lo, hi) = bounds(&pts);
    assert!(
        (lo.y - 0.0).abs() < 1e-4 && hi.y > 0.79 && hi.y < 0.81,
        "{lo} {hi}"
    );
    let lamp = worlds.iter().find(|w| w.0 == "lamp").unwrap().1;
    assert!(
        lamp.transform_point3(Vec3::ZERO)
            .abs_diff_eq(Vec3::new(0.5, 0.65, 0.0), 1e-4)
    );

    let dir = folder(
        "linked",
        &[
            (
                "props-linked.fbx",
                &std::fs::read(fixture("props-linked.fbx")).unwrap(),
            ),
            ("crate.png", &std::fs::read(fixture("crate.png")).unwrap()),
        ],
    );
    let p = dir.join("props-linked.fbx");
    let linked = to_glb(&std::fs::read(&p).unwrap(), &p, "fbx").unwrap();
    let lj = json_of(&linked.glb);
    assert_eq!(lj["images"][0]["uri"], "crate.png");
    assert_eq!(lj["materials"], j["materials"]);
    assert_eq!(world_points(&linked.glb).0, pts);
}

/// Blender's skinned arm: one skin over root, upper and lower; the bulge
/// shape key a morph target named so; the clips wave (1 s: lower turns
/// 90 degrees about the engine's X, the bulge rises 0 to 1) and lift
/// (0.5 s), each starting at 0.
#[test]
fn a_skinned_fbx_keeps_its_joints_morphs_and_clips() {
    let i = import("arm.fbx");
    let g = gltf::Gltf::from_slice(&i.glb).unwrap();
    let blob = g.blob.as_deref().unwrap();
    let skin = g.skins().next().unwrap();
    let joints: Vec<&str> = skin.joints().map(|n| n.name().unwrap()).collect();
    assert_eq!(joints, ["root", "upper", "lower"]);
    let mesh = g.meshes().next().unwrap();
    let j = json_of(&i.glb);
    assert_eq!(j["meshes"][0]["extras"]["targetNames"], json!(["bulge"]));
    assert_eq!(mesh.primitives().next().unwrap().morph_targets().count(), 1);
    let clips: Vec<(String, f32)> = g
        .animations()
        .map(|a| {
            let end = a
                .channels()
                .map(|c| {
                    c.reader(|_| Some(blob))
                        .read_inputs()
                        .unwrap()
                        .fold(0f32, f32::max)
                })
                .fold(0f32, f32::max);
            (a.name().unwrap().to_string(), end)
        })
        .collect();
    assert_eq!(clips.len(), 2, "{clips:?}");
    assert!(
        clips
            .iter()
            .any(|(n, e)| n == "wave" && (e - 1.0).abs() < 1e-4),
        "{clips:?}"
    );
    assert!(
        clips
            .iter()
            .any(|(n, e)| n == "lift" && (e - 0.5).abs() < 1e-4),
        "{clips:?}"
    );
    let wave = g.animations().find(|a| a.name() == Some("wave")).unwrap();
    let lower = wave
        .channels()
        .find(|c| {
            c.target().node().name() == Some("lower")
                && c.target().property() == gltf::animation::Property::Rotation
        })
        .unwrap();
    let r = lower.reader(|_| Some(blob));
    let first = match r.read_outputs().unwrap() {
        gltf::animation::util::ReadOutputs::Rotations(q) => q.into_f32().collect::<Vec<_>>(),
        _ => unreachable!(),
    };
    let turned = Quat::from_array(*first.last().unwrap()) * Quat::from_array(first[0]).inverse();
    let (axis, angle) = turned.to_axis_angle();
    assert!((angle.to_degrees() - 90.0).abs() < 0.1, "{angle}");
    assert!(axis.abs().abs_diff_eq(Vec3::X, 1e-3), "{axis}");
    let weights = wave
        .channels()
        .find(|c| c.target().property() == gltf::animation::Property::MorphTargetWeights)
        .unwrap();
    let w: Vec<f32> = match weights.reader(|_| Some(blob)).read_outputs().unwrap() {
        gltf::animation::util::ReadOutputs::MorphTargetWeights(w) => w.into_f32().collect(),
        _ => unreachable!(),
    };
    assert!(
        w[0].abs() < 1e-4 && (w.last().unwrap() - 1.0).abs() < 1e-4,
        "{w:?}"
    );
}

/// A file of ufbx's corpus (`third_party/ufbx/data`).
fn ufbx_data(name: &str) -> Imported {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../third_party/ufbx/data")
        .join(name);
    to_glb(&std::fs::read(&p).unwrap(), &p, "fbx").unwrap()
}

/// The first mesh's node in world space: its vertices posed by the
/// first clip's morph weights at `t` (linear between keys, held past
/// the ends), its target names and its rest weights.
fn morphed_at(glb: &[u8], t: f32) -> (Vec<Vec3>, Vec<String>, Vec<f32>) {
    let g = gltf::Gltf::from_slice(glb).unwrap();
    let blob = g.blob.as_deref().unwrap();
    let (_, worlds) = world_points(glb);
    let node = g.nodes().find(|n| n.mesh().is_some()).unwrap();
    let w = worlds[node.index()].1;
    let mesh = node.mesh().unwrap();
    let j = json_of(glb);
    let names: Vec<String> = j["meshes"][mesh.index()]["extras"]["targetNames"]
        .as_array()
        .map(|a| a.iter().map(|n| n.as_str().unwrap().to_string()).collect())
        .unwrap_or_default();
    let rest: Vec<f32> = mesh.weights().map(<[f32]>::to_vec).unwrap_or_default();
    let mut weights = rest.clone();
    if let Some(a) = g.animations().next() {
        for c in a.channels() {
            if c.target().node().index() != node.index()
                || c.target().property() != gltf::animation::Property::MorphTargetWeights
            {
                continue;
            }
            let r = c.reader(|_| Some(blob));
            let times: Vec<f32> = r.read_inputs().unwrap().collect();
            let values: Vec<f32> = match r.read_outputs().unwrap() {
                gltf::animation::util::ReadOutputs::MorphTargetWeights(v) => v.into_f32().collect(),
                _ => unreachable!(),
            };
            let n = weights.len();
            let k = times.iter().position(|x| *x >= t);
            for (i, w) in weights.iter_mut().enumerate() {
                let v = |f: usize| values[f * n + i];
                *w = match k {
                    None => v(times.len() - 1),
                    Some(0) => v(0),
                    Some(f) => {
                        let s = (t - times[f - 1]) / (times[f] - times[f - 1]);
                        v(f - 1) + (v(f) - v(f - 1)) * s
                    }
                };
            }
        }
    }
    let mut pts = Vec::new();
    for p in mesh.primitives() {
        let r = p.reader(|_| Some(blob));
        let mut pos: Vec<Vec3> = r.read_positions().unwrap().map(Vec3::from).collect();
        for (k, target) in r.read_morph_targets().enumerate() {
            if let Some(d) = target.0 {
                for (p, d) in pos.iter_mut().zip(d) {
                    *p += Vec3::from(d) * weights[k];
                }
            }
        }
        pts.extend(pos.iter().map(|p| w.transform_point3(*p)));
    }
    (pts, names, rest)
}

/// Maya's blend-shape cube (ufbx's file and its expected values): each
/// channel a morph target with Maya's rest weight, and the clip drives
/// the two weights as Maya evaluates them at frames 16 and 53.
#[test]
fn blend_shapes_are_morph_targets_with_maya_s_weights() {
    let i = ufbx_data("maya_blend_shape_cube_7700_binary.fbx");
    let (_, names, rest) = morphed_at(&i.glb, 0.0);
    assert_eq!(names, ["Squish.TopH", "Squish.TopV"]);
    assert!(rest.iter().all(|w| (w - 1.0).abs() < 1e-6), "{rest:?}");
    let g = gltf::Gltf::from_slice(&i.glb).unwrap();
    let blob = g.blob.as_deref().unwrap();
    let c = g.animations().next().unwrap().channels().next().unwrap();
    let r = c.reader(|_| Some(blob));
    let times: Vec<f32> = r.read_inputs().unwrap().collect();
    let gltf::animation::util::ReadOutputs::MorphTargetWeights(v) = r.read_outputs().unwrap()
    else {
        panic!("weights")
    };
    let values: Vec<f32> = v.into_f32().collect();
    for (frame, want) in [(16.0f32, [0.279f32, 0.670]), (53.0, [0.901, 0.168])] {
        let t = (frame - 1.0) / 24.0;
        let k = times.iter().position(|x| *x >= t).unwrap();
        let f = (t - times[k - 1]) / (times[k] - times[k - 1]);
        let got = [0, 1].map(|n| values[(k - 1) * 2 + n] * (1.0 - f) + values[k * 2 + n] * f);
        for n in 0..2 {
            assert!((got[n] - want[n]).abs() < 2e-3, "frame {frame}: {got:?}");
        }
    }
}

/// A channel with shapes on the way to its full one (ufbx's file, and
/// three frames of it as Maya wrote them out): each shape is a target,
/// the clip gives each its part, and the mesh stands where Maya's does.
#[test]
fn in_between_shapes_are_targets_of_their_own() {
    let i = ufbx_data("maya_blend_inbetween_7500_binary.fbx");
    let (_, names, _) = morphed_at(&i.glb, 0.0);
    assert_eq!(
        names,
        [
            "KeyBlend.LiftTop@50",
            "KeyBlend.LiftTop",
            "KeyBlend.MoveToSide@-49.9",
            "KeyBlend.MoveToSide@-24.9",
            "KeyBlend.MoveToSide"
        ]
    );
    for frame in [30, 65, 89] {
        let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../../third_party/ufbx/data/maya_blend_inbetween_{frame}.obj"
        ));
        // Maya's corners, in its centimetres with Y up.
        let want: Vec<Vec3> = std::fs::read_to_string(p)
            .unwrap()
            .lines()
            .filter_map(|l| l.strip_prefix("v "))
            .map(|l| {
                let v: Vec<f32> = l.split_whitespace().map(|x| x.parse().unwrap()).collect();
                Vec3::new(v[0], v[1], v[2]) * 0.01
            })
            .collect();
        let (got, _, _) = morphed_at(&i.glb, (frame as f32 - 1.0) / 24.0);
        // The cook welds corners into vertices: each Maya corner has one
        // within 0.05 mm, and each vertex a corner. The shapes and their
        // shares are exact; what is left is ufbx's evaluation of Maya's
        // eased curve (its weights up to 0.004 from Maya's at frame 89,
        // 0.031 mm, at any bake rate).
        for w in &want {
            let d = got
                .iter()
                .map(|p| (*p - *w).length())
                .fold(f32::MAX, f32::min);
            assert!(
                d < 5e-5,
                "frame {frame}: {w:?} is {d} m from the nearest vertex"
            );
        }
        for p in &got {
            let d = want
                .iter()
                .map(|w| (*p - *w).length())
                .fold(f32::MAX, f32::min);
            assert!(d < 5e-5, "frame {frame}: {p:?} is {d} m from Maya's");
        }
    }
}
