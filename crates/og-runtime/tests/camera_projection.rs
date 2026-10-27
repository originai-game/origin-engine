//! camera.worldToScreen and camera.screenToRay project through og-core's
//! sin/cos, never glam's `perspective_*` (which call the host libm), so a
//! system that aims or picks with them reads the same bits on every
//! platform. Their answers are checked against a projection computed in
//! f64, then pinned to the bit.
//!
//! Ways this could fail: the projection goes back to a host-libm
//! constructor (the pinned bits move on one platform and not another), the
//! field of view is taken as the full angle or in radians, the aspect ratio
//! or the y flip is lost, or the ray is not unprojected through the camera's
//! turn and place.

use og_runtime::{
    og_camera_screen_to_ray, og_camera_world_to_screen, og_engine_create, og_engine_write_resource,
    og_game_game, og_host_begin_bundle,
};
use serde_json::{Value, json};

/// A yaw of 0.5 rad about +Y as a quaternion, written out so the test does
/// not compute it: (sin 0.25, cos 0.25).
const YAW: [f64; 4] = [0.0, 0.247_403_959_254_522_9, 0.0, 0.968_912_421_710_644_8];

fn engine(camera: Value) -> u32 {
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({ "seed": 1 }).to_string()));
    let v: Value = serde_json::from_str(&og_engine_create(game, "{}".into())).unwrap();
    let e = v["value"].as_u64().unwrap() as u32;
    let w: Value = serde_json::from_str(&og_engine_write_resource(
        e,
        "og.Camera".into(),
        camera.to_string(),
    ))
    .unwrap();
    assert_eq!(w["ok"], json!(true), "{w}");
    e
}

fn bits(v: &Value) -> u32 {
    (v.as_f64().unwrap() as f32).to_bits()
}

/// The reference in f64: rotate `p - eye` by the inverse yaw, then the
/// perspective of three.js (fov the full vertical angle, in degrees).
// Test reference values: libm checks the result, never the tick.
#[allow(clippy::disallowed_methods)]
fn reference(eye: [f64; 3], p: [f64; 3], fov: f64, w: f64, h: f64) -> (f64, f64) {
    let (s, c) = (2.0 * YAW[1] * YAW[3], YAW[3] * YAW[3] - YAW[1] * YAW[1]);
    let (dx, dy, dz) = (p[0] - eye[0], p[1] - eye[1], p[2] - eye[2]);
    // Inverse of a yaw by angle a: x' = c x - s z, z' = s x + c z.
    let (x, y, z) = (c * dx - s * dz, dy, s * dx + c * dz);
    let f = 1.0 / (fov.to_radians() / 2.0).tan();
    let (nx, ny) = (f / (w / h) * x / -z, f * y / -z);
    ((nx + 1.0) / 2.0 * w, (1.0 - ny) / 2.0 * h)
}

#[test]
fn the_camera_projects_and_unprojects_through_og_core_to_the_bit() {
    let eye = [1.0, 2.0, 6.0];
    let e = engine(json!({
        "position": { "x": eye[0], "y": eye[1], "z": eye[2] },
        "quaternion": { "x": YAW[0], "y": YAW[1], "z": YAW[2], "w": YAW[3] },
        "fov": 55.0,
    }));
    let (w, h) = (1280.0, 720.0);

    let p = [-0.75, 1.25, -3.0];
    let screen: Value = serde_json::from_str(&og_camera_world_to_screen(
        e,
        json!({ "x": p[0], "y": p[1], "z": p[2] }).to_string(),
    ))
    .unwrap();
    let (rx, ry) = reference(eye, p, 55.0, w, h);
    let (sx, sy) = (screen["x"].as_f64().unwrap(), screen["y"].as_f64().unwrap());
    assert!(
        (sx - rx).abs() < 1e-3 && (sy - ry).abs() < 1e-3,
        "worldToScreen {screen} against the f64 projection ({rx}, {ry})"
    );

    // The ray through that pixel starts on the near plane and points at p.
    let ray: Value = serde_json::from_str(&og_camera_screen_to_ray(e, sx, sy)).unwrap();
    let (o, d) = (&ray["origin"], &ray["direction"]);
    let to = [p[0] - eye[0], p[1] - eye[1], p[2] - eye[2]];
    let len = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt();
    for (axis, k) in ["x", "y", "z"].iter().zip(0..) {
        let got = d[axis].as_f64().unwrap();
        assert!(
            (got - to[k] / len).abs() < 1e-4,
            "screenToRay {ray} toward {to:?}"
        );
    }

    // Pinned: these bits are what every platform must answer, the host's
    // libm notwithstanding.
    let pinned = [
        bits(&screen["x"]),
        bits(&screen["y"]),
        bits(&o["x"]),
        bits(&o["y"]),
        bits(&o["z"]),
        bits(&d["x"]),
        bits(&d["y"]),
        bits(&d["z"]),
    ];
    assert_eq!(
        pinned.map(|b| format!("{b:#010x}")),
        PINNED.map(|b| format!("{b:#010x}")),
        "screen {screen}, ray {ray}"
    );
}

const PINNED: [u32; 8] = [
    0x4456fda2, 0x43d1ae6a, 0x3f7adf64, 0x3ffee6b8, 0x40bcb42b, 0xbe42cc59, 0xbda6f961, 0xbf7a7501,
];
