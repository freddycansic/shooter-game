use crate::ecs::subsystem::Subsystem;
use crate::ecs::system::System;
use crate::ecs::system_parameters::application_context::ApplicationContext;
use crate::ecs::system_parameters::event::{EventReader, EventWriter};
use crate::ecs::system_parameters::res::ResMut;
use crate::engine::scheduler::{Stage, SystemOrder};
use crate::runtime::{RuntimeContext, WinitWindowEvent};
use common::engine::renderer::{Viewport, ViewportChanged};
use common::engine::scheduler::Scheduler;
use common::world::World;
use common_macros::Resource;
use egui_glium::egui_winit::egui;
use egui_glium::egui_winit::egui::{Align, Button, ViewportId};
use egui_glium::EguiGlium;
use log::info;

// TODO move gui state into its own resource
#[derive(Resource)]
pub struct GuiState {
    pub render_lights: bool,
    pub debug_cube_index: usize,
    pub debug_cube_opacity: f32,
    pub render_debug_mouse_rays: bool,
}

impl Default for GuiState {
    fn default() -> Self {
        GuiState {
            render_lights: true,
            debug_cube_index: 0,
            debug_cube_opacity: 0.5,
            render_debug_mouse_rays: false,
        }
    }
}

#[derive(Resource)]
pub struct Gui(pub EguiGlium);

pub struct GuiSubsystem;

impl Subsystem for GuiSubsystem {
    fn register_resources(&self, world: &mut World, context: Option<&RuntimeContext>) {
        world.register_resource(Gui(EguiGlium::new(
            ViewportId::ROOT,
            context.unwrap().display,
            context.unwrap().window,
            context.unwrap().event_loop,
        )));

        world.register_resource(GuiState::default());
    }

    fn register_systems(&self, scheduler: &mut Scheduler) {
        scheduler.register_triggered::<WinitWindowEvent, _, _>(GuiSubsystem::winit_window_event, Stage::Pre);
    }
}

impl GuiSubsystem {
    fn winit_window_event(
        mut gui: ResMut<Gui>,
        commands: ApplicationContext,
        mut winit_window_event: EventReader<WinitWindowEvent>,
    ) {
        for event in winit_window_event.read() {
            let gui_event_response = gui.0.on_event(commands.window(), &event.0);

            if gui_event_response.repaint {
                commands.window().request_redraw();
            }
        }
    }
}
