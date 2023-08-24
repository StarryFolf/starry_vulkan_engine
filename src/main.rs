pub mod app;
pub mod engine;

use std::{sync::Arc, time::Instant};

use cgmath::Vector3;
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
    camera::StarryCamera,
    game::game_object::{StarryGameObject, TransformComponent},
    rendering::{
        command_buffer::StarryCommandBuffer, device::StarryDevice, pipeline::StarryPipeline,
        render_pass::StarryRenderPass, shader::StarryShader, surface::StarrySurface,
        swapchain::StarrySwapchain,
    },
    resources::{model::StarryModel, vertex::StarryVertex},
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

    let surface = StarrySurface::new(
        800,
        800,
        "Starry Vulkan Engine",
        &event_loop,
        instance.clone(),
    );

    let (device, mut queues) = StarryDevice::create_device_and_queues(instance, surface.clone());

    let queue = queues.next().expect("No queues available.");

    let (mut swapchain, images) =
        StarrySwapchain::create_swapchain_and_images(device.clone(), surface.clone());

    let memory_allocator = StandardMemoryAllocator::new_default(device.clone());

    let command_buffers_allocator =
        StandardCommandBufferAllocator::new(device.clone(), Default::default());

    let model = StarryModel::new(
        Box::new([
            StarryVertex {
                position: [0.5, -0.5, 0.0],
                color: [1.0, 0.0, 0.0, 1.0],
            },
            StarryVertex {
                position: [0.5, 0.5, 0.0],
                color: [0.0, 1.0, 0.0, 1.0],
            },
            StarryVertex {
                position: [-0.5, 0.5, 0.0],
                color: [0.0, 0.0, 1.0, 1.0],
            },
            StarryVertex {
                position: [-0.5, -0.5, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
        ]),
        Box::new([3, 0, 1, 1, 2, 3]),
        &memory_allocator,
        &command_buffers_allocator,
        queue.clone().queue_family_index(),
        queue.clone(),
    );

    let mut quad = StarryGameObject::create_new_game_object_with_transform(
        model,
        TransformComponent {
            translation: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 2.0,
            },
            scale: Vector3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            rotation: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        },
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

    let mut recreate_swapchain = false;

    let mut previous_frame_end = Some(sync::now(device.clone()).boxed());

    let mut camera = StarryCamera::new();
    // camera.set_view_direction(
    //     Vector3::zero(),
    //     Vector3 { x: 0.5, y: 0.0, z: 1.0 },
    //     Vector3 { x: 0.0, y: -1.0, z: 0.0 }
    // );

    camera.set_view_target(
        Vector3 {
            x: 10.0,
            y: -10.0,
            z: 10.0,
        },
        quad.transform.translation,
        Vector3 {
            x: 0.0,
            y: -1.0,
            z: 0.0,
        },
    );

    let mut view_object = StarryGameObject::create_new_game_object(
        StarryModel::new(
            Box::new([
                StarryVertex {
                color: [0.0, 0.0, 0.0, 0.0],
                position: [0.0, 0.0, 0.0]
                }
            ]), 
            Box::new([0, 0, 0]), 
            &memory_allocator,
            &command_buffers_allocator,
            queue.clone().queue_family_index(),
            queue.clone()
        )
    );

    println!("{:?}", view_object.transform.translation);

    // let mut current_scale = 0.0;

    let mut current_time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        let new_time = Instant::now();
        let frame_time = (new_time - current_time).as_secs_f32();

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

            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                view_object.move_in_plane_xz(frame_time, input, 180.0, 5.0);
            }

            Event::RedrawEventsCleared => {
                current_time = new_time;

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
                    frame_buffers =
                        redraw_swapchain(&new_images, render_pass.clone(), &mut viewport);

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

                quad.transform.rotation.x = (quad.transform.rotation.x + 180.0 * frame_time) % 360.0;
                quad.transform.rotation.z = (quad.transform.rotation.z + 180.0 * frame_time) % 360.0;
                quad.transform.rotation.y = (quad.transform.rotation.y - 180.0 * frame_time) % 360.0;

                let extent = swapchain.image_extent();
                let aspect = extent[0] as f32 / extent[1] as f32;
                // camera.set_orthographic_projection(-aspect, aspect, -1.0, 1.0, -1.0, 1.0);
                camera.set_perspective_projection(50.0, aspect, 0.1, 100.0);
                camera.set_view_xyz(view_object.transform.translation, view_object.transform.rotation);

                let projection_view = camera.get_projection_matrix() * camera.get_view_matrix();

                let command_buffer = StarryCommandBuffer::create_command_buffer_with_push_constant(
                    &command_buffers_allocator,
                    queue.queue_family_index(),
                    image_index,
                    frame_buffers.clone(),
                    viewport.clone(),
                    graphics_pipeline.clone(),
                    quad.model.clone(),
                    StarryShader::create_push_constant_data_struct(
                        projection_view * quad.transform.get_transform_matrix(),
                    ),
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
