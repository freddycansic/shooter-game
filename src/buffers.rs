use color_eyre::eyre::Result;
use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};

pub fn create_depth_buffer(
    extent: [u32; 3],
    memory_allocator: Arc<StandardMemoryAllocator>,
) -> Result<Arc<ImageView>> {
    let image_view = ImageView::new_default(Image::new(
        memory_allocator,
        ImageCreateInfo {
            image_type: ImageType::Dim2d,
            format: Format::D16_UNORM,
            extent,
            usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
            ..Default::default()
        },
        AllocationCreateInfo::default(),
    )?)?;

    Ok(image_view)
}

pub fn create_mapped_buffer_from_iter<T, I>(
    memory_allocator: Arc<StandardMemoryAllocator>,
    usage: BufferUsage,
    memory_type_filter: MemoryTypeFilter,
    iter: I,
) -> Result<Subbuffer<[T]>>
where
    T: BufferContents,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
{
    Ok(Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter,
            ..Default::default()
        },
        iter,
    )?)
}
