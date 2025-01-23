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

fn main() {
    let _helium = Helium::new().add_startup(add_model).run();
}
