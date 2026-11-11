pub fn test_scene_loader() -> propellant::scene::SceneLoader {
    propellant::scene::SceneLoader::new("Test Scene", load_test_scene)
}

fn load_test_scene(scene: &mut propellant::scene::Scene, assets: &propellant::assets::AssetManager) {
    /* Add a camera holding entity */
    use propellant::components;

    let camera = components::Camera::main(1.0);
    let cam_transform = components::Transform::IDENTITY
        .at(glam::vec3(0., 2., 8.))
        .look_at(glam::Vec3::ZERO);
    scene.spawn((camera, cam_transform));

    /* Add a single segment entity */
    match assets.get_handle::<crate::assets::SegmentAsset>("tube_small") {
        Some(handle) => {
            let segment_component = crate::components::SegmentRenderer { handle };
            let segment_transform = components::Transform::IDENTITY;
            scene.spawn((segment_component, segment_transform));
        }
        None => log::warn!("Failed to load objects for test scene: missing asset \"tube_small\""),
    }

    /* Fixme: little spinner as I implement raymarching */
    scene.load_system(Spinner { speed: 0.5 });
}

/// Fixme: delete, for the test
struct Spinner {
    speed: f32,
}

impl propellant::scene::System for Spinner {
    fn name(&self) -> &'static str {
        "Spinner"
    }

    fn frequency(&self) -> propellant::scene::UpdateFrequency {
        propellant::scene::UpdateFrequency::PerFrame
    }

    fn update(
        &mut self,
        _: &propellant::assets::AssetManager,
        world: &mut propellant::scene::World,
        delta_time: std::time::Duration,
    ) {
        use crate::components::SegmentRenderer;
        use propellant::components::Transform;

        for (_, (transform, _)) in world.query_mut::<(&mut Transform, &SegmentRenderer)>() {
            transform.rotate_around(glam::Vec3::Y, self.speed * delta_time.as_secs_f32());
        }
    }
}
