//! Equal-depth software triangles choose a stable cluster, not whichever
//! culling invocation happened to append last.
use glam::{Mat4, Vec3};
use og_render::meshlet::{Cluster, ClusterLevel, MeshClusters};
use og_render::{
    Camera, Culling, InstanceFx, Material, MeshData, RenderDesc, RenderScene, RenderTarget,
    Renderer, Vertex,
};

#[test]
#[ignore = "requires GPU"]
fn equal_depth_meshlets_choose_the_same_triangle_across_frames() {
    let mut mesh = MeshData::default();
    let mut clusters = Vec::new();
    // More than two rounds of the 64-thread cull: cluster 128 is handled
    // by thread zero, not the last thread to append in a wave.
    for i in 0..129 {
        for position in [[-0.1, -0.1, 0.0], [0.1, -0.1, 0.0], [0.0, 0.1, 0.0]] {
            mesh.indices.push(mesh.vertices.len() as u32);
            mesh.vertices.push(Vertex {
                position,
                normal: [0.0, 0.0, 1.0],
                ..Default::default()
            });
        }
        clusters.push(Cluster {
            first: i * 3,
            triangles: 1,
            radius: 0.15,
            cone_cutoff: 1.0,
            ..Default::default()
        });
    }
    let clustered = MeshClusters {
        levels: vec![ClusterLevel {
            count: mesh.indices.len() as u32,
            clusters: 0..129,
            ..Default::default()
        }],
        indices: mesh.indices.clone(),
        clusters,
    };
    let mut scene = RenderScene::new();
    let mesh = scene.add_mesh(mesh);
    assert!(scene.set_mesh_clusters(mesh, Some(clustered)));
    let material = scene.add_material(Material::default());
    for i in 0..65 {
        scene.add_instance(mesh, material, Mat4::IDENTITY);
        scene.instances.last_mut().unwrap().fx = InstanceFx {
            flash: 1.0,
            flash_color: if i == 64 { Vec3::Y } else { Vec3::X },
            ..Default::default()
        };
    }
    scene.camera = Camera {
        position: Vec3::new(0.0, 0.0, 3.0),
        target: Vec3::ZERO,
        ..Default::default()
    };
    scene.light.cast_shadows = false;
    // The meshlet path follows GPU culling, which `Culling::Auto` keeps for
    // frames of GPU_CULLING_MIN_INSTANCES instances; these are 65.
    let mut renderer = Renderer::new_blocking(&RenderDesc {
        width: 128,
        height: 128,
        culling: Culling::Gpu,
        ..Default::default()
    })
    .expect("renderer");
    let mut previous = None;
    for _ in 0..3 {
        renderer
            .render(&scene, &RenderTarget::Offscreen)
            .expect("render");
        let image = renderer.read_offscreen().expect("capture");
        let center = &image.pixels[(64 * 128 + 64) * 4..][..4];
        assert!(
            center[1] > center[0] + 20,
            "last instance must win: {center:?}"
        );
        if let Some(before) = previous {
            assert_eq!(image.pixels, before);
        }
        previous = Some(image.pixels);
    }
}
