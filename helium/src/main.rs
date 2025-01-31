use cgmath::{InnerSpace, Rad, Rotation3};
use helium::*;
#[allow(unused_imports)]
use log::*;

fn add_model(manager: &mut HeliumManager) {
    let suzzane = manager.create_object(
        Model3d::from_obj("./assets/suzzane.obj".to_string()),
        Transform3d::new(
            Vector3 {
                x: 0.0,
                y: 15.0,
                z: 0.0,
            },
            Quaternion::one(),
        ),
    );

    manager.add_component(suzzane, Label("Suzzane".to_string()));
    manager.add_component(
        suzzane,
        Gravity::new(Vector3 {
            x: 0.0,
            y: -98.0,
            z: 0.0,
        }),
    );

    manager.add_component(
        suzzane,
        RectangleCollider::new(
            1.0,
            2.0,
            1.0,
            Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        ),
    );

    let cube = manager.create_object(
        Model3d::from_obj("./assets/cube.obj".to_string()),
        Transform3d::new(
            Vector3 {
                x: 0.0,
                y: 5.0,
                z: 0.0,
            },
            Quaternion::one(),
        ),
    );

    manager.add_component(cube, Label("Cube".to_string()));

    let floor = manager.create_object(
        Model3d::from_obj("./assets/plane.obj".to_string()),
        Transform3d::new(
            Vector3 {
                x: 0.0,
                y: -10.0,
                z: 0.0,
            },
            Quaternion::one(),
        ),
    );

    manager.add_component(
        floor,
        StationaryPlaneCollider::new(
            10.0,
            10.0,
            Vector3 {
                x: 0.0,
                y: -10.0,
                z: 0.0,
            },
            Quaternion::one(),
        ),
    );

    let light1 = manager.add_light(Light::new((1.0, 0.1, 0.1)));
    manager.add_component(
        light1,
        Transform3d::new(
            Vector3 {
                x: 5.0,
                y: 5.0,
                z: 0.0,
            },
            Quaternion::one(),
        ),
    );

    manager.add_component(light1, Label("Red Light".to_string()));

    let light2 = manager.add_light(Light::new((0.1, 0.1, 1.0)));
    manager.add_component(
        light2,
        Transform3d::new(
            Vector3 {
                x: -5.0,
                y: 5.0,
                z: 0.0,
            },
            Quaternion::one(),
        ),
    );

    manager.add_component(light2, Label("Blue Light".to_string()));
}

fn add_camera(manager: &mut HeliumManager) {
    let config = manager.get_render_config();
    let camera = manager.create_camera(Camera3d::new(
        (5.0, 5.0, 5.0).into(),
        (-5.0, -5.0, -5.0).into(),
        Vector3::unit_y(),
        config.width as f32 / config.height as f32,
        45.0,
        0.1,
        100.0,
    ));

    manager.add_component(camera, CameraController::default());
    manager.add_component(
        camera,
        Transform3d::new(
            Vector3 {
                x: 5.0,
                y: 5.0,
                z: 5.0,
            },
            Quaternion::one(),
        ),
    );
}

fn update_model(manager: &mut HeliumManager) {
    let labels = manager.query::<Label>().unwrap();

    // let mut suzzane = None;
    let mut cube = None;

    let entities_with_labels = manager.entities_with::<Label>(|label| {
        label == &Label("Suzzane".to_string()) || label == &Label("Cube".to_string())
    });

    for entity in entities_with_labels {
        if let Some(label) = labels.get(&entity) {
            if label == &Label("Suzzane".to_string()) {
                // suzzane = Some(entity);
            } else if label == &Label("Cube".to_string()) {
                cube = Some(entity);
            }
        }
    }

    drop(labels);

    manager.set_rotation(
        cube.unwrap(),
        Quaternion::from_axis_angle(
            Vector3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            }
            .normalize(),
            Rad(manager.time.elapsed().as_secs_f32()),
        ),
    );
}

fn process_inputs(manager: &mut HeliumManager, event: &InputEvent) {
    let mut cameras = manager.query_mut::<CameraController>().unwrap();

    for (_, camera) in cameras.iter_mut() {
        camera.process_events(event);
    }
}

fn update_lights(manager: &mut HeliumManager) {
    let lights = match manager.query::<Light>() {
        Some(lights) => lights,
        None => return,
    };

    let mut transforms = match manager.query_mut::<Transform3d>() {
        Some(transforms) => transforms,
        None => return,
    };

    let labels = match manager.query::<Label>() {
        Some(labels) => labels,
        None => return,
    };

    for (entity, _light) in lights.iter() {
        if let Some(transform) = transforms.get_mut(&entity) {
            if let Some(label) = labels.get(&entity) {
                let x = if label == &Label("Red Light".to_string()) {
                    5.0 * f32::cos(manager.time.elapsed().as_secs_f32())
                } else {
                    -5.0 * f32::cos(manager.time.elapsed().as_secs_f32())
                };

                let z = if label == &Label("Red Light".to_string()) {
                    5.0 * f32::sin(manager.time.elapsed().as_secs_f32())
                } else {
                    -5.0 * f32::sin(manager.time.elapsed().as_secs_f32())
                };

                let new_position = Vector3 { x, y: 5.0, z };

                transform.update_position(new_position);
                // info!("Transform: {:#?}", transform);
            }
        }
    }
}

fn main() {
    let _helium = Helium::new()
        .add_startup(add_model)
        .add_startup(add_camera)
        .add_update(update_model)
        .add_update(update_lights)
        .add_input(process_inputs)
        .run();
}
