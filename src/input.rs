use cgmath::{Vector2, Zero};
use log::{debug, error, warn};
use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{KeyCode, NativeKeyCode, PhysicalKey},
};
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;

const NUM_KEYS: usize = 194;
const NUM_MOUSE_BUTTONS: usize = 6;

pub struct Input {
    key_states: [KeyState; NUM_KEYS],
    mouse_button_states: [KeyState; NUM_MOUSE_BUTTONS],
    last_cursor_position: PhysicalPosition<f64>,
    window_offset: Vector2<f32>,
    device_offset: Vector2<f32>
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum KeyState {
    Released,
    Pressed,
    Repeat,
    JustReleased,
}

impl Input {
    pub fn new() -> Self {
        Self {
            key_states: [KeyState::Released; NUM_KEYS],
            mouse_button_states: [KeyState::Released; NUM_MOUSE_BUTTONS],
            last_cursor_position: PhysicalPosition::new(0.0, 0.0),
            window_offset: Vector2::zero(),
            device_offset: Vector2::zero()
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
        self.mouse_button_states[Self::mouse_button_to_index(mouse_button)] == KeyState::JustReleased
    }

    pub fn window_offset(&self) -> Vector2<f32> {
        self.window_offset
    }

    pub fn device_offset(&self) -> Vector2<f32> {
        self.device_offset
    }

    pub fn reset_internal_state(&mut self) {
        for key_state in self.key_states.iter_mut() {
            if *key_state == KeyState::JustReleased {
                *key_state = KeyState::Released;
            }
        }

        self.window_offset = Vector2::zero();
        self.device_offset = Vector2::zero();
    }

    pub fn process_key_event(&mut self, key_event: KeyEvent) {
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
                    NativeKeyCode::Unidentified => {
                        return warn!("Unidentified key event received")
                    }
                };

                warn!("Unidentified {} key event {}", platform, code)
            }
        }
    }

    pub fn process_mouse_button_event(&mut self, button: MouseButton, state: ElementState) {
        match button {
            MouseButton::Other(code) => warn!("Unidentified mouse button event received with code {}", code),
            // Offsets into the key_states member
            _ => Self::update_key_state(&mut self.mouse_button_states, Self::mouse_button_to_index(button), state)
        };
    }

    const CURSOR_SENSITIVITY: f64 = 0.002;

    pub fn process_cursor_moved_window_event(&mut self, position: PhysicalPosition<f64>) {

        self.window_offset = Vector2::new(
            ((position.x - self.last_cursor_position.x) * Self::CURSOR_SENSITIVITY) as f32,
            ((position.y - self.last_cursor_position.y) * Self::CURSOR_SENSITIVITY) as f32
        );

        self.last_cursor_position = position;
    }

    pub fn process_cursor_moved_device_event(&mut self, offset: (f64, f64)) {
        self.device_offset = Vector2::new(
            (offset.0 * Self::CURSOR_SENSITIVITY) as f32,
            (offset.1 * Self::CURSOR_SENSITIVITY) as f32
        );
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
            MouseButton::Other(code) => panic!("Cannot query for unidentified mouse button with code {}", code)
        }
    }
}
