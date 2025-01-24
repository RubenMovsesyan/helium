use cgmath::{One, Quaternion, Vector3, Zero};
use helium_renderer::instance::Instance;

#[derive(Clone, Copy)]
pub struct Transform3d {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
}

impl Default for Transform3d {
    fn default() -> Self {
        Self {
            position: Vector3::zero(),
            rotation: Quaternion::one(),
        }
    }
}

impl Transform3d {
    pub fn new(position: Vector3<f32>, rotation: Quaternion<f32>) -> Self {
        Self { position, rotation }
    }

    // Setters
    pub fn update_position(&mut self, new_position: Vector3<f32>) {
        self.position = new_position;
    }

    pub fn update_rotation(&mut self, new_rotation: Quaternion<f32>) {
        self.rotation = new_rotation;
    }

    pub fn update_transform(&mut self, new_position: Vector3<f32>, new_rotation: Quaternion<f32>) {
        self.position = new_position;
        self.rotation = new_rotation;
    }

    // Getters
    pub fn get_position(&self) -> &Vector3<f32> {
        &self.position
    }

    pub fn get_rotation(&self) -> &Quaternion<f32> {
        &self.rotation
    }

    pub fn get_transform(&self) -> (&Vector3<f32>, &Quaternion<f32>) {
        (&self.position, &self.rotation)
    }
}

impl Into<Instance> for Transform3d {
    fn into(self) -> Instance {
        Instance {
            position: self.position,
            rotation: self.rotation,
        }
    }
}
