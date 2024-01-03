use crate::buffers;
use color_eyre::Result;




use std::sync::Arc;
use vulkano::buffer::{BufferUsage};
use vulkano::command_buffer::{CopyBufferToImageInfo, RecordingCommandBuffer};
use vulkano::device::Device;
use vulkano::format::Format;
use vulkano::image::sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo};
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};


pub struct Texture {
    pub image_view: Arc<ImageView>,
    pub sampler: Arc<Sampler>,
}

impl Texture {
    pub fn load(
        path: &str,
        memory_allocator: Arc<StandardMemoryAllocator>,
        device: Arc<Device>,
        command_buffer: &mut RecordingCommandBuffer,
    ) -> Result<Self> {
        let image_view = Self::create_image_view(path, memory_allocator, command_buffer)?;

        let sampler = Sampler::new(
            device,
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                address_mode: [SamplerAddressMode::Repeat; 3],
                ..Default::default()
            },
        )?;

        Ok(Self {
            image_view,
            sampler,
        })
    }

    fn create_image_view(
        path: &str,
        memory_allocator: Arc<StandardMemoryAllocator>,
        command_buffer: &mut RecordingCommandBuffer,
    ) -> Result<Arc<ImageView>> {
        let texture_file = image::open(path)?;

        let extent = [texture_file.width(), texture_file.height(), 1];

        // copy texture into generic buffer
        let image_buffer = buffers::create_mapped_buffer_from_iter(
            memory_allocator.clone(),
            BufferUsage::TRANSFER_SRC,
            MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            texture_file.into_rgba8().into_raw(),
        )?;

        let image = Image::new(
            memory_allocator,
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                // https://www.reddit.com/r/vulkan/comments/4w0w8o/why_doesnt_vulkan_support_24bit_image_formats/
                format: Format::R8G8B8A8_SRGB,
                extent,
                usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap();

        // copy texture from generic buffer to image
        command_buffer
            .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
                image_buffer,
                image.clone(),
            ))
            .unwrap();

        Ok(ImageView::new_default(image)?)
    }
}
