use std::{fs::File, sync::Arc};

use png::Decoder;
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer},
    device::Device,
    format::Format,
    image::{view::ImageView, ImageDimensions, ImmutableImage, MipmapsCount},
    memory::allocator::MemoryAllocator,
    sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo},
};

pub struct StarryTexture {}

impl StarryTexture {
    pub fn create_texture(
        img_path: &str,
        memory_allocator: &(impl MemoryAllocator + ?Sized),
        command_buffers_builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) -> Arc<ImageView<ImmutableImage>> {
        let image = File::open(img_path).unwrap();
        let decoder = Decoder::new(image);
        let mut reader = decoder.read_info().unwrap();
        let info = reader.info();
        let dimensions = ImageDimensions::Dim2d {
            width: info.width,
            height: info.height,
            array_layers: 1,
        };
        let mut image_data = Vec::new();
        image_data.resize((info.width * info.height * 4) as usize, 0);
        reader.next_frame(&mut image_data).unwrap();

        let image = ImmutableImage::from_iter(
            memory_allocator,
            image_data,
            dimensions,
            MipmapsCount::One,
            Format::R8G8B8A8_SRGB,
            command_buffers_builder,
        )
        .unwrap();
        ImageView::new_default(image).unwrap()
    }

    pub fn create_default_sampler(device: Arc<Device>) -> Arc<Sampler> {
        Sampler::new(
            device.clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                address_mode: [SamplerAddressMode::Repeat; 3],
                ..Default::default()
            },
        )
        .unwrap()
    }
}
