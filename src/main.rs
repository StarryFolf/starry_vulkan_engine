pub mod app;
pub mod engine;

use std::sync::Arc;

use cgmath::Vector2;
use vulkano::{
    command_buffer::allocator::StandardCommandBufferAllocator,
    image::{view::ImageView, ImageAccess, SwapchainImage},
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::StandardMemoryAllocator,
    pipeline::graphics::viewport::Viewport,
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
    swapchain::{acquire_next_image, AcquireError, SwapchainCreationError, SwapchainPresentInfo},
    sync::{self, FlushError, GpuFuture},
    VulkanLibrary,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use crate::engine::{
    rendering::{
        command_buffer::StarryCommandBuffer, device::StarryDevice, pipeline::StarryPipeline,
        render_pass::StarryRenderPass, shader::StarryShader, surface::StarrySurface,
        swapchain::StarrySwapchain,
    },
    resources::{model::StarryModel, vertex::StarryVertex}, game::game_object::{StarryGameObject, Transform2DComponent},
};

fn main() {
    let library = VulkanLibrary::new().unwrap();

    let required_extensions = vulkano_win::required_extensions(&library);

    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: required_extensions,
            ..Default::default()
        },
    )
    .unwrap();

    let event_loop = EventLoop::new();

    let surface = StarrySurface::new(800, 800, "Hello", &event_loop, instance.clone());

    let (device, mut queues) = StarryDevice::create_device_and_queues(instance, surface.clone());

    let queue = queues.next().expect("No queues available.");

    let (mut swapchain, images) =
        StarrySwapchain::create_swapchain_and_images(device.clone(), surface.clone());

    let memory_allocator = StandardMemoryAllocator::new_default(device.clone());

    let model = StarryModel::new(
        Box::new([
            StarryVertex {
                position: [0.5, -0.5],
                color: [1.0, 0.0, 0.0, 1.0]
            },
            StarryVertex {
                position: [0.5, 0.5],
                color: [0.0, 1.0, 0.0, 1.0]
            },
            StarryVertex {
                position: [-0.5, 0.5],
                color: [0.0, 0.0, 1.0, 1.0]
            },
            StarryVertex {
                position: [-0.5, -0.5],
                color: [1.0, 1.0, 1.0, 1.0]
            },
        ]),
        Box::new([3, 0, 1, 1, 2, 3]),
        &memory_allocator
    );

    let mut quad = StarryGameObject::create_new_game_object_with_transform(
        model, 
        Transform2DComponent {
            translation: Vector2 { x: 0.0, y: 0.0 },
            scale: Vector2 { x: 0.5, y: 1.5 },
            rotation: 90.0,
        }
    );

    let vs = StarryShader::load_vertex_shader(device.clone());
    let fs = StarryShader::load_fragment_shader(device.clone());

    let render_pass =
        StarryRenderPass::create_single_pass_render_pass(device.clone(), swapchain.clone());

    let graphics_pipeline = StarryPipeline::create_default_graphics_pipeline(
        vs,
        fs,
        device.clone(),
        render_pass.clone(),
    );

    let mut viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [0.0, 0.0],
        depth_range: 0.0..1.0,
    };

    let mut frame_buffers = redraw_swapchain(&images, render_pass.clone(), &mut viewport);

    let command_buffers_allocator =
        StandardCommandBufferAllocator::new(device.clone(), Default::default());

    let mut recreate_swapchain = false;

    let mut previous_frame_end = Some(sync::now(device.clone()).boxed());

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                recreate_swapchain = true;
            }

            Event::RedrawEventsCleared => {
                let window_dimensions = StarrySurface::get_extent(surface.clone());

                if window_dimensions.height == 0 || window_dimensions.width == 0 {
                    return;
                }

                previous_frame_end.as_mut().unwrap().cleanup_finished();

                if recreate_swapchain {
                    let (new_swapchain, new_images) =
                        match StarrySwapchain::recreate_swapchain_and_images(
                            swapchain.clone(),
                            window_dimensions,
                            device.clone(),
                            surface.clone(),
                        ) {
                            Ok(r) => r,
                            Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
                            Err(e) => panic!("{e}"),
                        };

                    swapchain = new_swapchain;

                    // Because framebuffers contains a reference to the old swapchain, we need to
                    // recreate framebuffers as well.
                    frame_buffers = redraw_swapchain(
                        &new_images,
                        render_pass.clone(),
                        &mut viewport,
                    );

                    recreate_swapchain = false;
                }

                let (image_index, suboptimal, acquire_future) =
                    match acquire_next_image(swapchain.clone(), None) {
                        Ok(r) => r,
                        Err(AcquireError::OutOfDate) => {
                            recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("Failed to acquire next image: {e}"),
                    };

                if suboptimal {
                    recreate_swapchain = true;
                }

                // let command_buffer = StarryCommandBuffer::create_command_buffer(
                //     &command_buffers_allocator,
                //     queue.queue_family_index(),
                //     image_index,
                //     frame_buffers.clone(),
                //     viewport.clone(),
                //     graphics_pipeline.clone(),
                //     vertex_buffer.clone(),
                // );

                quad.transform.rotation = (quad.transform.rotation + 1.0) % 360.0;

                let command_buffer = StarryCommandBuffer::create_command_buffer_with_push_constant(
                    &command_buffers_allocator,
                    queue.queue_family_index(),
                    image_index,
                    frame_buffers.clone(),
                    viewport.clone(),
                    graphics_pipeline.clone(),
                    quad.model.clone(),
                    StarryShader::create_push_constant_data_struct(
                        quad.transform.get_2d_transform_matrix(),
                        quad.transform.translation,
                    )
                );

                let future = previous_frame_end
                    .take()
                    .unwrap()
                    .join(acquire_future)
                    .then_execute(queue.clone(), command_buffer)
                    .unwrap()
                    .then_swapchain_present(
                        queue.clone(),
                        SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_index),
                    )
                    .then_signal_fence_and_flush();

                match future {
                    Ok(future) => {
                        previous_frame_end = Some(future.boxed());
                    }

                    Err(FlushError::OutOfDate) => {
                        recreate_swapchain = true;
                        previous_frame_end = Some(sync::now(device.clone()).boxed());
                    }

                    Err(e) => {
                        panic!("Failed to flush future: {e}");
                    }
                }
            }

            _ => (),
        }
    });

    fn redraw_swapchain(
        images: &[Arc<SwapchainImage>],
        render_pass: Arc<RenderPass>,
        viewport: &mut Viewport,
    ) -> Vec<Arc<Framebuffer>> {
        let dimensions = images[0].dimensions().width_height();
        viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];

        images
            .iter()
            .map(|image| {
                let view = ImageView::new_default(image.clone()).unwrap();
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view],
                        ..Default::default()
                    },
                )
                .unwrap()
            })
            .collect::<Vec<_>>()
    }
}