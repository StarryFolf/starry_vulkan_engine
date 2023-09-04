pub mod app;
pub mod engine;

use std::{sync::Arc, time::{Instant, Duration}};

use cgmath::Vector3;
use vulkano::{
    command_buffer::{allocator::StandardCommandBufferAllocator, CommandBufferUsage, AutoCommandBufferBuilder, PrimaryCommandBufferAbstract},
    image::{view::ImageView, ImageAccess, SwapchainImage, AttachmentImage},
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::StandardMemoryAllocator,
    pipeline::{graphics::viewport::Viewport, Pipeline, GraphicsPipeline},
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
    swapchain::{acquire_next_image, AcquireError, SwapchainCreationError, SwapchainPresentInfo},
    sync::{self, FlushError, GpuFuture},
    VulkanLibrary, descriptor_set::{PersistentDescriptorSet, allocator::StandardDescriptorSetAllocator, WriteDescriptorSet}, format::Format, shader::ShaderModule, device::Device,
};
use winit::{
    event::{Event, WindowEvent, ElementState},
    event_loop::{ControlFlow, EventLoop},
};

use crate::engine::{
    camera::StarryCamera,
    game::game_object::{StarryGameObject, TransformComponent},
    rendering::{
        command_buffer::StarryCommandBuffer, device::StarryDevice, pipeline::StarryPipeline,
        render_pass::StarryRenderPass, surface::StarrySurface,
        swapchain::StarrySwapchain,
    },
    resources::{model::StarryModel, vertex::StarryVertex, texture::StarryTexture},
};

create_shader!("vertex", "assets/shaders/shader.vert", vs);
create_shader!("fragment", "assets/shaders/shader.frag", fs);

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

    // StarryTexture::create_image(device.clone());

    let queue = queues.next().expect("No queues available.");

    let (mut swapchain, images) =
        StarrySwapchain::create_swapchain_and_images(device.clone(), surface.clone());

    let memory_allocator = StandardMemoryAllocator::new_default(device.clone());

    let command_buffers_allocator =
        StandardCommandBufferAllocator::new(device.clone(), Default::default());

    let descriptor_set_allocator = StandardDescriptorSetAllocator::new(device.clone());

    let mut texture_builder = AutoCommandBufferBuilder::primary(
        &command_buffers_allocator,
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    let model = StarryModel::create_model_from_file(
        "assets/models/viking_room.obj",
        &memory_allocator,
        &command_buffers_allocator,
        queue.clone(),
    );

    #[allow(unused_mut)]
    let mut object = StarryGameObject::create_new_game_object_with_transform(
        model,
        TransformComponent {
            translation: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 2.0,
            },
            scale: Vector3 {
                x: 0.5,
                y: 0.5,
                z: 0.5,
            },
            rotation: Vector3 {
                x: 90.0,
                y: 180.0,
                z: 0.0,
            },
        },
    );

    let vs = vs::load(device.clone()).unwrap();
    let fs = fs::load(device.clone()).unwrap();

    let render_pass =
        StarryRenderPass::create_single_pass_render_pass(device.clone(), swapchain.clone());

    let mut viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [0.0, 0.0],
        depth_range: 0.0..1.0,
    };

    let (mut graphics_pipeline ,mut frame_buffers) = redraw_frame_buffers_and_pipeline(
        vs.clone(), 
        fs.clone(), 
        &memory_allocator, 
        &images, 
        render_pass.clone(), 
        &mut viewport, 
        device.clone()
    );

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
        object.transform.translation,
        Vector3 {
            x: 0.0,
            y: -1.0,
            z: 0.0,
        },
    );

    let mut view_object = StarryGameObject::create_new_game_object(
        StarryModel::new(
            Box::new(
                vec![StarryVertex {
                    color: [0.0, 0.0, 0.0, 0.0],
                    position: [0.0, 0.0, 0.0],
                    normal: [0.0, 0.0, 0.0],
                    uv: [0.0, 0.0]
                }]
            ), 
            Box::new(vec![0, 0, 0]), 
            &memory_allocator,
            &command_buffers_allocator,
            queue.clone()
        )
    );

    // let mut current_scale = 0.0;

    let texture = StarryTexture::create_texture(
        "assets/textures/viking_room.png", 
        &memory_allocator, 
        &mut texture_builder,
    );

    let sampler = StarryTexture::create_default_sampler(device.clone());

    let layout = graphics_pipeline.layout().set_layouts().get(0).unwrap();
    let set = PersistentDescriptorSet::new(
        &descriptor_set_allocator,
        layout.clone(),
        [WriteDescriptorSet::image_view_sampler(0, texture, sampler)]
    )
    .unwrap();

    let mut recreate_swapchain = false;

    let mut previous_frame_end = Some(
        texture_builder
            .build()
            .unwrap()
            .execute(queue.clone())
            .unwrap()
            .boxed()
    );

    let target_fps = 140.0;
    let max_frame_time = Duration::from_secs_f32(1.0 / target_fps);

    let mut previous_frame_time = Instant::now();

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

            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                let current_time = Instant::now();
                let delta_time = current_time - previous_frame_time;
                match input.state {
                    ElementState::Pressed => {
                        view_object.move_in_plane_xz(delta_time.as_secs_f64() as f32, input, 180.0, 5.0);
                    }

                    ElementState::Released => ()
                }
            }

            Event::RedrawEventsCleared => {
            let current_time = Instant::now();
            let delta_time = current_time - previous_frame_time;
            previous_frame_time = current_time;
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
                    
                    let (new_pipeline, new_framebuffers) = redraw_frame_buffers_and_pipeline(
                        vs.clone(),
                        fs.clone(),
                        &memory_allocator,
                        &new_images,
                        render_pass.clone(),
                        &mut viewport,
                        device.clone()
                    );

                    frame_buffers = new_framebuffers;

                    graphics_pipeline = new_pipeline;

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

                object.transform.rotation.x = (object.transform.rotation.x + 90.0 * delta_time.as_secs_f64() as f32) % 360.0;
                object.transform.rotation.z = (object.transform.rotation.z + 180.0 * delta_time.as_secs_f64() as f32) % 360.0;
                object.transform.rotation.y = (object.transform.rotation.y - 270.0 * delta_time.as_secs_f64() as f32) % 360.0;

                if delta_time < max_frame_time {
                    std::thread::sleep(max_frame_time - delta_time);
                }

                let extent = swapchain.image_extent();
                let aspect = extent[0] as f32 / extent[1] as f32;
                // camera.set_orthographic_projection(-aspect, aspect, -1.0, 1.0, -1.0, 1.0);
                camera.set_perspective_projection(50.0, aspect, 0.1, 100.0);
                camera.set_view_xyz(view_object.transform.translation, view_object.transform.rotation);

                let projection_view = camera.get_projection_matrix() * camera.get_view_matrix();

                let command_buffer = StarryCommandBuffer::create_standard_command_buffer(
                    &command_buffers_allocator,
                    queue.queue_family_index(),
                    image_index,
                    frame_buffers.clone(),
                    viewport.clone(),
                    graphics_pipeline.clone(),
                    object.model.clone(),
                    vs::PushConstantData {
                        transform: (projection_view * object.transform.get_transform_matrix()).into(),
                    },
                    set.clone()
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
                        println!("failed to flush future: {e}");
                        previous_frame_end = Some(sync::now(device.clone()).boxed());
                    }
                }
            }

            _ => (),
        }
    });


    fn redraw_frame_buffers_and_pipeline(
        vs: Arc<ShaderModule>,
        fs: Arc<ShaderModule>,
        memory_allocator: &StandardMemoryAllocator,
        images: &[Arc<SwapchainImage>],
        render_pass: Arc<RenderPass>,
        viewport: &mut Viewport,
        device: Arc<Device>
    ) -> (Arc<GraphicsPipeline>, Vec<Arc<Framebuffer>>) {
        let dimensions = images[0].dimensions().width_height();
        viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];

        let depth_buffer = ImageView::new_default(
            AttachmentImage::transient(memory_allocator, dimensions, Format::D32_SFLOAT).unwrap(),
        )
        .unwrap();

        let frame_buffers = images
            .iter()
            .map(|image| {
                let view = ImageView::new_default(image.clone()).unwrap();
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view, depth_buffer.clone()],
                        ..Default::default()
                    },
                )
                .unwrap()
            })
            .collect::<Vec<_>>();

        let pipeline = StarryPipeline::create_default_graphics_pipeline(
            vs, 
            fs, 
            device.clone(), 
            render_pass
        );

        (pipeline, frame_buffers)
    }
}
