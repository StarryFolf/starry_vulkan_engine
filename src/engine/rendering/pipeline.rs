use std::sync::Arc;

use vulkano::{
    device::Device,
    pipeline::{
        graphics::{
            color_blend::ColorBlendState,
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::Vertex,
            viewport::ViewportState, depth_stencil::DepthStencilState,
        },
        GraphicsPipeline,
    },
    render_pass::{RenderPass, Subpass},
    shader::ShaderModule
};

use crate::engine::resources::vertex::StarryVertex;

pub struct StarryPipeline;

impl StarryPipeline {
    pub fn create_default_graphics_pipeline(
        vertex_shader: Arc<ShaderModule>,
        fragment_shader: Arc<ShaderModule>,
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
    ) -> Arc<GraphicsPipeline> {
        GraphicsPipeline::start()
            .vertex_input_state(StarryVertex::per_vertex())
            .vertex_shader(vertex_shader.entry_point("main").unwrap(), ())
            .input_assembly_state(
                InputAssemblyState::new().topology(PrimitiveTopology::TriangleList),
            )
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .rasterization_state(RasterizationState::default())
            .multisample_state(MultisampleState::default())
            .color_blend_state(ColorBlendState::default())
            .fragment_shader(fragment_shader.entry_point("main").unwrap(), ())
            .depth_stencil_state(DepthStencilState::simple_depth_test())
            .render_pass(Subpass::from(render_pass, 0).unwrap())
            .build(device.clone())
            .unwrap()
    }
}
