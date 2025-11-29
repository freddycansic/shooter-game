use log::warn;
use nalgebra::Vector2;
use winit::dpi::PhysicalPosition;
use winit::event::{DeviceEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{KeyCode, NativeKeyCode, PhysicalKey},
};

const NUM_KEYS: usize = 194;
const NUM_MOUSE_BUTTONS: usize = 6;

pub struct Input {
    key_states: [KeyState; NUM_KEYS],
    mouse_button_states: [KeyState; NUM_MOUSE_BUTTONS],
    mouse_position: Option<Vector2<f64>>,
    window_offset: Vector2<f32>,
    device_offset: Vector2<f32>,
    mouse_wheel_offset: f32,
    mouse_on_window: bool,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum KeyState {
    Released,
    Pressed,
    Repeat,
    JustReleased,
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

impl Input {
    pub fn new() -> Self {
        Self {
            key_states: [KeyState::Released; NUM_KEYS],
            mouse_button_states: [KeyState::Released; NUM_MOUSE_BUTTONS],
            mouse_position: None,
            window_offset: Vector2::zeros(),
            device_offset: Vector2::zeros(),
            mouse_wheel_offset: 0.0,
            mouse_on_window: false,
        }
    }

    pub fn key_pressed(&self, key_code: KeyCode) -> bool {
        self.key_states[key_code as usize] == KeyState::Pressed
    }

    pub fn key_released(&self, key_code: KeyCode) -> bool {
        self.key_states[key_code as usize] == KeyState::Released
    }

    pub fn key_down(&self, key_code: KeyCode) -> bool {
        let state = self.key_states[key_code as usize];
        state == KeyState::Pressed || state == KeyState::Repeat
    }

    pub fn key_just_released(&self, key_code: KeyCode) -> bool {
        self.key_states[key_code as usize] == KeyState::JustReleased
    }

    pub fn mouse_button_pressed(&self, mouse_button: MouseButton) -> bool {
        self.mouse_button_states[Self::mouse_button_to_index(mouse_button)] == KeyState::Pressed
    }

    pub fn mouse_button_released(&self, mouse_button: MouseButton) -> bool {
        self.mouse_button_states[Self::mouse_button_to_index(mouse_button)] == KeyState::Released
    }

    pub fn mouse_button_down(&self, mouse_button: MouseButton) -> bool {
        let state = self.mouse_button_states[Self::mouse_button_to_index(mouse_button)];
        state == KeyState::Pressed || state == KeyState::Repeat
    }

    pub fn mouse_button_just_released(&self, mouse_button: MouseButton) -> bool {
        self.mouse_button_states[Self::mouse_button_to_index(mouse_button)]
            == KeyState::JustReleased
    }

    pub fn window_offset(&self) -> Vector2<f32> {
        self.window_offset
    }

    pub fn device_offset(&self) -> Vector2<f32> {
        self.device_offset
    }

    pub fn mouse_wheel_offset(&self) -> f32 {
        self.mouse_wheel_offset
    }

    pub fn mouse_position(&self) -> Option<Vector2<f64>> {
        self.mouse_position
    }

    pub fn mouse_on_window(&self) -> bool {
        self.mouse_on_window
    }

    pub fn reset_internal_state(&mut self) {
        for key_state in self.key_states.iter_mut() {
            if *key_state == KeyState::JustReleased {
                *key_state = KeyState::Released;
            }
        }

        for mouse_button_state in self.mouse_button_states.iter_mut() {
            if *mouse_button_state == KeyState::JustReleased {
                *mouse_button_state = KeyState::Released;
            }
        }

        self.window_offset = Vector2::zeros();
        self.device_offset = Vector2::zeros();
        self.mouse_wheel_offset = 0.0;
    }

    pub fn process_window_event(&mut self, window_event: &WindowEvent) {
        match &window_event {
            WindowEvent::KeyboardInput { event, .. } => {
                self.process_key_event(event.clone());
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.process_mouse_moved_window_event(*position);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.process_mouse_button_event(*button, *state);
            }
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::LineDelta(_, y_offset),
                ..
            } => {
                self.process_mouse_wheel_event(*y_offset);
            }
            WindowEvent::CursorEntered { .. } => {
                self.mouse_on_window = true;
            }
            WindowEvent::CursorLeft { .. } => {
                self.mouse_on_window = false;
            }
            _ => (),
        };
    }

    pub fn process_device_event(&mut self, device_event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta, .. } = device_event {
            self.process_mouse_moved_device_event(delta);
        }
    }

    fn process_key_event(&mut self, key_event: KeyEvent) {
        match key_event.physical_key {
            PhysicalKey::Code(key_code) => {
                Self::update_key_state(&mut self.key_states, key_code as usize, key_event.state);
            }
            PhysicalKey::Unidentified(native_key_code) => {
                let (platform, code) = match native_key_code {
                    NativeKeyCode::Windows(code) => ("Windows", code as u32),
                    NativeKeyCode::MacOS(code) => ("MacOS", code as u32),
                    NativeKeyCode::Android(code) => ("Android", code),
                    NativeKeyCode::Xkb(code) => ("XKB", code),
                    NativeKeyCode::Unidentified => return warn!("Unidentified key event received"),
                };

                warn!("Unidentified {} key event {}", platform, code)
            }
        }
    }

    fn process_mouse_button_event(&mut self, button: MouseButton, state: ElementState) {
        match button {
            MouseButton::Other(code) => warn!(
                "Unidentified mouse button event received with code {}",
                code
            ),
            // Offsets into the key_states member
            _ => Self::update_key_state(
                &mut self.mouse_button_states,
                Self::mouse_button_to_index(button),
                state,
            ),
        };
    }

    const MOUSE_SENSITIVITY: f64 = 0.002;

    fn process_mouse_moved_window_event(&mut self, new_position: PhysicalPosition<f64>) {
        let new_vector_position = Vector2::new(new_position.x, new_position.y);

        if self.mouse_position.is_none() {
            self.mouse_position = Some(new_vector_position);
            return;
        }

        self.window_offset = Vector2::new(
            ((new_vector_position.x - self.mouse_position.unwrap().x) * Self::MOUSE_SENSITIVITY)
                as f32,
            ((new_vector_position.y - self.mouse_position.unwrap().y) * Self::MOUSE_SENSITIVITY)
                as f32,
        );

        self.mouse_position = Some(new_vector_position);
    }

    fn process_mouse_moved_device_event(&mut self, offset: (f64, f64)) {
        self.device_offset = Vector2::new(
            (offset.0 * Self::MOUSE_SENSITIVITY) as f32,
            (offset.1 * Self::MOUSE_SENSITIVITY) as f32,
        );
    }

    fn process_mouse_wheel_event(&mut self, y_offset: f32) {
        self.mouse_wheel_offset = y_offset;
    }

    fn update_key_state(key_states: &mut [KeyState], index: usize, state: ElementState) {
        let old_state = key_states[index];

        let new_state = match state {
            ElementState::Pressed => {
                if old_state == KeyState::Pressed || old_state == KeyState::Repeat {
                    KeyState::Repeat
                } else {
                    KeyState::Pressed
                }
            }
            ElementState::Released => {
                if old_state == KeyState::Pressed || old_state == KeyState::Repeat {
                    KeyState::JustReleased
                } else {
                    KeyState::Released
                }
            }
        };

        key_states[index] = new_state;
    }

    fn mouse_button_to_index(button: MouseButton) -> usize {
        match button {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
            MouseButton::Back => 3,
            MouseButton::Forward => 4,
            MouseButton::Other(code) => panic!(
                "Cannot query for unidentified mouse button with code {}",
                code
            ),
        }
    }
}
