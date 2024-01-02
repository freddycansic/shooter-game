use crate::{buffers, debug, model, shaders, texture, vertex};
use cgmath::{Euler, Matrix, Matrix4, Point3, Quaternion, Rad, SquareMatrix, Vector3};
use itertools::Itertools;
use log::info;
use std::sync::Arc;
use std::time::Instant;
use vulkano::buffer::allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo};
use vulkano::buffer::BufferUsage;
use vulkano::command_buffer::sys::CommandBufferBeginInfo;
use vulkano::command_buffer::{
    allocator::StandardCommandBufferAllocator, RecordingCommandBuffer, RenderPassBeginInfo,
    SubpassBeginInfo, SubpassContents,
};
use vulkano::command_buffer::{CommandBufferLevel, CommandBufferUsage};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{DescriptorSet, WriteDescriptorSet};
use vulkano::device::physical::PhysicalDeviceType;
use vulkano::device::{
    Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageUsage};
use vulkano::instance::debug::{
    DebugUtilsMessageSeverity, DebugUtilsMessageType, DebugUtilsMessenger,
};
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo, InstanceExtensions};
use vulkano::library::VulkanLibrary;
use vulkano::memory::allocator::{MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::graphics::color_blend::{
    AttachmentBlend, ColorBlendAttachmentState, ColorBlendState,
};
use vulkano::pipeline::graphics::depth_stencil::{DepthState, DepthStencilState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::VertexDefinition;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{
    DynamicState, GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout,
    PipelineShaderStageCreateInfo,
};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::swapchain::{
    acquire_next_image, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo,
};
use vulkano::sync::GpuFuture;
use vulkano::{sync, Validated, VulkanError};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

// God class
pub struct App {
    _debug_callback: DebugUtilsMessenger,
    instance: Arc<Instance>,
    event_loop: EventLoop<()>,
    queue: Arc<Queue>,
    device: Arc<Device>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    framebuffers: Vec<Arc<Framebuffer>>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    pipeline: Arc<GraphicsPipeline>,
    swapchain: Arc<Swapchain>,
    images: Vec<Arc<Image>>,
    window: Arc<Window>,
    uniform_buffer_allocator: SubbufferAllocator,
    memory_allocator: Arc<StandardMemoryAllocator>,
    viewport: Viewport,
    render_pass: Arc<RenderPass>,
    // TODO TEMP
    texture: texture::Texture,
    model: model::Model,
}

impl App {
    pub fn new() -> Self {
        color_eyre::install().unwrap();
        debug::set_up_logging();

        // TODO deferred rendering https://learnopengl.com/Advanced-Lighting/Deferred-Shading
        let library = VulkanLibrary::new().expect("Failed to create VulkanLibrary.");

        let required_layers = vec!["VK_LAYER_KHRONOS_validation".to_owned()];
        let available_layers = library
            .layer_properties()
            .unwrap()
            .into_iter()
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

        let event_loop = EventLoop::new().expect("Failed to create EventLoop");

        let surface_required_extensions = Surface::required_extensions(&event_loop).unwrap();
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

        let window = Arc::new(
            WindowBuilder::new()
                .build(&event_loop)
                .expect("Failed to create Window."),
        );

        let surface = Surface::from_window(instance.clone(), window.clone())
            .expect("Failed to create Surface.");

        let required_device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .unwrap()
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

        let surface_capabilities = device
            .physical_device()
            .surface_capabilities(&surface, Default::default())
            .unwrap();

        let (mut swapchain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                image_format: device
                    .clone()
                    .physical_device()
                    .surface_formats(&surface.clone(), Default::default())
                    .unwrap()[0]
                    .0,
                // 2 images is needed for fullscreen capabilities
                min_image_count: surface_capabilities.min_image_count.max(2),
                image_extent: window.clone().inner_size().into(),
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha: surface_capabilities
                    .supported_composite_alpha
                    .into_iter()
                    .next()
                    .unwrap(),
                ..Default::default()
            },
        )
        .expect("Failed to create Swapchain and/or Images");

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let teapot = model::Model::load("assets/models/teapot2.gltf", memory_allocator.clone())
            .expect("Could not load model.");

        let cube = model::Model::load("assets/models/cube.glb", memory_allocator.clone()).unwrap();

        let model = &teapot;

        let uniform_buffer_allocator = SubbufferAllocator::new(
            memory_allocator.clone(),
            SubbufferAllocatorCreateInfo {
                buffer_usage: BufferUsage::UNIFORM_BUFFER,
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
        );

        let render_pass = vulkano::single_pass_renderpass!(
            device.clone(),
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

        let pipeline = {
            let vs = shaders::vs::load(device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap();
            let fs = shaders::fs::load(device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap();

            let vertex_input_state =
                <vertex::Vertex as vulkano::pipeline::graphics::vertex_input::Vertex>::per_vertex()
                    .definition(&vs.info().input_interface)
                    .unwrap();

            let stages = [
                PipelineShaderStageCreateInfo::new(vs),
                PipelineShaderStageCreateInfo::new(fs),
            ];

            let layout = PipelineLayout::new(
                device.clone(),
                PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                    .into_pipeline_layout_create_info(device.clone())
                    .unwrap(),
            )
            .unwrap();

            let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

            GraphicsPipeline::new(
                device.clone(),
                None,
                GraphicsPipelineCreateInfo {
                    stages: stages.into_iter().collect(),
                    // How vertex data is read from the vertex buffers into the vertex shader
                    vertex_input_state: Some(vertex_input_state),
                    // How vertices are arranged into primitive shapes (triangle)
                    input_assembly_state: Some(InputAssemblyState::default()),
                    viewport_state: Some(ViewportState::default()),
                    rasterization_state: Some(RasterizationState::default()),
                    multisample_state: Some(MultisampleState::default()),
                    color_blend_state: Some(ColorBlendState::with_attachment_states(
                        subpass.num_color_attachments(),
                        ColorBlendAttachmentState {
                            blend: Some(AttachmentBlend::alpha()),
                            ..Default::default()
                        },
                    )),
                    // By making the viewport dynamic, we can simply recreate it when the window is resized instead of having to recreate the entire pipeline
                    dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                    subpass: Some(subpass.into()),
                    depth_stencil_state: Some(DepthStencilState {
                        depth: Some(DepthState::simple()),
                        ..Default::default()
                    }),
                    ..GraphicsPipelineCreateInfo::layout(layout)
                },
            )
            .unwrap()
        };

        let mut viewport = Viewport::default();

        let framebuffers = Self::create_framebuffers(
            &images,
            render_pass.clone(),
            &mut viewport,
            memory_allocator.clone(),
        );

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let mut uploads = RecordingCommandBuffer::new(
            command_buffer_allocator.clone(),
            queue.queue_family_index(),
            CommandBufferLevel::Primary,
            CommandBufferBeginInfo {
                usage: CommandBufferUsage::OneTimeSubmit,
                ..Default::default()
            },
        )
        .unwrap();

        let ferris = texture::Texture::load(
            "assets/textures/ferris.png",
            memory_allocator.clone(),
            device.clone(),
            &mut uploads,
        )
        .unwrap();

        let wojak = texture::Texture::load(
            "assets/textures/wojak.jpg",
            memory_allocator.clone(),
            device.clone(),
            &mut uploads,
        )
        .unwrap();

        let gmod = texture::Texture::load(
            "assets/textures/gmod.jpg",
            memory_allocator.clone(),
            device.clone(),
            &mut uploads,
        )
        .unwrap();

        // Submit uploading textures
        let previous_frame_end = Some(
            uploads
                .end()
                .unwrap()
                .execute(queue.clone())
                .unwrap()
                .boxed(),
        );

        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        Self {
            instance,
            event_loop,
            _debug_callback,
            queue,
            descriptor_set_allocator,
            texture: wojak,
            model: teapot,
            previous_frame_end,
            command_buffer_allocator,
            framebuffers,
            pipeline,
            swapchain,
            images,
            window,
            uniform_buffer_allocator,
            memory_allocator,
            render_pass,
            viewport,
            device,
        }
    }

    pub fn run(mut self) {
        let mut recreate_swapchain = false;
        let start = Instant::now();

        self.event_loop
            .run(move |event, event_loop_window_target| {
                event_loop_window_target.set_control_flow(ControlFlow::Poll);

                match event {
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } => {
                        event_loop_window_target.exit();
                    }
                    Event::WindowEvent {
                        event: WindowEvent::Resized(_),
                        ..
                    } => {
                        recreate_swapchain = true;
                    }
                    Event::WindowEvent {
                        event: WindowEvent::RedrawRequested,
                        ..
                    } => {
                        let image_extent: [u32; 2] = self.window.inner_size().into();
                        if image_extent.contains(&0) {
                            return;
                        }

                        // Clean up last frame's resources
                        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

                        if recreate_swapchain {
                            let (new_swapchain, new_images) = self
                                .swapchain
                                .recreate(SwapchainCreateInfo {
                                    image_extent,
                                    ..self.swapchain.create_info()
                                })
                                .expect("Failed to recreate swapchain.");

                            self.swapchain = new_swapchain;

                            // Recreate framebuffers which are pointing to old swapchain
                            self.framebuffers = Self::create_framebuffers(
                                &new_images,
                                self.render_pass.clone(),
                                &mut self.viewport,
                                self.memory_allocator.clone(),
                            );

                            recreate_swapchain = false;
                        }

                        let camera_uniform_subbuffer = {
                            let aspect_ratio = self.swapchain.image_extent()[0] as f32
                                / self.swapchain.image_extent()[1] as f32;

                            let proj = cgmath::perspective(
                                Rad(std::f32::consts::FRAC_PI_2),
                                aspect_ratio,
                                0.01,
                                100.0,
                            );

                            let camera_position = Point3::new(5.0, 2.0, 5.0);

                            let view = Matrix4::look_at_rh(
                                camera_position,
                                Point3::new(0.0, 0.0, 0.0),
                                Vector3::new(0.0, -1.0, 0.0),
                            );

                            let uniform_data = shaders::vs::CameraUniform {
                                view: view.into(),
                                projection: proj.into(),
                                camera_position: camera_position.into(),
                            };

                            let subbuffer = self.uniform_buffer_allocator.allocate_sized().unwrap();
                            *subbuffer.write().unwrap() = uniform_data;

                            subbuffer
                        };

                        let model_normal_uniform_subbuffer = {
                            let model = self.model.model_matrix.clone()
                                * Matrix4::from(Quaternion::from(Euler::new(
                                    Rad(0.0),
                                    Rad(start.elapsed().as_millis() as f32 / 1000.0),
                                    Rad(start.elapsed().as_millis() as f32 / 2000.0),
                                )));

                            let uniform_data = shaders::vs::ModelUniform {
                                model: model.into(),
                                normal: model.invert().unwrap().transpose().into(),
                            };

                            let subbuffer = self.uniform_buffer_allocator.allocate_sized().unwrap();
                            *subbuffer.write().unwrap() = uniform_data;

                            subbuffer
                        };

                        let lights_uniform_subbuffer = {
                            const MAX_LIGHTS: usize = 10;
                            let mut lights = [shaders::fs::Light::default(); MAX_LIGHTS];

                            let elapsed = start.elapsed().as_millis() as f32 / 1000.0;
                            let radius = 7.0;
                            let light_z = radius * elapsed.sin();
                            let light_x = radius * elapsed.cos();

                            lights[0] = shaders::fs::Light {
                                position: [light_x, 0.5, light_z].into(),
                                // position: [7.0, 0.7, 7.0].into(),
                                color: [1.0, 1.0, 1.0].into(),
                                intensity: 1.0,
                            };

                            let lights_data = shaders::fs::LightsUniform { lights };

                            let subbuffer = self.uniform_buffer_allocator.allocate_sized().unwrap();
                            *subbuffer.write().unwrap() = lights_data;

                            subbuffer
                        };

                        let layout = &self.pipeline.layout().set_layouts()[0];

                        let set = DescriptorSet::new(
                            self.descriptor_set_allocator.clone(),
                            layout.clone(),
                            [
                                WriteDescriptorSet::buffer(0, camera_uniform_subbuffer),
                                WriteDescriptorSet::buffer(1, model_normal_uniform_subbuffer),
                                WriteDescriptorSet::buffer(2, lights_uniform_subbuffer),
                                WriteDescriptorSet::sampler(3, self.texture.sampler.clone()),
                                WriteDescriptorSet::image_view(4, self.texture.image_view.clone()),
                            ],
                            [],
                        )
                        .unwrap();

                        // Acquire next image to draw upon
                        let (image_index, suboptimal, acquire_future) =
                            match acquire_next_image(self.swapchain.clone(), None)
                                .map_err(Validated::unwrap)
                            {
                                Ok(next) => next,
                                Err(VulkanError::OutOfDate) => {
                                    recreate_swapchain = true;
                                    return;
                                }
                                Err(error) => panic!("Failed to acquire next image: {error}"),
                            };

                        // Drawing on suboptimal images can produce graphical errors
                        if suboptimal {
                            return;
                        }

                        // Holds list of commands to be executed
                        let mut builder = RecordingCommandBuffer::new(
                            self.command_buffer_allocator.clone(),
                            self.queue.queue_family_index(),
                            CommandBufferLevel::Primary,
                            CommandBufferBeginInfo {
                                usage: CommandBufferUsage::OneTimeSubmit,
                                ..Default::default()
                            },
                        )
                        .unwrap();

                        builder
                            .begin_render_pass(
                                RenderPassBeginInfo {
                                    // Clear values for each attachment
                                    clear_values: vec![
                                        Some([0.0, 0.0, 1.0, 1.0].into()),
                                        Some(1_f32.into()),
                                    ],
                                    ..RenderPassBeginInfo::framebuffer(
                                        self.framebuffers[image_index as usize].clone(),
                                    )
                                },
                                SubpassBeginInfo {
                                    contents: SubpassContents::Inline,
                                    ..Default::default()
                                },
                            )
                            .unwrap()
                            .set_viewport(0, [self.viewport.clone()].into_iter().collect())
                            .unwrap()
                            .bind_pipeline_graphics(self.pipeline.clone())
                            .unwrap()
                            .bind_descriptor_sets(
                                PipelineBindPoint::Graphics,
                                self.pipeline.layout().clone(),
                                0,
                                set,
                            )
                            .unwrap()
                            .bind_vertex_buffers(
                                0,
                                self.model.meshes[0].primitives[0].vertex_buffer.clone(),
                            )
                            .unwrap()
                            .bind_index_buffer(
                                self.model.meshes[0].primitives[0].index_buffer.clone(),
                            )
                            .unwrap();

                        unsafe {
                            builder
                                .draw_indexed(
                                    self.model.meshes[0].primitives[0].index_buffer.len() as u32,
                                    1,
                                    0,
                                    0,
                                    0,
                                )
                                .unwrap();
                        }

                        builder.end_render_pass(Default::default()).unwrap();

                        // Finish recording commands
                        let command_buffer = builder.end().unwrap();

                        let future = self
                            .previous_frame_end
                            .take()
                            .unwrap()
                            .join(acquire_future)
                            .then_execute(self.queue.clone(), command_buffer)
                            .unwrap()
                            .then_swapchain_present(
                                self.queue.clone(),
                                SwapchainPresentInfo::swapchain_image_index(
                                    self.swapchain.clone(),
                                    image_index,
                                ),
                            )
                            .then_signal_fence_and_flush();

                        match future.map_err(Validated::unwrap) {
                            Ok(future) => self.previous_frame_end = Some(future.boxed()),
                            Err(VulkanError::OutOfDate) => {
                                recreate_swapchain = true;
                                self.previous_frame_end =
                                    Some(sync::now(self.device.clone()).boxed());
                            }
                            Err(error) => {
                                panic!("Failed to flush future: {error}");
                            }
                        };
                    }
                    Event::AboutToWait => self.window.request_redraw(),
                    _ => (),
                };
            })
            .unwrap();
    }

    fn create_framebuffers(
        images: &[Arc<Image>],
        render_pass: Arc<RenderPass>,
        viewport: &mut Viewport,
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> Vec<Arc<Framebuffer>> {
        let extent = images[0].extent();
        // Change size of viewport
        viewport.extent = [extent[0] as f32, extent[1] as f32];

        let depth_buffer = buffers::create_depth_buffer(extent, memory_allocator.clone()).unwrap();

        images
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
            .collect::<Vec<_>>()
    }
}
