pub struct Model3d {
    model_path: String,
    renderer_index: Option<usize>,
}

impl Model3d {
    pub fn from_obj(file_path: String) -> Self {
        Self {
            model_path: file_path,
            renderer_index: None,
        }
    }

    pub fn get_path(&self) -> &str {
        &self.model_path
    }

    /// Used internally to link the component to the renderer
    pub fn set_renderer_index(&mut self, index: usize) {
        self.renderer_index = Some(index);
    }

    /// Used internally to get information about the model from the renderer
    pub fn get_renderer_index(&self) -> Option<&usize> {
        self.renderer_index.as_ref()
    }
}
