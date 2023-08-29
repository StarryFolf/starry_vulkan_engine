use std::sync::Arc;

use vulkano::{
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassContents,
    },
    pipeline::{graphics::viewport::Viewport, GraphicsPipeline, Pipeline, PipelineBindPoint},
    render_pass::Framebuffer, descriptor_set::PersistentDescriptorSet,
};

use crate::{engine::resources::model::StarryModel, vs};

pub struct StarryCommandBuffer;

impl StarryCommandBuffer {
    pub fn create_default_command_buffer(
        command_buffers_allocator: &StandardCommandBufferAllocator,
        queue_family_index: u32,
        image_index: u32,
        frame_buffers: Vec<Arc<Framebuffer>>,
        viewport: Viewport,
        pipeline: Arc<GraphicsPipeline>,
        model: StarryModel,
    ) -> PrimaryAutoCommandBuffer {
        let mut builder = AutoCommandBufferBuilder::primary(
            command_buffers_allocator,
            queue_family_index,
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(frame_buffers[image_index as usize].clone())
                },
                SubpassContents::Inline,
            )
            .unwrap()
            .set_viewport(0, [viewport])
            .bind_pipeline_graphics(pipeline)
            .bind_vertex_buffers(0, model.vertex_buffer)
            .bind_index_buffer(model.index_buffer.clone())
            .draw_indexed(model.index_buffer.len() as u32, 1, 0, 0, 0)
            .unwrap()
            .end_render_pass()
            .unwrap();

        builder.build().unwrap()
    }

    pub fn create_standard_command_buffer(
        command_buffers_allocator: &StandardCommandBufferAllocator,
        queue_family_index: u32,
        image_index: u32,
        frame_buffers: Vec<Arc<Framebuffer>>,
        viewport: Viewport,
        pipeline: Arc<GraphicsPipeline>,
        model: StarryModel,
        push_constants: vs::PushConstantData,
        descriptor_sets: Arc<PersistentDescriptorSet>,
    ) -> PrimaryAutoCommandBuffer {
        let mut builder = AutoCommandBufferBuilder::primary(
            command_buffers_allocator,
            queue_family_index,
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![
                        Some([0.0, 0.0, 0.0, 1.0].into()),
                        Some(1f32.into()),
                    ],
                    ..RenderPassBeginInfo::framebuffer(frame_buffers[image_index as usize].clone())
                },
                SubpassContents::Inline,
            )
            .unwrap()
            .set_viewport(0, [viewport])
            .bind_pipeline_graphics(pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics, 
                pipeline.layout().clone(), 
                0, 
                descriptor_sets
            )
            .bind_vertex_buffers(0, model.vertex_buffer)
            .bind_index_buffer(model.index_buffer.clone())
            .push_constants(pipeline.layout().clone(), 0, push_constants)
            .draw_indexed(model.index_buffer.len() as u32, 1, 0, 0, 0)
            .unwrap()
            .end_render_pass()
            .unwrap();

        builder.build().unwrap()
    }
}
