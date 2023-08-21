use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryUsage},
};

use super::vertex::StarryVertex;

pub struct StarryModelBuilder {}

#[derive(Clone /*, Copy*/)]
pub struct StarryModel {
    pub vertices: Box<[StarryVertex]>,
    pub vertex_buffer: Subbuffer<[StarryVertex]>,
    pub indices: Box<[u32]>,
    pub index_buffer: Subbuffer<[u32]>,
}

impl StarryModel {
    pub fn new(
        vertices: Box<[StarryVertex]>,
        indices: Box<[u32]>,
        memory_allocator: &(impl MemoryAllocator + ?Sized),
    ) -> Self {
        let vertex_buffer = Self::create_vertex_buffer(memory_allocator, vertices.clone());
        let index_buffer = Self::create_index_buffer(memory_allocator, indices.clone());
        Self {
            vertices,
            vertex_buffer,
            indices,
            index_buffer,
        }
    }

    pub fn create_vertex_buffer(
        memory_allocator: &(impl MemoryAllocator + ?Sized),
        vertices: Box<[StarryVertex]>,
    ) -> Subbuffer<[StarryVertex]> {
        let vertices_iter = vertices.into_vec().into_iter();

        if vertices_iter.len() < 3 {
            panic!("Vertices count must be no less than 3.")
        }

        Buffer::from_iter(
            memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            vertices_iter,
        )
        .unwrap()
    }

    pub fn create_index_buffer(
        memory_allocator: &(impl MemoryAllocator + ?Sized),
        indices: Box<[u32]>,
    ) -> Subbuffer<[u32]> {
        let indices_iter = indices.into_vec().into_iter();

        if indices_iter.len() % 3 != 0 {
            panic!("Indices count must be divisable by 3.")
        }

        Buffer::from_iter(
            memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            indices_iter,
        )
        .unwrap()
    }

    // pub fn
}
