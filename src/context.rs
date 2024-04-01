use crate::{buffers, debug, pipeline};

use itertools::Itertools;
use log::{debug, info};
use std::sync::Arc;
use vulkano::buffer::allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo};
use vulkano::buffer::BufferUsage;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::device::physical::PhysicalDeviceType;
use vulkano::device::{
    Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
};
use vulkano::format::Format;

use vulkano::image::view::{ImageView, ImageViewCreateInfo};
use vulkano::image::ImageUsage;
use vulkano::instance::debug::{
    DebugUtilsMessageSeverity, DebugUtilsMessageType, DebugUtilsMessenger,
};
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo, InstanceExtensions};
use vulkano::memory::allocator::{MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::{Framebuffer, RenderPass};
use vulkano::swapchain::{Surface, Swapchain, SwapchainCreateFlags, SwapchainCreateInfo};

use vulkano::VulkanLibrary;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

pub struct RenderingContext {
    pub framebuffers: Vec<Arc<Framebuffer>>,
    pub pipeline: Arc<GraphicsPipeline>,
    pub swapchain: Arc<Swapchain>,
    pub swapchain_image_views: Vec<Arc<ImageView>>,
    pub gui_image_views: Vec<Arc<ImageView>>,
    pub render_pass: Arc<RenderPass>,
}

impl RenderingContext {
    pub fn new(
        vulkan_context: &VulkanContext,
        window_context: &mut WindowContext,
        allocators: &Allocators,
    ) -> Self {
        let surface_capabilities = vulkan_context
            .device
            .physical_device()
            .surface_capabilities(&vulkan_context.surface, Default::default())
            .unwrap();

        let swapchain_format = vulkan_context
            .device
            .clone()
            .physical_device()
            .surface_formats(&vulkan_context.surface.clone(), Default::default())
            .unwrap()[0]
            .0;

        let (swapchain, images) = Swapchain::new(
            vulkan_context.device.clone(),
            vulkan_context.surface.clone(),
            SwapchainCreateInfo {
                image_format: swapchain_format,
                image_view_formats: vec![swapchain_format, Format::R8G8B8A8_UNORM],
                // 2 images is needed for fullscreen capabilities
                min_image_count: surface_capabilities.min_image_count.max(2),
                image_extent: window_context.window.clone().inner_size().into(),
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha: surface_capabilities
                    .supported_composite_alpha
                    .into_iter()
                    .next()
                    .unwrap(),
                flags: SwapchainCreateFlags::MUTABLE_FORMAT,
                ..Default::default()
            },
        )
        .expect("Failed to create Swapchain and/or Images");

        let render_pass = vulkano::single_pass_renderpass!(
            vulkan_context.device.clone(),
            attachments: {
                color: {
                    format: swapchain.image_format(),
                    // Multisampling = more than 1
                    samples: 1,
                    // Clear attachment on load
                    load_op: Clear,
                    // Store output of draw in the image
                    store_op: Store,
                },
                depth_stencil: {
                    format: Format::D16_UNORM,
                    samples: 1,
                    load_op: Clear,
                    store_op: DontCare,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {depth_stencil},
            },
        )
        .unwrap();

        let pipeline =
            pipeline::create_pipeline(vulkan_context.device.clone(), render_pass.clone()).unwrap();

        let framebuffers = buffers::create_framebuffers(
            &images,
            &mut window_context.viewport,
            allocators.memory_allocator.clone(),
            render_pass.clone(),
        )
        .unwrap();

        let gui_image_views = images
            .iter()
            .map(|image| {
                ImageView::new(
                    image.clone(),
                    ImageViewCreateInfo {
                        format: Format::R8G8B8A8_UNORM,
                        ..ImageViewCreateInfo::from_image(image)
                    },
                )
            })
            .map(Result::unwrap)
            .collect_vec();

        let swapchain_image_views = images
            .into_iter()
            .map(ImageView::new_default)
            .map(Result::unwrap)
            .collect_vec();

        assert_eq!(gui_image_views.len(), swapchain_image_views.len());

        Self {
            swapchain,
            framebuffers,
            swapchain_image_views,
            gui_image_views,
            pipeline,
            render_pass,
        }
    }
}

pub struct Allocators {
    pub memory_allocator: Arc<StandardMemoryAllocator>,
    pub subbuffer_allocator: SubbufferAllocator,
    pub command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    pub descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
}

impl Allocators {
    pub fn new(device: Arc<Device>) -> Self {
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let subbuffer_allocator = SubbufferAllocator::new(
            memory_allocator.clone(),
            SubbufferAllocatorCreateInfo {
                buffer_usage: BufferUsage::UNIFORM_BUFFER,
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
        );

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        Self {
            memory_allocator,
            subbuffer_allocator,
            command_buffer_allocator,
            descriptor_set_allocator,
        }
    }
}

pub struct WindowContext {
    pub viewport: Viewport,
    pub window: Arc<Window>,
}

impl WindowContext {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        Self {
            window: Arc::new(
                WindowBuilder::new()
                    .build(event_loop)
                    .expect("Failed to create Window."),
            ),
            viewport: Viewport::default(),
        }
    }
}

pub struct VulkanContext {
    _debug_callback: DebugUtilsMessenger,
    pub instance: Arc<Instance>,
    pub queue: Arc<Queue>,
    pub device: Arc<Device>,
    pub surface: Arc<Surface>,
}

impl VulkanContext {
    pub fn new(window_context: &WindowContext, event_loop: &EventLoop<()>) -> Self {
        let library = VulkanLibrary::new().expect("Failed to create VulkanLibrary.");

        let required_layers = vec!["VK_LAYER_KHRONOS_validation".to_owned()];
        let available_layers = library
            .layer_properties()
            .unwrap()
            .map(|layer| layer.name().to_owned())
            .collect_vec();

        for required_layer in required_layers.iter() {
            if !available_layers.contains(required_layer) {
                panic!(
                    "This device does not support the required layer {}",
                    required_layer
                );
            }
        }

        let surface_required_extensions = Surface::required_extensions(event_loop).unwrap();
        let debug_required_extensions = InstanceExtensions {
            ext_debug_utils: true,
            ..InstanceExtensions::empty()
        };

        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_layers: required_layers,
                enabled_extensions: surface_required_extensions.union(&debug_required_extensions),
                ..Default::default()
            },
        )
        .expect("Failed to create Instance.");

        info!("Vulkan version: {}", instance.api_version());
        info!(
            "Enabled extensions: {:?}",
            instance
                .enabled_extensions()
                .into_iter()
                .filter(|(_, enabled)| *enabled)
                .map(|(extension, _)| extension)
                .format(", ")
        );

        let _debug_callback = debug::create_debug_callback(
            instance.clone(),
            DebugUtilsMessageSeverity::WARNING,
            DebugUtilsMessageType::GENERAL
                | DebugUtilsMessageType::VALIDATION
                | DebugUtilsMessageType::PERFORMANCE,
        )
        .unwrap();

        let surface = Surface::from_window(instance.clone(), window_context.window.clone())
            .expect("Failed to create Surface.");

        let required_device_extensions = DeviceExtensions {
            khr_swapchain: true,
            khr_swapchain_mutable_format: true, // For egui
            ..DeviceExtensions::empty()
        };

        let physical_devices = instance
            .enumerate_physical_devices()
            .unwrap()
            .collect_vec();

        for physical_device in physical_devices.iter() {
            debug!(
                "Available device {:?} of type {:?}",
                physical_device.properties().device_name,
                physical_device.properties().device_type,
            );
        }

        let (physical_device, queue_family_index) = physical_devices
            .into_iter()
            .filter(|device| {
                device
                    .supported_extensions()
                    .contains(&required_device_extensions)
            })
            .filter_map(|device| {
                device
                    .queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, queue)| {
                        // Ensure a queue exists which can perform graphics operations
                        queue.queue_flags.intersects(QueueFlags::GRAPHICS)
                            // And can perform them on the given surface
                            && device.surface_support(i as u32, &surface).unwrap_or(false)
                    })
                    .map(|i| (device, i as u32))
            })
            .max_by_key(|(device, _)| match device.properties().device_type {
                // DiscreteGpu gives the best score of 5
                PhysicalDeviceType::DiscreteGpu => 5,
                PhysicalDeviceType::IntegratedGpu => 4,
                PhysicalDeviceType::VirtualGpu => 3,
                PhysicalDeviceType::Cpu => 2,
                PhysicalDeviceType::Other => 1,
                _ => 0,
            })
            .expect("No suitable physical device found.");

        info!(
            "Using device {} of type {:?}",
            physical_device.properties().device_name,
            physical_device.properties().device_type
        );

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: required_device_extensions,
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .expect("Failed to create Device and/or Queues.");

        let queue = queues.next().unwrap();

        Self {
            _debug_callback,
            instance,
            queue,
            device,
            surface,
        }
    }
}
