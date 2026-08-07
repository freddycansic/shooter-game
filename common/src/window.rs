use common_macros::Event;

#[derive(Event)]
pub struct WindowResized {
    pub new_width: u32,
    pub new_height: u32,
}

#[derive(Event)]
pub struct WinitWindowEvent(pub winit::event::WindowEvent);

#[derive(Event)]
pub struct WinitDeviceEvent(pub winit::event::DeviceEvent);

#[derive(Event)]
pub struct CaptureCursor(pub bool);

#[derive(Event)]
pub struct CenterCursor;
