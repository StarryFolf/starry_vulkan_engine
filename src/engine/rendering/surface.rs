use std::sync::Arc;

use vulkano::{instance::Instance, swapchain::Surface};
use vulkano_win::VkSurfaceBuild;
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event_loop::EventLoopWindowTarget,
    window::{Window, WindowBuilder},
};

pub struct StarrySurface;

impl StarrySurface {
    pub fn new(
        width: i32,
        height: i32,
        window_name: &str,
        event_loop: &EventLoopWindowTarget<()>,
        instance: Arc<Instance>,
    ) -> Arc<Surface> {
        WindowBuilder::new()
            .with_inner_size(LogicalSize::new(width, height))
            .with_title(window_name)
            .build_vk_surface(event_loop, instance)
            .unwrap()
    }

    pub fn new_default(
        event_loop: &EventLoopWindowTarget<()>,
        instance: Arc<Instance>,
    ) -> Arc<Surface> {
        WindowBuilder::new()
            .build_vk_surface(event_loop, instance)
            .unwrap()
    }

    // pub fn get_window(surface: Arc<Surface>) -> &'static Window {
    //     surface.object().unwrap().downcast_ref::<&'static Window>().expect("Cannot get window.")
    // }

    pub fn get_extent(surface: Arc<Surface>) -> PhysicalSize<u32> {
        surface
            .object()
            .unwrap()
            .downcast_ref::<Window>()
            .unwrap()
            .inner_size()
    }
}
