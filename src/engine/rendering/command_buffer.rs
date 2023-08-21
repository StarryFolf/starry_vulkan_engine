use std::sync::Arc;

use vulkano::{
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassContents,
    },
    pipeline::{graphics::viewport::Viewport, GraphicsPipeline, Pipeline},
    render_pass::Framebuffer,
};

use crate::engine::resources::model::StarryModel;

use super::shader::vs;

pub struct StarryCommandBuffer;

impl StarryCommandBuffer {
    pub fn create_command_buffer(
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

    pub fn create_command_buffer_with_push_constant(
        command_buffers_allocator: &StandardCommandBufferAllocator,
        queue_family_index: u32,
        image_index: u32,
        frame_buffers: Vec<Arc<Framebuffer>>,
        viewport: Viewport,
        pipeline: Arc<GraphicsPipeline>,
        model: StarryModel,
        push_constants: vs::PushConstantData,
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
            .bind_pipeline_graphics(pipeline.clone())
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
