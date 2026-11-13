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
