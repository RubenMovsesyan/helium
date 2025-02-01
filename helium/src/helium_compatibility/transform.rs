use cgmath::{One, Quaternion, Vector3, Zero};
use helium_renderer::instance::Instance;

#[derive(Clone, Copy, Debug)]
pub struct Transform3d {
    position: Vector3<f32>,
    rotation: Quaternion<f32>,
    update_flag: bool,
}

impl Default for Transform3d {
    fn default() -> Self {
        Self {
            position: Vector3::zero(),
            rotation: Quaternion::one(),
            update_flag: false,
        }
    }
}

impl Transform3d {
    pub fn new(position: Vector3<f32>, rotation: Quaternion<f32>) -> Self {
        Self {
            position,
            rotation,
            update_flag: false,
        }
    }

    // Setters
    pub fn update_position(&mut self, new_position: Vector3<f32>) {
        self.position = new_position;
        self.update_flag = true;
    }

    pub fn add_position(&mut self, position_add: Vector3<f32>) {
        self.position += position_add;
        self.update_flag = true;
    }

    pub fn update_rotation(&mut self, new_rotation: Quaternion<f32>) {
        self.rotation = new_rotation;
        self.update_flag = true;
    }

    pub fn update_transform(&mut self, new_position: Vector3<f32>, new_rotation: Quaternion<f32>) {
        self.position = new_position;
        self.rotation = new_rotation;
        self.update_flag = true;
    }

    pub fn update(&mut self) {
        self.update_flag = false;
    }

    pub fn get_update_flag(&self) -> &bool {
        &self.update_flag
    }

    // Getters
    pub fn get_position(&self) -> &Vector3<f32> {
        &self.position
    }

    pub fn get_rotation(&self) -> &Quaternion<f32> {
        &self.rotation
    }

    pub fn set_rotation(&mut self, new_rotation: Quaternion<f32>) {
        self.rotation = new_rotation;
        self.update_flag = true;
    }

    pub fn get_transform(&self) -> (&Vector3<f32>, &Quaternion<f32>) {
        (&self.position, &self.rotation)
    }

    // Static functions
    pub fn translate(transform: &mut Self, translation: Vector3<f32>) {
        transform.position += translation;
        transform.update_flag = true;
    }

    pub fn set_position(transform: &mut Self, position: Vector3<f32>) {
        transform.position = position;
        transform.update_flag = true;
    }

    // pub fn set_rotation(transform: &mut Self, rotation: Quaternion<f32>) {
    //     transform.rotation = rotation;
    //     transform.update_flag = true;
    // }
}

impl From<Transform3d> for Instance {
    fn from(value: Transform3d) -> Self {
        Instance {
            position: value.position,
            rotation: value.rotation,
        }
    }
}
