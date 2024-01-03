

use color_eyre::eyre::Result;
use itertools::Itertools;
use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};

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

pub fn create_framebuffers(
    images: &[Arc<Image>],
    viewport: &mut Viewport,
    memory_allocator: Arc<StandardMemoryAllocator>,
    render_pass: Arc<RenderPass>,
) -> Result<Vec<Arc<Framebuffer>>> {
    let extent = images[0].extent();
    // Make sure new framebuffers are the same size as the window
    viewport.extent = [extent[0] as f32, extent[1] as f32];

    let depth_buffer = create_depth_buffer(extent, memory_allocator)?;

    Ok(images
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
        .collect_vec())
}
