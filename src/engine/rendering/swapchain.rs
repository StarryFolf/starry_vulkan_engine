use std::sync::Arc;

use vulkano::{swapchain::{Swapchain, Surface, SwapchainCreateInfo, SwapchainCreationError}, image::{SwapchainImage, ImageUsage}, device::Device};
use winit::dpi::PhysicalSize;

use super::surface::StarrySurface;

pub struct StarrySwapchain {

}

impl StarrySwapchain {
    pub fn create_swapchain_and_images(
        device: Arc<Device>, 
        surface: Arc<Surface>,

    ) -> (
        Arc<Swapchain>, 
        Vec<Arc<SwapchainImage>>
    ) {
        let surface_capabilities = device
            .physical_device()
            .surface_capabilities(&surface, Default::default())
            .unwrap();

        let composite_alpha = surface_capabilities
            .supported_composite_alpha
            .into_iter()
            .next()
            .unwrap();

        let image_format = Some(device
            .physical_device()
            .surface_formats(&surface, Default::default())
            .unwrap()[0]
            .0);


        Swapchain::new(
            device.clone(), 
            surface.clone(), 
            SwapchainCreateInfo {
                image_format,
                composite_alpha,
                image_extent: StarrySurface::get_extent(surface.clone()).into(),
                min_image_count: surface_capabilities.min_image_count,
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                ..Default::default()
            }
        )
        .unwrap()
    }

    pub fn recreate_swapchain_and_images(
        swapchain: Arc<Swapchain>,
        dimensions: PhysicalSize<u32>,
        device: Arc<Device>,
        surface: Arc<Surface>
    ) -> Result<(Arc<Swapchain>, Vec<Arc<SwapchainImage>>), SwapchainCreationError> {
        match swapchain.recreate(SwapchainCreateInfo {
            image_extent: dimensions.into(),
            ..swapchain.create_info()
        }) {
            Ok(r) => Ok(r),
            // This error tends to happen when the user is manually resizing the
            // window. Simply restarting the loop is the easiest way to fix this
            // issue.
            Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => {
                let surface_capabilities = device
                    .physical_device()
                    .surface_capabilities(&surface, Default::default())
                    .unwrap();
                Err(SwapchainCreationError::ImageExtentNotSupported {
                    provided: surface_capabilities.current_extent.unwrap(),
                    min_supported: surface_capabilities.min_image_extent,
                    max_supported: surface_capabilities.max_image_extent
                })
            },
            Err(e) => Err(e),
        }
    }
}