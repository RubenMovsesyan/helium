use helium::*;

fn add_model(state: &mut HeliumState) {
    state.create_object("./assets/suzzane.obj", {
        let mut instances = Vec::new();
        for i in 0..1 {
            instances.push(Instance {
                position: Vector3 {
                    x: 1.0 * i as f32,
                    y: 0.0,
                    z: 0.0,
                },
                rotation: Quaternion::one(),
            });
        }
        instances
    });
}

fn update_model(state: &mut HeliumState, time: Instant) {
    state.update_instances(0, {
        let mut instances = Vec::new();

        for i in -5..=5 {
            instances.push(Instance {
                position: Vector3 {
                    x: 5.0 * i as f32,
                    y: 1.0 * f32::sin((Instant::now() - time).as_secs_f32() + i as f32),
                    z: 1.0 * f32::cos((Instant::now() - time).as_secs_f32() - i as f32),
                },
                rotation: Quaternion::one(),
            })
        }

        instances
    });
}

fn main() {
    let _helium = Helium::new()
        .add_startup(add_model)
        .add_update(update_model)
        .run();
}
