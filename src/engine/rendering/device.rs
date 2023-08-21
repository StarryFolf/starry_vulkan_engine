use std::sync::Arc;

use vulkano::{instance::Instance, swapchain::{Surface, PresentMode}, device::{Device, Queue, DeviceExtensions, QueueFlags, physical::PhysicalDeviceType, QueueCreateInfo, DeviceCreateInfo}};

pub struct StarryDevice {

}

impl StarryDevice {
    pub fn create_device_and_queues(
        instance: Arc<Instance>, 
        surface: Arc<Surface>,
    ) -> (
        Arc<Device>, 
        impl ExactSizeIterator<Item = Arc<Queue>>
    ) {
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..Default::default()
        };

        let (physical_device, queue_family_index) = instance
        .enumerate_physical_devices()
        .unwrap()
        .filter(|p| {
            p.supported_extensions().contains(&device_extensions)
        })
        .filter_map(|p| {
            p.queue_family_properties()
            .iter()
            .enumerate()
            .position(|(i, q)| {
                q.queue_flags.intersects(QueueFlags::GRAPHICS)
                    && p.surface_support(i as u32, &surface).unwrap_or(false)
            })
            .map(|i| (p, i as u32))
        })
        .min_by_key(|(p, _)| {
            match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5
            }
        })
        .expect("Error: No physical device found.");

        Device::new(
            physical_device, 
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            }
        )
        .unwrap()
    }

    pub fn get_present_modes(device: Arc<Device>, surface: Arc<Surface>) -> Vec<PresentMode>{
        device.physical_device().surface_present_modes(&surface).unwrap().collect()
    }
}