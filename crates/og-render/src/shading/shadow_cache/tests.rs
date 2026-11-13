use super::*;
use glam::Mat4;

fn box_frustum(x: f32) -> Frustum {
    // An orthographic volume 2 m wide around (x, 0, 0), looking down -z.
    let proj = glam::camera::rh::proj::directx::orthographic(-1.0, 1.0, -1.0, 1.0, 0.0, 20.0);
    let view = Mat4::from_translation(Vec3::new(-x, 0.0, -10.0));
    Frustum::from_view_proj(proj * view)
}

/// Two batches of one instance each: one at x = 0, one at x = 10.
struct World {
    cache: ShadowCache,
    hashes: Vec<u64>,
    spheres: Vec<(Vec3, f32)>,
}

impl World {
    fn new() -> World {
        World {
            cache: ShadowCache::default(),
            hashes: vec![1, 2],
            spheres: vec![(Vec3::ZERO, 0.5), (Vec3::new(10.0, 0.0, 0.0), 0.5)],
        }
    }

    /// The next frame, the last one submitted.
    fn frame(&mut self) -> Split {
        self.cache.submitted();
        self.unsubmitted()
    }

    /// The next frame, the last one never submitted.
    fn unsubmitted(&mut self) -> Split {
        let batches = [0, 1].map(|first| BatchInfo {
            first,
            count: 1,
            cast: true,
            triangles: 12,
            always_dynamic: false,
        });
        self.cache
            .split(2, &batches, &[0, 1], &self.hashes, &self.spheres)
    }

    /// Moves the second instance by `dx`.
    fn nudge(&mut self, dx: f32) {
        self.hashes[1] += 1;
        self.spheres[1].0.x += dx;
    }
}

/// A still scene draws each layer once and then keeps it; a layer whose
/// view changes draws again.
#[test]
fn a_still_layer_is_drawn_once() {
    let mut w = World::new();
    let near = box_frustum(0.0);
    let draw = Plan {
        draw_static: true,
        ..Plan::default()
    };
    for frame in 0..4 {
        let split = w.frame();
        assert!(split.any_static && split.dirty.len() == if frame == 0 { 2 } else { 0 });
        let plan = w.cache.plan(0, 7, &near, false, 1);
        assert_eq!(plan, if frame == 0 { draw } else { Plan::default() });
    }
    w.frame();
    assert_eq!(w.cache.plan(0, 8, &near, false, 1), draw, "a new view");
    // A layer skipped for a frame missed its invalidations.
    w.frame();
    w.frame();
    assert_eq!(w.cache.plan(0, 8, &near, false, 1), draw);
    // A frame planned and never submitted drew nothing.
    w.frame();
    assert_eq!(w.cache.plan(0, 8, &near, false, 1), Plan::default());
    w.unsubmitted();
    assert_eq!(w.cache.plan(0, 8, &near, false, 1), draw);
}

/// Under an overlay a layer is kept only when its static casters cost
/// more to draw than the kept layer costs to put back.
#[test]
fn a_layer_is_kept_only_when_its_static_casters_cost_more() {
    let near = box_frustum(0.0);
    // Twelve triangles in a layer worth thirteen: one pass a frame.
    let mut w = World::new();
    for _ in 0..3 {
        w.frame();
        let plan = w
            .cache
            .plan(0, 1, &near, true, 13 * KEEP_TEXELS_PER_TRIANGLE);
        assert_eq!(plan, Plan::ALL);
    }
    // In a layer worth twelve: drawn and saved once the view holds,
    // then put back under the overlay.
    let mut w = World::new();
    let texels = 12 * KEEP_TEXELS_PER_TRIANGLE;
    let save = Plan {
        draw_static: true,
        save: true,
        overlay: true,
        ..Plan::default()
    };
    let back = Plan {
        restore: true,
        overlay: true,
        ..Plan::default()
    };
    for expected in [Plan::ALL, save, back, back] {
        w.frame();
        assert_eq!(w.cache.plan(0, 1, &near, true, texels), expected);
    }
}

/// A caster that moves turns dynamic at once and back to static after
/// [`QUIET_FRAMES`] still frames; only the layers its spheres reach
/// draw again, and while it moves the layer it is in, which holds no
/// other caster, draws in one pass.
#[test]
fn a_moving_caster_invalidates_only_the_layers_it_reaches() {
    let mut w = World::new();
    let near = box_frustum(0.0);
    let far = box_frustum(10.0);
    w.frame();
    w.cache.plan(0, 1, &near, false, 1);
    w.cache.plan(1, 2, &far, false, 1);
    // It moves: dynamic, its old sphere dirty.
    w.nudge(0.1);
    let split = w.frame();
    assert_eq!(split.dynamic, vec![false, true]);
    assert_eq!(split.dirty.len(), 1);
    assert!(!ShadowCache::moving_in(&split, &near));
    assert!(ShadowCache::moving_in(&split, &far));
    assert_eq!(w.cache.plan(0, 1, &near, false, 1), Plan::default());
    // The far layer has no static caster left to keep: one pass.
    assert_eq!(w.cache.plan(1, 2, &far, true, 1), Plan::ALL);
    // Still moving: the same.
    for _ in 0..3 {
        w.nudge(0.1);
        let split = w.frame();
        assert!(split.dirty.is_empty());
        assert_eq!(w.cache.plan(0, 1, &near, false, 1), Plan::default());
        assert_eq!(w.cache.plan(1, 2, &far, true, 1), Plan::ALL);
    }
    // It stops: dynamic for QUIET_FRAMES - 1 more frames, then static
    // and dirty where it stands.
    for f in 1..=QUIET_FRAMES {
        let split = w.frame();
        let still = f == QUIET_FRAMES;
        assert_eq!(split.dynamic[1], !still, "frame {f}");
        assert_eq!(split.dirty.len(), still as usize);
        assert_eq!(w.cache.plan(0, 1, &near, false, 1), Plan::default());
        let plan = w.cache.plan(1, 2, &far, !still, 1);
        if still {
            assert_eq!(
                plan,
                Plan {
                    draw_static: true,
                    ..Plan::default()
                }
            );
        } else {
            assert_eq!(plan, Plan::ALL);
        }
    }
    // Without the mover the far layer's cache comes back once.
    let split = w.frame();
    assert!(split.dirty.is_empty());
    assert_eq!(w.cache.plan(1, 2, &far, false, 1), Plan::default());
}

/// A view that moves every frame never pays for the cache: one pass of
/// every caster, nothing saved; one that takes a step after holding
/// still is kept from the step on; a layer without static casters is
/// not cached.
#[test]
fn a_moving_view_draws_in_one_pass() {
    let mut w = World::new();
    let near = box_frustum(0.0);
    for key in 0..4 {
        w.frame();
        assert_eq!(w.cache.plan(0, key, &near, true, 1), Plan::ALL);
    }
    w.frame();
    assert!(w.cache.plan(0, 3, &near, true, 1).save, "still: kept");
    w.frame();
    let held = w.cache.plan(0, 3, &near, true, 1);
    assert!(held.cached() && held.restore && !held.save);
    w.frame();
    let step = w.cache.plan(0, 4, &near, true, 1);
    assert!(step.draw_static && step.save && !step.single(), "{step:?}");
    w.frame();
    assert!(w.cache.plan(0, 4, &near, true, 1).cached());
    w.frame();
    w.cache.plan(0, 5, &near, true, 1);
    w.frame();
    assert_eq!(w.cache.plan(0, 6, &near, true, 1), Plan::ALL, "moving");
    w.frame();
    assert!(w.cache.plan(0, 6, &near, true, 1).save, "still: kept");
    w.frame();
    let plan = w.cache.plan(0, 3, &near, true, u64::MAX);
    assert_eq!(plan, Plan::ALL, "nothing static");
    assert!(plan.single() && !plan.cached());
}

/// Foliage is dynamic only in a wind, a material shader always; a plain
/// batch never is (its instances' transforms say when it moves).
#[test]
fn what_a_vertex_shader_moves_is_dynamic() {
    assert!(deforms(true, true, false));
    assert!(
        !deforms(true, false, false),
        "foliage without wind is still"
    );
    assert!(!deforms(false, true, false), "wind moves only foliage");
    assert!(deforms(false, false, true));
    assert!(!deforms(false, false, false));
}

/// A layer's lists ask the table what a binary search of the dynamic
/// batches' sorted firsts answered, for every batch and for firsts that
/// are no batch's (an empty batch at a dynamic one's first, past the
/// table's end).
#[test]
fn dynamic_firsts_answer_as_the_sorted_firsts_did() {
    // (first, count, dynamic): runs of instances, empty batches among
    // them, the last batch dynamic.
    let batches: Vec<(u32, u32, bool)> = vec![
        (0, 3, false),
        (3, 2, true),
        (5, 0, true),
        (5, 4, false),
        (9, 1, true),
        (10, 0, false),
        (10, 6, false),
        (16, 2, true),
        (18, 5, false),
        (23, 1, true),
    ];
    let mut firsts: Vec<u32> = batches
        .iter()
        .filter(|(_, count, dynamic)| *dynamic && *count > 0)
        .map(|(first, _, _)| *first)
        .collect();
    firsts.sort_unstable();
    let before = |first: u32, count: u32| count > 0 && firsts.binary_search(&first).is_ok();
    let table = DynamicFirsts::new(batches.iter().copied());
    for &(first, count, _) in &batches {
        assert_eq!(
            table.contains(first, count),
            before(first, count),
            "batch at {first}, {count}"
        );
    }
    for first in 0..40 {
        for count in [0, 1, 7] {
            assert_eq!(
                table.contains(first, count),
                before(first, count),
                "{first}, {count}"
            );
        }
    }
    assert!(table.contains(23, 1), "the last dynamic batch");
    assert!(
        !table.contains(3, 0),
        "an empty batch at a dynamic one's first"
    );
    assert!(
        !DynamicFirsts::new(std::iter::empty()).contains(0, 1),
        "no dynamic batch"
    );
}

/// An instance gone or not drawn takes its cached shadow with it; a
/// new one starts still and only dirties where it appears.
#[test]
fn instances_that_come_and_go_dirty_their_spheres() {
    let mut w = World::new();
    w.frame();
    let one = [BatchInfo {
        first: 0,
        count: 1,
        cast: true,
        triangles: 12,
        always_dynamic: false,
    }];
    let split = w.cache.split(1, &one, &[0], &[1], &w.spheres[..1]);
    assert_eq!(split.dirty.len(), 1);
    assert_eq!(split.dirty[0].0.x, 10.0);
    let split = w.cache.split(2, &one, &[0], &[1], &w.spheres[..1]);
    assert!(split.dirty.is_empty(), "the second is not drawn yet");
    let split = w.frame();
    assert_eq!(split.dynamic, vec![false, false], "new instances are still");
    assert_eq!(split.dirty.len(), 1);
    // A batch that is always dynamic is never cached.
    let swaying = [BatchInfo {
        always_dynamic: true,
        ..one[0]
    }];
    let split = w.cache.split(1, &swaying, &[0], &[1], &w.spheres[..1]);
    assert!(!split.any_static && split.dynamic == vec![true]);
    assert_eq!(split.dirty.len(), 2, "both leave the cache");
}

/// Kept across frames, the instance hashes are the ones hashing every
/// instance gives, after each change they read: a transform, a look, a
/// sway the GPU transform does not show, an own material the sort does
/// not see (an alias), a mesh's and a material's revision, a new order,
/// and an update the hashes missed.
#[test]
fn kept_instance_hashes_are_a_full_passes() {
    use crate::instances::SortedInstances;
    let mut scene = crate::RenderScene::new();
    let meshes = [0, 1].map(|_| scene.add_mesh(crate::MeshData::default()));
    let materials = [0, 1, 2].map(|_| scene.add_material(crate::Material::default()));
    for i in 0..60u32 {
        let m = Mat4::from_translation(Vec3::new(i as f32, 0.0, 0.0));
        scene.add_instance(meshes[(i % 2) as usize], materials[(i % 2) as usize], m);
    }
    assert!(scene.alias_material(materials[2], materials[1]));
    let info = |_: usize| (Some((Vec3::ZERO, Vec3::splat(0.5))), false);
    let mut revisions = (vec![1u64; 2], vec![1u64; 3]);
    let mut sorted = SortedInstances::default();
    let mut kept = InstanceHashes::default();
    // One frame: the kept hashes, a full pass's, and how many changed.
    let mut frame = |scene: &crate::RenderScene, revisions: &(Vec<u64>, Vec<u64>), hashed: bool| {
        let s = sorted
            .update(scene, 2, info, 3, |_, _| None, |_| false, |_| false)
            .unwrap();
        let hash = |k: usize| {
            let inst = &scene.objects.instances[s.submission[k] as usize];
            let (m, t) = (inst.mesh.index(), inst.material.index());
            let h = instance_hash(&s.transforms[k], inst, [revisions.0[m], revisions.1[t]]);
            (h, (m as u32, t as u32))
        };
        let n = s.transforms.len();
        let full: Vec<u64> = (0..n).map(|k| hash(k).0).collect();
        let got = if hashed {
            let (m, t) = (revisions.0.iter().copied(), revisions.1.iter().copied());
            kept.update(n, (s.serial, s.changed.as_deref()), m, t, hash)
                .to_vec()
        } else {
            full.clone()
        };
        let changed = s.changed.as_ref().map(Vec::len);
        sorted.restore(s.transforms, s.spheres);
        (got == full, changed)
    };
    assert_eq!(frame(&scene, &revisions, true), (true, None), "the first");
    assert_eq!(
        frame(&scene, &revisions, true),
        (true, Some(0)),
        "unchanged"
    );
    scene.objects.instances[3].transform.w_axis.y = 2.0;
    assert_eq!(frame(&scene, &revisions, true), (true, Some(1)), "moved");
    scene.objects.instances[4].fx.flash = 0.5;
    assert_eq!(frame(&scene, &revisions, true), (true, Some(1)), "flashed");
    // Its GPU transform carries max(sway, 0): both 0.
    scene.objects.instances[5].sway = -1.0;
    assert_eq!(frame(&scene, &revisions, true), (true, Some(1)), "sway");
    // Drawn in the same batches: the sort keeps its place.
    scene.objects.instances[7].material = materials[2];
    assert_eq!(
        frame(&scene, &revisions, true),
        (true, Some(1)),
        "aliased material"
    );
    revisions.0[1] += 1;
    assert_eq!(
        frame(&scene, &revisions, true),
        (true, Some(0)),
        "a mesh's revision"
    );
    revisions.1[2] += 1;
    assert_eq!(
        frame(&scene, &revisions, true),
        (true, Some(0)),
        "a material's revision"
    );
    scene.objects.instances.swap(0, 1);
    assert_eq!(
        frame(&scene, &revisions, true),
        (true, None),
        "another order"
    );
    // A frame that sorted and failed before hashing: the next hashes all.
    scene.objects.instances[9].transform.w_axis.z = 1.0;
    assert_eq!(frame(&scene, &revisions, false), (true, Some(1)));
    assert_eq!(
        frame(&scene, &revisions, true),
        (true, Some(0)),
        "after a missed update"
    );
}
