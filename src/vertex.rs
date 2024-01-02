use vulkano::buffer::BufferContents;

#[derive(
    BufferContents, vulkano::pipeline::graphics::vertex_input::Vertex, Default, Clone, Debug,
)]
#[repr(C)]
pub struct Vertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    pub normal: [f32; 3],
    #[format(R32G32_SFLOAT)]
    pub tex_coord: [f32; 2],
}
