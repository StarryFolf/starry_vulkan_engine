use std::sync::Arc;

use vulkano::{pipeline::{GraphicsPipeline, graphics::{vertex_input::Vertex, input_assembly::{InputAssemblyState, PrimitiveTopology}, viewport::ViewportState, rasterization::RasterizationState, multisample::MultisampleState, color_blend::ColorBlendState}}, shader::ShaderModule, device::Device, render_pass::{RenderPass, Subpass}};

use crate::engine::resources::vertex::StarryVertex;

pub struct StarryPipeline;

impl StarryPipeline {
    pub fn create_default_graphics_pipeline(
        vertex_shader: Arc<ShaderModule>,
        fragment_shader: Arc<ShaderModule>,
        device: Arc<Device>,
        render_pass: Arc<RenderPass>
    ) -> Arc<GraphicsPipeline> {
        GraphicsPipeline::start()
            .vertex_input_state(StarryVertex::per_vertex())
            .vertex_shader(vertex_shader.entry_point("main").unwrap(), ())
            .input_assembly_state(InputAssemblyState::new().topology(PrimitiveTopology::TriangleList))
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .rasterization_state(RasterizationState::default())
            .multisample_state(MultisampleState::default())
            .color_blend_state(ColorBlendState::default())
            .fragment_shader(fragment_shader.entry_point("main").unwrap(), ())
            .render_pass(Subpass::from(render_pass, 0).unwrap())
            .build(device)
            .unwrap()
    }
}