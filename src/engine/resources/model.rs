use std::sync::Arc;

use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryUsage}, device::Queue, command_buffer::{AutoCommandBufferBuilder, allocator::StandardCommandBufferAllocator, CommandBufferUsage, CopyBufferInfo, PrimaryCommandBufferAbstract}, sync::GpuFuture,
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
        command_buffers_allocator: &StandardCommandBufferAllocator,
        queue_family_index: u32,
        queue: Arc<Queue>
    ) -> Self {
        let vertex_buffer = Self::create_vertex_buffer(
            memory_allocator, 
            vertices.clone(), 
            command_buffers_allocator, 
            queue_family_index,
            queue.clone()
        );
        let index_buffer = Self::create_index_buffer(
            memory_allocator, 
            indices.clone(), 
            command_buffers_allocator, 
            queue_family_index,
            queue.clone()
        );
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
        command_buffers_allocator: &StandardCommandBufferAllocator,
        queue_family_index: u32,
        queue: Arc<Queue>
    ) -> Subbuffer<[StarryVertex]> {
        let vertices_iter = vertices.into_vec().into_iter();

        let staging_buffer = Buffer::from_iter(
            memory_allocator, 
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            }, 
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            }, 
            vertices_iter.clone()
        )
        .unwrap();

        let vertex_buffer = Buffer::new_slice::<StarryVertex>(
            memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER | BufferUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::DeviceOnly,
                ..Default::default()
            },
            vertices_iter.len() as u64,
        )
        .unwrap();

        let mut ccb = AutoCommandBufferBuilder::primary(
            command_buffers_allocator, 
            queue_family_index, 
            CommandBufferUsage::OneTimeSubmit
        )
        .unwrap();

        ccb.copy_buffer(
            CopyBufferInfo::buffers(
                staging_buffer, 
                vertex_buffer.clone()
            )
        )
        .unwrap();

        let cb = ccb.build().unwrap();

        cb
            .execute(queue)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();

        vertex_buffer
    }

    pub fn create_index_buffer(
        memory_allocator: &(impl MemoryAllocator + ?Sized),
        indices: Box<[u32]>,
        command_buffers_allocator: &StandardCommandBufferAllocator,
        queue_family_index: u32,
        queue: Arc<Queue>
    ) -> Subbuffer<[u32]> {
        let indices_iter = indices.into_vec().into_iter();

        if indices_iter.len() % 3 != 0 {
            panic!("Indices count must be divisable by 3.")
        }

        let staging_buffer = Buffer::from_iter(
            memory_allocator, 
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            }, 
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            }, 
            indices_iter.clone()
        )
        .unwrap();

        let index_buffer = Buffer::new_slice::<u32>(
            memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER | BufferUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::DeviceOnly,
                ..Default::default()
            },
            indices_iter.len() as u64,
        )
        .unwrap();

        let mut ccb = AutoCommandBufferBuilder::primary(
            command_buffers_allocator, 
            queue_family_index, 
            CommandBufferUsage::OneTimeSubmit
        )
        .unwrap();

        ccb.copy_buffer(
            CopyBufferInfo::buffers(
                staging_buffer, 
                index_buffer.clone()
            )
        )
        .unwrap();

        let cb = ccb.build().unwrap();

        cb
            .execute(queue)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();

        index_buffer
    }

    // pub fn
}
