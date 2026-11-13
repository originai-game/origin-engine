use super::*;

/// A perspective camera at (0, 0, 10) looking down -z, 60° tall, on a
/// 1000×500 surface.
fn persp() -> View {
    let proj = glam::camera::rh::proj::directx::perspective(60f32.to_radians(), 2.0, 0.1, 100.0);
    let view = glam::camera::rh::view::look_at_mat4(Vec3::new(0.0, 0.0, 10.0), Vec3::ZERO, Vec3::Y);
    View::new(
        proj * view,
        Vec2::new(1000.0, 500.0),
        Vec3::new(0.0, 0.0, 10.0),
        Vec3::NEG_Z,
        false,
    )
}

/// The same camera looking from (6, 5, 8) at the origin: no axis end on.
fn angled() -> View {
    let eye = Vec3::new(6.0, 5.0, 8.0);
    let proj = glam::camera::rh::proj::directx::perspective(60f32.to_radians(), 2.0, 0.1, 100.0);
    let view = glam::camera::rh::view::look_at_mat4(eye, Vec3::ZERO, Vec3::Y);
    View::new(
        proj * view,
        Vec2::new(1000.0, 500.0),
        eye,
        -eye.normalize(),
        false,
    )
}

fn at(view: &View, p: Vec3) -> Vec2 {
    view.project(p).unwrap()
}

const ID: Pose = (Vec3::ZERO, Quat::IDENTITY, Vec3::ONE);

#[test]
fn the_handles_keep_their_size_on_screen_at_any_distance() {
    let v = persp();
    for z in [5.0, 0.0, -20.0] {
        let g = Gizmo::new(
            &v,
            Mode::Translate,
            Space::World,
            Vec3::new(1.0, 0.5, z),
            Quat::IDENTITY,
        );
        let o = at(&v, g.origin);
        let x = at(&v, g.origin + Vec3::X * g.length);
        assert!(
            (o.distance(x) - PIXELS).abs() < PIXELS * 0.03,
            "at z {z}: {}",
            o.distance(x)
        );
    }
    // Further away is longer in the world.
    let near = Gizmo::new(
        &v,
        Mode::Translate,
        Space::World,
        Vec3::new(0.0, 0.0, 5.0),
        Quat::IDENTITY,
    );
    let far = Gizmo::new(
        &v,
        Mode::Translate,
        Space::World,
        Vec3::new(0.0, 0.0, -20.0),
        Quat::IDENTITY,
    );
    assert!(far.length > near.length * 4.0);
}

#[test]
fn local_space_turns_translate_and_rotate_but_scale_is_always_local() {
    let v = angled();
    let q = Quat::from_rotation_y(90f32.to_radians());
    let world = Gizmo::new(&v, Mode::Translate, Space::World, Vec3::ZERO, q);
    assert!(world.axes[0].abs_diff_eq(Vec3::X, 1e-5));
    let local = Gizmo::new(&v, Mode::Translate, Space::Local, Vec3::ZERO, q);
    assert!(
        local.axes[0].abs_diff_eq(Vec3::NEG_Z, 1e-5),
        "{:?}",
        local.axes[0]
    );
    let scale = Gizmo::new(&v, Mode::Scale, Space::World, Vec3::ZERO, q);
    assert!(scale.axes[0].abs_diff_eq(Vec3::NEG_Z, 1e-5));
}

#[test]
fn every_handles_grab_point_hits_that_handle() {
    let v = angled();
    for mode in [Mode::Translate, Mode::Rotate, Mode::Scale] {
        for space in [Space::World, Space::Local] {
            let g = Gizmo::new(
                &v,
                mode,
                space,
                Vec3::new(0.5, 0.0, -1.0),
                Quat::from_rotation_z(0.4),
            );
            for h in g.handles() {
                let p = at(&v, g.grab_point(&v, h));
                assert_eq!(g.hit(&v, p), Some(h), "{mode:?} {space:?} {h:?} at {p}");
            }
            // Far from it, nothing.
            let o = at(&v, g.origin);
            assert_eq!(g.hit(&v, o + Vec2::new(300.0, 160.0)), None);
        }
    }
}

#[test]
fn an_axis_seen_end_on_is_not_grabbed() {
    // The camera looks down z: the z arrow is a point on screen.
    let v = persp();
    let g = Gizmo::new(
        &v,
        Mode::Translate,
        Space::World,
        Vec3::ZERO,
        Quat::IDENTITY,
    );
    let tip = at(&v, g.origin + Vec3::Z * g.length * 0.8);
    assert_ne!(g.hit(&v, tip), Some(Handle::Axis(2)));
    assert!(Drag::begin(&v, g, Handle::Axis(2), tip, ID).is_none());
}

#[test]
fn an_arrow_follows_the_pointer_along_its_axis_only() {
    let v = angled();
    let g = Gizmo::new(
        &v,
        Mode::Translate,
        Space::World,
        Vec3::ZERO,
        Quat::IDENTITY,
    );
    let grab = at(&v, g.grab_point(&v, Handle::Axis(0)));
    let d = Drag::begin(&v, g, Handle::Axis(0), grab, ID).unwrap();
    // The pointer where the world point x = 2 would be, then pulled
    // off the axis on screen: the pose moves along x alone.
    let target = at(&v, Vec3::new(2.0, 0.0, 0.0) + Vec3::X * g.length * 0.8);
    let (p, q, s) = d.pose(&v, target, false);
    assert!(p.abs_diff_eq(Vec3::new(2.0, 0.0, 0.0), 1e-3), "{p}");
    assert_eq!((q, s), (Quat::IDENTITY, Vec3::ONE));
    let (p, _, _) = d.pose(&v, target + Vec2::new(0.0, 60.0), false);
    assert!(p.y == 0.0 && p.z == 0.0 && p.x != 0.0, "{p}");
    // Snapped: whole steps of SNAP_MOVE.
    let near = at(&v, Vec3::new(1.13, 0.0, 0.0) + Vec3::X * g.length * 0.8);
    let (p, _, _) = d.pose(&v, near, true);
    assert!((p.x - 1.0).abs() < 1e-5, "{p}");
    let (p, _, _) = d.pose(&v, near, false);
    assert!((p.x - 1.13).abs() < 1e-3, "{p}");
}

#[test]
fn a_plane_square_moves_in_its_plane_and_the_centre_in_the_views() {
    let v = angled();
    let g = Gizmo::new(
        &v,
        Mode::Translate,
        Space::World,
        Vec3::ZERO,
        Quat::IDENTITY,
    );
    // The xz plane is across y (Plane(1)).
    let grab = at(&v, g.grab_point(&v, Handle::Plane(1)));
    let d = Drag::begin(&v, g, Handle::Plane(1), grab, ID).unwrap();
    let start = g.grab_point(&v, Handle::Plane(1));
    let to = at(&v, start + Vec3::new(1.5, 0.0, -0.8));
    let (p, _, _) = d.pose(&v, to, false);
    assert!(p.abs_diff_eq(Vec3::new(1.5, 0.0, -0.8), 1e-3), "{p}");
    let (p, _, _) = d.pose(&v, to, true);
    assert!(p.abs_diff_eq(Vec3::new(1.5, 0.0, -1.0), 1e-5), "{p}");
    // The centre: what it moves to stays in the plane facing the eye.
    let grab = at(&v, Vec3::ZERO);
    let d = Drag::begin(&v, g, Handle::View, grab, ID).unwrap();
    let (p, _, _) = d.pose(&v, grab + Vec2::new(50.0, -20.0), false);
    assert!(
        p.dot(v.toward_eye(Vec3::ZERO)).abs() < 1e-3 && p.length() > 0.1,
        "{p}"
    );
}

#[test]
fn a_ring_face_on_turns_by_the_pointers_angle_and_snaps_to_fifteen() {
    let v = persp();
    let g = Gizmo::new(&v, Mode::Rotate, Space::World, Vec3::ZERO, Quat::IDENTITY);
    // The z ring faces the camera: from its right edge to its top is a
    // quarter turn anticlockwise to the viewer, +90° about z.
    let o = at(&v, Vec3::ZERO);
    let r = PIXELS;
    let d = Drag::begin(&v, g, Handle::Ring(2), o + Vec2::new(r, 0.0), ID).unwrap();
    let (_, q, _) = d.pose(&v, o + Vec2::new(0.0, -r), false);
    let (axis, angle) = q.to_axis_angle();
    assert!(
        axis.abs_diff_eq(Vec3::Z, 1e-4) && (angle.to_degrees() - 90.0).abs() < 0.1,
        "{axis} {}",
        angle.to_degrees()
    );
    // 20° on screen snaps to 15°.
    let a = 20f32.to_radians();
    let (_, q, _) = d.pose(&v, o + Vec2::new(a.cos(), -a.sin()) * r, true);
    assert!((q.to_axis_angle().1.to_degrees() - 15.0).abs() < 0.01);
}

#[test]
fn a_ring_edge_on_turns_by_travel_along_its_tangent() {
    let v = persp();
    let g = Gizmo::new(&v, Mode::Rotate, Space::World, Vec3::ZERO, Quat::IDENTITY);
    // The y ring is seen edge on (a flat line across the screen).
    let grab = at(&v, g.grab_point(&v, Handle::Ring(1)));
    let d = Drag::begin(&v, g, Handle::Ring(1), grab, ID).unwrap();
    let Anchor::Tangent { dir, radius } = d.anchor else {
        panic!("{:?}", d.anchor);
    };
    let (_, q, _) = d.pose(&v, grab + dir * radius * 0.5, false);
    let (axis, angle) = q.to_axis_angle();
    assert!(axis.abs_diff_eq(Vec3::Y, 1e-4), "{axis}");
    assert!((angle - 0.5).abs() < 1e-3, "{angle}");
    // The other way turns back.
    let (_, back, _) = d.pose(&v, grab - dir * radius * 0.5, false);
    assert!(
        (back * q).abs_diff_eq(Quat::IDENTITY, 1e-4)
            || (back * q).abs_diff_eq(-Quat::IDENTITY, 1e-4)
    );
}

#[test]
fn a_turn_keeps_what_the_entity_was_turned_by_already() {
    let v = persp();
    let q0 = Quat::from_rotation_x(0.3);
    let g = Gizmo::new(&v, Mode::Rotate, Space::World, Vec3::ZERO, q0);
    let o = at(&v, Vec3::ZERO);
    let d = Drag::begin(
        &v,
        g,
        Handle::Ring(2),
        o + Vec2::new(PIXELS, 0.0),
        (Vec3::ZERO, q0, Vec3::ONE),
    )
    .unwrap();
    let (_, q, _) = d.pose(&v, o + Vec2::new(0.0, -PIXELS), false);
    let want = Quat::from_rotation_z(90f32.to_radians()) * q0;
    assert!(q.abs_diff_eq(want, 1e-3), "{q} {want}");
}

#[test]
fn scale_by_the_reach_along_an_axis_and_uniformly_by_distance() {
    let v = angled();
    let s0 = Vec3::new(2.0, 1.0, 1.0);
    let g = Gizmo::new(&v, Mode::Scale, Space::World, Vec3::ZERO, Quat::IDENTITY);
    let grab_world = g.grab_point(&v, Handle::Axis(0));
    let d = Drag::begin(
        &v,
        g,
        Handle::Axis(0),
        at(&v, grab_world),
        (Vec3::ZERO, Quat::IDENTITY, s0),
    )
    .unwrap();
    // Pulled to 1.5 times the reach it was grabbed at: x scale × 1.5.
    let (_, _, s) = d.pose(&v, at(&v, grab_world * 1.5), false);
    assert!(s.abs_diff_eq(Vec3::new(3.0, 1.0, 1.0), 1e-3), "{s}");
    let (_, _, s) = d.pose(&v, at(&v, grab_world * 1.53), true);
    assert!(s.abs_diff_eq(Vec3::new(3.0, 1.0, 1.0), 1e-4), "{s}");
    // Through the origin it stops at 1 %.
    let (_, _, s) = d.pose(&v, at(&v, -grab_world), false);
    assert!((s.x - 0.02).abs() < 1e-5, "{s}");
    // The centre: twice as far on screen doubles every axis.
    let o = at(&v, Vec3::ZERO);
    let d = Drag::begin(
        &v,
        g,
        Handle::View,
        o + Vec2::new(10.0, 0.0),
        (Vec3::ZERO, Quat::IDENTITY, s0),
    )
    .unwrap();
    let (_, _, s) = d.pose(&v, o + Vec2::new(0.0, 20.0), false);
    assert!(s.abs_diff_eq(s0 * 2.0, 1e-4), "{s}");
}

#[test]
fn a_world_pose_goes_back_under_its_parent() {
    let parent: Pose = (
        Vec3::new(1.0, 2.0, 0.0),
        Quat::from_rotation_y(0.7),
        Vec3::splat(2.0),
    );
    let local: Pose = (
        Vec3::new(0.5, -1.0, 3.0),
        Quat::from_rotation_x(0.2),
        Vec3::new(1.0, 2.0, 3.0),
    );
    let world = og_core::math::world_pose(
        1u8,
        local,
        |k| (k == 1).then_some(0u8),
        |k| (k == 0).then_some(parent),
    );
    let back = to_local(Some(parent), world);
    assert!(back.0.abs_diff_eq(local.0, 1e-5), "{} {}", back.0, local.0);
    assert!(back.1.abs_diff_eq(local.1, 1e-5));
    assert_eq!(back.2, world.2);
    assert_eq!(to_local(None, world), world);
}

#[test]
fn an_orthographic_view_casts_parallel_rays() {
    let half_h = 5.0;
    let proj = glam::camera::rh::proj::directx::orthographic(
        -half_h * 2.0,
        half_h * 2.0,
        -half_h,
        half_h,
        0.1,
        100.0,
    );
    let view = glam::camera::rh::view::look_at_mat4(Vec3::new(0.0, 0.0, 10.0), Vec3::ZERO, Vec3::Y);
    let v = View::new(
        proj * view,
        Vec2::new(1000.0, 500.0),
        Vec3::new(0.0, 0.0, 10.0),
        Vec3::NEG_Z,
        true,
    );
    let (_, a) = v.ray(Vec2::new(10.0, 10.0));
    let (_, b) = v.ray(Vec2::new(900.0, 400.0));
    assert!(a.abs_diff_eq(b, 1e-5) && a.abs_diff_eq(Vec3::NEG_Z, 1e-5));
    // 500 px is 10 units: a handle is 1.92 units long everywhere.
    let g = Gizmo::new(
        &v,
        Mode::Translate,
        Space::World,
        Vec3::new(3.0, 1.0, 0.0),
        Quat::IDENTITY,
    );
    assert!((g.length - PIXELS / 50.0).abs() < 1e-3, "{}", g.length);
    let grab = at(&v, g.grab_point(&v, Handle::Axis(1)));
    let d = Drag::begin(
        &v,
        g,
        Handle::Axis(1),
        grab,
        (g.origin, Quat::IDENTITY, Vec3::ONE),
    )
    .unwrap();
    let (p, _, _) = d.pose(&v, grab + Vec2::new(0.0, -100.0), false);
    assert!(p.abs_diff_eq(Vec3::new(3.0, 3.0, 0.0), 1e-3), "{p}");
}
