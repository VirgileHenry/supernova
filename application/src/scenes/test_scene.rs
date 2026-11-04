pub fn test_scene_loader() -> propellant::scene::SceneLoader {
    propellant::scene::SceneLoader::new("Test Scene", load_test_scene)
}

fn load_test_scene(scene: &mut propellant::scene::Scene) {
    let mut scene = Self::empty("Test Scene".to_string());

    /* Add a camera holding entity */
    use crate::components;
    let camera = components::Camera::main(1.0);
    let cam_transform = components::Transform::IDENTITY
        .at(glam::vec3(0., 2., 8.))
        .look_at(glam::Vec3::ZERO);
    scene.world.spawn((camera, cam_transform));

    /* Add a single segment entity */
    match assets.get_segment_handle("tube_small") {
        Some(segment) => {
            let segment_component = components::SegmentRenderer::new(segment);
            let segment_transform = components::Transform::IDENTITY;
            scene.world.spawn((segment_component, segment_transform));
        }
        None => log::warn!("Failed to load objects for test scene: missing asset \"tube_small\""),
    }

    /* Fixme: little spinner as I implement raymarching */
    scene.load_system(Spinner { speed: 0.5 });

    // Insert all objects for menu world

    scene
}

/// Fixme: delete, for the test
struct Spinner {
    speed: f32,
}

impl System for Spinner {
    fn name(&self) -> &'static str {
        "Spinner"
    }

    fn frequency(&self) -> UpdateFrequency {
        UpdateFrequency::PerFrame
    }

    fn handle_event(&mut self, _: &mut hecs::World, _: SystemEvent) {}

    fn update(&mut self, _: &crate::assets::AssetManager, world: &mut hecs::World, delta_time: std::time::Duration) {
        use crate::components;

        for (_, (transform, _)) in world.query_mut::<(&mut components::Transform, &components::SegmentRenderer)>() {
            transform.rotate_around(glam::Vec3::Y, self.speed * delta_time.as_secs_f32());
        }
    }
}
