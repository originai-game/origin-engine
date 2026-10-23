//! A small Rust world the probe tests drive: a player that moves by its
//! tunable speed and steers by the `og.Input` resource, a crate, three
//! unnamed rocks, a camera, and a system drawing from the world's Rng.
#![allow(dead_code)]

use og_core::glam::{Quat, Vec3};
use og_core::{Reflect, Rng};
use og_ecs::{QueryDesc, SystemDesc, World, WorldDesc};
use og_probe::{Capture, Frame, Input, Pick, Probe, StaticUi, UiElement};
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.Transform")]
pub struct Transform {
    pub position: Vec3,
    pub quaternion: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn at(x: f32, y: f32, z: f32) -> Transform {
        Transform {
            position: Vec3::new(x, y, z),
            quaternion: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

/// How fast an entity moves along its velocity.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub struct Mover {
    /// Metres per second along the velocity.
    #[reflect(tunable, min = 0, max = 20, label = "Speed", unit = "m/s", default = 2)]
    pub speed: f32,
    /// A lane index; not tunable.
    pub lane: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub struct Velocity {
    pub v: Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub struct Health {
    #[reflect(tunable, min = 0, max = 1000, label = "Hit points")]
    pub hp: f32,
    pub max: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.Camera")]
pub struct Camera {
    pub position: Vec3,
    pub quaternion: Quat,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
pub struct Stats {
    pub draws: u64,
    pub sum: f64,
}

pub struct Harness {
    pub world: World,
    pub probe: Probe,
}

pub fn ui() -> StaticUi {
    let el = |id: &str, parent: &str, text: &str, x: f32, y: f32| UiElement {
        id: id.into(),
        parent: parent.into(),
        text: text.into(),
        x,
        y,
        width: 100.0,
        height: 40.0,
        opaque: false,
        ..UiElement::default()
    };
    StaticUi(vec![
        el("Menu", "", "", 0.0, 0.0),
        el("Start", "Menu", "Start game", 10.0, 10.0),
        el("Options", "Menu", "Options", 10.0, 60.0),
        el("Dialog", "", "", 200.0, 0.0),
        el("Start", "Dialog", "Start", 210.0, 10.0),
    ])
}

/// A capture that renders nothing real: the "PNG" names the size, the tick
/// and the player's x, so frames differ exactly when the world does.
pub struct FakeCapture;

impl Capture for FakeCapture {
    fn tier(&self) -> &str {
        "vulkan"
    }
    fn target_size(&self) -> (u32, u32) {
        (64, 36)
    }
    fn capture(&mut self, world: &World, width: u32, height: u32) -> Option<Frame> {
        let (width, height) = if width == 0 {
            (64, 36)
        } else {
            (width, height)
        };
        let x: f32 = world
            .query(&QueryDesc::new().read::<Transform>().read::<Mover>())
            .ok()?
            .flat_map(|v| {
                v.read::<Vec3>(world.field::<Transform>("position").unwrap())
                    .unwrap()
                    .to_vec()
            })
            .map(|p| p.x)
            .sum();
        Some(Frame {
            width,
            height,
            png: format!("{width}x{height} tick={} x={x:.4}", world.time().tick).into_bytes(),
        })
    }
    fn pick(&mut self, world: &World, x: f32, y: f32) -> Option<Pick> {
        let player = world
            .query(&QueryDesc::new().read::<Mover>())
            .ok()?
            .flat_map(|v| v.entities().to_vec())
            .next()?;
        (x < 10.0).then(|| Pick {
            entity: player,
            world: vec![x, y, 0.0],
            source: Some("player.ts::move".into()),
        })
    }
}

/// The web's readback over [`FakeCapture`]: the first call at a size draws
/// the world as it is then and answers `None`; the next call at that size
/// answers that frame.
#[derive(Default)]
pub struct ReadbackCapture(Vec<((u32, u32), Frame)>);

impl Capture for ReadbackCapture {
    fn tier(&self) -> &str {
        "webgpu"
    }
    fn target_size(&self) -> (u32, u32) {
        (64, 36)
    }
    fn capture(&mut self, world: &World, width: u32, height: u32) -> Option<Frame> {
        if let Some(i) = self.0.iter().position(|(size, _)| *size == (width, height)) {
            return Some(self.0.remove(i).1);
        }
        let frame = FakeCapture.capture(world, width, height)?;
        self.0.push(((width, height), frame));
        None
    }
    fn deferred(&self) -> bool {
        true
    }
}

/// The game's types and systems in a fresh world: what a bundle declares.
/// `with_health: false` is a bundle that dropped the Health component.
pub fn bundle(seed: u32, with_health: bool, declare: &mut dyn FnMut(&SystemDesc)) -> World {
    let mut world = World::new(&WorldDesc {
        tick_rate: 60,
        seed,
    })
    .unwrap();
    world.register::<Transform>().unwrap();
    world.register::<Mover>().unwrap();
    world.register::<Velocity>().unwrap();
    if with_health {
        world.register::<Health>().unwrap();
    }
    world.register::<Camera>().unwrap();
    world.register::<Stats>().unwrap();
    world.register::<Input>().unwrap();

    let steer = SystemDesc::new("steer")
        .read::<Input>()
        .write::<Velocity>()
        .query(QueryDesc::new().write::<Velocity>());
    let vel = world.field::<Velocity>("v").unwrap();
    declare(&steer);
    world
        .add_system(steer, move |ctx| {
            let axis = ctx.resource::<Input>()?.axis("ArrowLeft", "ArrowRight");
            for a in ctx.query(0)? {
                for v in a.write::<Vec3>(vel)?.iter_mut() {
                    v.x = 1.0 + 2.0 * axis;
                }
            }
            Ok(())
        })
        .unwrap();

    let movement = SystemDesc::new("move")
        .read::<Velocity>()
        .read::<Mover>()
        .write::<Transform>()
        .query(
            QueryDesc::new()
                .read::<Velocity>()
                .read::<Mover>()
                .write::<Transform>(),
        );
    let pos = world.field::<Transform>("position").unwrap();
    let speed = world.field::<Mover>("speed").unwrap();
    declare(&movement);
    world
        .add_system(movement, move |ctx| {
            let dt = ctx.time().dt;
            for a in ctx.query(0)? {
                let v = a.read::<Vec3>(vel)?;
                let s = a.read::<f32>(speed)?;
                for ((p, v), s) in a.write::<Vec3>(pos)?.iter_mut().zip(v.iter()).zip(s.iter()) {
                    *p += *v * *s * dt;
                }
            }
            Ok(())
        })
        .unwrap();

    // Undeclared to the probe on purpose: dump_world reports what it cannot know as null.
    world
        .add_system(
            SystemDesc::new("draw").write::<Stats>().write::<Rng>(),
            |ctx| {
                let r = ctx.resource_mut::<Rng>()?.next_f64();
                let mut stats = ctx.resource_mut::<Stats>()?;
                stats.draws += 1;
                stats.sum += r;
                Ok(())
            },
        )
        .unwrap();
    world
}

pub fn build(seed: u32, with_renderer: bool) -> Harness {
    let mut world = bundle(seed, true, &mut |_| {});
    let mut probe = Probe::new(&mut world).unwrap();
    world
        .insert_resource(Camera {
            position: Vec3::new(0.0, 0.0, 10.0),
            quaternion: Quat::IDENTITY,
        })
        .unwrap();
    world.insert_resource(Stats::default()).unwrap();

    let player = world
        .spawn((
            Transform::at(0.0, 0.0, 0.0),
            Velocity { v: Vec3::X },
            Mover {
                speed: 2.0,
                lane: 1,
            },
            Health {
                hp: 100.0,
                max: 100.0,
            },
        ))
        .unwrap();
    let crate_ = world
        .spawn((
            Transform::at(5.0, 0.0, 0.0),
            Health {
                hp: 30.0,
                max: 30.0,
            },
        ))
        .unwrap();
    for i in 0..3 {
        world
            .spawn((Transform::at(-1.0 - i as f32, 0.0, 0.0),))
            .unwrap();
    }
    world.set_name(player, "player").unwrap();
    world.set_name(crate_, "crate").unwrap();
    probe.set_ui(Box::new(ui()));
    probe.set_facts(|w| json!({"draws": w.resource::<Stats>().map(|s| s.draws).unwrap_or(0)}));
    if with_renderer {
        probe.set_capture(Box::new(FakeCapture));
    }
    Harness { world, probe }
}

impl Harness {
    /// One JSON-RPC call through the transport-free entry; the whole response.
    pub fn call(&mut self, method: &str, params: Value) -> Value {
        let frame =
            json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}).to_string();
        self.raw(&frame)
    }

    pub fn raw(&mut self, frame: &str) -> Value {
        serde_json::from_str(&self.probe.call(&mut self.world, frame)).unwrap()
    }

    pub fn result(&mut self, method: &str, params: Value) -> Value {
        let r = self.call(method, params);
        assert!(r.get("error").is_none(), "{method} answered an error: {r}");
        r["result"].clone()
    }

    pub fn player_x(&self) -> f32 {
        let e = self.probe.names().resolve(&self.world, "player").unwrap();
        self.world.get::<Transform>(e).unwrap().position.x
    }
}

pub fn base64_decode(text: &str) -> Vec<u8> {
    let value = |c: u8| -> u32 {
        match c {
            b'A'..=b'Z' => u32::from(c - b'A'),
            b'a'..=b'z' => u32::from(c - b'a') + 26,
            b'0'..=b'9' => u32::from(c - b'0') + 52,
            b'+' => 62,
            b'/' => 63,
            _ => panic!("not base64: {c}"),
        }
    };
    let mut out = Vec::new();
    for chunk in text.as_bytes().chunks(4) {
        let data: Vec<u8> = chunk.iter().copied().filter(|c| *c != b'=').collect();
        let n = data
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, c)| n | (value(*c) << (18 - 6 * i)));
        for i in 0..data.len().saturating_sub(1) {
            out.push((n >> (16 - 8 * i)) as u8);
        }
    }
    out
}
