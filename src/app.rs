use crate::{buffers, context, debug, model, scene, shaders, texture};
use cgmath::{Euler, Matrix, Matrix4, Point3, Quaternion, Rad, SquareMatrix, Vector3};
use color_eyre::Result;
use itertools::Itertools;
use std::time::Instant;
use vulkano::command_buffer::sys::CommandBufferBeginInfo;
use vulkano::command_buffer::{CommandBufferLevel, CommandBufferUsage};
use vulkano::command_buffer::{
    RecordingCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents,
};
use vulkano::descriptor_set::{DescriptorSet, WriteDescriptorSet};

use vulkano::pipeline::{Pipeline, PipelineBindPoint};

use vulkano::swapchain::{acquire_next_image, SwapchainCreateInfo, SwapchainPresentInfo};
use vulkano::sync::GpuFuture;
use vulkano::{sync, Validated, VulkanError};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

pub struct App {
    scene: scene::Scene,
    texture: texture::Texture,
    model: model::Model,
    vulkan_context: context::VulkanContext,
    rendering_context: context::RenderingContext,
    window_context: context::WindowContext,
    allocators: context::Allocators,
    // TODO decide what to do with this
    previous_frame_end: Option<Box<dyn GpuFuture>>,
}

impl App {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        color_eyre::install().unwrap();
        debug::set_up_logging();

        // TODO deferred rendering https://learnopengl.com/Advanced-Lighting/Deferred-Shading
        let mut window_context = context::WindowContext::new(event_loop);
        let vulkan_context = context::VulkanContext::new(&window_context, event_loop);
        let allocators = context::Allocators::new(vulkan_context.device.clone());
        let rendering_context =
            context::RenderingContext::new(&vulkan_context, &mut window_context, &allocators);

        let teapot = model::Model::load(
            "assets/models/teapot2.gltf",
            allocators.memory_allocator.clone(),
        )
        .expect("Could not load model.");

        let _cube = model::Model::load(
            "assets/models/cube.glb",
            allocators.memory_allocator.clone(),
        )
        .unwrap();

        let _model = &teapot;

        let mut texture_uploads = RecordingCommandBuffer::new(
            allocators.command_buffer_allocator.clone(),
            vulkan_context.queue.queue_family_index(),
            CommandBufferLevel::Primary,
            CommandBufferBeginInfo {
                usage: CommandBufferUsage::OneTimeSubmit,
                ..Default::default()
            },
        )
        .unwrap();

        let _ferris = texture::Texture::load(
            "assets/textures/ferris.png",
            allocators.memory_allocator.clone(),
            vulkan_context.device.clone(),
            &mut texture_uploads,
        )
        .unwrap();

        let wojak = texture::Texture::load(
            "assets/textures/wojak.jpg",
            allocators.memory_allocator.clone(),
            vulkan_context.device.clone(),
            &mut texture_uploads,
        )
        .unwrap();

        let _gmod = texture::Texture::load(
            "assets/textures/gmod.jpg",
            allocators.memory_allocator.clone(),
            vulkan_context.device.clone(),
            &mut texture_uploads,
        )
        .unwrap();

        // Submit uploading textures
        let texture_uploads_end = Some(
            texture_uploads
                .end()
                .unwrap()
                .execute(vulkan_context.queue.clone())
                .unwrap()
                .boxed(),
        );

        Self {
            window_context,
            rendering_context,
            vulkan_context,
            allocators,
            previous_frame_end: texture_uploads_end,
            texture: wojak,
            model: teapot,
            scene: scene::Scene { models: vec![] },
        }
    }

    pub fn run(mut self, event_loop: EventLoop<()>) {
        let mut frame_state = FrameState {
            start: Instant::now(),
            recreate_swapchain: false,
        };

        event_loop
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
                        // Purposefully ignore new window size to retain 16:9 aspect ratio = no stretching
                        event: WindowEvent::Resized(_new_size),
                        ..
                    } => {
                        frame_state.recreate_swapchain = true;
                    }
                    Event::WindowEvent {
                        event: WindowEvent::RedrawRequested,
                        ..
                    } => {
                        self.render(&mut frame_state);
                    }
                    Event::AboutToWait => self.window_context.window.request_redraw(),
                    _ => (),
                };
            })
            .unwrap();
    }

    fn render(&mut self, frame_state: &mut FrameState) {
        let current_window_extent: [u32; 2] = self.window_context.window.inner_size().into();
        if current_window_extent.contains(&0) {
            return;
        }

        // Clean up last frame's resources
        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        if frame_state.recreate_swapchain {
            self.resize_swapchain_and_framebuffers(current_window_extent)
                .unwrap();

            frame_state.recreate_swapchain = false;
        }

        let camera_uniform_subbuffer = {
            let aspect_ratio = self.rendering_context.swapchain.image_extent()[0] as f32
                / self.rendering_context.swapchain.image_extent()[1] as f32;

            let proj =
                cgmath::perspective(Rad(std::f32::consts::FRAC_PI_2), aspect_ratio, 0.01, 100.0);

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

            let subbuffer = self
                .allocators
                .subbuffer_allocator
                .allocate_sized()
                .unwrap();
            *subbuffer.write().unwrap() = uniform_data;

            subbuffer
        };

        let model_normal_uniform_subbuffer = {
            let model = self.model.model_matrix
                * Matrix4::from(Quaternion::from(Euler::new(
                    Rad(0.0),
                    Rad(frame_state.start.elapsed().as_millis() as f32 / 1000.0),
                    Rad(frame_state.start.elapsed().as_millis() as f32 / 2000.0),
                )));

            let uniform_data = shaders::vs::ModelUniform {
                model: model.into(),
                normal: model.invert().unwrap().transpose().into(),
            };

            let subbuffer = self
                .allocators
                .subbuffer_allocator
                .allocate_sized()
                .unwrap();
            *subbuffer.write().unwrap() = uniform_data;

            subbuffer
        };

        let lights_uniform_subbuffer = {
            const MAX_LIGHTS: usize = 10;
            let mut lights = [shaders::fs::Light::default(); MAX_LIGHTS];

            let elapsed = frame_state.start.elapsed().as_millis() as f32 / 1000.0;
            let radius = 7.0;
            let light_z = radius * elapsed.sin();
            let light_x = radius * elapsed.cos();

            lights[0] = shaders::fs::Light {
                position: [light_x, 0.5, light_z].into(),
                // position: [7.0, 0.7, 7.0].into(),
                color: [1.0, 1.0, 1.0],
                intensity: 1.0,
            };

            let lights_data = shaders::fs::LightsUniform { lights };

            let subbuffer = self
                .allocators
                .subbuffer_allocator
                .allocate_sized()
                .unwrap();
            *subbuffer.write().unwrap() = lights_data;

            subbuffer
        };

        let per_frame_descriptor_set = DescriptorSet::new(
            self.allocators.descriptor_set_allocator.clone(),
            self.rendering_context.pipeline.layout().set_layouts()[0].clone(),
            [
                WriteDescriptorSet::buffer(0, camera_uniform_subbuffer),
                WriteDescriptorSet::buffer(1, lights_uniform_subbuffer),
            ],
            [],
        )
        .unwrap();

        let per_primitive_descriptor_set = DescriptorSet::new(
            self.allocators.descriptor_set_allocator.clone(),
            self.rendering_context.pipeline.layout().set_layouts()[1].clone(),
            [
                WriteDescriptorSet::buffer(0, model_normal_uniform_subbuffer),
                WriteDescriptorSet::sampler(1, self.texture.sampler.clone()),
                WriteDescriptorSet::image_view(2, self.texture.image_view.clone()),
            ],
            [],
        )
        .unwrap();

        // Acquire next image to draw upon
        let (image_index, suboptimal, acquire_future) =
            match acquire_next_image(self.rendering_context.swapchain.clone(), None)
                .map_err(Validated::unwrap)
            {
                Ok(next) => next,
                Err(VulkanError::OutOfDate) => {
                    frame_state.recreate_swapchain = true;
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
            self.allocators.command_buffer_allocator.clone(),
            self.vulkan_context.queue.queue_family_index(),
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
                    clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into()), Some(1_f32.into())],
                    ..RenderPassBeginInfo::framebuffer(
                        self.rendering_context.framebuffers[image_index as usize].clone(),
                    )
                },
                SubpassBeginInfo {
                    contents: SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .unwrap()
            .set_viewport(
                0,
                [self.window_context.viewport.clone()].into_iter().collect(),
            )
            .unwrap()
            .bind_pipeline_graphics(self.rendering_context.pipeline.clone())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.rendering_context.pipeline.layout().clone(),
                0,
                per_frame_descriptor_set,
            )
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.rendering_context.pipeline.layout().clone(),
                1,
                per_primitive_descriptor_set,
            )
            .unwrap()
            .bind_vertex_buffers(0, self.model.meshes[0].primitives[0].vertex_buffer.clone())
            .unwrap()
            .bind_index_buffer(self.model.meshes[0].primitives[0].index_buffer.clone())
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
            .then_execute(self.vulkan_context.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                self.vulkan_context.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    self.rendering_context.swapchain.clone(),
                    image_index,
                ),
            )
            .then_signal_fence_and_flush();

        match future.map_err(Validated::unwrap) {
            Ok(future) => self.previous_frame_end = Some(future.boxed()),
            Err(VulkanError::OutOfDate) => {
                frame_state.recreate_swapchain = true;
                self.previous_frame_end =
                    Some(sync::now(self.vulkan_context.device.clone()).boxed());
            }
            Err(error) => {
                panic!("Failed to flush future: {error}");
            }
        };
    }

    fn resize_swapchain_and_framebuffers(&mut self, new_window_extent: [u32; 2]) -> Result<()> {
        let (new_swapchain, new_images) =
            self.rendering_context
                .swapchain
                .recreate(SwapchainCreateInfo {
                    // New size of window
                    image_extent: new_window_extent,
                    ..self.rendering_context.swapchain.create_info()
                })?;

        self.rendering_context.swapchain = new_swapchain;

        self.rendering_context.framebuffers = buffers::create_framebuffers(
            &new_images,
            &mut self.window_context.viewport,
            self.allocators.memory_allocator.clone(),
            self.rendering_context.render_pass.clone(),
        )?;

        Ok(())
    }
}

struct FrameState {
    start: Instant,
    recreate_swapchain: bool,
}
