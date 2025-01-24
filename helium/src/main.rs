use helium::*;

fn add_model(manager: &mut HeliumManager) {
    manager.create_object(
        Model3d::from_obj("./assets/suzzane.obj".to_string()),
        Transform3d::default(),
    );
}

fn update_model(manager: &mut HeliumManager) {
    let models = manager.query::<Model3d>();
    let mut entity = 0;

    while !models.contains_key(&entity) {
        entity += 1;
    }

    drop(models);
    manager.update_transform(
        entity,
        Transform3d {
            position: Vector3 {
                x: 0.0,
                y: 1.0 * f32::sin(manager.time.elapsed().as_secs_f32()),
                z: 1.0 * f32::cos(manager.time.elapsed().as_secs_f32()),
            },
            rotation: Quaternion::one(),
        },
    );
}

fn main() {
    let _helium = Helium::new()
        .add_startup(add_model)
        .add_update(update_model)
        .run();
}
