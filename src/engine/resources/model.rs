use std::{collections::HashMap, fs::File, io::BufReader, sync::Arc};

use cgmath::{Vector2, Vector3};
use tobj::LoadOptions;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        CopyBufferInfo, PrimaryCommandBufferAbstract,
    },
    device::Queue,
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryUsage},
    sync::GpuFuture,
};

use super::vertex::StarryVertex;

#[derive(Clone /*, Copy*/)]
pub struct StarryModel {
    pub vertex_buffer: Subbuffer<[StarryVertex]>,
    pub index_buffer: Subbuffer<[u32]>,
}

impl StarryModel {
    pub fn new(
        vertices: Box<Vec<StarryVertex>>,
        indices: Box<Vec<u32>>,
        memory_allocator: &(impl MemoryAllocator + ?Sized),
        command_buffers_allocator: &StandardCommandBufferAllocator,
        queue: Arc<Queue>,
    ) -> Self {
        let vertex_buffer = Self::create_vertex_buffer(
            memory_allocator,
            vertices.clone(),
            command_buffers_allocator,
            queue.clone(),
        );
        let index_buffer = Self::create_index_buffer(
            memory_allocator,
            indices.clone(),
            command_buffers_allocator,
            queue.clone(),
        );
        Self {
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn create_vertex_buffer(
        memory_allocator: &(impl MemoryAllocator + ?Sized),
        vertices: Box<Vec<StarryVertex>>,
        command_buffers_allocator: &StandardCommandBufferAllocator,
        queue: Arc<Queue>,
    ) -> Subbuffer<[StarryVertex]> {
        let vertices_iter = vertices.into_iter();

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
            vertices_iter.clone(),
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

        let mut cbb = AutoCommandBufferBuilder::primary(
            command_buffers_allocator,
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        cbb.copy_buffer(CopyBufferInfo::buffers(
            staging_buffer,
            vertex_buffer.clone(),
        ))
        .unwrap();

        let cb = cbb.build().unwrap();

        cb.execute(queue)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();

        vertex_buffer
    }

    pub fn create_index_buffer(
        memory_allocator: &(impl MemoryAllocator + ?Sized),
        indices: Box<Vec<u32>>,
        command_buffers_allocator: &StandardCommandBufferAllocator,
        queue: Arc<Queue>,
    ) -> Subbuffer<[u32]> {
        let indices_iter = indices.into_iter();

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
            indices_iter.clone(),
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

        let mut cbb = AutoCommandBufferBuilder::primary(
            command_buffers_allocator,
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        cbb.copy_buffer(CopyBufferInfo::buffers(
            staging_buffer,
            index_buffer.clone(),
        ))
        .unwrap();

        let cb = cbb.build().unwrap();

        cb.execute(queue)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();

        index_buffer
    }

    pub fn create_model_from_file(
        file_path: &str,
        memory_allocator: &(impl MemoryAllocator + ?Sized),
        command_buffers_allocator: &StandardCommandBufferAllocator,
        queue: Arc<Queue>,
    ) -> Self {
        let mut vertices: Vec<StarryVertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut unique_vertices = HashMap::new();

        let mut reader = BufReader::new(File::open(file_path).unwrap());

        let (models, _) = tobj::load_obj_buf(
            &mut reader,
            &LoadOptions {
                triangulate: true,
                ..Default::default()
            },
            |_| Ok(Default::default()),
        )
        .unwrap();

        for model in &models {
            let max_index = model.mesh.indices.clone().into_iter().max().unwrap();
            let pos_len = model.mesh.positions.len() as u32;
            let uv_len = model.mesh.texcoords.len() as u32;
            let normal_len = model.mesh.normals.len() as u32;

            for index in &model.mesh.indices {
                let pos_offset = (3 * index) as usize;
                let uv_offset = (2 * index) as usize;
                let normal_offset = (3 * index) as usize;

                let vertex = StarryVertex {
                    position: if max_index * 3 < pos_len {
                        Vector3 {
                            x: model.mesh.positions[pos_offset],
                            y: model.mesh.positions[pos_offset + 1],
                            z: model.mesh.positions[pos_offset + 2],
                        }
                        .into()
                    } else {
                        Vector3 {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        }
                        .into()
                    },
                    color: [1.0, 1.0, 1.0, 1.0],
                    uv: if max_index * 2 < uv_len {
                        Vector2 {
                            x: model.mesh.texcoords[uv_offset],
                            y: 1.0 - model.mesh.texcoords[uv_offset + 1],
                        }
                        .into()
                    } else {
                        Vector2 { x: 0.0, y: 0.0 }.into()
                    },
                    normal: if max_index * 3 < normal_len {
                        Vector3 {
                            x: model.mesh.normals[normal_offset],
                            y: model.mesh.normals[normal_offset + 1],
                            z: model.mesh.normals[normal_offset + 2],
                        }
                        .into()
                    } else {
                        Vector3 {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        }
                        .into()
                    },
                };

                if let Some(index) = unique_vertices.get(&vertex) {
                    indices.push(*index as u32);
                } else {
                    let index = vertices.len();
                    unique_vertices.insert(vertex, index);
                    vertices.push(vertex);
                    indices.push(index as u32);
                }
            }
        }

        Self::new(
            Box::new(vertices),
            Box::new(indices),
            memory_allocator,
            command_buffers_allocator,
            queue,
        )
    }
}
