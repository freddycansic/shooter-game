use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{KeyCode, NativeKeyCode, PhysicalKey},
};

const NUM_KEY_CODES: usize = 194;

pub struct Input {
    key_states: [KeyState; NUM_KEY_CODES],
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
            key_states: [KeyState::Released; NUM_KEY_CODES],
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

    pub fn reset_just_released(&mut self) {
        for key_state in self.key_states.iter_mut() {
            if *key_state == KeyState::JustReleased {
                *key_state = KeyState::Released;
            }
        }
    }

    pub fn process_key_event(&mut self, key_event: KeyEvent) {
        match key_event.physical_key {
            PhysicalKey::Code(key_code) => {
                let index = key_code as usize;
                let old_state = self.key_states[index];

                let state = match key_event.state {
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

                self.key_states[index] = state;
            }
            PhysicalKey::Unidentified(native_key_code) => {
                let (platform, code) = match native_key_code {
                    NativeKeyCode::Windows(code) => ("Windows", code as u32),
                    NativeKeyCode::MacOS(code) => ("MacOS", code as u32),
                    NativeKeyCode::Android(code) => ("Android", code),
                    NativeKeyCode::Xkb(code) => ("XKB", code),
                    NativeKeyCode::Unidentified => {
                        return log::warn!("Unidentified key event received")
                    }
                };

                log::warn!("Unidentified {} key event {}", platform, code)
            }
        }
    }
}
