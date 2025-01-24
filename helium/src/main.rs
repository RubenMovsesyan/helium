use helium::*;
use log::info;

fn add_model(manager: &mut HeliumManager) {
    manager.create_object(
        Model3d::from_obj("./assets/suzzane.obj".to_string()),
        Transform3d::default(),
    );
}

fn add_camera(manager: &mut HeliumManager) {
    let config = manager.get_render_config();
    manager.create_camera(Camera3d {
        eye: (5.0, 5.0, 5.0).into(),
        target: (-5.0, -5.0, -5.0).into(),
        up: Vector3::unit_y(),
        aspect: config.width as f32 / config.height as f32,
        fovy: 45.0,
        znear: 0.1,
        zfar: 100.0,
    });
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

// FIX this because the input is tied to the key repeat
fn process_inputs(manager: &mut HeliumManager, event: &WindowEvent) {
    match event {
        WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    state,
                    physical_key: PhysicalKey::Code(keycode),
                    ..
                },
            ..
        } => {
            let camera_id = manager.camera_id.unwrap();
            let time = manager.delta_time;
            let mut cameras = manager.query_mut::<Camera3d>();

            let is_pressed = *state == ElementState::Pressed;

            let mut forward = false;
            let mut backward = false;
            let mut left = false;
            let mut right = false;
            let mut tl = false;
            let mut tr = false;

            match keycode {
                KeyCode::KeyW => {
                    forward = is_pressed;
                }
                KeyCode::KeyS => {
                    backward = is_pressed;
                }
                KeyCode::KeyA => {
                    left = is_pressed;
                }
                KeyCode::KeyD => {
                    right = is_pressed;
                }
                _ => {}
            }

            cameras
                .get_mut(&camera_id)
                .unwrap()
                .update_camera(forward, backward, left, right, tl, tr, time);

            drop(cameras);

            manager.move_camera_to_render();
        }
        _ => {}
    }
}

fn main() {
    let _helium = Helium::new()
        .add_startup(add_model)
        .add_startup(add_camera)
        .add_update(update_model)
        .add_input(process_inputs)
        .run();
}
