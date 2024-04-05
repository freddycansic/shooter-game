use crate::{buffers, camera, colors, context, debug, input, model, scene, shaders, texture};
use cgmath::{Matrix4, Point3, Vector3, Vector4};
use color_eyre::Result;
use std::time::Instant;
use vulkano::command_buffer::sys::CommandBufferBeginInfo;
use vulkano::command_buffer::{CommandBufferLevel, CommandBufferUsage};
use vulkano::command_buffer::{
    RecordingCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents,
};
use vulkano::descriptor_set::{DescriptorSet, WriteDescriptorSet};
use winit::keyboard::KeyCode;

use vulkano::pipeline::{Pipeline, PipelineBindPoint};

use vulkano::padded::Padded;
use vulkano::swapchain::{acquire_next_image, SwapchainCreateInfo, SwapchainPresentInfo};
use vulkano::sync::GpuFuture;
use vulkano::{sync, Validated, VulkanError};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use egui_winit_vulkano::{Gui, GuiConfig};

use input::Input;
use scene::Scene;

pub struct App {
    input: input::Input,
    scene: scene::Scene,
    texture: texture::Texture,
    vulkan_context: context::VulkanContext,
    rendering_context: context::RenderingContext,
    window_context: context::WindowContext,
    allocators: context::Allocators,
    gui: Gui,
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

        // TODO move models, textures to resource manager / scene
        let load_model = |path| {
            model::Model::load(path, allocators.memory_allocator.clone())
                .expect("Could not load model.")
        };

        let teapot = load_model("assets/models/teapot.glb");
        let backdrop = load_model("assets/models/backdrop.glb");

        let mut scene = Scene::new(camera::Camera::new(
            Point3::new(5.0, 2.0, 5.0),
            Point3::new(0.0, 0.0, 0.0),
        ));
        scene.models = vec![teapot.into(), backdrop.into()];

        let _cube = model::Model::load(
            "assets/models/cube.glb",
            allocators.memory_allocator.clone(),
        )
        .unwrap();

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

        let mut load_texture = |path| {
            texture::Texture::load(
                path,
                allocators.memory_allocator.clone(),
                vulkan_context.device.clone(),
                &mut texture_uploads,
            )
            .unwrap()
        };

        let _ferris = load_texture("assets/textures/ferris.png");
        let wojak = load_texture("assets/textures/wojak.jpg");
        let _gmod = load_texture("assets/textures/gmod.jpg");
        let _white = load_texture("assets/textures/white.jpg");

        let texture = wojak;

        // Submit uploading textures
        let texture_uploads_end = Some(
            texture_uploads
                .end()
                .unwrap()
                .execute(vulkan_context.queue.clone())
                .unwrap()
                .boxed(),
        );

        let gui = Gui::new(
            event_loop,
            vulkan_context.surface.clone(),
            vulkan_context.queue.clone(),
            rendering_context.gui_image_views[0].format(),
            GuiConfig {
                is_overlay: true,
                ..Default::default()
            },
        );

        let input = Input::new();

        Self {
            window_context,
            rendering_context,
            vulkan_context,
            allocators,
            previous_frame_end: texture_uploads_end,
            scene,
            gui,
            texture,
            input,
        }
    }

    pub fn run(mut self, event_loop: EventLoop<()>) {
        let mut frame_state = FrameState {
            start: Instant::now(),
            recreate_swapchain: false,
            frame_count: 0,
        };

        event_loop
            .run(move |event, event_loop_window_target| {
                println!("{:?}", event);
                event_loop_window_target.set_control_flow(ControlFlow::Poll);

                match event {
                    Event::WindowEvent {
                        event: window_event,
                        window_id,
                    } if window_id == self.window_context.window.id() => {
                        let pass_events_to_game = !self.gui.update(&window_event);

                        match window_event {
                            WindowEvent::CloseRequested => event_loop_window_target.exit(),
                            WindowEvent::KeyboardInput { event, .. } if pass_events_to_game => {
                                self.input.process_key_event(event)
                            }
                            WindowEvent::Resized(_new_size) => {
                                frame_state.recreate_swapchain = true
                            }
                            WindowEvent::ScaleFactorChanged { .. } => {
                                frame_state.recreate_swapchain = true
                            }
                            WindowEvent::RedrawRequested => {
                                if self.input.key_pressed(KeyCode::Escape) {
                                    event_loop_window_target.exit();
                                }

                                println!("{}", frame_state.frame_count);

                                self.gui.immediate_ui(|gui| {
                                    let ctx = gui.context();

                                    egui::TopBottomPanel::top("top_panel").show(&ctx, |ui| {
                                        ui.menu_button("File", |ui| {
                                            if ui.add(egui::Button::new("Import model")).clicked() {
                                                ui.close_menu();
                                            }
                                        });
                                    });

                                    egui::SidePanel::left("left_panel").show(&ctx, |ui| {
                                        ui.heading("My egui Application");
                                        ui.label("Hello world");
                                    });
                                });
                                self.render(&mut frame_state);
                                frame_state.frame_count = (frame_state.frame_count + 1) % u128::MAX;
                                self.input.reset_just_released();
                            }
                            _ => (),
                        }
                    }
                    Event::AboutToWait => self.window_context.window.request_redraw(),
                    _ => (),
                }
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

        // Set current aspect ratio
        self.scene.camera.set_aspect_ratio(
            self.rendering_context.swapchain.image_extent()[0] as f32
                / self.rendering_context.swapchain.image_extent()[1] as f32,
        );

        let camera_uniform_subbuffer = self
            .scene
            .camera
            .create_subbuffer(&self.allocators.subbuffer_allocator);

        let lights_uniform_subbuffer = {
            const MAX_LIGHTS: usize = 10;
            let mut lights: [Padded<shaders::fs::Light, 12>; 10] =
                [shaders::fs::Light::default().into(); MAX_LIGHTS];

            let elapsed = frame_state.start.elapsed().as_millis() as f32 / 1000.0;
            let radius = 4.0;
            let light_z = radius * elapsed.sin();
            let light_x = radius * elapsed.cos();
            lights[0].position = Vector4::new(light_x, 0.5, light_z, 1.0);

            let color = colors::shift_hue_from_named(palette::named::RED, elapsed * 100.0);
            lights[0].color = colors::to_vector4(color);

            let lights_data = shaders::fs::LightsUniform { lights };

            buffers::create_subbuffer(&self.allocators.subbuffer_allocator, lights_data)
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
            .unwrap();

        // Teapot
        self.scene.models[0].render(
            &mut builder,
            &self.allocators,
            self.rendering_context.pipeline.clone(),
            &self.texture,
        );

        // Backdrop
        self.scene.models[1].transform =
            Matrix4::from_translation(Vector3::new(0.0, -2.0, 0.0)) * Matrix4::from_scale(10.0);
        self.scene.models[1].render(
            &mut builder,
            &self.allocators,
            self.rendering_context.pipeline.clone(),
            &self.texture,
        );

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
            .then_signal_fence_and_flush()
            .unwrap();

        let gui_future = self.gui.draw_on_image(
            future,
            self.rendering_context.gui_image_views[image_index as usize].clone(),
        );

        let future = gui_future
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
    frame_count: u128,
}
