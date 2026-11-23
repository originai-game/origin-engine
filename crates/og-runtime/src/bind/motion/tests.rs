use super::*;

fn walk(acceleration: f32) -> Goal {
    Goal {
        kind: Kind::WalkTo,
        pose: 0,
        target: Vec3::new(2.0, 0.0, 0.0),
        heading: 0.0,
        duration: 0.0,
        speed: 1.4,
        arrival_heading: None,
        deceleration: 3.0,
        acceleration,
        height: 0.0,
        prepare: 0.0,
    }
}

#[test]
fn a_motion4_save_restores_with_its_walks_at_full_speed_from_rest() {
    // MOTION4 by hand: one agent, one queued walk, no current goal.
    let mut b = b"MOTION4\0".to_vec();
    let u = |b: &mut Vec<u8>, v: u32| b.extend_from_slice(&v.to_le_bytes());
    let f = |b: &mut Vec<u8>, v: f32| b.extend_from_slice(&v.to_le_bytes());
    u(&mut b, 1);
    b.extend_from_slice(&7u64.to_le_bytes());
    for v in [0, 0, 0, 1] {
        u(&mut b, v);
    }
    let g = walk(0.0);
    u(&mut b, g.kind as u32);
    u(&mut b, g.pose);
    for v in [2.0, 0.0, 0.0, 0.0, 0.0, 1.4] {
        f(&mut b, v);
    }
    u(&mut b, 0);
    f(&mut b, 0.0);
    f(&mut b, 3.0);
    f(&mut b, 0.0);
    f(&mut b, 0.0);
    u(&mut b, 0);
    let perf = serde_json::to_vec(&performance::Performance::default()).unwrap();
    u(&mut b, perf.len() as u32);
    b.extend_from_slice(&perf);
    let mut goals = load(&b).unwrap();
    let a = goals.agents.get_mut(&7).unwrap();
    assert_eq!(a.speed, 0.0);
    assert_eq!(a.queue[0], walk(f32::INFINITY));
    // As MOTION4 ran it: at full speed in the first tick.
    let s = Settings::of(&wire::record::MotionAgent {
        drive: true,
        ..Default::default()
    });
    let (mut p, mut h) = (Vec3::ZERO, std::f32::consts::FRAC_PI_2);
    advance(a, &s, &mut p, &mut h, 1.0 / 60.0);
    assert!((p.x - 1.4 / 60.0).abs() < 1e-6, "{}", p.x);

    // MOTION5 keeps a walk's acceleration and the mover's speed.
    a.queue.push_back(walk(ACCELERATION));
    let mut out = Vec::new();
    save(&goals, &mut out);
    assert!(out.starts_with(b"MOTION5\0"));
    assert_eq!(load(&out).unwrap(), goals);
}

#[test]
fn the_blocks_form_names_each_agent_with_its_values() {
    use og_core::block_form::{BlockForm, Field};
    let mut goals = Goals::default();
    let a = goals.agents.entry((3 << 32) | 9).or_default();
    a.completed = 2;
    a.speed = 0.75;
    a.queue.push_back(walk(ACCELERATION));
    let mut bytes = Vec::new();
    save(&goals, &mut bytes);
    let block = state::Block(Arc::new(Mutex::new(Goals::default())));
    let BlockForm::Entities(r) = block.form(&bytes).unwrap() else {
        panic!("og.motion's agents are by entity");
    };
    assert!(r.global.is_empty());
    assert_eq!(r.entities.len(), 1);
    let agent = &r.entities[0];
    assert_eq!(
        (agent.entity, agent.kind.as_str()),
        ((3 << 32) | 9, "agent")
    );
    let get = |n: &str| {
        agent
            .fields
            .iter()
            .find(|(k, _)| k == n)
            .map(|(_, f)| f.clone())
    };
    assert_eq!(get("completed"), Some(Field::of(&2u32)));
    assert_eq!(get("speed"), Some(Field::of(&0.75f32)));
    let Some(Field::Value(queue)) = get("queue") else {
        panic!("the queue is a value");
    };
    assert_eq!(queue[0]["kind"], "walkTo");
    assert_eq!(queue[0]["target"], serde_json::json!([2.0, 0.0, 0.0]));
    // The block it was asked through is unchanged.
    assert!(lock(&block.0).agents.is_empty());
    assert!(block.form(&bytes[..bytes.len() - 1]).is_err());
}
