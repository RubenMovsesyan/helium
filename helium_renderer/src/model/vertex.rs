use wgpu::VertexBufferLayout;

pub trait Vertex {
    fn desc() -> VertexBufferLayout<'static>;
}
