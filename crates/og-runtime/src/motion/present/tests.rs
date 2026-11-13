use super::*;
use og_anim::motion::contract::{FRAMES, MOTION_FEATURES};

fn shown() -> Shown {
    Shown {
        character: 0,
        prev: None,
        next: None,
        phase: 0.0,
        serial: 0,
        requested: None,
        planter: Planter::new(Vec::new()),
        skeleton: 1,
        rig: 42,
        target: None,
        failure: None,
        last_position: Vec3::ZERO,
        velocity: Vec3::ZERO,
        shown: 0,
        starved: 0,
        warmup: 0,
        facing: 0.0,
        planted: [None; 2],
        slide: (0.0, 0, 0.0),
        lean: None,
        drawn_lean: None,
        warp: 1.0,
        feet: [Foot::default(); 2],
        offset: Vec3::ZERO,
        followed: None,
        footing: Footing::default(),
        feet_ahead: None,
        issued: VecDeque::new(),
        next_id: 0,
        clip: None,
        pending: VecDeque::new(),
    }
}

#[test]
fn presentation_reports_load_failure_through_api_and_probe() {
    use crate::{
        og_engine_create, og_game_game, og_host_begin_bundle, og_motion_presentation,
        og_world_insert, og_world_spawn,
    };
    og_host_begin_bundle();
    let game = og_game_game(None);
    let created: Value = serde_json::from_str(&og_engine_create(game, "{}".into())).unwrap();
    let world = created["value"].as_u64().unwrap() as u32;
    let spawned: Value =
        serde_json::from_str(&crate::host_impl::answer(og_world_spawn(world, None))).unwrap();
    let entity = spawned["value"].as_f64().unwrap();
    let inserted: Value = serde_json::from_str(&crate::host_impl::answer(og_world_insert(
        world,
        entity,
        AGENT.into(),
        None,
    )))
    .unwrap();
    assert_eq!(inserted["ok"], true);
    let eng = crate::engine::engine(world).unwrap();
    eng.render.borrow_mut().motion.model = Model::Failed("weights could not be read".into());
    let expected = json!({"state":"failed", "reason":"weights could not be read"});
    let answer: Value = serde_json::from_str(&crate::host_impl::answer(og_motion_presentation(
        world, entity,
    )))
    .unwrap();
    assert_eq!(answer["ok"], true);
    assert_eq!(answer["value"], expected);
    assert_eq!(
        super::super::dump(world)["agents"][0]["presentation"],
        expected
    );
}

#[test]
fn jump_clip_lands_on_the_simulations_landing() {
    // Five frames standing, ten in the air, five landed.
    let pose = Pose {
        positions: [[0.0; 3]; og_anim::motion::contract::JOINTS],
        rotations: [[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
            og_anim::motion::contract::JOINTS],
    };
    let frame = |index: u32| Frame {
        pose,
        contacts: if (5..15).contains(&index) {
            [0.0; 4]
        } else {
            [1.0; 4]
        },
        tag: 0,
        index,
    };
    let mut motion = Motion::default();
    let mut s = shown();
    s.character = motion.add([pose; 4]);
    s.prev = Some(frame(0));
    s.next = Some(frame(1));
    s.pending.extend((2..20).map(frame));
    s.issued.push_back(Issued {
        id: 0,
        pose: JUMP,
        start: 0.0,
    });
    let (v0, gravity) = (3.13, 9.81);
    let goal = super::super::Goal {
        kind: Kind::Jump,
        pose: 0,
        duration: 2.0,
        target: Vec3::ZERO,
        heading: 0.0,
        speed: 0.0,
        arrival_heading: None,
        deceleration: 0.0,
        acceleration: 0.0,
        height: 0.5,
        prepare: 0.25,
    };
    let jump = |stage, velocity, landed| super::super::Jump {
        stage,
        v0,
        gravity,
        base: 0.0,
        air: 0.0,
        landed,
        velocity,
        left: true,
        rest: None,
        eta: None,
    };
    let mut agent = super::super::Agent::default();
    let at = |agent: &mut super::super::Agent, j: Option<super::super::Jump>| {
        agent.current = j.map(|j| super::super::Current {
            goal,
            elapsed: 0.5,
            start: Vec3::ZERO,
            start_heading: 0.0,
            jump: Some(j),
        });
    };
    let drawn = |s: &Shown| s.prev.unwrap().index as f32 + s.phase;
    // At the apex: halfway through the clip's flight.
    at(&mut agent, Some(jump(JumpStage::Air, 0.0, 0.0)));
    jump_clip(&mut motion, &mut s, Some(&agent), 1.0);
    assert_eq!(drawn(&s), 10.0);
    // Falling past the takeoff height: held before the clip's
    // touchdown until the simulation lands.
    at(&mut agent, Some(jump(JumpStage::Air, -2.0 * v0, 0.0)));
    jump_clip(&mut motion, &mut s, Some(&agent), 1.0);
    assert_eq!(drawn(&s), 14.0);
    // Landed: the touchdown frame, reached at no more than CATCH_UP
    // times the clip's rate.
    at(&mut agent, Some(jump(JumpStage::Landed, 0.0, 0.0)));
    jump_clip(&mut motion, &mut s, Some(&agent), 0.25 / (CATCH_UP * RATE));
    assert_eq!(drawn(&s), 14.25);
    jump_clip(&mut motion, &mut s, Some(&agent), 1.0);
    assert_eq!(drawn(&s), 15.0);
    // The goal over: the frames after the drawn one play on.
    at(&mut agent, None);
    jump_clip(&mut motion, &mut s, Some(&agent), 1.0);
    assert!(s.clip.is_none());
    assert_eq!((s.prev.unwrap().index, s.next.unwrap().index), (15, 16));
    assert_eq!(
        s.pending.iter().map(|f| f.index).collect::<Vec<_>>(),
        [17, 18, 19]
    );

    // A jump starting before frames generated toward a walk end: the
    // walk's frames past the one after next are dropped and the model
    // starts again from the kept ones, to be asked for the jump.
    let mut s = shown();
    s.character = motion.add([pose; 4]);
    s.issued.push_back(Issued {
        id: 0,
        pose: 1,
        start: 0.0,
    });
    s.prev = Some(frame(0));
    s.next = Some(frame(1));
    s.pending.extend((2..20).map(frame));
    at(&mut agent, Some(jump(JumpStage::Prepare, 0.0, 0.0)));
    jump_clip(&mut motion, &mut s, Some(&agent), 0.0);
    let clip = s.clip.as_ref().unwrap();
    assert_eq!(
        clip.frames.iter().map(|f| f.index).collect::<Vec<_>>(),
        [0, 1, 2]
    );
    assert_eq!((s.ahead(&motion), motion.goals(s.character)), (2, 0));
}

#[test]
fn initial_face_to_idle_keeps_the_first_generated_window() {
    let a = Seen {
        entity: 1,
        settings: Settings::of(&record::MotionAgent {
            drive: true,
            ..Default::default()
        }),
        position: Vec3::new(2.0, 0.0, -3.0),
        rotation: glam::Quat::IDENTITY,
        scale: 1.0,
        skeleton: 1,
        skin: 0,
    };
    let face = super::super::Goal {
        kind: Kind::Face,
        pose: 0,
        duration: 0.1,
        target: Vec3::ZERO,
        heading: std::f32::consts::PI + 0.25,
        speed: 0.0,
        arrival_heading: None,
        deceleration: 0.0,
        acceleration: 0.0,
        height: 0.0,
        prepare: 0.0,
    };
    let mut goals = Goals::default();
    let mut agent = super::super::Agent::default();
    agent.queue.push_back(face);
    agent.queue.push_back(super::super::Goal {
        kind: Kind::Pose,
        duration: 600.0,
        ..face
    });
    goals.agents.insert(a.entity, agent);
    let mut s = shown();
    let mut pose = Pose {
        positions: [[0.0; 3]; og_anim::motion::contract::JOINTS],
        rotations: [[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
            og_anim::motion::contract::JOINTS],
    };
    pose.positions[LEFT_LEG][0] = -0.1;
    pose.positions[RIGHT_LEG][0] = 0.1;
    let request = |goals: &Goals, a: &Seen, s: &Shown| {
        let (at, h, index, seconds) = predict(goals, a, s, 0.0);
        assert_eq!(index, 0);
        let mut goal = Goal::placed(&[pose], [at.x, at.z], h);
        goal.token_counts = token_counts(seconds);
        goal
    };
    let first = request(&goals, &a, &s);
    let mut motion = Motion::default();
    s.character = motion.add([pose; 4]);
    s.requested = Some(first.clone());
    motion.push(s.character, first.clone());
    let batch = motion.batch(1).unwrap();
    let mut a = a;
    let mut h = 0.0;
    let agent = goals.agents.get_mut(&a.entity).unwrap();
    for _ in 0..60 {
        advance(agent, &a.settings, &mut a.position, &mut h, 1.0 / 60.0);
    }
    assert_ne!(agent.serial, s.serial);
    a.rotation = rotation_of(h);
    let next = request(&goals, &a, &s);
    s.reconcile_request(&mut motion, &next);
    let output = vec![0.0; FRAMES * MOTION_FEATURES];
    motion.accept_character(&batch, 0, 12, &output);
    assert!(
        motion.pop(s.character).is_some(),
        "initial window was discarded"
    );

    let mut changed_pose = first.clone();
    changed_pose.targets[0].positions[5][1] += 0.01;
    for changed in [
        Goal {
            targets: Goal::placed(&[pose], [2.001, -3.0], h).targets,
            ..first.clone()
        },
        changed_pose,
        Goal {
            token_counts: vec![6],
            ..first.clone()
        },
    ] {
        let mut motion = Motion::default();
        s.character = motion.add([pose; 4]);
        motion.push(s.character, first.clone());
        let batch = motion.batch(1).unwrap();
        s.reconcile_request(&mut motion, &changed);
        motion.accept_character(&batch, 0, 12, &output);
        assert!(
            motion.pop(s.character).is_none(),
            "changed request survived"
        );
    }
}

#[test]
fn prediction_keeps_a_short_new_pose_and_merges_the_rest_of_an_asked_one() {
    let goal = |pose, duration| super::super::Goal {
        kind: Kind::Pose,
        pose,
        duration,
        target: Vec3::ZERO,
        heading: 0.0,
        speed: 0.0,
        arrival_heading: None,
        deceleration: 0.0,
        acceleration: 0.0,
        height: 0.0,
        prepare: 0.0,
    };
    let a = Seen {
        entity: 1,
        settings: Settings::of(&record::MotionAgent {
            drive: true,
            ..Default::default()
        }),
        position: Vec3::ZERO,
        rotation: glam::Quat::IDENTITY,
        scale: 1.0,
        skeleton: 1,
        skin: 0,
    };
    let crouch = og_anim::motion::library::index("crouch").unwrap();
    let wave = og_anim::motion::library::index("wave").unwrap();
    for (pose, asked, kept) in [
        (JUMP, None, true),
        (wave, None, true),
        (wave, Some(0), true),
        // The last goal asked for the wave: its last 0.2 s belong to
        // the crouch's goal, so the model does not stand behind the
        // simulation finishing it.
        (wave, Some(wave), false),
        // A jump's rise always keeps its own goal.
        (JUMP, Some(JUMP), true),
    ] {
        let mut goals = Goals::default();
        let mut agent = super::super::Agent::default();
        agent.queue.push_back(goal(pose as u32, 0.2));
        agent.queue.push_back(goal(crouch as u32, 1.2));
        goals.agents.insert(1, agent);
        let mut s = shown();
        s.issued.extend(asked.map(|pose| Issued {
            id: 0,
            pose,
            start: 0.0,
        }));
        let (_, _, predicted, seconds) = predict(&goals, &a, &s, 0.0);
        if kept {
            assert_eq!(predicted, pose, "the imminent pose was skipped ({asked:?})");
            assert!(seconds <= 0.3);
        } else {
            assert_eq!(
                predicted, crouch,
                "the rest of the asked pose kept its own goal"
            );
        }
    }
}

#[test]
fn prediction_never_merges_a_change_of_gait() {
    // The model is walking (the last goal asked for a walk) and the
    // simulation runs 1 m at 3.6 m/s, then walks on: the run is shorter
    // than the shortest generation, but a change of gait keeps its own
    // goal instead of being folded into the walk after it.
    let walk = |x: f32, speed: f32| super::super::Goal {
        kind: Kind::WalkTo,
        pose: 0,
        target: Vec3::new(x, 0.0, 0.0),
        heading: 0.0,
        duration: 0.0,
        speed,
        arrival_heading: None,
        deceleration: 3.0,
        acceleration: 1000.0,
        height: 0.0,
        prepare: 0.0,
    };
    let a = Seen {
        entity: 1,
        settings: Settings::of(&record::MotionAgent {
            drive: true,
            ..Default::default()
        }),
        position: Vec3::ZERO,
        rotation: rotation_of(std::f32::consts::FRAC_PI_2),
        scale: 1.0,
        skeleton: 1,
        skin: 0,
    };
    let mut goals = Goals::default();
    let mut agent = super::super::Agent::default();
    agent.queue.push_back(walk(1.0, 3.6));
    agent.queue.push_back(walk(4.0, 1.3));
    goals.agents.insert(1, agent);
    let mut s = shown();
    s.issued.push_back(Issued {
        id: 0,
        pose: gait(1.3),
        start: 0.0,
    });
    let (at, _, pose, seconds) = predict(&goals, &a, &s, 0.0);
    assert_eq!(pose, gait(3.6), "the run was merged into the walk");
    assert!(seconds < MIN_SEGMENT, "{seconds} s");
    assert!(
        (at.x - 1.0).abs() < 0.05,
        "the run's goal ends at x {}",
        at.x
    );
}

/// A SOMA-named skeleton standing upright: hips 1 m up, legs of two
/// 0.45 m bones, feet forward.
fn standing() -> Skeleton {
    let parent = |j: usize| match j {
        0 => NONE,
        1..=9 => j as u32 - 1,
        10..=21 => 3,
        22 | 26 => 0,
        _ => j as u32 - 1,
    };
    let offset = |j: usize| match j {
        22 => Vec3::new(0.1, -0.05, 0.0),
        26 => Vec3::new(-0.1, -0.05, 0.0),
        23 | 24 | 27 | 28 => Vec3::new(0.0, -0.45, 0.0),
        25 | 29 => Vec3::new(0.0, -0.05, 0.12),
        _ => Vec3::new(0.0, 0.12, 0.0),
    };
    let mut rest: [Trs; og_anim::motion::contract::JOINTS] =
        std::array::from_fn(|j| Trs::new(offset(j), glam::Quat::IDENTITY, Vec3::ONE));
    rest[0].translation = Vec3::new(0.0, 1.0, 0.0);
    Skeleton {
        parents: std::array::from_fn(parent),
        rest,
    }
}

/// Planted feet outrank an authored reaction: a reaction that lowers and
/// pitches the hips leaves both planted ankles on their anchors, one
/// whose clip steps the left foot moves that foot only, and one turned
/// 90° from the facing steps both (planted, the legs would cross).
#[test]
fn a_planter_on_the_ground_puts_feet_down_and_steps_them_one_at_a_time() {
    // Ways this could fail: a foot planted where the pose holds it up
    // (a floating foot), a foot left behind sliding to its pose along
    // the ground instead of stepping, both feet stepping at once.
    let asset = standing().asset();
    let map = HumanoidMap::detect(&asset);
    let at_rest = asset.model_space(&asset.rest_pose());
    let legs: Vec<Leg> = [(14usize, 0usize), (18, 2)]
        .iter()
        .map(|&(b, contact)| Leg {
            chain: [map.nodes[b], map.nodes[b + 1], map.nodes[b + 2]],
            contact,
            rest: at_rest[map.nodes[b + 2] as usize].translation.y,
        })
        .collect();
    let rest_y = legs[0].rest;
    // Knees bent: the generated ankles are held 0.12 m over the ground.
    let mut pose = asset.rest_pose();
    for (hip, knee) in [(22, 23), (26, 27)] {
        pose[hip].rotation = glam::Quat::from_rotation_x(-0.6);
        pose[knee].rotation = glam::Quat::from_rotation_x(1.2);
    }
    let ankles = legs.iter().map(|l| l.chain[2] as usize).collect::<Vec<_>>();
    let lifted = asset.model_space(&pose)[ankles[0]].translation.y - rest_y;
    assert!(lifted > 0.05, "the generated ankle is {lifted} m up");
    let mut planter = Planter::new(legs);
    let dt = 1.0 / 60.0;
    let mut root = Trs::IDENTITY;
    let mut nodes = pose.clone();
    planter.step(&asset, &mut nodes, &root, &[1.0; 4], Some(0.0), dt);
    let m = asset.model_space(&nodes);
    for &a in &ankles {
        let up = m[a].translation.y - rest_y;
        assert!(up.abs() < 0.01, "a planted foot {up} m off the ground");
    }
    // The body carried on at 0.9 m/s over its planted feet.
    let mut last: Vec<Vec3> = ankles.iter().map(|&a| m[a].translation).collect();
    let mut steps = [0u32; 2];
    let mut highest = [0.0f32; 2];
    for frame in 1..90 {
        root.translation.x = 0.015 * frame as f32;
        let mut nodes = pose.clone();
        planter.step(&asset, &mut nodes, &root, &[1.0; 4], Some(0.0), dt);
        let m = asset.model_space(&nodes);
        let mut moving = 0;
        for (k, &a) in ankles.iter().enumerate() {
            let w = root.transform_point(m[a].translation);
            let across = Vec3::new(w.x - last[k].x, 0.0, w.z - last[k].z).length();
            // A step moves a foot centimetres a frame; a planted foot
            // at the edge of its reach eases by a fraction of one.
            if across > 2e-3 {
                moving += 1;
                steps[k] += 1;
                // Across the ground only ever lifted, past the first
                // and last few millimetres of the step.
                highest[k] = highest[k].max(w.y - rest_y);
            }
            last[k] = w;
        }
        assert!(moving < 2, "frame {frame}: both feet stepping at once");
    }
    for k in 0..2 {
        assert!(steps[k] > 0, "foot {k} never stepped after the body");
        assert!(
            highest[k] > 0.03,
            "foot {k} stepped {} m high: slid along the ground",
            highest[k]
        );
    }
    // Each foot kept up with the body's pose.
    let m = asset.model_space(&pose);
    for (k, &a) in ankles.iter().enumerate() {
        let generated = root.transform_point(m[a].translation);
        assert!(
            (last[k].x - generated.x).abs() < 0.25,
            "foot {k} at {} behind {}",
            last[k].x,
            generated.x
        );
    }
}

#[test]
fn planted_feet_outrank_a_reaction_except_where_it_steps() {
    use super::super::performance::Performance;
    let asset = standing().asset();
    let map = HumanoidMap::detect(&asset);
    let legs: Vec<Leg> = [(14usize, 0usize), (18, 2)]
        .iter()
        .map(|&(b, contact)| Leg {
            chain: [map.nodes[b], map.nodes[b + 1], map.nodes[b + 2]],
            contact,
            rest: 0.0,
        })
        .collect();
    assert!(legs.iter().all(|l| l.chain.iter().all(|&n| n != NONE)));
    let mut world = og_ecs::World::new(&og_ecs::WorldDesc {
        tick_rate: 60,
        seed: 1,
    })
    .unwrap();
    let sys = og_anim::install(&mut world).unwrap();
    // Knees bent 0.5 rad, within reach (straight legs are soft IK's).
    let mut rest = asset.rest_pose();
    for (hip, knee) in [(22, 23), (26, 27)] {
        rest[hip].rotation = glam::Quat::from_rotation_x(-0.25);
        rest[knee].rotation = glam::Quat::from_rotation_x(0.5);
    }
    let frames = |f: &dyn Fn(usize, &mut Vec<Trs>)| -> Vec<Vec<Trs>> {
        (0..31)
            .map(|k| {
                let mut p = rest.clone();
                f(k, &mut p);
                p
            })
            .collect()
    };
    // Held: the hips 0.15 m lower and pitched 20° forward.
    let crouch = frames(&|_, p| {
        p[0].translation.y -= 0.15;
        p[0].rotation = glam::Quat::from_rotation_x(20f32.to_radians());
    });
    // The left thigh swinging forward at 0.6 rad/s: a step.
    let step = frames(&|k, p| {
        p[22].rotation = glam::Quat::from_rotation_x(-0.25 - 0.02 * k as f32);
    });
    sys.add_clip(1, og_anim::motion::soma::clip("crouch", &asset, &crouch))
        .unwrap();
    sys.add_clip(2, og_anim::motion::soma::clip("step", &asset, &step))
        .unwrap();
    let root = Trs::IDENTITY;
    let ankles = [map.nodes[16], map.nodes[20]].map(|n| n as usize);
    let down = [1.0; 4];
    for (clip, turned, moved) in [
        (1, false, [false, false]),
        (2, false, [true, false]),
        (1, true, [true, true]),
    ] {
        let mut planter = Planter::new(legs.clone());
        let mut nodes = rest.clone();
        perform_and_plant(
            &mut planter,
            None,
            &sys,
            (&asset, &map),
            &root,
            &mut nodes,
            (down, None, 1.0 / 30.0),
        );
        let anchors = ankles.map(|a| asset.model_space(&nodes)[a].translation);
        let performance = Performance::holding(
            clip,
            1.0,
            0.5,
            if turned { [1.0, 0.0, 0.0] } else { [0.0; 3] },
        );
        let mut nodes = rest.clone();
        perform_and_plant(
            &mut planter,
            Some(&performance),
            &sys,
            (&asset, &map),
            &root,
            &mut nodes,
            (down, None, 1.0 / 30.0),
        );
        let m = asset.model_space(&nodes);
        for k in 0..2 {
            let off = (m[ankles[k]].translation - anchors[k]).length();
            assert_eq!(
                off > 0.05,
                moved[k],
                "clip {clip} ankle {k}: {off} m off its anchor"
            );
            if !moved[k] {
                assert!(off < 0.005, "clip {clip} ankle {k}: {off} m off its anchor");
            }
        }
        // The reaction itself was drawn: the hips lowered, or the thigh swung.
        if clip == 1 && !turned {
            assert!((m[0].translation.y - 0.85).abs() < 1e-3);
        }
    }
}

#[test]
fn a_display_frame_between_two_ticks_draws_a_pose_between_theirs() {
    use glam::Quat;
    let joint = |x: f32, yaw: f32| {
        Mat4::from_rotation_translation(Quat::from_rotation_y(yaw), Vec3::new(x, 0.0, 0.0))
    };
    let before = vec![joint(0.0, 0.0), joint(1.0, 0.2)];
    let now = vec![joint(1.0, 0.6), joint(3.0, -0.4)];
    let mut p = Presenter {
        previous: Some((4, IdMap::from_iter([(7, before.clone())]))),
        stepped: Some(5),
        ..Presenter::default()
    };
    p.palettes.insert(7, now.clone());
    // The tick itself (captures, the probe, a tick boundary).
    assert_eq!(p.drawn(7, None), Some(now.clone()));
    for a in [0.25f32, 0.5, 0.75] {
        let m = p.drawn(7, Some(a)).unwrap();
        for (k, (want0, want1)) in [((0.0, 0.0), (1.0, 0.6)), ((1.0, 0.2), (3.0, -0.4))]
            .into_iter()
            .enumerate()
        {
            let (s, r, t) = m[k].to_scale_rotation_translation();
            let (yaw, _, _) = r.to_euler(glam::EulerRot::YXZ);
            let x = want0.0 + (want1.0 - want0.0) * a;
            let y = want0.1 + (want1.1 - want0.1) * a;
            assert!(
                (t.x - x).abs() < 1e-5,
                "joint {k} at {a}: x {} not {x}",
                t.x
            );
            assert!(
                (yaw - y).abs() < 1e-5,
                "joint {k} at {a}: yaw {yaw} not {y}"
            );
            assert!(
                (s - Vec3::ONE).abs().max_element() < 1e-5,
                "joint {k}: scale {s}"
            );
            assert!(m[k] != before[k] && m[k] != now[k]);
        }
    }
    // A tick skipped since, or another rig: the tick's pose.
    p.previous.as_mut().unwrap().0 = 3;
    assert_eq!(p.drawn(7, Some(0.5)), Some(now.clone()));
    p.previous = Some((4, IdMap::from_iter([(7, before[..1].to_vec())])));
    assert_eq!(p.drawn(7, Some(0.5)), Some(now));
}

#[test]
fn warmup_is_not_playback_starvation() {
    let mut s = shown();
    for _ in 0..27 {
        s.count_wait();
    }
    assert_eq!((s.warmup, s.starved), (27, 0));
    let frame = Frame {
        pose: Pose {
            positions: [[0.0; 3]; og_anim::motion::contract::JOINTS],
            rotations: [[[0.0; 3]; 3]; og_anim::motion::contract::JOINTS],
        },
        contacts: [0.0; 4],
        tag: 0,
        index: 0,
    };
    s.prev = Some(frame);
    s.count_wait();
    assert_eq!((s.warmup, s.starved), (27, 1));
    s.next = Some(frame);
    s.phase = 0.99;
    s.count_wait();
    assert_eq!(s.starved, 1);
    s.phase = 1.0;
    s.count_wait();
    assert_eq!(s.starved, 2);
}

#[test]
fn dump_distinguishes_the_rig_map_from_its_naming_scheme() {
    use crate::{og_engine_create, og_game_game, og_host_begin_bundle};
    use og_anim::{RigAsset, SkeletonNode};

    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let created: Value = serde_json::from_str(&og_engine_create(game, "{}".into())).unwrap();
    let eng = crate::engine::engine(created["value"].as_u64().unwrap() as u32).unwrap();
    let asset = SkeletonAsset {
        nodes: ["Hips", "Spine02", "Spine01", "Spine", "neck", "Head"]
            .iter()
            .enumerate()
            .map(|(i, name)| SkeletonNode {
                name: (*name).into(),
                parent: if i == 0 { NONE } else { i as u32 - 1 },
                rest: Trs::IDENTITY,
                mesh: NONE,
            })
            .collect(),
        ..Default::default()
    };
    let rig = RigAsset::parse("type rig\nversion 1\n---\nscheme mixamo\nspine \"Spine02\"\nchest \"Spine01\"\nupperChest \"Spine\"\nneck \"neck\"\n").unwrap();
    let map = rig.map(&asset).unwrap();
    let mut s = shown();
    s.target = Some((Arc::new(asset), map));
    s.lean = Some(15.0_f32.to_radians());
    s.drawn_lean = Some(7.0_f32.to_radians());
    eng.render.borrow_mut().motion.agents.insert(1, s);
    let result = dump(&eng);
    let a = &result["agents"][0];
    assert_eq!(a["rig"], "mixamo");
    assert_eq!(a["rigMap"], 42.0);
    let bones = a["bones"].as_array().unwrap();
    for pair in [
        ["spine", "Spine02"],
        ["chest", "Spine01"],
        ["upperChest", "Spine"],
        ["neck", "neck"],
    ] {
        assert!(bones.contains(&json!(pair)), "{a}");
    }
    assert!((a["leanDeg"].as_f64().unwrap() - 7.0).abs() < 1e-4);
    assert!((a["sourceLeanDeg"].as_f64().unwrap() - 15.0).abs() < 1e-4);
}
