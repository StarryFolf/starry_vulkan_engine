use std::sync::Arc;

use vulkano::{render_pass::RenderPass, device::Device, swapchain::Swapchain};

pub struct StarryRenderPass{

}

impl StarryRenderPass {
    pub fn create_single_pass_render_pass(
        device: Arc<Device>,
        swapchain: Arc<Swapchain>
    ) -> Arc<RenderPass> {
        vulkano::single_pass_renderpass!(
            device,
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.image_format(),
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
            )
            .unwrap()
    }
}